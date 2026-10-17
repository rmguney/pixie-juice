#pragma once

// Pixie Juice core: format detection, optimization and conversion of images
// and meshes. Every codec and mesh algorithm comes from a third-party library
// (mozjpeg, zopflipng/lodepng, libwebp, giflib, libtiff with zstd and xz,
// Little CMS, stb, meshoptimizer/gltfpack, fast_obj, miniply); this layer
// chooses candidates and verifies them. pixie.h is this API for C and other
// languages, without convert_pixels, to_srgb and glTF resources.

#include <cstdint>
#include <expected>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace pixie {

using Bytes = std::vector<std::uint8_t>;
using ByteView = std::span<const std::uint8_t>;

enum class Format {
  Unknown,
  // Images. HEIC and AVIF are recognised but not decoded by the core: see convert_pixels.
  Png, Jpeg, WebP, Gif, Bmp, Tga, Tiff, Ico, Svg, Heic, Avif,
  // Meshes
  Obj, Stl, Ply, Gltf, Glb, Fbx,
};

enum class Kind { Unknown, Image, Mesh };

[[nodiscard]] Kind kind_of(Format format);
/// Lower-case name, e.g. "png", "glb".
[[nodiscard]] std::string_view format_name(Format format);
[[nodiscard]] Format format_from_name(std::string_view name);
[[nodiscard]] Format detect(ByteView data);

struct Options {
  /// 1-100. Drives lossy encoders and the error budget of the fidelity check.
  int quality = 75;
  /// Never change pixels or geometry.
  bool lossless = false;
  /// Keep EXIF/XMP/comments and other non-visual metadata.
  bool preserve_metadata = false;
  /// Share of triangles to keep when simplifying meshes, in (0, 1].
  float mesh_ratio = 1.0f;
  /// Images larger than this on their longest side, in pixels, are scaled down
  /// to fit and encoded again, upright (0: keep the size). Optimizing leaves
  /// the size of animations, multi-page TIFFs, icons, SVG, HEIC and AVIF alone.
  std::uint32_t max_size = 0;
};

struct Error {
  std::string message;
};

template <class T>
using Result = std::expected<T, Error>;

struct Output {
  /// The new file; empty when `changed` is false (the input is the result).
  Bytes data;
  Format format = Format::Unknown;
  bool changed = false;
};

/// A file a .gltf refers to by URI (a .bin buffer, a texture), under its
/// file name or the path the glTF names it by.
struct Resource {
  std::string name;
  ByteView data;
};

/// Optimizes within the input's own format. Every candidate must be smaller
/// and pass verification (decodes, same size, alpha and colour kept, pixel
/// error within the quality budget; OBJ, STL and PLY keep their exact triangles
/// unless simplification was asked for, while glTF output from gltfpack is
/// accepted on size); otherwise the input is the result. HEIC and AVIF, which
/// the core has no encoder for, are always the result.
[[nodiscard]] Result<Output> optimize(ByteView input, const Options& options);
/// The same, with the files a .gltf refers to: the GLB it becomes holds them.
[[nodiscard]] Result<Output> optimize(ByteView input, const Options& options, std::span<const Resource> resources);

/// Converts a decodable image to `target`: Png, Jpeg, WebP, Gif, Bmp, Tga, Ico or Tiff.
/// The result is upright (EXIF/TIFF orientation applied) and keeps the ICC
/// profile where the target can embed one; otherwise colours are converted to sRGB.
[[nodiscard]] Result<Output> convert(ByteView input, Format target, const Options& options);

struct Pixels {
  std::uint32_t width = 0;
  std::uint32_t height = 0;
  Bytes rgba;  // 8-bit RGBA, row-major
};

/// Decodes the first frame (or largest icon entry) of a raster image for
/// display: upright and in sRGB.
[[nodiscard]] Result<Pixels> decode(ByteView input);

/// Converts pixels decoded elsewhere, upright and in the colour space `icc`
/// describes (sRGB when empty), to `target`, as `convert` does a file. For
/// formats the core recognises but doesn't decode: HEIC (see pixie/heif.hpp)
/// and AVIF (the web app decodes it with the browser).
[[nodiscard]] Result<Output> convert_pixels(Pixels pixels, ByteView icc, Format target, const Options& options);

/// Pixels in the colour space `icc` describes, converted to sRGB for display
/// (unchanged when `icc` is empty or can't be used).
[[nodiscard]] Pixels to_srgb(Pixels pixels, ByteView icc);

struct Geometry {
  std::vector<float> positions;         // xyz per vertex
  std::vector<float> normals;           // xyz per vertex, or empty
  std::vector<float> colors;            // rgb in 0..1 per vertex, or empty
  std::vector<std::uint32_t> indices;   // three per triangle
  std::vector<std::uint32_t> lines;     // two per line segment (OBJ "l")
};

/// Reads an OBJ, STL or PLY for display, including files that `optimize`
/// passes through because it can't write them back exactly: its triangles and
/// lines, or with neither, its vertices as a point cloud.
[[nodiscard]] Result<Geometry> decode_mesh(ByteView input);

}  // namespace pixie
