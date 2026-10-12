// glTF/GLB through gltfpack (meshoptimizer's glTF optimiser), called as a
// library on files in the temp directory (Emscripten's in-memory file system
// in WASM). Output is always GLB.
//
// Settings: nodes, materials and every vertex attribute are kept; geometry is
// deduplicated, reordered for the GPU, and quantized with KHR_mesh_quantization
// unless lossless (which also keeps animation keyframes as authored);
// simplification follows the mesh ratio. Meshopt compression
// (EXT_meshopt_compression) is left off so any glTF viewer can open the result.

#include "internal.hpp"

#include <cgltf.h>
#include <gltfpack.h>

#include <cstring>
#include <memory>
#include <string>

// Defined in gltfpack.cpp, which has no header for them.
Settings defaults();
int gltfpack(const char* input, const char* output, const char* report, Settings settings);

namespace pixie::detail {

namespace {

/// gltfpack resolves buffer and image URIs against the input's directory, so
/// only files whose resources are all embedded (GLB chunk or data: URI) are
/// passed on: anything else isn't part of the input, and natively it would
/// read from the local file system.
Result<void> check_self_contained(ByteView data) {
  const cgltf_options options{};
  cgltf_data* parsed = nullptr;
  if (cgltf_parse(&options, data.data(), data.size(), &parsed) != cgltf_result_success) {
    return fail("Invalid glTF");
  }
  const std::unique_ptr<cgltf_data, decltype(&cgltf_free)> owner(parsed, cgltf_free);
  auto external = [](const char* uri) { return uri && std::strncmp(uri, "data:", 5) != 0; };
  bool references_files = false;
  for (cgltf_size i = 0; i < parsed->buffers_count; ++i) references_files |= external(parsed->buffers[i].uri);
  for (cgltf_size i = 0; i < parsed->images_count; ++i) references_files |= external(parsed->images[i].uri);
  if (references_files) return fail("glTF references external files; use GLB or embedded data");
  return {};
}

}  // namespace

Result<Bytes> optimize_gltf(ByteView data, Format format, const Options& options) {
  if (auto contained = check_self_contained(data); !contained) return std::unexpected(contained.error());

  TempFile input(format == Format::Glb ? ".glb" : ".gltf");
  TempFile output(".glb");
  if (!input.write(data)) return fail("Could not stage glTF input");

  Settings settings = defaults();
  settings.keep_nodes = true;
  settings.keep_materials = true;
  // Every vertex attribute stays, even ones no material reads (lightmap UVs, tangents, colours).
  settings.keep_attributes = true;
  settings.keep_extras = options.preserve_metadata;
  if (options.lossless) {
    settings.quantize = false;
    settings.anim_freq = 0;  // keep keyframes as authored instead of resampling at 30 Hz
  }
  settings.simplify_ratio = options.mesh_ratio;
  settings.verbose = 0;

  if (const int code = gltfpack(input.path().c_str(), output.path().c_str(), nullptr, settings); code != 0) {
    return fail("gltfpack could not process this file (code " + std::to_string(code) + ")");
  }
  auto bytes = output.read();
  if (!bytes || bytes->empty()) return fail("gltfpack produced no output");
  return std::move(*bytes);
}

}  // namespace pixie::detail
