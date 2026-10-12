#include "internal.hpp"

#include <algorithm>
#include <array>
#include <bit>
#include <cmath>

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
  if (!image) return std::unexpected(image.error());
  const Reference reference{*image, image->has_alpha(), image->is_gray()};

  const double budget = allowed_error(options);
  Output best = unchanged(format);
  std::size_t best_size = input.size();
  for (auto& candidate : image_candidates(input, format, *image, options)) {
    if (candidate.size() >= best_size) continue;
    const Format candidate_format = detect(candidate);
    if (is_faithful(reference, candidate, candidate_format, budget)) {
      best_size = candidate.size();
      best = Output{std::move(candidate), candidate_format, true};
    }
  }
  return best;
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

/// The same triangles in any order and rotation, compared by exact bit patterns.
bool same_geometry(const Mesh& a, const Mesh& b) {
  if (a.triangle_count() != b.triangle_count()) return false;
  using Triangle = std::array<std::array<std::uint32_t, 3>, 3>;
  auto triangles = [](const Mesh& m) {
    std::vector<Triangle> out(m.triangle_count());
    for (std::size_t t = 0; t < out.size(); ++t) {
      for (std::size_t k = 0; k < 3; ++k) {
        const std::size_t v = m.indices[t * 3 + k];
        for (std::size_t c = 0; c < 3; ++c) out[t][k][c] = std::bit_cast<std::uint32_t>(m.positions[v * 3 + c]);
      }
      std::rotate(out[t].begin(), std::min_element(out[t].begin(), out[t].end()), out[t].end());
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
    if (packed->size() >= input.size()) return unchanged(format);
    return Output{std::move(*packed), Format::Glb, true};
  }

  if (format == Format::Fbx) return unchanged(format);  // detection only
  StlInfo stl;
  PlyInfo ply;
  const auto source = read_mesh(input, format, MeshUse::Optimize, &stl, &ply);
  if (!source) return std::unexpected(source.error());
  if (!source->exact) return unchanged(format);

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
  // Formats without ICC support get sRGB pixels instead of a profile.
  const bool embeds_icc = target == Format::Png || target == Format::Jpeg || target == Format::WebP ||
                          target == Format::Tiff;
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
