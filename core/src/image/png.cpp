// PNG through zopflipng (lodepng + zopfli). Always lossless: zopflipng picks
// the smallest colour type (palette, grayscale, with or without alpha) and
// filter strategy itself. Zopfli's slow, strong deflate is used for images up
// to 256K pixels; larger ones use lodepng's deflate so the browser tab stays
// responsive. Colour chunks are kept; an EXIF orientation survives as a
// minimal eXIf chunk.

#include "internal.hpp"

#include <lodepng.h>
#include <zopflipng_lib.h>

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
  // Chunks that change how the image looks always survive.
  options.keepchunks = {"iCCP", "sRGB", "gAMA", "cHRM"};
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

  if (auto result = run_zopflipng(input, use_zopfli, options.preserve_metadata)) {
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
  // A stored (uncompressed) intermediate: zopflipng re-encodes it anyway.
  lodepng::State state;
  state.encoder.zlibsettings.btype = 0;
  state.encoder.auto_convert = 0;
  state.info_raw.colortype = LCT_RGBA;
  state.info_png.color.colortype = LCT_RGBA;
  if (!icc.empty() && lodepng_set_icc(&state.info_png, "icc", icc.data(), unsigned(icc.size())) != 0) {
    return fail("Invalid ICC profile");
  }
  Bytes encoded;
  if (const unsigned error = lodepng::encode(encoded, image.rgba, image.width, image.height, state)) {
    return fail(std::string("PNG encode failed: ") + lodepng_error_text(error));
  }
  auto optimized = run_zopflipng(encoded, image.pixel_count() <= kZopfliMaxPixels, false);
  if (!optimized) return fail("PNG optimization failed");
  return std::move(*optimized);
}

}  // namespace pixie::detail
