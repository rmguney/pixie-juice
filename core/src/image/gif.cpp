// GIF through giflib. The optimizer re-encodes every frame from its own
// raster and palette (so pixels, timing, disposal and transparency are
// identical) and drops comment, plain-text and application extensions other
// than the loop count (NETSCAPE2.0/ANIMEXTS1.0) and ICC profile, unless
// metadata is preserved. Conversions quantize with giflib's median cut.

#include "internal.hpp"

#include <gif_lib.h>

#include <algorithm>
#include <array>
#include <cstdlib>
#include <cstring>
#include <unordered_map>

extern "C" int GifQuantizeBuffer(unsigned int width, unsigned int height, int* color_map_size,
                                 const GifByteType* red, const GifByteType* green,
                                 const GifByteType* blue, GifByteType* output, GifColorType* color_map);

namespace pixie::detail {

namespace {

// Total frame pixels a GIF may slurp into memory (each frame and the screen
// are also held to kMaxPixels).
constexpr std::uint64_t kMaxGifPixels = 512ull * 1024 * 1024;

unsigned le16(ByteView d, std::size_t at) { return unsigned(d[at] | d[at + 1] << 8); }

struct FrameScan {
  bool within_limits = true;
  unsigned frames = 0;
};

/// Walks the block structure (without decoding LZW), counting frames and
/// checking the screen and every frame against the limits before giflib
/// allocates for them. Malformed structure is left for giflib to report.
FrameScan scan_frames(ByteView d) {
  FrameScan scan;
  if (d.size() < 13) return scan;
  if (std::uint64_t(le16(d, 6)) * le16(d, 8) > kMaxPixels) return {false, 0};
  std::size_t pos = 13;
  if (d[10] & 0x80) pos += 3u << ((d[10] & 7) + 1);  // global colour table
  auto skip_sub_blocks = [&] {
    while (pos < d.size()) {
      const std::size_t n = d[pos++];
      if (n == 0) return;
      pos += n;
    }
  };
  std::uint64_t total = 0;
  while (pos < d.size()) {
    const std::uint8_t block = d[pos++];
    if (block == 0x21) {  // extension: label, then sub-blocks
      ++pos;
      skip_sub_blocks();
    } else if (block == 0x2C && pos + 9 <= d.size()) {  // image descriptor
      const std::uint64_t pixels = std::uint64_t(le16(d, pos + 4)) * le16(d, pos + 6);
      const std::uint8_t flags = d[pos + 8];
      total += pixels;
      ++scan.frames;
      if (pixels > kMaxPixels || total > kMaxGifPixels) return {false, scan.frames};
      pos += 9;
      if (flags & 0x80) pos += 3u << ((flags & 7) + 1);  // local colour table
      ++pos;                                              // LZW minimum code size
      skip_sub_blocks();
    } else {
      return scan;  // trailer, or structure giflib will reject
    }
  }
  return scan;
}

bool frames_within_limits(ByteView d) { return scan_frames(d).within_limits; }

struct Reader {
  ByteView data;
  std::size_t pos = 0;
};

int read_gif(GifFileType* gif, GifByteType* buffer, int length) {
  auto* reader = static_cast<Reader*>(gif->UserData);
  const auto n = std::min<std::size_t>(std::size_t(length), reader->data.size() - reader->pos);
  std::memcpy(buffer, reader->data.data() + reader->pos, n);
  reader->pos += n;
  return int(n);
}

int write_gif(GifFileType* gif, const GifByteType* buffer, int length) {
  auto* out = static_cast<Bytes*>(gif->UserData);
  out->insert(out->end(), buffer, buffer + length);
  return length;
}

GifFileType* open_gif(Reader& reader) {
  if (!frames_within_limits(reader.data)) return nullptr;
  int error = 0;
  GifFileType* gif = DGifOpen(&reader, read_gif, &error);
  if (!gif) return nullptr;
  if (DGifSlurp(gif) != GIF_OK) {
    DGifCloseFile(gif, &error);
    return nullptr;
  }
  std::uint64_t pixels = 0;
  for (int i = 0; i < gif->ImageCount; ++i) {
    pixels += std::uint64_t(gif->SavedImages[i].ImageDesc.Width) * gif->SavedImages[i].ImageDesc.Height;
  }
  if (gif->ImageCount == 0 || pixels > kMaxGifPixels) {
    DGifCloseFile(gif, &error);
    return nullptr;
  }
  return gif;
}

bool is_kept_application(const ExtensionBlock& block) {
  if (block.ByteCount < 11) return false;
  const auto* id = reinterpret_cast<const char*>(block.Bytes);
  return std::memcmp(id, "NETSCAPE2.0", 11) == 0 || std::memcmp(id, "ANIMEXTS1.0", 11) == 0 ||
         std::memcmp(id, "ICCRGBG1012", 11) == 0;
}

/// Frees an encoder file that will not be written: EGifCloseFile alone leaks
/// the frames and extension blocks.
void discard(GifFileType* gif) {
  GifFreeSavedImages(gif);
  GifFreeExtensions(&gif->ExtensionBlockCount, &gif->ExtensionBlocks);
  int error = 0;
  EGifCloseFile(gif, &error);
}

/// Extension blocks worth keeping without preserve_metadata: graphics control
/// blocks, and the kept application blocks with their continuation sub-blocks.
void copy_extensions(const SavedImage& from, SavedImage& to, bool keep_all) {
  bool keep_continuations = false;
  for (int i = 0; i < from.ExtensionBlockCount; ++i) {
    const ExtensionBlock& block = from.ExtensionBlocks[i];
    bool keep = keep_all;
    if (!keep_all) {
      if (block.Function == CONTINUE_EXT_FUNC_CODE) {
        keep = keep_continuations;
      } else {
        keep = block.Function == GRAPHICS_EXT_FUNC_CODE ||
               (block.Function == APPLICATION_EXT_FUNC_CODE && is_kept_application(block));
        keep_continuations = keep && block.Function == APPLICATION_EXT_FUNC_CODE;
      }
    }
    if (keep) {
      GifAddExtensionBlock(&to.ExtensionBlockCount, &to.ExtensionBlocks, block.Function, unsigned(block.ByteCount),
                           block.Bytes);
    }
  }
}

/// Appends a copy of `from` to `target`. (GifMakeSavedImage's own copy drops the
/// extension block count in giflib 6.1, so frames are copied field by field.)
bool copy_frame(GifFileType* target, const SavedImage& from, bool keep_all_extensions) {
  SavedImage* frame = GifMakeSavedImage(target, nullptr);
  if (!frame) return false;
  frame->ImageDesc = from.ImageDesc;
  frame->ImageDesc.ColorMap = nullptr;
  if (from.ImageDesc.ColorMap) {
    frame->ImageDesc.ColorMap = GifMakeMapObject(from.ImageDesc.ColorMap->ColorCount, from.ImageDesc.ColorMap->Colors);
    if (!frame->ImageDesc.ColorMap) return false;
  }
  const std::size_t pixels = std::size_t(from.ImageDesc.Width) * std::size_t(from.ImageDesc.Height);
  frame->RasterBits = static_cast<GifByteType*>(std::malloc(pixels ? pixels : 1));
  if (!frame->RasterBits) return false;
  std::memcpy(frame->RasterBits, from.RasterBits, pixels);
  copy_extensions(from, *frame, keep_all_extensions);
  return true;
}

/// Every frame decodes the same: placement, palette colours, raster and
/// graphics control (delay, disposal, transparency).
bool same_frames(GifFileType* a, GifFileType* b) {
  if (a->ImageCount != b->ImageCount || a->SWidth != b->SWidth || a->SHeight != b->SHeight) return false;
  auto colours = [](const ColorMapObject* map) {
    return map ? std::vector<GifColorType>(map->Colors, map->Colors + map->ColorCount) : std::vector<GifColorType>();
  };
  auto same_colours = [&](const ColorMapObject* x, const ColorMapObject* y) {
    const auto cx = colours(x), cy = colours(y);
    return cx.size() == cy.size() && std::equal(cx.begin(), cx.end(), cy.begin(), [](const GifColorType& p, const GifColorType& q) {
             return p.Red == q.Red && p.Green == q.Green && p.Blue == q.Blue;
           });
  };
  if (!same_colours(a->SColorMap, b->SColorMap)) return false;
  for (int i = 0; i < a->ImageCount; ++i) {
    const GifImageDesc& x = a->SavedImages[i].ImageDesc;
    const GifImageDesc& y = b->SavedImages[i].ImageDesc;
    if (x.Left != y.Left || x.Top != y.Top || x.Width != y.Width || x.Height != y.Height ||
        !same_colours(x.ColorMap, y.ColorMap) ||
        std::memcmp(a->SavedImages[i].RasterBits, b->SavedImages[i].RasterBits, std::size_t(x.Width) * x.Height) != 0) {
      return false;
    }
    GraphicsControlBlock gx{}, gy{};
    gx.TransparentColor = gy.TransparentColor = NO_TRANSPARENT_COLOR;
    DGifSavedExtensionToGCB(a, i, &gx);
    DGifSavedExtensionToGCB(b, i, &gy);
    if (gx.DisposalMode != gy.DisposalMode || gx.DelayTime != gy.DelayTime || gx.TransparentColor != gy.TransparentColor) {
      return false;
    }
  }
  return true;
}

}  // namespace

bool gif_is_animated(ByteView data) { return scan_frames(data).frames > 1; }

Result<Image> decode_gif(ByteView data) {
  if (!frames_within_limits(data)) return fail("Not a valid GIF");
  Reader reader{data};
  int error = 0;
  GifFileType* gif = DGifOpen(&reader, read_gif, &error);
  if (!gif) return fail("Not a valid GIF");
  // Only the first frame is read: the records up to its image, then its raster.
  GraphicsControlBlock control{};
  control.TransparentColor = NO_TRANSPARENT_COLOR;
  std::vector<GifPixelType> raster;
  bool ok = true, found = false;
  while (ok && !found) {
    GifRecordType type = UNDEFINED_RECORD_TYPE;
    ok = DGifGetRecordType(gif, &type) == GIF_OK && type != TERMINATE_RECORD_TYPE;
    if (ok && type == EXTENSION_RECORD_TYPE) {
      int code = 0;
      GifByteType* block = nullptr;
      ok = DGifGetExtension(gif, &code, &block) == GIF_OK;
      if (ok && code == GRAPHICS_EXT_FUNC_CODE && block) DGifExtensionToGCB(block[0], block + 1, &control);
      while (ok && block) ok = DGifGetExtensionNext(gif, &block) == GIF_OK;
    } else if (ok && type == IMAGE_DESC_RECORD_TYPE) {
      ok = DGifGetImageDesc(gif) == GIF_OK;
      if (!ok) break;
      const GifImageDesc& d = gif->Image;
      raster.resize(std::size_t(d.Width) * std::size_t(d.Height));
      // Interlaced rows arrive in four passes.
      static constexpr int offsets[] = {0, 4, 2, 1}, steps[] = {8, 8, 4, 2};
      for (int pass = 0; pass < (d.Interlace ? 4 : 1) && ok; ++pass) {
        const int first = d.Interlace ? offsets[pass] : 0, step = d.Interlace ? steps[pass] : 1;
        for (int y = first; y < d.Height && ok; y += step) {
          ok = DGifGetLine(gif, raster.data() + std::size_t(y) * std::size_t(d.Width), d.Width) == GIF_OK;
        }
      }
      found = ok;
    }
  }
  if (!found) {
    DGifCloseFile(gif, &error);
    return fail("Not a valid GIF");
  }
  const GifImageDesc& desc = gif->Image;
  const ColorMapObject* map = desc.ColorMap ? desc.ColorMap : gif->SColorMap;

  Image image;
  // A zero screen size means the first frame's extent.
  image.width = gif->SWidth ? std::uint32_t(gif->SWidth) : std::uint32_t(desc.Left + desc.Width);
  image.height = gif->SHeight ? std::uint32_t(gif->SHeight) : std::uint32_t(desc.Top + desc.Height);
  Result<Image> result = fail("GIF dimensions too large");
  if (pixels_in_range(image.width, image.height)) {
    // Pixels the first frame doesn't cover stay transparent, as browsers show them
    // (they ignore the background colour).
    image.rgba.assign(image.pixel_count() * 4, 0);
    for (int y = 0; y < desc.Height; ++y) {
      for (int x = 0; x < desc.Width; ++x) {
        const std::uint32_t cx = std::uint32_t(desc.Left + x), cy = std::uint32_t(desc.Top + y);
        const int index = raster[std::size_t(y) * desc.Width + x];
        if (cx >= image.width || cy >= image.height || index == control.TransparentColor || !map ||
            index >= map->ColorCount) {
          continue;
        }
        std::uint8_t* pixel = &image.rgba[(std::size_t(cy) * image.width + cx) * 4];
        pixel[0] = map->Colors[index].Red;
        pixel[1] = map->Colors[index].Green;
        pixel[2] = map->Colors[index].Blue;
        pixel[3] = 255;
      }
    }
    result = std::move(image);
  }
  DGifCloseFile(gif, &error);
  return result;
}

std::vector<Bytes> gif_candidates(ByteView input, const Options& options) {
  std::vector<Bytes> candidates;
  Reader reader{input};
  GifFileType* source = open_gif(reader);
  if (!source) return candidates;

  Bytes out;
  int error = 0;
  GifFileType* target = EGifOpen(&out, write_gif, &error);
  if (!target) {
    DGifCloseFile(source, &error);
    return candidates;
  }
  target->SWidth = source->SWidth;
  target->SHeight = source->SHeight;
  target->SColorResolution = source->SColorResolution;
  target->SBackGroundColor = source->SBackGroundColor;
  target->AspectByte = source->AspectByte;
  target->SColorMap = source->SColorMap
                          ? GifMakeMapObject(source->SColorMap->ColorCount, source->SColorMap->Colors)
                          : nullptr;
  bool ok = true;
  for (int i = 0; i < source->ImageCount && ok; ++i) {
    ok = copy_frame(target, source->SavedImages[i], options.preserve_metadata);
  }
  if (ok && options.preserve_metadata) {
    for (int i = 0; i < source->ExtensionBlockCount; ++i) {
      const ExtensionBlock& block = source->ExtensionBlocks[i];
      GifAddExtensionBlock(&target->ExtensionBlockCount, &target->ExtensionBlocks, block.Function,
                           unsigned(block.ByteCount), block.Bytes);
    }
  }
  if (!ok) {
    discard(target);
    DGifCloseFile(source, &error);
    return candidates;
  }
  // The first frame is checked by pixels like any image; this checks them all.
  if (EGifSpew(target, &error) == GIF_OK) {
    Reader written{out};
    if (GifFileType* reread = open_gif(written)) {
      if (same_frames(source, reread)) candidates.push_back(std::move(out));
      DGifCloseFile(reread, &error);
    }
  }
  DGifCloseFile(source, &error);
  return candidates;
}

namespace {

/// Palette indices for `image`: exact when it has at most 256 colours (255
/// plus a transparent entry), otherwise median-cut over the opaque pixels with
/// each entry set to the true mean of the pixels mapped to it. Pixels with
/// alpha below 128 get one extra entry, reported in `clear_index` (-1 if none).
bool build_palette(const Image& image, std::vector<GifColorType>& palette, Bytes& indices, int& clear_index) {
  const std::size_t pixels = image.pixel_count();
  auto is_clear = [&](std::size_t i) { return image.rgba[i * 4 + 3] < 128; };
  bool any_clear = false;
  for (std::size_t i = 0; i < pixels && !any_clear; ++i) any_clear = is_clear(i);
  const std::size_t capacity = any_clear ? 255 : 256;
  auto rgb = [&](std::size_t i) {
    return std::uint32_t(image.rgba[i * 4]) << 16 | std::uint32_t(image.rgba[i * 4 + 1]) << 8 | image.rgba[i * 4 + 2];
  };
  indices.resize(pixels);

  std::unordered_map<std::uint32_t, std::uint8_t> exact;
  bool fits = true;
  for (std::size_t i = 0; i < pixels && fits; ++i) {
    if (is_clear(i)) continue;
    const auto [it, inserted] = exact.try_emplace(rgb(i), std::uint8_t(exact.size()));
    if (inserted && exact.size() > capacity) fits = false;
    indices[i] = it->second;
  }
  if (fits) {
    palette.assign(exact.size(), GifColorType{});
    for (const auto& [colour, index] : exact) {
      palette[index] = GifColorType{std::uint8_t(colour >> 16), std::uint8_t(colour >> 8), std::uint8_t(colour)};
    }
  } else {
    Bytes red, green, blue;
    std::vector<std::size_t> opaque;
    for (std::size_t i = 0; i < pixels; ++i) {
      if (is_clear(i)) continue;
      opaque.push_back(i);
      red.push_back(image.rgba[i * 4]);
      green.push_back(image.rgba[i * 4 + 1]);
      blue.push_back(image.rgba[i * 4 + 2]);
    }
    int colors = int(capacity);
    GifColorType quantized[256] = {};
    Bytes mapped(opaque.size());
    if (GifQuantizeBuffer(unsigned(opaque.size()), 1, &colors, red.data(), green.data(), blue.data(), mapped.data(),
                          quantized) != GIF_OK) {
      return false;
    }
    // giflib's median cut works on 5-bit channels; refine each entry to the 8-bit mean.
    std::vector<std::array<std::uint64_t, 4>> sums(static_cast<std::size_t>(colors));
    for (std::size_t k = 0; k < opaque.size(); ++k) {
      auto& sum = sums[mapped[k]];
      sum[0] += red[k];
      sum[1] += green[k];
      sum[2] += blue[k];
      ++sum[3];
      indices[opaque[k]] = mapped[k];
    }
    palette.resize(std::size_t(colors));
    for (std::size_t c = 0; c < palette.size(); ++c) {
      const auto n = sums[c][3];
      palette[c] = n ? GifColorType{std::uint8_t((sums[c][0] + n / 2) / n), std::uint8_t((sums[c][1] + n / 2) / n),
                                    std::uint8_t((sums[c][2] + n / 2) / n)}
                     : quantized[c];
    }
  }
  clear_index = -1;
  if (any_clear) {
    clear_index = int(palette.size());
    palette.push_back(GifColorType{0, 0, 0});
    for (std::size_t i = 0; i < pixels; ++i) {
      if (is_clear(i)) indices[i] = std::uint8_t(clear_index);
    }
  }
  // GIF colour tables hold a power of two entries, at least 2.
  std::size_t size = 2;
  while (size < palette.size()) size *= 2;
  palette.resize(size, GifColorType{0, 0, 0});
  return true;
}

}  // namespace

Result<Bytes> encode_gif(const Image& image) {
  if (image.width > 0xFFFF || image.height > 0xFFFF) return fail("GIF dimensions exceed 65535");
  std::vector<GifColorType> palette;
  Bytes indices;
  int clear_index = -1;
  if (!build_palette(image, palette, indices, clear_index)) return fail("GIF quantization failed");

  Bytes out;
  int error = 0;
  GifFileType* gif = EGifOpen(&out, write_gif, &error);
  if (!gif) return fail("GIF encoder init failed");
  gif->SWidth = int(image.width);
  gif->SHeight = int(image.height);
  gif->SColorResolution = 8;
  gif->SBackGroundColor = 0;
  gif->SColorMap = GifMakeMapObject(int(palette.size()), palette.data());
  SavedImage* frame = GifMakeSavedImage(gif, nullptr);
  if (!gif->SColorMap || !frame) {
    discard(gif);
    return fail("GIF encoder out of memory");
  }
  frame->ImageDesc = GifImageDesc{0, 0, int(image.width), int(image.height), false, nullptr};
  frame->RasterBits = static_cast<GifByteType*>(std::malloc(indices.size()));
  if (!frame->RasterBits) {
    discard(gif);
    return fail("GIF encoder out of memory");
  }
  std::memcpy(frame->RasterBits, indices.data(), indices.size());
  if (clear_index >= 0) {
    GraphicsControlBlock control{DISPOSAL_UNSPECIFIED, false, 0, clear_index};
    EGifGCBToSavedExtension(&control, gif, 0);
  }
  if (EGifSpew(gif, &error) != GIF_OK) return fail("GIF encode failed");
  return out;
}

}  // namespace pixie::detail
