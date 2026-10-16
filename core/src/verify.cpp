#include "internal.hpp"

#include <algorithm>
#include <cstdlib>
#include <limits>
#include <vector>

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

namespace {

/// One pixel's error, scaled by 765: (colour difference / 3) weighted by
/// opacity / 255, plus the alpha difference; integers throughout.
std::uint64_t pixel_error(const std::uint8_t* a, const std::uint8_t* b) {
  const int colour = std::abs(a[0] - b[0]) + std::abs(a[1] - b[1]) + std::abs(a[2] - b[2]);
  const int opacity = std::max(a[3], b[3]);
  return std::uint64_t(colour) * opacity + 765ull * std::abs(a[3] - b[3]);
}

struct Errors {
  double mean = 0;
  double worst_block = 0;  // the mean over the worst 8x8 block
};

Errors errors(const Image& a, const Image& b) {
  if (a.width != b.width || a.height != b.height) {
    constexpr double kInfinite = std::numeric_limits<double>::infinity();
    return {kInfinite, kInfinite};
  }
  constexpr std::size_t kBlock = 8;
  const std::size_t width = a.width, height = a.height;
  std::vector<std::uint64_t> blocks((width + kBlock - 1) / kBlock);
  std::uint64_t total = 0;
  double worst = 0;
  for (std::size_t y = 0; y < height; ++y) {
    for (std::size_t x = 0; x < width; ++x) {
      const std::size_t i = (y * width + x) * 4;
      const std::uint64_t e = pixel_error(&a.rgba[i], &b.rgba[i]);
      total += e;
      blocks[x / kBlock] += e;
    }
    if (y % kBlock == kBlock - 1 || y + 1 == height) {  // a row of blocks is complete
      const std::size_t rows = y % kBlock + 1;
      for (std::size_t bx = 0; bx < blocks.size(); ++bx) {
        const std::size_t columns = std::min(kBlock, width - bx * kBlock);
        worst = std::max(worst, double(blocks[bx]) / (765.0 * double(rows * columns)));
        blocks[bx] = 0;
      }
    }
  }
  return {double(total) / (765.0 * double(std::max<std::size_t>(width * height, 1))), worst};
}

}  // namespace

double mean_error(const Image& a, const Image& b) { return errors(a, b).mean; }

bool is_faithful(const Reference& input, ByteView output, Format output_format, double budget) {
  auto decoded = decode_rgba(output, output_format);
  if (!decoded) return false;
  if (decoded->width != input.image.width || decoded->height != input.image.height) return false;
  if (input.alpha && !decoded->has_alpha()) return false;
  if (!input.gray && decoded->is_gray()) return false;
  // Within budget on average, and no region badly off: the mean alone would let
  // a few percent of the pixels be completely wrong.
  const Errors e = errors(input.image, *decoded);
  return e.mean <= budget + 1e-9 && e.worst_block <= 4 * budget + 1e-9;
}

}  // namespace pixie::detail
