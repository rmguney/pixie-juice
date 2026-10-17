// Writes the fuzzer's seed corpus: small, valid files of every format the
// engine reads, each behind the mode byte that fuzz_pixie.cpp reads first.
//
//   pixie_fuzz_seeds <directory>

#include "../tests/fixtures.hpp"

#include <lcms2.h>

#include <cstdio>
#include <filesystem>
#include <fstream>
#include <initializer_list>
#include <iterator>
#include <string>

namespace {

namespace fs = std::filesystem;
using namespace pixie;

// Mode bytes (see fuzz_pixie.cpp): what to do in bits 0-2, options above.
constexpr std::uint8_t kOptimize = 2 << 3 | 0;            // quality 60
constexpr std::uint8_t kOptimizeLossless = 0x40 | kOptimize;
constexpr std::uint8_t kOptimizeKeepMetadata = 0x80 | kOptimize;
constexpr std::uint8_t kSimplify = 0x20 | kOptimize;
constexpr std::uint8_t kToPng = 0 << 3 | 3;
constexpr std::uint8_t kToJpeg = 1 << 3 | 3;
constexpr std::uint8_t kToGif = 3 << 3 | 3;
constexpr std::uint8_t kDecode = 4;
constexpr std::uint8_t kDecodeMesh = 5;
constexpr std::uint8_t kDecodeHeic = 6;
constexpr std::uint8_t kPixelsToPng = 0 << 3 | 7;
constexpr std::uint8_t kPixelsToJpeg = 1 << 3 | 7;
constexpr std::uint8_t kPixelsToTiff = 7 << 3 | 7;

void write(const fs::path& directory, const std::string& name, const Bytes& data,
           std::initializer_list<std::uint8_t> modes) {
  for (const std::uint8_t mode : modes) {
    std::ofstream out(directory / (name + "-" + std::to_string(mode)), std::ios::binary);
    out.put(char(mode));
    out.write(reinterpret_cast<const char*>(data.data()), std::streamsize(data.size()));
  }
}

/// Pixels from elsewhere, as mode 7 reads them: width - 1, height - 1, the profile's
/// length (16-bit little-endian), the RGBA, then the profile.
Bytes pixels_input(const detail::Image& image, const Bytes& icc) {
  Bytes out{std::uint8_t(image.width - 1), std::uint8_t(image.height - 1), std::uint8_t(icc.size()),
            std::uint8_t(icc.size() >> 8)};
  out.insert(out.end(), image.rgba.begin(), image.rgba.end());
  out.insert(out.end(), icc.begin(), icc.end());
  return out;
}

Bytes srgb_profile() {
  cmsHPROFILE profile = cmsCreate_sRGBProfile();
  cmsUInt32Number size = 0;
  cmsSaveProfileToMem(profile, nullptr, &size);
  Bytes out(size);
  cmsSaveProfileToMem(profile, out.data(), &size);
  cmsCloseProfile(profile);
  return out;
}

[[maybe_unused]] Bytes read(const fs::path& path) {
  std::ifstream in(path, std::ios::binary);
  return Bytes(std::istreambuf_iterator<char>(in), std::istreambuf_iterator<char>());
}

}  // namespace

int main(int argc, char** argv) {
  if (argc != 2) {
    std::fprintf(stderr, "usage: pixie_fuzz_seeds <directory>\n");
    return 2;
  }
  const fs::path directory = argv[1];
  fs::create_directories(directory);

  const auto opaque = fixtures::make_image(24, 16, 12);
  const auto alpha = fixtures::make_image(24, 16, 12, true);
  const auto flat = fixtures::make_flat_image(32, 32);
  const auto image_modes = {kOptimize, kOptimizeLossless, kOptimizeKeepMetadata, kToPng, kToJpeg, kToGif, kDecode};
  write(directory, "png", *detail::encode_png(alpha), image_modes);
  write(directory, "png-palette", *detail::encode_png(flat), image_modes);
  write(directory, "jpeg", *detail::encode_jpeg(opaque, 85), image_modes);
  write(directory, "webp", *detail::encode_webp(alpha, 80, false), image_modes);
  write(directory, "webp-lossless", *detail::encode_webp(flat, 100, true), image_modes);
  write(directory, "gif", *detail::encode_gif(flat), image_modes);
  write(directory, "bmp", *detail::encode_bmp(alpha), image_modes);
  write(directory, "tga", *detail::encode_tga(opaque), image_modes);
  write(directory, "ico", *detail::encode_ico(alpha), image_modes);
  write(directory, "tiff", *detail::encode_tiff(alpha), image_modes);
  const auto pixel_modes = {kPixelsToPng, kPixelsToJpeg, kPixelsToTiff};
  write(directory, "pixels", pixels_input(alpha, {}), pixel_modes);
  write(directory, "pixels-icc", pixels_input(opaque, srgb_profile()), pixel_modes);
  write(directory, "svg", fixtures::text_bytes("<svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"4\" height=\"4\"/></svg>"),
        {kOptimize});

  const auto mesh_modes = {kOptimize, kOptimizeLossless, kSimplify, kDecodeMesh};
  const auto obj = fixtures::text_bytes(fixtures::grid_obj(3));
  write(directory, "obj", obj, mesh_modes);
  const auto mesh = detail::read_obj(obj, detail::MeshUse::Optimize)->mesh;
  write(directory, "stl-ascii", detail::write_stl(mesh, {false, "grid"}), mesh_modes);
  write(directory, "stl-binary", detail::write_stl(mesh, {true, "grid"}), mesh_modes);
  detail::PlyInfo ply;
  write(directory, "ply-ascii", detail::write_ply(mesh, ply), mesh_modes);
  ply.binary = true;
  write(directory, "ply-binary", detail::write_ply(mesh, ply), mesh_modes);
  write(directory, "gltf", fixtures::grid_gltf(3), {kOptimize, kSimplify});
  // A GLB as gltfpack writes it, from a grid large enough to come out smaller.
  if (const auto glb = optimize(fixtures::grid_gltf(8), {}); glb && glb->changed) {
    write(directory, "glb", glb->data, {kOptimize, kSimplify});
  }

#ifdef PIXIE_HEIF_TEST_DATA
  for (const char* name : {"rainbow-451x461.heic", "with-alpha-512x512.heic"}) {
    write(directory, name, read(fs::path(PIXIE_HEIF_TEST_DATA) / name), {kDecodeHeic, kOptimize});
  }
#endif
  return 0;
}
