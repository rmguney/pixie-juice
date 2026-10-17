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

/// A file the glTF refers to, at the path (relative to the glTF) gltfpack reads it from.
struct Staged {
  std::string path;
  ByteView data;
};

/// What gltfpack needs besides the file: the files its URIs name, found among
/// `resources` by path or file name. A URI that would leave the directory is
/// refused; one no resource matches fails, naming the file.
Result<std::vector<Staged>> staged_resources(const cgltf_data& parsed, std::span<const Resource> resources) {
  std::vector<Staged> staged;
  auto stage = [&](const char* uri) -> std::optional<Error> {
    if (!uri || std::strncmp(uri, "data:", 5) == 0) return std::nullopt;
    std::string path(uri);
    cgltf_decode_uri(path.data());
    path.resize(std::strlen(path.c_str()));
    std::replace(path.begin(), path.end(), '\\', '/');
    const bool escapes = path.empty() || path.front() == '/' || path.find(':') != std::string::npos ||
                         path == ".." || path.starts_with("../") || path.find("/../") != std::string::npos ||
                         path.ends_with("/..");
    if (escapes) return Error{"The glTF refers to a file outside its folder: " + path};
    const std::string_view file_name = std::string_view(path).substr(path.rfind('/') + 1);
    const Resource* match = nullptr;
    for (const Resource& resource : resources) {
      if (resource.name == path) match = &resource;
    }
    for (const Resource& resource : resources) {
      if (!match && resource.name == file_name) match = &resource;
    }
    if (!match) return Error{"The glTF needs " + std::string(file_name) + "; add it with the .gltf"};
    const bool seen = std::any_of(staged.begin(), staged.end(), [&](const Staged& s) { return s.path == path; });
    if (!seen) staged.push_back({std::move(path), match->data});
    return std::nullopt;
  };
  for (cgltf_size i = 0; i < parsed.buffers_count; ++i) {
    if (auto error = stage(parsed.buffers[i].uri)) return std::unexpected(*error);
  }
  for (cgltf_size i = 0; i < parsed.images_count; ++i) {
    if (auto error = stage(parsed.images[i].uri)) return std::unexpected(*error);
  }
  return staged;
}

/// Whether every extension the file uses is one gltfpack carries (false: pass it through).
bool packable(const cgltf_data& parsed) {
  for (cgltf_size i = 0; i < parsed.extensions_used_count; ++i) {
    const std::string_view name = parsed.extensions_used[i];
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

Result<std::optional<Bytes>> optimize_gltf(ByteView data, Format format, const Options& options,
                                           std::span<const Resource> resources) {
  const cgltf_options parse_options{};
  cgltf_data* parsed = nullptr;
  if (cgltf_parse(&parse_options, data.data(), data.size(), &parsed) != cgltf_result_success) {
    return fail("Invalid glTF");
  }
  const std::unique_ptr<cgltf_data, decltype(&cgltf_free)> owner(parsed, cgltf_free);
  // gltfpack reads the files a glTF refers to from beside it: they are staged
  // in a directory of their own (natively, nothing else on disk is ever read).
  const auto staged = staged_resources(*parsed, resources);
  if (!staged) return std::unexpected(staged.error());
  if (!packable(*parsed)) return std::optional<Bytes>{};

  const TempDir directory;
  const std::string input_name = format == Format::Glb ? "input.glb" : "input.gltf";
  if (!directory.write(input_name, data)) return fail("Could not stage glTF input");
  for (const Staged& file : *staged) {
    if (!directory.write(file.path, file.data)) return fail("Could not stage " + file.path);
  }
  const std::string input = directory.path() + "/" + input_name;
  TempFile output(".glb");

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
    if (const int code = gltfpack(input.c_str(), output.path().c_str(), nullptr, settings); code != 0) {
      return fail("gltfpack could not process this file (code " + std::to_string(code) + ")");
    }
  }
  auto bytes = output.read();
  if (!bytes || bytes->empty()) return fail("gltfpack produced no output");
  // gltfpack restructures the file, so it is accepted on size, against the file
  // together with what it uses; it keeps every primitive.
  std::size_t original = data.size();
  for (const Staged& file : *staged) original += file.data.size();
  if (bytes->size() >= original) return std::optional<Bytes>{};
  return std::optional<Bytes>(std::move(*bytes));
}

}  // namespace pixie::detail
