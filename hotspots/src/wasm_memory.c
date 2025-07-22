/*
 * WASM Memory Management - provides malloc/free for WASM C code
 */

#include "wasm_memory.h"
#include <stddef.h>
#include <stdint.h>

#ifdef __wasm32__

// Simple bump allocator for WASM
static uint8_t memory_pool[1024 * 1024]; // 1MB pool
static size_t memory_offset = 0;

void* wasm_malloc(size_t size) {
    // Align to 8 bytes
    size = (size + 7) & ~7;
    
    if (memory_offset + size > sizeof(memory_pool)) {
        return 0; // Out of memory
    }
    
    void* ptr = &memory_pool[memory_offset];
    memory_offset += size;
    return ptr;
}

void wasm_free(void* ptr) {
    // Simple bump allocator doesn't support free
    // In a real implementation, you'd want a proper allocator
    (void)ptr;
}

void* wasm_memcpy(void* dest, const void* src, size_t n) {
    uint8_t* d = (uint8_t*)dest;
    const uint8_t* s = (const uint8_t*)src;
    
    while (n--) {
        *d++ = *s++;
    }
    
    return dest;
}

void* wasm_memset(void* dest, int value, size_t n) {
    uint8_t* d = (uint8_t*)dest;
    uint8_t val = (uint8_t)value;
    
    while (n--) {
        *d++ = val;
    }
    
    return dest;
}

int wasm_memcmp(const void* s1, const void* s2, size_t n) {
    const uint8_t* p1 = (const uint8_t*)s1;
    const uint8_t* p2 = (const uint8_t*)s2;
    
    while (n--) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    
    return 0;
}

#endif // __wasm32__
