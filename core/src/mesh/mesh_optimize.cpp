// Mesh optimization with meshoptimizer: exact vertex welding, optional
// simplification, then vertex cache, overdraw and vertex fetch ordering.

#include "internal.hpp"

#include <meshoptimizer.h>

#include <algorithm>

namespace pixie::detail {

namespace {

constexpr float kSimplifyError = 1e-2f;  // relative to the mesh extent, as gltfpack
constexpr float kOverdrawThreshold = 1.05f;
constexpr unsigned kNone = ~0u;

template <class T>
std::vector<T> remapped(const std::vector<T>& values, std::size_t components, const std::vector<unsigned>& remap,
                        std::size_t unique) {
  if (values.empty()) return {};
  std::vector<T> out(unique * components);
  meshopt_remapVertexBuffer(out.data(), values.data(), values.size() / components, sizeof(T) * components, remap.data());
  return out;
}

/// `from`'s vertices and indices through `remap` into `to` (which may be `from`).
void remap_all(Mesh& to, const Mesh& from, const std::vector<unsigned>& remap, std::size_t unique) {
  to.positions = remapped(from.positions, 3, remap, unique);
  to.normals = remapped(from.normals, 3, remap, unique);
  to.uvs = remapped(from.uvs, 2, remap, unique);
  to.colors = remapped(from.colors, 3, remap, unique);
  to.indices.resize(from.indices.size());
  meshopt_remapIndexBuffer(to.indices.data(), from.indices.data(), from.indices.size(), remap.data());
}

}  // namespace

Mesh optimize_mesh(const Mesh& input, float ratio) {
  if (input.vertex_count() == 0 || input.indices.empty()) return input;

  // 1. Weld vertices whose every attribute is bit-identical.
  std::vector<meshopt_Stream> streams{{input.positions.data(), sizeof(float) * 3, sizeof(float) * 3}};
  if (!input.normals.empty()) streams.push_back({input.normals.data(), sizeof(float) * 3, sizeof(float) * 3});
  if (!input.uvs.empty()) streams.push_back({input.uvs.data(), sizeof(float) * 2, sizeof(float) * 2});
  if (!input.colors.empty()) streams.push_back({input.colors.data(), sizeof(float) * 3, sizeof(float) * 3});
  std::vector<unsigned> remap(input.vertex_count());
  const std::size_t unique = meshopt_generateVertexRemapMulti(remap.data(), input.indices.data(), input.indices.size(),
                                                              input.vertex_count(), streams.data(), streams.size());
  Mesh mesh;
  mesh.group_names = input.group_names;
  remap_all(mesh, input, remap, unique);

  // 2. Per material group: simplify, then order for the vertex cache and overdraw.
  const std::size_t groups = mesh.group_names.empty() ? 1 : mesh.group_names.size();
  std::vector<std::vector<unsigned>> group_indices(groups);
  // A position used by more than one group (even with other normals or UVs)
  // stays put when simplifying, so the groups still meet.
  std::vector<unsigned> position_of(unique);
  meshopt_generatePositionRemap(position_of.data(), mesh.positions.data(), unique, sizeof(float) * 3);
  std::vector<unsigned> owner(unique, kNone);
  std::vector<unsigned char> shared(unique, 0);
  for (std::size_t t = 0; t < mesh.triangle_count(); ++t) {
    const auto g = input.triangle_groups.empty() ? 0u : input.triangle_groups[t];
    for (std::size_t k = 0; k < 3; ++k) {
      const unsigned v = mesh.indices[t * 3 + k];
      group_indices[g].push_back(v);
      const unsigned p = position_of[v];
      if (owner[p] == kNone) owner[p] = g;
      else if (owner[p] != g) shared[p] = 1;
    }
  }
  // Errors are measured against the whole mesh, whatever the group's own size.
  const float error = kSimplifyError * meshopt_simplifyScale(mesh.positions.data(), unique, sizeof(float) * 3);

  // Each group is processed on its own vertices only, renumbered from 0.
  std::vector<unsigned> local_of(unique, kNone);
  std::vector<unsigned> indices;
  std::vector<std::uint32_t> triangle_groups;
  for (std::size_t g = 0; g < groups; ++g) {
    auto& idx = group_indices[g];
    if (idx.empty()) continue;
    std::vector<unsigned> vertices;
    for (auto& v : idx) {
      if (local_of[v] == kNone) {
        local_of[v] = unsigned(vertices.size());
        vertices.push_back(v);
      }
      v = local_of[v];
    }
    std::vector<float> positions(vertices.size() * 3);
    std::vector<unsigned char> locked(vertices.size());
    for (std::size_t i = 0; i < vertices.size(); ++i) {
      std::copy_n(mesh.positions.begin() + std::ptrdiff_t(std::size_t(vertices[i]) * 3), 3,
                  positions.begin() + std::ptrdiff_t(i * 3));
      locked[i] = shared[position_of[vertices[i]]];
      local_of[vertices[i]] = kNone;  // ready for the next group
    }

    if (ratio < 1.0f) {
      const std::size_t target = std::max<std::size_t>(3, std::size_t(double(idx.size()) * ratio) / 3 * 3);
      std::vector<unsigned> simplified(idx.size());
      simplified.resize(meshopt_simplifyWithAttributes(simplified.data(), idx.data(), idx.size(), positions.data(),
                                                       vertices.size(), sizeof(float) * 3, nullptr, 0, nullptr, 0,
                                                       locked.data(), target, error, meshopt_SimplifyErrorAbsolute,
                                                       nullptr));
      if (!simplified.empty()) idx = std::move(simplified);
    }
    meshopt_optimizeVertexCache(idx.data(), idx.data(), idx.size(), vertices.size());
    meshopt_optimizeOverdraw(idx.data(), idx.data(), idx.size(), positions.data(), vertices.size(), sizeof(float) * 3,
                             kOverdrawThreshold);
    for (const unsigned v : idx) indices.push_back(vertices[v]);
    if (!mesh.group_names.empty()) triangle_groups.insert(triangle_groups.end(), idx.size() / 3, std::uint32_t(g));
  }
  mesh.indices = std::move(indices);
  mesh.triangle_groups = std::move(triangle_groups);

  // 3. Order vertices by first use (also drops vertices simplification left unused).
  std::vector<unsigned> fetch(unique);
  const std::size_t used = meshopt_optimizeVertexFetchRemap(fetch.data(), mesh.indices.data(), mesh.indices.size(), unique);
  remap_all(mesh, mesh, fetch, used);
  return mesh;
}

}  // namespace pixie::detail
