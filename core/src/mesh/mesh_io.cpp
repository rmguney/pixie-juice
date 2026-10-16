// Mesh import/export. OBJ is parsed by fast_obj and PLY by miniply; STL is a
// fixed record layout read directly. Writers emit the same format back, with
// floats in their shortest exact form. A file that uses something a writer
// cannot reproduce (OBJ curves or several objects, extra PLY properties,
// double-precision coordinates) is read as inexact: optimizing passes it
// through, display shows its triangles.

#include "internal.hpp"

#include <fast_obj.h>
#include <miniply.h>

#include <algorithm>
#include <array>
#include <bit>
#include <charconv>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <memory>
#include <random>
#include <unordered_map>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>  // GetShortPathNameW
#endif

namespace pixie::detail {

namespace {

void append_float(std::string& out, float value) {
  char buffer[32];
  const auto result = std::to_chars(buffer, buffer + sizeof buffer, value);
  out.append(buffer, result.ptr);
}

void append_uint(std::string& out, std::uint64_t value) {
  char buffer[24];
  const auto result = std::to_chars(buffer, buffer + sizeof buffer, value);
  out.append(buffer, result.ptr);
}

template <std::size_t N>
struct FloatKey {
  std::array<std::uint32_t, N> bits{};
  bool operator==(const FloatKey&) const = default;
};

template <std::size_t N>
struct FloatKeyHash {
  std::size_t operator()(const FloatKey<N>& key) const {
    std::uint64_t h = 1469598103934665603ull;
    for (auto b : key.bits) h = (h ^ b) * 1099511628211ull;
    return std::size_t(h);
  }
};

/// Assigns 1-based indices to distinct float tuples, in first-seen order.
template <std::size_t N>
class Pool {
 public:
  std::uint32_t add(const float* values) {
    FloatKey<N> key;
    for (std::size_t i = 0; i < N; ++i) key.bits[i] = std::bit_cast<std::uint32_t>(values[i]);
    auto [it, inserted] = index_.try_emplace(key, std::uint32_t(order_.size() / N + 1));
    if (inserted) order_.insert(order_.end(), values, values + N);
    return it->second;
  }
  [[nodiscard]] const std::vector<float>& values() const { return order_; }

 private:
  std::unordered_map<FloatKey<N>, std::uint32_t, FloatKeyHash<N>> index_;
  std::vector<float> order_;
};

std::string_view as_text(ByteView data) { return {reinterpret_cast<const char*>(data.data()), data.size()}; }

/// Splits `text` into lines (without the newline) and calls `fn` for each.
template <class Fn>
void for_each_line(std::string_view text, Fn&& fn) {
  while (!text.empty()) {
    const auto end = text.find('\n');
    auto line = text.substr(0, end);
    if (!line.empty() && line.back() == '\r') line.remove_suffix(1);
    fn(line);
    if (end == std::string_view::npos) break;
    text.remove_prefix(end + 1);
  }
}

std::string_view trim(std::string_view s) {
  while (!s.empty() && (s.front() == ' ' || s.front() == '\t')) s.remove_prefix(1);
  while (!s.empty() && (s.back() == ' ' || s.back() == '\t')) s.remove_suffix(1);
  return s;
}

std::array<float, 3> face_normal(const float* a, const float* b, const float* c) {
  const float ux = b[0] - a[0], uy = b[1] - a[1], uz = b[2] - a[2];
  const float vx = c[0] - a[0], vy = c[1] - a[1], vz = c[2] - a[2];
  float nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
  const float length = std::sqrt(nx * nx + ny * ny + nz * nz);
  if (length > 0) {
    nx /= length;
    ny /= length;
    nz /= length;
  }
  return {nx, ny, nz};
}

// ---------------------------------------------------------------------------
// OBJ

struct MemoryObj {
  ByteView data;
  std::size_t pos = 0;
  bool opened = false;
};

void* obj_open(const char*, void* user) {
  // The first open is the model itself; later ones are .mtl libraries, which aren't available.
  auto* file = static_cast<MemoryObj*>(user);
  if (file->opened) return nullptr;
  file->opened = true;
  return file;
}
void obj_close(void*, void*) {}
std::size_t obj_read(void* file, void* dst, std::size_t bytes, void*) {
  auto* f = static_cast<MemoryObj*>(file);
  const auto n = std::min(bytes, f->data.size() - f->pos);
  std::memcpy(dst, f->data.data() + f->pos, n);
  f->pos += n;
  return n;
}
unsigned long obj_size(void* file, void*) { return static_cast<unsigned long>(static_cast<MemoryObj*>(file)->data.size()); }

/// What a text pass over the OBJ establishes before parsing.
struct ObjScan {
  /// False when the file uses something the writer can't reproduce.
  bool exact = true;
  /// Every "v" line carries an RGB colour after the position.
  bool colors = false;
  /// A comment after a statement or a "\" line continuation, which fast_obj misreads.
  bool needs_cleaning = false;
  /// Faces that come before the first "usemtl" (they have no material).
  std::size_t faces_without_material = 0;
};

/// The statement's keyword and the rest of the line; fast_obj accepts tabs as well as spaces after it.
std::pair<std::string_view, std::string_view> split_keyword(std::string_view line) {
  const auto end = std::min(line.find_first_of(" \t"), line.size());
  return {line.substr(0, end), trim(line.substr(end))};
}

/// Every value is a finite number fast_obj reads back as written: it reads an
/// exponent of 20 or more (either sign) as 0, and "inf" or "nan" as 0.
bool exact_numbers(std::string_view values) {
  for (auto remaining = values; !remaining.empty();) {
    const auto end = std::min(remaining.find_first_of(" \t"), remaining.size());
    auto token = remaining.substr(0, end);
    remaining = trim(remaining.substr(end));
    if (token.starts_with('+')) token.remove_prefix(1);
    double value = 0;
    const auto parsed = std::from_chars(token.data(), token.data() + token.size(), value);
    if (parsed.ec != std::errc() || parsed.ptr != token.data() + token.size() || !std::isfinite(value)) return false;
    if (const auto e = token.find_first_of("eE"); e != std::string_view::npos) {
      auto digits = token.substr(e + 1);
      if (digits.starts_with('+')) digits.remove_prefix(1);
      int exponent = 0;
      std::from_chars(digits.data(), digits.data() + digits.size(), exponent);
      if (exponent >= 20 || exponent <= -20) return false;
    }
  }
  return true;
}

ObjScan scan_obj(std::string_view text) {
  // Statements the writer doesn't reproduce: free-form geometry, points, lines, display attributes.
  static constexpr std::array<std::string_view, 29> kUnsupported{
      "l", "p", "vp", "curv", "curv2", "surf", "cstype", "deg", "bmat", "step", "parm", "trim", "hole", "scrv", "sp",
      "end", "con", "mg", "lod", "usemap", "maplib", "shadow_obj", "trace_obj", "ctech", "stech", "bevel", "c_interp",
      "d_interp", "res"};
  ObjScan scan;
  int objects = 0, position_width = 0;
  bool has_normals = false, smoothing = false, material_seen = false, mixed_widths = false;
  for_each_line(text, [&](std::string_view raw) {
    const auto comment = raw.find('#');
    const auto line = trim(raw.substr(0, comment));
    if (line.empty()) return;
    if (comment != std::string_view::npos || line.back() == '\\') scan.needs_cleaning = true;
    const auto [keyword, rest] = split_keyword(line);
    if (std::find(kUnsupported.begin(), kUnsupported.end(), keyword) != kUnsupported.end()) scan.exact = false;
    if (keyword == "o" || keyword == "g") ++objects;
    if (keyword == "vn") has_normals = true;
    if (keyword == "s" && rest != "off" && rest != "0") smoothing = true;
    if (keyword == "usemtl") material_seen = true;
    if (keyword == "f" && !material_seen) ++scan.faces_without_material;
    if ((keyword == "v" || keyword == "vt" || keyword == "vn") && !exact_numbers(rest)) scan.exact = false;
    if (keyword == "v") {
      // xyz, or xyz plus RGB; a "w" weight or anything else isn't reproduced.
      int values = 0;
      for (auto remaining = rest; !remaining.empty(); ++values) {
        remaining = trim(remaining.substr(std::min(remaining.find_first_of(" \t"), remaining.size())));
      }
      if ((values != 3 && values != 6) || (position_width != 0 && values != position_width)) {
        scan.exact = false;
        mixed_widths = true;
      }
      position_width = values;
    }
  });
  // Without explicit normals, smoothing groups decide shading; they aren't reproduced.
  if (objects > 1 || (smoothing && !has_normals)) scan.exact = false;
  scan.colors = position_width == 6 && !mixed_widths;
  return scan;
}

/// The OBJ text with comments cut and "\" continuations joined. fast_obj stops
/// a face at either and keeps the corners it read without the face that owns
/// them, which shifts every later triangle.
std::string clean_obj(std::string_view text) {
  std::string out;
  out.reserve(text.size());
  for_each_line(text, [&](std::string_view raw) {
    auto line = raw.substr(0, raw.find('#'));
    while (!line.empty() && (line.back() == ' ' || line.back() == '\t')) line.remove_suffix(1);
    if (!line.empty() && line.back() == '\\') {
      line.remove_suffix(1);
      out.append(line);
      out.push_back(' ');  // joined with the next line
      return;
    }
    out.append(line);
    out.push_back('\n');
  });
  return out;
}

/// Bytes the smallest possible row of an element takes, so a declared row
/// count can be checked against the file before miniply allocates for it.
std::uint64_t min_row_size(const miniply::PLYElement& element, bool binary) {
  using miniply::PLYPropertyType;
  auto size = [](PLYPropertyType type) -> std::uint64_t {
    switch (type) {
      case PLYPropertyType::Char: case PLYPropertyType::UChar: return 1;
      case PLYPropertyType::Short: case PLYPropertyType::UShort: return 2;
      case PLYPropertyType::Int: case PLYPropertyType::UInt: case PLYPropertyType::Float: return 4;
      case PLYPropertyType::Double: return 8;
      case PLYPropertyType::None: break;
    }
    return 0;
  };
  std::uint64_t bytes = 0;
  for (const auto& property : element.properties) {
    // ASCII: at least one digit and a separator per value; binary: a list's count, or the value.
    bytes += binary ? size(property.countType != PLYPropertyType::None ? property.countType : property.type) : 2;
  }
  return std::max<std::uint64_t>(bytes, 1);
}

/// "comment" and "obj_info" header lines (such as "comment TextureFile wood.png"),
/// carried into the rewritten header. Our own marker is left out, as the writer adds it.
std::vector<std::string> ply_comments(ByteView data) {
  std::vector<std::string> comments;
  const auto text = as_text(data);
  for_each_line(text.substr(0, text.find("end_header")), [&](std::string_view raw) {
    const auto line = trim(raw);
    if ((line.starts_with("comment ") || line.starts_with("obj_info ")) && line != "comment Optimized by Pixie Juice") {
      comments.emplace_back(line);
    }
  });
  return comments;
}

/// A read in progress. inexact() marks it and says whether to stop reading.
struct Reading {
  MeshUse use;
  ReadMesh read{};

  bool inexact() {
    read.exact = false;
    return use == MeshUse::Optimize;
  }

  /// Also inexact: nothing to optimize, or values the writers can't hold.
  ReadMesh finish() {
    const Mesh& m = read.mesh;
    if (m.indices.empty() || !all_finite(m.positions) || !all_finite(m.normals) || !all_finite(m.uvs) ||
        !all_finite(m.colors)) {
      read.exact = false;
    }
    return std::move(read);
  }
};

}  // namespace

bool all_finite(std::span<const float> values) {
  return std::all_of(values.begin(), values.end(), [](float v) { return std::isfinite(v); });
}

// ---------------------------------------------------------------------------
// Temporary files

namespace {

/// The temp directory as a path fopen (and the libraries given it) can open.
std::filesystem::path temp_directory() {
  std::error_code ec;
  auto dir = std::filesystem::temp_directory_path(ec);
  if (ec) return ".";
#ifdef _WIN32
  // Narrow paths are in the ANSI code page, which may not hold the user's name
  // (path::string() throws then); the short 8.3 form of the path is plain ASCII.
  try {
    (void)dir.string();
    return dir;
  } catch (const std::exception&) {
  }
  std::wstring shortened(MAX_PATH, L'\0');
  const DWORD length = GetShortPathNameW(dir.c_str(), shortened.data(), DWORD(shortened.size()));
  if (length == 0 || length >= shortened.size()) return ".";
  shortened.resize(length);
  return shortened;
#else
  return dir;
#endif
}

}  // namespace

TempFile::TempFile(std::string_view extension) {
  // A random name, created exclusively: concurrent calls and processes never
  // share a file, and nothing else can claim the name first. On failure the
  // path stays empty and every use of it fails.
  const auto dir = temp_directory();
  std::random_device entropy;
  for (int attempt = 0; attempt < 8 && path_.empty(); ++attempt) {
    const std::uint64_t id = std::uint64_t(entropy()) << 32 | entropy();
    auto path = (dir / ("pixie-" + std::to_string(id) + std::string(extension))).string();
    if (std::FILE* file = std::fopen(path.c_str(), "wbx")) {
      std::fclose(file);
      path_ = std::move(path);
    }
  }
}

TempFile::~TempFile() {
  std::error_code ec;
  std::filesystem::remove(path_, ec);
}

bool TempFile::write(ByteView data) const {
  std::FILE* file = std::fopen(path_.c_str(), "wb");
  if (!file) return false;
  const bool ok = std::fwrite(data.data(), 1, data.size(), file) == data.size();
  return std::fclose(file) == 0 && ok;
}

std::optional<Bytes> TempFile::read() const {
  std::FILE* file = std::fopen(path_.c_str(), "rb");
  if (!file) return std::nullopt;
  Bytes out;
  std::uint8_t buffer[65536];
  std::size_t n;
  while ((n = std::fread(buffer, 1, sizeof buffer, file)) > 0) out.insert(out.end(), buffer, buffer + n);
  std::fclose(file);
  return out;
}

// ---------------------------------------------------------------------------
// OBJ

std::string obj_header(ByteView source) {
  std::string header = "# Optimized by Pixie Juice\n";
  for_each_line(as_text(source), [&](std::string_view raw) {
    const auto line = trim(raw);
    const auto keyword = split_keyword(line).first;
    if (keyword == "mtllib" || keyword == "o" || keyword == "g") {
      header.append(line);
      header.push_back('\n');
    }
  });
  return header;
}

Result<ReadMesh> read_obj(ByteView data, MeshUse use) {
  Reading reading{use};
  std::string_view text = as_text(data);
  ObjScan scan = scan_obj(text);
  std::string cleaned;
  if (scan.needs_cleaning) {
    cleaned = clean_obj(text);
    text = cleaned;
    scan = scan_obj(text);
  }
  if (!scan.exact && reading.inexact()) return std::move(reading.read);

  MemoryObj file{ByteView(reinterpret_cast<const std::uint8_t*>(text.data()), text.size())};
  const fastObjCallbacks callbacks{obj_open, obj_close, obj_read, obj_size};
  const std::unique_ptr<fastObjMesh, decltype(&fast_obj_destroy)> owner(
      fast_obj_read_with_callbacks("model.obj", &callbacks, &file), fast_obj_destroy);
  const fastObjMesh* obj = owner.get();
  if (!obj) return fail("OBJ parse failed");
  if (obj->face_count == 0) return reading.finish();  // a point cloud: no triangles

  // An index 0 (invalid OBJ) still stops fast_obj mid-face: every corner must belong to a face.
  std::size_t corners = 0;
  for (unsigned f = 0; f < obj->face_count; ++f) corners += obj->face_vertices[f];
  if (corners != obj->index_count) {
    if (use == MeshUse::Display) return fail("OBJ face with a vertex index of 0");
    reading.inexact();
    return std::move(reading.read);
  }

  // fast_obj doesn't range-check face references; index 0 is its "missing" slot.
  unsigned with_uv = 0, with_normal = 0;
  for (unsigned i = 0; i < obj->index_count; ++i) {
    const fastObjIndex& idx = obj->indices[i];
    if (idx.p == 0 || idx.p >= obj->position_count || idx.t >= obj->texcoord_count ||
        idx.n >= obj->normal_count) {
      return fail("OBJ face references a vertex that doesn't exist");
    }
    with_uv += idx.t != 0;
    with_normal += idx.n != 0;
  }
  // Corners that mix "with" and "without" texture coordinates or normals can't
  // be written back as they were; display drops the partial attribute.
  const bool any_uv = with_uv == obj->index_count, any_normal = with_normal == obj->index_count;
  if (((with_uv > 0 && !any_uv) || (with_normal > 0 && !any_normal)) && reading.inexact()) {
    return std::move(reading.read);
  }
  const bool colors = scan.colors && obj->color_count == obj->position_count;
  if (scan.colors && !colors && reading.inexact()) return std::move(reading.read);

  Mesh& mesh = reading.read.mesh;
  // Faces before the first "usemtl" have no material: group 0, written first, without a usemtl line.
  const bool grouped = obj->material_count > 0;
  const std::uint32_t first_material = scan.faces_without_material > 0 ? 1 : 0;
  if (grouped && first_material) mesh.group_names.emplace_back();
  for (unsigned m = 0; m < obj->material_count; ++m) {
    mesh.group_names.emplace_back(obj->materials[m].name ? obj->materials[m].name : "");
  }
  std::size_t base = 0;
  for (unsigned f = 0; f < obj->face_count; ++f) {
    const unsigned count = obj->face_vertices[f];
    if (grouped && obj->face_materials[f] >= obj->material_count) return fail("Inconsistent OBJ face data");
    // Polylines ("l") have no surface; files with them are already inexact, and display skips them.
    if (obj->face_lines && obj->face_lines[f]) {
      base += count;
      continue;
    }
    for (unsigned k = 1; k + 1 < count; ++k) {
      for (const std::size_t corner : {base, base + k, base + k + 1}) {
        const fastObjIndex& idx = obj->indices[corner];
        mesh.indices.push_back(std::uint32_t(mesh.vertex_count()));
        mesh.positions.insert(mesh.positions.end(), obj->positions + idx.p * 3, obj->positions + idx.p * 3 + 3);
        if (colors) mesh.colors.insert(mesh.colors.end(), obj->colors + idx.p * 3, obj->colors + idx.p * 3 + 3);
        if (any_uv) mesh.uvs.insert(mesh.uvs.end(), obj->texcoords + idx.t * 2, obj->texcoords + idx.t * 2 + 2);
        if (any_normal) mesh.normals.insert(mesh.normals.end(), obj->normals + idx.n * 3, obj->normals + idx.n * 3 + 3);
      }
      if (grouped) {
        mesh.triangle_groups.push_back(f < scan.faces_without_material ? 0 : obj->face_materials[f] + first_material);
      }
    }
    base += count;
  }
  return reading.finish();
}

Bytes write_obj(const Mesh& mesh, std::string_view header) {
  Pool<6> positions;  // position + colour, which OBJ stores on the same "v" line
  Pool<2> uvs;
  Pool<3> normals;
  const bool colors = !mesh.colors.empty();
  std::vector<std::array<std::uint32_t, 3>> corners(mesh.indices.size());
  for (std::size_t i = 0; i < mesh.indices.size(); ++i) {
    const auto v = mesh.indices[i];
    float pos[6] = {mesh.positions[v * 3], mesh.positions[v * 3 + 1], mesh.positions[v * 3 + 2], 0, 0, 0};
    if (colors) std::memcpy(pos + 3, &mesh.colors[v * 3], sizeof(float) * 3);
    corners[i][0] = positions.add(pos);
    corners[i][1] = mesh.uvs.empty() ? 0 : uvs.add(&mesh.uvs[v * 2]);
    corners[i][2] = mesh.normals.empty() ? 0 : normals.add(&mesh.normals[v * 3]);
  }

  std::string out(header);
  const auto& p = positions.values();
  for (std::size_t i = 0; i < p.size(); i += 6) {
    out += "v ";
    append_float(out, p[i]);
    for (std::size_t k = 1; k < (colors ? 6u : 3u); ++k) {
      out.push_back(' ');
      append_float(out, p[i + k]);
    }
    out.push_back('\n');
  }
  for (std::size_t i = 0; i < uvs.values().size(); i += 2) {
    out += "vt ";
    append_float(out, uvs.values()[i]);
    out.push_back(' ');
    append_float(out, uvs.values()[i + 1]);
    out.push_back('\n');
  }
  for (std::size_t i = 0; i < normals.values().size(); i += 3) {
    out += "vn ";
    for (std::size_t k = 0; k < 3; ++k) {
      if (k) out.push_back(' ');
      append_float(out, normals.values()[i + k]);
    }
    out.push_back('\n');
  }

  std::int64_t current_group = -1;
  for (std::size_t t = 0; t < mesh.triangle_count(); ++t) {
    if (!mesh.triangle_groups.empty() && std::int64_t(mesh.triangle_groups[t]) != current_group) {
      current_group = mesh.triangle_groups[t];
      if (const auto& name = mesh.group_names[std::size_t(current_group)]; !name.empty()) {
        out += "usemtl ";
        out += name;
        out.push_back('\n');
      }
    }
    out += "f";
    for (std::size_t k = 0; k < 3; ++k) {
      const auto& c = corners[t * 3 + k];
      out.push_back(' ');
      append_uint(out, c[0]);
      if (c[1] || c[2]) {
        out.push_back('/');
        if (c[1]) append_uint(out, c[1]);
        if (c[2]) {
          out.push_back('/');
          append_uint(out, c[2]);
        }
      }
    }
    out.push_back('\n');
  }
  return Bytes(out.begin(), out.end());
}

// ---------------------------------------------------------------------------
// STL

Result<ReadMesh> read_stl(ByteView data, MeshUse use, StlInfo* info) {
  Reading reading{use};
  Mesh& mesh = reading.read.mesh;
  std::uint32_t declared = 0;
  if (data.size() >= 84) std::memcpy(&declared, data.data() + 80, 4);
  // Binary when the size matches the declared triangle count, or when the file
  // isn't ASCII STL and its size fits whole records (some exporters write a wrong count).
  const auto text = trim(as_text(data.first(std::min<std::size_t>(data.size(), 1024))));
  const bool ascii = text.starts_with("solid") && text.find("facet") != std::string_view::npos;
  const bool binary = data.size() >= 84 && (data.size() == 84 + std::uint64_t(declared) * 50 ||
                                            (!ascii && data.size() > 84 && (data.size() - 84) % 50 == 0));
  if (info) info->binary = binary;
  if (binary) {
    const std::size_t triangles = (data.size() - 84) / 50;
    mesh.positions.resize(triangles * 9);
    for (std::size_t t = 0; t < triangles; ++t) {
      const auto* record = data.data() + 84 + t * 50;
      // Non-zero attribute bytes hold per-facet colours (VisCAM/Materialise), which the writer drops.
      if ((record[48] != 0 || record[49] != 0) && reading.inexact()) return std::move(reading.read);
      std::memcpy(&mesh.positions[t * 9], record + 12, sizeof(float) * 9);
    }
  } else {
    bool first = true;
    bool ok = true;
    int solids = 0;
    for_each_line(as_text(data), [&](std::string_view raw) {
      const auto line = trim(raw);
      if (line.starts_with("solid")) ++solids;
      if (first && line.starts_with("solid")) {
        if (info) info->name = std::string(trim(line.substr(5)));
        first = false;
        return;
      }
      first = false;
      if (!ok || !line.starts_with("vertex")) return;
      auto rest = trim(line.substr(6));
      for (int k = 0; k < 3; ++k) {
        float value = 0;
        const auto result = std::from_chars(rest.data(), rest.data() + rest.size(), value);
        if (result.ec != std::errc()) {
          ok = false;
          return;
        }
        mesh.positions.push_back(value);
        rest = trim(rest.substr(std::size_t(result.ptr - rest.data())));
      }
    });
    if (!ok || mesh.positions.size() % 9 != 0) return fail("Invalid ASCII STL");
    // The writer emits one solid; several would be merged under the first name.
    if (solids > 1 && reading.inexact()) return std::move(reading.read);
  }
  mesh.indices.resize(mesh.vertex_count());
  for (std::size_t i = 0; i < mesh.indices.size(); ++i) mesh.indices[i] = std::uint32_t(i);
  return reading.finish();
}

Bytes write_stl(const Mesh& mesh, const StlInfo& info) {
  const std::size_t triangles = mesh.triangle_count();
  auto vertex = [&](std::size_t t, int k) { return &mesh.positions[mesh.indices[t * 3 + k] * 3]; };
  if (info.binary) {
    Bytes out(84 + triangles * 50, 0);
    static constexpr char kHeader[] = "Optimized by Pixie Juice";
    std::memcpy(out.data(), kHeader, sizeof kHeader - 1);
    const auto count = std::uint32_t(triangles);
    std::memcpy(out.data() + 80, &count, 4);
    for (std::size_t t = 0; t < triangles; ++t) {
      auto* record = out.data() + 84 + t * 50;
      const auto n = face_normal(vertex(t, 0), vertex(t, 1), vertex(t, 2));
      std::memcpy(record, n.data(), 12);
      for (int k = 0; k < 3; ++k) std::memcpy(record + 12 + k * 12, vertex(t, k), 12);
    }
    return out;
  }
  std::string out = "solid " + info.name + "\n";
  for (std::size_t t = 0; t < triangles; ++t) {
    const auto n = face_normal(vertex(t, 0), vertex(t, 1), vertex(t, 2));
    out += "facet normal ";
    for (int k = 0; k < 3; ++k) {
      if (k) out.push_back(' ');
      append_float(out, n[std::size_t(k)]);
    }
    out += "\nouter loop\n";
    for (int k = 0; k < 3; ++k) {
      out += "vertex ";
      for (int c = 0; c < 3; ++c) {
        if (c) out.push_back(' ');
        append_float(out, vertex(t, k)[c]);
      }
      out.push_back('\n');
    }
    out += "endloop\nendfacet\n";
  }
  out += "endsolid " + info.name + "\n";
  return Bytes(out.begin(), out.end());
}

// ---------------------------------------------------------------------------
// PLY

Result<ReadMesh> read_ply(ByteView data, MeshUse use, PlyInfo* info) {
  TempFile file(".ply");
  if (!file.write(data)) return fail("Could not stage PLY input");
  miniply::PLYReader reader(file.path().c_str());
  if (!reader.valid()) return fail("Invalid PLY");
  const bool binary = reader.file_type() != miniply::PLYFileType::ASCII;
  if (info) {
    info->binary = binary;
    info->comments = ply_comments(data);
  }

  using miniply::PLYPropertyType;
  Reading reading{use};
  Mesh& mesh = reading.read.mesh;
  bool vertices_read = false, faces_read = false;
  for (; reader.has_element(); reader.next_element()) {
    const auto* element = reader.element();
    // miniply allocates the whole element up front: its declared rows must fit in the file.
    if (std::uint64_t(element->count) * min_row_size(*element, binary) > data.size()) {
      return fail("PLY declares more data than the file holds");
    }
    if (reader.element_is(miniply::kPLYVertexElement) && !vertices_read) {
      if (!reader.load_element()) return fail("Invalid PLY vertex data");
      const std::uint32_t rows = reader.num_rows();
      std::uint32_t pos[3], nrm[3], uv[2], col[3];
      if (!reader.find_pos(pos)) return fail("PLY has no vertex positions");
      std::size_t handled = 3;
      auto all_float = [&](const std::uint32_t* idx, int n) {
        for (int i = 0; i < n; ++i) {
          if (element->properties[idx[i]].type != PLYPropertyType::Float) return false;
        }
        return true;
      };
      // miniply extracts list properties as if they were scalars, reading past its row buffer.
      auto scalar = [&](const std::uint32_t* idx, int n) {
        for (int i = 0; i < n; ++i) {
          if (element->properties[idx[i]].countType != PLYPropertyType::None) return false;
        }
        return true;
      };
      if (!scalar(pos, 3) || (reader.find_normal(nrm) && !scalar(nrm, 3)) ||
          (reader.find_texcoord(uv) && !scalar(uv, 2)) || (reader.find_color(col) && !scalar(col, 3))) {
        return fail("PLY vertex attributes declared as lists");
      }
      // Doubles would lose precision in the float writer.
      if (!all_float(pos, 3) && reading.inexact()) return std::move(reading.read);
      mesh.positions.resize(std::size_t(rows) * 3);
      reader.extract_properties(pos, 3, PLYPropertyType::Float, mesh.positions.data());
      if (reader.find_normal(nrm)) {
        if (!all_float(nrm, 3) && reading.inexact()) return std::move(reading.read);
        mesh.normals.resize(std::size_t(rows) * 3);
        reader.extract_properties(nrm, 3, PLYPropertyType::Float, mesh.normals.data());
        handled += 3;
      }
      if (reader.find_texcoord(uv)) {
        if (!all_float(uv, 2) && reading.inexact()) return std::move(reading.read);
        mesh.uvs.resize(std::size_t(rows) * 2);
        reader.extract_properties(uv, 2, PLYPropertyType::Float, mesh.uvs.data());
        if (info) {
          info->uv_names = {element->properties[uv[0]].name, element->properties[uv[1]].name};
        }
        handled += 2;
      }
      // Colours other than 8-bit or float have no agreed range, so they are neither read nor reproduced.
      if (reader.find_color(col)) {
        const auto type = element->properties[col[0]].type;
        if (type == PLYPropertyType::UChar || type == PLYPropertyType::Float) {
          mesh.colors.resize(std::size_t(rows) * 3);
          reader.extract_properties(col, 3, PLYPropertyType::Float, mesh.colors.data());
          if (type == PLYPropertyType::UChar) {
            for (auto& c : mesh.colors) c /= 255.0f;
          }
          if (info) {
            info->color_uchar = type == PLYPropertyType::UChar;
            info->color_names = {element->properties[col[0]].name, element->properties[col[1]].name,
                                 element->properties[col[2]].name};
          }
          handled += 3;
        } else if (reading.inexact()) {
          return std::move(reading.read);
        }
      }
      // Any other vertex property (alpha, quality, confidence...) would be dropped.
      if (element->properties.size() != handled && reading.inexact()) return std::move(reading.read);
      vertices_read = true;
    } else if (reader.element_is(miniply::kPLYFaceElement) && vertices_read && !faces_read) {
      if (!reader.load_element()) return fail("Invalid PLY face data");
      std::uint32_t idx[1];
      if (!reader.find_indices(idx)) {
        if (reading.inexact()) return std::move(reading.read);
        continue;
      }
      // Other face properties (colours, flags) would be dropped.
      if (element->properties.size() != 1 && reading.inexact()) return std::move(reading.read);
      const std::uint32_t faces = reader.num_rows();
      const std::uint32_t* counts = reader.get_list_counts(idx[0]);
      std::vector<std::uint32_t> corners(reader.sum_of_list_counts(idx[0]));
      faces_read = true;
      if (corners.empty()) continue;  // no faces (and nothing to extract into)
      if (!counts || !reader.extract_list_property(idx[0], PLYPropertyType::UInt, corners.data())) {
        return fail("Invalid PLY faces");
      }
      std::size_t base = 0;
      for (std::uint32_t f = 0; f < faces; ++f) {
        const std::uint32_t count = counts[f];
        if (base + count > corners.size()) return fail("Invalid PLY faces");
        for (std::uint32_t k = 1; k + 1 < count; ++k) {
          mesh.indices.insert(mesh.indices.end(), {corners[base], corners[base + k], corners[base + k + 1]});
        }
        base += count;
      }
      for (auto i : mesh.indices) {
        if (i >= mesh.vertex_count()) return fail("PLY face index out of range");
      }
    } else if (reading.inexact()) {  // edges, materials, extra elements: not reproduced
      return std::move(reading.read);
    }
  }
  return reading.finish();
}

Bytes write_ply(const Mesh& mesh, const PlyInfo& info) {
  std::string header = "ply\nformat ";
  header += info.binary ? "binary_little_endian 1.0\n" : "ascii 1.0\n";
  for (const auto& comment : info.comments) {
    header += comment;
    header.push_back('\n');
  }
  header += "comment Optimized by Pixie Juice\nelement vertex ";
  append_uint(header, mesh.vertex_count());
  header += "\nproperty float x\nproperty float y\nproperty float z\n";
  if (!mesh.normals.empty()) header += "property float nx\nproperty float ny\nproperty float nz\n";
  if (!mesh.uvs.empty()) {
    header += "property float " + info.uv_names[0] + "\nproperty float " + info.uv_names[1] + "\n";
  }
  const char* color_type = info.color_uchar ? "uchar" : "float";
  if (!mesh.colors.empty()) {
    for (const auto& name : info.color_names) header += std::string("property ") + color_type + " " + name + "\n";
  }
  header += "element face ";
  append_uint(header, mesh.triangle_count());
  header += "\nproperty list uchar uint vertex_indices\nend_header\n";

  Bytes out(header.begin(), header.end());
  auto put = [&](const void* p, std::size_t n) {
    const auto* b = static_cast<const std::uint8_t*>(p);
    out.insert(out.end(), b, b + n);
  };
  std::string text;
  std::vector<float> floats;
  floats.reserve(11);
  for (std::size_t v = 0; v < mesh.vertex_count(); ++v) {
    floats.assign(&mesh.positions[v * 3], &mesh.positions[v * 3] + 3);
    if (!mesh.normals.empty()) floats.insert(floats.end(), &mesh.normals[v * 3], &mesh.normals[v * 3] + 3);
    if (!mesh.uvs.empty()) floats.insert(floats.end(), &mesh.uvs[v * 2], &mesh.uvs[v * 2] + 2);
    std::array<std::uint8_t, 3> rgb{};
    if (!mesh.colors.empty()) {
      for (int c = 0; c < 3; ++c) {
        const float value = mesh.colors[v * 3 + std::size_t(c)];
        if (info.color_uchar) {
          rgb[std::size_t(c)] = std::uint8_t(std::lround(std::clamp(value, 0.0f, 1.0f) * 255.0f));
        } else {
          floats.push_back(value);
        }
      }
    }
    if (info.binary) {
      put(floats.data(), floats.size() * sizeof(float));
      if (!mesh.colors.empty() && info.color_uchar) put(rgb.data(), 3);
    } else {
      for (std::size_t i = 0; i < floats.size(); ++i) {
        if (i) text.push_back(' ');
        append_float(text, floats[i]);
      }
      if (!mesh.colors.empty() && info.color_uchar) {
        for (auto c : rgb) {
          text.push_back(' ');
          append_uint(text, c);
        }
      }
      text.push_back('\n');
    }
  }
  for (std::size_t t = 0; t < mesh.triangle_count(); ++t) {
    if (info.binary) {
      const std::uint8_t three = 3;
      put(&three, 1);
      put(&mesh.indices[t * 3], sizeof(std::uint32_t) * 3);
    } else {
      text += "3";
      for (int k = 0; k < 3; ++k) {
        text.push_back(' ');
        append_uint(text, mesh.indices[t * 3 + std::size_t(k)]);
      }
      text.push_back('\n');
    }
  }
  out.insert(out.end(), text.begin(), text.end());
  return out;
}

}  // namespace pixie::detail
