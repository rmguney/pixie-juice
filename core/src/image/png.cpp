// PNG through zopflipng (lodepng + zopfli). Always lossless: zopflipng picks
// the smallest colour type (palette, grayscale, with or without alpha) and
// filter strategy itself. Zopfli's slow, strong deflate is used for images up
// to 256K pixels; larger ones use lodepng's deflate so the browser tab stays
// responsive. Colour chunks are kept; an EXIF orientation survives as a
// minimal eXIf chunk.

#include "internal.hpp"

#include <lcms2.h>
#include <lodepng.h>
#include <zlib.h>
#include <zopflipng_lib.h>

#include <algorithm>
#include <array>
#include <cstdlib>
#include <cstring>

namespace pixie::detail {

namespace {

constexpr std::size_t kZopfliMaxPixels = 256 * 1024;
constexpr std::size_t kSignatureAndHeader = 8 + 25;  // PNG signature + IHDR chunk

std::uint32_t be32(const std::uint8_t* p) {
  return std::uint32_t(p[0]) << 24 | std::uint32_t(p[1]) << 16 | std::uint32_t(p[2]) << 8 | p[3];
}

std::optional<Bytes> run_zopflipng(ByteView input, bool use_zopfli, bool preserve_metadata) {
  ZopfliPNGOptions options;
  options.verbose = false;
  options.lossy_transparent = false;  // keep RGB under alpha = 0, so pixels stay exact
  options.lossy_8bit = false;
  options.auto_filter_strategy = true;
  options.use_zopfli = use_zopfli;
  options.num_iterations = 15;
  options.num_iterations_large = 5;
  // Chunks that change how the image looks always survive (colour profile, gamma,
  // primaries, and the HDR colour space and light levels).
  options.keepchunks = {"iCCP", "sRGB", "gAMA", "cHRM", "cICP", "mDCV", "cLLI"};
  if (preserve_metadata) {
    options.keepchunks.insert(options.keepchunks.end(), {"pHYs", "tEXt", "zTXt", "iTXt", "eXIf", "tIME"});
  }
  const Bytes original(input.begin(), input.end());
  Bytes result;
  if (ZopfliPNGOptimize(original, options, false, &result) != 0 || result.empty()) return std::nullopt;
  return result;
}

/// An acTL chunk before the first IDAT makes it an APNG.
bool png_is_animated(ByteView input) {
  std::size_t pos = 8;
  while (pos + 8 <= input.size()) {
    const std::uint32_t length = be32(input.data() + pos);
    const auto* type = input.data() + pos + 4;
    if (std::memcmp(type, "acTL", 4) == 0) return true;
    if (std::memcmp(type, "IDAT", 4) == 0) return false;
    if (length > input.size() - pos - 8 || input.size() - pos - 8 - length < 4) return false;
    pos += 12 + std::size_t(length);
  }
  return false;
}

/// A grey profile that renders grey pixels exactly as the RGB profile renders
/// R = G = B (its tone curve and the PCS white), or empty when no such profile
/// exists (unequal channel curves, or colorants that don't sum to white).
Bytes grey_equivalent(ByteView rgb_icc) {
  Bytes out;
  cmsHPROFILE rgb = cmsOpenProfileFromMem(rgb_icc.data(), cmsUInt32Number(rgb_icc.size()));
  if (!rgb) return out;
  const auto* curve = static_cast<const cmsToneCurve*>(cmsReadTag(rgb, cmsSigGreenTRCTag));
  cmsHPROFILE grey = curve ? cmsCreateGrayProfile(cmsD50_xyY(), curve) : nullptr;
  cmsHPROFILE lab = cmsCreateLab4Profile(nullptr);
  cmsHTRANSFORM from_rgb = grey ? cmsCreateTransform(rgb, TYPE_RGB_8, lab, TYPE_Lab_DBL, INTENT_RELATIVE_COLORIMETRIC, 0) : nullptr;
  cmsHTRANSFORM from_grey = grey ? cmsCreateTransform(grey, TYPE_GRAY_8, lab, TYPE_Lab_DBL, INTENT_RELATIVE_COLORIMETRIC, 0) : nullptr;
  if (from_rgb && from_grey) {
    double worst = 0;
    for (int v = 0; v < 256; ++v) {
      const std::uint8_t rgb_pixel[3] = {std::uint8_t(v), std::uint8_t(v), std::uint8_t(v)};
      const std::uint8_t grey_pixel = std::uint8_t(v);
      cmsCIELab a, b;
      cmsDoTransform(from_rgb, rgb_pixel, &a, 1);
      cmsDoTransform(from_grey, &grey_pixel, &b, 1);
      worst = std::max(worst, cmsDeltaE(&a, &b));
    }
    cmsUInt32Number size = 0;
    if (worst < 0.5 && cmsSaveProfileToMem(grey, nullptr, &size) && size > 0) {
      out.resize(size);
      if (!cmsSaveProfileToMem(grey, out.data(), &size)) out.clear();
    }
  }
  if (from_grey) cmsDeleteTransform(from_grey);
  if (from_rgb) cmsDeleteTransform(from_rgb);
  cmsCloseProfile(lab);
  if (grey) cmsCloseProfile(grey);
  cmsCloseProfile(rgb);
  return out;
}

/// Replaces the iCCP chunk's profile.
bool replace_icc(Bytes& png, ByteView icc) {
  std::size_t pos = 8;
  while (pos + 12 <= png.size()) {
    const std::size_t length = be32(png.data() + pos);
    if (length > png.size() - pos - 12) return false;
    if (std::memcmp(png.data() + pos + 4, "iCCP", 4) == 0) {
      unsigned char* compressed = nullptr;
      std::size_t compressed_size = 0;
      LodePNGCompressSettings settings;
      lodepng_compress_settings_init(&settings);
      if (lodepng_zlib_compress(&compressed, &compressed_size, icc.data(), icc.size(), &settings) != 0) return false;
      Bytes data{'i', 'c', 'c', 0, 0};  // profile name, NUL, compression method 0
      data.insert(data.end(), compressed, compressed + compressed_size);
      std::free(compressed);
      unsigned char* chunk = nullptr;
      std::size_t chunk_size = 0;
      if (lodepng_chunk_create(&chunk, &chunk_size, unsigned(data.size()), "iCCP", data.data()) != 0) return false;
      png.erase(png.begin() + std::ptrdiff_t(pos), png.begin() + std::ptrdiff_t(pos + 12 + length));
      png.insert(png.begin() + std::ptrdiff_t(pos), chunk, chunk + chunk_size);
      std::free(chunk);
      return true;
    }
    pos += 12 + length;
  }
  return false;
}

/// zopflipng copies the iCCP chunk without telling lodepng, so it may store an
/// RGB image as greyscale under its RGB profile (or the reverse), which PNG
/// forbids and viewers resolve by ignoring the profile. A grey image gets the
/// equivalent grey profile; false when no profile fits the colour type.
bool match_profile_to_colour_type(Bytes& png) {
  if (png.size() < kSignatureAndHeader) return false;
  const std::uint8_t colour_type = png[25];
  const bool grey = colour_type == 0 || colour_type == 4;
  const ColourInfo info = read_colour_info(png, Format::Png);
  const std::string_view space = icc_colour_space(info.icc);
  if (space == "GRAY") return grey;
  if (space != "RGB " || !grey) return true;
  const Bytes grey_icc = grey_equivalent(info.icc);
  return !grey_icc.empty() && replace_icc(png, grey_icc);
}

/// Inserts an eXIf chunk holding only the orientation right after IHDR.
void insert_orientation(Bytes& png, int orientation) {
  const Bytes exif = minimal_exif(orientation);
  unsigned char* chunk = nullptr;
  std::size_t size = 0;
  if (lodepng_chunk_create(&chunk, &size, unsigned(exif.size()), "eXIf", exif.data()) != 0) return;
  png.insert(png.begin() + kSignatureAndHeader, chunk, chunk + size);
  std::free(chunk);
}

}  // namespace

std::vector<Bytes> png_candidates(ByteView input, const Options& options) {
  std::vector<Bytes> candidates;
  // zopflipng would keep only the default frame of an animated PNG.
  if (png_is_animated(input)) return candidates;

  unsigned width = 0, height = 0;
  lodepng::State state;
  if (lodepng_inspect(&width, &height, &state, input.data(), input.size()) != 0) return candidates;
  const bool use_zopfli = std::size_t(width) * height <= kZopfliMaxPixels;

  if (auto result = run_zopflipng(input, use_zopfli, options.preserve_metadata);
      result && match_profile_to_colour_type(*result)) {
    if (!options.preserve_metadata) {
      if (const int orientation = read_colour_info(input, Format::Png).orientation; orientation != 1) {
        insert_orientation(*result, orientation);
      }
    }
    candidates.push_back(std::move(*result));
  }
  return candidates;
}

Result<Bytes> encode_png(const Image& image, ByteView icc) {
  // lodepng picks a colour type the profile allows (an RGB profile keeps RGB);
  // zopflipng then compresses harder but may pick one it doesn't, in which case
  // lodepng's file stands.
  lodepng::State state;
  state.info_raw.colortype = LCT_RGBA;
  if (!icc.empty() && lodepng_set_icc(&state.info_png, "icc", icc.data(), unsigned(icc.size())) != 0) {
    return fail("Invalid ICC profile");
  }
  Bytes encoded;
  if (const unsigned error = lodepng::encode(encoded, image.rgba, image.width, image.height, state)) {
    return fail(std::string("PNG encode failed: ") + lodepng_error_text(error));
  }
  auto optimized = run_zopflipng(encoded, image.pixel_count() <= kZopfliMaxPixels, false);
  if (optimized && optimized->size() < encoded.size() && match_profile_to_colour_type(*optimized)) {
    return std::move(*optimized);
  }
  return encoded;
}

namespace {

/// Inflates `parts` as one zlib stream into a scratch buffer, failing once the
/// output passes `limit`; corrupt data is left for lodepng to report.
bool inflates_within(const std::vector<ByteView>& parts, std::uint64_t limit) {
  z_stream z{};
  if (inflateInit(&z) != Z_OK) return false;
  std::array<unsigned char, 32768> scratch;
  std::uint64_t total = 0;
  int status = Z_OK;
  for (const ByteView part : parts) {
    z.next_in = const_cast<Bytef*>(part.data());
    z.avail_in = uInt(part.size());
    do {
      z.next_out = scratch.data();
      z.avail_out = uInt(scratch.size());
      status = inflate(&z, Z_NO_FLUSH);
      total += scratch.size() - z.avail_out;
      if (total > limit) {
        inflateEnd(&z);
        return false;
      }
    } while (status == Z_OK && (z.avail_in > 0 || z.avail_out == 0));
    if (status != Z_OK) break;
  }
  inflateEnd(&z);
  return true;
}

}  // namespace

bool png_within_limits(ByteView data) {
  // Text and profile chunks are inflated whole by lodepng; real ones are far below this.
  constexpr std::uint64_t kMaxAncillary = 16 * 1024 * 1024;
  if (data.size() < kSignatureAndHeader) return true;  // lodepng reports it
  const std::uint64_t width = be32(data.data() + 16), height = be32(data.data() + 20);
  const unsigned depth = data[24], colour_type = data[25];
  const unsigned channels = colour_type == 2 ? 3 : colour_type == 4 ? 2 : colour_type == 6 ? 4 : 1;
  const std::uint64_t bits = std::uint64_t(channels) * depth;
  auto scanlines = [&](std::uint64_t w, std::uint64_t h) { return w && h ? h * (1 + (w * bits + 7) / 8) : 0; };
  // The image data inflates to exactly its scanlines (Adam7: those of its 7 passes).
  std::uint64_t expected = scanlines(width, height);
  if (data[28] == 1) {
    static constexpr std::uint64_t ix[7] = {0, 4, 0, 2, 0, 1, 0}, iy[7] = {0, 0, 4, 0, 2, 0, 1};
    static constexpr std::uint64_t dx[7] = {8, 8, 4, 4, 2, 2, 1}, dy[7] = {8, 8, 8, 4, 4, 2, 2};
    expected = 0;
    for (int p = 0; p < 7; ++p) {
      expected += scanlines((width + dx[p] - ix[p] - 1) / dx[p], (height + dy[p] - iy[p] - 1) / dy[p]);
    }
  }
  std::vector<ByteView> image_data;
  std::size_t pos = 8;
  while (pos + 12 <= data.size()) {
    const std::size_t length = be32(data.data() + pos);
    if (length > data.size() - pos - 12) break;
    const auto* type = data.data() + pos + 4;
    const ByteView body = data.subspan(pos + 8, length);
    if (std::memcmp(type, "IDAT", 4) == 0) {
      image_data.push_back(body);
    } else if (std::memcmp(type, "iCCP", 4) == 0 || std::memcmp(type, "zTXt", 4) == 0 ||
               std::memcmp(type, "iTXt", 4) == 0) {
      // keyword NUL, then: iCCP/zTXt method byte; iTXt flag, method, language NUL, translation NUL.
      std::size_t at = std::find(body.begin(), body.end(), 0) - body.begin() + 1;
      bool compressed = true;
      if (std::memcmp(type, "iTXt", 4) == 0) {
        compressed = at < body.size() && body[at] == 1;
        at += 2;
        for (int field = 0; field < 2 && at < body.size(); ++field) {
          at = std::size_t(std::find(body.begin() + std::ptrdiff_t(at), body.end(), 0) - body.begin()) + 1;
        }
      } else {
        at += 1;
      }
      if (compressed && at < body.size() && !inflates_within({body.subspan(at)}, kMaxAncillary)) return false;
    }
    pos += 12 + length;
  }
  return inflates_within(image_data, expected);
}

}  // namespace pixie::detail
