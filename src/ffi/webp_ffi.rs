use crate::types::{OptConfig, OptResult, OptError};

#[cfg(c_hotspots_available)]
use crate::ffi::c_bindings::{WebPOptConfig, WebPOptResult, webp_optimize_c, webp_opt_result_free, webp_has_alpha_channel, webp_get_info};

// WASM console logging
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
macro_rules! wasm_log {
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string()) })
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! wasm_log {
    ($($t:tt)*) => {}
}

/// Optimize WebP using C library with SIMD acceleration
pub fn optimize_webp_with_c(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        wasm_log!("🔧 Using C WebP optimization with SIMD acceleration");
        
        // Check if WebP has alpha channel before optimization
        let has_alpha = unsafe { webp_has_alpha_channel(data.as_ptr(), data.len()) };
        wasm_log!("🔍 WebP alpha channel detection: {}", if has_alpha != 0 { "HAS ALPHA" } else { "NO ALPHA" });
        
        let c_config = WebPOptConfig {
            quality: config.quality.unwrap_or(85) as i32,
            method: if config.quality.unwrap_or(85) > 90 { 6 } else { 4 }, // Better method for high quality
            use_lossless: if config.quality.unwrap_or(85) >= 95 { 1 } else { 0 },
            alpha_quality: if has_alpha != 0 { config.quality.unwrap_or(85) as i32 } else { 100 },
            preserve_alpha: if has_alpha != 0 { 1 } else { 0 },
            optimize_filters: 1, // Always optimize filters
            use_sharp_yuv: if config.quality.unwrap_or(85) > 80 { 1 } else { 0 },
        };
        
        wasm_log!("📊 C WebP config: quality={}, method={}, lossless={}, alpha_quality={}, preserve_alpha={}", 
                  c_config.quality, c_config.method, c_config.use_lossless, c_config.alpha_quality, c_config.preserve_alpha);
        
        let mut result = unsafe {
            webp_optimize_c(data.as_ptr(), data.len(), &c_config as *const WebPOptConfig)
        };
        
        if result.error_code == 0 && !result.data.is_null() && result.size > 0 {
            // Success - convert C result to Rust Vec
            let optimized_data = unsafe {
                std::slice::from_raw_parts(result.data, result.size).to_vec()
            };
            
            wasm_log!("✅ C WebP optimization successful: {} -> {} bytes ({:.1}% reduction)", 
                      data.len(), optimized_data.len(),
                      (1.0 - optimized_data.len() as f64 / data.len() as f64) * 100.0);
            
            // Free the result memory
            unsafe {
                webp_opt_result_free(&mut result as *mut WebPOptResult);
            }
            
            return Ok(optimized_data);
        } else {
            // Handle error case
            let error_msg = if !result.error_message.is_null() {
                unsafe {
                    let c_str = std::ffi::CStr::from_ptr(result.error_message);
                    c_str.to_string_lossy().to_string()
                }
            } else {
                format!("C WebP optimization failed with error code: {}", result.error_code)
            };
            wasm_log!("⚠️ C WebP optimization failed: {}", error_msg);
            
            // Free the result memory even on error
            unsafe {
                webp_opt_result_free(&mut result as *mut WebPOptResult);
            }
        }
    }
    
    #[cfg(not(c_hotspots_available))]
    wasm_log!("⚠️ C hotspots not available, using Rust fallback");
    
    // Fallback to Rust implementation if C optimization fails or not available
    optimize_webp_fallback(data, config)
}

/// Get WebP image information using C library
pub fn get_webp_info_c(data: &[u8]) -> Option<(i32, i32, bool)> {
    #[cfg(c_hotspots_available)]
    {
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        let mut has_alpha: i32 = 0;
        
        let success = unsafe {
            webp_get_info(data.as_ptr(), data.len(), &mut width, &mut height, &mut has_alpha)
        };
        
        if success != 0 {
            return Some((width, height, has_alpha != 0));
        }
    }
    
    None
}

/// Fallback WebP optimization using pure Rust implementation
fn optimize_webp_fallback(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    wasm_log!("🔧 Using Rust fallback for WebP optimization");
    
    // Use image crate for basic WebP handling
    use image::{ImageFormat, DynamicImage};
    use std::io::Cursor;
    
    // Decode the WebP image
    let img = image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| OptError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to decode WebP: {}", e))))?;
    
    // Convert to RGBA for processing
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    wasm_log!("📏 WebP fallback: {}x{} pixels", width, height);
    
    // Apply quality reduction if needed
    let processed_img = if let Some(quality) = config.quality {
        if quality < 95 {
            // Simple quality reduction by downsampling and upsampling
            let scale_factor = (quality as f32 / 100.0).sqrt();
            let new_width = ((width as f32 * scale_factor) as u32).max(1);
            let new_height = ((height as f32 * scale_factor) as u32).max(1);
            
            wasm_log!("🔧 Quality reduction: scaling to {}x{}", new_width, new_height);
            
            let scaled = DynamicImage::ImageRgba8(rgba)
                .resize(new_width, new_height, image::imageops::FilterType::Triangle)
                .resize(width, height, image::imageops::FilterType::Triangle);
            scaled
        } else {
            DynamicImage::ImageRgba8(rgba)
        }
    } else {
        DynamicImage::ImageRgba8(rgba)
    };
    
    // Encode back to WebP
    let mut result = Vec::new();
    let mut cursor = Cursor::new(&mut result);
    
    processed_img.write_to(&mut cursor, ImageFormat::WebP)
        .map_err(|e| OptError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to encode WebP: {}", e))))?;
    
    let reduction = 1.0 - (result.len() as f64 / data.len() as f64);
    wasm_log!("✅ WebP fallback complete: {:.1}% reduction", reduction * 100.0);
    
    Ok(result)
}
