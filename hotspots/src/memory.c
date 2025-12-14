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

// CRITICAL: Memory pool for WASM - 100MB maximum
// This ensures we stay under the 100MB memory peak target
static uint8_t memory_pool[100 * 1024 * 1024]; // 100MB pool
static size_t memory_offset = 0;

#define WASM_EXPORT __attribute__((visibility("default")))

WASM_EXPORT void* wasm_malloc(size_t size) {
    size = (size + 7) & ~7;
    
    if (memory_offset + size > sizeof(memory_pool)) {
        return 0;
    }
    
    void* ptr = &memory_pool[memory_offset];
    memory_offset += size;
    return ptr;
}

WASM_EXPORT void wasm_free(void* ptr) {
    (void)ptr;
}

WASM_EXPORT void wasm_reset_allocator(void) {
    memory_offset = 0;
}

WASM_EXPORT size_t wasm_get_memory_usage(void) {
    return memory_offset;
}

WASM_EXPORT size_t wasm_get_memory_limit(void) {
    return sizeof(memory_pool);
}

WASM_EXPORT void* wasm_memcpy(void* dest, const void* src, size_t n) {
    uint8_t* d = (uint8_t*)dest;
    const uint8_t* s = (const uint8_t*)src;
    
    for (size_t i = 0; i < n; i++) {
        d[i] = s[i];
    }
    
    return dest;
}

WASM_EXPORT void* wasm_memset(void* dest, int value, size_t n) {
    uint8_t* d = (uint8_t*)dest;
    uint8_t val = (uint8_t)value;
    
    for (size_t i = 0; i < n; i++) {
        d[i] = val;
    }
    
    return dest;
}

WASM_EXPORT int wasm_memcmp(const void* s1, const void* s2, size_t n) {
    const uint8_t* p1 = (const uint8_t*)s1;
    const uint8_t* p2 = (const uint8_t*)s2;
    
    for (size_t i = 0; i < n; i++) {
        if (p1[i] != p2[i]) {
            return (int)p1[i] - (int)p2[i];
        }
    }
    
    return 0;
}

WASM_EXPORT double wasm_sqrt(double x) {
    if (x < 0.0) return 0.0/0.0; // NaN
    if (x == 0.0) return 0.0;
    
    double guess = x;
    for (int i = 0; i < 10; i++) {
        guess = (guess + x / guess) / 2.0;
    }
    return guess;
}

WASM_EXPORT double wasm_floor(double x) {
    return (double)(long long)x - (x < 0 && x != (double)(long long)x ? 1 : 0);
}

WASM_EXPORT double wasm_ceil(double x) {
    return (double)(long long)x + (x > 0 && x != (double)(long long)x ? 1 : 0);
}

WASM_EXPORT double wasm_pow(double base, double exp) {
    if (exp == 0.0) return 1.0;
    if (exp == 1.0) return base;
    if (exp < 0.0) return 1.0 / wasm_pow(base, -exp);
    
    double result = 1.0;
    long long int_exp = (long long)exp;
    
    for (long long i = 0; i < int_exp; i++) {
        result *= base;
    }
    
    return result;
}

WASM_EXPORT size_t wasm_strlen(const char* s) {
    size_t len = 0;
    while (s[len]) len++;
    return len;
}

WASM_EXPORT char* wasm_strcpy(char* dest, const char* src) {
    char* original_dest = dest;
    while ((*dest++ = *src++));
    return original_dest;
}

WASM_EXPORT char* wasm_strncpy(char* dest, const char* src, size_t n) {
    char* original_dest = dest;
    while (n && (*dest++ = *src++)) n--;
    while (n--) *dest++ = '\0';
    return original_dest;
}

WASM_EXPORT int wasm_strcmp(const char* s1, const char* s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(uint8_t*)s1 - *(uint8_t*)s2;
}

WASM_EXPORT int wasm_strncmp(const char* s1, const char* s2, size_t n) {
    while (n && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0) return 0;
    return *(uint8_t*)s1 - *(uint8_t*)s2;
}

WASM_EXPORT void wasm_abort(void) {
    while (1) {}
}

WASM_EXPORT int wasm_printf(const char* format, ...) {
    (void)format;
    return 0;
}

WASM_EXPORT uint32_t wasm_swap_bytes_32(uint32_t val) {
    return ((val & 0xFF000000) >> 24) |
           ((val & 0x00FF0000) >> 8)  |
           ((val & 0x0000FF00) << 8)  |
           ((val & 0x000000FF) << 24);
}

WASM_EXPORT uint16_t wasm_swap_bytes_16(uint16_t val) {
    return ((val & 0xFF00) >> 8) | ((val & 0x00FF) << 8);
}

WASM_EXPORT void wasm_qsort(void* base, size_t nmemb, size_t size, int (*compar)(const void*, const void*)) {
    uint8_t* arr = (uint8_t*)base;
    uint8_t* temp = wasm_malloc(size);
    
    for (size_t i = 0; i < nmemb - 1; i++) {
        for (size_t j = 0; j < nmemb - i - 1; j++) {
            void* elem1 = arr + j * size;
            void* elem2 = arr + (j + 1) * size;
            
            if (compar(elem1, elem2) > 0) {
                wasm_memcpy(temp, elem1, size);
                wasm_memcpy(elem1, elem2, size);
                wasm_memcpy(elem2, temp, size);
            }
        }
    }
}

#endif // __wasm32__
