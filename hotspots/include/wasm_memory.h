#ifndef WASM_MEMORY_H
#define WASM_MEMORY_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

// WASM compatibility
#ifdef __wasm__
#define WASM_EXPORT __attribute__((visibility("default")))
#else
#define WASM_EXPORT
#endif

#ifdef __wasm32__

// WASM memory management functions
WASM_EXPORT void* wasm_malloc(size_t size);
WASM_EXPORT void wasm_free(void* ptr);
WASM_EXPORT void* wasm_memcpy(void* dest, const void* src, size_t n);
WASM_EXPORT void* wasm_memset(void* s, int c, size_t n);
WASM_EXPORT int wasm_memcmp(const void* s1, const void* s2, size_t n);

#else

// For native builds, use standard library functions
#include <stdlib.h>
#include <string.h>

#define wasm_malloc malloc
#define wasm_free free
#define wasm_memcpy memcpy
#define wasm_memset memset
#define wasm_memcmp memcmp

#endif

#ifdef __cplusplus
}
#endif

#endif // WASM_MEMORY_H
