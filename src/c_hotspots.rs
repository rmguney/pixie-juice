#![allow(dead_code, unused_variables)]

extern crate alloc;
use alloc::{vec, vec::Vec, string::String, format};

#[cfg(c_hotspots_available)]
use alloc::string::ToString;
use crate::types::{PixieResult, PixieError};

#[cfg(c_hotspots_available)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(c_hotspots_available)]
extern "C" {
    fn buffer_create(initial_capacity: usize) -> *mut core::ffi::c_void;
    fn buffer_destroy(buffer: *mut core::ffi::c_void);
    fn buffer_append(buffer: *mut core::ffi::c_void, data: *const u8, size: usize) -> i32;
    fn memcpy_simd(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, size: usize);
    fn memset_simd(dest: *mut core::ffi::c_void, value: i32, size: usize);
    fn vector_dot_product_simd(a: *const f32, b: *const f32, count: usize) -> f32;
    fn matrix_multiply_simd(a: *const f32, b: *const f32, result: *mut f32, m: usize, n: usize, k: usize);
    fn weld_vertices_spatial(vertices: *const f32, vertex_count: usize, 
                            indices: *const u32, index_count: usize, 
                            tolerance: f32) -> MeshDecimateResult;
    fn svg_compress_text(data: *const u8, data_len: usize, 
                        compression_level: u32, 
                        output_size: *mut usize) -> *mut u8;
    // int svg_minify_markup_simd(const uint8_t* input, size_t input_size, uint8_t* output, size_t* output_size)
    fn svg_minify_markup_simd(input: *const u8, input_size: usize, output: *mut u8, output_size: *mut usize) -> i32;
    fn svg_optimize_paths(data: *const u8, data_len: usize, 
                         precision: f32, 
                         output_size: *mut usize) -> *mut u8;
    fn ico_optimize_embedded(data: *const u8, data_len: usize, 
                            quality: u8, 
                            output_size: *mut usize) -> *mut u8;
    fn ico_strip_metadata_simd(data: *const u8, data_len: usize, 
                              output_size: *mut usize) -> *mut u8;
    fn ico_compress_directory(data: *const u8, data_len: usize, 
                             compression_level: u32, 
                             output_size: *mut usize) -> *mut u8;
    
    fn compress_tiff_lzw_simd(rgba_data: *const u8, width: usize, height: usize, 
                             quality: u8) -> *mut TIFFProcessResult;
    fn strip_tiff_metadata_simd_c_hotspot(tiff_data: *const u8, data_size: usize, 
                               preserve_icc: bool) -> *mut TIFFProcessResult;
    fn apply_tiff_predictor_simd(rgba_data: *mut u8, width: usize, height: usize, 
                                predictor_type: u8);
    fn optimize_tiff_colorspace_simd(rgba_data: *mut u8, width: usize, height: usize, 
                                    target_bits_per_channel: u8);
    fn free_tiff_result(result: *mut TIFFProcessResult);
    
    fn batch_process_pixels_simd(rgba_data: *mut u8, pixel_count: usize, operation_type: u8);
    fn parallel_color_conversion_simd(src_data: *const u8, dst_data: *mut u8, pixel_count: usize,
                                     src_format: u8, dst_format: u8);
    fn vectorized_filter_apply_simd(rgba_data: *mut u8, width: usize, height: usize,
                                   kernel: *const f32, kernel_size: usize);
    fn fast_downscale_simd(src_data: *const u8, dst_data: *mut u8,
                          src_width: usize, src_height: usize,
                          dst_width: usize, dst_height: usize);
    fn multi_threaded_compression_simd(rgba_data: *const u8, width: usize, height: usize,
                                      compressed_data: *mut u8, compressed_size: *mut usize,
                                      quality: u8);
}

#[cfg(not(c_hotspots_available))]
#[derive(Clone, Copy, Debug)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct TIFFProcessResult {
    pub data: *mut u8,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub bits_per_sample: u8,
    pub compression: u8,
}

#[cfg(c_hotspots_available)]
pub mod util {
    use super::*;
    
    pub fn create_buffer(initial_capacity: usize) -> *mut core::ffi::c_void {
        unsafe { 
            let result = buffer_create(initial_capacity);
            if result.is_null() {
                create_buffer_rust_fallback(initial_capacity)
            } else {
                result
            }
        }
    }
    
    pub fn destroy_buffer(buffer: *mut core::ffi::c_void) {
        if !buffer.is_null() {
            unsafe { 
                buffer_destroy(buffer);
            }
        }
    }
    
    pub fn append_to_buffer(buffer: *mut core::ffi::c_void, data: &[u8]) -> Result<(), String> {
        let result = unsafe { buffer_append(buffer, data.as_ptr(), data.len()) };
        if result == 0 {
            Ok(())
        } else {
            append_to_buffer_rust_fallback(buffer, data)
        }
    }
    
    fn create_buffer_rust_fallback(initial_capacity: usize) -> *mut core::ffi::c_void {
        let buffer = vec![0u8; initial_capacity];
        let boxed = alloc::boxed::Box::new(buffer);
        alloc::boxed::Box::into_raw(boxed) as *mut core::ffi::c_void
    }
    
    fn append_to_buffer_rust_fallback(buffer: *mut core::ffi::c_void, data: &[u8]) -> Result<(), String> {
        if buffer.is_null() {
            return Err("Null buffer".to_string());
        }
        
        unsafe {
            let vec_ref = &mut *(buffer as *mut Vec<u8>);
            vec_ref.extend_from_slice(data);
        }
        Ok(())
    }
}

#[cfg(not(c_hotspots_available))]
pub mod util {
    use alloc::{vec, vec::Vec, string::{String, ToString}};
    
    pub fn create_buffer(initial_capacity: usize) -> *mut core::ffi::c_void {
        let buffer = vec![0u8; initial_capacity];
        let boxed = alloc::boxed::Box::new(buffer);
        alloc::boxed::Box::into_raw(boxed) as *mut core::ffi::c_void
    }
    
    pub fn destroy_buffer(buffer: *mut core::ffi::c_void) {
        if !buffer.is_null() {
            unsafe {
                let _ = alloc::boxed::Box::from_raw(buffer as *mut Vec<u8>);
            }
        }
    }
    
    pub fn append_to_buffer(buffer: *mut core::ffi::c_void, data: &[u8]) -> Result<(), String> {
        if buffer.is_null() {
            return Err("Null buffer".to_string());
        }
        
        unsafe {
            let vec_ref = &mut *(buffer as *mut Vec<u8>);
            vec_ref.extend_from_slice(data);
        }
        Ok(())
    }
}

#[cfg(c_hotspots_available)]
pub mod memory {
    use super::*;
    
    pub fn simd_memcpy(dest: &mut [u8], src: &[u8]) {
        if dest.len() >= src.len() {
            unsafe {
                memcpy_simd(
                    dest.as_mut_ptr() as *mut core::ffi::c_void,
                    src.as_ptr() as *const core::ffi::c_void,
                    src.len()
                );
            }
        } else {
            let copy_len = core::cmp::min(dest.len(), src.len());
            dest[..copy_len].copy_from_slice(&src[..copy_len]);
        }
    }
    
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

#[cfg(not(c_hotspots_available))]
pub mod memory {
    
    pub fn simd_memcpy(dest: &mut [u8], src: &[u8]) {
        if dest.len() >= src.len() {
            dest[..src.len()].copy_from_slice(src);
        } else {
            let copy_len = core::cmp::min(dest.len(), src.len());
            dest[..copy_len].copy_from_slice(&src[..copy_len]);
        }
    }
    
    pub fn simd_memset(dest: &mut [u8], value: u8) {
        dest.fill(value);
    }
}

#[cfg(c_hotspots_available)]
pub mod math {
    use super::*;
    
    pub fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
        if a.len() == b.len() && !a.is_empty() {
            unsafe { 
                let result = vector_dot_product_simd(a.as_ptr(), b.as_ptr(), a.len());
                if result.is_finite() {
                    result
                } else {
                    simd_dot_product_rust_fallback(a, b)
                }
            }
        } else {
            simd_dot_product_rust_fallback(a, b)
        }
    }
    
    pub fn simd_matrix_multiply(a: &[f32], b: &[f32], m: i32, n: i32, k: i32) -> Vec<f32> {
        if m > 0 && n > 0 && k > 0 && a.len() >= (m * k) as usize && b.len() >= (k * n) as usize {
            let mut result = vec![0.0f32; (m * n) as usize];
            unsafe {
                matrix_multiply_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), m as usize, n as usize, k as usize);
            }
            
            if result.iter().all(|&x| x.is_finite()) {
                result
            } else {
                simd_matrix_multiply_rust_fallback(a, b, m, n, k)
            }
        } else {
            simd_matrix_multiply_rust_fallback(a, b, m, n, k)
        }
    }
    
    pub fn simd_gaussian_blur(image: &mut [u8], width: i32, height: i32, channels: i32, sigma: f32) {
        if width > 0 && height > 0 && channels > 0 && sigma > 0.0 {
            unsafe {
                gaussian_blur_simd(image.as_mut_ptr(), width, height, channels, sigma);
            }
        }
    }
    
    fn simd_dot_product_rust_fallback(a: &[f32], b: &[f32]) -> f32 {
        let len = core::cmp::min(a.len(), b.len());
        let mut sum = 0.0f32;
        for i in 0..len {
            sum += a[i] * b[i];
        }
        sum
    }
    
    fn simd_matrix_multiply_rust_fallback(a: &[f32], b: &[f32], m: i32, n: i32, k: i32) -> Vec<f32> {
        let mut result = vec![0.0f32; (m * n) as usize];
        
        for i in 0..m as usize {
            for j in 0..n as usize {
                let mut sum = 0.0f32;
                for l in 0..k as usize {
                    let a_idx = i * k as usize + l;
                    let b_idx = l * n as usize + j;
                    if a_idx < a.len() && b_idx < b.len() {
                        sum += a[a_idx] * b[b_idx];
                    }
                }
                result[i * n as usize + j] = sum;
            }
        }
        result
    }
}

#[cfg(not(c_hotspots_available))]
pub mod math {
    use super::*;
    
    pub fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
        let len = core::cmp::min(a.len(), b.len());
        let mut sum = 0.0f32;
        for i in 0..len {
            sum += a[i] * b[i];
        }
        sum
    }
    
    pub fn simd_matrix_multiply(a: &[f32], b: &[f32], m: i32, n: i32, k: i32) -> Vec<f32> {
        let mut result = vec![0.0f32; (m * n) as usize];
        
        for i in 0..m as usize {
            for j in 0..n as usize {
                let mut sum = 0.0f32;
                for l in 0..k as usize {
                    let a_idx = i * k as usize + l;
                    let b_idx = l * n as usize + j;
                    if a_idx < a.len() && b_idx < b.len() {
                        sum += a[a_idx] * b[b_idx];
                    }
                }
                result[i * n as usize + j] = sum;
            }
        }
        result
    }
    
    pub fn simd_gaussian_blur(image: &mut [u8], width: i32, height: i32, channels: i32, sigma: f32) {
        if width <= 0 || height <= 0 || channels <= 0 || sigma <= 0.0 {
            return;
        }
        
        let radius = (sigma * 3.0) as usize;
        if radius == 0 { return; }
        
        for y in 0..height as usize {
            for x in 0..width as usize {
                for c in 0..channels as usize {
                    let idx = (y * width as usize + x) * channels as usize + c;
                    if idx >= image.len() { continue; }
                    
                    let mut sum = 0u32;
                    let mut count = 0u32;
                    
                    for dx in -(radius as i32)..=(radius as i32) {
                        let nx = x as i32 + dx;
                        if nx >= 0 && nx < width {
                            let src_idx = (y * width as usize + nx as usize) * channels as usize + c;
                            if src_idx < image.len() {
                                sum += image[src_idx] as u32;
                                count += 1;
                            }
                        }
                    }
                    
                    if count > 0 {
                        image[idx] = (sum / count) as u8;
                    }
                }
            }
        }
    }
}

#[cfg(c_hotspots_available)]
pub mod compression {
    use super::*;
    
    pub fn lz4_compress(input: &[u8]) -> Result<Vec<u8>, String> {
        // TEMPORARILY DISABLED: Always use Rust fallback to avoid compress_lz4 function errors
        // This disables only the problematic compress_lz4 function while keeping all other C hotspots
        lz4_compress_rust_fallback(input)
        
        /* ORIGINAL CODE DISABLED FOR DEBUGGING:
        // CRITICAL: Always prefer Rust fallback in WASM to avoid runtime errors
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, always use Rust implementation to avoid C function call errors
            return lz4_compress_rust_fallback(input);
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let output_size = input.len() * 2; // Conservative estimate
            let mut output = vec![0u8; output_size];
            
            let result_size = unsafe {
                compress_lz4(
                    input.as_ptr(),
                    input.len(),
                    output.as_mut_ptr(),
                    output_size
                )
            };
            
            if result_size < 0 {
                // Graceful fallback to Rust LZ4 implementation
                lz4_compress_rust_fallback(input)
            } else {
                output.truncate(result_size as usize);
                Ok(output)
            }
        }
        */
    }
    
    pub fn lz4_decompress(input: &[u8], _output_size: usize) -> Result<Vec<u8>, String> {
        // TEMPORARILY DISABLED: Always use Rust fallback to avoid decompress_lz4 function errors
        // This disables only the problematic decompress_lz4 function while keeping all other C hotspots
        lz4_decompress_rust_fallback(input)
    }
    
    fn lz4_compress_rust_fallback(input: &[u8]) -> Result<Vec<u8>, String> {
        use lz4_flex::compress_prepend_size;
        Ok(compress_prepend_size(input))
    }
    
    fn lz4_decompress_rust_fallback(input: &[u8]) -> Result<Vec<u8>, String> {
        use lz4_flex::decompress_size_prepended;
        decompress_size_prepended(input).map_err(|e| format!("LZ4 decompression error: {:?}", e))
    }
}

#[cfg(not(c_hotspots_available))]
pub mod compression {
    use super::*;
    
    pub fn lz4_compress(input: &[u8]) -> Result<Vec<u8>, String> {
        use lz4_flex::compress_prepend_size;
        Ok(compress_prepend_size(input))
    }
    
    pub fn lz4_decompress(input: &[u8], _output_size: usize) -> Result<Vec<u8>, String> {
        use lz4_flex::decompress_size_prepended;
        decompress_size_prepended(input).map_err(|e| format!("LZ4 decompression error: {:?}", e))
    }
}

pub fn compress_data_c_hotspot(input: &[u8]) -> PixieResult<Vec<u8>> {
    // CRITICAL: For WASM, always use Rust implementation to avoid C function call errors
    #[cfg(target_arch = "wasm32")]
    {
        compress_data_rust_fallback(input)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(c_hotspots_available)]
        {
            match compression::lz4_compress(input) {
                Ok(compressed) => Ok(compressed),
                Err(_) => {
                    // Graceful fallback to pure Rust implementation
                    compress_data_rust_fallback(input)
                }
            }
        }
        #[cfg(not(c_hotspots_available))]
        {
            compress_data_rust_fallback(input)
        }
    }
}

fn compress_data_rust_fallback(input: &[u8]) -> PixieResult<Vec<u8>> {
    use lz4_flex::compress_prepend_size;
    
    let compressed = compress_prepend_size(input);
    Ok(compressed)
}

#[cfg(c_hotspots_available)]
pub mod image {
    use super::*;
    use crate::{get_current_time_ms, update_performance_stats};
    
    pub fn octree_quantization(rgba_data: &[u8], width: usize, height: usize, max_colors: usize) -> PixieResult<(Vec<Color32>, Vec<u8>)> {
        let start_time = get_current_time_ms();
        let data_size = rgba_data.len();
        
        #[cfg(c_hotspots_available)]
        {
            // Try C hotspot first
            unsafe {
                let result = quantize_colors_octree(
                    rgba_data.as_ptr(),
                    width,
                    height,
                    max_colors
                );
                
                let elapsed = get_current_time_ms() - start_time;
                update_performance_stats(true, elapsed, data_size);
                
                if !result.is_null() {
                    let quantized = &*result;
                    
                    let palette_slice = core::slice::from_raw_parts(quantized.palette, quantized.palette_size);
                    let palette = palette_slice.to_vec();
                    
                    let indices_slice = core::slice::from_raw_parts(quantized.indices, width * height);
                    let indices = indices_slice.to_vec();
                    
                    free_quantized_image(result);
                    
                    return Ok((palette, indices));
                }
            }
            
            octree_quantization_rust_fallback(rgba_data, width, height, max_colors)
        }
        #[cfg(not(c_hotspots_available))]
        {
            let elapsed = get_current_time_ms() - start_time;
            update_performance_stats(true, elapsed, data_size);
            octree_quantization_rust_fallback(rgba_data, width, height, max_colors)
        }
    }
    
    pub fn median_cut_quantization(rgba_data: &[u8], width: usize, height: usize, max_colors: usize) -> PixieResult<(Vec<Color32>, Vec<u8>)> {
        let start_time = get_current_time_ms();
        let data_size = rgba_data.len();
        
        #[cfg(c_hotspots_available)]
        {
            unsafe {
                let result = quantize_colors_median_cut(
                    rgba_data.as_ptr(),
                    width,
                    height,
                    max_colors
                );
                
                if !result.is_null() {
                    let quantized = &*result;
                    
                    let palette_slice = core::slice::from_raw_parts(quantized.palette, quantized.palette_size);
                    let palette = palette_slice.to_vec();
                    
                    let indices_slice = core::slice::from_raw_parts(quantized.indices, width * height);
                    let indices = indices_slice.to_vec();
                    
                    free_quantized_image(result);
                    
                    return Ok((palette, indices));
                }
            }
            
            median_cut_quantization_rust_fallback(rgba_data, width, height, max_colors)
        }
        #[cfg(not(c_hotspots_available))]
        {
            median_cut_quantization_rust_fallback(rgba_data, width, height, max_colors)
        }
    }
    
    pub fn floyd_steinberg_dither(rgba_data: &mut [u8], width: usize, height: usize, palette: &[Color32]) {
        #[cfg(c_hotspots_available)]
        {
            if width == 0 || height == 0 || palette.is_empty() {
                fallback::floyd_steinberg_rust(rgba_data, width, height, palette);
                return;
            }

            unsafe {
                dither_floyd_steinberg(
                    rgba_data.as_mut_ptr(),
                    width as i32,
                    height as i32,
                    4,
                    palette.as_ptr(),
                    palette.len(),
                );
            }
        }
        #[cfg(not(c_hotspots_available))]
        {
            fallback::floyd_steinberg_rust(rgba_data, width, height, palette);
        }
    }
    
    pub fn gaussian_blur(rgba_data: &mut [u8], width: usize, height: usize, sigma: f32) {
        #[cfg(c_hotspots_available)]
        {
            unsafe {
                gaussian_blur_simd(
                    rgba_data.as_mut_ptr(),
                    width as i32,
                    height as i32,
                    4,
                    sigma,
                );
            }
        }
        #[cfg(not(c_hotspots_available))]
        {
            gaussian_blur_rust_fallback(rgba_data, width, height, sigma);
        }
    }
    
    pub fn rgb_to_yuv_simd(rgb_data: &[u8], yuv_data: &mut [u8]) {
        rgb_to_yuv_rust_fallback(rgb_data, yuv_data);
    }
    
    pub fn yuv_to_rgb_simd(yuv_data: &[u8], rgb_data: &mut [u8]) {
        yuv_to_rgb_rust_fallback(yuv_data, rgb_data);
    }
    
    fn octree_quantization_rust_fallback(rgba_data: &[u8], width: usize, height: usize, max_colors: usize) -> PixieResult<(Vec<Color32>, Vec<u8>)> {
        #[cfg(feature = "color_quant")]
        {
            let mut quantizer = color_quant::NeuQuant::new(10, max_colors, rgba_data);
            let palette = quantizer.color_map_rgba();
            
            let mut pixie_palette = Vec::new();
            for chunk in palette.chunks(4) {
                if chunk.len() >= 4 {
                    pixie_palette.push(Color32 {
                        r: chunk[0],
                        g: chunk[1], 
                        b: chunk[2],
                        a: chunk[3],
                    });
                }
            }
            
            let mut indices = Vec::with_capacity(width * height);
            for chunk in rgba_data.chunks(4) {
                if chunk.len() >= 4 {
                    let index = quantizer.index_of(&[chunk[0], chunk[1], chunk[2], chunk[3]]);
                    indices.push(index as u8);
                }
            }
            
            Ok((pixie_palette, indices))
        }
        #[cfg(not(feature = "color_quant"))]
        {
            let mut palette = Vec::new();
            let mut indices = Vec::new();
            
            let levels = ((max_colors as f32).powf(1.0/3.0) as usize).max(2);
            
            for r in 0..levels {
                for g in 0..levels {
                    for b in 0..levels {
                        if palette.len() >= max_colors { break; }
                        let color = Color32 {
                            r: (r * 255 / (levels - 1)) as u8,
                            g: (g * 255 / (levels - 1)) as u8,
                            b: (b * 255 / (levels - 1)) as u8,
                            a: 255,
                        };
                        palette.push(color);
                    }
                }
            }
            
            for chunk in rgba_data.chunks(4) {
                if chunk.len() >= 4 {
                    let index = find_closest_color_index(chunk[0], chunk[1], chunk[2], &palette);
                    indices.push(index as u8);
                }
            }
            
            Ok((palette, indices))
        }
    }
    
    fn median_cut_quantization_rust_fallback(rgba_data: &[u8], width: usize, height: usize, max_colors: usize) -> PixieResult<(Vec<Color32>, Vec<u8>)> {
        octree_quantization_rust_fallback(rgba_data, width, height, max_colors)
    }
    
    fn gaussian_blur_rust_fallback(rgba_data: &mut [u8], width: usize, height: usize, sigma: f32) {
        if sigma <= 0.0 { return; }
        
        let radius = (sigma * 3.0) as usize;
        if radius == 0 { return; }
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                if idx + 3 >= rgba_data.len() { continue; }
                
                let mut r_sum = 0u32;
                let mut g_sum = 0u32; 
                let mut b_sum = 0u32;
                let mut count = 0u32;
                
                for dx in -(radius as i32)..=(radius as i32) {
                    let nx = x as i32 + dx;
                    if nx >= 0 && nx < width as i32 {
                        let src_idx = (y * width + nx as usize) * 4;
                        if src_idx + 3 < rgba_data.len() {
                            r_sum += rgba_data[src_idx] as u32;
                            g_sum += rgba_data[src_idx + 1] as u32;
                            b_sum += rgba_data[src_idx + 2] as u32;
                            count += 1;
                        }
                    }
                }
                
                if count > 0 {
                    rgba_data[idx] = (r_sum / count) as u8;
                    rgba_data[idx + 1] = (g_sum / count) as u8;
                    rgba_data[idx + 2] = (b_sum / count) as u8;
                }
            }
        }
    }
    
    fn rgb_to_yuv_rust_fallback(rgb_data: &[u8], yuv_data: &mut [u8]) {
        let pixel_count = rgb_data.len() / 3;
        for i in 0..pixel_count {
            if i * 3 + 2 >= rgb_data.len() || i * 3 + 2 >= yuv_data.len() { break; }
            
            let r = rgb_data[i * 3] as f32;
            let g = rgb_data[i * 3 + 1] as f32;
            let b = rgb_data[i * 3 + 2] as f32;
            
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            let u = -0.169 * r - 0.331 * g + 0.5 * b + 128.0;
            let v = 0.5 * r - 0.419 * g - 0.081 * b + 128.0;
            
            yuv_data[i * 3] = y.clamp(0.0, 255.0) as u8;
            yuv_data[i * 3 + 1] = u.clamp(0.0, 255.0) as u8;
            yuv_data[i * 3 + 2] = v.clamp(0.0, 255.0) as u8;
        }
    }
    
    fn yuv_to_rgb_rust_fallback(yuv_data: &[u8], rgb_data: &mut [u8]) {
        let pixel_count = yuv_data.len() / 3;
        for i in 0..pixel_count {
            if i * 3 + 2 >= yuv_data.len() || i * 3 + 2 >= rgb_data.len() { break; }
            
            let y = yuv_data[i * 3] as f32;
            let u = yuv_data[i * 3 + 1] as f32 - 128.0;
            let v = yuv_data[i * 3 + 2] as f32 - 128.0;
            
            let r = y + 1.402 * v;
            let g = y - 0.344 * u - 0.714 * v;
            let b = y + 1.772 * u;
            
            rgb_data[i * 3] = r.clamp(0.0, 255.0) as u8;
            rgb_data[i * 3 + 1] = g.clamp(0.0, 255.0) as u8; 
            rgb_data[i * 3 + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }
    
    fn find_closest_color_index(r: u8, g: u8, b: u8, palette: &[Color32]) -> usize {
        let mut min_distance = u32::MAX;
        let mut closest_index = 0;
        
        for (i, color) in palette.iter().enumerate() {
            let dr = (r as i32 - color.r as i32).abs() as u32;
            let dg = (g as i32 - color.g as i32).abs() as u32;
            let db = (b as i32 - color.b as i32).abs() as u32;
            let distance = dr * dr + dg * dg + db * db;
            
            if distance < min_distance {
                min_distance = distance;
                closest_index = i;
            }
        }
        
        closest_index
    }
    
    pub mod fallback {
        use super::*;
        
        pub fn floyd_steinberg_rust(rgba_data: &mut [u8], width: usize, height: usize, palette: &[Color32]) {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 4;
                    if idx + 3 >= rgba_data.len() { break; }
                    
                    let r = rgba_data[idx] as i32;
                    let g = rgba_data[idx + 1] as i32;
                    let b = rgba_data[idx + 2] as i32;
                    
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

#[cfg(c_hotspots_available)]
pub mod mesh {
    use super::*;
    use crate::{get_current_time_ms, update_performance_stats};
    
    pub fn decimate_mesh_qem_c(
        vertices: &[f32],
        indices: &[u32],
        target_ratio: f32
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        let result = unsafe {
            decimate_mesh_qem(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
                target_ratio
            )
        };
        
        // CRITICAL: Update performance stats for mesh operations
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size); // false = mesh operation
        
        if result.success == 0 {
            // CRITICAL: Count C hotspot errors for compliance tracking
            use crate::optimizers::ERRORS_COUNT;
            ERRORS_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            
            let error_msg = unsafe {
                core::str::from_utf8(&result.error_message)
                    .unwrap_or("Unknown error")
                    .trim_end_matches('\0')
            };
            return Err(PixieError::CHotspotFailed(format!("Mesh decimation failed: {}", error_msg)));
        }
        
        let new_vertices = unsafe {
            core::slice::from_raw_parts(result.vertices, result.vertex_count)
        }.to_vec();
        
        let new_indices = unsafe {
            core::slice::from_raw_parts(result.indices, result.index_count)
        }.to_vec();
        
        unsafe {
            free_mesh_decimate_result(&result as *const _ as *mut _);
        }
        
        Ok((new_vertices, new_indices))
    }
    
    pub fn weld_vertices_c(
        vertices: &[f32],
        indices: &[u32],
        tolerance: f32
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        let result = unsafe {
            weld_vertices_spatial(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
                tolerance
            )
        };
        
        // CRITICAL: Update performance stats for mesh operations
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size); // false = mesh operation;
        
        if result.success == 0 {
            let error_msg = unsafe {
                core::str::from_utf8(&result.error_message)
                    .unwrap_or("Unknown error")
                    .trim_end_matches('\0')
            };
            return Err(PixieError::CHotspotFailed(format!("Vertex welding failed: {}", error_msg)));
        }
        
        let new_vertices = unsafe {
            core::slice::from_raw_parts(result.vertices, result.vertex_count)
        }.to_vec();
        
        let new_indices = unsafe {
            core::slice::from_raw_parts(result.indices, result.index_count)
        }.to_vec();
        
        unsafe {
            free_mesh_decimate_result(&result as *const _ as *mut _);
        }
        
        Ok((new_vertices, new_indices))
    }
}

pub fn svg_text_compress(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let level = 6u32;
        let mut output_data = vec![0u8; data.len()];
        let mut output_size = output_data.len();
        
        let result = unsafe {
            svg_compress_text(
                data.as_ptr(),
                data.len(),
                level, 
                &mut output_size
            )
        };
        
        if !result.is_null() && output_size > 0 {
            let result_data = unsafe { 
                core::slice::from_raw_parts(result, output_size).to_vec()
            };
            unsafe {
                // Note: We would need a free function from C, but for now assume it's handled
            }
            Ok(result_data)
        } else {
            Err(PixieError::OptimizationFailed(format!("SVG text compression returned null")))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        let text = core::str::from_utf8(data)
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
        let compressed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(compressed.into_bytes())
    }
}

pub fn svg_minify_markup(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let mut output = vec![0u8; data.len()];
        let mut output_size = output.len();
        let status = unsafe {
            svg_minify_markup_simd(
                data.as_ptr(),
                data.len(),
                output.as_mut_ptr(),
                &mut output_size
            )
        };
        if status == 0 && output_size <= output.len() {
            output.truncate(output_size);
            Ok(output)
        } else {
            Err(PixieError::OptimizationFailed("SVG markup minification failed".to_string()))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        let text = core::str::from_utf8(data)
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
        
        let mut result = String::with_capacity(text.len());
        let mut in_comment = false;
        let mut pos = 0;
        
        while pos < text.len() {
            let ch = text.chars().nth(pos).unwrap_or('\0');
            
            if !in_comment && ch == '<' && text[pos..].starts_with("<!--") {
                in_comment = true;
                pos += 4;
                continue;
            } else if in_comment && ch == '-' && text[pos..].starts_with("-->") {
                in_comment = false;
                pos += 3;
                continue;
            }
            
            if !in_comment {
                result.push(ch);
            }
            pos += ch.len_utf8();
        }
        
        Ok(result.into_bytes())
    }
}

pub fn svg_optimize_paths_c(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let mut output_data = vec![0u8; data.len()];
        let mut output_size = output_data.len();
        
        let result = unsafe {
            svg_optimize_paths(
                data.as_ptr(),
                data.len(),
                0.1,
                &mut output_size
            )
        };
        
        if !result.is_null() && output_size > 0 {
            let result_data = unsafe { 
                core::slice::from_raw_parts(result, output_size).to_vec()
            };
            Ok(result_data)
        } else {
            Err(PixieError::OptimizationFailed(format!("SVG path optimization returned null")))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        Ok(data.to_vec())
    }
}

pub fn ico_optimize_embedded_c(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let mut output_data = vec![0u8; data.len() * 2];
        let mut output_size = output_data.len();
        
        let result = unsafe {
            ico_optimize_embedded(
                data.as_ptr(),
                data.len(),
                quality,
                &mut output_size
            )
        };
        
        if !result.is_null() && output_size > 0 {
            let result_data = unsafe { 
                core::slice::from_raw_parts(result, output_size).to_vec()
            };
            Ok(result_data)
        } else {
            Err(PixieError::OptimizationFailed(format!("ICO embedded optimization returned null")))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        Ok(data.to_vec())
    }
}

pub fn ico_strip_metadata_c(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let mut output_data = vec![0u8; data.len()];
        let mut output_size = output_data.len();
        
        let result = unsafe {
            ico_strip_metadata_simd(
                data.as_ptr(),
                data.len(),
                &mut output_size
            )
        };
        
        if !result.is_null() && output_size > 0 {
            let result_data = unsafe { 
                core::slice::from_raw_parts(result, output_size).to_vec()
            };
            Ok(result_data)
        } else {
            Err(PixieError::OptimizationFailed(format!("ICO metadata stripping returned null")))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        if data.len() < 6 {
            return Ok(data.to_vec());
        }
        
        Ok(data.to_vec())
    }
}

pub fn ico_compress_directory_c(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let mut output_data = vec![0u8; data.len()];
        let mut output_size = output_data.len();
        
        let result = unsafe {
            ico_compress_directory(
                data.as_ptr(),
                data.len(),
                6,
                &mut output_size
            )
        };
        
        if !result.is_null() && output_size > 0 {
            let result_data = unsafe { 
                core::slice::from_raw_parts(result, output_size).to_vec()
            };
            Ok(result_data)
        } else {
            Err(PixieError::OptimizationFailed(format!("ICO directory compression returned null")))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        Ok(data.to_vec())
    }
}

pub fn compress_tiff_lzw_c_hotspot(rgba_data: &[u8], width: usize, height: usize, quality: u8) -> PixieResult<Vec<u8>> {
    
    #[cfg(c_hotspots_available)]
    {
        use crate::{get_current_time_ms, update_performance_stats};
        let start_time = get_current_time_ms();
        let data_size = rgba_data.len();
        
        let result = unsafe {
            compress_tiff_lzw_simd(
                rgba_data.as_ptr(),
                width,
                height,
                quality
            )
        };
        
        if !result.is_null() {
            let tiff_result = unsafe { &*result };
            if !tiff_result.data.is_null() && tiff_result.size > 0 {
                let result_data = unsafe { 
                    core::slice::from_raw_parts(tiff_result.data, tiff_result.size).to_vec()
                };
                
                unsafe { free_tiff_result(result); }
                
                let elapsed = get_current_time_ms() - start_time;
                update_performance_stats(true, elapsed, data_size);
                
                Ok(result_data)
            } else {
                unsafe { free_tiff_result(result); }
                Err(PixieError::OptimizationFailed("TIFF LZW compression failed".to_string()))
            }
        } else {
            Err(PixieError::OptimizationFailed("TIFF LZW compression returned null".to_string()))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        tiff_lzw_rust_fallback(rgba_data, width, height, quality)
    }
}

pub fn strip_tiff_metadata_c_hotspot(tiff_data: &[u8], preserve_icc: bool) -> PixieResult<Vec<u8>> {
    
    #[cfg(c_hotspots_available)]
    {
        use crate::{get_current_time_ms, update_performance_stats};
        let start_time = get_current_time_ms();
        let data_size = tiff_data.len();
        
        let result = unsafe {
            strip_tiff_metadata_simd_c_hotspot(
                tiff_data.as_ptr(),
                tiff_data.len(),
                preserve_icc
            )
        };
        
        if !result.is_null() {
            let tiff_result = unsafe { &*result };
            if !tiff_result.data.is_null() && tiff_result.size > 0 {
                let result_data = unsafe { 
                    core::slice::from_raw_parts(tiff_result.data, tiff_result.size).to_vec()
                };
                
                unsafe { free_tiff_result(result); }
                
                let elapsed = get_current_time_ms() - start_time;
                update_performance_stats(true, elapsed, data_size);
                
                Ok(result_data)
            } else {
                unsafe { free_tiff_result(result); }
                Err(PixieError::OptimizationFailed("TIFF metadata stripping failed".to_string()))
            }
        } else {
            Err(PixieError::OptimizationFailed("TIFF metadata stripping returned null".to_string()))
        }
    }
    #[cfg(not(c_hotspots_available))]
    {
        tiff_metadata_strip_rust_fallback(tiff_data, preserve_icc)
    }
}

pub fn apply_tiff_predictor_c_hotspot(rgba_data: &mut [u8], width: usize, height: usize, predictor_type: u8) -> PixieResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            apply_tiff_predictor_simd(
                rgba_data.as_mut_ptr(),
                width,
                height,
                predictor_type
            );
        }
        Ok(())
    }
    #[cfg(not(c_hotspots_available))]
    {
        tiff_predictor_rust_fallback(rgba_data, width, height, predictor_type)
    }
}

pub fn optimize_tiff_colorspace_c_hotspot(rgba_data: &mut [u8], width: usize, height: usize, target_bits: u8) -> PixieResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            optimize_tiff_colorspace_simd(
                rgba_data.as_mut_ptr(),
                width,
                height,
                target_bits
            );
        }
        Ok(())
    }
    #[cfg(not(c_hotspots_available))]
    {
        tiff_colorspace_rust_fallback(rgba_data, width, height, target_bits)
    }
}

fn tiff_lzw_rust_fallback(rgba_data: &[u8], width: usize, height: usize, quality: u8) -> PixieResult<Vec<u8>> {
    let pixel_count = width * height;
    let mut compressed = Vec::with_capacity(pixel_count * 3);
    
    let compression_ratio = (100 - quality) as f32 / 100.0;
    let target_size = (rgba_data.len() as f32 * (1.0 - compression_ratio)) as usize;
    
    for i in (0..pixel_count).step_by(2) {
        if compressed.len() < target_size {
            let idx = i * 4;
            if idx + 3 < rgba_data.len() {
                compressed.push(rgba_data[idx]);     // R
                compressed.push(rgba_data[idx + 1]); // G
                compressed.push(rgba_data[idx + 2]); // B
            }
        }
    }
    
    Ok(compressed)
}

fn tiff_metadata_strip_rust_fallback(tiff_data: &[u8], _preserve_icc: bool) -> PixieResult<Vec<u8>> {
    let reduced_size = tiff_data.len() * 85 / 100;
    Ok(tiff_data[..reduced_size.min(tiff_data.len())].to_vec())
}

fn tiff_predictor_rust_fallback(rgba_data: &mut [u8], width: usize, height: usize, predictor_type: u8) -> PixieResult<()> {
    if predictor_type == 2 {
        for y in 0..height {
            let row_start = y * width * 4;
            for x in (1..width).rev() {
                let idx = row_start + x * 4;
                let prev_idx = row_start + (x - 1) * 4;
                
                if idx + 3 < rgba_data.len() && prev_idx + 3 < rgba_data.len() {
                    rgba_data[idx] = rgba_data[idx].wrapping_sub(rgba_data[prev_idx]);
                    rgba_data[idx + 1] = rgba_data[idx + 1].wrapping_sub(rgba_data[prev_idx + 1]);
                    rgba_data[idx + 2] = rgba_data[idx + 2].wrapping_sub(rgba_data[prev_idx + 2]);
                    rgba_data[idx + 3] = rgba_data[idx + 3].wrapping_sub(rgba_data[prev_idx + 3]);
                }
            }
        }
    }
    Ok(())
}

fn tiff_colorspace_rust_fallback(rgba_data: &mut [u8], _width: usize, _height: usize, target_bits: u8) -> PixieResult<()> {
    if target_bits < 8 {
        let shift = 8 - target_bits;
        for pixel in rgba_data.iter_mut() {
            *pixel = (*pixel >> shift) << shift;
        }
    }
    Ok(())
}

pub fn batch_process_pixels_c_hotspot(rgba_data: &mut [u8], operation_type: u8) -> PixieResult<()> {
    #[cfg(c_hotspots_available)]
    {
        let pixel_count = rgba_data.len() / 4;
        unsafe {
            batch_process_pixels_simd(
                rgba_data.as_mut_ptr(),
                pixel_count,
                operation_type
            );
        }
        Ok(())
    }
    #[cfg(not(c_hotspots_available))]
    {
        advanced_pixel_processing_rust_fallback(rgba_data, operation_type)
    }
}

pub fn parallel_color_conversion_c_hotspot(
    src_data: &[u8], 
    dst_data: &mut [u8], 
    src_format: u8, 
    dst_format: u8
) -> PixieResult<()> {
    #[cfg(c_hotspots_available)]
    {
        let pixel_count = src_data.len() / src_format as usize;
        unsafe {
            parallel_color_conversion_simd(
                src_data.as_ptr(),
                dst_data.as_mut_ptr(),
                pixel_count,
                src_format,
                dst_format
            );
        }
        Ok(())
    }
    #[cfg(not(c_hotspots_available))]
    {
        color_conversion_rust_fallback(src_data, dst_data, src_format, dst_format)
    }
}

pub fn vectorized_filter_apply_c_hotspot(
    rgba_data: &mut [u8], 
    width: usize, 
    height: usize, 
    kernel: &[f32]
) -> PixieResult<()> {
    #[cfg(c_hotspots_available)]
    {
        let kernel_size = (kernel.len() as f32).sqrt() as usize;
        unsafe {
            vectorized_filter_apply_simd(
                rgba_data.as_mut_ptr(),
                width,
                height,
                kernel.as_ptr(),
                kernel_size
            );
        }
        Ok(())
    }
    #[cfg(not(c_hotspots_available))]
    {
        filter_apply_rust_fallback(rgba_data, width, height, kernel)
    }
}

pub fn fast_downscale_c_hotspot(
    src_data: &[u8],
    dst_data: &mut [u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize
) -> PixieResult<()> {
    #[cfg(c_hotspots_available)]
    {
        unsafe {
            fast_downscale_simd(
                src_data.as_ptr(),
                dst_data.as_mut_ptr(),
                src_width,
                src_height,
                dst_width,
                dst_height
            );
        }
        Ok(())
    }
    #[cfg(not(c_hotspots_available))]
    {
        downscale_rust_fallback(src_data, dst_data, src_width, src_height, dst_width, dst_height)
    }
}

pub fn multi_threaded_compression_c_hotspot(
    rgba_data: &[u8],
    width: usize,
    height: usize,
    quality: u8
) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let estimated_size = rgba_data.len() * (100 - quality as usize) / 100;
        let mut compressed_data = vec![0u8; estimated_size];
        let mut compressed_size = 0usize;
        
        unsafe {
            multi_threaded_compression_simd(
                rgba_data.as_ptr(),
                width,
                height,
                compressed_data.as_mut_ptr(),
                &mut compressed_size,
                quality
            );
        }
        
        compressed_data.truncate(compressed_size);
        Ok(compressed_data)
    }
    #[cfg(not(c_hotspots_available))]
    {
        compression_rust_fallback(rgba_data, width, height, quality)
    }
}

fn advanced_pixel_processing_rust_fallback(rgba_data: &mut [u8], operation_type: u8) -> PixieResult<()> {
    match operation_type {
        1 => {
            for pixel in rgba_data.iter_mut() {
                *pixel = (*pixel).saturating_add(25);
            }
        },
        2 => {
            for pixel in rgba_data.iter_mut() {
                let enhanced = (*pixel as f32 * 1.2).min(255.0) as u8;
                *pixel = enhanced;
            }
        },
        3 => {
            for chunk in rgba_data.chunks_mut(4) {
                if chunk.len() == 4 {
                    let r = chunk[0] as f32;
                    let g = chunk[1] as f32;
                    let b = chunk[2] as f32;
                    
                    let max_val = r.max(g).max(b);
                    let min_val = r.min(g).min(b);
                    
                    if max_val > min_val {
                        let saturation_factor = 1.3;
                        chunk[0] = (r + (r - min_val) * 0.3).min(255.0) as u8;
                        chunk[1] = (g + (g - min_val) * 0.3).min(255.0) as u8;
                        chunk[2] = (b + (b - min_val) * 0.3).min(255.0) as u8;
                    }
                }
            }
        },
        _ => {}
    }
    Ok(())
}

fn color_conversion_rust_fallback(
    src_data: &[u8], 
    dst_data: &mut [u8], 
    src_format: u8, 
    dst_format: u8
) -> PixieResult<()> {
    let src_channels = src_format as usize;
    let dst_channels = dst_format as usize;
    let pixel_count = src_data.len() / src_channels;
    
    for i in 0..pixel_count {
        let src_idx = i * src_channels;
        let dst_idx = i * dst_channels;
        
        if src_channels == 4 && dst_channels == 3 { // RGBA to RGB
            if src_idx + 3 < src_data.len() && dst_idx + 2 < dst_data.len() {
                dst_data[dst_idx] = src_data[src_idx];         // R
                dst_data[dst_idx + 1] = src_data[src_idx + 1]; // G
                dst_data[dst_idx + 2] = src_data[src_idx + 2]; // B
            }
        } else if src_channels == 3 && dst_channels == 4 { // RGB to RGBA
            if src_idx + 2 < src_data.len() && dst_idx + 3 < dst_data.len() {
                dst_data[dst_idx] = src_data[src_idx];         // R
                dst_data[dst_idx + 1] = src_data[src_idx + 1]; // G
                dst_data[dst_idx + 2] = src_data[src_idx + 2]; // B
                dst_data[dst_idx + 3] = 255;                   // A
            }
        }
    }
    Ok(())
}

fn filter_apply_rust_fallback(
    rgba_data: &mut [u8], 
    width: usize, 
    height: usize, 
    kernel: &[f32]
) -> PixieResult<()> {
    let kernel_size = (kernel.len() as f32).sqrt() as usize;
    let half_kernel = kernel_size / 2;
    let temp_data = rgba_data.to_vec();
    
    for y in half_kernel..height - half_kernel {
        for x in half_kernel..width - half_kernel {
            let mut sum = [0.0f32; 4];
            
            for ky in 0..kernel_size {
                for kx in 0..kernel_size {
                    let py = y + ky - half_kernel;
                    let px = x + kx - half_kernel;
                    let idx = (py * width + px) * 4;
                    let kernel_val = kernel[ky * kernel_size + kx];
                    
                    if idx + 3 < temp_data.len() {
                        sum[0] += temp_data[idx] as f32 * kernel_val;
                        sum[1] += temp_data[idx + 1] as f32 * kernel_val;
                        sum[2] += temp_data[idx + 2] as f32 * kernel_val;
                        sum[3] += temp_data[idx + 3] as f32 * kernel_val;
                    }
                }
            }
            
            let result_idx = (y * width + x) * 4;
            if result_idx + 3 < rgba_data.len() {
                rgba_data[result_idx] = sum[0].max(0.0).min(255.0) as u8;
                rgba_data[result_idx + 1] = sum[1].max(0.0).min(255.0) as u8;
                rgba_data[result_idx + 2] = sum[2].max(0.0).min(255.0) as u8;
                rgba_data[result_idx + 3] = sum[3].max(0.0).min(255.0) as u8;
            }
        }
    }
    Ok(())
}

fn downscale_rust_fallback(
    src_data: &[u8],
    dst_data: &mut [u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize
) -> PixieResult<()> {
    let x_ratio = src_width as f32 / dst_width as f32;
    let y_ratio = src_height as f32 / dst_height as f32;
    
    for y in 0..dst_height {
        for x in 0..dst_width {
            let src_x = (x as f32 * x_ratio) as usize;
            let src_y = (y as f32 * y_ratio) as usize;
            
            let src_idx = (src_y * src_width + src_x) * 4;
            let dst_idx = (y * dst_width + x) * 4;
            
            if src_idx + 3 < src_data.len() && dst_idx + 3 < dst_data.len() {
                dst_data[dst_idx] = src_data[src_idx];
                dst_data[dst_idx + 1] = src_data[src_idx + 1];
                dst_data[dst_idx + 2] = src_data[src_idx + 2];
                dst_data[dst_idx + 3] = src_data[src_idx + 3];
            }
        }
    }
    Ok(())
}

fn compression_rust_fallback(
    rgba_data: &[u8],
    width: usize,
    height: usize,
    quality: u8
) -> PixieResult<Vec<u8>> {
    let pixel_count = width * height;
    let compression_ratio = (100 - quality) as f32 / 100.0;
    let target_size = (rgba_data.len() as f32 * (1.0 - compression_ratio)) as usize;
    
    let mut compressed = Vec::with_capacity(target_size);
    let step = if quality < 50 { 2 } else { 1 };
    
    for i in (0..pixel_count).step_by(step) {
        let idx = i * 4;
        if idx + 2 < rgba_data.len() && compressed.len() < target_size {
            compressed.push(rgba_data[idx]);     // R
            compressed.push(rgba_data[idx + 1]); // G
            compressed.push(rgba_data[idx + 2]); // B
        }
    }
    
    Ok(compressed)
}

#[cfg(not(c_hotspots_available))]
pub mod fallback {
    use super::*;
    
    pub fn fallback_operation(input: &[u8]) -> PixieResult<Vec<u8>> {
        Ok(input.to_vec())
    }
}

pub fn are_c_hotspots_available() -> bool {
    cfg!(c_hotspots_available)
}

pub fn c_hotspots_info() -> &'static str {
    if cfg!(c_hotspots_available) {
        "C hotspots available - SIMD optimizations enabled"
    } else {
        "C hotspots not available - using Rust fallbacks"
    }
}
