#ifndef VERTEX_CACHE_H
#define VERTEX_CACHE_H

#include "memory.h"
#include "util.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint32_t* indices;
    size_t index_count;
    int success;
    char error_message[256];
} VertexCacheResult;

WASM_EXPORT VertexCacheResult optimize_vertex_cache_forsyth(
    const uint32_t* indices,
    size_t index_count,
    size_t vertex_count,
    uint32_t cache_size
);

WASM_EXPORT void free_vertex_cache_result(VertexCacheResult* result);

#ifdef __cplusplus
}
#endif

#endif
