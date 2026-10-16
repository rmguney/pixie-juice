// One test per confirmed finding of the October 2026 code review (C: correctness,
// S: safety), so each stays fixed.

#include "fixtures.hpp"

#include <doctest/doctest.h>
#include <gif_lib.h>
#include <lcms2.h>
#include <lodepng.h>
#include <tiffio.h>

#include <algorithm>
#include <array>
#include <csetjmp>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <memory>

#include <jpeglib.h>
#include <zlib.h>

using namespace pixie;
using fixtures::as_text;
using fixtures::text_bytes;

namespace {

using Triangle = std::array<std::array<float, 3>, 3>;

/// The triangles of a mesh file as position triples, sorted, for comparison.
std::vector<Triangle> triangles_of(const Bytes& mesh) {
  const auto geometry = decode_mesh(mesh);
  REQUIRE(geometry.has_value());
  std::vector<Triangle> out(geometry->indices.size() / 3);
  for (std::size_t t = 0; t < out.size(); ++t) {
    for (std::size_t k = 0; k < 3; ++k) {
      const std::size_t v = geometry->indices[t * 3 + k];
      out[t][k] = {geometry->positions[v * 3], geometry->positions[v * 3 + 1], geometry->positions[v * 3 + 2]};
    }
    std::rotate(out[t].begin(), std::min_element(out[t].begin(), out[t].end()), out[t].end());
  }
  std::sort(out.begin(), out.end());
  return out;
}

/// After optimizing, the file still has exactly `expected`'s triangles.
void check_same_triangles(const Bytes& input, const Bytes& expected) {
  const auto out = optimize(input, {});
  REQUIRE(out.has_value());
  CHECK(triangles_of(out->changed ? out->data : input) == triangles_of(expected));
}

Bytes chunk(const char* type, const Bytes& data) {
  unsigned char* out = nullptr;
  std::size_t size = 0;
  REQUIRE(lodepng_chunk_create(&out, &size, unsigned(data.size()), type, data.data()) == 0);
  Bytes bytes(out, out + size);
  std::free(out);
  return bytes;
}

/// zlib data; `stored` skips compression, so an optimizer is sure to do better.
Bytes deflate(const Bytes& data, bool stored = false) {
  unsigned char* out = nullptr;
  std::size_t size = 0;
  LodePNGCompressSettings settings;
  lodepng_compress_settings_init(&settings);
  if (stored) settings.btype = 0;
  REQUIRE(lodepng_zlib_compress(&out, &size, data.data(), data.size(), &settings) == 0);
  Bytes bytes(out, out + size);
  std::free(out);
  return bytes;
}

/// A PNG made of the given chunks after an RGBA IHDR of `width` x `height`.
Bytes png_with(std::uint32_t width, std::uint32_t height, const std::vector<Bytes>& chunks) {
  Bytes png = {0x89, 'P', 'N', 'G', '\r', '\n', 0x1A, '\n'};
  const Bytes ihdr = {std::uint8_t(width >> 24), std::uint8_t(width >> 16), std::uint8_t(width >> 8), std::uint8_t(width),
                      std::uint8_t(height >> 24), std::uint8_t(height >> 16), std::uint8_t(height >> 8),
                      std::uint8_t(height), 8, 6, 0, 0, 0};
  const auto header = chunk("IHDR", ihdr);
  png.insert(png.end(), header.begin(), header.end());
  for (const auto& c : chunks) png.insert(png.end(), c.begin(), c.end());
  const auto end = chunk("IEND", {});
  png.insert(png.end(), end.begin(), end.end());
  return png;
}

Bytes profile_bytes(cmsHPROFILE profile) {
  cmsUInt32Number size = 0;
  cmsSaveProfileToMem(profile, nullptr, &size);
  Bytes out(size);
  cmsSaveProfileToMem(profile, out.data(), &size);
  cmsCloseProfile(profile);
  return out;
}

/// A JPEG made by `encode_jpeg` with an extra APPn segment right after SOI.
Bytes jpeg_with_segment(const detail::Image& image, std::uint8_t marker, const Bytes& payload) {
  const auto plain = *detail::encode_jpeg(image, 90);
  Bytes segment = {0xFF, marker, std::uint8_t((payload.size() + 2) >> 8), std::uint8_t(payload.size() + 2)};
  segment.insert(segment.end(), payload.begin(), payload.end());
  Bytes out(plain.begin(), plain.begin() + 2);
  out.insert(out.end(), segment.begin(), segment.end());
  out.insert(out.end(), plain.begin() + 2, plain.end());
  return out;
}

}  // namespace

TEST_CASE("C1: OBJ comments, continuations and index 0 on face lines don't shift triangles") {
  const std::string vertices = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nv 2 0 0\n";
  const auto clean = text_bytes(vertices + "f 1 2 3 4\nf 2 5 3\n");
  check_same_triangles(text_bytes(vertices + "f 1 2 3 4 # a quad\nf 2 5 3\n"), clean);
  check_same_triangles(text_bytes(vertices + "f 1 2 \\\n3 4\nf 2 5 3\n"), clean);

  const auto zero = text_bytes(vertices + "f 1 2 3 0\nf 2 5 3\n");
  const auto out = optimize(zero, {});
  REQUIRE(out.has_value());
  CHECK_FALSE(out->changed);
  CHECK_FALSE(decode_mesh(zero).has_value());
}

TEST_CASE("C2: OBJ statements separated by tabs keep their materials and objects") {
  std::string obj = "mtllib m.mtl\n";
  for (int i = 0; i < 6; ++i) obj += "v\t" + std::to_string(i) + ".000000 0.000000 " + std::to_string(i % 2) + ".000000\n";
  obj += "usemtl\tred\nf\t1 2 3\nf\t4 5 6\n# padding so the rewrite is smaller ##########################\n";
  const auto out = optimize(text_bytes(obj), {});
  REQUIRE(out.has_value());
  REQUIRE(out->changed);
  CHECK(as_text(out->data).find("usemtl red") != std::string_view::npos);

  const auto two_objects = text_bytes("o\ta\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\no\tb\nv 0 0 1\nf 1 2 4\n");
  CHECK_FALSE(optimize(two_objects, {})->changed);
}

TEST_CASE("C3: a greyscale PNG optimized from RGB gets a matching grey profile") {
  auto image = fixtures::make_image(64, 64, 0);
  for (std::size_t i = 0; i < image.rgba.size(); i += 4) image.rgba[i + 1] = image.rgba[i + 2] = image.rgba[i];
  const auto srgb = profile_bytes(cmsCreate_sRGBProfile());
  // An RGBA PNG (so zopflipng will choose greyscale) carrying an RGB profile.
  lodepng::State state;
  state.encoder.auto_convert = 0;
  state.info_raw.colortype = LCT_RGBA;
  state.info_png.color.colortype = LCT_RGBA;
  REQUIRE(lodepng_set_icc(&state.info_png, "icc", srgb.data(), unsigned(srgb.size())) == 0);
  Bytes input;
  REQUIRE(lodepng::encode(input, image.rgba, 64, 64, state) == 0);

  const auto out = optimize(input, {});
  REQUIRE(out.has_value());
  REQUIRE(out->changed);
  const bool grey = out->data[25] == 0 || out->data[25] == 4;
  const auto icc = detail::read_colour_info(out->data, Format::Png).icc;
  CHECK(detail::icc_colour_space(icc) == (grey ? "GRAY" : "RGB "));
}

TEST_CASE("C4: converting an image with a grey profile works and drops the profile") {
  auto image = fixtures::make_image(32, 32, 0);
  for (std::size_t i = 0; i < image.rgba.size(); i += 4) image.rgba[i + 1] = image.rgba[i + 2] = image.rgba[i];
  cmsToneCurve* gamma = cmsBuildGamma(nullptr, 1.8);
  const auto grey_profile = profile_bytes(cmsCreateGrayProfile(cmsD50_xyY(), gamma));
  cmsFreeToneCurve(gamma);
  const auto jpeg = *detail::encode_jpeg(image, 92, grey_profile);
  REQUIRE(detail::icc_colour_space(detail::read_colour_info(jpeg, Format::Jpeg).icc) == "GRAY");

  for (const auto target : {Format::Png, Format::WebP, Format::Tiff}) {
    const auto out = convert(jpeg, target, {});
    REQUIRE(out.has_value());
    CHECK(detail::read_colour_info(out->data, target).icc.empty());
  }
}

TEST_CASE("C9: glTF with extensions gltfpack can't carry over passes through") {
  const auto draco = text_bytes(
      R"({"asset":{"version":"2.0"},"extensionsUsed":["KHR_draco_mesh_compression"],)"
      R"("extensionsRequired":["KHR_draco_mesh_compression"]})");
  // A mesh gltfpack makes smaller, and the same mesh using an extension it would drop.
  const auto grid = fixtures::grid_gltf(16);
  const auto plain = optimize(grid, {});
  REQUIRE(plain.has_value());
  REQUIRE(plain->changed);
  auto unknown = grid;
  const std::string_view asset = R"({"asset":{"version":"2.0"},)";
  const std::string_view extension = R"("extensionsUsed":["EXT_lights_ies"],)";
  REQUIRE(as_text(unknown).starts_with(asset));
  unknown.insert(unknown.begin() + std::ptrdiff_t(asset.size()), extension.begin(), extension.end());
  for (const auto& input : {draco, unknown}) {
    const auto out = optimize(input, {});
    REQUIRE(out.has_value());
    CHECK_FALSE(out->changed);
  }
}

TEST_CASE("C10: one badly wrong block fails verification even when the mean is within budget") {
  const auto image = fixtures::make_image(64, 64, 8);
  auto damaged = image;
  for (std::uint32_t y = 16; y < 24; ++y) {
    for (std::uint32_t x = 16; x < 24; ++x) {
      for (int c = 0; c < 3; ++c) damaged.rgba[(y * 64 + x) * 4 + c] ^= 0xFF;
    }
  }
  const double budget = detail::allowed_error({.quality = 75});
  REQUIRE(detail::mean_error(image, damaged) < budget);
  const detail::Reference reference{image, false, false};
  CHECK_FALSE(detail::is_faithful(reference, *detail::encode_png(damaged), Format::Png, budget));
  CHECK(detail::is_faithful(reference, *detail::encode_png(image), Format::Png, budget));
}

TEST_CASE("C11: HDR colour chunks survive PNG optimization; JPEGs with more images pass through") {
  const auto image = fixtures::make_image(48, 48, 4);
  Bytes raw;
  for (std::uint32_t y = 0; y < 48; ++y) {
    raw.push_back(0);
    raw.insert(raw.end(), image.rgba.begin() + y * 48 * 4, image.rgba.begin() + (y + 1) * 48 * 4);
  }
  const Bytes cicp = {9, 16, 0, 1};  // BT.2100 primaries, PQ transfer, full range
  const auto png = png_with(48, 48, {chunk("cICP", cicp), chunk("IDAT", deflate(raw, true))});
  const auto out = optimize(png, {});
  REQUIRE(out.has_value());
  REQUIRE(out->changed);
  CHECK(as_text(out->data).find("cICP") != std::string_view::npos);

  const Bytes mpf = {'M', 'P', 'F', 0, 'M', 'M', 0, 42, 0, 0, 0, 8};
  CHECK_FALSE(optimize(jpeg_with_segment(fixtures::make_image(64, 64, 6), 0xE2, mpf), {.quality = 50})->changed);
}

TEST_CASE("C12: STL and PLY details survive or pass through") {
  const std::string facet = "facet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\n";
  const auto two_solids = text_bytes("solid a\n" + facet + "endsolid a\nsolid b\n" + facet + "endsolid b\n");
  CHECK_FALSE(optimize(two_solids, {})->changed);

  // A binary STL whose triangle count field was left at 0.
  const auto grid = detail::read_obj(text_bytes(fixtures::grid_obj(4)), detail::MeshUse::Optimize);
  auto stl = detail::write_stl(grid->mesh, {.binary = true});
  std::fill(stl.begin() + 80, stl.begin() + 84, std::uint8_t(0));
  CHECK(detect(stl) == Format::Stl);
  CHECK(triangles_of(stl).size() == 2 * 4 * 4);

  const auto ply = text_bytes(
      "ply\nformat ascii 1.0\ncomment TextureFile wood.png\nelement vertex 4\nproperty float x\nproperty float y\n"
      "property float z\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n"
      "0.000000 0.000000 0.000000\n1.000000 0.000000 0.000000\n1.000000 1.000000 0.000000\n0.000000 1.000000 0.000000\n"
      "3 0 1 2\n3 0 2 3\n");
  const auto out = optimize(ply, {});
  REQUIRE(out.has_value());
  REQUIRE(out->changed);
  CHECK(as_text(out->data).find("comment TextureFile wood.png") != std::string_view::npos);

  // A polyline has no surface: the preview shows only the face.
  const auto with_line = text_bytes("v 0 0 0\nv 1 0 0\nv 0 1 0\nv 1 1 0\nf 1 2 3\nl 1 2 4 3\n");
  CHECK(triangles_of(with_line).size() == 1);
}

TEST_CASE("C13: GIF areas outside the first frame are transparent; damaged TIFFs fail") {
  // A 4x4 screen with an opaque background colour; its only frame covers the top-left 2x2.
  Bytes gif;
  int error = 0;
  GifFileType* writer = EGifOpen(&gif, [](GifFileType* g, const GifByteType* data, int length) {
    static_cast<Bytes*>(g->UserData)->insert(static_cast<Bytes*>(g->UserData)->end(), data, data + length);
    return length;
  }, &error);
  REQUIRE(writer);
  const GifColorType colours[2] = {{255, 0, 0}, {0, 0, 255}};
  writer->SWidth = writer->SHeight = 4;
  writer->SColorResolution = 8;
  writer->SBackGroundColor = 1;
  writer->SColorMap = GifMakeMapObject(2, colours);
  SavedImage* frame = GifMakeSavedImage(writer, nullptr);
  frame->ImageDesc = GifImageDesc{0, 0, 2, 2, false, nullptr};
  frame->RasterBits = static_cast<GifByteType*>(std::calloc(4, 1));
  REQUIRE(EGifSpew(writer, &error) == GIF_OK);
  const auto shown = decode(gif);
  REQUIRE(shown.has_value());
  CHECK(shown->rgba[3] == 255);                  // inside the frame
  CHECK(shown->rgba[(3 * 4 + 3) * 4 + 3] == 0);  // outside it

  // Damaged compressed strip data: an error, not rows of garbage.
  auto tiff = convert(*detail::encode_png(fixtures::make_image(64, 64, 6)), Format::Tiff, {})->data;
  std::fill(tiff.begin() + 64, tiff.begin() + 256, std::uint8_t(0xFF));
  CHECK_FALSE(convert(tiff, Format::Png, {}).has_value());
}

TEST_CASE("C14: Lossless conversions refuse lossy targets") {
  const auto png = *detail::encode_png(fixtures::make_image(32, 32, 6));
  CHECK_FALSE(convert(png, Format::Jpeg, {.lossless = true}).has_value());
  CHECK(convert(png, Format::WebP, {.lossless = true}).has_value());
  CHECK(convert(*detail::encode_png(fixtures::make_flat_image(32, 32)), Format::Gif, {.lossless = true}).has_value());
}

TEST_CASE("S1: PLY vertex attributes declared as lists are refused") {
  const auto ply = text_bytes(
      "ply\nformat ascii 1.0\nelement vertex 3\nproperty list uchar float x\nproperty float y\nproperty float z\n"
      "element face 1\nproperty list uchar int vertex_indices\nend_header\n"
      "1 0 0 0\n1 1 0 0\n1 0 1 0\n3 0 1 2\n");
  CHECK_FALSE(optimize(ply, {}).has_value());
  CHECK_FALSE(decode_mesh(ply).has_value());
}

TEST_CASE("S2: declared sizes are checked before decoders allocate") {
  // WebP (VP8L) declaring 16384 x 16384.
  const Bytes webp = {'R', 'I', 'F', 'F', 18, 0, 0, 0, 'W', 'E', 'B', 'P', 'V', 'P', '8', 'L', 6, 0, 0, 0,
                      0x2F, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F};
  CHECK_FALSE(decode(webp).has_value());

  // GIF with a 65535 x 65535 frame on a 1 x 1 screen.
  const Bytes gif = {'G', 'I', 'F', '8', '9', 'a', 1, 0, 1, 0, 0, 0, 0,
                     0x2C, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 0, 0x3B};
  CHECK_FALSE(decode(gif).has_value());

  // PLY declaring 100 million vertices in a few bytes.
  const auto ply = text_bytes(
      "ply\nformat binary_little_endian 1.0\nelement vertex 100000000\nproperty float x\nproperty float y\n"
      "property float z\nend_header\n");
  CHECK_FALSE(optimize(ply, {}).has_value());

  // A 1 x 1 PNG whose image data inflates to 1 MB.
  const auto bomb = png_with(1, 1, {chunk("IDAT", deflate(Bytes(1024 * 1024, 0)))});
  CHECK_FALSE(decode(bomb).has_value());
}

TEST_CASE("S3: ICO entries may not overlap") {
  const auto png = *detail::encode_png(fixtures::make_flat_image(16, 16));
  Bytes ico;
  detail::append_le16(ico, 0);
  detail::append_le16(ico, 1);
  detail::append_le16(ico, 2);
  for (int i = 0; i < 2; ++i) {  // both entries point at the same bytes
    ico.insert(ico.end(), {16, 16, 0, 0});
    detail::append_le16(ico, 1);
    detail::append_le16(ico, 32);
    detail::append_le32(ico, std::uint32_t(png.size()));
    detail::append_le32(ico, 6 + 2 * 16);
  }
  ico.insert(ico.end(), png.begin(), png.end());
  CHECK_FALSE(decode(ico).has_value());
}

TEST_CASE("S6: a PLY face element without faces is read without extracting into nothing") {
  const auto ply = text_bytes(
      "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\n"
      "element face 0\nproperty list uchar int vertex_indices\nend_header\n0 0 0\n1 0 0\n0 1 0\n");
  const auto out = optimize(ply, {});
  REQUIRE(out.has_value());
  CHECK_FALSE(out->changed);
}

TEST_CASE("TIFF layouts that can't be decoded here pass through") {
  detail::TempFile file(".tif");
  TIFF* tif = TIFFOpen(file.path().c_str(), "w");
  REQUIRE(tif);
  TIFFSetField(tif, TIFFTAG_IMAGEWIDTH, 4);
  TIFFSetField(tif, TIFFTAG_IMAGELENGTH, 4);
  TIFFSetField(tif, TIFFTAG_BITSPERSAMPLE, 32);
  TIFFSetField(tif, TIFFTAG_SAMPLESPERPIXEL, 1);
  TIFFSetField(tif, TIFFTAG_SAMPLEFORMAT, SAMPLEFORMAT_IEEEFP);
  TIFFSetField(tif, TIFFTAG_PHOTOMETRIC, PHOTOMETRIC_MINISBLACK);
  TIFFSetField(tif, TIFFTAG_PLANARCONFIG, PLANARCONFIG_CONTIG);
  float row[4] = {0.0f, 0.25f, 0.5f, 1.0f};
  for (std::uint32_t y = 0; y < 4; ++y) REQUIRE(TIFFWriteScanline(tif, row, y, 0) == 1);
  TIFFClose(tif);
  const auto out = optimize(*file.read(), {});
  REQUIRE(out.has_value());
  CHECK_FALSE(out->changed);
}

namespace {

/// A mozjpeg compression that stops itself halfway through its passes.
struct StoppedCompression {
  jpeg_compress_struct cinfo{};
  jpeg_error_mgr errors{};
  jpeg_progress_mgr progress{};
  std::jmp_buf jump{};
  unsigned char* output = nullptr;
  unsigned long size = 0;
};

[[noreturn]] void jump_out(j_common_ptr cinfo) {
  std::longjmp(static_cast<StoppedCompression*>(cinfo->client_data)->jump, 1);
}

// By then the earlier scans' buffers and the current one's are allocated.
void stop_halfway(j_common_ptr cinfo) {
  const jpeg_progress_mgr* progress = cinfo->progress;
  if (progress->total_passes > 2 && progress->completed_passes * 2 >= progress->total_passes) jump_out(cinfo);
}

}  // namespace

// LeakSanitizer (the native-asan build on Linux) fails the run if the buffers leak.
TEST_CASE("S7: a compression stopped by an error frees its scan buffers") {
  auto job = std::make_unique<StoppedCompression>();
  jpeg_compress_struct& cinfo = job->cinfo;
  cinfo.err = jpeg_std_error(&job->errors);
  job->errors.error_exit = jump_out;
  jpeg_create_compress(&cinfo);
  cinfo.client_data = job.get();
  job->progress.progress_monitor = stop_halfway;
  cinfo.progress = &job->progress;
  jpeg_mem_dest(&cinfo, &job->output, &job->size);

  cinfo.image_width = 64;
  cinfo.image_height = 64;
  cinfo.input_components = 3;
  cinfo.in_color_space = JCS_RGB;
  jpeg_set_defaults(&cinfo);
  jpeg_c_set_bool_param(&cinfo, JBOOLEAN_OPTIMIZE_SCANS, TRUE);
  jpeg_simple_progression(&cinfo);

  std::array<JSAMPLE, 64 * 3> row{};
  bool stopped = true;
  if (setjmp(job->jump) == 0) {
    jpeg_start_compress(&cinfo, TRUE);
    for (JDIMENSION y = 0; y < cinfo.image_height; ++y) {
      for (std::size_t i = 0; i < row.size(); ++i) row[i] = JSAMPLE((i * 5 + y * 3) & 0xFF);
      JSAMPROW rows[] = {row.data()};
      jpeg_write_scanlines(&cinfo, rows, 1);
    }
    jpeg_finish_compress(&cinfo);
    stopped = false;
  }
  CHECK(stopped);
  jpeg_destroy_compress(&cinfo);
  std::free(job->output);
}

// Findings of the follow-up review.

TEST_CASE("Follow-up: a 32-bit icon with an all-zero alpha channel uses its AND mask") {
  Bytes dib(40, 0);
  auto put32 = [](Bytes& b, std::size_t at, std::uint32_t v) { std::memcpy(&b[at], &v, 4); };
  put32(dib, 0, 40);
  put32(dib, 4, 16);
  put32(dib, 8, 32);  // doubled height
  dib[12] = 1;
  dib[14] = 32;
  for (int i = 0; i < 16 * 16; ++i) dib.insert(dib.end(), {200, 100, 50, 0});  // BGRA with alpha unused
  Bytes mask(16 * 4, 0);
  mask[15 * 4] = 0x80;  // rows run bottom-up: the top-left pixel is transparent
  dib.insert(dib.end(), mask.begin(), mask.end());
  Bytes ico = {0, 0, 1, 0, 1, 0, 16, 16, 0, 0, 1, 0, 32, 0};
  const auto size = std::uint32_t(dib.size());
  ico.insert(ico.end(), {std::uint8_t(size), std::uint8_t(size >> 8), std::uint8_t(size >> 16), std::uint8_t(size >> 24)});
  ico.insert(ico.end(), {22, 0, 0, 0});
  ico.insert(ico.end(), dib.begin(), dib.end());
  const auto image = decode(ico);
  REQUIRE(image.has_value());
  CHECK(image->rgba[3] == 0);
  CHECK(image->rgba[4 + 3] == 255);
}

TEST_CASE("Follow-up: TIFF keeps straight alpha exactly, in grey and in colour") {
  for (const bool grey : {true, false}) {
    detail::Image image;
    image.width = image.height = 8;
    for (int i = 0; i < 64; ++i) {
      const auto v = std::uint8_t(i * 4);
      image.rgba.insert(image.rgba.end(), {v, grey ? v : std::uint8_t(255 - v), v, std::uint8_t(i % 2 ? 128 : 255)});
    }
    const auto png = detail::encode_png(image);
    REQUIRE(png.has_value());
    // Lossless conversion used to fail: decoding rounded the colour under partial alpha.
    const auto tiff = convert(*png, Format::Tiff, {.lossless = true});
    REQUIRE(tiff.has_value());
    const auto back = decode(tiff->data);
    REQUIRE(back.has_value());
    CHECK(back->rgba == image.rgba);
  }
}

TEST_CASE("Follow-up: OBJ numbers fast_obj can't read back pass through") {
  for (const std::string first : {"v 1e25 0 0\n", "v 1 inf 5\n", "v 2.5e-21 0 0\n"}) {
    std::string obj = first + "v 1.000000 0.000000 0.000000\nv 0.000000 1.000000 0.000000\nf 1 2 3\n";
    obj += "# padding, so a rewrite would be smaller #################################################\n";
    const auto out = optimize(text_bytes(obj), {});
    REQUIRE(out.has_value());
    CHECK_FALSE(out->changed);
  }
}

TEST_CASE("Follow-up: OBJ header statements written with a tab are kept") {
  std::string obj = "mtllib\tscene.mtl\nv 0.000000 0.000000 0.000000\nv 1.000000 0.000000 0.000000\n";
  obj += "v 0.000000 1.000000 0.000000\nusemtl red\nf 1 2 3\n";
  obj += "# padding, so the rewrite is smaller #####################################################\n";
  const auto out = optimize(text_bytes(obj), {});
  REQUIRE(out.has_value());
  REQUIRE(out->changed);
  CHECK(as_text(out->data).find("mtllib\tscene.mtl") != std::string_view::npos);
}

TEST_CASE("Follow-up: PNG text chunks are limited in total, not one by one") {
  const auto base = detail::encode_png(fixtures::make_image(8, 8, 1));
  REQUIRE(base.has_value());
  const std::string text(9 * 1024 * 1024, 'a');
  uLongf length = compressBound(uLong(text.size()));
  Bytes compressed(length);
  REQUIRE(compress2(compressed.data(), &length, reinterpret_cast<const Bytef*>(text.data()), uLong(text.size()), 9) == Z_OK);
  compressed.resize(length);
  Bytes body{'k', 0, 0};  // keyword, NUL, compression method
  body.insert(body.end(), compressed.begin(), compressed.end());
  auto with_chunks = [&](int count) {
    Bytes png = *base;
    for (int i = 0; i < count; ++i) {
      unsigned char* chunk = nullptr;
      std::size_t chunk_size = 0;
      REQUIRE(lodepng_chunk_create(&chunk, &chunk_size, unsigned(body.size()), "zTXt", body.data()) == 0);
      png.insert(png.begin() + 33, chunk, chunk + chunk_size);  // after IHDR
      std::free(chunk);
    }
    return png;
  };
  CHECK(decode(with_chunks(1)).has_value());        // 9 MiB of text
  CHECK_FALSE(decode(with_chunks(2)).has_value());  // 18 MiB in total
}

TEST_CASE("Follow-up: the first frame of an interlaced animated GIF decodes in row order") {
  Bytes gif;
  int error = 0;
  GifFileType* writer = EGifOpen(&gif, [](GifFileType* g, const GifByteType* data, int length) {
    static_cast<Bytes*>(g->UserData)->insert(static_cast<Bytes*>(g->UserData)->end(), data, data + length);
    return length;
  }, &error);
  REQUIRE(writer);
  const GifColorType colours[4] = {{0, 0, 0}, {255, 0, 0}, {0, 255, 0}, {0, 0, 255}};
  writer->SWidth = writer->SHeight = 16;
  writer->SColorResolution = 8;
  writer->SColorMap = GifMakeMapObject(4, colours);
  for (int f = 0; f < 2; ++f) {
    SavedImage* frame = GifMakeSavedImage(writer, nullptr);
    frame->ImageDesc = GifImageDesc{0, 0, 16, 16, true, nullptr};
    frame->RasterBits = static_cast<GifByteType*>(std::malloc(256));
    for (int i = 0; i < 256; ++i) frame->RasterBits[i] = GifByteType((i / 16 + f) % 4);
  }
  REQUIRE(EGifSpew(writer, &error) == GIF_OK);
  const auto shown = decode(gif);
  REQUIRE(shown.has_value());
  for (std::size_t y = 0; y < 16; ++y) {
    const GifColorType& expected = colours[y % 4];
    const auto* px = &shown->rgba[y * 16 * 4];
    CHECK(px[0] == expected.Red);
    CHECK(px[1] == expected.Green);
    CHECK(px[2] == expected.Blue);
  }
}
