// C API (pixie.h) over the C++ core. Results keep their data in a
// pixie_storage, so files, pixels, meshes and error messages aren't copied again.
// No exception crosses the C boundary: allocation failures become errors.

#include "pixie/pixie.h"

#include "pixie/build_info.hpp"
#include "pixie/pixie.hpp"

#include <new>
#include <string>

struct pixie_storage {
  pixie::Bytes bytes;
  pixie::Geometry geometry;
  std::string error;
};

namespace {

pixie::ByteView view(const std::uint8_t* data, std::size_t size) {
  return data ? pixie::ByteView(data, size) : pixie::ByteView();
}

pixie::Options to_options(const pixie_options* options) {
  if (!options) return {};
  return {options->quality, options->lossless != 0, options->preserve_metadata != 0, options->mesh_ratio};
}

/// Format names are string literals, so their views are NUL-terminated.
const char* name_of(pixie::Format format) { return pixie::format_name(format).data(); }

/// Runs `call` (returning a pixie::Result) and hands its value to `fill`, or
/// stores the error in `out`, whose `error` and `storage` fields it sets.
template <class Out, class Call, class Fill>
Out run(Call call, Fill fill) noexcept {
  Out out{};
  try {
    auto result = call();
    if (result) {
      fill(out, *result);
    } else {
      out.storage = new pixie_storage{.error = std::move(result.error().message)};
      out.error = out.storage->error.c_str();
    }
  } catch (const std::bad_alloc&) {
    out = Out{};
    out.error = "Out of memory";
  } catch (...) {
    out = Out{};
    out.error = "Internal error";
  }
  return out;
}

void fill_result(pixie_result& out, pixie::Output& output) {
  if (output.changed) {
    out.storage = new pixie_storage{.bytes = std::move(output.data)};
    out.data = out.storage->bytes.data();
    out.size = out.storage->bytes.size();
  }
  out.format = name_of(output.format);
  out.changed = output.changed;
}

}  // namespace

extern "C" {

pixie_options pixie_default_options(void) {
  const pixie::Options defaults;
  return {defaults.quality, defaults.lossless, defaults.preserve_metadata, defaults.mesh_ratio};
}

const char* pixie_detect(const std::uint8_t* data, std::size_t size) {
  return name_of(pixie::detect(view(data, size)));
}

pixie_result pixie_optimize(const std::uint8_t* data, std::size_t size, const pixie_options* options) {
  return run<pixie_result>([&] { return pixie::optimize(view(data, size), to_options(options)); }, fill_result);
}

pixie_result pixie_convert(const std::uint8_t* data, std::size_t size, const char* target,
                           const pixie_options* options) {
  return run<pixie_result>(
      [&] {
        const auto format = pixie::format_from_name(target ? target : "");
        return pixie::convert(view(data, size), format, to_options(options));
      },
      fill_result);
}

pixie_image pixie_decode(const std::uint8_t* data, std::size_t size) {
  return run<pixie_image>([&] { return pixie::decode(view(data, size)); },
                          [](pixie_image& out, pixie::Pixels& pixels) {
                            out.storage = new pixie_storage{.bytes = std::move(pixels.rgba)};
                            out.width = pixels.width;
                            out.height = pixels.height;
                            out.rgba = out.storage->bytes.data();
                          });
}

pixie_mesh pixie_decode_mesh(const std::uint8_t* data, std::size_t size) {
  return run<pixie_mesh>([&] { return pixie::decode_mesh(view(data, size)); },
                         [](pixie_mesh& out, pixie::Geometry& geometry) {
                           out.storage = new pixie_storage{.geometry = std::move(geometry)};
                           const auto& g = out.storage->geometry;
                           out.positions = g.positions.data();
                           out.normals = g.normals.empty() ? nullptr : g.normals.data();
                           out.colors = g.colors.empty() ? nullptr : g.colors.data();
                           out.vertex_count = g.positions.size() / 3;
                           out.indices = g.indices.data();
                           out.index_count = g.indices.size();
                         });
}

void pixie_result_free(pixie_result* result) {
  if (!result) return;
  delete result->storage;
  *result = {};
}

void pixie_image_free(pixie_image* image) {
  if (!image) return;
  delete image->storage;
  *image = {};
}

void pixie_mesh_free(pixie_mesh* mesh) {
  if (!mesh) return;
  delete mesh->storage;
  *mesh = {};
}

const char* pixie_version(void) { return pixie::build::version; }

}  // extern "C"
