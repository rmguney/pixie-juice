//! WASM-compatible WebP processing using C optimizations and image crate fallback

use crate::types::{OptConfig, OptError, OptResult};
use image::{ImageFormat, DynamicImage, GenericImageView};
use std::io::Cursor;

#[cfg(c_hotspots_available)]
use crate::ffi::webp_ffi::optimize_webp_with_c;

// Use webp crate for better quality control when available
#[cfg(feature = "webp")]
use webp;

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
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! wasm_log {
    ($($t:tt)*) => {}
}

/// Convert to WebP format using C optimization with SIMD acceleration
/// Falls back to Rust implementation if C optimization is not available
pub fn optimize_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    wasm_log!("🔍 WebP WASM optimization starting: {} bytes", data.len());
    wasm_log!("🔧 Config: quality={:?}, target_reduction={:?}, reduce_colors={:?}", 
              config.quality, config.target_reduction, config.reduce_colors);

    // Try C optimization first
    #[cfg(c_hotspots_available)]
    {
        match optimize_webp_with_c(data, config) {
            Ok(optimized_data) => {
                let c_reduction = 1.0 - (optimized_data.len() as f64 / data.len() as f64);
                if c_reduction > 0.05 { // At least 5% improvement
                    wasm_log!("✅ C WebP optimization successful: {:.1}% reduction", c_reduction * 100.0);
                    return Ok(optimized_data);
                } else {
                    wasm_log!("🔧 C WebP optimization didn't improve enough, trying Rust fallback");
                }
            }
            Err(_) => {
                wasm_log!("⚠️ C WebP optimization failed, using Rust fallback");
            }
        }
    }

    // Continue with Rust fallback implementation
    optimize_webp_rust_fallback(data, config)
}

/// Rust fallback WebP optimization implementation
fn optimize_webp_rust_fallback(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load the image
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load image for WebP processing: {}", e)))?;
    
    let mut best_result = data.to_vec();
    let mut best_reduction = 0.0;
    
    // More aggressive optimization trigger for WebP
    let should_optimize = config.quality.unwrap_or(85) < 95 || 
                         config.reduce_colors.unwrap_or(true) ||
                         config.target_reduction.unwrap_or(0.0) > 0.05 ||
                         data.len() > 1000; // Try for files over 1KB
    
    wasm_log!("🔧 Should optimize: {} (size: {} bytes)", should_optimize, data.len());
    
    // If input is already WebP, try to optimize it
    if is_webp_format(data) {
        wasm_log!("🔧 Input is WebP format, attempting re-optimization...");
        if should_optimize {
            // Use a single optimized quality setting to avoid infinite loops
            let target_quality = config.quality.unwrap_or(75) as f32;
            wasm_log!("🔧 WebP re-encoding with quality {}...", target_quality);
            
            // Apply format optimizations for WebP re-encoding
            let optimized_img = if !config.lossless.unwrap_or(false) {
                // Try converting RGBA to RGB for better compression if alpha preservation is disabled
                match &img {
                    DynamicImage::ImageRgba8(rgba_img) => {
                        let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                        let preserve_alpha = config.preserve_alpha.unwrap_or(true);
                        wasm_log!("🔧 RGBA WebP detected, has_transparency: {}, preserve_alpha: {}", has_transparency, preserve_alpha);
                        if !has_transparency && !preserve_alpha {
                            wasm_log!("🔄 Converting RGBA to RGB for better compression (alpha preservation disabled)");
                            DynamicImage::ImageRgb8(img.to_rgb8())
                        } else {
                            img.clone()
                        }
                    }
                    _ => {
                        wasm_log!("ℹ️ Non-RGBA WebP: {:?}", img.color());
                        img.clone()
                    }
                }
            } else {
                img.clone()
            };
            
            // Single WebP encoding attempt to avoid infinite loops
            match encode_webp_with_quality(&optimized_img, target_quality) {
                Ok(encoded_data) => {
                    if encoded_data.len() < best_result.len() {
                        let reduction = 1.0 - (encoded_data.len() as f64 / data.len() as f64);
                        best_result = encoded_data;
                        best_reduction = reduction;
                        wasm_log!("📊 WebP re-encoded: {:.1}% reduction ({} -> {} bytes)", 
                                 reduction * 100.0, data.len(), best_result.len());
                    } else {
                        wasm_log!("⚠️ WebP re-encoding didn't improve size: {} -> {} bytes", 
                                 data.len(), encoded_data.len());
                    }
                },
                Err(e) => {
                    wasm_log!("❌ WebP re-encoding failed: {}", e);
                }
            }
        } else {
            wasm_log!("⚠️ Skipping WebP optimization due to config");
        }
        
        // If WebP re-optimization didn't work well, try PNG fallback
        if best_reduction < 0.1 && should_optimize {
            wasm_log!("🔧 WebP optimization minimal, trying PNG fallback...");
            match convert_webp_to_png(&img) {
                Ok(png_data) => {
                    if png_data.len() < best_result.len() {
                        let reduction = 1.0 - (png_data.len() as f64 / data.len() as f64);
                        best_result = png_data;
                        best_reduction = reduction;
                        wasm_log!("📊 WebP→PNG conversion: {:.1}% reduction ({} -> {} bytes)", 
                                 reduction * 100.0, data.len(), best_result.len());
                    }
                },
                Err(e) => {
                    wasm_log!("❌ WebP→PNG conversion failed: {}", e);
                }
            }
        }
    } else {
        // For non-WebP inputs, try multiple conversion strategies
        wasm_log!("🔧 Input is not WebP, trying format conversion strategies...");
        let conversion_strategies = vec![
            // Strategy 1: Direct WebP encoding with quality optimization
            ("WebP_high", ImageFormat::WebP, 65.0),
            ("WebP_medium", ImageFormat::WebP, 80.0),
            // Strategy 2: PNG fallback (often better compression than original)
            ("PNG", ImageFormat::Png, 0.0), // PNG doesn't use quality
        ];
        
        for (strategy_name, format, quality) in conversion_strategies {
            wasm_log!("🔧 Trying {} conversion strategy...", strategy_name);
            let mut output = Vec::new();
            let mut cursor = Cursor::new(&mut output);
            
            // Apply optimizations before encoding
            let optimized_img = if !config.lossless.unwrap_or(false) && should_optimize {
                match &img {
                    DynamicImage::ImageRgba8(rgba_img) => {
                        let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                        let preserve_alpha = config.preserve_alpha.unwrap_or(true);
                        wasm_log!("🔧 RGBA image for {}, has_transparency: {}, preserve_alpha: {}", strategy_name, has_transparency, preserve_alpha);
                        if !has_transparency && !preserve_alpha {
                            wasm_log!("🔄 Converting RGBA to RGB for {} (alpha preservation disabled)", strategy_name);
                            DynamicImage::ImageRgb8(img.to_rgb8())
                        } else {
                            img.clone()
                        }
                    }
                    _ => {
                        wasm_log!("ℹ️ Non-RGBA image for {}: {:?}", strategy_name, img.color());
                        img.clone()
                    }
                }
            } else {
                img.clone()
            };
            
            let result = if format == ImageFormat::WebP && quality > 0.0 {
                encode_webp_with_quality(&optimized_img, quality)
            } else {
                optimized_img.write_to(&mut cursor, format)
                    .map(|_| output)
                    .map_err(|e| e.to_string())
            };
            
            match result {
                Ok(encoded_data) => {
                    if encoded_data.len() < best_result.len() {
                        let reduction = 1.0 - (encoded_data.len() as f64 / data.len() as f64);
                        best_result = encoded_data;
                        best_reduction = reduction;
                        wasm_log!("📊 {} conversion: {:.1}% reduction ({} -> {} bytes)", 
                                 strategy_name, reduction * 100.0, data.len(), best_result.len());
                    } else {
                        wasm_log!("⚠️ {} conversion didn't improve size: {} -> {} bytes", 
                                 strategy_name, data.len(), encoded_data.len());
                    }
                },
                Err(e) => {
                    wasm_log!("❌ {} conversion failed: {}", strategy_name, e);
                }
            }
        }
    }
    
    // Return best result
    if best_reduction > 0.005 {  // Accept 0.5% improvement
        wasm_log!("🎯 WebP WASM optimization successful: {} bytes -> {} bytes ({:.1}% reduction)", 
                  data.len(), best_result.len(), best_reduction * 100.0);
        Ok(best_result)
    } else {
        wasm_log!("ℹ️ WebP WASM optimization: no improvement, keeping original");
        Ok(data.to_vec())
    }
}

/// Encode image as WebP with specific quality using image crate fallback
fn encode_webp_with_quality(img: &DynamicImage, quality: f32) -> Result<Vec<u8>, String> {
    // Use webp crate for better quality control if available
    #[cfg(feature = "webp")]
    {
        let rgba_img = img.to_rgba8();
        let width = rgba_img.width();
        let height = rgba_img.height();
        let pixels = rgba_img.as_raw();
        
        // Use webp crate for quality encoding
        let encoder = webp::Encoder::from_rgba(pixels, width, height);
        let encoded = if quality >= 99.0 {
            encoder.encode_lossless()
        } else {
            encoder.encode(quality)
        };
        
        Ok(encoded.to_vec())
    }
    
    #[cfg(not(feature = "webp"))]
    {
        // Advanced fallback for WASM - optimize before encoding
        wasm_log!("🔧 Using image crate fallback with quality: {}", quality);
        
        // Apply aggressive pre-processing for better compression
        let optimized_img = if quality < 85.0 {
            // For lower quality, reduce image dimensions slightly
            let (width, height) = img.dimensions();
            let new_width = ((width as f32 * 0.95) as u32).max(1);
            let new_height = ((height as f32 * 0.95) as u32).max(1);
            
            if new_width < width || new_height < height {
                wasm_log!("🔧 Resizing for quality {}: {}x{} -> {}x{}", quality, width, height, new_width, new_height);
                img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
            } else {
                img.clone()
            }
        } else {
            img.clone()
        };
        
        // Convert to optimal format for WebP encoding
        let processed_img = match &optimized_img {
            DynamicImage::ImageRgba8(rgba_img) => {
                let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                // Always preserve alpha in this fallback function for safety
                if has_transparency {
                    optimized_img
                } else if quality < 90.0 {
                    wasm_log!("🔄 Converting RGBA to RGB for WebP (no transparency detected)");
                    DynamicImage::ImageRgb8(optimized_img.to_rgb8())
                } else {
                    optimized_img
                }
            }
            _ => optimized_img
        };
        
        // Encode to WebP using image crate
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        
        match processed_img.write_to(&mut cursor, ImageFormat::WebP) {
            Ok(_) => {
                wasm_log!("✅ WebP encoding successful: {} bytes", output.len());
                Ok(output)
            },
            Err(e) => Err(format!("WebP encoding failed: {}", e))
        }
    }
}

/// Convert WebP to PNG for better compatibility
fn convert_webp_to_png(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    
    match img.write_to(&mut cursor, ImageFormat::Png) {
        Ok(_) => Ok(output),
        Err(e) => Err(format!("PNG conversion failed: {}", e))
    }
}

/// Check if data is WebP format
fn is_webp_format(data: &[u8]) -> bool {
    data.len() >= 12 && 
    &data[0..4] == b"RIFF" && 
    &data[8..12] == b"WEBP"
}

/// Convert from WebP format using the image crate
pub fn convert_from_webp(data: &[u8], _config: &OptConfig) -> OptResult<DynamicImage> {
    image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load WebP: {}", e)))
}

/// Convert various formats to WebP
pub fn convert_to_webp(data: &[u8], _config: &OptConfig) -> OptResult<Vec<u8>> {
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load image: {}", e)))?;
    
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    
    // Try WebP encoding, fallback to PNG
    match img.write_to(&mut cursor, ImageFormat::WebP) {
        Ok(_) => Ok(output),
        Err(_) => {
            // WebP not supported, use PNG instead
            output.clear();
            cursor = Cursor::new(&mut output);
            img.write_to(&mut cursor, ImageFormat::Png)
                .map_err(|e| OptError::ProcessingError(format!("Failed to encode as PNG: {}", e)))?;
            Ok(output)
        }
    }
}

/// Basic WebP validation
pub fn validate_webp(data: &[u8]) -> OptResult<()> {
    image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| OptError::ProcessingError(format!("Invalid WebP: {}", e)))?;
    Ok(())
}
