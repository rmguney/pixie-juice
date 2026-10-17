#include "fixtures.hpp"

#include <doctest/doctest.h>

using namespace pixie;
using fixtures::text_bytes;

TEST_CASE("detects images from their signatures") {
  const auto image = fixtures::make_image(16, 16, 0);
  CHECK(detect(*detail::encode_png(image)) == Format::Png);
  CHECK(detect(*detail::encode_jpeg(image, 80)) == Format::Jpeg);
  CHECK(detect(*detail::encode_webp(image, 80, false)) == Format::WebP);
  CHECK(detect(*detail::encode_gif(image)) == Format::Gif);
  CHECK(detect(*detail::encode_bmp(image)) == Format::Bmp);
  CHECK(detect(*detail::encode_tga(image)) == Format::Tga);
  CHECK(detect(*detail::encode_ico(image)) == Format::Ico);
  CHECK(detect(*detail::encode_tiff(image)) == Format::Tiff);
  CHECK(detect(text_bytes("<?xml version=\"1.0\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>")) == Format::Svg);
}

TEST_CASE("detects HEIC and AVIF by their ftyp brands") {
  // An ISO base media file type box: size, "ftyp", major brand, minor version, compatible brands.
  auto ftyp = [](std::string_view major, std::string_view compatible) {
    const auto size = std::uint8_t(16 + compatible.size());
    Bytes box{0, 0, 0, size, 'f', 't', 'y', 'p'};
    box.insert(box.end(), major.begin(), major.end());
    box.insert(box.end(), 4, 0);
    box.insert(box.end(), compatible.begin(), compatible.end());
    box.insert(box.end(), {0, 0, 0, 8, 'm', 'e', 't', 'a'});
    return box;
  };
  CHECK(detect(ftyp("heic", "mif1heic")) == Format::Heic);
  CHECK(detect(ftyp("mif1", "mif1heic")) == Format::Heic);
  CHECK(detect(ftyp("avif", "mif1avif")) == Format::Avif);
  CHECK(detect(ftyp("mif1", "mif1avifmiaf")) == Format::Avif);
  CHECK(detect(ftyp("isom", "isomiso2mp41")) == Format::Unknown);  // MP4 video
  // The compact 'mif3' layout: the brand in the minor version's place, no compatible brands.
  Bytes mif3{0, 0, 0, 16, 'f', 't', 'y', 'p', 'm', 'i', 'f', '3', 'a', 'v', 'i', 'f', 0, 0, 0, 8, 'm', 'i', 'n', 'i'};
  CHECK(detect(mif3) == Format::Avif);
  CHECK(kind_of(Format::Heic) == Kind::Image);
  CHECK(format_from_name("heif") == Format::Heic);
  CHECK(format_name(Format::Avif) == "avif");

  // The core recognises them but has no codec for them: optimizing keeps the file.
  const auto out = optimize(ftyp("avif", "mif1avif"), {});
  REQUIRE(out.has_value());
  CHECK_FALSE(out->changed);
  CHECK(out->format == Format::Avif);
  CHECK_FALSE(convert(ftyp("heic", "mif1heic"), Format::Png, {}).has_value());
}

TEST_CASE("convert_pixels converts pixels decoded elsewhere, and checks them") {
  const auto image = fixtures::make_image(40, 30, 4, true);
  const Pixels pixels{image.width, image.height, image.rgba};
  const auto png = convert_pixels(pixels, {}, Format::Png, {.lossless = true});
  REQUIRE(png.has_value());
  CHECK(detect(png->data) == Format::Png);
  CHECK(decode(png->data)->rgba == image.rgba);
  CHECK_FALSE(convert_pixels(pixels, {}, Format::Jpeg, {.lossless = true}).has_value());
  CHECK_FALSE(convert_pixels(Pixels{40, 31, image.rgba}, {}, Format::Png, {}).has_value());
  CHECK_FALSE(convert_pixels(Pixels{0, 0, {}}, {}, Format::Png, {}).has_value());
  CHECK_FALSE(convert_pixels(pixels, {}, Format::Svg, {}).has_value());
  CHECK(to_srgb(pixels, {}).rgba == image.rgba);
}

TEST_CASE("detects meshes") {
  CHECK(detect(text_bytes("# cube\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n")) == Format::Obj);
  CHECK(detect(text_bytes("solid t\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\n")) == Format::Stl);
  CHECK(detect(text_bytes("ply\nformat ascii 1.0\nend_header\n")) == Format::Ply);
  CHECK(detect(text_bytes("{\"asset\":{\"version\":\"2.0\"}}")) == Format::Gltf);
  CHECK(detect(text_bytes("glTF\x02\0\0\0")) == Format::Glb);
  CHECK(detect(text_bytes("Kaydara FBX Binary  \0")) == Format::Fbx);

  // Binary STL is recognised by size, even when its header starts with "solid".
  Bytes stl(84 + 2 * 50, 0);
  std::memcpy(stl.data(), "solid binary", 12);
  stl[80] = 2;
  CHECK(detect(stl) == Format::Stl);
}

TEST_CASE("rejects unknown data") {
  CHECK(detect(text_bytes("hello world, nothing to see")) == Format::Unknown);
  CHECK(detect(Bytes{0x00, 0x01, 0x02}) == Format::Unknown);
  CHECK(!optimize(text_bytes("not a file format at all"), {}).has_value());
}

TEST_CASE("format names round-trip") {
  for (auto f : {Format::Png, Format::Jpeg, Format::WebP, Format::Gif, Format::Bmp, Format::Tga, Format::Tiff,
                 Format::Ico, Format::Svg, Format::Obj, Format::Stl, Format::Ply, Format::Gltf, Format::Glb,
                 Format::Fbx}) {
    CHECK(format_from_name(format_name(f)) == f);
  }
  CHECK(format_from_name("jpg") == Format::Jpeg);
}
