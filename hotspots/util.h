#ifndef UTIL_H
#define UTIL_H

#include <stdint.h>
#include <stddef.h>

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

#ifdef __cplusplus
}
#endif

#endif // UTIL_H
