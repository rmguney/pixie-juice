#pragma once

// Deterministic test inputs, encoded with the same libraries the core wraps.

#include "internal.hpp"

#include <cstring>
#include <ostream>
#include <string>

namespace fixtures {

using pixie::Bytes;
using pixie::detail::Image;

/// Gradient plus deterministic noise; `alpha` adds a transparent checker.
inline Image make_image(std::uint32_t width, std::uint32_t height, int noise, bool alpha = false) {
  Image image;
  image.width = width;
  image.height = height;
  image.rgba.resize(image.pixel_count() * 4);
  std::uint32_t seed = 1234567;
  for (std::uint32_t y = 0; y < height; ++y) {
    for (std::uint32_t x = 0; x < width; ++x) {
      seed ^= seed << 13;
      seed ^= seed >> 17;
      seed ^= seed << 5;
      const int n = noise ? int(seed % unsigned(noise * 2 + 1)) - noise : 0;
      auto clamp = [](int v) { return std::uint8_t(v < 0 ? 0 : v > 255 ? 255 : v); };
      auto* px = &image.rgba[(std::size_t(y) * width + x) * 4];
      px[0] = clamp(int(x * 255 / width) + n);
      px[1] = clamp(int(y * 200 / height) + 30 + n);
      px[2] = clamp(220 - int(x * 180 / width) + n);
      px[3] = alpha ? std::uint8_t(((x / 8 + y / 8) % 3 == 0) ? 0 : 160 + (x + y) % 96) : 255;
    }
  }
  return image;
}

/// Few distinct colours, so palette encodings apply.
inline Image make_flat_image(std::uint32_t width, std::uint32_t height) {
  Image image;
  image.width = width;
  image.height = height;
  image.rgba.resize(image.pixel_count() * 4);
  for (std::size_t i = 0; i < image.pixel_count(); ++i) {
    const std::uint8_t c = std::uint8_t(((i % width) / 16 + (i / width) / 16) % 4 * 60);
    image.rgba[i * 4] = c;
    image.rgba[i * 4 + 1] = std::uint8_t(255 - c);
    image.rgba[i * 4 + 2] = 40;
    image.rgba[i * 4 + 3] = 255;
  }
  return image;
}

inline Bytes text_bytes(std::string_view text) { return Bytes(text.begin(), text.end()); }

inline std::size_t count(std::string_view haystack, std::string_view needle) {
  std::size_t n = 0;
  for (auto pos = haystack.find(needle); pos != std::string_view::npos; pos = haystack.find(needle, pos + 1)) ++n;
  return n;
}

inline std::string_view as_text(const Bytes& b) { return {reinterpret_cast<const char*>(b.data()), b.size()}; }

/// Lines of a text file that start with `prefix`.
inline std::size_t lines_starting(const Bytes& data, std::string_view prefix) {
  std::size_t n = 0;
  const auto text = as_text(data);
  for (std::size_t pos = 0; pos < text.size();) {
    if (text.substr(pos).starts_with(prefix)) ++n;
    const auto end = text.find('\n', pos);
    if (end == std::string_view::npos) break;
    pos = end + 1;
  }
  return n;
}

/// An n x n grid of quads (2 n^2 triangles), each triangle with its own corner copies.
inline std::string grid_obj(int n) {
  std::string text = "# grid\no grid\n";
  int count = 0;
  for (int y = 0; y < n; ++y) {
    for (int x = 0; x < n; ++x) {
      const float x0 = float(x), y0 = float(y), x1 = x0 + 1, y1 = y0 + 1;
      const float corners[6][2] = {{x0, y0}, {x1, y0}, {x1, y1}, {x0, y0}, {x1, y1}, {x0, y1}};
      for (const auto& c : corners) text += "v " + std::to_string(c[0]) + " " + std::to_string(c[1]) + " 0.000000\n";
      text += "f " + std::to_string(count + 1) + " " + std::to_string(count + 2) + " " + std::to_string(count + 3) + "\n";
      text += "f " + std::to_string(count + 4) + " " + std::to_string(count + 5) + " " + std::to_string(count + 6) + "\n";
      count += 6;
    }
  }
  return text;
}

/// A glTF with one triangle-list primitive (the grid), its buffer as a base64 data URI.
inline Bytes grid_gltf(int n) {
  auto parsed = pixie::detail::read_obj(text_bytes(grid_obj(n)), pixie::detail::MeshUse::Optimize);
  const auto& mesh = parsed->mesh;
  std::string buffer;
  buffer.append(reinterpret_cast<const char*>(mesh.positions.data()), mesh.positions.size() * 4);
  buffer.append(reinterpret_cast<const char*>(mesh.indices.data()), mesh.indices.size() * 4);
  static constexpr char kAlphabet[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  std::string base64;
  for (std::size_t i = 0; i < buffer.size(); i += 3) {
    std::uint32_t v = std::uint8_t(buffer[i]) << 16;
    if (i + 1 < buffer.size()) v |= std::uint8_t(buffer[i + 1]) << 8;
    if (i + 2 < buffer.size()) v |= std::uint8_t(buffer[i + 2]);
    base64 += kAlphabet[v >> 18 & 63];
    base64 += kAlphabet[v >> 12 & 63];
    base64 += i + 1 < buffer.size() ? kAlphabet[v >> 6 & 63] : '=';
    base64 += i + 2 < buffer.size() ? kAlphabet[v & 63] : '=';
  }
  const auto vertices = std::to_string(mesh.vertex_count());
  const auto pos_bytes = std::to_string(mesh.positions.size() * 4);
  const auto idx_bytes = std::to_string(mesh.indices.size() * 4);
  std::string json = R"({"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"grid","mesh":0}],)"
                     R"("meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],"accessors":[)"
                     R"({"bufferView":0,"componentType":5126,"count":)" + vertices +
                     R"(,"type":"VEC3","min":[0,0,0],"max":[)" + std::to_string(n) + "," + std::to_string(n) +
                     R"(,0]},{"bufferView":1,"componentType":5125,"count":)" + std::to_string(mesh.indices.size()) +
                     R"(,"type":"SCALAR"}],"bufferViews":[{"buffer":0,"byteLength":)" + pos_bytes +
                     R"(},{"buffer":0,"byteOffset":)" + pos_bytes + R"(,"byteLength":)" + idx_bytes +
                     R"(}],"buffers":[{"byteLength":)" + std::to_string(buffer.size()) +
                     R"(,"uri":"data:application/octet-stream;base64,)" + base64 + R"("}]})";
  return text_bytes(json);
}

}  // namespace fixtures
