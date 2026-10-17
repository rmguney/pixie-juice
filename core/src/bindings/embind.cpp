// JavaScript API (Embind). Byte arrays cross the boundary as Uint8Array;
// results are plain objects { data, format, changed } or { error }, since the
// module is built without C++ exceptions.

#include "pixie/build_info.hpp"
#include "pixie/pixie.hpp"

#include <emscripten/bind.h>
#include <emscripten/val.h>

#include <algorithm>

namespace {

using emscripten::val;

pixie::Bytes from_js(const val& array) {
  pixie::Bytes bytes(array["length"].as<std::size_t>());
  val(emscripten::typed_memory_view(bytes.size(), bytes.data())).call<void>("set", array);
  return bytes;
}

/// A typed array (Uint8Array, Float32Array, Uint32Array) holding a copy of `values`.
template <class T>
val to_js(const std::vector<T>& values) {
  // slice() copies out of WASM memory, which may move when it grows.
  return val(emscripten::typed_memory_view(values.size(), values.data())).call<val>("slice");
}

pixie::Options read_options(const val& options) {
  pixie::Options out;
  if (options.isUndefined() || options.isNull()) return out;
  if (const val q = options["quality"]; q.isNumber()) out.quality = q.as<int>();
  if (const val l = options["lossless"]; !l.isUndefined()) out.lossless = l.as<bool>();
  if (const val m = options["preserveMetadata"]; !m.isUndefined()) out.preserve_metadata = m.as<bool>();
  if (const val r = options["meshRatio"]; r.isNumber()) out.mesh_ratio = r.as<float>();
  if (const val s = options["maxSize"]; s.isNumber()) out.max_size = std::uint32_t(std::max(0.0, s.as<double>()));
  return out;
}

/// `input` is returned as the data when the file came back unchanged, so it isn't copied twice.
val to_result(const pixie::Result<pixie::Output>& result, const val& input) {
  val object = val::object();
  if (!result) {
    object.set("error", result.error().message);
    return object;
  }
  object.set("data", result->changed ? to_js(result->data) : input);
  object.set("format", std::string(pixie::format_name(result->format)));
  object.set("changed", result->changed);
  return object;
}

/// options.resources: the files a .gltf refers to, as [{ name, data }].
val optimize(const val& data, const val& options) {
  const auto bytes = from_js(data);
  std::vector<std::string> names;
  std::vector<pixie::Bytes> contents;
  if (!options.isUndefined() && !options.isNull()) {
    if (const val list = options["resources"]; !list.isUndefined() && !list.isNull()) {
      const auto count = list["length"].as<std::size_t>();
      for (std::size_t i = 0; i < count; ++i) {
        names.push_back(list[i]["name"].as<std::string>());
        contents.push_back(from_js(list[i]["data"]));
      }
    }
  }
  std::vector<pixie::Resource> resources;
  for (std::size_t i = 0; i < names.size(); ++i) resources.push_back({names[i], contents[i]});
  return to_result(pixie::optimize(bytes, read_options(options), resources), data);
}

val convert(const val& data, const std::string& target, const val& options) {
  const auto format = pixie::format_from_name(target);
  const auto bytes = from_js(data);
  return to_result(pixie::convert(bytes, format, read_options(options)), data);
}

val decode_image(const val& data) {
  const auto bytes = from_js(data);
  auto pixels = pixie::decode(bytes);
  val object = val::object();
  if (!pixels) {
    object.set("error", pixels.error().message);
    return object;
  }
  object.set("width", pixels->width);
  object.set("height", pixels->height);
  object.set("rgba", to_js(pixels->rgba));
  return object;
}

pixie::Pixels pixels_from_js(const val& rgba, std::uint32_t width, std::uint32_t height) {
  return pixie::Pixels{width, height, from_js(rgba)};
}

/// Pixels decoded outside the core (HEIC by its own module, AVIF by the browser).
val convert_pixels(const val& rgba, std::uint32_t width, std::uint32_t height, const val& icc,
                   const std::string& target, const val& options) {
  const auto result = pixie::convert_pixels(pixels_from_js(rgba, width, height), from_js(icc),
                                            pixie::format_from_name(target), read_options(options));
  return to_result(result, val::undefined());
}

val to_srgb(const val& rgba, std::uint32_t width, std::uint32_t height, const val& icc) {
  return to_js(pixie::to_srgb(pixels_from_js(rgba, width, height), from_js(icc)).rgba);
}

val decode_mesh(const val& data) {
  const auto bytes = from_js(data);
  auto geometry = pixie::decode_mesh(bytes);
  val object = val::object();
  if (!geometry) {
    object.set("error", geometry.error().message);
    return object;
  }
  object.set("positions", to_js(geometry->positions));
  if (!geometry->normals.empty()) object.set("normals", to_js(geometry->normals));
  if (!geometry->colors.empty()) object.set("colors", to_js(geometry->colors));
  object.set("indices", to_js(geometry->indices));
  if (!geometry->lines.empty()) object.set("lines", to_js(geometry->lines));
  return object;
}

/// "image:png", "mesh:obj" or "unknown".
std::string detect(const val& data) {
  const auto format = pixie::detect(from_js(data));
  switch (pixie::kind_of(format)) {
    case pixie::Kind::Image: return "image:" + std::string(pixie::format_name(format));
    case pixie::Kind::Mesh: return "mesh:" + std::string(pixie::format_name(format));
    case pixie::Kind::Unknown: break;
  }
  return "unknown";
}

std::string version() { return pixie::build::version; }
std::string build_timestamp() { return pixie::build::timestamp; }
std::string build_source() { return pixie::build::source; }

}  // namespace

EMSCRIPTEN_BINDINGS(pixie_core) {
  emscripten::function("optimize", &optimize);
  emscripten::function("convert", &convert);
  emscripten::function("detect", &detect);
  emscripten::function("decodeImage", &decode_image);
  emscripten::function("decodeMesh", &decode_mesh);
  emscripten::function("convertPixels", &convert_pixels);
  emscripten::function("toSrgb", &to_srgb);
  emscripten::function("version", &version);
  emscripten::function("buildTimestamp", &build_timestamp);
  emscripten::function("buildSource", &build_source);
}
