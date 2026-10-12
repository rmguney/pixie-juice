// One test per bug found in review, so each stays fixed.

#include "fixtures.hpp"

#include <doctest/doctest.h>
#include <lcms2.h>
#include <lodepng.h>
#include <tiffio.h>

using namespace pixie;
using fixtures::as_text;
using fixtures::text_bytes;

namespace {

Bytes result_of(const Bytes& input, const Options& options = {}) {
  auto out = optimize(input, options);
  REQUIRE(out.has_value());
  return out->changed ? out->data : input;
}

std::size_t lines_starting(const Bytes& data, std::string_view prefix) {
  std::size_t n = 0;
  const auto text = as_text(data);
  for (std::size_t pos = 0; pos < text.size();) {
    if (text.substr(pos).starts_with(prefix)) ++n;
    const auto end = text.find('\n', pos);
    if (end == std::string_view::npos) break;
    pos = end + 1;
  }
  return n;
}

/// A JPEG with an APP1 EXIF block carrying `orientation`.
Bytes jpeg_with_orientation(const detail::Image& image, int orientation) {
  const auto plain = *detail::encode_jpeg(image, 90);
  const Bytes tiff = detail::minimal_exif(orientation);
  Bytes app1 = {0xFF, 0xE1, 0, 0, 'E', 'x', 'i', 'f', 0, 0};
  app1.insert(app1.end(), tiff.begin(), tiff.end());
  const auto length = app1.size() - 2;
  app1[2] = std::uint8_t(length >> 8);
  app1[3] = std::uint8_t(length);
  Bytes out(plain.begin(), plain.begin() + 2);
  out.insert(out.end(), app1.begin(), app1.end());
  out.insert(out.end(), plain.begin() + 2, plain.end());
  return out;
}

/// An RGB profile with Display P3 primaries, made with Little CMS.
Bytes display_p3_profile() {
  const cmsCIExyY white{0.3127, 0.3290, 1.0};
  const cmsCIExyYTRIPLE primaries{{0.680, 0.320, 1.0}, {0.265, 0.690, 1.0}, {0.150, 0.060, 1.0}};
  cmsToneCurve* gamma = cmsBuildGamma(nullptr, 2.2);
  cmsToneCurve* curves[3] = {gamma, gamma, gamma};
  cmsHPROFILE profile = cmsCreateRGBProfile(&white, &primaries, curves);
  cmsUInt32Number size = 0;
  cmsSaveProfileToMem(profile, nullptr, &size);
  Bytes out(size);
  cmsSaveProfileToMem(profile, out.data(), &size);
  cmsCloseProfile(profile);
  cmsFreeToneCurve(gamma);
  return out;
}

}  // namespace

TEST_CASE("OBJ point clouds pass through instead of being emptied") {
  const auto cloud = text_bytes("v 0 0 0\nv 1 0 0\nv 0 1 0\nv 1 1 1\n");
  const auto out = optimize(cloud, {});
  REQUIRE(out.has_value());
  CHECK_FALSE(out->changed);
}

TEST_CASE("OBJ vertex colours are kept, and a w weight passes through") {
  const auto coloured = text_bytes("v 0 0 0 1 0 0\nv 1 0 0 0 1 0\nv 0 1 0 0 0 1\nv 0 0 0 1 0 0\nf 1 2 3\nf 4 2 3\n");
  auto mesh = detail::read_obj(coloured, detail::MeshUse::Optimize);
  REQUIRE(mesh.has_value());
  REQUIRE(mesh->exact);
  const auto written = detail::write_obj(detail::optimize_mesh(mesh->mesh, 1.0f), "");
  CHECK(lines_starting(written, "v 0 0 0 1 0 0") == 1);
  CHECK(lines_starting(written, "v ") == 3);
  const auto weighted = text_bytes("v 0 0 0 1\nv 1 0 0 1\nv 0 1 0 1\nf 1 2 3\n");
  CHECK_FALSE(optimize(weighted, {})->changed);
}

TEST_CASE("OBJ faces before the first usemtl keep having no material") {
  std::string obj = "mtllib m.mtl\n";
  for (int i = 0; i < 6; ++i) obj += "v " + std::to_string(i) + ".000000 0.000000 " + std::to_string(i % 2) + ".000000\n";
  obj += "f 1 2 3\nusemtl red\nf 4 5 6\n";
  obj += "# padding so the rewrite is smaller ##############################################\n";
  const auto bytes = result_of(text_bytes(obj));
  const auto out = as_text(bytes);
  const auto first_face = out.find("\nf ");
  const auto first_material = out.find("usemtl red");
  REQUIRE(first_face != std::string_view::npos);
  REQUIRE(first_material != std::string_view::npos);
  CHECK(first_face < first_material);
  CHECK(fixtures::count(out, "usemtl") == 1);
}

TEST_CASE("PLY polygons with more than four corners are triangulated completely") {
  const auto ply = text_bytes(
      "ply\nformat ascii 1.0\ncomment a pentagon and a quad\nelement vertex 7\nproperty float x\nproperty float y\n"
      "property float z\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n"
      "0.000000 0.000000 0.000000\n1.000000 0.000000 0.000000\n1.500000 1.000000 0.000000\n"
      "0.500000 2.000000 0.000000\n-0.500000 1.000000 0.000000\n3.000000 0.000000 0.000000\n"
      "3.000000 1.000000 0.000000\n5 0 1 2 3 4\n4 1 5 6 2\n");
  detail::PlyInfo info;
  auto mesh = detail::read_ply(ply, detail::MeshUse::Optimize, &info);
  REQUIRE(mesh.has_value());
  REQUIRE(mesh->exact);
  CHECK(mesh->mesh.triangle_count() == 5);
  for (std::size_t t = 0; t < mesh->mesh.triangle_count(); ++t) {
    const auto* tri = &mesh->mesh.indices[t * 3];
    CHECK((tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2]));
  }
  CHECK(as_text(result_of(ply)).find("element face 5") != std::string_view::npos);
}

TEST_CASE("non-finite coordinates and coloured binary STL pass through") {
  const auto nan_stl = text_bytes(
      "solid s\nfacet normal 0 0 1\nouter loop\nvertex nan 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\n"
      "endsolid s\n");
  CHECK_FALSE(optimize(nan_stl, {})->changed);

  auto grid = detail::read_obj(text_bytes(fixtures::grid_obj(4)), detail::MeshUse::Optimize);
  auto stl = detail::write_stl(grid->mesh, {.binary = true});
  stl[84 + 48] = 0x1F;  // a facet colour in the attribute bytes
  CHECK_FALSE(optimize(stl, {.mesh_ratio = 0.5f})->changed);
}

TEST_CASE("conversions apply EXIF orientation") {
  const auto image = fixtures::make_image(64, 32, 4);
  const auto rotated = jpeg_with_orientation(image, 6);
  auto png = convert(rotated, Format::Png, {});
  REQUIRE(png.has_value());
  const auto decoded = detail::decode_rgba(png->data, Format::Png);
  CHECK(decoded->width == 32);
  CHECK(decoded->height == 64);
}

TEST_CASE("JPEG optimization keeps the orientation as minimal EXIF") {
  const auto rotated = jpeg_with_orientation(fixtures::make_image(64, 32, 4), 8);
  const auto out = result_of(rotated, {.quality = 60});
  CHECK(detail::read_colour_info(out, Format::Jpeg).orientation == 8);
}

TEST_CASE("conversions keep or apply the ICC profile") {
  const auto profile = display_p3_profile();
  detail::Image image;
  image.width = 16;
  image.height = 16;
  image.rgba.assign(16 * 16 * 4, 0);
  for (std::size_t i = 0; i < image.pixel_count(); ++i) {
    image.rgba[i * 4] = std::uint8_t(i % 2 ? 40 : 200);
    image.rgba[i * 4 + 1] = 180;
    image.rgba[i * 4 + 2] = 60;
    image.rgba[i * 4 + 3] = 255;
  }
  const auto source = *detail::encode_png(image, profile);
  CHECK(detail::read_colour_info(source, Format::Png).icc.size() == profile.size());

  // Targets with ICC support carry the profile...
  for (auto target : {Format::Jpeg, Format::WebP, Format::Tiff, Format::Png}) {
    CAPTURE(format_name(target));
    auto out = convert(source, target, {.quality = 95});
    REQUIRE(out.has_value());
    CHECK(detail::read_colour_info(out->data, target).icc.size() == profile.size());
  }
  // ...the others get pixels converted to sRGB: exactly what decode() shows for the source.
  auto gif = convert(source, Format::Gif, {});
  REQUIRE(gif.has_value());
  const auto shown = decode(source);
  const auto converted = detail::decode_rgba(gif->data, Format::Gif);
  CHECK(converted->rgba == shown->rgba);
  CHECK(converted->rgba != image.rgba);
}

TEST_CASE("GIF conversion reproduces up to 256 colours exactly, including white") {
  auto image = fixtures::make_flat_image(64, 64);
  for (std::size_t i = 0; i < 64; ++i) {
    image.rgba[i * 4] = image.rgba[i * 4 + 1] = image.rgba[i * 4 + 2] = 255;
  }
  const auto png = *detail::encode_png(image);
  auto gif = convert(png, Format::Gif, {});
  REQUIRE(gif.has_value());
  CHECK(detail::decode_rgba(gif->data, Format::Gif)->rgba == image.rgba);

  // Partial alpha above the threshold doesn't steal a palette entry for transparency.
  for (std::size_t i = 0; i < image.pixel_count(); ++i) image.rgba[i * 4 + 3] = 200;
  auto translucent = convert(*detail::encode_png(image), Format::Gif, {});
  const auto decoded = detail::decode_rgba(translucent->data, Format::Gif);
  CHECK_FALSE(decoded->has_alpha());
}

TEST_CASE("GIF conversion of a photo keeps colours close") {
  const auto photo = fixtures::make_image(96, 64, 30);
  auto gif = convert(*detail::encode_png(photo), Format::Gif, {});
  REQUIRE(gif.has_value());
  CHECK(detail::mean_error(photo, *detail::decode_rgba(gif->data, Format::Gif)) < 12.0);
}

TEST_CASE("TIFF orientation is kept and applied for display") {
  const auto image = fixtures::make_image(48, 24, 4);
  detail::TempFile file(".tif");
  TIFF* tif = TIFFOpen(file.path().c_str(), "w");
  REQUIRE(tif);
  TIFFSetField(tif, TIFFTAG_IMAGEWIDTH, 48);
  TIFFSetField(tif, TIFFTAG_IMAGELENGTH, 24);
  TIFFSetField(tif, TIFFTAG_BITSPERSAMPLE, 8);
  TIFFSetField(tif, TIFFTAG_SAMPLESPERPIXEL, 4);
  const std::uint16_t extra[] = {EXTRASAMPLE_UNASSALPHA};
  TIFFSetField(tif, TIFFTAG_EXTRASAMPLES, 1, extra);
  TIFFSetField(tif, TIFFTAG_PHOTOMETRIC, PHOTOMETRIC_RGB);
  TIFFSetField(tif, TIFFTAG_PLANARCONFIG, PLANARCONFIG_CONTIG);
  TIFFSetField(tif, TIFFTAG_ORIENTATION, ORIENTATION_RIGHTTOP);  // 6: rotate 90° clockwise to display
  for (std::uint32_t y = 0; y < 24; ++y) {
    REQUIRE(TIFFWriteScanline(tif, const_cast<std::uint8_t*>(&image.rgba[y * 48 * 4]), y, 0) == 1);
  }
  TIFFClose(tif);
  const auto input = *file.read();

  const auto shown = decode(input);
  CHECK(shown->width == 24);
  CHECK(shown->height == 48);
  // Display pixel (23, 0) is stored pixel (0, 0).
  CHECK(std::equal(image.rgba.begin(), image.rgba.begin() + 4, shown->rgba.begin() + 23 * 4));

  const auto out = result_of(input);
  CHECK(detail::read_colour_info(out, Format::Tiff).orientation == 6);
  CHECK(decode(out)->rgba == shown->rgba);
}

TEST_CASE("PNG optimization keeps EXIF orientation") {
  // An uncompressed PNG (so the optimizer wins) with an eXIf chunk after IHDR.
  const auto image = fixtures::make_image(40, 20, 6);
  lodepng::State state;
  state.encoder.zlibsettings.btype = 0;
  state.encoder.auto_convert = 0;
  Bytes png;
  REQUIRE(lodepng::encode(png, image.rgba, image.width, image.height, state) == 0);
  const Bytes exif = detail::minimal_exif(3);
  unsigned char* chunk = nullptr;
  std::size_t size = 0;
  REQUIRE(lodepng_chunk_create(&chunk, &size, unsigned(exif.size()), "eXIf", exif.data()) == 0);
  png.insert(png.begin() + 33, chunk, chunk + size);
  std::free(chunk);
  REQUIRE(detail::read_colour_info(png, Format::Png).orientation == 3);

  const auto out = result_of(png);
  CHECK(out.size() < png.size());
  CHECK(detail::read_colour_info(out, Format::Png).orientation == 3);
}

TEST_CASE("glTF detection finds the asset property after embedded buffers") {
  std::string json = R"({"accessors":[],"buffers":[{"byteLength":3,"uri":"data:application/octet-stream;base64,)";
  json += std::string(8000, 'A');
  json += R"("}],"asset":{"version":"2.0"}})";
  CHECK(detect(text_bytes(json)) == Format::Gltf);
}

TEST_CASE("glTF that references external files is refused instead of read from disk") {
  for (const char* json : {R"({"asset":{"version":"2.0"},"buffers":[{"byteLength":4,"uri":"secret.bin"}]})",
                           R"({"asset":{"version":"2.0"},"images":[{"uri":"../../photo.png"}]})"}) {
    const auto result = optimize(text_bytes(json), {});
    REQUIRE_FALSE(result.has_value());
    CHECK(result.error().message.find("external") != std::string::npos);
  }
}

TEST_CASE("temporary files have distinct names and are created up front") {
  const detail::TempFile a(".ply"), b(".ply");
  CHECK_FALSE(a.path().empty());
  CHECK(a.path() != b.path());
  CHECK(detail::TempFile(".glb").read().has_value());
}
