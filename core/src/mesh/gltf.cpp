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

#include <algorithm>
#include <array>
#include <clocale>
#include <cstring>
#include <memory>
#include <string>
#include <string_view>

#if !defined(_WIN32) && !defined(__EMSCRIPTEN__)
#include <locale.h>  // newlocale, uselocale
#ifdef __APPLE__
#include <xlocale.h>
#endif
#endif

// Defined in gltfpack.cpp, which has no header for them.
Settings defaults();
int gltfpack(const char* input, const char* output, const char* report, Settings settings);

namespace pixie::detail {

namespace {

/// Extensions gltfpack 1.3 carries through. It skips any other extension, even
/// a required one, with only a warning; it refuses Draco outright.
constexpr std::array<std::string_view, 22> kCarriedExtensions{
    "KHR_mesh_quantization", "EXT_meshopt_compression", "KHR_meshopt_compression", "KHR_texture_transform",
    "KHR_materials_pbrSpecularGlossiness", "KHR_materials_clearcoat", "KHR_materials_transmission",
    "KHR_materials_ior", "KHR_materials_specular", "KHR_materials_sheen", "KHR_materials_volume",
    "KHR_materials_emissive_strength", "KHR_materials_iridescence", "KHR_materials_anisotropy",
    "KHR_materials_dispersion", "KHR_materials_diffuse_transmission", "KHR_materials_unlit",
    "KHR_materials_variants", "KHR_lights_punctual", "KHR_texture_basisu", "EXT_texture_webp",
    "EXT_mesh_gpu_instancing"};

/// Whether gltfpack can take the file: it must parse, its resources must all be
/// embedded (GLB chunk or data: URI; gltfpack resolves any other URI against the
/// input's directory, which natively reads the local file system), and every
/// extension it uses must be one gltfpack carries (false: pass it through).
Result<bool> packable(ByteView data) {
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
  for (cgltf_size i = 0; i < parsed->extensions_used_count; ++i) {
    const std::string_view name = parsed->extensions_used[i];
    if (std::find(kCarriedExtensions.begin(), kCarriedExtensions.end(), name) == kCarriedExtensions.end()) {
      return false;
    }
  }
  return true;
}

/// The "C" locale on this thread while it lives. cgltf reads and gltfpack
/// writes floats through the C library, which follows the host's locale: under
/// a comma-decimal one ("de_DE") a library caller would get altered geometry.
/// (WebAssembly only has the "C" locale.)
class CLocaleScope {
 public:
  CLocaleScope() {
#if defined(_WIN32)
    mode_ = _configthreadlocale(_ENABLE_PER_THREAD_LOCALE);
    if (const char* current = std::setlocale(LC_ALL, nullptr)) previous_ = current;
    std::setlocale(LC_ALL, "C");
#elif !defined(__EMSCRIPTEN__)
    c_ = newlocale(LC_ALL_MASK, "C", locale_t(0));
    if (c_) previous_ = uselocale(c_);
#endif
  }
  ~CLocaleScope() {
#if defined(_WIN32)
    if (!previous_.empty()) std::setlocale(LC_ALL, previous_.c_str());
    _configthreadlocale(mode_);
#elif !defined(__EMSCRIPTEN__)
    if (c_) {
      uselocale(previous_);
      freelocale(c_);
    }
#endif
  }
  CLocaleScope(const CLocaleScope&) = delete;
  CLocaleScope& operator=(const CLocaleScope&) = delete;

 private:
#if defined(_WIN32)
  int mode_ = 0;
  std::string previous_;
#elif !defined(__EMSCRIPTEN__)
  locale_t c_ = locale_t(0);
  locale_t previous_ = locale_t(0);
#endif
};

}  // namespace

Result<std::optional<Bytes>> optimize_gltf(ByteView data, Format format, const Options& options) {
  const auto accepted = packable(data);
  if (!accepted) return std::unexpected(accepted.error());
  if (!*accepted) return std::optional<Bytes>{};

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

  {
    const CLocaleScope c_locale;
    if (const int code = gltfpack(input.path().c_str(), output.path().c_str(), nullptr, settings); code != 0) {
      return fail("gltfpack could not process this file (code " + std::to_string(code) + ")");
    }
  }
  auto bytes = output.read();
  if (!bytes || bytes->empty()) return fail("gltfpack produced no output");
  return std::optional<Bytes>(std::move(*bytes));
}

}  // namespace pixie::detail
