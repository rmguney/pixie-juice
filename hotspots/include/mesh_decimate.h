#ifndef MESH_DECIMATE_H
#define MESH_DECIMATE_H

#include "memory.h"
#include "util.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float* vertices;
    uint32_t* indices;
    size_t vertex_count;
    size_t index_count;
    int success;
    char error_message[256];
} MeshDecimateResult;

WASM_EXPORT MeshDecimateResult decimate_mesh_qem(const float* vertices, size_t vertex_count,
                                    const uint32_t* indices, size_t index_count,
                                    float target_ratio);

WASM_EXPORT MeshDecimateResult weld_vertices_spatial(const float* vertices, size_t vertex_count, 
                                        const uint32_t* indices, size_t index_count, 
                                        float tolerance);

WASM_EXPORT void free_mesh_decimate_result(MeshDecimateResult* result);

#ifdef __cplusplus
}
#endif

#endif
