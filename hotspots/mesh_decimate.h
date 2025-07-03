#ifndef MESH_DECIMATE_H
#define MESH_DECIMATE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Mesh decimation result structure
typedef struct {
    float* vertices;
    uint32_t* indices;
    size_t vertex_count;
    size_t index_count;
    int success;
    char error_message[256];
} MeshDecimateResult;

// High-performance mesh decimation using quadric error metrics
MeshDecimateResult decimate_mesh_qem(const float* vertices, size_t vertex_count,
                                    const uint32_t* indices, size_t index_count,
                                    float target_ratio);

// Fast vertex welding with spatial hashing for performance
MeshDecimateResult weld_vertices_spatial(const float* vertices, size_t vertex_count, 
                                        const uint32_t* indices, size_t index_count, 
                                        float tolerance);

// Memory cleanup
void free_mesh_decimate_result(MeshDecimateResult* result);

#ifdef __cplusplus
}
#endif

#endif // MESH_DECIMATE_H
