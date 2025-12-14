#ifndef UTIL_H
#define UTIL_H

#include "memory.h"

#ifdef __wasm32__
#define WASM_EXPORT __attribute__((visibility("default")))
#define WASM_IMPORT __attribute__((import_module("env")))
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

typedef struct {
    uint8_t* data;
    size_t size;
    size_t capacity;
} Buffer;

WASM_EXPORT Buffer* buffer_create(size_t initial_capacity);
WASM_EXPORT int buffer_append(Buffer* buffer, const uint8_t* data, size_t size);
int buffer_resize(Buffer* buffer, size_t new_capacity);
WASM_EXPORT void buffer_destroy(Buffer* buffer);

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

int read_file_to_buffer(const char* filename, Buffer* buffer);
int write_buffer_to_file(const char* filename, const Buffer* buffer);

typedef enum {
    LOG_ERROR,
    LOG_WARN,
    LOG_INFO,
    LOG_DEBUG
} LogLevel;

void log_message(LogLevel level, const char* format, ...);
void set_log_level(LogLevel level);

void memcpy_simd(void* dest, const void* src, size_t size);
void memset_simd(void* dest, int value, size_t size);

WASM_EXPORT uint8_t* svg_compress_text(const uint8_t* input, size_t input_size,
                                 uint32_t compression_level, size_t* output_size);

WASM_EXPORT int svg_minify_markup_simd(const uint8_t* input, size_t input_size,
                                      uint8_t* output, size_t* output_size);

WASM_EXPORT uint8_t* svg_optimize_paths(const uint8_t* input, size_t input_size,
                                  float precision, size_t* output_size);

WASM_EXPORT uint8_t* ico_optimize_embedded(const uint8_t* input, size_t input_size,
                                     uint8_t quality, size_t* output_size);

WASM_EXPORT uint8_t* ico_strip_metadata_simd(const uint8_t* input, size_t input_size,
                                       size_t* output_size);

WASM_EXPORT uint8_t* ico_compress_directory(const uint8_t* input, size_t input_size,
                                      uint32_t compression_level, size_t* output_size);

#ifdef __cplusplus
}
#endif

#endif
