//! FFI module for C hotspot integration
//! 
//! This module provides safe Rust wrappers around C performance hotspots.
//! Each sub-module corresponds to a specific category of optimizations.

// Include generated C bindings when C hotspots are available
#[cfg(c_hotspots_available)]
mod c_bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(c_hotspots_available)]
pub use c_bindings::*;

pub mod image_ffi;
pub mod math_ffi;
pub mod memory_ffi;
pub mod mesh_ffi;

// Re-export commonly used types and functions
pub use image_ffi::{Color32, QuantizedImageWrapper};
pub use math_ffi::{Vec3, Vec4, Mat4, Quat};
pub use memory_ffi::{MediaAllocatorWrapper, MemoryPoolWrapper, ZeroCopyBufferWrapper, MemoryStats};
pub use mesh_ffi::{MeshData, decimate_mesh_safe, weld_vertices_safe};

// Re-export batch operation functions
pub use math_ffi::{
    vec3_add_batch, vec3_sub_batch, vec3_mul_scalar_batch, vec3_dot_batch, vec3_cross_batch,
    mat4_multiply_batch, transform_points_batch, transform_vectors_batch, quat_slerp_batch_safe
};

pub use image_ffi::{
    quantize_colors_octree_safe, quantize_colors_median_cut_safe,
    apply_floyd_steinberg_dither_safe, apply_ordered_dither_safe,
    apply_gaussian_blur_safe, apply_sharpen_filter_safe, apply_edge_detection_safe,
    rgb_to_yuv_safe, yuv_to_rgb_safe
};

pub use memory_ffi::{
    memcpy_simd_safe, memset_simd_safe, memcmp_fast_safe,
    prefetch_memory_safe, flush_cache_safe, get_cache_line_size_safe,
    fill_pattern_u32_safe, fill_pattern_u64_safe, find_pattern_safe,
    validate_buffer_bounds_safe, detect_buffer_overflow_safe
};
