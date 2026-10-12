// First-frame RGBA decoding for verification and conversion, each format
// through its own library: lodepng (PNG), mozjpeg (JPEG), libwebp (WebP),
// libtiff (TIFF), stb_image (GIF, BMP, TGA, and BMP icon entries).

#include "internal.hpp"

#include <lodepng.h>
#include <stb_image.h>
#include <webp/demux.h>

#include <algorithm>
#include <cstring>

namespace pixie::detail {

namespace {

bool too_large(std::uint64_t width, std::uint64_t height) {
  return width == 0 || height == 0 || width * height > kMaxPixels;
}

}  // namespace

Result<Image> decode_png(ByteView data) {
  lodepng::State state;
  unsigned width = 0, height = 0;
  if (lodepng_inspect(&width, &height, &state, data.data(), data.size()) != 0) {
    return fail("Not a valid PNG");
  }
  if (too_large(width, height)) return fail("PNG dimensions too large");
  Image image;
  if (const unsigned error = lodepng::decode(image.rgba, width, height, data.data(), data.size(), LCT_RGBA, 8)) {
    return fail(std::string("PNG decode failed: ") + lodepng_error_text(error));
  }
  image.width = width;
  image.height = height;
  return image;
}

namespace {

Result<Image> decode_webp(ByteView data) {
  WebPAnimDecoderOptions options;
  WebPAnimDecoderOptionsInit(&options);
  options.color_mode = MODE_RGBA;
  options.use_threads = 0;
  const WebPData webp{data.data(), data.size()};
  WebPAnimDecoder* decoder = WebPAnimDecoderNew(&webp, &options);
  if (!decoder) return fail("WebP decode failed");
  WebPAnimInfo info;
  std::uint8_t* frame = nullptr;
  int timestamp = 0;
  Result<Image> result = fail("WebP decode failed");
  if (WebPAnimDecoderGetInfo(decoder, &info) && !too_large(info.canvas_width, info.canvas_height) &&
      WebPAnimDecoderGetNext(decoder, &frame, &timestamp)) {
    Image image;
    image.width = info.canvas_width;
    image.height = info.canvas_height;
    image.rgba.assign(frame, frame + image.pixel_count() * 4);
    result = std::move(image);
  }
  WebPAnimDecoderDelete(decoder);
  return result;
}

Result<Image> decode_stb(ByteView data, const char* what) {
  int width = 0, height = 0, channels = 0;
  const auto size = static_cast<int>(std::min<std::size_t>(data.size(), 0x7FFFFFFF));
  if (!stbi_info_from_memory(data.data(), size, &width, &height, &channels)) {
    return fail(std::string("Not a valid ") + what);
  }
  if (too_large(std::uint64_t(width), std::uint64_t(height))) {
    return fail(std::string(what) + " dimensions too large");
  }
  stbi_uc* pixels = stbi_load_from_memory(data.data(), size, &width, &height, &channels, 4);
  if (!pixels) return fail(std::string(what) + " decode failed: " + stbi_failure_reason());
  Image image;
  image.width = std::uint32_t(width);
  image.height = std::uint32_t(height);
  image.rgba.assign(pixels, pixels + image.pixel_count() * 4);
  stbi_image_free(pixels);
  return image;
}

}  // namespace

Result<Image> decode_rgba(ByteView data, Format format) {
  switch (format) {
    case Format::Png: return decode_png(data);
    case Format::Jpeg: return decode_jpeg(data);
    case Format::WebP: return decode_webp(data);
    case Format::Gif: return decode_stb(data, "GIF");
    case Format::Bmp: return decode_stb(data, "BMP");
    case Format::Tga: return decode_stb(data, "TGA");
    case Format::Ico: return decode_ico(data);
    case Format::Tiff: return decode_tiff(data);
    default: return fail(std::string("Cannot decode ") + std::string(format_name(format)));
  }
}

}  // namespace pixie::detail
