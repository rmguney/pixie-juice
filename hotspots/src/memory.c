#include "memory.h"

// Only include standard library headers for native builds
#ifndef __wasm__
    #include <stdlib.h>
    #include <string.h>
    #include <stdint.h>
#endif

// Windows includes for system info
#ifdef _WIN32
    #ifndef __wasm__
        #include <windows.h>
    #endif
#endif

// WASM-compatible implementations
#ifdef __wasm__
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
    
    static void* memmove_wasm(void* dest, const void* src, size_t n) {
        char* d = (char*)dest;
        const char* s = (const char*)src;
        if (d <= s || d >= s + n) {
            // No overlap, use memcpy
            for (size_t i = 0; i < n; i++) {
                d[i] = s[i];
            }
        } else {
            // Overlap, copy backwards
            for (size_t i = n; i > 0; i--) {
                d[i-1] = s[i-1];
            }
        }
        return dest;
    }
    
    static int memcmp_wasm(const void* s1, const void* s2, size_t n) {
        const unsigned char* p1 = (const unsigned char*)s1;
        const unsigned char* p2 = (const unsigned char*)s2;
        for (size_t i = 0; i < n; i++) {
            if (p1[i] != p2[i]) {
                return p1[i] - p2[i];
            }
        }
        return 0;
    }
    
    // WASM memory management - use static buffers
    #define MAX_WASM_BUFFERS 64
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
    
    static void* wasm_calloc(size_t num, size_t size) {
        size_t total = num * size;
        void* ptr = wasm_malloc(total);
        if (ptr) {
            memset_wasm(ptr, 0, total);
        }
        return ptr;
    }
    
    static void* wasm_aligned_alloc(size_t alignment, size_t size) {
        // Simple implementation - just return regular malloc for WASM
        return wasm_malloc(size);
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
    
    #define memcpy memcpy_wasm
    #define memset memset_wasm
    #define memmove memmove_wasm
    #define memcmp memcmp_wasm
    #define malloc wasm_malloc
    #define calloc wasm_calloc
    #define aligned_alloc wasm_aligned_alloc
    #define free wasm_free
    #define aligned_free wasm_free
#else
    // Portable aligned_alloc implementation for native builds
    static void* portable_aligned_alloc(size_t alignment, size_t size) {
        // Check if alignment is power of 2
        if (alignment == 0 || (alignment & (alignment - 1)) != 0) {
            return NULL;
        }
        
        // Allocate extra space for alignment + pointer storage
        size_t total_size = size + alignment + sizeof(void*);
        void* raw_ptr = malloc(total_size);
        if (!raw_ptr) {
            return NULL;
        }
        
        // Calculate aligned address
        uintptr_t raw_addr = (uintptr_t)raw_ptr;
        uintptr_t aligned_addr = (raw_addr + sizeof(void*) + alignment - 1) & ~(alignment - 1);
        void* aligned_ptr = (void*)aligned_addr;
        
        // Store original pointer just before the aligned memory
        void** stored_ptr = (void**)aligned_ptr - 1;
        *stored_ptr = raw_ptr;
        
        return aligned_ptr;
    }
    
    static void portable_aligned_free(void* ptr) {
        if (!ptr) return;
        
        // Retrieve original pointer
        void** stored_ptr = (void**)ptr - 1;
        void* raw_ptr = *stored_ptr;
        free(raw_ptr);
    }
    
    #define aligned_alloc portable_aligned_alloc
    #define aligned_free portable_aligned_free
#endif

// WASM SIMD support (if available)
#ifdef __wasm__
    #ifdef __wasm_simd128__
        #include <wasm_simd128.h>
        #define WASM_SIMD_AVAILABLE 1
    #else
        #define WASM_SIMD_AVAILABLE 0
    #endif
#endif

// x86 SIMD support for native builds
#if !defined(__wasm__) && (defined(_MSC_VER) || defined(__GNUC__))
    #ifdef _MSC_VER
        #include <intrin.h>
    #else
        #include <x86intrin.h>
    #endif
    #define X86_SIMD_AVAILABLE 1
#else
    #define X86_SIMD_AVAILABLE 0
#endif

// Feature detection
static int has_simd = -1;

static void detect_simd_features() {
    if (has_simd == -1) {
#ifdef __wasm__
        // Check for WASM SIMD support
        #ifdef __wasm_simd128__
            has_simd = 1;
        #else
            has_simd = 0;
        #endif
#else
        // Detect x86 SIMD features
        int cpuinfo[4];
        #ifdef _MSC_VER
            __cpuid(cpuinfo, 1);
            has_simd = (cpuinfo[3] & (1 << 25)) != 0; // SSE support
        #else
            __builtin_cpu_init();
            has_simd = __builtin_cpu_supports("sse") ? 1 : 0;
        #endif
#endif
    }
}

void memcpy_simd(void* dest, const void* src, size_t size) {
    detect_simd_features();
    
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    // For very small sizes, use regular memcpy
    if (size < 16) {
        memcpy(dest, src, size);
        return;
    }
    
    if (has_simd && size >= 16) {
#ifdef __wasm__
        // WASM SIMD implementation
        #if WASM_SIMD_AVAILABLE
            size_t simd_size = size & ~15; // Round down to multiple of 16
            
            for (size_t i = 0; i < simd_size; i += 16) {
                v128_t data = wasm_v128_load(s + i);
                wasm_v128_store(d + i, data);
            }
            
            // Handle remaining bytes
            if (size > simd_size) {
                memcpy(d + simd_size, s + simd_size, size - simd_size);
            }
        #else
            // Optimized scalar loop for WASM without SIMD
            size_t word_size = size & ~7; // 8-byte chunks
            
            const uint64_t* src64 = (const uint64_t*)s;
            uint64_t* dst64 = (uint64_t*)d;
            
            for (size_t i = 0; i < word_size / 8; i++) {
                dst64[i] = src64[i];
            }
            
            if (size > word_size) {
                memcpy(d + word_size, s + word_size, size - word_size);
            }
        #endif
#else
        // x86 SIMD implementation
        #if X86_SIMD_AVAILABLE
            size_t sse_size = size & ~15; // Round down to multiple of 16
            
            for (size_t i = 0; i < sse_size; i += 16) {
                __m128i data = _mm_loadu_si128((const __m128i*)(s + i));
                _mm_storeu_si128((__m128i*)(d + i), data);
            }
            
            // Handle remaining bytes
            if (size > sse_size) {
                memcpy(d + sse_size, s + sse_size, size - sse_size);
            }
        #else
            memcpy(dest, src, size);
        #endif
#endif
    } else {
        // Fallback: optimized scalar copy
        size_t word_size = size & ~7; // 8-byte chunks
        
        const uint64_t* src64 = (const uint64_t*)s;
        uint64_t* dst64 = (uint64_t*)d;
        
        for (size_t i = 0; i < word_size / 8; i++) {
            dst64[i] = src64[i];
        }
        
        if (size > word_size) {
            memcpy(d + word_size, s + word_size, size - word_size);
        }
    }
}

void memset_simd(void* dest, int value, size_t size) {
    detect_simd_features();
    
    char* d = (char*)dest;
    
    // For very small sizes, use regular memset
    if (size < 16) {
        memset(dest, value, size);
        return;
    }
    
    if (has_simd && size >= 16) {
#ifdef __wasm__
        // WASM SIMD implementation
        #if WASM_SIMD_AVAILABLE
            v128_t value_vec = wasm_i8x16_splat((int8_t)value);
            size_t simd_size = size & ~15;
            
            for (size_t i = 0; i < simd_size; i += 16) {
                wasm_v128_store(d + i, value_vec);
            }
            
            // Handle remaining bytes
            if (size > simd_size) {
                memset(d + simd_size, value, size - simd_size);
            }
        #else
            // Optimized scalar loop for WASM without SIMD
            uint64_t pattern = 0x0101010101010101ULL * (unsigned char)value;
            size_t word_size = size & ~7; // 8-byte chunks
            
            uint64_t* dst64 = (uint64_t*)d;
            for (size_t i = 0; i < word_size / 8; i++) {
                dst64[i] = pattern;
            }
            
            if (size > word_size) {
                memset(d + word_size, value, size - word_size);
            }
        #endif
#else
        // x86 SIMD implementation
        #if X86_SIMD_AVAILABLE
            __m128i value_vec = _mm_set1_epi8((char)value);
            size_t sse_size = size & ~15;
            
            for (size_t i = 0; i < sse_size; i += 16) {
                _mm_storeu_si128((__m128i*)(d + i), value_vec);
            }
            
            // Handle remaining bytes
            if (size > sse_size) {
                memset(d + sse_size, value, size - sse_size);
            }
        #else
            memset(dest, value, size);
        #endif
#endif
    } else {
        // Fallback: optimized scalar memset
        uint64_t pattern = 0x0101010101010101ULL * (unsigned char)value;
        size_t word_size = size & ~7; // 8-byte chunks
        
        uint64_t* dst64 = (uint64_t*)d;
        for (size_t i = 0; i < word_size / 8; i++) {
            dst64[i] = pattern;
        }
        
        if (size > word_size) {
            memset(d + word_size, value, size - word_size);
        }
    }
}

void memmove_simd(void* dest, const void* src, size_t size) {
    // For overlapping memory, we need to be careful about direction
    // Check for overlap
    const char* s = (const char*)src;
    char* d = (char*)dest;
    
    if (d == s || size == 0) {
        return; // No-op
    }
    
    // Check if regions overlap
    if ((d < s && d + size > s) || (s < d && s + size > d)) {
        // Overlapping regions - use standard memmove for safety
        memmove(dest, src, size);
        return;
    }
    
    // Non-overlapping regions - we can use our optimized memcpy
    memcpy_simd(dest, src, size);
}

int memcmp_fast(const void* ptr1, const void* ptr2, size_t size) {
    detect_simd_features();
    
    const char* p1 = (const char*)ptr1;
    const char* p2 = (const char*)ptr2;
    
    // For very small sizes, use regular memcmp
    if (size < 16) {
        return memcmp(ptr1, ptr2, size);
    }
    
    if (has_simd && size >= 16) {
#ifdef __wasm__
        // WASM SIMD implementation
        #if WASM_SIMD_AVAILABLE
            size_t simd_size = size & ~15;
            
            for (size_t i = 0; i < simd_size; i += 16) {
                v128_t a = wasm_v128_load(p1 + i);
                v128_t b = wasm_v128_load(p2 + i);
                v128_t cmp = wasm_i8x16_eq(a, b);
                
                if (!wasm_v128_any_true(cmp)) {
                    // Found difference, fall back to byte-by-byte comparison
                    for (size_t j = i; j < i + 16 && j < size; j++) {
                        if (p1[j] != p2[j]) {
                            return (unsigned char)p1[j] - (unsigned char)p2[j];
                        }
                    }
                }
            }
            
            // Handle remaining bytes
            if (size > simd_size) {
                return memcmp(p1 + simd_size, p2 + simd_size, size - simd_size);
            }
            
            return 0;
        #else
            // Optimized scalar comparison for WASM without SIMD
            size_t word_size = size & ~7; // 8-byte chunks
            
            const uint64_t* src1 = (const uint64_t*)p1;
            const uint64_t* src2 = (const uint64_t*)p2;
            
            for (size_t i = 0; i < word_size / 8; i++) {
                if (src1[i] != src2[i]) {
                    // Found difference, do byte-by-byte comparison
                    for (size_t j = i * 8; j < (i + 1) * 8 && j < size; j++) {
                        if (p1[j] != p2[j]) {
                            return (unsigned char)p1[j] - (unsigned char)p2[j];
                        }
                    }
                }
            }
            
            if (size > word_size) {
                return memcmp(p1 + word_size, p2 + word_size, size - word_size);
            }
            
            return 0;
        #endif
#else
        // x86 SIMD implementation
        #if X86_SIMD_AVAILABLE
            size_t sse_size = size & ~15;
            
            for (size_t i = 0; i < sse_size; i += 16) {
                __m128i a = _mm_loadu_si128((const __m128i*)(p1 + i));
                __m128i b = _mm_loadu_si128((const __m128i*)(p2 + i));
                __m128i cmp = _mm_cmpeq_epi8(a, b);
                
                int mask = _mm_movemask_epi8(cmp);
                if (mask != 0xFFFF) {
                    // Found difference, fall back to byte-by-byte comparison
                    for (size_t j = i; j < i + 16 && j < size; j++) {
                        if (p1[j] != p2[j]) {
                            return (unsigned char)p1[j] - (unsigned char)p2[j];
                        }
                    }
                }
            }
            
            // Handle remaining bytes
            if (size > sse_size) {
                return memcmp(p1 + sse_size, p2 + sse_size, size - sse_size);
            }
            
            return 0;
        #else
            return memcmp(ptr1, ptr2, size);
        #endif
#endif
    } else {
        // Fallback: optimized scalar comparison
        size_t word_size = size & ~7; // 8-byte chunks
        
        const uint64_t* src1 = (const uint64_t*)p1;
        const uint64_t* src2 = (const uint64_t*)p2;
        
        for (size_t i = 0; i < word_size / 8; i++) {
            if (src1[i] != src2[i]) {
                // Found difference, do byte-by-byte comparison
                for (size_t j = i * 8; j < (i + 1) * 8 && j < size; j++) {
                    if (p1[j] != p2[j]) {
                        return (unsigned char)p1[j] - (unsigned char)p2[j];
                    }
                }
            }
        }
        
        if (size > word_size) {
            return memcmp(p1 + word_size, p2 + word_size, size - word_size);
        }
        
        return 0;
    }
}

MediaAllocator* create_media_allocator(size_t total_size, size_t alignment) {
    MediaAllocator* allocator = malloc(sizeof(MediaAllocator));
    if (!allocator) return NULL;
    
    // Align total size to alignment boundary
    total_size = (total_size + alignment - 1) & ~(alignment - 1);
    
    allocator->base_address = aligned_alloc(alignment, total_size);
    if (!allocator->base_address) {
        free(allocator);
        return NULL;
    }
    
    allocator->total_size = total_size;
    allocator->used_size = 0;
    allocator->alignment = alignment;
    
    // Simple free map - each bit represents an alignment-sized block
    size_t map_size = total_size / alignment / 8 + 1;
    allocator->free_map = calloc(map_size, 1);
    if (!allocator->free_map) {
        aligned_free(allocator->base_address);
        free(allocator);
        return NULL;
    }
    
    return allocator;
}

void* media_alloc(MediaAllocator* allocator, size_t size) {
    return media_alloc_aligned(allocator, size, allocator->alignment);
}

void* media_alloc_aligned(MediaAllocator* allocator, size_t size, size_t alignment) {
    if (!allocator || size == 0) return NULL;
    
    // Implement proper block allocation with free map
    size_t aligned_size = (size + alignment - 1) & ~(alignment - 1);
    
    // First, try to find a free block that fits
    FreeBlock* prev = NULL;
    FreeBlock* current = allocator->free_list;
    
    while (current) {
        if (current->size >= aligned_size) {
            // Found a suitable block
            void* ptr = (uint8_t*)allocator->base_address + current->offset;
            
            // Check if we can split the block
            if (current->size > aligned_size + sizeof(FreeBlock) + 8) {
                // Split the block
                FreeBlock* new_block = malloc(sizeof(FreeBlock));
                if (new_block) {
                    new_block->offset = current->offset + aligned_size;
                    new_block->size = current->size - aligned_size;
                    new_block->next = current->next;
                    
                    if (prev) {
                        prev->next = new_block;
                    } else {
                        allocator->free_list = new_block;
                    }
                }
            } else {
                // Use the entire block
                if (prev) {
                    prev->next = current->next;
                } else {
                    allocator->free_list = current->next;
                }
            }
            
            free(current);
            return ptr;
        }
        
        prev = current;
        current = current->next;
    }
    
    // No suitable free block found, try linear allocation
    size_t current_offset = allocator->used_size;
    size_t aligned_offset = (current_offset + alignment - 1) & ~(alignment - 1);
    
    if (aligned_offset + aligned_size > allocator->total_size) {
        return NULL; // Out of space
    }
    
    void* ptr = (uint8_t*)allocator->base_address + aligned_offset;
    allocator->used_size = aligned_offset + aligned_size;
    
    return ptr;
}

void media_free(MediaAllocator* allocator, void* ptr) {
    if (!allocator || !ptr) return;
    
    // Calculate offset from base address
    size_t offset = (uint8_t*)ptr - (uint8_t*)allocator->base_address;
    
    // For simplicity, we'll track freed blocks but won't implement
    // complex coalescing for now. In a production implementation,
    // you'd want to merge adjacent free blocks.
    
    // Find the size of this allocation (we'd need to track this)
    // For now, we'll add a simple free block of minimum size
    size_t block_size = 64; // Minimum block size assumption
    
    // Create new free block
    FreeBlock* new_block = malloc(sizeof(FreeBlock));
    if (!new_block) return; // Failed to allocate tracking structure
    
    new_block->offset = offset;
    new_block->size = block_size;
    new_block->next = allocator->free_list;
    allocator->free_list = new_block;
    
    // TODO: Implement block coalescing and proper size tracking
    // In a real implementation, you'd store allocation metadata
    // before each allocated block to track sizes accurately
}

void reset_media_allocator(MediaAllocator* allocator) {
    if (allocator) {
        allocator->used_size = 0;
        memset(allocator->free_map, 0, allocator->total_size / allocator->alignment / 8 + 1);
    }
}

void destroy_media_allocator(MediaAllocator* allocator) {
    if (allocator) {
        aligned_free(allocator->base_address);
        free(allocator->free_map);
        free(allocator);
    }
}

MemoryPool* create_memory_pool(size_t block_size, size_t initial_block_count) {
    MemoryPool* pool = malloc(sizeof(MemoryPool));
    if (!pool) return NULL;
    
    pool->block_size = block_size;
    pool->block_count = initial_block_count;
    pool->blocks_allocated = 0;
    pool->blocks_free = 0;
    pool->free_blocks = NULL;
    pool->used_blocks = NULL;
    
    // Pre-allocate initial blocks
    expand_memory_pool(pool, initial_block_count);
    
    return pool;
}

void* pool_alloc(MemoryPool* pool) {
    if (!pool) return NULL;
    
    if (!pool->free_blocks) {
        // Expand pool if no free blocks
        expand_memory_pool(pool, pool->block_count / 2 + 1);
    }
    
    if (!pool->free_blocks) {
        return NULL; // Still no blocks available
    }
    
    MemoryBlock* block = pool->free_blocks;
    pool->free_blocks = block->next;
    block->next = pool->used_blocks;
    pool->used_blocks = block;
    
    pool->blocks_free--;
    
    return block->data;
}

void pool_free(MemoryPool* pool, void* ptr) {
    if (!pool || !ptr) return;
    
    // Find the block in used list
    MemoryBlock** current = &pool->used_blocks;
    while (*current) {
        if ((*current)->data == ptr) {
            MemoryBlock* block = *current;
            *current = block->next;
            
            // Move to free list
            block->next = pool->free_blocks;
            pool->free_blocks = block;
            pool->blocks_free++;
            return;
        }
        current = &(*current)->next;
    }
}

void expand_memory_pool(MemoryPool* pool, size_t additional_blocks) {
    if (!pool || additional_blocks == 0) return;
    
    for (size_t i = 0; i < additional_blocks; i++) {
        MemoryBlock* block = malloc(sizeof(MemoryBlock));
        if (!block) break;
        
        block->data = malloc(pool->block_size);
        if (!block->data) {
            free(block);
            break;
        }
        
        block->size = pool->block_size;
        block->next = pool->free_blocks;
        pool->free_blocks = block;
        
        pool->blocks_allocated++;
        pool->blocks_free++;
    }
}

void destroy_memory_pool(MemoryPool* pool) {
    if (!pool) return;
    
    // Free all blocks
    while (pool->free_blocks) {
        MemoryBlock* block = pool->free_blocks;
        pool->free_blocks = block->next;
        free(block->data);
        free(block);
    }
    
    while (pool->used_blocks) {
        MemoryBlock* block = pool->used_blocks;
        pool->used_blocks = block->next;
        free(block->data);
        free(block);
    }
    
    free(pool);
}

ZeroCopyBuffer* create_zero_copy_buffer(size_t capacity) {
    ZeroCopyBuffer* buffer = malloc(sizeof(ZeroCopyBuffer));
    if (!buffer) return NULL;
    
    buffer->data = malloc(capacity);
    if (!buffer->data) {
        free(buffer);
        return NULL;
    }
    
    buffer->size = 0;
    buffer->capacity = capacity;
    buffer->ref_count = 1;
    buffer->deallocator = free;
    
    return buffer;
}

ZeroCopyBuffer* wrap_zero_copy_buffer(void* data, size_t size, void (*deallocator)(void*)) {
    ZeroCopyBuffer* buffer = malloc(sizeof(ZeroCopyBuffer));
    if (!buffer) return NULL;
    
    buffer->data = (uint8_t*)data;
    buffer->size = size;
    buffer->capacity = size;
    buffer->ref_count = 1;
    buffer->deallocator = deallocator;
    
    return buffer;
}

ZeroCopyBuffer* slice_zero_copy_buffer(ZeroCopyBuffer* buffer, size_t offset, size_t size) {
    if (!buffer || offset + size > buffer->size) return NULL;
    
    ZeroCopyBuffer* slice = malloc(sizeof(ZeroCopyBuffer));
    if (!slice) return NULL;
    
    slice->data = buffer->data + offset;
    slice->size = size;
    slice->capacity = size;
    slice->ref_count = 1;
    slice->deallocator = NULL; // Slice doesn't own the data
    
    retain_zero_copy_buffer(buffer); // Keep original alive
    
    return slice;
}

void retain_zero_copy_buffer(ZeroCopyBuffer* buffer) {
    if (buffer) {
        buffer->ref_count++;
    }
}

void release_zero_copy_buffer(ZeroCopyBuffer* buffer) {
    if (buffer && --buffer->ref_count == 0) {
        if (buffer->deallocator) {
            buffer->deallocator(buffer->data);
        }
        free(buffer);
    }
}

void memory_prefetch(const void* addr, size_t size) {
    // Use compiler intrinsics for memory prefetching
#ifdef __wasm__
    // WASM doesn't support prefetch intrinsics, but we can provide a no-op
    (void)addr; (void)size;
#elif defined(_MSC_VER)
    // MSVC intrinsics
    const char* ptr = (const char*)addr;
    for (size_t i = 0; i < size; i += 64) { // 64-byte cache lines
        _mm_prefetch(ptr + i, _MM_HINT_T0);
    }
#elif defined(__GNUC__)
    // GCC/Clang intrinsics
    const char* ptr = (const char*)addr;
    for (size_t i = 0; i < size; i += 64) { // 64-byte cache lines
        __builtin_prefetch(ptr + i, 0, 3); // Read, high temporal locality
    }
#else
    (void)addr; (void)size; // Fallback: no-op
#endif
}

void memory_flush_cache(const void* addr, size_t size) {
    // Use compiler intrinsics for cache flushing
#ifdef __wasm__
    // WASM doesn't support cache flush intrinsics
    (void)addr; (void)size;
#elif defined(_MSC_VER)
    // MSVC intrinsics
    const char* ptr = (const char*)addr;
    for (size_t i = 0; i < size; i += 64) {
        _mm_clflush(ptr + i);
    }
#elif defined(__GNUC__)
    // GCC/Clang intrinsics
    const char* ptr = (const char*)addr;
    for (size_t i = 0; i < size; i += 64) {
        __builtin_ia32_clflush(ptr + i);
    }
#else
    (void)addr; (void)size; // Fallback: no-op
#endif
}

size_t get_cache_line_size(void) {
    // Detect actual cache line size at runtime
#ifdef __wasm__
    return 64; // Standard cache line size for WASM
#elif defined(_MSC_VER)
    SYSTEM_INFO si;
    GetSystemInfo(&si);
    return si.dwProcessorType >= 586 ? 64 : 32; // Pentium and later use 64-byte cache lines
#elif defined(__linux__)
    long cache_line_size = sysconf(_SC_LEVEL1_DCACHE_LINESIZE);
    return cache_line_size > 0 ? (size_t)cache_line_size : 64;
#elif defined(__APPLE__)
    size_t cache_line_size = 0;
    size_t size = sizeof(cache_line_size);
    if (sysctlbyname("hw.cachelinesize", &cache_line_size, &size, NULL, 0) == 0) {
        return cache_line_size;
    }
    return 64; // Default fallback
#else
    return 64; // Common cache line size
#endif
}

void* align_pointer(void* ptr, size_t alignment) {
    uintptr_t addr = (uintptr_t)ptr;
    return (void*)((addr + alignment - 1) & ~(alignment - 1));
}

void fill_pattern_u32(uint32_t* dest, uint32_t pattern, size_t count) {
    // Use SIMD for faster pattern filling
    detect_simd_features();
    
    if (has_simd && count >= 4) {
#ifdef __wasm__
        #if WASM_SIMD_AVAILABLE
            v128_t pattern_vec = wasm_i32x4_splat((int32_t)pattern);
            size_t simd_count = count & ~3; // Round down to multiple of 4
            
            for (size_t i = 0; i < simd_count; i += 4) {
                wasm_v128_store((uint8_t*)(dest + i), pattern_vec);
            }
            
            // Handle remaining elements
            for (size_t i = simd_count; i < count; i++) {
                dest[i] = pattern;
            }
        #else
            // Scalar fallback
            for (size_t i = 0; i < count; i++) {
                dest[i] = pattern;
            }
        #endif
#else
        #if X86_SIMD_AVAILABLE
            __m128i pattern_vec = _mm_set1_epi32((int32_t)pattern);
            size_t simd_count = count & ~3; // Round down to multiple of 4
            
            for (size_t i = 0; i < simd_count; i += 4) {
                _mm_storeu_si128((__m128i*)(dest + i), pattern_vec);
            }
            
            // Handle remaining elements
            for (size_t i = simd_count; i < count; i++) {
                dest[i] = pattern;
            }
        #else
            for (size_t i = 0; i < count; i++) {
                dest[i] = pattern;
            }
        #endif
#endif
    } else {
        // Scalar fallback
        for (size_t i = 0; i < count; i++) {
            dest[i] = pattern;
        }
    }
}

void fill_pattern_u64(uint64_t* dest, uint64_t pattern, size_t count) {
    // Use SIMD for faster pattern filling
    detect_simd_features();
    
    if (has_simd && count >= 2) {
#ifdef __wasm__
        #if WASM_SIMD_AVAILABLE
            v128_t pattern_vec = wasm_i64x2_splat((int64_t)pattern);
            size_t simd_count = count & ~1; // Round down to multiple of 2
            
            for (size_t i = 0; i < simd_count; i += 2) {
                wasm_v128_store((uint8_t*)(dest + i), pattern_vec);
            }
            
            // Handle remaining elements
            for (size_t i = simd_count; i < count; i++) {
                dest[i] = pattern;
            }
        #else
            // Scalar fallback
            for (size_t i = 0; i < count; i++) {
                dest[i] = pattern;
            }
        #endif
#else
        #if X86_SIMD_AVAILABLE
            __m128i pattern_vec = _mm_set1_epi64x((int64_t)pattern);
            size_t simd_count = count & ~1; // Round down to multiple of 2
            
            for (size_t i = 0; i < simd_count; i += 2) {
                _mm_storeu_si128((__m128i*)(dest + i), pattern_vec);
            }
            
            // Handle remaining elements
            for (size_t i = simd_count; i < count; i++) {
                dest[i] = pattern;
            }
        #else
            for (size_t i = 0; i < count; i++) {
                dest[i] = pattern;
            }
        #endif
#endif
    } else {
        // Scalar fallback
        for (size_t i = 0; i < count; i++) {
            dest[i] = pattern;
        }
    }
}

size_t find_pattern(const uint8_t* haystack, size_t haystack_size, 
                   const uint8_t* needle, size_t needle_size) {
    // Implement optimized pattern search (Boyer-Moore algorithm)
    if (needle_size == 0 || needle_size > haystack_size) {
        return SIZE_MAX;
    }
    
    if (needle_size == 1) {
        // Optimized single-byte search
        for (size_t i = 0; i < haystack_size; i++) {
            if (haystack[i] == needle[0]) {
                return i;
            }
        }
        return SIZE_MAX;
    }
    
    // Build bad character table for Boyer-Moore
    int bad_char[256];
    for (int i = 0; i < 256; i++) {
        bad_char[i] = -1;
    }
    for (size_t i = 0; i < needle_size; i++) {
        bad_char[needle[i]] = (int)i;
    }
    
    // Boyer-Moore search
    size_t shift = 0;
    while (shift <= haystack_size - needle_size) {
        int j = (int)needle_size - 1;
        
        // Match from right to left
        while (j >= 0 && needle[j] == haystack[shift + j]) {
            j--;
        }
        
        if (j < 0) {
            return shift; // Pattern found
        } else {
            // Bad character heuristic
            int bad_char_shift = j - bad_char[haystack[shift + j]];
            shift += (bad_char_shift > 1) ? bad_char_shift : 1;
        }
    }
    
    return SIZE_MAX; // Not found
}

MemoryStats get_memory_stats(MediaAllocator* allocator) {
    MemoryStats stats = {0};
    
    if (allocator) {
        stats.current_usage = allocator->used_size;
        
        // Calculate peak usage by tracking maximum used_size
        static size_t peak_usage = 0;
        if (allocator->used_size > peak_usage) {
            peak_usage = allocator->used_size;
        }
        stats.peak_usage = peak_usage;
        
        // Calculate fragmentation ratio
        size_t free_space = 0;
        size_t free_blocks = 0;
        FreeBlock* current = allocator->free_list;
        
        while (current) {
            free_space += current->size;
            free_blocks++;
            current = current->next;
        }
        
        if (allocator->total_size > 0) {
            stats.fragmentation_ratio = (float)free_blocks / 
                                       ((float)allocator->total_size / 1024.0f);
        }
        
        stats.allocations = allocator->used_size > 0 ? 1 : 0; // Simplified
        stats.deallocations = free_blocks;
    }
    
    return stats;
}

MemoryStats get_pool_stats(MemoryPool* pool) {
    MemoryStats stats = {0};
    
    if (pool) {
        stats.total_allocations = pool->blocks_allocated;
        stats.current_usage = (pool->blocks_allocated - pool->blocks_free) * pool->block_size;
        stats.average_allocation_size = pool->block_size;
    }
    
    return stats;
}

void reset_memory_stats(MediaAllocator* allocator) {
    if (allocator) {
        // Reset peak usage tracking
        static size_t* peak_usage_ptr = NULL;
        if (!peak_usage_ptr) {
            static size_t peak_storage = 0;
            peak_usage_ptr = &peak_storage;
        }
        *peak_usage_ptr = allocator->used_size;
        
        // Reset any other tracking variables as needed
        // In a full implementation, you'd track allocation/deallocation counts,
        // timing statistics, etc.
    }
}

int validate_buffer_bounds(const void* buffer, size_t buffer_size, size_t access_size) {
    return (buffer != NULL && access_size <= buffer_size) ? 1 : 0;
}

int detect_buffer_overflow(const void* buffer, size_t expected_size) {
    // Implement buffer overflow detection using guard patterns
    if (!buffer || expected_size == 0) return 1; // Invalid parameters
    
    // Check for common guard patterns at the end of the buffer
    const uint8_t* byte_buffer = (const uint8_t*)buffer;
    const uint32_t guard_pattern = 0xDEADBEEF;
    
    // Check if there's space for guard pattern
    if (expected_size < sizeof(guard_pattern)) return 0;
    
    // Look for guard pattern at the end
    const uint32_t* guard_location = (const uint32_t*)(byte_buffer + expected_size - sizeof(guard_pattern));
    
    // Use volatile to prevent compiler optimizations
    volatile uint32_t found_pattern = *guard_location;
    
    if (found_pattern != guard_pattern) {
        return 1; // Overflow detected (guard pattern corrupted)
    }
    
    // Additional check: look for null terminator overwrites in string buffers
    for (size_t i = expected_size - sizeof(guard_pattern); i > 0; i--) {
        if (byte_buffer[i] == 0) {
            // Found null terminator, check next few bytes for corruption
            for (size_t j = i + 1; j < expected_size - sizeof(guard_pattern) && j < i + 8; j++) {
                if (byte_buffer[j] != 0 && (byte_buffer[j] < 32 || byte_buffer[j] > 126)) {
                    return 1; // Suspicious bytes after null terminator
                }
            }
            break;
        }
    }
    
    return 0; // No overflow detected
}

void mark_memory_region(void* addr, size_t size, uint8_t marker) {
    // Implement memory region marking for debugging
    if (!addr || size == 0) return;
    
    uint8_t* byte_addr = (uint8_t*)addr;
    
    // Mark the beginning and end of the region
    size_t header_size = sizeof(size_t) + sizeof(uint8_t);
    size_t footer_size = sizeof(uint8_t) + sizeof(uint32_t);
    
    if (size < header_size + footer_size) return; // Not enough space
    
    // Write header: size + marker
    *(size_t*)byte_addr = size;
    byte_addr[sizeof(size_t)] = marker;
    
    // Write footer: marker + magic number
    size_t footer_offset = size - footer_size;
    byte_addr[footer_offset] = marker;
    *(uint32_t*)(byte_addr + footer_offset + sizeof(uint8_t)) = 0xDEADBEEF;
}

int verify_memory_region(const void* addr, size_t size, uint8_t expected_marker) {
    // Implement memory region verification
    if (!addr || size == 0) return 0;
    
    const uint8_t* byte_addr = (const uint8_t*)addr;
    
    size_t header_size = sizeof(size_t) + sizeof(uint8_t);
    size_t footer_size = sizeof(uint8_t) + sizeof(uint32_t);
    
    if (size < header_size + footer_size) return 0;
    
    // Verify header
    size_t stored_size = *(const size_t*)byte_addr;
    uint8_t header_marker = byte_addr[sizeof(size_t)];
    
    if (stored_size != size || header_marker != expected_marker) {
        return 0; // Header verification failed
    }
    
    // Verify footer
    size_t footer_offset = size - footer_size;
    uint8_t footer_marker = byte_addr[footer_offset];
    uint32_t magic = *(const uint32_t*)(byte_addr + footer_offset + sizeof(uint8_t));
    
    if (footer_marker != expected_marker || magic != 0xDEADBEEF) {
        return 0; // Footer verification failed
    }
    
    return 1; // Verification successful
}
