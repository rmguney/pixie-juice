#pragma once

#include "pixie/pixie.hpp"

#include <array>
#include <cstddef>
#include <optional>

namespace pixie::detail {

inline std::unexpected<Error> fail(std::string message) {
  return std::unexpected(Error{std::move(message)});
}

inline void append_le16(Bytes& out, unsigned value) {
  out.push_back(std::uint8_t(value));
  out.push_back(std::uint8_t(value >> 8));
}

inline void append_le32(Bytes& out, std::uint32_t value) {
  append_le16(out, value & 0xFFFF);
  append_le16(out, value >> 16);
}

/// Most pixels any decoder will allocate for (256 MB of RGBA).
inline constexpr std::uint64_t kMaxPixels = 64ull * 1024 * 1024;

// ---------------------------------------------------------------------------
// Images

/// 8-bit RGBA pixels, row-major, no padding, in stored (not display) orientation.
struct Image {
  std::uint32_t width = 0;
  std::uint32_t height = 0;
  Bytes rgba;

  [[nodiscard]] std::size_t pixel_count() const { return std::size_t(width) * height; }
  /// Some pixel is not fully opaque.
  [[nodiscard]] bool has_alpha() const;
  /// R = G = B for every pixel.
  [[nodiscard]] bool is_gray() const;
};

/// Decodes the first frame (or largest icon entry) of a raster format.
Result<Image> decode_rgba(ByteView data, Format format);
Result<Image> decode_png(ByteView data);
Result<Image> decode_jpeg(ByteView data);
Result<Image> decode_ico(ByteView data);
Result<Image> decode_tiff(ByteView data);
/// The first frame on a transparent screen.
Result<Image> decode_gif(ByteView data);
/// Inflating the PNG's image data and compressed chunks stays within what its header allows.
bool png_within_limits(ByteView data);
/// A PNG lodepng may decode: a valid header, within kMaxPixels and png_within_limits.
bool png_decodable(ByteView data);
/// A TIFF libtiff opens but can't decode here (a codec not built in, floating-point samples):
/// optimizing passes it through.
bool tiff_passes_through(ByteView data);

/// What an image needs beyond its pixels to display correctly.
struct ColourInfo {
  Bytes icc;            // ICC profile, empty when none (sRGB)
  int orientation = 1;  // EXIF/TIFF orientation, 1-8
};
ColourInfo read_colour_info(ByteView data, Format format);
ColourInfo tiff_colour_info(ByteView data);
/// Orientation from a TIFF-structured EXIF block (with or without "Exif\0\0").
int exif_orientation(ByteView exif);
/// A TIFF-structured EXIF block holding only the orientation tag.
Bytes minimal_exif(int orientation);
/// Turns stored pixels into display orientation.
void apply_orientation(Image& image, int orientation);
/// Converts pixels from an RGB ICC profile to sRGB; false if the profile can't be used.
bool convert_to_srgb(Image& image, ByteView icc);
/// The profile's colour space from its header: "RGB ", "GRAY", "CMYK"...; empty without one.
std::string_view icc_colour_space(ByteView icc);

/// The input facts every candidate is checked against, computed once.
struct Reference {
  const Image& image;
  bool alpha;
  bool gray;
};

/// Mean per-pixel error the fidelity check allows, in 0-255 units: zero in
/// lossless mode, then a budget that grows as quality drops (about 10 at 75).
double allowed_error(const Options& options);
/// Mean absolute difference over RGB (weighted by opacity) plus alpha.
double mean_error(const Image& a, const Image& b);
/// `output` decodes as `output_format`, has the input's size, keeps alpha
/// and colour when the input had them, is within `budget` of the input on
/// average, and within 4 x `budget` over every 8x8 block.
bool is_faithful(const Reference& input, ByteView output, Format output_format, double budget);

// Candidate generators: each returns zero or more encodings of the input, all
// produced by the format's library. The pipeline keeps the smallest one that
// passes the fidelity check. `image` is the decoded input.
std::vector<Bytes> jpeg_candidates(ByteView input, const Image& image, const Options& options);
std::vector<Bytes> png_candidates(ByteView input, const Options& options);
std::vector<Bytes> webp_candidates(ByteView input, const Image& image, const Options& options);
std::vector<Bytes> gif_candidates(ByteView input, const Options& options);
std::vector<Bytes> ico_candidates(ByteView input, const Options& options);
std::vector<Bytes> tiff_candidates(ByteView input, const Options& options);

// Encoders for conversions (and BMP/TGA optimization). `icc` is embedded by
// formats that can carry a profile; callers convert to sRGB for the others.
Result<Bytes> encode_png(const Image& image, ByteView icc = {});
Result<Bytes> encode_jpeg(const Image& image, int quality, ByteView icc = {});
Result<Bytes> encode_webp(const Image& image, int quality, bool lossless, ByteView icc = {});
Result<Bytes> encode_tiff(const Image& image, ByteView icc = {});
Result<Bytes> encode_gif(const Image& image);
Result<Bytes> encode_bmp(const Image& image);
Result<Bytes> encode_tga(const Image& image);
Result<Bytes> encode_ico(const Image& image);

// ---------------------------------------------------------------------------
// Files: some libraries (miniply, gltfpack) only read and write paths. In
// WASM these live in Emscripten's in-memory file system.

class TempFile {
 public:
  explicit TempFile(std::string_view extension);
  ~TempFile();
  TempFile(const TempFile&) = delete;
  TempFile& operator=(const TempFile&) = delete;

  [[nodiscard]] const std::string& path() const { return path_; }
  bool write(ByteView data) const;
  [[nodiscard]] std::optional<Bytes> read() const;

 private:
  std::string path_;
};

// ---------------------------------------------------------------------------
// Meshes

/// Triangle mesh with optional per-vertex attributes, indexed.
struct Mesh {
  std::vector<float> positions;  // xyz
  std::vector<float> normals;    // xyz, empty or one per vertex
  std::vector<float> uvs;        // uv, empty or one per vertex
  std::vector<float> colors;     // rgb in 0..1, empty or one per vertex
  std::vector<std::uint32_t> indices;
  /// Material of each triangle, indexing `group_names`; both empty when the
  /// file has no materials. An empty name means "no material".
  std::vector<std::uint32_t> triangle_groups;
  std::vector<std::string> group_names;

  [[nodiscard]] std::size_t vertex_count() const { return positions.size() / 3; }
  [[nodiscard]] std::size_t triangle_count() const { return indices.size() / 3; }
};

/// Welds identical vertices, optionally simplifies to `ratio` of the
/// triangles, then orders for vertex cache, overdraw and vertex fetch.
Mesh optimize_mesh(const Mesh& mesh, float ratio);

[[nodiscard]] bool all_finite(std::span<const float> values);

/// What a mesh file is read for. Optimizing needs a file the writer can
/// reproduce exactly and stops reading once it can't; display takes whatever
/// triangles there are.
enum class MeshUse { Optimize, Display };

struct ReadMesh {
  Mesh mesh;
  /// False when the writer can't reproduce the file (OBJ curves or several
  /// objects, extra PLY properties, double precision, STL facet colours) or
  /// there is nothing to optimize (no triangles, non-finite coordinates).
  /// Such files pass through unchanged.
  bool exact = true;
};

Result<ReadMesh> read_obj(ByteView data, MeshUse use);
/// `header` holds lines copied from the source (mtllib, object name).
Bytes write_obj(const Mesh& mesh, std::string_view header);
std::string obj_header(ByteView source);

struct StlInfo {
  bool binary = false;
  std::string name;
};
Result<ReadMesh> read_stl(ByteView data, MeshUse use, StlInfo* info);
Bytes write_stl(const Mesh& mesh, const StlInfo& info);

struct PlyInfo {
  bool binary = false;
  /// Header comment and obj_info lines, kept: some carry meaning (comment TextureFile).
  std::vector<std::string> comments;
  bool color_uchar = true;
  std::array<std::string, 2> uv_names{"s", "t"};
  std::array<std::string, 3> color_names{"red", "green", "blue"};
};
Result<ReadMesh> read_ply(ByteView data, MeshUse use, PlyInfo* info);
Bytes write_ply(const Mesh& mesh, const PlyInfo& info);

/// glTF/GLB through gltfpack; output is always GLB.
/// nullopt: a file gltfpack can't carry over faithfully (Draco, other extensions), passed through.
Result<std::optional<Bytes>> optimize_gltf(ByteView data, Format format, const Options& options);

}  // namespace pixie::detail
