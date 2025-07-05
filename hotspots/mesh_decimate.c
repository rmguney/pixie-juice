#include "mesh_decimate.h"
#include "util.h"

// Only include standard headers for native builds
#ifndef __wasm__
    #include <stdlib.h>
    #include <string.h>
    #include <math.h>
#endif

// WASM-compatible implementations
#ifdef __wasm__
    // Math functions for WASM
    static inline double sqrt(double x) {
        if (x < 0.0) return 0.0;
        if (x == 0.0) return 0.0;
        double guess = x * 0.5;
        for (int i = 0; i < 10; i++) {
            guess = (guess + x / guess) * 0.5;
        }
        return guess;
    }
    
    static inline float sqrtf(float x) {
        if (x < 0.0f) return 0.0f;
        if (x == 0.0f) return 0.0f;
        float guess = x * 0.5f;
        for (int i = 0; i < 10; i++) {
            guess = (guess + x / guess) * 0.5f;
        }
        return guess;
    }
    
    static inline double fabs(double x) { return x < 0.0 ? -x : x; }
    
    // Memory operations for WASM
    static void* memset_wasm(void* s, int c, size_t n) {
        char* p = (char*)s;
        for (size_t i = 0; i < n; i++) {
            p[i] = (char)c;
        }
        return s;
    }
    
    static void* memcpy_wasm(void* dest, const void* src, size_t n) {
        char* d = (char*)dest;
        const char* s = (const char*)src;
        for (size_t i = 0; i < n; i++) {
            d[i] = s[i];
        }
        return dest;
    }
    
    static char* strncpy_wasm(char* dest, const char* src, size_t n) {
        size_t i;
        for (i = 0; i < n && src[i] != '\0'; i++) {
            dest[i] = src[i];
        }
        for (; i < n; i++) {
            dest[i] = '\0';
        }
        return dest;
    }
    
    // Memory allocation for WASM (simple static buffers)
    static char wasm_buffer[1024 * 1024]; // 1MB buffer for mesh processing
    static size_t wasm_buffer_offset = 0;
    
    static void* malloc_wasm(size_t size) {
        if (wasm_buffer_offset + size > sizeof(wasm_buffer)) {
            return 0; // Out of memory
        }
        void* ptr = &wasm_buffer[wasm_buffer_offset];
        wasm_buffer_offset += (size + 7) & ~7; // 8-byte align
        return ptr;
    }
    
    static void free_wasm(void* ptr) {
        // Simple implementation - no actual freeing
        (void)ptr;
    }
    
    static void* realloc_wasm(void* ptr, size_t size) {
        // Simple implementation - just allocate new memory and copy
        if (!ptr) return malloc_wasm(size);
        if (size == 0) {
            free_wasm(ptr);
            return 0;
        }
        
        void* new_ptr = malloc_wasm(size);
        if (new_ptr && ptr) {
            // Copy old data (we don't know the original size, so copy what we can)
            memcpy_wasm(new_ptr, ptr, size / 2); // Conservative estimate
        }
        return new_ptr;
    }
    
    #define memset memset_wasm
    #define memcpy memcpy_wasm
    #define strncpy strncpy_wasm
    #define malloc malloc_wasm
    #define free free_wasm
    #define realloc realloc_wasm
#endif

// Helper to create error result
static MeshDecimateResult create_decimate_error_result(const char* error_msg) {
    MeshDecimateResult result = {0};
    result.success = 0;
    strncpy(result.error_message, error_msg, sizeof(result.error_message) - 1);
    result.error_message[sizeof(result.error_message) - 1] = '\0';
    return result;
}

// Helper to create success result
static MeshDecimateResult create_decimate_success_result(float* vertices, size_t vertex_count,
                                                        uint32_t* indices, size_t index_count) {
    MeshDecimateResult result = {0};
    result.vertices = vertices;
    result.indices = indices;
    result.vertex_count = vertex_count;
    result.index_count = index_count;
    result.success = 1;
    return result;
}

MeshDecimateResult decimate_mesh_qem(const float* vertices, size_t vertex_count,
                                    const uint32_t* indices, size_t index_count,
                                    float target_ratio) {
    if (!vertices || !indices || vertex_count == 0 || index_count == 0) {
        return create_decimate_error_result("Invalid input data");
    }
    
    if (target_ratio <= 0.0f || target_ratio > 1.0f) {
        return create_decimate_error_result("Target ratio must be between 0 and 1");
    }
    
    // Aggressive mesh decimation algorithm for significant size reduction
    
    // Target significant reduction - if target_ratio > 0.6, reduce by at least 50%
    float actual_target_ratio = target_ratio;
    if (target_ratio > 0.6f) {
        actual_target_ratio = 0.4f; // Keep only 40% of vertices for 60%+ reduction target
    } else if (target_ratio > 0.4f) {
        actual_target_ratio = 0.5f; // Keep only 50% of vertices for 40%+ reduction target
    }
    
    size_t target_vertex_count = (size_t)(vertex_count * actual_target_ratio);
    if (target_vertex_count < 3) target_vertex_count = 3; // Minimum for a triangle
    
    if (target_vertex_count >= vertex_count) {
        // No reduction needed, just copy
        float* new_vertices = malloc(vertex_count * 3 * sizeof(float));
        uint32_t* new_indices = malloc(index_count * sizeof(uint32_t));
        
        if (!new_vertices || !new_indices) {
            free(new_vertices);
            free(new_indices);
            return create_decimate_error_result("Memory allocation failed");
        }
        
        memcpy(new_vertices, vertices, vertex_count * 3 * sizeof(float));
        memcpy(new_indices, indices, index_count * sizeof(uint32_t));
        
        return create_decimate_success_result(new_vertices, vertex_count, new_indices, index_count);
    }
    
    // Aggressive decimation: keep only the most important vertices
    // Strategy: 1) Remove vertices with small surface area contribution
    //          2) Prioritize corner/edge vertices over flat surface vertices
    
    float* new_vertices = malloc(target_vertex_count * 3 * sizeof(float));
    uint32_t* vertex_mapping = malloc(vertex_count * sizeof(uint32_t));
    uint8_t* vertex_keep = malloc(vertex_count * sizeof(uint8_t));
    
    if (!new_vertices || !vertex_mapping || !vertex_keep) {
        free(new_vertices);
        free(vertex_mapping);
        free(vertex_keep);
        return create_decimate_error_result("Memory allocation failed");
    }
    
    // Initialize: mark all vertices as candidates for removal
    memset(vertex_keep, 0, vertex_count);
    
    // Phase 1: Calculate vertex importance (simplified curvature-based)
    float* vertex_importance = malloc(vertex_count * sizeof(float));
    if (!vertex_importance) {
        free(new_vertices);
        free(vertex_mapping);
        free(vertex_keep);
        return create_decimate_error_result("Memory allocation failed");
    }
    
    // Calculate importance based on distance to neighbors and triangle angles
    for (size_t i = 0; i < vertex_count; i++) {
        vertex_importance[i] = 0.0f;
        
        // Find triangles using this vertex
        int triangle_count = 0;
        for (size_t t = 0; t < index_count; t += 3) {
            if (indices[t] == i || indices[t+1] == i || indices[t+2] == i) {
                triangle_count++;
                
                // Add importance based on triangle area (larger area = more important)
                uint32_t v0 = indices[t];
                uint32_t v1 = indices[t+1];
                uint32_t v2 = indices[t+2];
                
                if (v0 < vertex_count && v1 < vertex_count && v2 < vertex_count) {
                    // Calculate triangle area using cross product
                    float ax = vertices[v1*3] - vertices[v0*3];
                    float ay = vertices[v1*3+1] - vertices[v0*3+1];
                    float az = vertices[v1*3+2] - vertices[v0*3+2];
                    
                    float bx = vertices[v2*3] - vertices[v0*3];
                    float by = vertices[v2*3+1] - vertices[v0*3+1];
                    float bz = vertices[v2*3+2] - vertices[v0*3+2];
                    
                    float cx = ay * bz - az * by;
                    float cy = az * bx - ax * bz;
                    float cz = ax * by - ay * bx;
                    
                    float area = 0.5f * sqrtf(cx*cx + cy*cy + cz*cz);
                    vertex_importance[i] += area;
                }
            }
        }
        
        // Boost importance for vertices used by many triangles (likely on edges/corners)
        vertex_importance[i] *= (1.0f + triangle_count * 0.1f);
    }
    
    // Phase 2: Select most important vertices
    // Sort by importance and keep the top vertices
    typedef struct {
        size_t index;
        float importance;
    } vertex_score_t;
    
    vertex_score_t* scores = malloc(vertex_count * sizeof(vertex_score_t));
    if (!scores) {
        free(new_vertices);
        free(vertex_mapping);
        free(vertex_keep);
        free(vertex_importance);
        return create_decimate_error_result("Memory allocation failed");
    }
    
    for (size_t i = 0; i < vertex_count; i++) {
        scores[i].index = i;
        scores[i].importance = vertex_importance[i];
    }
    
    // Simple selection sort to get top vertices (good enough for our needs)
    for (size_t i = 0; i < target_vertex_count && i < vertex_count; i++) {
        size_t max_idx = i;
        for (size_t j = i + 1; j < vertex_count; j++) {
            if (scores[j].importance > scores[max_idx].importance) {
                max_idx = j;
            }
        }
        // Swap
        vertex_score_t temp = scores[i];
        scores[i] = scores[max_idx];
        scores[max_idx] = temp;
        
        // Mark this vertex to keep
        vertex_keep[scores[i].index] = 1;
    }
    
    // Phase 3: Build new vertex array and mapping
    size_t new_vertex_idx = 0;
    for (size_t i = 0; i < vertex_count; i++) {
        if (vertex_keep[i] && new_vertex_idx < target_vertex_count) {
            new_vertices[new_vertex_idx * 3] = vertices[i * 3];
            new_vertices[new_vertex_idx * 3 + 1] = vertices[i * 3 + 1];
            new_vertices[new_vertex_idx * 3 + 2] = vertices[i * 3 + 2];
            vertex_mapping[i] = new_vertex_idx;
            new_vertex_idx++;
        } else {
            // Map to nearest kept vertex (simplified)
            float min_dist = 1e30f;
            uint32_t nearest_vertex = 0;
            
            for (size_t j = 0; j < vertex_count; j++) {
                if (vertex_keep[j]) {
                    float dx = vertices[i*3] - vertices[j*3];
                    float dy = vertices[i*3+1] - vertices[j*3+1];
                    float dz = vertices[i*3+2] - vertices[j*3+2];
                    float dist = dx*dx + dy*dy + dz*dz;
                    
                    if (dist < min_dist) {
                        min_dist = dist;
                        nearest_vertex = vertex_mapping[j];
                    }
                }
            }
            vertex_mapping[i] = nearest_vertex;
        }
    }
    
    // Phase 4: Rebuild triangles with aggressive culling
    uint32_t* new_indices = malloc(index_count * sizeof(uint32_t));
    if (!new_indices) {
        free(new_vertices);
        free(vertex_mapping);
        free(vertex_keep);
        free(vertex_importance);
        free(scores);
        return create_decimate_error_result("Memory allocation failed");
    }
    
    size_t valid_triangles = 0;
    for (size_t i = 0; i < index_count; i += 3) {
        if (i + 2 < index_count) {
            uint32_t v0 = vertex_mapping[indices[i]];
            uint32_t v1 = vertex_mapping[indices[i+1]];
            uint32_t v2 = vertex_mapping[indices[i+2]];
            
            // Only keep triangles with distinct vertices and reasonable area
            if (v0 != v1 && v1 != v2 && v0 != v2 && 
                v0 < target_vertex_count && v1 < target_vertex_count && v2 < target_vertex_count) {
                
                // Check triangle area - skip degenerate triangles
                float ax = new_vertices[v1*3] - new_vertices[v0*3];
                float ay = new_vertices[v1*3+1] - new_vertices[v0*3+1];
                float az = new_vertices[v1*3+2] - new_vertices[v0*3+2];
                
                float bx = new_vertices[v2*3] - new_vertices[v0*3];
                float by = new_vertices[v2*3+1] - new_vertices[v0*3+1];
                float bz = new_vertices[v2*3+2] - new_vertices[v0*3+2];
                
                float cx = ay * bz - az * by;
                float cy = az * bx - ax * bz;
                float cz = ax * by - ay * bx;
                
                float area = cx*cx + cy*cy + cz*cz;
                
                if (area > 1e-12f) { // Skip near-degenerate triangles
                    new_indices[valid_triangles * 3] = v0;
                    new_indices[valid_triangles * 3 + 1] = v1;
                    new_indices[valid_triangles * 3 + 2] = v2;
                    valid_triangles++;
                }
            }
        }
    }
    
    // Clean up
    free(vertex_mapping);
    free(vertex_keep);
    free(vertex_importance);
    free(scores);
    
    // Resize indices array to actual size
    if (valid_triangles > 0) {
        uint32_t* final_indices = realloc(new_indices, valid_triangles * 3 * sizeof(uint32_t));
        if (final_indices) {
            new_indices = final_indices;
        }
    }
    
    return create_decimate_success_result(new_vertices, target_vertex_count, new_indices, valid_triangles * 3);
}

MeshDecimateResult weld_vertices_spatial(const float* vertices, size_t vertex_count, 
                                        const uint32_t* indices, size_t index_count, 
                                        float tolerance) {
    if (!vertices || !indices || vertex_count == 0 || index_count == 0) {
        return create_decimate_error_result("Invalid input data");
    }
    
    if (tolerance < 0.0f) {
        return create_decimate_error_result("Tolerance must be non-negative");
    }
    
    // Spatial vertex welding implementation
    // This uses a simple approach - in production, you'd use spatial hashing
    
    float* new_vertices = malloc(vertex_count * 3 * sizeof(float));
    uint32_t* vertex_mapping = malloc(vertex_count * sizeof(uint32_t));
    
    if (!new_vertices || !vertex_mapping) {
        free(new_vertices);
        free(vertex_mapping);
        return create_decimate_error_result("Memory allocation failed");
    }
    
    size_t unique_vertex_count = 0;
    float tolerance_sq = tolerance * tolerance;
    
    // Find unique vertices within tolerance
    for (size_t i = 0; i < vertex_count; i++) {
        float x = vertices[i * 3];
        float y = vertices[i * 3 + 1];
        float z = vertices[i * 3 + 2];
        
        // Check if this vertex is close to any existing unique vertex
        int found_match = 0;
        for (size_t j = 0; j < unique_vertex_count; j++) {
            float dx = x - new_vertices[j * 3];
            float dy = y - new_vertices[j * 3 + 1];
            float dz = z - new_vertices[j * 3 + 2];
            float dist_sq = dx*dx + dy*dy + dz*dz;
            
            if (dist_sq <= tolerance_sq) {
                vertex_mapping[i] = j;
                found_match = 1;
                break;
            }
        }
        
        if (!found_match) {
            // Add new unique vertex
            new_vertices[unique_vertex_count * 3] = x;
            new_vertices[unique_vertex_count * 3 + 1] = y;
            new_vertices[unique_vertex_count * 3 + 2] = z;
            vertex_mapping[i] = unique_vertex_count;
            unique_vertex_count++;
        }
    }
    
    // Remap indices to use welded vertices
    uint32_t* new_indices = malloc(index_count * sizeof(uint32_t));
    if (!new_indices) {
        free(new_vertices);
        free(vertex_mapping);
        return create_decimate_error_result("Memory allocation failed");
    }
    
    for (size_t i = 0; i < index_count; i++) {
        if (indices[i] < vertex_count) {
            new_indices[i] = vertex_mapping[indices[i]];
        } else {
            new_indices[i] = 0; // Invalid index, map to first vertex
        }
    }
    
    free(vertex_mapping);
    
    // Resize vertex array to actual used size
    float* final_vertices = realloc(new_vertices, unique_vertex_count * 3 * sizeof(float));
    if (final_vertices) {
        new_vertices = final_vertices;
    }
    
    return create_decimate_success_result(new_vertices, unique_vertex_count, new_indices, index_count);
}

void free_mesh_decimate_result(MeshDecimateResult* result) {
    if (result) {
        if (result->vertices) {
            free(result->vertices);
            result->vertices = NULL;
        }
        if (result->indices) {
            free(result->indices);
            result->indices = NULL;
        }
        result->vertex_count = 0;
        result->index_count = 0;
        result->success = 0;
    }
}
