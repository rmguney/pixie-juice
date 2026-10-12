// What an image needs beyond its pixels to display correctly: the ICC colour
// profile and the EXIF/TIFF orientation. Read with each format's library,
// applied with Little CMS (colour) and a pixel transpose/flip (orientation).

#include "internal.hpp"

#include <lcms2.h>
#include <lodepng.h>
#include <tiffio.h>
#include <webp/mux.h>

#include <algorithm>
#include <cstring>

namespace pixie::detail {

namespace {

using namespace std::string_view_literals;

bool starts_with(std::span<const std::uint8_t> data, std::string_view prefix) {
  return data.size() >= prefix.size() && std::memcmp(data.data(), prefix.data(), prefix.size()) == 0;
}

/// JPEG: APP2 "ICC_PROFILE" segments (concatenated in sequence order) and the APP1 EXIF orientation.
ColourInfo jpeg_info(ByteView data) {
  ColourInfo info;
  std::vector<std::pair<int, ByteView>> icc_chunks;
  std::size_t pos = 2;
  while (pos + 4 <= data.size() && data[pos] == 0xFF) {
    const int marker = data[pos + 1];
    if (marker == 0xFF) {  // fill byte
      ++pos;
      continue;
    }
    if (marker == 0xDA || marker == 0xD9) break;
    const std::size_t length = std::size_t(data[pos + 2]) << 8 | data[pos + 3];
    if (length < 2 || length > data.size() - pos - 2) break;
    const ByteView payload = data.subspan(pos + 4, length - 2);
    if (marker == 0xE2 && starts_with(payload, "ICC_PROFILE\0"sv) && payload.size() > 14) {
      icc_chunks.emplace_back(payload[12], payload.subspan(14));
    } else if (marker == 0xE1 && starts_with(payload, "Exif\0\0"sv)) {
      info.orientation = exif_orientation(payload.subspan(6));
    }
    pos += 2 + length;
  }
  std::sort(icc_chunks.begin(), icc_chunks.end(), [](const auto& a, const auto& b) { return a.first < b.first; });
  for (const auto& [sequence, chunk] : icc_chunks) info.icc.insert(info.icc.end(), chunk.begin(), chunk.end());
  return info;
}

ColourInfo png_info(ByteView data) {
  ColourInfo info;
  lodepng::State state;
  state.decoder.read_text_chunks = 0;
  unsigned width = 0, height = 0;
  // Inspecting chunks needs only the header; the profile is decompressed by lodepng.
  if (lodepng_inspect(&width, &height, &state, data.data(), data.size()) != 0) return info;
  std::size_t pos = 8;
  while (pos + 12 <= data.size()) {
    const std::size_t length = std::size_t(data[pos]) << 24 | std::size_t(data[pos + 1]) << 16 |
                               std::size_t(data[pos + 2]) << 8 | data[pos + 3];
    if (length > data.size() - pos - 12) break;
    const auto* chunk = data.data() + pos;
    if (std::memcmp(chunk + 4, "iCCP", 4) == 0) {
      if (lodepng_inspect_chunk(&state, pos, data.data(), data.size()) == 0 && state.info_png.iccp_defined) {
        info.icc.assign(state.info_png.iccp_profile, state.info_png.iccp_profile + state.info_png.iccp_profile_size);
      }
    } else if (std::memcmp(chunk + 4, "eXIf", 4) == 0) {
      info.orientation = exif_orientation(data.subspan(pos + 8, length));
    } else if (std::memcmp(chunk + 4, "IDAT", 4) == 0) {
      break;
    }
    pos += 12 + length;
  }
  return info;
}

ColourInfo webp_info(ByteView data) {
  ColourInfo info;
  const WebPData webp{data.data(), data.size()};
  WebPMux* mux = WebPMuxCreate(&webp, 0);
  if (!mux) return info;
  WebPData chunk;
  if (WebPMuxGetChunk(mux, "ICCP", &chunk) == WEBP_MUX_OK) info.icc.assign(chunk.bytes, chunk.bytes + chunk.size);
  if (WebPMuxGetChunk(mux, "EXIF", &chunk) == WEBP_MUX_OK) {
    info.orientation = exif_orientation(ByteView(chunk.bytes, chunk.size));
  }
  WebPMuxDelete(mux);
  return info;
}

}  // namespace

int exif_orientation(ByteView exif) {
  // Some writers keep the JPEG "Exif\0\0" prefix in PNG/WebP EXIF chunks.
  if (starts_with(exif, "Exif\0\0"sv)) exif = exif.subspan(6);
  if (exif.size() < 8) return 1;
  const bool little = exif[0] == 'I' && exif[1] == 'I';
  if (!little && !(exif[0] == 'M' && exif[1] == 'M')) return 1;
  auto u16 = [&](std::size_t at) -> unsigned {
    return little ? unsigned(exif[at] | exif[at + 1] << 8) : unsigned(exif[at] << 8 | exif[at + 1]);
  };
  auto u32 = [&](std::size_t at) -> std::uint32_t {
    return little ? std::uint32_t(u16(at) | u16(at + 2) << 16) : std::uint32_t(u16(at) << 16 | u16(at + 2));
  };
  const std::uint32_t ifd = u32(4);
  if (ifd > exif.size() || exif.size() - ifd < 2) return 1;
  const unsigned entries = u16(ifd);
  for (unsigned i = 0; i < entries; ++i) {
    const std::size_t entry = ifd + 2 + std::size_t(i) * 12;
    if (entry + 12 > exif.size()) break;
    if (u16(entry) == 0x0112) {
      const unsigned value = u16(entry + 8);
      return value >= 1 && value <= 8 ? int(value) : 1;
    }
  }
  return 1;
}

Bytes minimal_exif(int orientation) {
  return {'I', 'I', 42, 0, 8, 0, 0, 0,                                               // TIFF header, IFD0 at 8
          1, 0,                                                                      // one entry
          0x12, 0x01, 3, 0, 1, 0, 0, 0, std::uint8_t(orientation), 0, 0, 0,          // Orientation, SHORT
          0, 0, 0, 0};                                                               // no next IFD
}

ColourInfo read_colour_info(ByteView data, Format format) {
  switch (format) {
    case Format::Jpeg: return jpeg_info(data);
    case Format::Png: return png_info(data);
    case Format::WebP: return webp_info(data);
    case Format::Tiff: return tiff_colour_info(data);
    default: return {};
  }
}

void apply_orientation(Image& image, int orientation) {
  if (orientation <= 1 || orientation > 8) return;
  // EXIF/TIFF orientation 5-8 store the image transposed.
  const bool transpose = orientation >= 5;
  const std::uint32_t w = image.width, h = image.height;
  const std::uint32_t out_w = transpose ? h : w, out_h = transpose ? w : h;
  Bytes out(image.rgba.size());
  for (std::uint32_t y = 0; y < out_h; ++y) {
    for (std::uint32_t x = 0; x < out_w; ++x) {
      // Where display pixel (x, y) is stored.
      std::uint32_t sx = 0, sy = 0;
      switch (orientation) {
        case 2: sx = w - 1 - x; sy = y; break;
        case 3: sx = w - 1 - x; sy = h - 1 - y; break;
        case 4: sx = x; sy = h - 1 - y; break;
        case 5: sx = y; sy = x; break;
        case 6: sx = y; sy = h - 1 - x; break;
        case 7: sx = w - 1 - y; sy = h - 1 - x; break;
        case 8: sx = w - 1 - y; sy = x; break;
        default: break;
      }
      std::memcpy(&out[(std::size_t(y) * out_w + x) * 4], &image.rgba[(std::size_t(sy) * w + sx) * 4], 4);
    }
  }
  image.width = out_w;
  image.height = out_h;
  image.rgba = std::move(out);
}

bool convert_to_srgb(Image& image, ByteView icc) {
  if (icc.empty()) return true;
  cmsHPROFILE source = cmsOpenProfileFromMem(icc.data(), cmsUInt32Number(icc.size()));
  if (!source) return false;
  bool ok = false;
  if (cmsGetColorSpace(source) == cmsSigRgbData) {
    cmsHPROFILE srgb = cmsCreate_sRGBProfile();
    cmsHTRANSFORM transform = cmsCreateTransform(source, TYPE_RGBA_8, srgb, TYPE_RGBA_8, INTENT_PERCEPTUAL,
                                                 cmsFLAGS_COPY_ALPHA);
    if (transform) {
      cmsDoTransform(transform, image.rgba.data(), image.rgba.data(), cmsUInt32Number(image.pixel_count()));
      cmsDeleteTransform(transform);
      ok = true;
    }
    cmsCloseProfile(srgb);
  }
  cmsCloseProfile(source);
  return ok;
}

}  // namespace pixie::detail
