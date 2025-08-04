#ifndef WASM_MEMORY_H
#define WASM_MEMORY_H

#ifdef __cplusplus
extern "C" {
#endif

// Self-contained type definitions to avoid standard library includes
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef signed char int8_t;
typedef unsigned short uint16_t;
typedef short int16_t;
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned long long uint64_t;
typedef long long int64_t;

#ifdef __wasm32__

// Export macro for WASM functions
#define WASM_EXPORT __attribute__((visibility("default")))

// Memory management
WASM_EXPORT void* wasm_malloc(size_t size);
WASM_EXPORT void wasm_free(void* ptr);
WASM_EXPORT void wasm_reset_allocator(void);

// Memory operations - replace stdlib functions
WASM_EXPORT void* wasm_memcpy(void* dest, const void* src, size_t n);
WASM_EXPORT void* wasm_memset(void* dest, int value, size_t n);
WASM_EXPORT int wasm_memcmp(const void* s1, const void* s2, size_t n);

// Math functions - replace math.h functions
WASM_EXPORT double wasm_sqrt(double x);
WASM_EXPORT double wasm_floor(double x);
WASM_EXPORT double wasm_ceil(double x);
WASM_EXPORT double wasm_pow(double base, double exp);

// String functions - replace string.h functions
WASM_EXPORT size_t wasm_strlen(const char* s);
WASM_EXPORT char* wasm_strcpy(char* dest, const char* src);
WASM_EXPORT char* wasm_strncpy(char* dest, const char* src, size_t n);
WASM_EXPORT int wasm_strcmp(const char* s1, const char* s2);
WASM_EXPORT int wasm_strncmp(const char* s1, const char* s2, size_t n);

// Error handling - replace stdlib.h functions
WASM_EXPORT void wasm_abort(void);

// stdio-like functions
WASM_EXPORT int wasm_printf(const char* format, ...);

// Utility functions for image processing
WASM_EXPORT uint32_t wasm_swap_bytes_32(uint32_t val);
WASM_EXPORT uint16_t wasm_swap_bytes_16(uint16_t val);
WASM_EXPORT void wasm_qsort(void* base, size_t nmemb, size_t size, int (*compar)(const void*, const void*));

// Macro definitions to replace standard library functions
#define malloc(size) wasm_malloc(size)
#define free(ptr) wasm_free(ptr)
#define memcpy(dest, src, n) wasm_memcpy(dest, src, n)
#define memset(dest, value, n) wasm_memset(dest, value, n)
#define memcmp(s1, s2, n) wasm_memcmp(s1, s2, n)
#define sqrt(x) wasm_sqrt(x)
#define floor(x) wasm_floor(x)
#define ceil(x) wasm_ceil(x)
#define pow(base, exp) wasm_pow(base, exp)
#define strlen(s) wasm_strlen(s)
#define strcpy(dest, src) wasm_strcpy(dest, src)
#define strncpy(dest, src, n) wasm_strncpy(dest, src, n)
#define strcmp(s1, s2) wasm_strcmp(s1, s2)
#define strncmp(s1, s2, n) wasm_strncmp(s1, s2, n)
#define abort() wasm_abort()
#define printf(format, ...) wasm_printf(format, ##__VA_ARGS__)
#define qsort(base, nmemb, size, compar) wasm_qsort(base, nmemb, size, compar)

// Constants that would normally come from standard headers
#define NULL ((void*)0)
#define EOF (-1)

#else

// For non-WASM builds, use standard library
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdio.h>

#endif // __wasm32__

#ifdef __cplusplus
}
#endif

#endif // WASM_MEMORY_H
