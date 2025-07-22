#ifndef MESH_OPT_H
#define MESH_OPT_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Mesh optimization result structure
typedef struct {
    float* vertices;
    uint32_t* indices;
    size_t vertex_count;
    size_t index_count;
    int success;
    char error_message[256];
} MeshOptResult;

// Performance-critical mesh decimation hotspot
// Uses advanced algorithms like quadric error metrics for high-quality reduction
MeshOptResult decimate_mesh(const float* vertices, size_t vertex_count,
                           const uint32_t* indices, size_t index_count,
                           float target_ratio);

// Optimized vertex welding with spatial hashing
MeshOptResult weld_vertices(const float* vertices, size_t vertex_count, 
                           const uint32_t* indices, size_t index_count, 
                           float tolerance);

// Fast mesh validation for topology checks
int validate_mesh_topology(const float* vertices, size_t vertex_count,
                          const uint32_t* indices, size_t index_count);

// Memory cleanup
void free_mesh_result(MeshOptResult* result);

#ifdef __cplusplus
}
#endif

#endif // MESH_OPT_H
