#ifndef MESH_ATTRIBUTES_H
#define MESH_ATTRIBUTES_H

#include "memory.h"
#include "util.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float* normals;
    float* tangents;
    size_t vertex_count;
    int success;
    char error_message[256];
} MeshAttributesResult;

WASM_EXPORT MeshAttributesResult compute_mesh_attributes(
    const float* vertices,
    size_t vertex_count,
    const uint32_t* indices,
    size_t index_count,
    const float* uvs,
    size_t uv_count,
    int compute_tangents
);

WASM_EXPORT void free_mesh_attributes_result(MeshAttributesResult* result);

#ifdef __cplusplus
}
#endif

#endif
