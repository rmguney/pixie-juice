/// FFI bindings for memory management C hotspots
/// Provides high-performance memory operations and custom allocators

use crate::types::{OptResult, OptError};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub failed_allocations: usize,
    pub average_allocation_size: f64,
    pub fragmentation_ratio: f64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_allocations: 0,
            total_deallocations: 0,
            current_usage: 0,
            peak_usage: 0,
            failed_allocations: 0,
            average_allocation_size: 0.0,
            fragmentation_ratio: 0.0,
        }
    }
}

// C FFI declarations for when C hotspots are enabled
#[cfg(c_hotspots_available)]
extern "C" {
    // SIMD memory operations - using c_void to match generated bindings
    pub fn memcpy_simd(dest: *mut std::ffi::c_void, src: *const std::ffi::c_void, size: usize);
    pub fn memset_simd(dest: *mut std::ffi::c_void, value: i32, size: usize);
    pub fn memcmp_fast(ptr1: *const u8, ptr2: *const u8, size: usize) -> i32;
    
    // Memory utilities
    pub fn memory_prefetch(addr: *const std::ffi::c_void, size: usize);
    pub fn memory_flush_cache(addr: *const std::ffi::c_void, size: usize);
    pub fn get_cache_line_size() -> usize;
    
    // Pattern operations
    pub fn fill_pattern_u32(dest: *mut u32, pattern: u32, count: usize);
    pub fn fill_pattern_u64(dest: *mut u64, pattern: u64, count: usize);
    pub fn find_pattern(haystack: *const u8, haystack_size: usize, 
                       needle: *const u8, needle_size: usize) -> usize;
    
    // Media allocator functions
    pub fn create_media_allocator(total_size: usize, alignment: usize) -> *mut MediaAllocator;
    pub fn destroy_media_allocator(allocator: *mut MediaAllocator);
    
    // Memory pool functions 
    pub fn create_memory_pool(block_size: usize, initial_block_count: usize) -> *mut MemoryPool;
    pub fn destroy_memory_pool(pool: *mut MemoryPool);
    
    // Zero-copy buffer functions
    pub fn create_zero_copy_buffer(capacity: usize) -> *mut std::ffi::c_void;
    pub fn wrap_zero_copy_buffer(data: *mut std::ffi::c_void, size: usize, deallocator: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn slice_zero_copy_buffer(buffer: *mut std::ffi::c_void, offset: usize, size: usize) -> *mut std::ffi::c_void;
    pub fn retain_zero_copy_buffer(buffer: *mut std::ffi::c_void);
    pub fn release_zero_copy_buffer(buffer: *mut std::ffi::c_void);
    
    // Validation
    pub fn validate_buffer_bounds(buffer: *const std::ffi::c_void, buffer_size: usize, access_size: usize) -> i32;
    pub fn detect_buffer_overflow(buffer: *const std::ffi::c_void, expected_size: usize) -> i32;
}

// Opaque C types
#[cfg(c_hotspots_available)]
#[repr(C)]
pub struct MediaAllocator {
    _private: [u8; 0],
}

#[cfg(c_hotspots_available)]
#[repr(C)]
pub struct MemoryPool {
    _private: [u8; 0],
}

/// Safe wrapper for SIMD memcpy
pub fn memcpy_simd_safe(dest: &mut [u8], src: &[u8]) -> OptResult<()> {
    if dest.len() < src.len() {
        return Err(OptError::ProcessingError("Destination buffer too small".to_string()));
    }
    
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            memcpy_simd(
                dest.as_mut_ptr() as *mut std::ffi::c_void,
                src.as_ptr() as *const std::ffi::c_void,
                src.len()
            );
        }
        Ok(())
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback using optimized copy_from_slice
        dest[..src.len()].copy_from_slice(src);
        Ok(())
    }
}

/// Safe wrapper for SIMD memset
pub fn memset_simd_safe(dest: &mut [u8], value: u8) -> OptResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            memset_simd(dest.as_mut_ptr() as *mut std::ffi::c_void, value as i32, dest.len());
        }
        Ok(())
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback
        dest.fill(value);
        Ok(())
    }
}

/// Safe wrapper for fast memcmp
pub fn memcmp_fast_safe(a: &[u8], b: &[u8]) -> OptResult<std::cmp::Ordering> {
    if a.len() != b.len() {
        return Err(OptError::ProcessingError("Buffer lengths don't match".to_string()));
    }
    
    #[cfg(c_hotspots_available)]
    {
        let result = unsafe {
            memcmp_fast(a.as_ptr(), b.as_ptr(), a.len())
        };
        Ok(match result {
            0 => std::cmp::Ordering::Equal,
            x if x < 0 => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Greater,
        })
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback
        Ok(a.cmp(b))
    }
}

/// Memory prefetching hint
pub fn prefetch_memory_safe(data: &[u8]) -> OptResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            memory_prefetch(data.as_ptr() as *const std::ffi::c_void, data.len());
        }
        Ok(())
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback - no-op since Rust doesn't have standard prefetch
        let _ = data; // Silence unused warning
        Ok(())
    }
}

/// Cache flush operation
pub fn flush_cache_safe(data: &[u8]) -> OptResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            memory_flush_cache(data.as_ptr() as *const std::ffi::c_void, data.len());
        }
        Ok(())
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback - no-op
        let _ = data; // Silence unused warning
        Ok(())
    }
}

/// Get cache line size
pub fn get_cache_line_size_safe() -> usize {
    #[cfg(c_hotspots_available)]
    {
        unsafe { get_cache_line_size() }
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        64 // Common cache line size fallback
    }
}

/// Fill buffer with 32-bit pattern
pub fn fill_pattern_u32_safe(dest: &mut [u32], pattern: u32) -> OptResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            fill_pattern_u32(dest.as_mut_ptr(), pattern, dest.len());
        }
        Ok(())
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback
        dest.fill(pattern);
        Ok(())
    }
}

/// Fill buffer with 64-bit pattern
pub fn fill_pattern_u64_safe(dest: &mut [u64], pattern: u64) -> OptResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            fill_pattern_u64(dest.as_mut_ptr(), pattern, dest.len());
        }
        Ok(())
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback
        dest.fill(pattern);
        Ok(())
    }
}

/// Find pattern in buffer
pub fn find_pattern_safe(haystack: &[u8], needle: &[u8]) -> OptResult<Option<usize>> {
    if needle.is_empty() {
        return Ok(Some(0));
    }
    
    #[cfg(c_hotspots_available)]
    {
        let result = unsafe {
            find_pattern(
                haystack.as_ptr(), 
                haystack.len(),
                needle.as_ptr(),
                needle.len()
            )
        };
        Ok(if result == usize::MAX { None } else { Some(result) })
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback using windows iterator
        Ok(haystack.windows(needle.len()).position(|window| window == needle))
    }
}

/// Validate buffer bounds
pub fn validate_buffer_bounds_safe(buffer: &[u8], access_size: usize) -> OptResult<bool> {
    #[cfg(c_hotspots_available)]
    {
        let result = unsafe {
            validate_buffer_bounds(
                buffer.as_ptr() as *const std::ffi::c_void,
                buffer.len(),
                access_size
            )
        };
        Ok(result != 0)
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback
        Ok(access_size <= buffer.len())
    }
}

/// Detect buffer overflow
pub fn detect_buffer_overflow_safe(buffer: &[u8], expected_size: usize) -> OptResult<bool> {
    #[cfg(c_hotspots_available)]
    {
        let result = unsafe {
            detect_buffer_overflow(
                buffer.as_ptr() as *const std::ffi::c_void,
                expected_size
            )
        };
        Ok(result != 0)
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback - simple size check
        Ok(buffer.len() != expected_size)
    }
}

/// High-performance memory allocator wrapper
pub struct MediaAllocatorWrapper {
    #[cfg(c_hotspots_available)]
    #[allow(dead_code)]
    allocator: *mut std::ffi::c_void,
    #[cfg(not(c_hotspots_available))]
    _data: Vec<u8>,
}

impl MediaAllocatorWrapper {
    pub fn new(total_size: usize, alignment: usize) -> OptResult<Self> {
        #[cfg(c_hotspots_available)]
        {
            // Call C create_media_allocator function
            let allocator = unsafe {
                create_media_allocator(total_size, alignment)
            };
            
            if allocator.is_null() {
                Err(OptError::Memory("Failed to create media allocator".to_string()))
            } else {
                Ok(Self { allocator: allocator as *mut std::ffi::c_void })
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback - use Vec as backing store
            let _ = alignment; // Ignore alignment for now in fallback
            Ok(Self { _data: Vec::with_capacity(total_size) })
        }
    }
}

impl Drop for MediaAllocatorWrapper {
    fn drop(&mut self) {
        #[cfg(c_hotspots_available)]
        {
            // Call C destroy_media_allocator function
            if !self.allocator.is_null() {
                unsafe {
                    destroy_media_allocator(self.allocator as *mut MediaAllocator);
                }
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback - Vec handles cleanup automatically
        }
    }
}

/// Memory pool for frequent allocations
pub struct MemoryPoolWrapper {
    #[cfg(c_hotspots_available)]
    #[allow(dead_code)]
    pool: *mut std::ffi::c_void,
    #[cfg(not(c_hotspots_available))]
    _blocks: Vec<Vec<u8>>,
    #[cfg(not(c_hotspots_available))]
    #[allow(dead_code)]
    block_size: usize,
}

impl MemoryPoolWrapper {
    pub fn new(block_size: usize, initial_block_count: usize) -> OptResult<Self> {
        #[cfg(c_hotspots_available)]
        {
            // Call C create_memory_pool function
            let pool = unsafe {
                create_memory_pool(block_size, initial_block_count)
            };
            
            if pool.is_null() {
                Err(OptError::Memory("Failed to create memory pool".to_string()))
            } else {
                Ok(Self { pool: pool as *mut std::ffi::c_void })
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback
            let mut blocks = Vec::with_capacity(initial_block_count);
            for _ in 0..initial_block_count {
                blocks.push(vec![0u8; block_size]);
            }
            Ok(Self { _blocks: blocks, block_size })
        }
    }
}

impl Drop for MemoryPoolWrapper {
    fn drop(&mut self) {
        #[cfg(c_hotspots_available)]
        {
            // Call C destroy_memory_pool function
            if !self.pool.is_null() {
                unsafe {
                    destroy_memory_pool(self.pool as *mut MemoryPool);
                }
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback - Vec handles cleanup automatically
        }
    }
}

/// Zero-copy buffer wrapper
pub struct ZeroCopyBufferWrapper {
    #[cfg(c_hotspots_available)]
    #[allow(dead_code)]
    buffer: *mut std::ffi::c_void,
    #[cfg(not(c_hotspots_available))]
    _data: Vec<u8>,
}

impl ZeroCopyBufferWrapper {
    pub fn new(capacity: usize) -> OptResult<Self> {
        #[cfg(c_hotspots_available)]
        {
            let buffer = unsafe {
                create_zero_copy_buffer(capacity)
            };
            
            if buffer.is_null() {
                Err(OptError::Memory("Failed to create zero-copy buffer".to_string()))
            } else {
                Ok(Self { buffer })
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback
            Ok(Self { _data: Vec::with_capacity(capacity) })
        }
    }
    
    pub fn wrap(data: Vec<u8>) -> OptResult<Self> {
        #[cfg(c_hotspots_available)]
        {
            let mut data = data;
            let ptr = data.as_mut_ptr() as *mut std::ffi::c_void;
            let size = data.len();
            
            // Prevent Vec from deallocating - transfer ownership to C
            std::mem::forget(data);
            
            let buffer = unsafe {
                wrap_zero_copy_buffer(ptr, size, std::ptr::null_mut())
            };
            
            if buffer.is_null() {
                // If wrapping failed, we need to restore the Vec and free it
                unsafe {
                    let _ = Vec::from_raw_parts(ptr as *mut u8, size, size);
                }
                Err(OptError::Memory("Failed to wrap zero-copy buffer".to_string()))
            } else {
                Ok(Self { buffer })
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback
            Ok(Self { _data: data })
        }
    }
}

impl Drop for ZeroCopyBufferWrapper {
    fn drop(&mut self) {
        #[cfg(c_hotspots_available)]
        {
            if !self.buffer.is_null() {
                unsafe {
                    release_zero_copy_buffer(self.buffer);
                }
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            // Rust fallback - Vec handles cleanup automatically
        }
    }
}
