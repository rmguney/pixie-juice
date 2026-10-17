#include "results.hpp"

#include <cgltf.h>
#include <doctest/doctest.h>

using namespace pixie;
using fixtures::as_text;
using fixtures::lines_starting;
using fixtures::optimized;
using fixtures::text_bytes;

namespace {

Bytes binary_stl(const detail::Mesh& mesh) { return detail::write_stl(mesh, {.binary = true}); }

detail::Mesh grid_mesh(int n) {
  auto mesh = detail::read_obj(text_bytes(fixtures::grid_obj(n)), detail::MeshUse::Optimize);
  REQUIRE(mesh.has_value());
  REQUIRE(mesh->exact);
  return mesh->mesh;
}

std::size_t glb_triangles(const Bytes& glb) {
  cgltf_options options{};
  cgltf_data* data = nullptr;
  REQUIRE(cgltf_parse(&options, glb.data(), glb.size(), &data) == cgltf_result_success);
  REQUIRE(cgltf_load_buffers(&options, data, nullptr) == cgltf_result_success);
  CHECK(cgltf_validate(data) == cgltf_result_success);
  std::size_t triangles = 0;
  for (std::size_t m = 0; m < data->meshes_count; ++m) {
    for (std::size_t p = 0; p < data->meshes[m].primitives_count; ++p) {
      triangles += data->meshes[m].primitives[p].indices->count / 3;
    }
  }
  cgltf_free(data);
  return triangles;
}

}  // namespace

TEST_CASE("OBJ: welds duplicates and keeps every triangle") {
  const auto input = text_bytes(fixtures::grid_obj(20));
  const auto out = optimized(input);
  CHECK(out.changed);
  CHECK(out.format == Format::Obj);
  CHECK(out.data.size() < input.size());
  CHECK(lines_starting(out.data, "f ") == 800);
  CHECK(lines_starting(out.data, "v ") == 21 * 21);
  CHECK(as_text(out.data).find("o grid") != std::string_view::npos);
}

TEST_CASE("OBJ: normals, UVs and materials survive") {
  const auto input = text_bytes(
      "mtllib cube.mtl\nusemtl red\n"
      "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\n"
      "vt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\n"
      "vn 0 0 1\n"
      "f 1/1/1 2/2/1 3/3/1 4/4/1\n"
      "usemtl blue\nf 1/1/1 3/3/1 4/4/1\n");
  auto mesh = detail::read_obj(input, detail::MeshUse::Optimize);
  REQUIRE(mesh.has_value());
  REQUIRE(mesh->exact);
  const auto written = detail::write_obj(detail::optimize_mesh(mesh->mesh, 1.0f), detail::obj_header(input));
  const auto text = as_text(written);
  CHECK(text.find("mtllib cube.mtl") != std::string_view::npos);
  CHECK(text.find("usemtl red") != std::string_view::npos);
  CHECK(text.find("usemtl blue") != std::string_view::npos);
  CHECK(lines_starting(written, "vt ") == 4);
  CHECK(lines_starting(written, "vn ") == 1);
  CHECK(lines_starting(written, "f ") == 3);
  CHECK(text.find("/1 ") != std::string_view::npos);
}

TEST_CASE("OBJ: features the writer can't reproduce pass through") {
  const auto curves = text_bytes("v 0 0 0\nv 1 0 0\nv 0 1 0\ncstype bezier\nf 1 2 3\n");
  CHECK_FALSE(optimized(curves).changed);
  const auto two_objects = text_bytes("o a\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\no b\nf 1 2 3\n");
  CHECK_FALSE(optimized(two_objects).changed);
}

TEST_CASE("simplification reduces triangles when asked") {
  const auto input = text_bytes(fixtures::grid_obj(20));
  const auto out = optimized(input, {.mesh_ratio = 0.25f});
  CHECK(lines_starting(out.data, "f ") < 800);
  CHECK(lines_starting(out.data, "f ") > 0);
}

TEST_CASE("STL: binary stays valid and unchanged without simplification") {
  const auto input = binary_stl(grid_mesh(10));
  CHECK(detect(input) == Format::Stl);
  CHECK_FALSE(optimized(input).changed);

  const auto reduced = optimized(input, {.mesh_ratio = 0.5f});
  CHECK(reduced.changed);
  std::uint32_t count = 0;
  std::memcpy(&count, reduced.data.data() + 80, 4);
  CHECK(reduced.data.size() == 84 + std::size_t(count) * 50);
  CHECK(count < 200);
}

TEST_CASE("STL: ASCII gets compact numbers and keeps its triangles") {
  std::string text = "solid demo\n";
  for (int t = 0; t < 20; ++t) {
    text += "  facet normal 0.000000e+00 0.000000e+00 1.000000e+00\n    outer loop\n";
    for (int k = 0; k < 3; ++k) {
      text += "      vertex " + std::to_string(t + (k == 1)) + ".000000 " + std::to_string(k == 2) + ".000000 0.000000\n";
    }
    text += "    endloop\n  endfacet\n";
  }
  text += "endsolid demo\n";
  const auto out = optimized(text_bytes(text));
  CHECK(out.changed);
  CHECK(out.data.size() < text.size());
  CHECK(fixtures::count(as_text(out.data), "facet normal") == 20);
  CHECK(as_text(out.data).starts_with("solid demo"));
}

TEST_CASE("PLY: ASCII and binary round trips keep triangles and colours") {
  const auto mesh = grid_mesh(8);
  detail::PlyInfo ascii;
  const auto text = detail::write_ply(mesh, ascii);
  CHECK(detect(text) == Format::Ply);
  detail::PlyInfo info;
  auto back = detail::read_ply(text, detail::MeshUse::Optimize, &info);
  REQUIRE(back.has_value());
  REQUIRE(back->exact);
  CHECK(back->mesh.triangle_count() == mesh.triangle_count());
  CHECK_FALSE(info.binary);

  auto coloured = mesh;
  coloured.colors.assign(mesh.positions.size(), 0.0f);
  for (std::size_t i = 0; i < coloured.colors.size(); i += 3) coloured.colors[i] = 1.0f;
  const auto binary = detail::write_ply(coloured, {.binary = true});
  const auto out = optimized(binary);
  detail::PlyInfo out_info;
  auto reread = detail::read_ply(out.data, detail::MeshUse::Optimize, &out_info);
  REQUIRE(reread.has_value());
  REQUIRE(reread->exact);
  CHECK(out_info.binary);
  CHECK(reread->mesh.triangle_count() == mesh.triangle_count());
  CHECK(reread->mesh.colors.size() == reread->mesh.positions.size());
  CHECK(reread->mesh.colors[0] == doctest::Approx(1.0f));
}

TEST_CASE("PLY: unknown vertex properties pass through") {
  const auto input = text_bytes(
      "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\n"
      "property float confidence\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n"
      "0 0 0 1\n1 0 0 1\n0 1 0 1\n3 0 1 2\n");
  CHECK_FALSE(optimized(input).changed);
}

TEST_CASE("glTF: gltfpack produces a valid, smaller GLB with all triangles") {
  const auto input = fixtures::grid_gltf(16);
  CHECK(detect(input) == Format::Gltf);
  const auto out = optimized(input);
  CHECK(out.format == Format::Glb);
  CHECK(out.data.size() < input.size());
  CHECK(glb_triangles(out.data) == 2 * 16 * 16);

  const auto simplified = optimized(input, {.mesh_ratio = 0.25f});
  CHECK(glb_triangles(simplified.data) < 2 * 16 * 16);
}

TEST_CASE("decode_mesh shows files that optimize passes through") {
  // Two objects, a polyline and a w weight: none can be written back exactly.
  const auto obj = text_bytes("o a\nv 0 0 0 1\nv 1 0 0 1\nv 0 1 0 1\nf 1 2 3\no b\nv 0 0 1 1\nf 1 2 4\nl 1 4\n");
  CHECK_FALSE(optimized(obj).changed);
  auto geometry = decode_mesh(obj);
  REQUIRE(geometry.has_value());
  CHECK(geometry->indices.size() == 6);
  CHECK(geometry->lines.size() == 2);  // the polyline's one segment
  CHECK(geometry->colors.empty());

  const auto ply = text_bytes(
      "ply\nformat ascii 1.0\nelement vertex 3\nproperty double x\nproperty double y\nproperty double z\n"
      "property uchar red\nproperty uchar green\nproperty uchar blue\nproperty float confidence\n"
      "element face 1\nproperty list uchar int vertex_indices\nend_header\n"
      "0 0 0 255 0 0 1\n1 0 0 0 255 0 1\n0 1 0 0 0 255 1\n3 0 1 2\n");
  CHECK_FALSE(optimized(ply).changed);
  geometry = decode_mesh(ply);
  REQUIRE(geometry.has_value());
  CHECK(geometry->positions == std::vector<float>{0, 0, 0, 1, 0, 0, 0, 1, 0});
  CHECK(geometry->colors == std::vector<float>{1, 0, 0, 0, 1, 0, 0, 0, 1});
  CHECK(geometry->indices == std::vector<std::uint32_t>{0, 1, 2});

  auto stl = binary_stl(grid_mesh(2));
  stl[84 + 48] = 0x1F;  // a facet colour
  geometry = decode_mesh(stl);
  REQUIRE(geometry.has_value());
  CHECK(geometry->indices.size() == 2 * 2 * 2 * 3);
}

TEST_CASE("decode_mesh shows point clouds and lines, and refuses what has nothing to show") {
  auto cloud = decode_mesh(text_bytes("v 0 0 0 1 0 0\nv 1 0 0 0 1 0\nv 0 1 0 0 0 1\n"));
  REQUIRE(cloud.has_value());
  CHECK(cloud->positions == std::vector<float>{0, 0, 0, 1, 0, 0, 0, 1, 0});
  CHECK(cloud->colors == std::vector<float>{1, 0, 0, 0, 1, 0, 0, 0, 1});
  CHECK(cloud->indices.empty());
  CHECK(cloud->lines.empty());

  auto ply = decode_mesh(text_bytes("ply\nformat ascii 1.0\nelement vertex 2\nproperty float x\nproperty float y\n"
                                    "property float z\nend_header\n0 0 0\n1 1 1\n"));
  REQUIRE(ply.has_value());
  CHECK(ply->positions.size() == 6);
  CHECK(ply->indices.empty());

  auto wire = decode_mesh(text_bytes("v 0 0 0\nv 1 0 0\nv 1 1 0\nl 1 2 3\n"));
  REQUIRE(wire.has_value());
  CHECK(wire->indices.empty());
  REQUIRE(wire->lines.size() == 4);  // two segments
  CHECK(wire->positions.size() == 3 * 3);  // the polyline's three corners, the middle one shared
  CHECK(std::vector<std::uint32_t>(wire->lines.begin(), wire->lines.end()) == std::vector<std::uint32_t>{0, 1, 1, 2});

  CHECK_FALSE(decode_mesh(text_bytes("# nothing\no empty\n")).has_value());
  CHECK_FALSE(decode_mesh(text_bytes(
                  "solid s\nfacet normal 0 0 1\nouter loop\nvertex nan 0 0\nvertex 1 0 0\nvertex 0 1 0\n"
                  "endloop\nendfacet\nendsolid s\n"))
                  .has_value());
  CHECK_FALSE(decode_mesh(fixtures::grid_gltf(2)).has_value());  // glTF is displayed by three.js
}

namespace {

/// grid_gltf with its buffer in a file of its own, "grid data.bin" (percent-encoded in the URI).
std::pair<Bytes, Bytes> gltf_with_bin(std::string_view extra = "") {
  const auto source = fixtures::grid_gltf(4);
  const auto embedded = as_text(source);
  const std::string prefix = "\"uri\":\"data:application/octet-stream;base64,";
  const auto start = embedded.find(prefix) + prefix.size();
  const auto end = embedded.find('"', start);
  // The buffer's bytes, decoded from the data URI.
  const std::string_view base64 = embedded.substr(start, end - start);
  Bytes bin;
  std::uint32_t bits = 0;
  int count = 0;
  for (const char c : base64) {
    if (c == '=') break;
    const auto value = std::string_view("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/").find(c);
    bits = bits << 6 | std::uint32_t(value);
    if ((count += 6) >= 8) {
      count -= 8;
      bin.push_back(std::uint8_t(bits >> count));
    }
  }
  std::string json(embedded.substr(0, start - prefix.size()));
  json += "\"uri\":\"grid%20data.bin\"";
  json += embedded.substr(end + 1);
  json.insert(json.rfind('}'), extra);
  return {text_bytes(json), bin};
}

}  // namespace

TEST_CASE("glTF: the files a .gltf refers to are packed into the GLB") {
  const auto [gltf, bin] = gltf_with_bin();
  CHECK_FALSE(optimize(gltf, {}).has_value());  // without its buffer

  const Resource resources[] = {{"grid data.bin", bin}};
  const auto out = optimize(gltf, {}, resources);
  REQUIRE(out.has_value());
  CHECK(out->changed);
  CHECK(out->format == Format::Glb);

  // A texture found by its file name, under a folder in the URI, ends up inside the GLB.
  const auto png = *detail::encode_png(fixtures::make_image(8, 8, 0));
  const auto [textured, buffer] = gltf_with_bin(
      ",\"images\":[{\"uri\":\"textures/wood.png\"}],\"textures\":[{\"source\":0}],"
      "\"materials\":[{\"pbrMetallicRoughness\":{\"baseColorTexture\":{\"index\":0}}}]");
  const Resource both[] = {{"grid data.bin", buffer}, {"wood.png", png}};
  const auto packed = optimize(textured, {}, both);
  REQUIRE(packed.has_value());
  CHECK(packed->format == Format::Glb);
  const std::string_view glb = as_text(packed->data);
  CHECK(glb.find("\x89PNG") != std::string_view::npos);
  CHECK(glb.find("wood.png") == std::string_view::npos);  // no file reference left

  const Resource just_buffer[] = {{"grid data.bin", buffer}};
  const auto missing = optimize(textured, {}, just_buffer);
  REQUIRE_FALSE(missing.has_value());
  CHECK(missing.error().message.find("wood.png") != std::string::npos);
}

TEST_CASE("glTF: a URI that leaves the folder is refused") {
  for (const char* uri : {"../secret.bin", "/etc/passwd", "C:/x.bin", "a/../../b.bin"}) {
    CAPTURE(std::string(uri));
    auto [gltf, bin] = gltf_with_bin();
    std::string json(as_text(gltf));
    const auto at = json.find("grid%20data.bin");
    json.replace(at, std::strlen("grid%20data.bin"), uri);
    const Resource resources[] = {{"secret.bin", bin}, {"passwd", bin}, {"x.bin", bin}, {"b.bin", bin}};
    const auto out = optimize(text_bytes(json), {}, resources);
    REQUIRE_FALSE(out.has_value());
    CHECK(out.error().message.find("outside its folder") != std::string::npos);
  }
}
