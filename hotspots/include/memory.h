#ifndef WASM_MEMORY_H
#define WASM_MEMORY_H

#ifdef __cplusplus
extern "C" {
#endif

#ifdef __wasm32__

#ifdef __wasm_simd128__
#include <wasm_simd128.h>
#endif

#ifndef _STDINT_H
#ifndef __CLANG_STDINT_H
typedef unsigned char uint8_t;
typedef signed char int8_t;
typedef unsigned short uint16_t;
typedef short int16_t;
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned long long uint64_t;
typedef long long int64_t;
typedef __INTPTR_TYPE__ intptr_t;
typedef __UINTPTR_TYPE__ uintptr_t;
#endif
#endif

typedef unsigned int size_t;
typedef int ptrdiff_t;

#ifndef SIZE_MAX
#define SIZE_MAX ((size_t)-1)
#endif

#ifndef __cplusplus
#ifndef __bool_true_false_are_defined
typedef _Bool bool;
#define true 1
#define false 0
#define __bool_true_false_are_defined 1
#endif
#endif

#ifndef NULL
#define NULL ((void*)0)
#endif

#define WASM_EXPORT __attribute__((visibility("default")))

WASM_EXPORT void* wasm_malloc(size_t size);
WASM_EXPORT void wasm_free(void* ptr);
WASM_EXPORT void wasm_reset_allocator(void);

WASM_EXPORT void* wasm_memcpy(void* dest, const void* src, size_t n);
WASM_EXPORT void* wasm_memset(void* dest, int value, size_t n);
WASM_EXPORT int wasm_memcmp(const void* s1, const void* s2, size_t n);

WASM_EXPORT double wasm_sqrt(double x);
WASM_EXPORT double wasm_floor(double x);
WASM_EXPORT double wasm_ceil(double x);
WASM_EXPORT double wasm_pow(double base, double exp);

WASM_EXPORT size_t wasm_strlen(const char* s);
WASM_EXPORT char* wasm_strcpy(char* dest, const char* src);
WASM_EXPORT char* wasm_strncpy(char* dest, const char* src, size_t n);
WASM_EXPORT int wasm_strcmp(const char* s1, const char* s2);
WASM_EXPORT int wasm_strncmp(const char* s1, const char* s2, size_t n);

WASM_EXPORT void wasm_abort(void);

WASM_EXPORT int wasm_printf(const char* format, ...);

WASM_EXPORT uint32_t wasm_swap_bytes_32(uint32_t val);
WASM_EXPORT uint16_t wasm_swap_bytes_16(uint16_t val);
WASM_EXPORT void wasm_qsort(void* base, size_t nmemb, size_t size, int (*compar)(const void*, const void*));

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

#define NULL ((void*)0)
#define EOF (-1)

#else

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdio.h>

#endif

#ifdef __cplusplus
}
#endif

#endif
