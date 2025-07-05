#include "memory.h"
#include <stdlib.h>
#include <string.h>

#ifdef _MSC_VER
#include <intrin.h>
#else
#include <x86intrin.h>
#endif

// Feature detection
static int has_sse = -1;
static int has_avx = -1;

static void detect_simd_features() {
    if (has_sse == -1) {
        int cpuinfo[4];
#ifdef _MSC_VER
        __cpuid(cpuinfo, 1);
#else
        __builtin_cpu_init();
        cpuinfo[3] = __builtin_cpu_supports("sse") ? (1 << 25) : 0;
        cpuinfo[2] = __builtin_cpu_supports("avx") ? (1 << 28) : 0;
#endif
        has_sse = (cpuinfo[3] & (1 << 25)) != 0;
        has_avx = (cpuinfo[2] & (1 << 28)) != 0;
    }
}

void memcpy_simd(void* dest, const void* src, size_t size) {
    detect_simd_features();
    
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    // For small sizes, use regular memcpy
    if (size < 64) {
        memcpy(dest, src, size);
        return;
    }
    
    if (has_avx && size >= 32) {
        // AVX implementation for large copies
        size_t avx_size = size & ~31; // Round down to multiple of 32
        
        for (size_t i = 0; i < avx_size; i += 32) {
            __m256 data = _mm256_loadu_ps((float*)(s + i));
            _mm256_storeu_ps((float*)(d + i), data);
        }
        
        // Handle remaining bytes
        if (size > avx_size) {
            memcpy(d + avx_size, s + avx_size, size - avx_size);
        }
    } else if (has_sse && size >= 16) {
        // SSE implementation
        size_t sse_size = size & ~15; // Round down to multiple of 16
        
        for (size_t i = 0; i < sse_size; i += 16) {
            __m128 data = _mm_loadu_ps((float*)(s + i));
            _mm_storeu_ps((float*)(d + i), data);
        }
        
        // Handle remaining bytes
        if (size > sse_size) {
            memcpy(d + sse_size, s + sse_size, size - sse_size);
        }
    } else {
        // Fallback to regular memcpy
        memcpy(dest, src, size);
    }
}

void memset_simd(void* dest, int value, size_t size) {
    detect_simd_features();
    
    char* d = (char*)dest;
    
    // For small sizes, use regular memset
    if (size < 64) {
        memset(dest, value, size);
        return;
    }
    
    if (has_avx && size >= 32) {
        // Create AVX register with repeated value
        __m256i value_vec = _mm256_set1_epi8((char)value);
        size_t avx_size = size & ~31;
        
        for (size_t i = 0; i < avx_size; i += 32) {
            _mm256_storeu_si256((__m256i*)(d + i), value_vec);
        }
        
        // Handle remaining bytes
        if (size > avx_size) {
            memset(d + avx_size, value, size - avx_size);
        }
    } else if (has_sse && size >= 16) {
        // SSE implementation
        __m128i value_vec = _mm_set1_epi8((char)value);
        size_t sse_size = size & ~15;
        
        for (size_t i = 0; i < sse_size; i += 16) {
            _mm_storeu_si128((__m128i*)(d + i), value_vec);
        }
        
        // Handle remaining bytes
        if (size > sse_size) {
            memset(d + sse_size, value, size - sse_size);
        }
    } else {
        // Fallback to regular memset
        memset(dest, value, size);
    }
}

void memmove_simd(void* dest, const void* src, size_t size) {
    // For overlapping memory, we need to be careful about direction
    // Use regular memmove for safety - SIMD optimization here requires
    // complex overlap detection and direction handling
    memmove(dest, src, size);
}

int memcmp_fast(const void* ptr1, const void* ptr2, size_t size) {
    detect_simd_features();
    
    const char* p1 = (const char*)ptr1;
    const char* p2 = (const char*)ptr2;
    
    // For small sizes, use regular memcmp
    if (size < 32) {
        return memcmp(ptr1, ptr2, size);
    }
    
    if (has_sse && size >= 16) {
        size_t sse_size = size & ~15;
        
        for (size_t i = 0; i < sse_size; i += 16) {
            __m128i a = _mm_loadu_si128((const __m128i*)(p1 + i));
            __m128i b = _mm_loadu_si128((const __m128i*)(p2 + i));
            __m128i cmp = _mm_cmpeq_epi8(a, b);
            
            int mask = _mm_movemask_epi8(cmp);
            if (mask != 0xFFFF) {
                // Found difference, fall back to byte-by-byte comparison for this chunk
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
    } else {
        return memcmp(ptr1, ptr2, size);
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
        free(allocator->base_address);
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
    
    // TODO: Implement proper block allocation with free map
    // For now, simple linear allocation
    
    size_t aligned_size = (size + alignment - 1) & ~(alignment - 1);
    
    if (allocator->used_size + aligned_size > allocator->total_size) {
        return NULL; // Out of space
    }
    
    void* ptr = (uint8_t*)allocator->base_address + allocator->used_size;
    allocator->used_size += aligned_size;
    
    return ptr;
}

void media_free(MediaAllocator* allocator, void* ptr) {
    // TODO: Implement proper free block tracking
    // For now, no-op since we're using simple linear allocation
    (void)allocator; (void)ptr;
}

void reset_media_allocator(MediaAllocator* allocator) {
    if (allocator) {
        allocator->used_size = 0;
        memset(allocator->free_map, 0, allocator->total_size / allocator->alignment / 8 + 1);
    }
}

void destroy_media_allocator(MediaAllocator* allocator) {
    if (allocator) {
        free(allocator->base_address);
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
    // TODO: Use compiler intrinsics for memory prefetching
    (void)addr; (void)size;
}

void memory_flush_cache(const void* addr, size_t size) {
    // TODO: Use compiler intrinsics for cache flushing
    (void)addr; (void)size;
}

size_t get_cache_line_size(void) {
    // TODO: Detect actual cache line size at runtime
    return 64; // Common cache line size
}

void* align_pointer(void* ptr, size_t alignment) {
    uintptr_t addr = (uintptr_t)ptr;
    return (void*)((addr + alignment - 1) & ~(alignment - 1));
}

void fill_pattern_u32(uint32_t* dest, uint32_t pattern, size_t count) {
    // TODO: Use SIMD for faster pattern filling
    for (size_t i = 0; i < count; i++) {
        dest[i] = pattern;
    }
}

void fill_pattern_u64(uint64_t* dest, uint64_t pattern, size_t count) {
    // TODO: Use SIMD for faster pattern filling
    for (size_t i = 0; i < count; i++) {
        dest[i] = pattern;
    }
}

size_t find_pattern(const uint8_t* haystack, size_t haystack_size, 
                   const uint8_t* needle, size_t needle_size) {
    // TODO: Implement optimized pattern search (Boyer-Moore or similar)
    for (size_t i = 0; i <= haystack_size - needle_size; i++) {
        if (memcmp(haystack + i, needle, needle_size) == 0) {
            return i;
        }
    }
    return SIZE_MAX; // Not found
}

MemoryStats get_memory_stats(MediaAllocator* allocator) {
    MemoryStats stats = {0};
    
    if (allocator) {
        stats.current_usage = allocator->used_size;
        stats.peak_usage = allocator->used_size; // TODO: Track actual peak
        stats.fragmentation_ratio = 0.0; // TODO: Calculate fragmentation
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
    // TODO: Reset statistics tracking
    (void)allocator;
}

int validate_buffer_bounds(const void* buffer, size_t buffer_size, size_t access_size) {
    return (buffer != NULL && access_size <= buffer_size) ? 1 : 0;
}

int detect_buffer_overflow(const void* buffer, size_t expected_size) {
    // TODO: Implement buffer overflow detection using guard pages or canaries
    (void)buffer; (void)expected_size;
    return 0; // No overflow detected (placeholder)
}

void mark_memory_region(void* addr, size_t size, uint8_t marker) {
    // TODO: Implement memory region marking for debugging
    (void)addr; (void)size; (void)marker;
}

int verify_memory_region(const void* addr, size_t size, uint8_t expected_marker) {
    // TODO: Implement memory region verification
    (void)addr; (void)size; (void)expected_marker;
    return 1; // Verified (placeholder)
}
