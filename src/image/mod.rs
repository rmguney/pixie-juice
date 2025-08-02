//! Image processing and optimization module
//! 
//! This module provides image format detection and optimization capabilities using
//! the pure Rust `image` crate for WASM compatibility.

extern crate alloc;
use alloc::{vec::Vec, format};

use crate::types::{OptError, OptResult, ImageOptConfig};

#[cfg(feature = "image")]
use image::load_from_memory;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Import console log function for WASM
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Helper function for console logging
fn log_to_console(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe { log(msg); }
    
    #[cfg(not(target_arch = "wasm32"))]
    let _ = msg; // Suppress unused variable warning for non-WASM builds
}

// Helper function to detect animated GIFs
fn detect_animated_gif(data: &[u8]) -> bool {
    if data.len() < 13 {
        return false;
    }
    
    // Check for GIF signature
    if !data.starts_with(b"GIF87a") && !data.starts_with(b"GIF89a") {
        return false;
    }
    
    let mut pos = 13; // Skip header and logical screen descriptor
    let mut image_count = 0;
    
    // Skip global color table if present
    let flags = data[10];
    if flags & 0x80 != 0 {
        let global_color_table_size = 2_usize.pow(((flags & 0x07) + 1) as u32) * 3;
        pos += global_color_table_size;
    }
    
    // Look for multiple image descriptors (0x2C) which indicate animation
    while pos < data.len() {
        match data[pos] {
            0x21 => { // Extension introducer
                pos += 1;
                if pos >= data.len() { break; }
                
                let _label = data[pos];
                pos += 1;
                
                // Skip extension data
                while pos < data.len() {
                    let block_size = data[pos] as usize;
                    pos += 1;
                    if block_size == 0 { break; }
                    pos += block_size;
                    if pos > data.len() { return false; }
                }
            },
            0x2C => { // Image descriptor - this is a frame
                image_count += 1;
                if image_count > 1 {
                    return true; // Multiple images = animated
                }
                
                pos += 10; // Skip image descriptor
                
                // Skip local color table if present
                if pos < data.len() {
                    let local_flags = data[pos - 1];
                    if local_flags & 0x80 != 0 {
                        let local_color_table_size = 2_usize.pow(((local_flags & 0x07) + 1) as u32) * 3;
                        pos += local_color_table_size;
                    }
                }
                
                // Skip LZW minimum code size
                if pos < data.len() { pos += 1; }
                
                // Skip image data
                while pos < data.len() {
                    let block_size = data[pos] as usize;
                    pos += 1;
                    if block_size == 0 { break; }
                    pos += block_size;
                    if pos > data.len() { return false; }
                }
            },
            0x3B => { // Trailer
                break;
            },
            _ => {
                pos += 1;
            }
        }
    }
    
    false // Only one image found = static GIF
}

pub mod bmp;
pub mod formats;
pub mod gif;
pub mod jpeg;
pub mod png;
pub mod tiff;
pub mod webp;
// Phase 3: Advanced Image Formats (commented out for now)
// pub mod avif;
// pub mod heic;
// pub mod svg;  // Commented out - needs usvg dependency
// pub mod pdf;
pub mod ico;
// pub mod hdr;

// Zune optimization modules (currently disabled due to API compatibility)
// pub mod zune_jpeg;
// pub mod zune_png;
// Advanced algorithms module (currently disabled due to no-std compatibility)
// pub mod advanced;

// Re-export format detection  
pub use crate::formats::{detect_image_format};
pub use crate::formats::ImageFormat as PixieImageFormat;

/// Image optimizer that uses the `image` crate for processing
#[derive(Debug, Clone)]
pub struct ImageOptimizer {
    config: ImageOptConfig,
}

impl ImageOptimizer {
    /// Create a new image optimizer with the given configuration
    pub fn new(config: ImageOptConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &ImageOptConfig {
        &self.config
    }

    /// Optimize an image based on its detected format using the image crate
    pub fn optimize(&self, data: &[u8]) -> OptResult<Vec<u8>> {
        self.optimize_with_quality(data, self.config.quality)
    }

    /// Optimize an image with specific quality parameter using the image crate
    #[cfg(feature = "image")]
    pub fn optimize_with_quality(&self, data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
        // Detect the original format first
        let format = detect_image_format(data)?;
        
        // Handle formats that need special processing before loading
        match format {
            PixieImageFormat::WebP => {
                // Use dedicated WebP optimizer directly
                return webp::optimize_webp_old(data, quality, &self.config);
            },
            // Phase 1 formats still in development - commented out for now
            // PixieImageFormat::SVG => {
            //     // Use dedicated SVG optimizer directly  
            //     return svg::optimize_svg(data, quality, &self.config);
            // },
            PixieImageFormat::Ico => {
                // Use dedicated ICO optimizer directly
                return ico::optimize_ico(data, quality, &self.config);
            },
            _ => {
                // Continue with standard image crate processing
            }
        }
        
        // Try to load the image using the image crate for standard formats
        let img = load_from_memory(data)
            .map_err(|e| OptError::ProcessingError(format!("Failed to load image: {}", e)))?;
        
        // Always try to optimize - be aggressive with compression
        let mut best_output = data.to_vec();
        let mut best_size = data.len();
        let original_size = data.len();
        
        // Quality mapping: ensure we get meaningful compression
        let aggressive_quality = match quality {
            0..=20 => 15,    // Very aggressive
            21..=40 => 35,   // Aggressive  
            41..=60 => 55,   // Moderate
            61..=80 => 75,   // Conservative
            _ => 85,         // High quality
        };
        
        match format {
            crate::formats::ImageFormat::Png => {
                // Strategy 1: Re-encode PNG with optimized settings
                let mut png_output = Vec::new();
                let png_encoder = image::codecs::png::PngEncoder::new(&mut png_output);
                
                if img.write_with_encoder(png_encoder).is_ok() && !png_output.is_empty() {
                    if png_output.len() < best_size {
                        best_output = png_output.clone();
                        best_size = best_output.len();
                    }
                }
                
                // Strategy 2: Convert to JPEG for better compression (except for transparency)
                let has_transparency = match &img {
                    image::DynamicImage::ImageRgba8(rgba_img) => {
                        rgba_img.pixels().any(|p| p[3] < 255)
                    },
                    _ => false,
                };
                
                // Be more aggressive with JPEG conversion for PNG
                if !has_transparency && quality <= 95 {  // Lowered threshold from 90 to 95
                    let mut jpeg_output = Vec::new();
                    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, aggressive_quality);
                    if img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                        best_output = jpeg_output;
                        best_size = best_output.len();
                    }
                }
                
                // Strategy 3: WebP conversion for better compression
                if quality <= 85 {  // Increased threshold for WebP conversion
                    if let Ok(webp_output) = webp::optimize_webp_old(data, quality, &self.config) {
                        if webp_output.len() < best_size {
                            best_output = webp_output;
                        }
                    }
                }
            },
            crate::formats::ImageFormat::Jpeg => {
                // Always re-encode JPEG with aggressive quality reduction
                let mut jpeg_output = Vec::new();
                let target_quality = if original_size > 50000 {
                    // For larger files, be more aggressive
                    aggressive_quality.saturating_sub(10).max(10)
                } else {
                    aggressive_quality
                };
                
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, target_quality);
                if img.write_with_encoder(encoder).is_ok() && !jpeg_output.is_empty() {
                    // Always use re-encoded version for JPEG unless it's dramatically larger
                    if jpeg_output.len() < best_size || jpeg_output.len() < (original_size * 120 / 100) {
                        best_output = jpeg_output;
                        best_size = best_output.len();
                    }
                }
                
                // For very low quality, try WebP
                if quality < 60 {
                    if let Ok(webp_output) = webp::optimize_webp_old(data, quality, &self.config) {
                        if webp_output.len() < best_size {
                            best_output = webp_output;
                        }
                    }
                }
            },
            PixieImageFormat::WebP => {
                // WebP optimization is tricky with the image crate - it only supports lossless
                // So we need to be more aggressive with format conversion
                
                // Strategy 1: Try re-encoding with dedicated WebP optimizer
                if let Ok(webp_output) = webp::optimize_webp_old(data, quality, &self.config) {
                    if webp_output.len() < best_size {
                        best_output = webp_output.clone();
                        best_size = best_output.len();
                    }
                }
                
                // Strategy 2: Convert to JPEG for significant compression (most effective)
                if quality <= 85 {  // For most quality levels, try JPEG conversion
                    let mut jpeg_output = Vec::new();
                    let jpeg_quality = aggressive_quality;
                    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, jpeg_quality);
                    if img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                        best_output = jpeg_output;
                        best_size = best_output.len();
                    }
                }
                
                // Strategy 3: For very high quality, try PNG conversion
                if quality >= 90 && best_size == original_size {
                    let mut png_output = Vec::new();
                    let png_encoder = image::codecs::png::PngEncoder::new(&mut png_output);
                    if img.write_with_encoder(png_encoder).is_ok() && png_output.len() < best_size {
                        best_output = png_output;
                    }
                }
            },
            PixieImageFormat::Gif => {
                log_to_console(&format!("Optimizing GIF: original size {} bytes", data.len()));
                
                // First, detect if this is an animated GIF
                let is_animated = detect_animated_gif(data);
                log_to_console(&format!("GIF type: {}", if is_animated { "animated" } else { "static" }));
                
                if is_animated {
                    // For animated GIFs, use GIF-specific optimization to preserve animation
                    #[cfg(feature = "codec-gif")]
                    {
                        match crate::image::gif::optimize_gif_rust(data, quality, &ImageOptConfig::default()) {
                            Ok(optimized_gif) => {
                                if optimized_gif.len() < best_size {
                                    log_to_console(&format!("Animated GIF optimization: {} -> {} bytes ({:.1}% savings)", 
                                        best_size, optimized_gif.len(),
                                        ((best_size - optimized_gif.len()) as f64 / best_size as f64) * 100.0));
                                    best_output = optimized_gif;
                                    best_size = best_output.len();
                                } else {
                                    log_to_console("Animated GIF optimization: no improvement, keeping original");
                                }
                            },
                            Err(_) => {
                                log_to_console("Animated GIF optimization failed, keeping original");
                            }
                        }
                    }
                    #[cfg(not(feature = "codec-gif"))]
                    {
                        log_to_console("Animated GIF optimization: GIF codec not available, keeping original");
                    }
                } else {
                    // For static GIFs, convert to more efficient formats
                    log_to_console("Static GIF: converting to more efficient format");
                    
                    // Strategy 1: Try PNG conversion for high quality (preserves transparency)
                    if quality >= 70 {
                        let mut png_output = Vec::new();
                        let png_encoder = image::codecs::png::PngEncoder::new(&mut png_output);
                        if img.write_with_encoder(png_encoder).is_ok() && png_output.len() < best_size {
                            log_to_console(&format!("PNG conversion: {} -> {} bytes", best_size, png_output.len()));
                            best_output = png_output;
                            best_size = best_output.len();
                        }
                    }
                    
                    // Strategy 2: Try JPEG conversion for better compression
                    if quality <= 85 {
                        let mut jpeg_output = Vec::new();
                        let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, aggressive_quality);
                        if img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                            log_to_console(&format!("JPEG conversion: {} -> {} bytes", best_size, jpeg_output.len()));
                            best_output = jpeg_output;
                            best_size = best_output.len();
                        }
                    }
                    
                    // Strategy 3: Try WebP conversion for modern browsers
                    if quality <= 80 {
                        if let Ok(webp_output) = webp::optimize_webp_old(data, quality, &self.config) {
                            if webp_output.len() < best_size {
                                log_to_console(&format!("WebP conversion: {} -> {} bytes", best_size, webp_output.len()));
                                best_output = webp_output;
                                best_size = best_output.len();
                            }
                        }
                    }
                    
                    // Strategy 4: If format conversion didn't help much, try GIF-specific optimization
                    if best_size >= original_size * 95 / 100 { // If we only achieved < 5% savings
                        log_to_console("Format conversion had minimal benefit, trying GIF-specific optimization");
                        #[cfg(feature = "codec-gif")]
                        {
                            match crate::image::gif::optimize_gif_rust(data, quality, &ImageOptConfig::default()) {
                                Ok(optimized_gif) => {
                                    if optimized_gif.len() < best_size {
                                        log_to_console(&format!("GIF optimization: {} -> {} bytes ({:.1}% savings)", 
                                            best_size, optimized_gif.len(),
                                            ((best_size - optimized_gif.len()) as f64 / best_size as f64) * 100.0));
                                        best_output = optimized_gif;
                                        best_size = best_output.len();
                                    }
                                },
                                Err(_) => {
                                    log_to_console("GIF-specific optimization failed");
                                }
                            }
                        }
                    }
                }
                
                log_to_console(&format!("GIF optimization result: {} -> {} bytes ({:.1}% savings)", 
                    data.len(), best_output.len(), 
                    ((data.len() - best_output.len()) as f64 / data.len() as f64) * 100.0));
            },
            PixieImageFormat::Bmp => {
                // BMP is always uncompressed, so any conversion will be smaller
                if quality >= 85 {
                    // High quality: convert to PNG
                    let mut png_output = Vec::new();
                    let png_encoder = image::codecs::png::PngEncoder::new(&mut png_output);
                    if img.write_with_encoder(png_encoder).is_ok() {
                        best_output = png_output;
                    }
                } else {
                    // Lower quality: convert to JPEG
                    let mut jpeg_output = Vec::new();
                    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, aggressive_quality);
                    if img.write_with_encoder(jpeg_encoder).is_ok() {
                        best_output = jpeg_output;
                    }
                }
            },
            PixieImageFormat::Tiff => {
                // Use dedicated TIFF optimizer with library-first approach
                if let Ok(tiff_output) = tiff::optimize_tiff(data, quality) {
                    if tiff_output.len() < best_size {
                        best_output = tiff_output;
                    }
                }
            },
            PixieImageFormat::Svg => {
                // SVG optimization - placeholder for now since it's not in Phase 1
                // Will be implemented in Phase 3
                best_output = data.to_vec();
            },
            // Phase 3: Advanced Image Formats (commented out for now)
            /*
            PixieImageFormat::Avif => {
                // AVIF optimization using ravif crate for re-encoding with quality control
                if let Ok(optimized) = crate::image::avif::optimize_avif(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    // Fallback: try converting to JPEG/WebP if AVIF optimization fails
                    if let Ok(converted) = crate::image::avif::convert_avif_to_compressed(data, aggressive_quality) {
                        best_output = converted;
                    } else {
                        best_output = data.to_vec();
                    }
                }
            },
            ImageFormat::HEIC => {
                // HEIC optimization using libheif-rs or conversion to JPEG/WebP
                if let Ok(optimized) = crate::image::heic::optimize_heic(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    // Fallback: try converting to JPEG if HEIC optimization fails
                    if let Ok(converted) = crate::image::heic::convert_heic_to_compressed(data, aggressive_quality) {
                        best_output = converted;
                    } else {
                        best_output = data.to_vec();
                    }
                }
            },
            */
            // Phase 1 formats still in development - SVG commented out  
            // ImageFormat::SVG => {
            //     // SVG optimization using text-based processing
            //     if let Ok(optimized) = crate::image::svg::optimize_svg(data, aggressive_quality, &self.config) {
            //         best_output = optimized;
            //     } else {
            //         best_output = data.to_vec();
            //     }
            // },
            /*
            ImageFormat::PDF => {
                // PDF optimization using basic processing
                if let Ok(optimized) = crate::image::pdf::optimize_pdf(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    best_output = data.to_vec();
                }
            },
            */
            PixieImageFormat::Ico => {
                // ICO optimization using embedded image processing
                if let Ok(optimized) = crate::image::ico::optimize_ico(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    best_output = data.to_vec();
                }
            },
            /*
            PixieImageFormat::Hdr => {
                // HDR optimization using specialized processing
                if let Ok(optimized) = crate::image::hdr::optimize_hdr(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    best_output = data.to_vec();
                }
            },
            ImageFormat::EXR => {
                // EXR optimization using specialized processing
                if let Ok(optimized) = crate::image::hdr::optimize_exr(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    best_output = data.to_vec();
                }
            },
            */
        }
        
        // Log the optimization result for debugging
        let savings = ((original_size as f64 - best_output.len() as f64) / original_size as f64 * 100.0) as i32;
        if savings > 0 {
            // Use console.log for WASM debugging
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::prelude::*;
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = console)]
                    fn log(s: &str);
                }
                log(&format!("Image optimization: {} bytes → {} bytes ({savings}% reduction)", original_size, best_output.len()));
            }
        }
        
        Ok(best_output)
    }

    /// Fallback optimization when image crate is not available
    #[cfg(not(feature = "image"))]
    pub fn optimize_with_quality(&self, data: &[u8], _quality: u8) -> OptResult<Vec<u8>> {
        Err(OptError::UnsupportedFormat("Image processing not available - missing image feature".into()))
    }

    /// Analyze image format and basic properties
    pub fn analyze(&self, data: &[u8]) -> OptResult<crate::types::ImageInfo> {
        let _format = detect_image_format(data)?;
        // TODO: Return actual image analysis using image crate
        Ok(crate::types::ImageInfo::default())
    }
}

impl Default for ImageOptimizer {
    fn default() -> Self {
        Self::new(ImageOptConfig::default())
    }
}
