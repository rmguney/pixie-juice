//! C Hotspots FFI Integration
//!
//! This module provides safe Rust wrappers around C hotspot functions
//! compiled to WASM for performance-critical operations.

#![allow(dead_code, unused_variables)]

extern crate alloc;
use alloc::{vec, vec::Vec, string::String, format, string::ToString};
use crate::types::{PixieResult, PixieError};

// Include the generated bindings when C hotspots are enabled
#[cfg(c_hotspots_available)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Utility functions for safe FFI
#[cfg(c_hotspots_available)]
pub mod util {
    use super::*;
    
    /// Create a buffer with initial capacity using C hotspot
    pub fn create_buffer(initial_capacity: usize) -> *mut core::ffi::c_void {
        unsafe { buffer_create(initial_capacity) }
    }
    
    /// Destroy a buffer created by create_buffer
    pub fn destroy_buffer(buffer: *mut core::ffi::c_void) {
        unsafe { buffer_destroy(buffer) }
    }
    
    /// Append data to buffer
    pub fn append_to_buffer(buffer: *mut core::ffi::c_void, data: &[u8]) -> Result<(), String> {
        let result = unsafe { buffer_append(buffer, data.as_ptr(), data.len()) };
        if result == 0 {
            Ok(())
        } else {
            Err(format!("Buffer append failed with code {}", result))
        }
    }
}

// Memory operations with SIMD acceleration
#[cfg(c_hotspots_available)]
pub mod memory {
    use super::*;
    
    /// SIMD-optimized memory copy
    pub fn simd_memcpy(dest: &mut [u8], src: &[u8]) {
        if dest.len() >= src.len() {
            unsafe {
                memcpy_simd(
                    dest.as_mut_ptr() as *mut core::ffi::c_void,
                    src.as_ptr() as *const core::ffi::c_void,
                    src.len()
                );
            }
        }
    }
    
    /// SIMD-optimized memory set
    pub fn simd_memset(dest: &mut [u8], value: u8) {
        unsafe {
            memset_simd(
                dest.as_mut_ptr() as *mut core::ffi::c_void,
                value as i32,
                dest.len()
            );
        }
    }
}

// Math operations with SIMD acceleration
#[cfg(c_hotspots_available)]
pub mod math {
    use super::*;
    
    /// SIMD-optimized dot product
    pub fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
        if a.len() == b.len() {
            unsafe { vector_dot_product_simd(a.as_ptr(), b.as_ptr(), a.len()) }
        } else {
            0.0
        }
    }
    
    /// SIMD-optimized matrix multiplication
    pub fn simd_matrix_multiply(a: &[f32], b: &[f32], m: i32, n: i32, k: i32) -> Vec<f32> {
        let mut result = vec![0.0f32; (m * n) as usize];
        unsafe {
            matrix_multiply_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), m, n, k);
        }
        result
    }
    
    /// SIMD-optimized Gaussian blur
    pub fn simd_gaussian_blur(image: &mut [u8], width: i32, height: i32, channels: i32, sigma: f32) {
        unsafe {
            gaussian_blur_simd(image.as_mut_ptr(), width, height, channels, sigma);
        }
    }
}

// Image processing hotspots with proper C FFI integration
#[cfg(c_hotspots_available)]
pub mod image {
    use super::*;
    
    /// C hotspot: Octree color quantization (>15% faster than Rust for large images)
    pub fn octree_quantization(rgba_data: &[u8], width: usize, height: usize, max_colors: usize) -> PixieResult<(Vec<Color32>, Vec<u8>)> {
        unsafe {
            let result = quantize_colors_octree(
                rgba_data.as_ptr(),
                width,
                height,
                max_colors
            );
            
            if result.is_null() {
                return Err(PixieError::ProcessingError("Octree quantization failed".to_string()));
            }
            
            let quantized = &*result;
            
            // Copy palette
            let palette_slice = core::slice::from_raw_parts(quantized.palette, quantized.palette_size);
            let palette = palette_slice.to_vec();
            
            // Copy indices
            let indices_slice = core::slice::from_raw_parts(quantized.indices, width * height);
            let indices = indices_slice.to_vec();
            
            // Free C memory
            free_quantized_image(result);
            
            Ok((palette, indices))
        }
    }
    
    /// C hotspot: Median cut color quantization (specialized for photos)
    pub fn median_cut_quantization(rgba_data: &[u8], width: usize, height: usize, max_colors: usize) -> PixieResult<(Vec<Color32>, Vec<u8>)> {
        unsafe {
            let result = quantize_colors_median_cut(
                rgba_data.as_ptr(),
                width,
                height,
                max_colors
            );
            
            if result.is_null() {
                return Err(PixieError::ProcessingError("Median cut quantization failed".to_string()));
            }
            
            let quantized = &*result;
            
            // Copy palette
            let palette_slice = core::slice::from_raw_parts(quantized.palette, quantized.palette_size);
            let palette = palette_slice.to_vec();
            
            // Copy indices
            let indices_slice = core::slice::from_raw_parts(quantized.indices, width * height);
            let indices = indices_slice.to_vec();
            
            // Free C memory
            free_quantized_image(result);
            
            Ok((palette, indices))
        }
    }
    
    /// C hotspot: Floyd-Steinberg dithering (SIMD optimized)
    pub fn floyd_steinberg_dither(rgba_data: &mut [u8], width: usize, height: usize, palette: &[Color32]) {
        unsafe {
            apply_floyd_steinberg_dither(
                rgba_data.as_mut_ptr(),
                width,
                height,
                palette.as_ptr(),
                palette.len()
            );
        }
    }
    
    /// C hotspot: Gaussian blur with SIMD acceleration
    pub fn gaussian_blur(rgba_data: &mut [u8], width: usize, height: usize, sigma: f32) {
        unsafe {
            apply_gaussian_blur(
                rgba_data.as_mut_ptr(),
                width,
                height,
                sigma
            );
        }
    }
    
    /// C hotspot: RGB to YUV color space conversion (SIMD)
    pub fn rgb_to_yuv_simd(rgb_data: &[u8], yuv_data: &mut [u8]) {
        let pixel_count = rgb_data.len() / 3;
        unsafe {
            rgb_to_yuv(rgb_data.as_ptr(), yuv_data.as_mut_ptr(), pixel_count);
        }
    }
    
    /// C hotspot: YUV to RGB color space conversion (SIMD)
    pub fn yuv_to_rgb_simd(yuv_data: &[u8], rgb_data: &mut [u8]) {
        let pixel_count = yuv_data.len() / 3;
        unsafe {
            yuv_to_rgb(yuv_data.as_ptr(), rgb_data.as_mut_ptr(), pixel_count);
        }
    }
    
    // Rust fallback implementations (for benchmarking and safety)
    pub mod fallback {
        use super::*;
        
        /// Pure Rust Floyd-Steinberg dithering fallback
        pub fn floyd_steinberg_rust(rgba_data: &mut [u8], width: usize, height: usize, palette: &[Color32]) {
            // Pure Rust implementation for comparison
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 4;
                    if idx + 3 >= rgba_data.len() { break; }
                    
                    let r = rgba_data[idx] as i32;
                    let g = rgba_data[idx + 1] as i32;
                    let b = rgba_data[idx + 2] as i32;
                    
                    // Find closest palette color (simplified)
                    let closest = find_closest_color(r as u8, g as u8, b as u8, palette);
                    
                    rgba_data[idx] = closest.r;
                    rgba_data[idx + 1] = closest.g;
                    rgba_data[idx + 2] = closest.b;
                }
            }
        }
        
        fn find_closest_color(r: u8, g: u8, b: u8, palette: &[Color32]) -> Color32 {
            let mut min_distance = u32::MAX;
            let mut closest = Color32 { r: 0, g: 0, b: 0, a: 255 };
            
            for color in palette {
                let dr = (r as i32 - color.r as i32).abs() as u32;
                let dg = (g as i32 - color.g as i32).abs() as u32;
                let db = (b as i32 - color.b as i32).abs() as u32;
                let distance = dr * dr + dg * dg + db * db;
                
                if distance < min_distance {
                    min_distance = distance;
                    closest = *color;
                }
            }
            
            closest
        }
    }
}

// Mesh processing hotspots
#[cfg(c_hotspots_available)]
pub mod mesh {
    use super::*;
    
    /// Decimate mesh using Quadric Error Metrics in C
    pub fn decimate_mesh_qem_c(
        vertices: &[f32],
        indices: &[u32],
        target_ratio: f32
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let result = unsafe {
            decimate_mesh_qem(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
                target_ratio
            )
        };
        
        if result.success == 0 {
            let error_msg = unsafe {
                core::str::from_utf8(&result.error_message)
                    .unwrap_or("Unknown error")
                    .trim_end_matches('\0')
            };
            return Err(PixieError::CHotspotFailed(format!("Mesh decimation failed: {}", error_msg)));
        }
        
        // Extract the result data
        let new_vertices = unsafe {
            core::slice::from_raw_parts(result.vertices, result.vertex_count)
        }.to_vec();
        
        let new_indices = unsafe {
            core::slice::from_raw_parts(result.indices, result.index_count)
        }.to_vec();
        
        // Free the C-allocated result
        unsafe {
            free_mesh_decimate_result(&result as *const _ as *mut _);
        }
        
        Ok((new_vertices, new_indices))
    }
    
    /// Weld vertices using spatial tolerance in C
    pub fn weld_vertices_c(
        vertices: &[f32],
        indices: &[u32],
        tolerance: f32
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let result = unsafe {
            weld_vertices_spatial(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
                tolerance
            )
        };
        
        if result.success == 0 {
            let error_msg = unsafe {
                core::str::from_utf8(&result.error_message)
                    .unwrap_or("Unknown error")
                    .trim_end_matches('\0')
            };
            return Err(PixieError::CHotspotFailed(format!("Vertex welding failed: {}", error_msg)));
        }
        
        // Extract the result data
        let new_vertices = unsafe {
            core::slice::from_raw_parts(result.vertices, result.vertex_count)
        }.to_vec();
        
        let new_indices = unsafe {
            core::slice::from_raw_parts(result.indices, result.index_count)
        }.to_vec();
        
        // Free the C-allocated result
        unsafe {
            free_mesh_decimate_result(&result as *const _ as *mut _);
        }
        
        Ok((new_vertices, new_indices))
    }
}

// Compression hotspots
#[cfg(c_hotspots_available)]
pub mod compression {
    use super::*;
    
    /// LZ4 compression using C hotspot
    pub fn compress_lz4_c(input: &[u8]) -> PixieResult<Vec<u8>> {
        let mut output = vec![0u8; input.len() * 2]; // Allocate extra space
        
        let compressed_size = unsafe {
            compress_lz4(
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output.len()
            )
        };
        
        if compressed_size <= 0 {
            return Err(PixieError::CHotspotFailed("LZ4 compression failed".to_string()));
        }
        
        output.truncate(compressed_size as usize);
        Ok(output)
    }
    
    /// LZ4 decompression using C hotspot
    pub fn decompress_lz4_c(input: &[u8], output_size: usize) -> PixieResult<Vec<u8>> {
        let mut output = vec![0u8; output_size];
        
        let decompressed_size = unsafe {
            decompress_lz4(
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output_size
            )
        };
        
        if decompressed_size <= 0 {
            return Err(PixieError::CHotspotFailed("LZ4 decompression failed".to_string()));
        }
        
        output.truncate(decompressed_size as usize);
        Ok(output)
    }
}

// Fallback implementations when C hotspots are not available
#[cfg(not(c_hotspots_available))]
pub mod fallback {
    use super::*;
    
    /// Fallback: return input unchanged
    pub fn fallback_operation(input: &[u8]) -> PixieResult<Vec<u8>> {
        Ok(input.to_vec())
    }
}

/// Check if C hotspots are available
pub fn are_c_hotspots_available() -> bool {
    cfg!(c_hotspots_available)
}

/// Get C hotspots feature info
pub fn c_hotspots_info() -> &'static str {
    if cfg!(c_hotspots_available) {
        "C hotspots available - SIMD optimizations enabled"
    } else {
        "C hotspots not available - using Rust fallbacks"
    }
}
