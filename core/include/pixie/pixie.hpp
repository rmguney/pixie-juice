#pragma once

// Pixie Juice core: format detection, optimization and conversion of images
// and meshes. Every codec and mesh algorithm comes from a third-party library
// (mozjpeg, zopflipng/lodepng, libwebp, giflib, libtiff, Little CMS, stb,
// meshoptimizer/gltfpack, fast_obj, miniply); this layer chooses candidates
// and verifies them. pixie.h is the same API for C and other languages.

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
  // Images
  Png, Jpeg, WebP, Gif, Bmp, Tga, Tiff, Ico, Svg,
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

/// Optimizes within the input's own format. Every candidate must be smaller
/// and pass verification (decodes, same size, alpha and colour kept, pixel
/// error within the quality budget; OBJ, STL and PLY keep their exact triangles
/// unless simplification was asked for, while glTF output from gltfpack is
/// accepted on size); otherwise the input is the result.
[[nodiscard]] Result<Output> optimize(ByteView input, const Options& options);

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

struct Geometry {
  std::vector<float> positions;         // xyz per vertex
  std::vector<float> normals;           // xyz per vertex, or empty
  std::vector<float> colors;            // rgb in 0..1 per vertex, or empty
  std::vector<std::uint32_t> indices;   // three per triangle
};

/// Reads the triangles of an OBJ, STL or PLY for display, including files that
/// `optimize` passes through because it can't write them back exactly.
[[nodiscard]] Result<Geometry> decode_mesh(ByteView input);

}  // namespace pixie
