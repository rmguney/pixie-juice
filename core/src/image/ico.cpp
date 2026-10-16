// ICO: a directory of PNG or BMP (DIB) entries. Entries are decoded with
// lodepng and stb_image; optimization re-encodes each entry as a lossless PNG
// through zopflipng and keeps whichever form is smaller. Every directory field
// is bounds-checked against the file before anything is read, and entries may
// neither overlap nor number more than kMaxEntries.

#include "internal.hpp"

#include <algorithm>
#include <cstring>

namespace pixie::detail {

namespace {

struct Entry {
  std::uint8_t width = 0, height = 0, colors = 0, reserved = 0;
  std::uint16_t planes = 0, bpp = 0;
  std::uint32_t size = 0, offset = 0;
};

unsigned u16(ByteView d, std::size_t at) { return unsigned(d[at] | d[at + 1] << 8); }
std::uint32_t u32(ByteView d, std::size_t at) { return std::uint32_t(u16(d, at) | u16(d, at + 2) << 16); }
std::int32_t s32(ByteView d, std::size_t at) { return std::int32_t(u32(d, at)); }

// Real icons hold a handful of sizes; each entry is decoded and re-encoded.
constexpr unsigned kMaxEntries = 64;

std::optional<std::vector<Entry>> parse_directory(ByteView data) {
  if (data.size() < 6 || u16(data, 0) != 0 || u16(data, 2) != 1) return std::nullopt;
  const unsigned count = u16(data, 4);
  if (count == 0 || count > kMaxEntries || 6 + std::size_t(count) * 16 > data.size()) return std::nullopt;
  std::vector<Entry> entries(count);
  for (unsigned i = 0; i < count; ++i) {
    const std::size_t at = 6 + std::size_t(i) * 16;
    Entry& e = entries[i];
    e.width = data[at];
    e.height = data[at + 1];
    e.colors = data[at + 2];
    e.reserved = data[at + 3];
    e.planes = std::uint16_t(u16(data, at + 4));
    e.bpp = std::uint16_t(u16(data, at + 6));
    e.size = u32(data, at + 8);
    e.offset = u32(data, at + 12);
    if (e.size == 0 || e.offset > data.size() || e.size > data.size() - e.offset) return std::nullopt;
  }
  // Entries may not share bytes: overlapping ones would multiply the work on one region.
  std::vector<std::pair<std::uint32_t, std::uint32_t>> ranges;
  for (const auto& e : entries) ranges.emplace_back(e.offset, e.offset + e.size);
  std::sort(ranges.begin(), ranges.end());
  for (std::size_t i = 1; i < ranges.size(); ++i) {
    if (ranges[i].first < ranges[i - 1].second) return std::nullopt;
  }
  return entries;
}

ByteView payload(ByteView data, const Entry& e) { return data.subspan(e.offset, e.size); }

bool is_png(ByteView payload) {
  return payload.size() >= 8 && std::memcmp(payload.data(), "\x89PNG\r\n\x1A\n", 8) == 0;
}

/// A DIB entry: a BITMAPINFOHEADER whose height covers the colour bitmap and
/// a 1-bit AND (transparency) mask below it.
Result<Image> decode_dib(ByteView dib) {
  if (dib.size() < 40 || u32(dib, 0) < 40) return fail("Invalid ICO bitmap entry");
  const std::uint32_t header = u32(dib, 0);
  const std::int32_t width = s32(dib, 4);
  const std::int32_t height = s32(dib, 8) / 2;
  const unsigned bpp = u16(dib, 14);
  if (width <= 0 || height <= 0 || header > dib.size()) return fail("Invalid ICO bitmap entry");
  const std::uint32_t palette_entries = bpp <= 8 ? (u32(dib, 32) ? u32(dib, 32) : 1u << bpp) : 0;
  const std::uint64_t xor_offset = std::uint64_t(header) + std::uint64_t(palette_entries) * 4;
  const std::uint64_t xor_stride = (std::uint64_t(width) * bpp + 31) / 32 * 4;
  const std::uint64_t mask_offset = xor_offset + xor_stride * std::uint64_t(height);
  const std::uint64_t mask_stride = (std::uint64_t(width) + 31) / 32 * 4;
  if (xor_offset > dib.size() || mask_offset > dib.size()) return fail("Truncated ICO bitmap entry");

  // Wrap the colour bitmap in a BMP file header with the real height and let stb decode it.
  Bytes bmp;
  bmp.reserve(14 + std::size_t(mask_offset));
  bmp.push_back('B');
  bmp.push_back('M');
  append_le32(bmp, std::uint32_t(14 + mask_offset));
  append_le32(bmp, 0);
  append_le32(bmp, std::uint32_t(14 + xor_offset));
  const std::size_t header_at = bmp.size();
  bmp.insert(bmp.end(), dib.begin(), dib.begin() + std::ptrdiff_t(mask_offset));
  const auto real_height = std::uint32_t(height);
  std::memcpy(bmp.data() + header_at + 8, &real_height, 4);  // little-endian hosts (x86, WASM)
  auto image = decode_rgba(bmp, Format::Bmp);
  if (!image) return image;

  // Below 32 bpp, transparency comes from the AND mask (bottom-up rows).
  if (bpp < 32 && mask_offset + mask_stride * std::uint64_t(height) <= dib.size()) {
    for (std::int32_t y = 0; y < height; ++y) {
      const auto* row = dib.data() + mask_offset + mask_stride * std::uint64_t(height - 1 - y);
      for (std::int32_t x = 0; x < width; ++x) {
        if (row[x / 8] & (0x80 >> (x % 8))) image->rgba[(std::size_t(y) * width + x) * 4 + 3] = 0;
      }
    }
  }
  return image;
}

Result<Image> decode_entry(ByteView data, const Entry& e) {
  const auto bytes = payload(data, e);
  return is_png(bytes) ? decode_png(bytes) : decode_dib(bytes);
}

}  // namespace

Result<Image> decode_ico(ByteView data) {
  const auto entries = parse_directory(data);
  if (!entries) return fail("Not a valid ICO");
  // The entry a viewer shows at full size: most pixels, then most colour depth.
  const Entry* best = &(*entries)[0];
  auto area = [](const Entry& e) { return (e.width ? e.width : 256u) * (e.height ? e.height : 256u); };
  for (const auto& e : *entries) {
    if (area(e) > area(*best) || (area(e) == area(*best) && e.bpp > best->bpp)) best = &e;
  }
  return decode_entry(data, *best);
}

std::vector<Bytes> ico_candidates(ByteView input, const Options& options) {
  std::vector<Bytes> candidates;
  const auto entries = parse_directory(input);
  if (!entries) return candidates;

  std::vector<Bytes> payloads;
  for (const auto& e : *entries) {
    const auto original = payload(input, e);
    Bytes best(original.begin(), original.end());
    if (is_png(original)) {
      for (auto& c : png_candidates(original, options)) {
        if (c.size() < best.size()) best = std::move(c);
      }
    } else if (auto image = decode_dib(original)) {
      // Each entry must stay pixel-identical, whatever the quality setting.
      const Reference reference{*image, image->has_alpha(), image->is_gray()};
      if (auto png = encode_png(*image); png && png->size() < best.size() &&
                                         is_faithful(reference, *png, Format::Png, 0.0)) {
        best = std::move(*png);
      }
    }
    payloads.push_back(std::move(best));
  }

  Bytes out;
  append_le16(out, 0);
  append_le16(out, 1);
  append_le16(out, unsigned(entries->size()));
  std::uint32_t offset = std::uint32_t(6 + entries->size() * 16);
  for (std::size_t i = 0; i < entries->size(); ++i) {
    const Entry& e = (*entries)[i];
    out.push_back(e.width);
    out.push_back(e.height);
    out.push_back(e.colors);
    out.push_back(e.reserved);
    append_le16(out, e.planes);
    append_le16(out, e.bpp);
    append_le32(out, std::uint32_t(payloads[i].size()));
    append_le32(out, offset);
    offset += std::uint32_t(payloads[i].size());
  }
  for (const auto& p : payloads) out.insert(out.end(), p.begin(), p.end());
  candidates.push_back(std::move(out));
  return candidates;
}

}  // namespace pixie::detail
