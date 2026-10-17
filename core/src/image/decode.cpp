// First-frame RGBA decoding for verification and conversion, each format
// through its own library: lodepng (PNG), mozjpeg (JPEG), libwebp (WebP),
// libtiff (TIFF), giflib (GIF), stb_image (BMP, TGA, and BMP icon entries).

#include "internal.hpp"

#include <lodepng.h>
#include <stb_image.h>
#include <webp/decode.h>
#include <webp/demux.h>

#include <algorithm>

namespace pixie::detail {

Result<Image> decode_png(ByteView data) {
  lodepng::State state;
  unsigned width = 0, height = 0;
  if (lodepng_inspect(&width, &height, &state, data.data(), data.size()) != 0) {
    return fail("Not a valid PNG");
  }
  if (!pixels_in_range(width, height)) return fail("PNG dimensions too large");
  if (!png_within_limits(data)) return fail("PNG data inflates beyond its declared size");
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
  // The animation decoder allocates its canvas on creation: check the size first.
  WebPBitstreamFeatures features;
  if (WebPGetFeatures(data.data(), data.size(), &features) != VP8_STATUS_OK) return fail("Not a valid WebP");
  if (!pixels_in_range(std::uint64_t(features.width), std::uint64_t(features.height))) {
    return fail("WebP dimensions too large");
  }
  if (!features.has_animation) {
    // A still image decodes straight into the result, without the animation decoder's canvases.
    Image image;
    image.width = std::uint32_t(features.width);
    image.height = std::uint32_t(features.height);
    image.rgba.resize(image.pixel_count() * 4);
    if (!WebPDecodeRGBAInto(data.data(), data.size(), image.rgba.data(), image.rgba.size(), int(image.width) * 4)) {
      return fail("WebP decode failed");
    }
    return image;
  }
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
  if (WebPAnimDecoderGetInfo(decoder, &info) && pixels_in_range(info.canvas_width, info.canvas_height) &&
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
  if (!pixels_in_range(std::uint64_t(width), std::uint64_t(height))) {
    return fail(std::string(what) + " dimensions too large");
  }
  // stb_image fills whatever the data doesn't cover, so a header alone could make it
  // allocate and fill 64 megapixels. Pixels stb reads take data: at least a bit
  // each in BMP, a 2-byte packet per 128 in run-length TGA.
  if (std::uint64_t(width) * std::uint64_t(height) > std::uint64_t(data.size()) * 128) {
    return fail(std::string(what) + " data too short for its dimensions");
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
    case Format::Gif: return decode_gif(data);
    case Format::Bmp: return decode_stb(data, "BMP");
    case Format::Tga: return decode_stb(data, "TGA");
    case Format::Ico: return decode_ico(data);
    case Format::Tiff: return decode_tiff(data);
    case Format::Heic: return fail("HEIC is decoded by pixie::heif::decode; convert its pixels with convert_pixels");
    case Format::Avif: return fail("AVIF is not decoded here; convert pixels decoded elsewhere with convert_pixels");
    default: return fail(std::string("Cannot decode ") + std::string(format_name(format)));
  }
}

}  // namespace pixie::detail
