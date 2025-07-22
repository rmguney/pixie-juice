/*
 * PNG Optimization Header - WASM-compatible PNG optimization
 */

#ifndef PNG_OPT_H
#define PNG_OPT_H

#include <stdint.h>
#include <stddef.h>

// For WASM, avoid stdlib - use our own memory management
#ifdef __wasm32__
// WASM memory functions - externally provided
extern void* wasm_malloc(size_t size);
extern void wasm_free(void* ptr);
extern void* wasm_memcpy(void* dest, const void* src, size_t n);
extern void* wasm_memset(void* dest, int value, size_t n);
extern int wasm_memcmp(const void* s1, const void* s2, size_t n);

#define malloc wasm_malloc
#define free wasm_free
#define memcpy wasm_memcpy
#define memset wasm_memset
#define memcmp wasm_memcmp

// Simple strlen for WASM
static inline size_t wasm_strlen(const char* s) {
    size_t len = 0;
    while (s[len]) len++;
    return len;
}

// Simple strcmp for WASM
static inline int wasm_strcmp(const char* s1, const char* s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

// Simple strcpy for WASM
static inline char* wasm_strcpy(char* dest, const char* src) {
    char* orig_dest = dest;
    while ((*dest++ = *src++));
    return orig_dest;
}

#define strlen wasm_strlen
#define strcmp wasm_strcmp
#define strcpy wasm_strcpy

#else
#include <stdlib.h>
#include <string.h>
#endif

#ifdef __cplusplus
extern "C" {
#endif

// PNG optimization configuration
typedef struct {
    int compress_level;      // 0-9, compression level
    int reduce_colors;       // 0 or 1, whether to reduce color palette
    int max_colors;         // Maximum colors in palette (2-256)
    int strip_metadata;     // 0 or 1, whether to remove metadata chunks
    int optimize_filters;   // 0 or 1, whether to optimize PNG filters
} PngOptConfig;

// PNG optimization result
typedef struct {
    uint8_t* output_data;           // Optimized PNG data (caller must free)
    size_t output_size;             // Size of optimized data
    double compression_ratio;       // output_size / input_size
    int error_code;                // 0 = success, negative = error
    char error_message[256];       // Error description if error_code != 0
} PngOptResult;

// Main PNG optimization function
PngOptResult png_optimize_c(const uint8_t* input_data, size_t input_len, 
                           const PngOptConfig* config);

// Free the result data
void png_opt_result_free(PngOptResult* result);

// Utility functions for color analysis
int analyze_png_colors(const uint8_t* rgba_data, size_t pixel_count, 
                      int* has_transparency, int* unique_colors);

// Check if PNG has alpha channel or transparency
int png_has_alpha_channel(const uint8_t* png_data, size_t png_size);

#ifdef __cplusplus
}
#endif

#endif // PNG_OPT_H
