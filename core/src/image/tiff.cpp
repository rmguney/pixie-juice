// TIFF through libtiff, read and written from memory. Optimization re-encodes
// single-page 8-bit greyscale/RGB(A) images with Deflate or LZW plus the
// horizontal predictor (lossless), keeping the ICC profile, orientation and
// resolution.
// Multi-page, 16-bit, float, CMYK and other layouts pass through unchanged,
// since an 8-bit RGBA round trip could not reproduce them.

#include "internal.hpp"

#include <tiffio.h>

#include <algorithm>
#include <cstring>

namespace pixie::detail {

namespace {

/// A seekable in-memory file for TIFFClientOpen: reads from `input`, or
/// reads/writes a growable buffer.
struct MemoryFile {
  ByteView input;
  Bytes* output = nullptr;
  std::uint64_t pos = 0;

  [[nodiscard]] std::uint64_t size() const { return output ? output->size() : input.size(); }
  [[nodiscard]] const std::uint8_t* data() const { return output ? output->data() : input.data(); }
};

tmsize_t mem_read(thandle_t handle, void* buffer, tmsize_t size) {
  auto* f = static_cast<MemoryFile*>(handle);
  if (size < 0 || f->pos >= f->size()) return 0;
  const auto n = std::min<std::uint64_t>(std::uint64_t(size), f->size() - f->pos);
  std::memcpy(buffer, f->data() + f->pos, std::size_t(n));
  f->pos += n;
  return tmsize_t(n);
}

tmsize_t mem_write(thandle_t handle, void* buffer, tmsize_t size) {
  auto* f = static_cast<MemoryFile*>(handle);
  if (!f->output || size < 0) return -1;
  const auto end = f->pos + std::uint64_t(size);
  if (end > f->output->size()) f->output->resize(std::size_t(end));
  std::memcpy(f->output->data() + f->pos, buffer, std::size_t(size));
  f->pos = end;
  return size;
}

toff_t mem_seek(thandle_t handle, toff_t offset, int whence) {
  auto* f = static_cast<MemoryFile*>(handle);
  std::uint64_t base = 0;
  if (whence == SEEK_CUR) base = f->pos;
  if (whence == SEEK_END) base = f->size();
  f->pos = base + offset;
  if (f->output && f->pos > f->output->size()) f->output->resize(std::size_t(f->pos));
  return f->pos;
}

int mem_close(thandle_t) { return 0; }
toff_t mem_size(thandle_t handle) { return static_cast<MemoryFile*>(handle)->size(); }
int mem_map(thandle_t, void**, toff_t*) { return 0; }
void mem_unmap(thandle_t, void*, toff_t) {}

// libtiff reports through process-wide handlers; silence them (stderr is the console in WASM).
void quiet(const char*, const char*, va_list) {}

TIFF* open(MemoryFile& file, const char* mode) {
  static const bool silenced = [] {
    TIFFSetErrorHandler(quiet);
    TIFFSetWarningHandler(quiet);
    return true;
  }();
  (void)silenced;
  return TIFFClientOpen("memory", mode, &file, mem_read, mem_write, mem_seek, mem_close, mem_size, mem_map,
                        mem_unmap);
}

struct Layout {
  std::uint32_t width = 0, height = 0;
  std::uint16_t bits = 0, samples = 0, photometric = 0, planar = 0, sample_format = SAMPLEFORMAT_UINT;
  std::uint16_t extra_type = EXTRASAMPLE_UNASSALPHA;
  bool multi_page = false;
};

Layout read_layout(TIFF* tif) {
  Layout l;
  TIFFGetField(tif, TIFFTAG_IMAGEWIDTH, &l.width);
  TIFFGetField(tif, TIFFTAG_IMAGELENGTH, &l.height);
  TIFFGetFieldDefaulted(tif, TIFFTAG_BITSPERSAMPLE, &l.bits);
  TIFFGetFieldDefaulted(tif, TIFFTAG_SAMPLESPERPIXEL, &l.samples);
  TIFFGetFieldDefaulted(tif, TIFFTAG_PLANARCONFIG, &l.planar);
  TIFFGetFieldDefaulted(tif, TIFFTAG_SAMPLEFORMAT, &l.sample_format);
  TIFFGetField(tif, TIFFTAG_PHOTOMETRIC, &l.photometric);
  std::uint16_t extra_count = 0;
  std::uint16_t* extra = nullptr;
  if (TIFFGetField(tif, TIFFTAG_EXTRASAMPLES, &extra_count, &extra) && extra_count > 0) l.extra_type = extra[0];
  l.multi_page = TIFFNumberOfDirectories(tif) > 1;
  return l;
}

/// Layouts an 8-bit RGBA round trip reproduces exactly.
bool reproducible(const Layout& l) {
  const bool gray = l.photometric == PHOTOMETRIC_MINISBLACK && (l.samples == 1 || l.samples == 2);
  const bool rgb = l.photometric == PHOTOMETRIC_RGB && (l.samples == 3 || l.samples == 4);
  return !l.multi_page && l.bits == 8 && l.sample_format == SAMPLEFORMAT_UINT &&
         l.planar == PLANARCONFIG_CONTIG && (gray || rgb) && std::uint64_t(l.width) * l.height <= kMaxPixels;
}

/// Exact samples of a reproducible layout, widened to RGBA.
std::optional<Image> read_samples(TIFF* tif, const Layout& l) {
  Image image;
  image.width = l.width;
  image.height = l.height;
  image.rgba.resize(image.pixel_count() * 4);
  Bytes row(std::size_t(TIFFScanlineSize64(tif)));
  if (row.size() < std::size_t(l.width) * l.samples) return std::nullopt;
  for (std::uint32_t y = 0; y < l.height; ++y) {
    if (TIFFReadScanline(tif, row.data(), y, 0) != 1) return std::nullopt;
    auto* px = image.rgba.data() + std::size_t(y) * l.width * 4;
    for (std::uint32_t x = 0; x < l.width; ++x, px += 4) {
      const auto* src = row.data() + std::size_t(x) * l.samples;
      if (l.samples <= 2) {
        px[0] = px[1] = px[2] = src[0];
        px[3] = l.samples == 2 ? src[1] : 255;
      } else {
        px[0] = src[0];
        px[1] = src[1];
        px[2] = src[2];
        px[3] = l.samples == 4 ? src[3] : 255;
      }
    }
  }
  return image;
}

/// Tags an optimized file keeps: they change how (or how large) the image displays.
struct Carried {
  Bytes icc;
  std::uint16_t orientation = ORIENTATION_TOPLEFT;
  float x_resolution = 0, y_resolution = 0;
  std::uint16_t resolution_unit = 0;
  bool has_resolution = false;
};

Carried read_carried(TIFF* tif) {
  Carried c;
  std::uint32_t icc_size = 0;
  void* icc = nullptr;
  if (TIFFGetField(tif, TIFFTAG_ICCPROFILE, &icc_size, &icc) && icc && icc_size) {
    const auto* p = static_cast<const std::uint8_t*>(icc);
    c.icc.assign(p, p + icc_size);
  }
  c.has_resolution = TIFFGetField(tif, TIFFTAG_XRESOLUTION, &c.x_resolution) &&
                     TIFFGetField(tif, TIFFTAG_YRESOLUTION, &c.y_resolution);
  TIFFGetFieldDefaulted(tif, TIFFTAG_RESOLUTIONUNIT, &c.resolution_unit);
  TIFFGetFieldDefaulted(tif, TIFFTAG_ORIENTATION, &c.orientation);
  return c;
}

/// Writes `image` with `samples` channels (1 grey, 2 grey+alpha, 3 RGB, 4 RGBA).
std::optional<Bytes> write_tiff(const Image& image, int samples, std::uint16_t extra_type, std::uint16_t compression,
                                const Carried& carried) {
  Bytes out;
  MemoryFile file{{}, &out};
  TIFF* tif = open(file, "w");
  if (!tif) return std::nullopt;
  const bool gray = samples <= 2;
  const bool alpha = samples == 2 || samples == 4;
  TIFFSetField(tif, TIFFTAG_IMAGEWIDTH, image.width);
  TIFFSetField(tif, TIFFTAG_IMAGELENGTH, image.height);
  TIFFSetField(tif, TIFFTAG_BITSPERSAMPLE, 8);
  TIFFSetField(tif, TIFFTAG_SAMPLESPERPIXEL, samples);
  TIFFSetField(tif, TIFFTAG_PHOTOMETRIC, gray ? PHOTOMETRIC_MINISBLACK : PHOTOMETRIC_RGB);
  TIFFSetField(tif, TIFFTAG_PLANARCONFIG, PLANARCONFIG_CONTIG);
  TIFFSetField(tif, TIFFTAG_ORIENTATION, carried.orientation);
  TIFFSetField(tif, TIFFTAG_COMPRESSION, compression);
  TIFFSetField(tif, TIFFTAG_PREDICTOR, PREDICTOR_HORIZONTAL);
  if (compression == COMPRESSION_ADOBE_DEFLATE) TIFFSetField(tif, TIFFTAG_ZIPQUALITY, 9);
  if (alpha) {
    const std::uint16_t extra[] = {extra_type};
    TIFFSetField(tif, TIFFTAG_EXTRASAMPLES, 1, extra);
  }
  if (!carried.icc.empty()) {
    TIFFSetField(tif, TIFFTAG_ICCPROFILE, std::uint32_t(carried.icc.size()), carried.icc.data());
  }
  if (carried.has_resolution) {
    TIFFSetField(tif, TIFFTAG_XRESOLUTION, carried.x_resolution);
    TIFFSetField(tif, TIFFTAG_YRESOLUTION, carried.y_resolution);
    TIFFSetField(tif, TIFFTAG_RESOLUTIONUNIT, carried.resolution_unit);
  }
  // Strips of about 1 MiB: each strip restarts the compressor's dictionary.
  const std::size_t row_bytes = std::size_t(image.width) * std::size_t(samples);
  const auto rows_per_strip = std::uint32_t(std::clamp<std::size_t>((1u << 20) / row_bytes, 1, image.height));
  TIFFSetField(tif, TIFFTAG_ROWSPERSTRIP, rows_per_strip);

  Bytes row(row_bytes);
  bool ok = true;
  for (std::uint32_t y = 0; y < image.height && ok; ++y) {
    const auto* px = image.rgba.data() + std::size_t(y) * image.width * 4;
    for (std::uint32_t x = 0; x < image.width; ++x, px += 4) {
      auto* dst = row.data() + std::size_t(x) * samples;
      if (gray) {
        dst[0] = px[0];
        if (alpha) dst[1] = px[3];
      } else {
        std::memcpy(dst, px, std::size_t(samples));
      }
    }
    ok = TIFFWriteScanline(tif, row.data(), y, 0) == 1;
  }
  TIFFClose(tif);
  if (!ok) return std::nullopt;
  return out;
}

}  // namespace

ColourInfo tiff_colour_info(ByteView data) {
  MemoryFile file{data};
  TIFF* tif = open(file, "r");
  if (!tif) return {};
  Carried carried = read_carried(tif);
  TIFFClose(tif);
  return ColourInfo{std::move(carried.icc), carried.orientation >= 1 && carried.orientation <= 8 ? carried.orientation : 1};
}

Result<Image> decode_tiff(ByteView data) {
  MemoryFile file{data};
  TIFF* tif = open(file, "r");
  if (!tif) return fail("Not a valid TIFF");
  std::uint32_t width = 0, height = 0;
  std::uint16_t orientation = ORIENTATION_TOPLEFT;
  TIFFGetField(tif, TIFFTAG_IMAGEWIDTH, &width);
  TIFFGetField(tif, TIFFTAG_IMAGELENGTH, &height);
  TIFFGetFieldDefaulted(tif, TIFFTAG_ORIENTATION, &orientation);
  if (width == 0 || height == 0 || std::uint64_t(width) * height > kMaxPixels) {
    TIFFClose(tif);
    return fail("TIFF dimensions too large");
  }
  Image image;
  image.width = width;
  image.height = height;
  image.rgba.resize(image.pixel_count() * 4);
  // Asking for the file's own orientation gives rows in stored order (libtiff
  // only flips, and mishandles the transposed orientations 5-8); display
  // orientation is applied by apply_orientation like every other format.
  // ABGR words in memory are R, G, B, A bytes on little-endian hosts (x86, WASM).
  // stop_on_error: a truncated or damaged strip is an error, not rows of garbage.
  const int ok = TIFFReadRGBAImageOriented(tif, width, height, reinterpret_cast<std::uint32_t*>(image.rgba.data()),
                                           orientation, 1);
  TIFFClose(tif);
  if (!ok) return fail("TIFF decode failed");
  // TIFFReadRGBAImage returns premultiplied colour; convert back to straight alpha.
  for (std::size_t i = 0; i < image.rgba.size(); i += 4) {
    const unsigned a = image.rgba[i + 3];
    if (a != 0 && a != 255) {
      for (int c = 0; c < 3; ++c) {
        image.rgba[i + c] = std::uint8_t(std::min(255u, (image.rgba[i + c] * 255u + a / 2) / a));
      }
    }
  }
  return image;
}

bool tiff_passes_through(ByteView data) {
  MemoryFile file{data};
  TIFF* tif = open(file, "r");
  if (!tif) return false;
  std::uint16_t compression = COMPRESSION_NONE, sample_format = SAMPLEFORMAT_UINT;
  TIFFGetFieldDefaulted(tif, TIFFTAG_COMPRESSION, &compression);
  TIFFGetFieldDefaulted(tif, TIFFTAG_SAMPLEFORMAT, &sample_format);
  const bool undecodable = !TIFFIsCODECConfigured(compression) || sample_format == SAMPLEFORMAT_IEEEFP ||
                           sample_format == SAMPLEFORMAT_COMPLEXIEEEFP || sample_format == SAMPLEFORMAT_COMPLEXINT;
  TIFFClose(tif);
  return undecodable;
}

std::vector<Bytes> tiff_candidates(ByteView input, const Options& options) {
  std::vector<Bytes> candidates;
  // Copying every private tag and sub-IFD faithfully is out of scope; with
  // metadata preservation on, TIFFs pass through.
  if (options.preserve_metadata) return candidates;

  MemoryFile file{input};
  TIFF* tif = open(file, "r");
  if (!tif) return candidates;
  const Layout layout = read_layout(tif);
  const Carried carried = read_carried(tif);
  std::optional<Image> image;
  if (reproducible(layout)) image = read_samples(tif, layout);
  TIFFClose(tif);
  if (!image) return candidates;

  for (std::uint16_t compression : {std::uint16_t(COMPRESSION_ADOBE_DEFLATE), std::uint16_t(COMPRESSION_LZW)}) {
    if (auto out = write_tiff(*image, layout.samples, layout.extra_type, compression, carried)) {
      candidates.push_back(std::move(*out));
    }
  }
  return candidates;
}

Result<Bytes> encode_tiff(const Image& image, ByteView icc) {
  const int samples = image.is_gray() ? (image.has_alpha() ? 2 : 1) : (image.has_alpha() ? 4 : 3);
  Carried carried;
  carried.icc.assign(icc.begin(), icc.end());
  if (auto out = write_tiff(image, samples, EXTRASAMPLE_UNASSALPHA, COMPRESSION_ADOBE_DEFLATE, carried)) {
    return std::move(*out);
  }
  return fail("TIFF encode failed");
}

}  // namespace pixie::detail
