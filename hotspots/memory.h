#ifndef MEMORY_H
#define MEMORY_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// High-performance memory management and buffer operations

// SIMD-accelerated memory operations
void memcpy_simd(void* dest, const void* src, size_t size);
void memset_simd(void* dest, int value, size_t size);
void memmove_simd(void* dest, const void* src, size_t size);

// Memory comparison with early exit optimization
int memcmp_fast(const void* ptr1, const void* ptr2, size_t size);

// Custom allocator for large media files
typedef struct {
    void* base_address;
    size_t total_size;
    size_t used_size;
    size_t alignment;
    uint8_t* free_map;
} MediaAllocator;

MediaAllocator* create_media_allocator(size_t total_size, size_t alignment);
void* media_alloc(MediaAllocator* allocator, size_t size);
void* media_alloc_aligned(MediaAllocator* allocator, size_t size, size_t alignment);
void media_free(MediaAllocator* allocator, void* ptr);
void reset_media_allocator(MediaAllocator* allocator);
void destroy_media_allocator(MediaAllocator* allocator);

// Memory pool for frequent allocations
typedef struct MemoryBlock {
    void* data;
    size_t size;
    struct MemoryBlock* next;
} MemoryBlock;

typedef struct {
    MemoryBlock* free_blocks;
    MemoryBlock* used_blocks;
    size_t block_size;
    size_t block_count;
    size_t blocks_allocated;
    size_t blocks_free;
} MemoryPool;

MemoryPool* create_memory_pool(size_t block_size, size_t initial_block_count);
void* pool_alloc(MemoryPool* pool);
void pool_free(MemoryPool* pool, void* ptr);
void expand_memory_pool(MemoryPool* pool, size_t additional_blocks);
void destroy_memory_pool(MemoryPool* pool);

// Zero-copy buffer management
typedef struct {
    uint8_t* data;
    size_t size;
    size_t capacity;
    size_t ref_count;
    void (*deallocator)(void* data);
} ZeroCopyBuffer;

ZeroCopyBuffer* create_zero_copy_buffer(size_t capacity);
ZeroCopyBuffer* wrap_zero_copy_buffer(void* data, size_t size, void (*deallocator)(void*));
ZeroCopyBuffer* slice_zero_copy_buffer(ZeroCopyBuffer* buffer, size_t offset, size_t size);
void retain_zero_copy_buffer(ZeroCopyBuffer* buffer);
void release_zero_copy_buffer(ZeroCopyBuffer* buffer);

// Advanced memory utilities
void memory_prefetch(const void* addr, size_t size);
void memory_flush_cache(const void* addr, size_t size);
size_t get_cache_line_size(void);
void* align_pointer(void* ptr, size_t alignment);

// Memory pattern operations
void fill_pattern_u32(uint32_t* dest, uint32_t pattern, size_t count);
void fill_pattern_u64(uint64_t* dest, uint64_t pattern, size_t count);
size_t find_pattern(const uint8_t* haystack, size_t haystack_size, 
                   const uint8_t* needle, size_t needle_size);

// Buffer statistics and analysis
typedef struct {
    size_t total_allocations;
    size_t total_deallocations;
    size_t current_usage;
    size_t peak_usage;
    size_t failed_allocations;
    double average_allocation_size;
    double fragmentation_ratio;
} MemoryStats;

MemoryStats get_memory_stats(MediaAllocator* allocator);
MemoryStats get_pool_stats(MemoryPool* pool);
void reset_memory_stats(MediaAllocator* allocator);

// Memory debugging and validation
int validate_buffer_bounds(const void* buffer, size_t buffer_size, size_t access_size);
int detect_buffer_overflow(const void* buffer, size_t expected_size);
void mark_memory_region(void* addr, size_t size, uint8_t marker);
int verify_memory_region(const void* addr, size_t size, uint8_t expected_marker);

#ifdef __cplusplus
}
#endif

#endif // MEMORY_H
