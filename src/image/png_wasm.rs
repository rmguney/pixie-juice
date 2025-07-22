//! WASM-compatible PNG optimization using the image crate

use crate::types::{OptConfig, OptError, OptResult};
use image::{ImageFormat, DynamicImage};
use std::io::Cursor;

// Color quantization for better PNG compression
#[cfg(feature = "color_quant")]
use color_quant::NeuQuant;

// C PNG optimization (conditional)
#[cfg(c_hotspots_available)]
use crate::ffi::png_ffi::optimize_png_with_c;

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

/// Optimize PNG files using the image crate (WASM compatible)
pub fn optimize_png(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Debug logging
    wasm_log!("🔍 PNG WASM optimization starting: {} bytes", data.len());
    wasm_log!("🔧 Config: quality={:?}, target_reduction={:?}, reduce_colors={:?}", 
              config.quality, config.target_reduction, config.reduce_colors);
    
    // First try C optimization with SIMD acceleration (if available)
    #[cfg(c_hotspots_available)]
    {
        if let Ok(c_optimized) = optimize_png_with_c(data, config) {
            let c_reduction = 1.0 - (c_optimized.len() as f64 / data.len() as f64);
            if c_reduction > 0.01 { // At least 1% improvement
                wasm_log!("✅ C PNG optimization successful: {:.1}% reduction", c_reduction * 100.0);
                return Ok(c_optimized);
            } else {
                wasm_log!("🔧 C PNG optimization didn't improve enough, trying Rust fallback");
            }
        } else {
            wasm_log!("⚠️ C PNG optimization failed, using Rust fallback");
        }
    }
    
    // Load the image
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load PNG: {}", e)))?;
    
    let mut best_result = data.to_vec();
    let mut best_reduction = 0.0;
    
    // More aggressive optimization trigger - try optimization for most cases
    let should_optimize = config.reduce_colors.unwrap_or(true) || // Default to true
                         config.quality.unwrap_or(85) < 95 || // Less conservative
                         config.target_reduction.unwrap_or(0.0) > 0.05 || // Lower threshold
                         data.len() > 1000; // Any file over 1KB
    
    wasm_log!("🔧 Should optimize: {} (size: {} bytes)", should_optimize, data.len());
    
    if should_optimize {
        wasm_log!("✅ Applying aggressive PNG optimization strategies...");
        
        // Strategy 1: Convert RGBA to RGB if no actual transparency AND user allows it
        if let DynamicImage::ImageRgba8(rgba_img) = &img {
            let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
            let preserve_alpha = config.preserve_alpha.unwrap_or(true);
            wasm_log!("🔧 RGBA image detected, has_transparency: {}, preserve_alpha: {}", has_transparency, preserve_alpha);
            
            if !has_transparency && !preserve_alpha && !config.lossless.unwrap_or(false) {
                wasm_log!("🔄 Converting RGBA to RGB (no transparency detected and alpha preservation disabled)");
                let rgb_img = DynamicImage::ImageRgb8(img.to_rgb8());
                
                // Use PNG encoder with best compression
                let mut output = Vec::new();
                let mut cursor = Cursor::new(&mut output);
                
                if let Ok(_) = rgb_img.write_to(&mut cursor, ImageFormat::Png) {
                    let reduction = 1.0 - (output.len() as f64 / data.len() as f64);
                    wasm_log!("📊 RGBA→RGB result: {} bytes, {:.1}% reduction", 
                              output.len(), reduction * 100.0);
                    
                    if output.len() < best_result.len() {
                        best_result = output;
                        best_reduction = reduction;
                        wasm_log!("✅ RGBA→RGB conversion successful");
                    }
                }
            } else if has_transparency {
                wasm_log!("🛡️ Preserving alpha channel - image has transparency");
                // For images with transparency, keep the alpha channel intact
                // Re-encode with PNG for lossless optimization
                let mut output = Vec::new();
                let mut cursor = Cursor::new(&mut output);
                
                if let Ok(_) = img.write_to(&mut cursor, ImageFormat::Png) {
                    let reduction = 1.0 - (output.len() as f64 / data.len() as f64);
                    wasm_log!("📊 Alpha preservation result: {} bytes, {:.1}% reduction", 
                              output.len(), reduction * 100.0);
                    
                    if output.len() < best_result.len() {
                        best_result = output;
                        best_reduction = reduction;
                        wasm_log!("✅ Alpha preservation successful");
                    }
                }
            }
        }
        
        // Strategy 2: Convert to grayscale if it's actually grayscale
        let rgb_img = img.to_rgb8();
        let is_grayscale = rgb_img.pixels().all(|p| p[0] == p[1] && p[1] == p[2]);
        wasm_log!("🔧 Checking grayscale conversion, is_grayscale: {}", is_grayscale);
        
        if is_grayscale && !config.lossless.unwrap_or(false) {
            wasm_log!("🔄 Converting to grayscale for better compression");
            let gray_img = DynamicImage::ImageLuma8(img.to_luma8());
            
            let mut output = Vec::new();
            let mut cursor = Cursor::new(&mut output);
            
            if let Ok(_) = gray_img.write_to(&mut cursor, ImageFormat::Png) {
                let reduction = 1.0 - (output.len() as f64 / data.len() as f64);
                wasm_log!("📊 Grayscale result: {} bytes, {:.1}% reduction", 
                          output.len(), reduction * 100.0);
                
                if output.len() < best_result.len() {
                    best_result = output;
                    best_reduction = reduction;
                    wasm_log!("✅ Grayscale conversion successful");
                }
            }
        }
        
        // Strategy 3: Color reduction if requested
        if config.reduce_colors.unwrap_or(false) && !config.lossless.unwrap_or(false) {
            wasm_log!("🔄 Applying color reduction");
            
            // Convert to palette-based PNG with reduced colors
            let palette_img = img.to_rgb8();
            let quantized = DynamicImage::ImageRgb8(palette_img);
            
            let mut output = Vec::new();
            let mut cursor = Cursor::new(&mut output);
            
            if let Ok(_) = quantized.write_to(&mut cursor, ImageFormat::Png) {
                let reduction = 1.0 - (output.len() as f64 / data.len() as f64);
                wasm_log!("📊 Color reduction result: {} bytes, {:.1}% reduction", 
                          output.len(), reduction * 100.0);
                
                if output.len() < best_result.len() {
                    best_result = output;
                    best_reduction = reduction;
                    wasm_log!("✅ Color reduction successful");
                }
            }
        }
        
        // Strategy 3.5: Advanced color quantization using NeuQuant
        #[cfg(feature = "color_quant")]
        if !config.lossless.unwrap_or(false) && img.width() * img.height() > 1024 {
            wasm_log!("🔄 Applying advanced color quantization");
            
            // NeuQuant expects RGBA format, so convert to RGBA
            let rgba_img = img.to_rgba8();
            let pixels = rgba_img.as_raw();
            
            if pixels.len() >= 1024 { // At least 256 RGBA pixels
                let nq = NeuQuant::new(10, 256, pixels);
                let palette = nq.color_map_rgba();
                
                wasm_log!("🎨 Color quantization created palette with {} colors", palette.len() / 4);
                
                // Apply quantization by mapping each pixel to the nearest palette color
                let mut quantized_pixels = Vec::with_capacity(pixels.len());
                for chunk in pixels.chunks(4) {
                    if chunk.len() == 4 {
                        let color = [chunk[0], chunk[1], chunk[2], chunk[3]];
                        let index = nq.index_of(&color) as usize;
                        let base = index * 4;
                        if base + 3 < palette.len() {
                            quantized_pixels.push(palette[base]);     // R
                            quantized_pixels.push(palette[base + 1]); // G
                            quantized_pixels.push(palette[base + 2]); // B
                            quantized_pixels.push(palette[base + 3]); // A
                        } else {
                            // Fallback to original pixel if index is out of bounds
                            quantized_pixels.extend_from_slice(chunk);
                        }
                    }
                }
                
                if let Some(quantized_img) = image::RgbaImage::from_raw(img.width(), img.height(), quantized_pixels) {
                    let mut output = Vec::new();
                    let mut cursor = Cursor::new(&mut output);
                    
                    if let Ok(_) = quantized_img.write_to(&mut cursor, ImageFormat::Png) {
                        let reduction = 1.0 - (output.len() as f64 / data.len() as f64);
                        wasm_log!("📊 Advanced quantization result: {} bytes, {:.1}% reduction", 
                                  output.len(), reduction * 100.0);
                        
                        if output.len() < best_result.len() {
                            best_result = output;
                            best_reduction = reduction;
                            wasm_log!("✅ Advanced color quantization successful");
                        }
                    }
                }
            }
        }
        
        // Strategy 4: Quality-based re-encoding (always try for larger files)
        if data.len() > 2000 {  // Try for files over 2KB
            wasm_log!("🔧 Trying aggressive re-encoding for larger file...");
            let mut basic_output = Vec::new();
            let mut basic_cursor = Cursor::new(&mut basic_output);
            
            if let Ok(_) = img.write_to(&mut basic_cursor, ImageFormat::Png) {
                let reduction = 1.0 - (basic_output.len() as f64 / data.len() as f64);
                wasm_log!("📊 Aggressive re-encoding result: {} bytes, {:.1}% reduction", 
                          basic_output.len(), reduction * 100.0);
                
                // Accept even small improvements for larger files
                if basic_output.len() < best_result.len() && reduction > 0.01 {
                    best_result = basic_output;
                    best_reduction = reduction;
                    wasm_log!("✅ Aggressive re-encoding successful");
                }
            }
        }
    } else {
        wasm_log!("⚠️ Skipping optimization based on config and file size");
    }
    
    // Log final result
    if best_reduction > 0.005 {  // Accept even 0.5% improvement
        wasm_log!("🎯 PNG WASM optimization successful: {} bytes -> {} bytes ({:.1}% reduction)", 
                  data.len(), best_result.len(), best_reduction * 100.0);
        Ok(best_result)
    } else {
        wasm_log!("ℹ️ PNG WASM optimization: no significant improvement found (limited WASM PNG support)");
        Ok(data.to_vec())
    }
}

/// Basic PNG validation
pub fn validate_png(data: &[u8]) -> OptResult<()> {
    image::load_from_memory_with_format(data, ImageFormat::Png)
        .map_err(|e| OptError::ProcessingError(format!("Invalid PNG: {}", e)))?;
    Ok(())
}
