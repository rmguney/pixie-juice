// Hostile input: truncated and bit-flipped files of every format must come
// back as a result or an error, never a crash. Most useful under the
// native-asan preset.

#include "fixtures.hpp"

#include <doctest/doctest.h>

using namespace pixie;

namespace {

std::vector<Bytes> samples() {
  const auto image = fixtures::make_image(48, 32, 8, true);
  const auto opaque = fixtures::make_image(48, 32, 8);
  std::vector<Bytes> out = {
      *detail::encode_png(image),     *detail::encode_jpeg(opaque, 85), *detail::encode_webp(image, 80, false),
      *detail::encode_webp(image, 100, true), *detail::encode_gif(image), *detail::encode_bmp(image),
      *detail::encode_tga(image),     *detail::encode_tiff(image),      *detail::encode_ico(image),
      fixtures::text_bytes(fixtures::grid_obj(3)),
      fixtures::text_bytes("solid s\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"
                           "endloop\nendfacet\nendsolid s\n"),
      fixtures::text_bytes("ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\n"
                           "property float z\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n"
                           "0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n"),
      fixtures::grid_gltf(3),
  };
  const auto grid = detail::read_obj(fixtures::text_bytes(fixtures::grid_obj(3)), detail::MeshUse::Optimize);
  out.push_back(detail::write_stl(grid->mesh, {.binary = true}));
  out.push_back(detail::write_ply(grid->mesh, {.binary = true}));
  return out;
}

void exercise(const Bytes& data, bool optimizing = true) {
  // Either outcome is fine; reaching the next line without a crash is the test.
  if (optimizing) (void)optimize(data, {.quality = 50});
  (void)decode(data);
  (void)decode_mesh(data);
}

}  // namespace

TEST_CASE("truncated files never crash") {
  for (const auto& sample : samples()) {
    CAPTURE(format_name(detect(sample)));
    for (const std::size_t keep : {std::size_t(1), std::size_t(7), std::size_t(16), std::size_t(40), sample.size() / 3,
                                   sample.size() / 2, sample.size() - 1}) {
      if (keep >= sample.size()) continue;
      exercise(Bytes(sample.begin(), sample.begin() + std::ptrdiff_t(keep)));
    }
  }
}

TEST_CASE("corrupted files never crash") {
  std::uint32_t seed = 42;
  for (const auto& sample : samples()) {
    CAPTURE(format_name(detect(sample)));
    for (int round = 0; round < 64; ++round) {
      Bytes corrupted = sample;
      for (int flip = 0; flip < 1 + round % 4; ++flip) {
        seed = seed * 1664525u + 1013904223u;
        const std::size_t at = seed % corrupted.size();
        corrupted[at] = std::uint8_t(corrupted[at] ^ (1u << (seed >> 29)));
      }
      // Optimizing mostly re-encodes what still decodes; the fuzzer covers it further.
      exercise(corrupted, round % 8 == 0);
    }
  }
}

TEST_CASE("headers that claim huge sizes are rejected without allocating them") {
  // A PNG header for 1,000,000 x 1,000,000 pixels with no image data.
  Bytes png = *detail::encode_png(fixtures::make_image(4, 4, 0));
  const std::uint8_t huge[] = {0x00, 0x0F, 0x42, 0x40};  // 1,000,000 big-endian
  std::copy(std::begin(huge), std::end(huge), png.begin() + 16);
  std::copy(std::begin(huge), std::end(huge), png.begin() + 20);
  CHECK_FALSE(optimize(png, {}).has_value());

  // A BMP claiming 100,000 x 100,000.
  Bytes bmp = *detail::encode_bmp(fixtures::make_image(4, 4, 0));
  const std::uint32_t side = 100000;
  std::memcpy(bmp.data() + 18, &side, 4);
  std::memcpy(bmp.data() + 22, &side, 4);
  CHECK_FALSE(optimize(bmp, {}).has_value());
}

TEST_CASE("a few bytes of BMP or TGA can't claim an image they couldn't hold") {
  // Found by fuzzing: a 58-byte run-length TGA header for 16664 x 3839 pixels,
  // just under the size cap, which stb_image would allocate and fill.
  Bytes tga{0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0x41, 0xFF, 0x0E, 24, 0};
  tga.resize(58, 0);
  CHECK(detect(tga) == Format::Tga);
  const auto out = optimize(tga, {});
  REQUIRE_FALSE(out.has_value());
  CHECK(out.error().message.find("too short") != std::string::npos);

  // A BMP header for 4096 x 4096 with a few rows of data.
  Bytes bmp = *detail::encode_bmp(fixtures::make_image(4, 4, 0));
  const std::uint32_t side = 4096;
  std::memcpy(bmp.data() + 18, &side, 4);
  std::memcpy(bmp.data() + 22, &side, 4);
  CHECK_FALSE(decode(bmp).has_value());

  // Genuine run-length TGA of a single colour compresses well, but nowhere near that.
  detail::Image solid{512, 512, Bytes(512 * 512 * 4, 0)};
  for (std::size_t i = 0; i < solid.rgba.size(); i += 4) {
    solid.rgba[i] = 200;
    solid.rgba[i + 3] = 255;
  }
  const auto encoded = *detail::encode_tga(solid);
  CHECK(encoded.size() * 16 < solid.pixel_count());
  const auto decoded = decode(encoded);
  REQUIRE(decoded.has_value());
  CHECK(decoded->rgba == solid.rgba);
}

TEST_CASE("a JPEG that runs out of data is an error, not an image filled in") {
  // A small JPEG whose header claims 8000 x 8000 pixels: libjpeg would decode
  // 64 megapixels of nothing from a few hundred bytes.
  Bytes jpeg = *detail::encode_jpeg(fixtures::make_image(16, 16, 4), 80);
  bool patched = false;
  for (std::size_t i = 2; i + 9 < jpeg.size(); ++i) {
    if (jpeg[i] == 0xFF && (jpeg[i + 1] == 0xC0 || jpeg[i + 1] == 0xC2)) {
      jpeg[i + 5] = 0x1F;  // height 8000
      jpeg[i + 6] = 0x40;
      jpeg[i + 7] = 0x1F;  // width 8000
      jpeg[i + 8] = 0x40;
      patched = true;
      break;
    }
  }
  REQUIRE(patched);
  const auto out = optimize(jpeg, {});
  REQUIRE_FALSE(out.has_value());
  CHECK(out.error().message.find("remature end") != std::string::npos);

  // A truncated photo too.
  Bytes photo = *detail::encode_jpeg(fixtures::make_image(64, 64, 8), 90);
  photo.resize(photo.size() / 2);
  CHECK_FALSE(decode(photo).has_value());
}
