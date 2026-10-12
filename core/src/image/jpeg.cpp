// JPEG through mozjpeg.
//
// Candidates:
//  1. lossless transcode (jpegtran-style): the DCT coefficients are copied and
//     re-entropy-coded as optimized progressive; pixels are bit-identical;
//  2. a re-encode at the requested quality with mozjpeg's defaults (trellis
//     quantisation, optimized progressive scans), unless lossless.
//
// Metadata: by default only what changes how the image looks survives, the ICC
// profile and the EXIF orientation (rewritten as a minimal EXIF block, so GPS,
// camera data and thumbnails are dropped). With preserve_metadata every APPn
// and comment segment is copied.
//
// libjpeg reports errors through longjmp, so each function that calls setjmp
// keeps only trivially destructible locals after it; the buffers it fills are
// owned by the caller and written through a destination manager, so nothing
// allocated by libjpeg outlives an error.

#include "internal.hpp"

#include <algorithm>
#include <csetjmp>
#include <cstdio>
#include <cstring>

#include <jpeglib.h>

namespace pixie::detail {

namespace {

#ifdef _MSC_VER
// 4324: padding from jmp_buf's alignment is expected. 4611: setjmp functions
// below hold no objects with destructors, so longjmp skips none.
#pragma warning(disable : 4324 4611)
#endif

using namespace std::string_view_literals;

struct ErrorManager {
  jpeg_error_mgr pub;
  std::jmp_buf jump;
  char message[JMSG_LENGTH_MAX];
};

extern "C" void on_jpeg_error(j_common_ptr cinfo) {
  auto* manager = reinterpret_cast<ErrorManager*>(cinfo->err);
  cinfo->err->format_message(cinfo, manager->message);
  std::longjmp(manager->jump, 1);
}

// Warnings would otherwise go to stderr, which is the browser console in WASM.
extern "C" void on_jpeg_output(j_common_ptr) {}

void init_errors(ErrorManager& manager) {
  jpeg_std_error(&manager.pub);
  manager.pub.error_exit = on_jpeg_error;
  manager.pub.output_message = on_jpeg_output;
  manager.message[0] = '\0';
}

/// Compressed output appended to a caller-owned buffer.
struct Destination {
  jpeg_destination_mgr pub;
  Bytes* out;
  JOCTET block[16384];
};

extern "C" void destination_init(j_compress_ptr cinfo) {
  auto* d = reinterpret_cast<Destination*>(cinfo->dest);
  d->pub.next_output_byte = d->block;
  d->pub.free_in_buffer = sizeof d->block;
}

extern "C" boolean destination_flush(j_compress_ptr cinfo) {
  auto* d = reinterpret_cast<Destination*>(cinfo->dest);
  d->out->insert(d->out->end(), d->block, d->block + sizeof d->block);
  d->pub.next_output_byte = d->block;
  d->pub.free_in_buffer = sizeof d->block;
  return TRUE;
}

extern "C" void destination_term(j_compress_ptr cinfo) {
  auto* d = reinterpret_cast<Destination*>(cinfo->dest);
  d->out->insert(d->out->end(), d->block, d->block + (sizeof d->block - d->pub.free_in_buffer));
}

void use_destination(j_compress_ptr cinfo, Destination& destination, Bytes& out) {
  destination.pub.init_destination = destination_init;
  destination.pub.empty_output_buffer = destination_flush;
  destination.pub.term_destination = destination_term;
  destination.out = &out;
  cinfo->dest = &destination.pub;
}

struct Marker {
  int code = 0;
  Bytes data;
};

bool has_signature(const Marker& marker, int code, std::string_view signature) {
  return marker.code == code && marker.data.size() >= signature.size() &&
         std::memcmp(marker.data.data(), signature.data(), signature.size()) == 0;
}

constexpr auto kExif = "Exif\0\0"sv;
constexpr auto kIcc = "ICC_PROFILE\0"sv;

/// APPn and COM segments before the first scan, or nullopt if the segment
/// structure is malformed. Bounded by the input; never reads past it.
std::optional<std::vector<Marker>> read_markers(ByteView data) {
  std::vector<Marker> out;
  std::size_t pos = 2;
  while (true) {
    while (pos + 1 < data.size() && data[pos] == 0xFF && data[pos + 1] == 0xFF) ++pos;  // fill bytes
    if (pos + 4 > data.size() || data[pos] != 0xFF) return std::nullopt;
    const int marker = data[pos + 1];
    if (marker == 0xDA || marker == 0xD9) return out;  // start of scan / end of image
    const std::size_t length = std::size_t(data[pos + 2]) << 8 | data[pos + 3];
    if (length < 2 || length > data.size() - pos - 2) return std::nullopt;
    if ((marker >= JPEG_APP0 && marker <= JPEG_APP0 + 15) || marker == JPEG_COM) {
      out.push_back(Marker{marker, Bytes(data.begin() + std::ptrdiff_t(pos + 4),
                                         data.begin() + std::ptrdiff_t(pos + 2 + length))});
    }
    pos += 2 + length;
  }
}

/// Markers to carry into an optimized file. The library writes JFIF (APP0)
/// and Adobe (APP14) itself, so those are never copied.
std::vector<Marker> markers_to_keep(const std::vector<Marker>& all, bool preserve_metadata) {
  std::vector<Marker> keep;
  int orientation = 1;
  for (const auto& marker : all) {
    if (has_signature(marker, JPEG_APP0, "JFIF\0"sv) || has_signature(marker, JPEG_APP0 + 14, "Adobe"sv)) {
      continue;
    }
    if (preserve_metadata || has_signature(marker, JPEG_APP0 + 2, kIcc)) {
      keep.push_back(marker);
    } else if (has_signature(marker, JPEG_APP0 + 1, kExif)) {
      orientation = exif_orientation(ByteView(marker.data).subspan(kExif.size()));
    }
  }
  if (!preserve_metadata && orientation != 1) {
    Marker exif{JPEG_APP0 + 1, Bytes(kExif.begin(), kExif.end())};
    const Bytes tiff = minimal_exif(orientation);
    exif.data.insert(exif.data.end(), tiff.begin(), tiff.end());
    keep.insert(keep.begin(), std::move(exif));
  }
  return keep;
}

/// An ICC profile as APP2 segments of at most 65,519 profile bytes each.
std::vector<Marker> icc_markers(ByteView icc) {
  constexpr std::size_t kChunk = 65519;
  std::vector<Marker> out;
  const std::size_t count = (icc.size() + kChunk - 1) / kChunk;
  if (count == 0 || count > 255) return out;
  for (std::size_t i = 0; i < count; ++i) {
    Marker marker{JPEG_APP0 + 2, Bytes(kIcc.begin(), kIcc.end())};
    marker.data.push_back(std::uint8_t(i + 1));
    marker.data.push_back(std::uint8_t(count));
    const auto chunk = icc.subspan(i * kChunk, std::min(kChunk, icc.size() - i * kChunk));
    marker.data.insert(marker.data.end(), chunk.begin(), chunk.end());
    out.push_back(std::move(marker));
  }
  return out;
}

// ---------------------------------------------------------------------------
// libjpeg wrappers (setjmp boundaries)

/// True for CMYK and YCCK images.
bool is_cmyk(ByteView data) {
  ErrorManager em;
  init_errors(em);
  jpeg_decompress_struct cinfo{};
  cinfo.err = &em.pub;
  if (setjmp(em.jump)) {
    jpeg_destroy_decompress(&cinfo);
    return false;
  }
  jpeg_create_decompress(&cinfo);
  jpeg_mem_src(&cinfo, data.data(), static_cast<unsigned long>(data.size()));
  jpeg_read_header(&cinfo, TRUE);
  const bool cmyk = cinfo.jpeg_color_space == JCS_CMYK || cinfo.jpeg_color_space == JCS_YCCK;
  jpeg_destroy_decompress(&cinfo);
  return cmyk;
}

bool transcode(ByteView data, const std::vector<Marker>& markers, Bytes& out, std::string& error) {
  ErrorManager em;
  init_errors(em);
  Destination destination;
  jpeg_decompress_struct src{};
  jpeg_compress_struct dst{};
  src.err = &em.pub;
  dst.err = &em.pub;
  if (setjmp(em.jump)) {
    error = em.message;
    jpeg_destroy_compress(&dst);
    jpeg_destroy_decompress(&src);
    return false;
  }
  jpeg_create_decompress(&src);
  jpeg_create_compress(&dst);
  jpeg_mem_src(&src, data.data(), static_cast<unsigned long>(data.size()));
  jpeg_read_header(&src, TRUE);
  jvirt_barray_ptr* coefficients = jpeg_read_coefficients(&src);
  jpeg_copy_critical_parameters(&src, &dst);
  jpeg_simple_progression(&dst);
  dst.optimize_coding = TRUE;
  use_destination(&dst, destination, out);
  jpeg_write_coefficients(&dst, coefficients);
  for (const auto& marker : markers) {
    jpeg_write_marker(&dst, marker.code, marker.data.data(), static_cast<unsigned>(marker.data.size()));
  }
  jpeg_finish_compress(&dst);
  jpeg_finish_decompress(&src);
  jpeg_destroy_compress(&dst);
  jpeg_destroy_decompress(&src);
  return true;
}

bool encode_rows(const Image& image, int quality, bool gray, const std::vector<Marker>& markers, Bytes& row,
                 Bytes& out, std::string& error) {
  ErrorManager em;
  init_errors(em);
  Destination destination;
  jpeg_compress_struct cinfo{};
  cinfo.err = &em.pub;
  if (setjmp(em.jump)) {
    error = em.message;
    jpeg_destroy_compress(&cinfo);
    return false;
  }
  jpeg_create_compress(&cinfo);
  use_destination(&cinfo, destination, out);
  cinfo.image_width = image.width;
  cinfo.image_height = image.height;
  cinfo.input_components = gray ? 1 : 3;
  cinfo.in_color_space = gray ? JCS_GRAYSCALE : JCS_RGB;
  jpeg_set_defaults(&cinfo);  // mozjpeg: trellis quantisation, optimized progressive scans
  jpeg_set_quality(&cinfo, quality, TRUE);
  jpeg_start_compress(&cinfo, TRUE);
  for (const auto& marker : markers) {
    jpeg_write_marker(&cinfo, marker.code, marker.data.data(), static_cast<unsigned>(marker.data.size()));
  }
  while (cinfo.next_scanline < cinfo.image_height) {
    const auto* px = image.rgba.data() + std::size_t(cinfo.next_scanline) * image.width * 4;
    for (std::uint32_t x = 0; x < image.width; ++x, px += 4) {
      // JPEG has no alpha: composite onto white.
      const unsigned a = px[3];
      auto over_white = [a](unsigned c) { return std::uint8_t((c * a + 255 * (255 - a) + 127) / 255); };
      if (gray) {
        row[x] = over_white(px[0]);
      } else {
        row[x * 3] = over_white(px[0]);
        row[x * 3 + 1] = over_white(px[1]);
        row[x * 3 + 2] = over_white(px[2]);
      }
    }
    JSAMPROW rows[1] = {row.data()};
    jpeg_write_scanlines(&cinfo, rows, 1);
  }
  jpeg_finish_compress(&cinfo);
  jpeg_destroy_compress(&cinfo);
  return true;
}

bool encode(const Image& image, int quality, const std::vector<Marker>& markers, Bytes& out, std::string& error) {
  const bool gray = image.is_gray() && !image.has_alpha();
  Bytes row(std::size_t(image.width) * (gray ? 1 : 3));
  return encode_rows(image, quality, gray, markers, row, out, error);
}

bool decode(ByteView data, Image& image, std::string& error) {
  ErrorManager em;
  init_errors(em);
  jpeg_decompress_struct cinfo{};
  cinfo.err = &em.pub;
  if (setjmp(em.jump)) {
    error = em.message;
    jpeg_destroy_decompress(&cinfo);
    return false;
  }
  jpeg_create_decompress(&cinfo);
  jpeg_mem_src(&cinfo, data.data(), static_cast<unsigned long>(data.size()));
  jpeg_read_header(&cinfo, TRUE);
  if (std::uint64_t(cinfo.image_width) * cinfo.image_height > kMaxPixels) {
    error = "JPEG dimensions too large";
    jpeg_destroy_decompress(&cinfo);
    return false;
  }
  const bool cmyk = cinfo.jpeg_color_space == JCS_CMYK || cinfo.jpeg_color_space == JCS_YCCK;
  cinfo.out_color_space = cmyk ? JCS_CMYK : JCS_EXT_RGBA;
  jpeg_start_decompress(&cinfo);
  image.width = cinfo.output_width;
  image.height = cinfo.output_height;
  image.rgba.resize(image.pixel_count() * 4);
  while (cinfo.output_scanline < cinfo.output_height) {
    JSAMPROW rows[1] = {image.rgba.data() + std::size_t(cinfo.output_scanline) * image.width * 4};
    jpeg_read_scanlines(&cinfo, rows, 1);
  }
  const bool inverted = cinfo.saw_Adobe_marker;
  jpeg_finish_decompress(&cinfo);
  jpeg_destroy_decompress(&cinfo);
  if (cmyk) {
    // Photoshop (Adobe marker) stores CMYK inverted.
    for (std::size_t i = 0; i < image.rgba.size(); i += 4) {
      unsigned c = image.rgba[i], m = image.rgba[i + 1], y = image.rgba[i + 2], k = image.rgba[i + 3];
      if (!inverted) {
        c = 255 - c;
        m = 255 - m;
        y = 255 - y;
        k = 255 - k;
      }
      image.rgba[i] = std::uint8_t(c * k / 255);
      image.rgba[i + 1] = std::uint8_t(m * k / 255);
      image.rgba[i + 2] = std::uint8_t(y * k / 255);
      image.rgba[i + 3] = 255;
    }
  }
  return true;
}

}  // namespace

Result<Image> decode_jpeg(ByteView data) {
  Image image;
  std::string error;
  if (!decode(data, image, error)) return fail("JPEG decode failed: " + error);
  return image;
}

std::vector<Bytes> jpeg_candidates(ByteView input, const Image& image, const Options& options) {
  std::vector<Bytes> candidates;
  const auto markers = read_markers(input);
  if (!markers) return candidates;
  const auto keep = markers_to_keep(*markers, options.preserve_metadata);
  std::string error;

  Bytes transcoded;
  if (transcode(input, keep, transcoded, error)) candidates.push_back(std::move(transcoded));

  // Re-encoding CMYK as RGB would change the colour model under its ICC profile.
  if (!options.lossless && !is_cmyk(input)) {
    Bytes encoded;
    if (encode(image, options.quality, keep, encoded, error)) candidates.push_back(std::move(encoded));
  }
  return candidates;
}

Result<Bytes> encode_jpeg(const Image& image, int quality, ByteView icc) {
  Bytes out;
  std::string error;
  if (!encode(image, quality, icc_markers(icc), out, error)) return fail("JPEG encode failed: " + error);
  return out;
}

}  // namespace pixie::detail
