// Encoders for conversions: BMP and TGA through stb_image_write, ICO as a
// PNG-in-ICO container (at most 256 px). Resizing, for ICO and for a size
// limit, through stb_image_resize2.

#include "internal.hpp"

#include <stb_image_resize2.h>
#include <stb_image_write.h>

#include <algorithm>

namespace pixie::detail {

namespace {

void append(void* context, void* data, int size) {
  auto* out = static_cast<Bytes*>(context);
  const auto* bytes = static_cast<const std::uint8_t*>(data);
  out->insert(out->end(), bytes, bytes + size);
}

/// Encodes through an stb writer: 32-bit with alpha, 24-bit when the image is opaque.
template <class Writer>
Result<Bytes> encode_stb(const Image& image, Writer writer, const char* format) {
  Bytes out;
  int ok = 0;
  if (image.has_alpha()) {
    ok = writer(append, &out, int(image.width), int(image.height), 4, image.rgba.data());
  } else {
    Bytes rgb(image.pixel_count() * 3);
    for (std::size_t i = 0, j = 0; i < image.rgba.size(); i += 4, j += 3) {
      rgb[j] = image.rgba[i];
      rgb[j + 1] = image.rgba[i + 1];
      rgb[j + 2] = image.rgba[i + 2];
    }
    ok = writer(append, &out, int(image.width), int(image.height), 3, rgb.data());
  }
  if (!ok) return fail(std::string(format) + " encode failed");
  return out;
}

}  // namespace

Result<Bytes> encode_bmp(const Image& image) { return encode_stb(image, stbi_write_bmp_to_func, "BMP"); }

/// Run-length encoded (stb_image_write's default).
Result<Bytes> encode_tga(const Image& image) { return encode_stb(image, stbi_write_tga_to_func, "TGA"); }

namespace {

/// The image scaled so its longest side is `longest` (smaller than it is), keeping the aspect ratio.
std::optional<Image> scaled(const Image& image, std::uint32_t longest) {
  const auto current = std::max(image.width, image.height);
  Image resized;
  resized.width = std::max<std::uint32_t>(1, std::uint32_t(std::uint64_t(image.width) * longest / current));
  resized.height = std::max<std::uint32_t>(1, std::uint32_t(std::uint64_t(image.height) * longest / current));
  resized.rgba.resize(resized.pixel_count() * 4);
  if (!stbir_resize_uint8_srgb(image.rgba.data(), int(image.width), int(image.height), 0, resized.rgba.data(),
                               int(resized.width), int(resized.height), 0, STBIR_RGBA)) {
    return std::nullopt;
  }
  return resized;
}

}  // namespace

bool fit_within(Image& image, std::uint32_t max_size) {
  if (max_size == 0 || std::max(image.width, image.height) <= max_size) return true;
  auto resized = scaled(image, max_size);
  if (!resized) return false;
  image = std::move(*resized);
  return true;
}

Result<Bytes> encode_ico(const Image& image) {
  // ICO entries are at most 256 x 256: larger images are scaled down.
  std::optional<Image> smaller;
  if (std::max(image.width, image.height) > 256 && !(smaller = scaled(image, 256))) return fail("ICO resize failed");
  const Image& entry = smaller ? *smaller : image;
  auto png = encode_png(entry);
  if (!png) return png;

  Bytes out;
  append_le16(out, 0);  // reserved
  append_le16(out, 1);  // type: icon
  append_le16(out, 1);  // one image
  out.push_back(std::uint8_t(entry.width == 256 ? 0 : entry.width));
  out.push_back(std::uint8_t(entry.height == 256 ? 0 : entry.height));
  out.push_back(0);      // palette size
  out.push_back(0);      // reserved
  append_le16(out, 1);   // colour planes
  append_le16(out, 32);  // bits per pixel
  append_le32(out, std::uint32_t(png->size()));
  append_le32(out, 6 + 16);  // data offset
  out.insert(out.end(), png->begin(), png->end());
  return out;
}

}  // namespace pixie::detail
