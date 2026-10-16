#include "internal.hpp"

#include <algorithm>
#include <array>
#include <cstring>

namespace pixie {

using namespace std::string_view_literals;

namespace {

struct NamedFormat {
  Format format;
  std::string_view name;
};

constexpr std::array kNames{
    NamedFormat{Format::Png, "png"},   NamedFormat{Format::Jpeg, "jpeg"},
    NamedFormat{Format::WebP, "webp"}, NamedFormat{Format::Gif, "gif"},
    NamedFormat{Format::Bmp, "bmp"},   NamedFormat{Format::Tga, "tga"},
    NamedFormat{Format::Tiff, "tiff"}, NamedFormat{Format::Ico, "ico"},
    NamedFormat{Format::Svg, "svg"},   NamedFormat{Format::Obj, "obj"},
    NamedFormat{Format::Stl, "stl"},   NamedFormat{Format::Ply, "ply"},
    NamedFormat{Format::Gltf, "gltf"}, NamedFormat{Format::Glb, "glb"},
    NamedFormat{Format::Fbx, "fbx"},
};

bool starts_with(ByteView data, std::string_view prefix) {
  return data.size() >= prefix.size() &&
         std::memcmp(data.data(), prefix.data(), prefix.size()) == 0;
}

std::uint32_t le32(ByteView data, std::size_t at) {
  return std::uint32_t(data[at]) | std::uint32_t(data[at + 1]) << 8 |
         std::uint32_t(data[at + 2]) << 16 | std::uint32_t(data[at + 3]) << 24;
}

/// The first `limit` bytes as text, or empty if they contain NUL bytes.
std::string_view text_prefix(ByteView data, std::size_t limit = 4096) {
  const auto n = std::min(limit, data.size());
  const auto* begin = reinterpret_cast<const char*>(data.data());
  if (std::find(begin, begin + n, '\0') != begin + n) return {};
  return {begin, n};
}

std::string_view skip_space(std::string_view text) {
  // UTF-8 byte order mark, then whitespace.
  if (text.starts_with("\xEF\xBB\xBF")) text.remove_prefix(3);
  while (!text.empty() && (text.front() == ' ' || text.front() == '\t' ||
                           text.front() == '\r' || text.front() == '\n')) {
    text.remove_prefix(1);
  }
  return text;
}

bool looks_like_obj(std::string_view text) {
  // Every non-blank, non-comment line of the prefix must be an OBJ statement
  // (or the rest of one continued with a trailing "\").
  int records = 0;
  bool continued = false;
  while (!text.empty()) {
    const auto end = text.find('\n');
    auto line = skip_space(text.substr(0, end));
    text = end == std::string_view::npos ? std::string_view{} : text.substr(end + 1);
    const bool was_continued = continued;
    const auto content = line.substr(0, line.find('#'));
    const auto last = content.find_last_not_of(" \t\r");
    continued = last != std::string_view::npos && content[last] == '\\';
    if (was_continued || line.empty() || line.front() == '#') continue;
    if (end == std::string_view::npos) break;  // possibly cut off at the prefix limit
    // Every OBJ statement keyword (geometry, free-form, grouping, display).
    static constexpr std::array<std::string_view, 37> kKeywords{
        "v", "vt", "vn", "vp", "f", "l", "p", "o", "g", "s", "mg", "usemtl", "mtllib", "cstype", "deg",
        "bmat", "step", "curv", "curv2", "surf", "parm", "trim", "hole", "scrv", "sp", "end", "con",
        "lod", "usemap", "maplib", "shadow_obj", "trace_obj", "ctech", "stech", "bevel", "c_interp", "d_interp"};
    const auto keyword = line.substr(0, line.find_first_of(" \t"));
    if (keyword.size() == line.size() && keyword != "end") return false;  // a statement without arguments
    if (std::find(kKeywords.begin(), kKeywords.end(), keyword) == kKeywords.end()) return false;
    ++records;
  }
  return records > 0;
}

}  // namespace

Kind kind_of(Format format) {
  switch (format) {
    case Format::Png: case Format::Jpeg: case Format::WebP: case Format::Gif:
    case Format::Bmp: case Format::Tga: case Format::Tiff: case Format::Ico:
    case Format::Svg:
      return Kind::Image;
    case Format::Obj: case Format::Stl: case Format::Ply: case Format::Gltf:
    case Format::Glb: case Format::Fbx:
      return Kind::Mesh;
    case Format::Unknown:
      break;
  }
  return Kind::Unknown;
}

std::string_view format_name(Format format) {
  for (const auto& entry : kNames) {
    if (entry.format == format) return entry.name;
  }
  return "unknown";
}

Format format_from_name(std::string_view name) {
  if (name == "jpg") return Format::Jpeg;
  if (name == "tif") return Format::Tiff;
  for (const auto& entry : kNames) {
    if (entry.name == name) return entry.format;
  }
  return Format::Unknown;
}

Format detect(ByteView data) {
  if (data.size() < 4) return Format::Unknown;

  if (starts_with(data, "\x89PNG\r\n\x1A\n")) return Format::Png;
  if (data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF) return Format::Jpeg;
  if (data.size() >= 12 && starts_with(data, "RIFF") &&
      std::memcmp(data.data() + 8, "WEBP", 4) == 0) {
    return Format::WebP;
  }
  if (starts_with(data, "GIF87a") || starts_with(data, "GIF89a")) return Format::Gif;
  if (starts_with(data, "II*\0"sv) || starts_with(data, "MM\0*"sv)) return Format::Tiff;
  if (starts_with(data, "glTF")) return Format::Glb;
  if (starts_with(data, "ply\n") || starts_with(data, "ply\r\n")) return Format::Ply;
  if (starts_with(data, "Kaydara FBX Binary")) return Format::Fbx;

  if (data.size() >= 26 && starts_with(data, "BM")) {
    // DIB header sizes of BITMAPCOREHEADER .. BITMAPV5HEADER.
    const auto dib = le32(data, 14);
    if (dib == 12 || dib == 40 || dib == 52 || dib == 56 || dib == 64 || dib == 108 || dib == 124) {
      return Format::Bmp;
    }
  }

  if (data.size() >= 22 && data[0] == 0 && data[1] == 0 && data[2] == 1 && data[3] == 0 &&
      (data[4] | data[5] << 8) > 0) {
    return Format::Ico;
  }

  // Binary STL: a free-form 80-byte header (sometimes starting with "solid"),
  // so it is recognised by its exact size, 84 + 50 bytes per triangle.
  if (data.size() >= 84) {
    const std::uint64_t triangles = le32(data, 80);
    if (data.size() == 84 + triangles * 50) return Format::Stl;
  }

  if (auto text = skip_space(text_prefix(data)); !text.empty()) {
    if (text.starts_with("<?xml") || text.starts_with("<svg") || text.starts_with("<!DOCTYPE svg") ||
        text.starts_with("<!--")) {
      if (text.find("<svg") != std::string_view::npos) return Format::Svg;
    }
    if (text.starts_with("{")) {
      // The "asset" property can come after large embedded buffers, so search the whole file.
      const std::string_view whole(reinterpret_cast<const char*>(data.data()), data.size());
      if (whole.find("\"asset\"") != std::string_view::npos) return Format::Gltf;
    }
    if (text.starts_with("solid") && text.find("facet") != std::string_view::npos) return Format::Stl;
    if (text.starts_with("; FBX")) return Format::Fbx;
    if (looks_like_obj(text)) return Format::Obj;
  }

  // TGA has no signature: version 2 files end with a footer; otherwise check
  // the header fields are plausible.
  if (data.size() >= 44 &&
      std::memcmp(data.data() + data.size() - 18, "TRUEVISION-XFILE.", 17) == 0) {
    return Format::Tga;
  }
  if (data.size() >= 18) {
    const auto colormap = data[1], type = data[2], bpp = data[16];
    const auto width = data[12] | data[13] << 8, height = data[14] | data[15] << 8;
    const bool known_type = type == 1 || type == 2 || type == 3 || type == 9 || type == 10 || type == 11;
    const bool known_bpp = bpp == 8 || bpp == 15 || bpp == 16 || bpp == 24 || bpp == 32;
    if (colormap <= 1 && known_type && known_bpp && width > 0 && height > 0 &&
        (colormap == 1) == (type == 1 || type == 9)) {
      return Format::Tga;
    }
  }

  // Last: a binary STL whose triangle count field is wrong (some exporters
  // leave it 0) still has a size of 84 + 50 bytes per triangle.
  if (data.size() > 84 && (data.size() - 84) % 50 == 0) return Format::Stl;

  return Format::Unknown;
}

}  // namespace pixie
