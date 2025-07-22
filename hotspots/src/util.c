#include "util.h"

// WASM-compatible standard library replacements
#ifdef __wasm32__
// Minimal WASM implementations - these are simplified stubs
extern void* malloc(size_t size);
extern void* realloc(void* ptr, size_t size);
extern void free(void* ptr);
extern void* memcpy(void* dest, const void* src, size_t n);
extern void* memset(void* s, int c, size_t n);
extern int memcmp(const void* s1, const void* s2, size_t n);

// WASM-specific simplified implementations
static inline char* strncpy(char* dest, const char* src, size_t n) {
    size_t i;
    for (i = 0; i < n && src[i] != '\0'; i++) {
        dest[i] = src[i];
    }
    for (; i < n; i++) {
        dest[i] = '\0';
    }
    return dest;
}

// Stub implementations for file operations (not available in WASM)
typedef void FILE;
#define SEEK_END 2
#define SEEK_SET 0
static FILE* stderr_stub;
#define stderr (&stderr_stub)
static inline FILE* fopen(const char* filename, const char* mode) { return NULL; }
static inline int fclose(FILE* stream) { return 0; }
static inline int fseek(FILE* stream, long offset, int whence) { return -1; }
static inline long ftell(FILE* stream) { return -1; }
static inline size_t fread(void* ptr, size_t size, size_t nmemb, FILE* stream) { return 0; }
static inline int fprintf(FILE* stream, const char* format, ...) { return 0; }
static inline int vfprintf(FILE* stream, const char* format, void* ap) { return 0; }
static inline int fflush(FILE* stream) { return 0; }
static inline size_t fwrite(const void* ptr, size_t size, size_t nmemb, FILE* stream) { return 0; }

// Varargs for WASM - simplified approach
typedef void* va_list;
#define va_start(ap, last) ((ap) = 0)
#define va_end(ap) ((ap) = 0)

#else
// Standard headers for native builds only
#ifndef __wasm__
    #include <stdlib.h>
    #include <string.h>
    #include <stdio.h>
    #include <stdarg.h>
#else
    // WASM freestanding implementations
    typedef void* va_list;
    #define va_start(ap, last) ((ap) = 0)
    #define va_end(ap) ((ap) = 0)
    
    // Memory operations for WASM
    static void* memcpy_wasm(void* dest, const void* src, size_t n) {
        char* d = (char*)dest;
        const char* s = (const char*)src;
        for (size_t i = 0; i < n; i++) {
            d[i] = s[i];
        }
        return dest;
    }
    
    static void* memset_wasm(void* s, int c, size_t n) {
        char* p = (char*)s;
        for (size_t i = 0; i < n; i++) {
            p[i] = (char)c;
        }
        return s;
    }
    
    #define memcpy memcpy_wasm
    #define memset memset_wasm
    
    // WASM memory management - use static buffers
    #define MAX_WASM_BUFFERS 32
    #define WASM_BUFFER_SIZE 65536
    
    static char wasm_buffer_pool[MAX_WASM_BUFFERS][WASM_BUFFER_SIZE];
    static int wasm_buffer_used[MAX_WASM_BUFFERS] = {0};
    
    static void* wasm_malloc(size_t size) {
        if (size > WASM_BUFFER_SIZE) return 0;
        for (int i = 0; i < MAX_WASM_BUFFERS; i++) {
            if (!wasm_buffer_used[i]) {
                wasm_buffer_used[i] = 1;
                return wasm_buffer_pool[i];
            }
        }
        return 0; // No free buffers
    }
    
    static void wasm_free(void* ptr) {
        if (!ptr) return;
        for (int i = 0; i < MAX_WASM_BUFFERS; i++) {
            if (ptr == wasm_buffer_pool[i]) {
                wasm_buffer_used[i] = 0;
                break;
            }
        }
    }
    
    #define malloc wasm_malloc
    #define free wasm_free
#endif
#endif

static LogLevel current_log_level = LOG_INFO;

// Buffer implementation
Buffer* buffer_create(size_t initial_capacity) {
    Buffer* buffer = malloc(sizeof(Buffer));
    if (!buffer) return NULL;
    
    buffer->data = malloc(initial_capacity);
    if (!buffer->data) {
        free(buffer);
        return NULL;
    }
    
    buffer->size = 0;
    buffer->capacity = initial_capacity;
    return buffer;
}

int buffer_append(Buffer* buffer, const uint8_t* data, size_t size) {
    if (!buffer || !data) return 0;
    
    if (buffer->size + size > buffer->capacity) {
        size_t new_capacity = buffer->capacity * 2;
        while (new_capacity < buffer->size + size) {
            new_capacity *= 2;
        }
        
        if (!buffer_resize(buffer, new_capacity)) {
            return 0;
        }
    }
    
    memcpy(buffer->data + buffer->size, data, size);
    buffer->size += size;
    return 1;
}

int buffer_resize(Buffer* buffer, size_t new_capacity) {
    if (!buffer) return 0;
    
    uint8_t* new_data = realloc(buffer->data, new_capacity);
    if (!new_data) return 0;
    
    buffer->data = new_data;
    buffer->capacity = new_capacity;
    
    if (buffer->size > new_capacity) {
        buffer->size = new_capacity;
    }
    
    return 1;
}

void buffer_free(Buffer* buffer) {
    if (buffer) {
        free(buffer->data);
        free(buffer);
    }
}

// Compression implementation (stubs for now)
static CompressionResult create_compression_error(const char* error_msg) {
    CompressionResult result = {0};
    result.success = 0;
    strncpy(result.error_message, error_msg, sizeof(result.error_message) - 1);
    result.error_message[sizeof(result.error_message) - 1] = '\0';
    return result;
}

CompressionResult compress_data(const uint8_t* input, size_t input_size, 
                               CompressionType type, int level) {
    if (!input || input_size == 0) {
        return create_compression_error("Invalid input data");
    }
    
    // TODO: Implement actual compression based on type
    // For now, just copy data (no compression)
    uint8_t* output = malloc(input_size);
    if (!output) {
        return create_compression_error("Memory allocation failed");
    }
    
    memcpy(output, input, input_size);
    
    CompressionResult result = {0};
    result.data = output;
    result.size = input_size;
    result.success = 1;
    
    log_message(LOG_DEBUG, "Compression placeholder: type=%d, level=%d, size=%zu", 
                type, level, input_size);
    
    return result;
}

CompressionResult decompress_data(const uint8_t* input, size_t input_size,
                                 CompressionType type) {
    if (!input || input_size == 0) {
        return create_compression_error("Invalid input data");
    }
    
    // TODO: Implement actual decompression based on type
    // For now, just copy data (no decompression)
    uint8_t* output = malloc(input_size);
    if (!output) {
        return create_compression_error("Memory allocation failed");
    }
    
    memcpy(output, input, input_size);
    
    CompressionResult result = {0};
    result.data = output;
    result.size = input_size;
    result.success = 1;
    
    log_message(LOG_DEBUG, "Decompression placeholder: type=%d, size=%zu", type, input_size);
    
    return result;
}

void free_compression_result(CompressionResult* result) {
    if (result && result->data) {
        free(result->data);
        result->data = NULL;
        result->size = 0;
    }
}

// File I/O implementation
int read_file_to_buffer(const char* filename, Buffer* buffer) {
    if (!filename || !buffer) return 0;
    
    FILE* file = fopen(filename, "rb");
    if (!file) {
        log_message(LOG_ERROR, "Failed to open file for reading: %s", filename);
        return 0;
    }
    
    // Get file size
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    if (file_size < 0) {
        fclose(file);
        log_message(LOG_ERROR, "Failed to get file size: %s", filename);
        return 0;
    }
    
    // Ensure buffer has enough capacity
    if (!buffer_resize(buffer, file_size)) {
        fclose(file);
        log_message(LOG_ERROR, "Failed to resize buffer for file: %s", filename);
        return 0;
    }
    
    // Read file data
    size_t bytes_read = fread(buffer->data, 1, file_size, file);
    fclose(file);
    
    if (bytes_read != (size_t)file_size) {
        log_message(LOG_ERROR, "Failed to read complete file: %s", filename);
        return 0;
    }
    
    buffer->size = bytes_read;
    log_message(LOG_DEBUG, "Successfully read %zu bytes from %s", bytes_read, filename);
    return 1;
}

int write_buffer_to_file(const char* filename, const Buffer* buffer) {
    if (!filename || !buffer || !buffer->data) return 0;
    
    FILE* file = fopen(filename, "wb");
    if (!file) {
        log_message(LOG_ERROR, "Failed to open file for writing: %s", filename);
        return 0;
    }
    
    size_t bytes_written = fwrite(buffer->data, 1, buffer->size, file);
    fclose(file);
    
    if (bytes_written != buffer->size) {
        log_message(LOG_ERROR, "Failed to write complete buffer to file: %s", filename);
        return 0;
    }
    
    log_message(LOG_DEBUG, "Successfully wrote %zu bytes to %s", bytes_written, filename);
    return 1;
}

// Logging implementation
void log_message(LogLevel level, const char* format, ...) {
    if (level > current_log_level) return;
    
    const char* level_str;
    switch (level) {
        case LOG_ERROR: level_str = "ERROR"; break;
        case LOG_WARN:  level_str = "WARN";  break;
        case LOG_INFO:  level_str = "INFO";  break;
        case LOG_DEBUG: level_str = "DEBUG"; break;
        default:        level_str = "UNKNOWN"; break;
    }
    
    fprintf(stderr, "[%s] ", level_str);
    
    va_list args;
    va_start(args, format);
    vfprintf(stderr, format, args);
    va_end(args);
    
    fprintf(stderr, "\n");
    fflush(stderr);
}

void set_log_level(LogLevel level) {
    current_log_level = level;
}
