// Mesh optimization with meshoptimizer: exact vertex welding, optional
// simplification, then vertex cache, overdraw and vertex fetch ordering.

#include "internal.hpp"

#include <meshoptimizer.h>

#include <algorithm>

namespace pixie::detail {

namespace {

constexpr float kSimplifyError = 1e-2f;  // relative to the mesh extent, as gltfpack
constexpr float kOverdrawThreshold = 1.05f;

template <class T>
void remap_attribute(std::vector<T>& values, std::size_t components, const std::vector<unsigned>& remap,
                     std::size_t vertex_count, std::size_t unique) {
  if (values.empty()) return;
  std::vector<T> out(unique * components);
  meshopt_remapVertexBuffer(out.data(), values.data(), vertex_count, sizeof(T) * components, remap.data());
  values = std::move(out);
}

void remap_all(Mesh& mesh, const std::vector<unsigned>& remap, std::size_t unique) {
  const std::size_t n = mesh.vertex_count();
  remap_attribute(mesh.positions, 3, remap, n, unique);
  remap_attribute(mesh.normals, 3, remap, n, unique);
  remap_attribute(mesh.uvs, 2, remap, n, unique);
  remap_attribute(mesh.colors, 3, remap, n, unique);
  meshopt_remapIndexBuffer(mesh.indices.data(), mesh.indices.data(), mesh.indices.size(), remap.data());
}

}  // namespace

Mesh optimize_mesh(const Mesh& input, float ratio) {
  Mesh mesh = input;
  const std::size_t vertex_count = mesh.vertex_count();
  if (vertex_count == 0 || mesh.indices.empty()) return mesh;

  // 1. Weld vertices whose every attribute is bit-identical.
  std::vector<meshopt_Stream> streams{{mesh.positions.data(), sizeof(float) * 3, sizeof(float) * 3}};
  if (!mesh.normals.empty()) streams.push_back({mesh.normals.data(), sizeof(float) * 3, sizeof(float) * 3});
  if (!mesh.uvs.empty()) streams.push_back({mesh.uvs.data(), sizeof(float) * 2, sizeof(float) * 2});
  if (!mesh.colors.empty()) streams.push_back({mesh.colors.data(), sizeof(float) * 3, sizeof(float) * 3});
  std::vector<unsigned> remap(vertex_count);
  const std::size_t unique = meshopt_generateVertexRemapMulti(remap.data(), mesh.indices.data(), mesh.indices.size(),
                                                              vertex_count, streams.data(), streams.size());
  remap_all(mesh, remap, unique);

  // 2. Per material group: simplify, then order for the vertex cache and overdraw.
  const std::size_t groups = mesh.group_names.empty() ? 1 : mesh.group_names.size();
  std::vector<std::vector<unsigned>> group_indices(groups);
  for (std::size_t t = 0; t < mesh.triangle_count(); ++t) {
    const std::size_t g = mesh.triangle_groups.empty() ? 0 : mesh.triangle_groups[t];
    group_indices[g].insert(group_indices[g].end(), mesh.indices.begin() + std::ptrdiff_t(t * 3),
                            mesh.indices.begin() + std::ptrdiff_t(t * 3 + 3));
  }

  std::vector<unsigned> indices;
  std::vector<std::uint32_t> triangle_groups;
  for (std::size_t g = 0; g < groups; ++g) {
    auto& idx = group_indices[g];
    if (idx.empty()) continue;
    if (ratio < 1.0f) {
      const std::size_t target = std::max<std::size_t>(3, std::size_t(double(idx.size()) * ratio) / 3 * 3);
      std::vector<unsigned> simplified(idx.size());
      float error = 0;
      simplified.resize(meshopt_simplify(simplified.data(), idx.data(), idx.size(), mesh.positions.data(), unique,
                                         sizeof(float) * 3, target, kSimplifyError, 0, &error));
      if (!simplified.empty()) idx = std::move(simplified);
    }
    meshopt_optimizeVertexCache(idx.data(), idx.data(), idx.size(), unique);
    meshopt_optimizeOverdraw(idx.data(), idx.data(), idx.size(), mesh.positions.data(), unique, sizeof(float) * 3,
                             kOverdrawThreshold);
    indices.insert(indices.end(), idx.begin(), idx.end());
    if (!mesh.group_names.empty()) triangle_groups.insert(triangle_groups.end(), idx.size() / 3, std::uint32_t(g));
  }
  mesh.indices = std::move(indices);
  mesh.triangle_groups = std::move(triangle_groups);

  // 3. Order vertices by first use (also drops vertices simplification left unused).
  std::vector<unsigned> fetch(unique);
  const std::size_t used = meshopt_optimizeVertexFetchRemap(fetch.data(), mesh.indices.data(), mesh.indices.size(), unique);
  remap_all(mesh, fetch, used);
  return mesh;
}

}  // namespace pixie::detail
