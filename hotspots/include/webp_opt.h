#ifndef WEBP_OPT_H
#define WEBP_OPT_H

#include <stddef.h>  // For size_t

#ifdef __cplusplus
extern "C" {
#endif

// WASM compatibility
#ifdef __wasm__
#define WASM_EXPORT __attribute__((visibility("default")))
#else
#define WASM_EXPORT
#endif

// Configuration for WebP optimization
typedef struct {
    int quality;           // 0-100, where 100 is lossless
    int method;           // 0-6, compression method (0=fast, 6=slower but better)
    int use_lossless;     // 0=lossy, 1=lossless
    int alpha_quality;    // 0-100, alpha channel quality
    int preserve_alpha;   // 0=no, 1=yes
    int optimize_filters; // 0=no, 1=yes - optimize filtering
    int use_sharp_yuv;    // 0=no, 1=yes - use sharp YUV conversion
} WebPOptConfig;

// Result structure
typedef struct {
    unsigned char* data;
    size_t size;
    int error_code;
    char* error_message;
} WebPOptResult;

// WebP image information
typedef struct {
    int width, height;
    int has_alpha;
    int is_lossless;
    int format_version;
} WebPImageInfo;

// Main optimization function
WASM_EXPORT WebPOptResult webp_optimize_c(const unsigned char* input_data, size_t input_len, 
                                          const WebPOptConfig* config);

// Free result memory
WASM_EXPORT void webp_opt_result_free(WebPOptResult* result);

// WebP analysis functions
WASM_EXPORT int webp_has_alpha_channel(const unsigned char* data, size_t size);
WASM_EXPORT int analyze_webp_header(const unsigned char* data, size_t size, WebPImageInfo* info);

// Utility functions for WASM environment
WASM_EXPORT int webp_get_info(const unsigned char* data, size_t size, int* width, int* height, int* has_alpha);

// WASM-compatible string functions (no stdlib dependency)
int wasm_strcmp_webp(const char* str1, const char* str2);
void wasm_strcpy_webp(char* dest, const char* src);
int wasm_strlen_webp(const char* str);

#ifdef __cplusplus
}
#endif

#endif // WEBP_OPT_H
