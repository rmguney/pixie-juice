// libFuzzer entry point for the whole engine API. The first byte of an input
// picks what is done with the rest and with which options, so one corpus of
// files of every format covers optimizing, converting, decoding, the mesh
// reader and the HEIC decoder. Built by the "fuzz" preset; see fuzz/README.md.

#include "pixie/pixie.hpp"
#ifdef PIXIE_FUZZ_HEIF
#include "pixie/heif.hpp"
#endif

#include <array>
#include <cstdint>
#include <utility>

extern "C" int LLVMFuzzerTestOneInput(const std::uint8_t* data, std::size_t size) {
  if (size == 0) return 0;
  const std::uint8_t mode = data[0];
  const pixie::ByteView input(data + 1, size - 1);

  // Bits 0-2: what to do; 3-4: quality; 5: simplify meshes and scale images down;
  // 6: lossless; 7: keep metadata. A conversion's target comes from bits 3-5.
  pixie::Options options;
  options.quality = 20 + (mode >> 3 & 3) * 20;
  options.mesh_ratio = (mode & 0x20) != 0 ? 0.5f : 1.0f;
  options.max_size = (mode & 0x20) != 0 ? 16 : 0;
  options.lossless = (mode & 0x40) != 0;
  options.preserve_metadata = (mode & 0x80) != 0;

  static constexpr std::array kTargets{pixie::Format::Png, pixie::Format::Jpeg, pixie::Format::WebP, pixie::Format::Gif,
                                       pixie::Format::Bmp, pixie::Format::Tga,  pixie::Format::Ico,  pixie::Format::Tiff};
  switch (mode & 0x7) {
    case 0:
    case 1:
    case 2:
      (void)pixie::optimize(input, options);
      break;
    case 3:
      (void)pixie::convert(input, kTargets[mode >> 3 & 7], options);
      break;
    case 4:
      if (auto pixels = pixie::decode(input)) (void)pixie::to_srgb(std::move(*pixels), {});
      break;
    case 5:
      (void)pixie::decode_mesh(input);
      break;
    case 6:
#ifdef PIXIE_FUZZ_HEIF
      // Then a conversion that costs little (BMP), so the time goes to the decoder.
      if (auto decoded = pixie::heif::decode(input)) {
        (void)pixie::convert_pixels(decoded->pixels, decoded->icc, pixie::Format::Bmp, options);
      }
#endif
      break;
    case 7: {
      // Pixels from elsewhere: a width and height, then RGBA, converted with the rest as an ICC profile.
      if (input.size() < 4) break;
      pixie::Pixels pixels;
      pixels.width = std::uint32_t(input[0]) + 1;
      pixels.height = std::uint32_t(input[1]) + 1;
      const std::size_t length = std::size_t(pixels.width) * pixels.height * 4;
      const std::size_t profile = input[2] | std::size_t(input[3]) << 8;
      if (input.size() < 4 + length + profile) break;
      pixels.rgba.assign(input.begin() + 4, input.begin() + 4 + std::ptrdiff_t(length));
      const pixie::ByteView icc = input.subspan(4 + length, profile);
      (void)pixie::convert_pixels(pixels, icc, kTargets[mode >> 3 & 7], options);
      break;
    }
  }
  return 0;
}
