#include "internal.hpp"

#include <algorithm>
#include <cstdlib>

namespace pixie::detail {

bool Image::has_alpha() const {
  for (std::size_t i = 3; i < rgba.size(); i += 4) {
    if (rgba[i] != 255) return true;
  }
  return false;
}

bool Image::is_gray() const {
  for (std::size_t i = 0; i + 2 < rgba.size(); i += 4) {
    if (rgba[i] != rgba[i + 1] || rgba[i + 1] != rgba[i + 2]) return false;
  }
  return true;
}

double allowed_error(const Options& options) {
  return options.lossless ? 0.0 : 3.0 + (100 - options.quality) * 0.3;
}

double mean_error(const Image& a, const Image& b) {
  const std::size_t pixels = std::max<std::size_t>(a.pixel_count(), 1);
  std::uint64_t total = 0;
  for (std::size_t i = 0; i + 3 < a.rgba.size() && i + 3 < b.rgba.size(); i += 4) {
    const int colour = std::abs(a.rgba[i] - b.rgba[i]) + std::abs(a.rgba[i + 1] - b.rgba[i + 1]) +
                       std::abs(a.rgba[i + 2] - b.rgba[i + 2]);
    const int opacity = std::max(a.rgba[i + 3], b.rgba[i + 3]);
    // (colour / 3) weighted by opacity / 255, plus the alpha difference; integers throughout.
    total += std::uint64_t(colour) * opacity + 765ull * std::abs(a.rgba[i + 3] - b.rgba[i + 3]);
  }
  return double(total) / (765.0 * double(pixels));
}

bool is_faithful(const Reference& input, ByteView output, Format output_format, double budget) {
  auto decoded = decode_rgba(output, output_format);
  if (!decoded) return false;
  if (decoded->width != input.image.width || decoded->height != input.image.height) return false;
  if (input.alpha && !decoded->has_alpha()) return false;
  if (!input.gray && decoded->is_gray()) return false;
  return mean_error(input.image, *decoded) <= budget + 1e-9;
}

}  // namespace pixie::detail
