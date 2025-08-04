#ifndef UTIL_H
#define UTIL_H

#include <stdint.h>
#include <stddef.h>

// WASM-specific definitions
#ifdef __wasm32__
#define WASM_EXPORT __attribute__((visibility("default")))
#define WASM_IMPORT __attribute__((import_module("env")))
// Simplified implementations for WASM
#ifndef NO_COMPLEX_MATH
#define NO_COMPLEX_MATH
#endif
#else
#define WASM_EXPORT
#define WASM_IMPORT
#endif

#ifdef __cplusplus
extern "C" {
#endif

// Buffer utilities
typedef struct {
    uint8_t* data;
    size_t size;
    size_t capacity;
} Buffer;

Buffer* buffer_create(size_t initial_capacity);
int buffer_append(Buffer* buffer, const uint8_t* data, size_t size);
int buffer_resize(Buffer* buffer, size_t new_capacity);
void buffer_free(Buffer* buffer);

// Compression helpers
typedef enum {
    COMPRESSION_NONE,
    COMPRESSION_ZLIB,
    COMPRESSION_LZ4,
    COMPRESSION_ZSTD
} CompressionType;

typedef struct {
    uint8_t* data;
    size_t size;
    int success;
    char error_message[256];
} CompressionResult;

CompressionResult compress_data(const uint8_t* input, size_t input_size, 
                               CompressionType type, int level);
CompressionResult decompress_data(const uint8_t* input, size_t input_size,
                                 CompressionType type);
void free_compression_result(CompressionResult* result);

// File I/O helpers
int read_file_to_buffer(const char* filename, Buffer* buffer);
int write_buffer_to_file(const char* filename, const Buffer* buffer);

// Logging
typedef enum {
    LOG_ERROR,
    LOG_WARN,
    LOG_INFO,
    LOG_DEBUG
} LogLevel;

void log_message(LogLevel level, const char* format, ...);
void set_log_level(LogLevel level);

// SVG optimization functions
WASM_EXPORT int svg_compress_text(const uint8_t* input, size_t input_size,
                                 uint8_t* output, size_t* output_size);

WASM_EXPORT int svg_minify_markup_simd(const uint8_t* input, size_t input_size,
                                      uint8_t* output, size_t* output_size);

WASM_EXPORT int svg_optimize_paths(const uint8_t* input, size_t input_size,
                                  uint8_t* output, size_t* output_size);

// ICO optimization functions  
WASM_EXPORT int ico_optimize_embedded(const uint8_t* input, size_t input_size,
                                     uint8_t* output, size_t* output_size,
                                     uint8_t quality);

WASM_EXPORT int ico_strip_metadata_simd(const uint8_t* input, size_t input_size,
                                       uint8_t* output, size_t* output_size);

WASM_EXPORT int ico_compress_directory(const uint8_t* input, size_t input_size,
                                      uint8_t* output, size_t* output_size);

#ifdef __cplusplus
}
#endif

#endif // UTIL_H
