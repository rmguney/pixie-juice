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
