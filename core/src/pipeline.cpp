#include "internal.hpp"

#include <algorithm>
#include <array>
#include <bit>
#include <cctype>
#include <cmath>
#include <string_view>

namespace pixie {

namespace {

using namespace detail;

/// Options as every stage sees them: quality in 1-100, a usable mesh ratio,
/// and no simplification in lossless mode.
Options normalized(const Options& options) {
  Options out = options;
  out.quality = std::clamp(options.quality, 1, 100);
  out.mesh_ratio = options.lossless || !std::isfinite(options.mesh_ratio) ? 1.0f
                                                                          : std::clamp(options.mesh_ratio, 0.01f, 1.0f);
  return out;
}

Output unchanged(Format format) { return Output{{}, format, false}; }

std::vector<Bytes> image_candidates(ByteView input, Format format, const Image& image, const Options& options) {
  switch (format) {
    case Format::Jpeg: return jpeg_candidates(input, image, options);
    case Format::Png: return png_candidates(input, options);
    case Format::WebP: return webp_candidates(input, image, options);
    case Format::Gif: return gif_candidates(input, options);
    case Format::Ico: return ico_candidates(input, options);
    case Format::Tiff: return tiff_candidates(input, options);
    case Format::Bmp:
    case Format::Tga: {
      // No useful compression of their own: re-encode as PNG (lossless) or,
      // for opaque images below quality 85, JPEG.
      std::vector<Bytes> out;
      if (auto png = encode_png(image)) out.push_back(std::move(*png));
      if (!options.lossless && options.quality < 85 && !image.has_alpha()) {
        if (auto jpeg = encode_jpeg(image, options.quality)) out.push_back(std::move(*jpeg));
      }
      return out;
    }
    default: return {};
  }
}

Result<Output> optimize_image(ByteView input, Format format, const Options& options) {
  // SVG is optimized by SVGO in the web layer.
  if (format == Format::Svg) return unchanged(format);

  auto image = decode_rgba(input, format);
  if (!image) {
    if (format == Format::Tiff && tiff_passes_through(input)) return unchanged(format);
    return std::unexpected(image.error());
  }
  const Reference reference{*image, image->has_alpha(), image->is_gray()};
  const ColourInfo colour = read_colour_info(input, format);

  const double budget = allowed_error(options);
  auto candidates = image_candidates(input, format, *image, options);
  // Smallest first: the first one that passes verification is the result.
  std::stable_sort(candidates.begin(), candidates.end(), [](const Bytes& a, const Bytes& b) { return a.size() < b.size(); });
  for (auto& candidate : candidates) {
    if (candidate.size() >= input.size()) break;
    const Format candidate_format = detect(candidate);
    // Pixels are compared in stored orientation and colour, so what displays
    // them (orientation, colour profile) must survive too.
    const ColourInfo kept = read_colour_info(candidate, candidate_format);
    if (kept.orientation != colour.orientation || kept.icc.empty() != colour.icc.empty()) continue;
    if (is_faithful(reference, candidate, candidate_format, budget)) return Output{std::move(candidate), candidate_format, true};
  }
  return unchanged(format);
}

Result<ReadMesh> read_mesh(ByteView input, Format format, MeshUse use, StlInfo* stl = nullptr,
                           PlyInfo* ply = nullptr) {
  switch (format) {
    case Format::Obj: return read_obj(input, use);
    case Format::Stl: return read_stl(input, use, stl);
    case Format::Ply: return read_ply(input, use, ply);
    default: return fail("Only OBJ, STL and PLY meshes can be read");
  }
}

/// The same triangles in any order and rotation: each corner's position,
/// normal, UV and colour, and each triangle's material group, compared by
/// exact bit patterns.
bool same_geometry(const Mesh& a, const Mesh& b) {
  if (a.triangle_count() != b.triangle_count() || a.normals.empty() != b.normals.empty() ||
      a.uvs.empty() != b.uvs.empty() || a.colors.empty() != b.colors.empty() ||
      a.triangle_groups.empty() != b.triangle_groups.empty()) {
    return false;
  }
  using Corner = std::array<std::uint32_t, 11>;  // position xyz, normal xyz, uv, rgb
  using Triangle = std::pair<std::string_view, std::array<Corner, 3>>;  // material group, corners
  auto triangles = [](const Mesh& m) {
    auto copy = [](const std::vector<float>& values, std::size_t vertex, std::size_t n, Corner& corner,
                   std::size_t at) {
      if (values.empty()) return;
      for (std::size_t i = 0; i < n; ++i) corner[at + i] = std::bit_cast<std::uint32_t>(values[vertex * n + i]);
    };
    std::vector<Triangle> out(m.triangle_count());
    for (std::size_t t = 0; t < out.size(); ++t) {
      auto& [group, corners] = out[t];
      if (!m.triangle_groups.empty()) group = m.group_names[m.triangle_groups[t]];
      for (std::size_t k = 0; k < 3; ++k) {
        const std::size_t v = m.indices[t * 3 + k];
        copy(m.positions, v, 3, corners[k], 0);
        copy(m.normals, v, 3, corners[k], 3);
        copy(m.uvs, v, 2, corners[k], 6);
        copy(m.colors, v, 3, corners[k], 8);
      }
      std::rotate(corners.begin(), std::min_element(corners.begin(), corners.end()), corners.end());
    }
    std::sort(out.begin(), out.end());
    return out;
  };
  return triangles(a) == triangles(b);
}

Result<Output> optimize_mesh_file(ByteView input, Format format, const Options& options) {
  if (format == Format::Gltf || format == Format::Glb) {
    auto packed = optimize_gltf(input, format, options);
    if (!packed) return std::unexpected(packed.error());
    // gltfpack restructures the file, so it is accepted on size; it keeps every primitive.
    if (!*packed || (*packed)->size() >= input.size()) return unchanged(format);
    return Output{std::move(**packed), Format::Glb, true};
  }

  if (format == Format::Fbx) return unchanged(format);  // detection only
  StlInfo stl;
  PlyInfo ply;
  const auto source = read_mesh(input, format, MeshUse::Optimize, &stl, &ply);
  if (!source) return std::unexpected(source.error());
  if (!source->exact) return unchanged(format);
  // Binary STL is 50 bytes per triangle: without simplification nothing can get smaller.
  if (format == Format::Stl && stl.binary && options.mesh_ratio >= 1.0f) return unchanged(format);

  const Mesh optimized = optimize_mesh(source->mesh, options.mesh_ratio);
  Bytes out = format == Format::Obj   ? write_obj(optimized, obj_header(input))
              : format == Format::Stl ? write_stl(optimized, stl)
                                      : write_ply(optimized, ply);
  if (out.size() >= input.size()) return unchanged(format);

  // Without simplification, reading the output back must give exactly the input's triangles.
  if (options.mesh_ratio >= 1.0f) {
    const auto reread = read_mesh(out, format, MeshUse::Optimize);
    if (!reread || !reread->exact || !same_geometry(source->mesh, reread->mesh)) return unchanged(format);
  }
  return Output{std::move(out), format, true};
}

}  // namespace

Result<Output> optimize(ByteView input, const Options& options) {
  const Options settings = normalized(options);
  const Format format = detect(input);
  switch (kind_of(format)) {
    case Kind::Image: return optimize_image(input, format, settings);
    case Kind::Mesh: return optimize_mesh_file(input, format, settings);
    case Kind::Unknown: break;
  }
  return fail("Unknown file format");
}

Result<Output> convert(ByteView input, Format target, const Options& options) {
  const Options settings = normalized(options);
  switch (target) {
    case Format::Png: case Format::Jpeg: case Format::WebP: case Format::Gif:
    case Format::Bmp: case Format::Tga: case Format::Ico: case Format::Tiff: break;
    default: return fail("Unsupported conversion target");
  }
  const Format source = detect(input);
  if (kind_of(source) != Kind::Image || source == Format::Svg) return fail("Only raster images can be converted");

  auto image = decode_rgba(input, source);
  if (!image) return std::unexpected(image.error());
  ColourInfo colour = read_colour_info(input, source);
  apply_orientation(*image, colour.orientation);
  // Only an RGB profile describes the decoded RGB pixels, and only PNG, JPEG,
  // WebP and TIFF can carry it. Otherwise the profile is applied to the pixels
  // (RGB and grey profiles); a CMYK profile no longer fits pixels already
  // turned into RGB, so it is dropped with them left as decoded.
  const bool embeds_icc = icc_colour_space(colour.icc) == "RGB " &&
                          (target == Format::Png || target == Format::Jpeg || target == Format::WebP ||
                           target == Format::Tiff);
  if (!embeds_icc && !colour.icc.empty()) {
    convert_to_srgb(*image, colour.icc);
    colour.icc.clear();
  }

  Result<Bytes> encoded = fail("Unsupported conversion target");
  switch (target) {
    case Format::Png: encoded = encode_png(*image, colour.icc); break;
    case Format::Jpeg: encoded = encode_jpeg(*image, settings.quality, colour.icc); break;
    case Format::WebP:
      encoded = encode_webp(*image, settings.quality, settings.lossless || settings.quality == 100, colour.icc);
      break;
    case Format::Tiff: encoded = encode_tiff(*image, colour.icc); break;
    case Format::Gif: encoded = encode_gif(*image); break;
    case Format::Bmp: encoded = encode_bmp(*image); break;
    case Format::Tga: encoded = encode_tga(*image); break;
    case Format::Ico: encoded = encode_ico(*image); break;
    default: break;
  }
  if (!encoded) return std::unexpected(encoded.error());
  // Lossless: the output must decode to exactly the pixels encoded. JPEG never
  // does; GIF only with up to 256 colours; ICO only up to 256 pixels across.
  if (settings.lossless &&
      !is_faithful(Reference{*image, image->has_alpha(), image->is_gray()}, *encoded, target, 0.0)) {
    std::string name(format_name(target));
    std::transform(name.begin(), name.end(), name.begin(), [](unsigned char c) { return char(std::toupper(c)); });
    return fail(name + " can't hold this image losslessly; turn off Lossless or choose PNG, WebP or TIFF");
  }
  return Output{std::move(*encoded), target, true};
}

Result<Geometry> decode_mesh(ByteView input) {
  auto read = read_mesh(input, detect(input), MeshUse::Display);
  if (!read) return std::unexpected(read.error());
  Mesh& mesh = read->mesh;
  if (mesh.indices.empty()) return fail("The mesh has no triangles");
  if (!all_finite(mesh.positions)) return fail("The mesh has non-finite coordinates");
  // Broken normals or colours are left out; positions alone still display.
  if (!all_finite(mesh.normals)) mesh.normals.clear();
  if (!all_finite(mesh.colors)) mesh.colors.clear();
  return Geometry{std::move(mesh.positions), std::move(mesh.normals), std::move(mesh.colors),
                  std::move(mesh.indices)};
}

Result<Pixels> decode(ByteView input) {
  const Format format = detect(input);
  if (kind_of(format) != Kind::Image || format == Format::Svg) return fail("Not a raster image");
  auto image = decode_rgba(input, format);
  if (!image) return std::unexpected(image.error());
  const ColourInfo colour = read_colour_info(input, format);
  apply_orientation(*image, colour.orientation);
  convert_to_srgb(*image, colour.icc);
  return Pixels{image->width, image->height, std::move(image->rgba)};
}

}  // namespace pixie
