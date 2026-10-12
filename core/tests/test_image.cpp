#include "fixtures.hpp"

#include <doctest/doctest.h>
#include <gif_lib.h>
#include <lodepng.h>
#include <tiffio.h>

using namespace pixie;
using detail::decode_rgba;
using detail::Image;
using detail::mean_error;

namespace {

Image decoded(const Bytes& data) {
  auto image = decode_rgba(data, detect(data));
  REQUIRE(image.has_value());
  return *image;
}

/// The result file: the new data, or the input itself when nothing changed.
Output optimized(const Bytes& input, Options options = {}) {
  auto result = optimize(input, options);
  REQUIRE(result.has_value());
  if (!result->changed) {
    CHECK(result->data.empty());
    result->data = input;
  }
  return *result;
}

Bytes uncompressed_png(const Image& image) {
  lodepng::State state;
  state.encoder.zlibsettings.btype = 0;
  state.encoder.auto_convert = 0;
  state.info_png.color.colortype = LCT_RGBA;
  state.info_png.color.bitdepth = 8;
  std::vector<unsigned char> out;
  REQUIRE(lodepng::encode(out, image.rgba, image.width, image.height, state) == 0);
  return Bytes(out.begin(), out.end());
}

/// Inserts an acTL chunk after IHDR, turning the file into an APNG header.
Bytes with_actl(const Bytes& png) {
  unsigned char* chunks = nullptr;
  std::size_t size = 0;
  const unsigned char actl[8] = {0, 0, 0, 1, 0, 0, 0, 0};
  REQUIRE(lodepng_chunk_create(&chunks, &size, 8, "acTL", actl) == 0);
  Bytes out(png.begin(), png.begin() + 33);  // signature + IHDR
  out.insert(out.end(), chunks, chunks + size);
  out.insert(out.end(), png.begin() + 33, png.end());
  std::free(chunks);
  return out;
}

int write_gif(GifFileType* gif, const GifByteType* data, int length) {
  auto* out = static_cast<Bytes*>(gif->UserData);
  out->insert(out->end(), data, data + length);
  return length;
}

/// A looping three-frame 16x16 GIF with a comment extension.
Bytes animated_gif() {
  Bytes out;
  int error = 0;
  GifFileType* gif = EGifOpen(&out, write_gif, &error);
  REQUIRE(gif);
  gif->SWidth = 16;
  gif->SHeight = 16;
  gif->SColorResolution = 8;
  GifColorType colors[4] = {{0, 0, 0}, {255, 0, 0}, {0, 255, 0}, {0, 0, 255}};
  gif->SColorMap = GifMakeMapObject(4, colors);
  for (int f = 0; f < 3; ++f) {
    SavedImage* frame = GifMakeSavedImage(gif, nullptr);
    frame->ImageDesc = GifImageDesc{0, 0, 16, 16, false, nullptr};
    frame->RasterBits = static_cast<GifByteType*>(std::malloc(256));
    for (int i = 0; i < 256; ++i) frame->RasterBits[i] = GifByteType((i / 16 + f) % 4);
    if (f == 0) {
      unsigned char netscape[] = "NETSCAPE2.0";
      unsigned char loop[] = {1, 0, 0};
      GifAddExtensionBlock(&frame->ExtensionBlockCount, &frame->ExtensionBlocks, APPLICATION_EXT_FUNC_CODE, 11,
                           netscape);
      GifAddExtensionBlock(&frame->ExtensionBlockCount, &frame->ExtensionBlocks, CONTINUE_EXT_FUNC_CODE, 3, loop);
      unsigned char comment[] = "a comment that the optimizer should drop";
      GifAddExtensionBlock(&frame->ExtensionBlockCount, &frame->ExtensionBlocks, COMMENT_EXT_FUNC_CODE,
                           sizeof comment - 1, comment);
    }
    GraphicsControlBlock control{DISPOSAL_UNSPECIFIED, false, 7, NO_TRANSPARENT_COLOR};
    EGifGCBToSavedExtension(&control, gif, f);
  }
  REQUIRE(EGifSpew(gif, &error) == GIF_OK);
  return out;
}

int gif_frames(const Bytes& data, int* delay) {
  struct Reader {
    const Bytes* data;
    std::size_t pos;
  } reader{&data, 0};
  auto read = [](GifFileType* g, GifByteType* buf, int n) {
    auto* r = static_cast<Reader*>(g->UserData);
    const auto k = std::min<std::size_t>(std::size_t(n), r->data->size() - r->pos);
    std::memcpy(buf, r->data->data() + r->pos, k);
    r->pos += k;
    return int(k);
  };
  int error = 0;
  GifFileType* gif = DGifOpen(&reader, read, &error);
  REQUIRE(gif);
  REQUIRE(DGifSlurp(gif) == GIF_OK);
  GraphicsControlBlock control{};
  DGifSavedExtensionToGCB(gif, 0, &control);
  *delay = control.DelayTime;
  const int frames = gif->ImageCount;
  DGifCloseFile(gif, &error);
  return frames;
}

Bytes uncompressed_tiff(const Image& image) {
  detail::TempFile file(".tif");
  TIFF* tif = TIFFOpen(file.path().c_str(), "w");
  REQUIRE(tif);
  TIFFSetField(tif, TIFFTAG_IMAGEWIDTH, image.width);
  TIFFSetField(tif, TIFFTAG_IMAGELENGTH, image.height);
  TIFFSetField(tif, TIFFTAG_BITSPERSAMPLE, 8);
  TIFFSetField(tif, TIFFTAG_SAMPLESPERPIXEL, 3);
  TIFFSetField(tif, TIFFTAG_PHOTOMETRIC, PHOTOMETRIC_RGB);
  TIFFSetField(tif, TIFFTAG_PLANARCONFIG, PLANARCONFIG_CONTIG);
  TIFFSetField(tif, TIFFTAG_COMPRESSION, COMPRESSION_NONE);
  Bytes row(image.width * 3);
  for (std::uint32_t y = 0; y < image.height; ++y) {
    for (std::uint32_t x = 0; x < image.width; ++x) {
      std::memcpy(&row[x * 3], &image.rgba[(y * image.width + x) * 4], 3);
    }
    REQUIRE(TIFFWriteScanline(tif, row.data(), y, 0) == 1);
  }
  TIFFClose(tif);
  auto bytes = file.read();
  REQUIRE(bytes.has_value());
  return *bytes;
}

/// A 16x16 icon whose single entry is a 32-bit DIB (BITMAPINFOHEADER + BGRA + AND mask).
Bytes dib_icon(const Image& image) {
  REQUIRE(image.width == 16);
  Bytes dib(40, 0);
  auto put32 = [](Bytes& b, std::size_t at, std::uint32_t v) { std::memcpy(&b[at], &v, 4); };
  put32(dib, 0, 40);
  put32(dib, 4, 16);
  put32(dib, 8, 32);  // doubled height
  dib[12] = 1;
  dib[14] = 32;
  for (int y = 15; y >= 0; --y) {
    for (int x = 0; x < 16; ++x) {
      const auto* px = &image.rgba[std::size_t(y * 16 + x) * 4];
      dib.insert(dib.end(), {px[2], px[1], px[0], px[3]});
    }
  }
  dib.insert(dib.end(), 16 * 4, 0);  // AND mask, all visible
  Bytes ico = {0, 0, 1, 0, 1, 0, 16, 16, 0, 0, 1, 0, 32, 0};
  const auto size = std::uint32_t(dib.size());
  ico.insert(ico.end(), {std::uint8_t(size), std::uint8_t(size >> 8), std::uint8_t(size >> 16), std::uint8_t(size >> 24)});
  ico.insert(ico.end(), {22, 0, 0, 0});
  ico.insert(ico.end(), dib.begin(), dib.end());
  return ico;
}

}  // namespace

TEST_CASE("PNG optimization is lossless and smaller") {
  const auto image = fixtures::make_image(96, 64, 6);
  const auto input = uncompressed_png(image);
  const auto out = optimized(input);
  CHECK(out.changed);
  CHECK(out.format == Format::Png);
  CHECK(out.data.size() < input.size());
  CHECK(decoded(out.data).rgba == image.rgba);
}

TEST_CASE("PNG keeps transparency exactly and uses a palette for flat images") {
  const auto with_alpha = fixtures::make_image(64, 64, 4, true);
  const auto out = optimized(uncompressed_png(with_alpha));
  CHECK(decoded(out.data).rgba == with_alpha.rgba);

  const auto flat = fixtures::make_flat_image(128, 128);
  const auto input = uncompressed_png(flat);
  const auto palette = optimized(input);
  CHECK(palette.data.size() * 20 < input.size());
  CHECK(decoded(palette.data).rgba == flat.rgba);
}

TEST_CASE("animated PNG is returned unchanged") {
  const auto input = with_actl(uncompressed_png(fixtures::make_image(32, 32, 4)));
  const auto out = optimized(input);
  CHECK_FALSE(out.changed);
  CHECK(out.data == input);
}

TEST_CASE("JPEG re-encode keeps colour within the quality budget") {
  const auto image = fixtures::make_image(160, 120, 12);
  const auto input = *detail::encode_jpeg(image, 95);
  for (int quality : {10, 50, 75, 90}) {
    CAPTURE(quality);
    const auto out = optimized(input, {.quality = quality});
    CHECK(out.format == Format::Jpeg);
    CHECK(out.data.size() <= input.size());
    const auto result = decoded(out.data);
    CHECK_FALSE(result.is_gray());
    CHECK(mean_error(decoded(input), result) <= detail::allowed_error({.quality = quality}));
  }
}

TEST_CASE("JPEG lossless mode never changes pixels") {
  const auto input = *detail::encode_jpeg(fixtures::make_image(120, 80, 8), 92);
  const auto out = optimized(input, {.quality = 10, .lossless = true});
  CHECK(decoded(out.data).rgba == decoded(input).rgba);
}

TEST_CASE("JPEG metadata: comments and XMP go, orientation and ICC stay") {
  const auto plain = *detail::encode_jpeg(fixtures::make_image(64, 64, 8), 90);
  // APP1 EXIF (orientation 6, plus a GPS-like padding), APP1 XMP, APP2 ICC, COM.
  const Bytes exif = {0xFF, 0xE1, 0, 46, 'E', 'x', 'i', 'f', 0, 0, 'I', 'I', 42, 0, 8, 0, 0, 0, 1, 0,
                      0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
                      'G', 'P', 'S', 'G', 'P', 'S', 'G', 'P', 'S', 'G', 'P', 'S'};
  const Bytes xmp = {0xFF, 0xE1, 0, 13, 'h', 't', 't', 'p', ':', '/', '/', 'n', 's', '.', 'a'};
  const Bytes icc = {0xFF, 0xE2, 0, 16, 'I', 'C', 'C', '_', 'P', 'R', 'O', 'F', 'I', 'L', 'E', 0, 1, 1};
  const Bytes comment = {0xFF, 0xFE, 0, 7, 'h', 'e', 'l', 'l', 'o'};
  Bytes input(plain.begin(), plain.begin() + 2);
  for (const auto* seg : {&exif, &xmp, &icc, &comment}) input.insert(input.end(), seg->begin(), seg->end());
  input.insert(input.end(), plain.begin() + 2, plain.end());

  const auto out = optimized(input, {.quality = 95, .lossless = true});
  const auto text = fixtures::as_text(out.data);
  CHECK(text.find("hello") == std::string_view::npos);
  CHECK(text.find("http://ns") == std::string_view::npos);
  CHECK(text.find("GPSGPS") == std::string_view::npos);
  CHECK(text.find("ICC_PROFILE") != std::string_view::npos);
  CHECK(text.find(std::string_view("\x12\x01\x03\x00\x01\x00\x00\x00\x06", 9)) != std::string_view::npos);

  const auto kept = optimized(input, {.quality = 95, .lossless = true, .preserve_metadata = true});
  CHECK(fixtures::as_text(kept.data).find("hello") != std::string_view::npos);
}

TEST_CASE("WebP: lossy re-encode within budget, lossless sources stay exact") {
  const auto image = fixtures::make_image(128, 96, 10);
  const auto lossy = *detail::encode_webp(image, 95, false);
  const auto out = optimized(lossy, {.quality = 60});
  CHECK(out.format == Format::WebP);
  CHECK(out.data.size() < lossy.size());
  CHECK(mean_error(decoded(lossy), decoded(out.data)) <= detail::allowed_error({.quality = 60}));

  const auto exact = *detail::encode_webp(fixtures::make_image(64, 64, 4, true), 100, true);
  const auto lossless_out = optimized(exact, {.lossless = true});
  CHECK(decoded(lossless_out.data).rgba == decoded(exact).rgba);
}

TEST_CASE("GIF keeps every frame, timing and loop; drops comments") {
  const auto input = animated_gif();
  int delay = 0;
  REQUIRE(gif_frames(input, &delay) == 3);
  REQUIRE(delay == 7);
  REQUIRE(fixtures::as_text(input).find("NETSCAPE2.0") != std::string_view::npos);
  const auto out = optimized(input);
  delay = 0;
  CHECK(gif_frames(out.data, &delay) == 3);
  CHECK(delay == 7);
  CHECK(fixtures::as_text(out.data).find("NETSCAPE2.0") != std::string_view::npos);
  CHECK(fixtures::as_text(out.data).find("a comment") == std::string_view::npos);
  CHECK(decoded(out.data).rgba == decoded(input).rgba);
}

TEST_CASE("BMP and TGA become lossless PNG") {
  const auto image = fixtures::make_image(64, 48, 6);
  for (const auto& input : {*detail::encode_bmp(image), *detail::encode_tga(image)}) {
    const auto out = optimized(input, {.quality = 90});
    CHECK(out.format == Format::Png);
    CHECK(decoded(out.data).rgba == image.rgba);
  }
}

TEST_CASE("TIFF: lossless recompression stays TIFF") {
  const auto image = fixtures::make_image(96, 64, 6);
  const auto input = uncompressed_tiff(image);
  const auto out = optimized(input);
  CHECK(out.changed);
  CHECK(out.format == Format::Tiff);
  CHECK(out.data.size() < input.size());
  CHECK(decoded(out.data).rgba == image.rgba);
}

TEST_CASE("ICO: bitmap entries become PNG entries, pixels unchanged") {
  const auto image = fixtures::make_image(16, 16, 4, true);
  const auto input = dib_icon(image);
  CHECK(decoded(input).rgba == image.rgba);
  const auto out = optimized(input);
  CHECK(out.format == Format::Ico);
  CHECK(out.data.size() < input.size());
  CHECK(decoded(out.data).rgba == image.rgba);
}

TEST_CASE("conversions produce decodable files of the target format") {
  const auto source = *detail::encode_png(fixtures::make_image(300, 200, 6, true));
  for (auto target : {Format::Png, Format::Jpeg, Format::WebP, Format::Gif, Format::Bmp, Format::Tga, Format::Ico,
                      Format::Tiff}) {
    CAPTURE(format_name(target));
    auto out = convert(source, target, {.quality = 80});
    REQUIRE(out.has_value());
    CHECK(detect(out->data) == target);
    const auto result = decoded(out->data);
    if (target == Format::Ico) {
      CHECK(result.width == 256);
    } else {
      CHECK(result.width == 300);
    }
  }
  CHECK_FALSE(convert(fixtures::text_bytes("<svg/>"), Format::Png, {}).has_value());
}

TEST_CASE("undecodable images are errors, not silent passes") {
  Bytes broken = {0x89, 'P', 'N', 'G', '\r', '\n', 0x1A, '\n', 0, 0, 0, 13, 'I', 'H', 'D', 'R'};
  CHECK_FALSE(optimize(broken, {}).has_value());
}
