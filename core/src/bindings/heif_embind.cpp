// JavaScript API of the HEIC module (Embind): decodeHeif(bytes) gives
// { width, height, rgba, icc } or { error }. The engine module converts what
// it returns (convertPixels), so this one holds only libheif and libde265.

#include "pixie/build_info.hpp"
#include "pixie/heif.hpp"

#include <emscripten/bind.h>
#include <emscripten/val.h>

namespace {

using emscripten::val;

val decode_heif(const val& data) {
  pixie::Bytes bytes(data["length"].as<std::size_t>());
  val(emscripten::typed_memory_view(bytes.size(), bytes.data())).call<void>("set", data);
  const auto decoded = pixie::heif::decode(bytes);
  val object = val::object();
  if (!decoded) {
    object.set("error", decoded.error().message);
    return object;
  }
  // slice() copies out of WASM memory, which may move when it grows.
  auto copy = [](const pixie::Bytes& values) {
    return val(emscripten::typed_memory_view(values.size(), values.data())).call<val>("slice");
  };
  object.set("width", decoded->pixels.width);
  object.set("height", decoded->pixels.height);
  object.set("rgba", copy(decoded->pixels.rgba));
  object.set("icc", copy(decoded->icc));
  return object;
}

std::string build_source() { return pixie::build::source; }

}  // namespace

EMSCRIPTEN_BINDINGS(pixie_heif) {
  emscripten::function("decodeHeif", &decode_heif);
  emscripten::function("buildSource", &build_source);
}
