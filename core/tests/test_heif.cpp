// HEIC through pixie::heif, on samples from libheif's own tests.

#include "fixtures.hpp"

#include "pixie/heif.hpp"

#include <doctest/doctest.h>

#include <cstdlib>
#include <fstream>
#include <iterator>
#include <string>

using namespace pixie;

namespace {

Bytes sample(const char* name) {
  std::ifstream in(std::string(PIXIE_HEIF_TEST_DATA) + "/" + name, std::ios::binary);
  REQUIRE(in.good());
  return Bytes(std::istreambuf_iterator<char>(in), std::istreambuf_iterator<char>());
}

/// More than a few pixels whose channels differ: not a blank or grey image.
bool colourful(const Pixels& pixels) {
  std::size_t count = 0;
  for (std::size_t i = 0; i < pixels.rgba.size(); i += 4) {
    const auto& p = pixels.rgba;
    if (std::abs(int(p[i]) - int(p[i + 1])) > 40 || std::abs(int(p[i + 1]) - int(p[i + 2])) > 40) ++count;
  }
  return count > pixels.rgba.size() / 4 / 10;
}

}  // namespace

TEST_CASE("HEIC: recognised by the core, decoded by pixie::heif, converted with convert_pixels") {
  const auto heic = sample("rainbow-451x461.heic");
  CHECK(detect(heic) == Format::Heic);
  const auto kept = optimize(heic, {});
  REQUIRE(kept.has_value());
  CHECK_FALSE(kept->changed);

  const auto decoded = heif::decode(heic);
  REQUIRE(decoded.has_value());
  // The coded size is 452 x 462; the file crops it to its display size.
  CHECK(decoded->pixels.width == 451);
  CHECK(decoded->pixels.height == 461);
  CHECK(decoded->pixels.rgba.size() == std::size_t(451) * 461 * 4);
  CHECK(colourful(decoded->pixels));

  const auto tiff = convert_pixels(decoded->pixels, decoded->icc, Format::Tiff, {.lossless = true});
  REQUIRE(tiff.has_value());
  const auto back = decode(tiff->data);
  REQUIRE(back.has_value());
  CHECK(back->rgba == to_srgb(decoded->pixels, decoded->icc).rgba);

  const auto jpeg = convert_pixels(decoded->pixels, decoded->icc, Format::Jpeg, {.quality = 80});
  REQUIRE(jpeg.has_value());
  CHECK(detect(jpeg->data) == Format::Jpeg);
}

TEST_CASE("HEIC: alpha stored as an auxiliary image comes out as straight alpha") {
  const auto decoded = heif::decode(sample("with-alpha-512x512.heic"));
  REQUIRE(decoded.has_value());
  CHECK(decoded->pixels.width == 512);
  const auto& rgba = decoded->pixels.rgba;
  std::size_t transparent = 0, opaque = 0;
  for (std::size_t i = 3; i < rgba.size(); i += 4) {
    transparent += rgba[i] < 128;
    opaque += rgba[i] == 255;
  }
  CHECK(transparent > 0);
  CHECK(opaque > 0);
  const auto tiff = convert_pixels(decoded->pixels, decoded->icc, Format::Tiff, {});
  REQUIRE(tiff.has_value());
  CHECK(detail::decode_rgba(tiff->data, Format::Tiff)->has_alpha());
}

TEST_CASE("HEIC: damaged files fail cleanly") {
  const auto heic = sample("rainbow-451x461.heic");
  for (const std::size_t keep : {std::size_t(12), std::size_t(64), heic.size() / 2, heic.size() - 100}) {
    CAPTURE(keep);
    const Bytes cut(heic.begin(), heic.begin() + std::ptrdiff_t(keep));
    (void)heif::decode(cut);  // an error or an image; never a crash
  }
  Bytes flipped = heic;
  for (std::size_t i = flipped.size() / 3; i < flipped.size(); i += 97) flipped[i] ^= 0x5A;
  (void)heif::decode(flipped);
  CHECK_FALSE(heif::decode(fixtures::text_bytes("not a heic file at all")).has_value());
}
