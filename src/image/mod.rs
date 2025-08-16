//! Image processing and optimization module

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
pub mod svg;
pub mod ico;
pub mod tga;

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
        // Debug logging - entry point
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!("🔥 Image optimization entry: {} bytes, quality {}%", data.len(), quality);
            log_to_console(&msg);
        }
        
        // Detect the original format first
        let format = detect_image_format(data);
        
        #[cfg(target_arch = "wasm32")]
        {
            match &format {
                Ok(f) => {
                    let msg = format!("🔍 Detected format: {:?}", f);
                    log_to_console(&msg);
                },
                Err(e) => {
                    let msg = format!("❌ Format detection failed: {:?}", e);
                    log_to_console(&msg);
                }
            }
        }
        
        let format = format?;
        
        // Handle formats that need special processing before loading
        match format {
            PixieImageFormat::WebP => {
                #[cfg(target_arch = "wasm32")]
                log_to_console("🎯 Routing to WebP-specific optimization");
                
                // Use comprehensive WebP optimizer with all strategies
                let result = webp::optimize_webp_with_config(data, quality, &self.config)
                    .map_err(|e| OptError::ProcessingError(format!("{:?}", e)));
                
                #[cfg(target_arch = "wasm32")]
                {
                    match &result {
                        Ok(optimized) => {
                            let compression = ((data.len() - optimized.len()) as f64 / data.len() as f64) * 100.0;
                            let msg = format!("✅ WebP optimization returned: {} -> {} bytes ({:.2}% compression)", 
                                            data.len(), optimized.len(), compression);
                            log_to_console(&msg);
                        },
                        Err(e) => {
                            let msg = format!("❌ WebP optimization failed: {:?}", e);
                            log_to_console(&msg);
                        }
                    }
                }
                
                return result;
            },
            // SVG optimization disabled - dependency issues
            PixieImageFormat::Svg => {
                // ✅ ENABLED: SVG optimization with   approach
                return svg::optimize_svg(data, quality, &self.config)
                    .map_err(|e| OptError::ProcessingError(format!("{:?}", e)));
            },
            PixieImageFormat::Ico => {
                // Use dedicated ICO optimizer directly
                return ico::optimize_ico(data, quality, &self.config);
            },
            PixieImageFormat::Tga => {
                // ✅ NEW: TGA optimization
                #[cfg(target_arch = "wasm32")]
                log_to_console("🎯 Routing to TGA-specific optimization");
                
                return tga::optimize_tga_with_quality(data, quality)
                    .map_err(|e| OptError::ProcessingError(format!("TGA optimization failed: {:?}", e)));
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
        let best_size = data.len();
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
                // CRITICAL FIX: Use the comprehensive PNG optimizer instead of basic re-encoding
                if let Ok(png_optimized) = crate::image::png::optimize_png_rust(data, quality) {
                    if png_optimized.len() < best_size {
                        best_output = png_optimized;
                    }
                }
            },
            crate::formats::ImageFormat::Jpeg => {
                // CRITICAL FIX: Use the comprehensive JPEG optimizer instead of basic re-encoding
                if let Ok(jpeg_optimized) = crate::image::jpeg::optimize_jpeg(data, quality, &self.config) {
                    if jpeg_optimized.len() < best_size {
                        best_output = jpeg_optimized;
                    }
                }
            },
            PixieImageFormat::WebP => {
                // WebP optimization is tricky with the image crate - it only supports lossless
                // So we need to be more aggressive with format conversion
                
                // Strategy 1: Try re-encoding with comprehensive WebP optimizer
                if let Ok(webp_output) = webp::optimize_webp_with_config(data, quality, &self.config) {
                    if webp_output.len() < best_size {
                        best_output = webp_output.clone();
                    }
                }
                
                // Strategy 2: Convert to JPEG for significant compression (most effective)
                if quality <= 85 {  // For most quality levels, try JPEG conversion
                    let mut jpeg_output = Vec::new();
                    let jpeg_quality = aggressive_quality;
                    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, jpeg_quality);
                    if img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                        best_output = jpeg_output;
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
                        }
                    }
                    
                    // Strategy 2: Try JPEG conversion for better compression
                    if quality <= 85 {
                        let mut jpeg_output = Vec::new();
                        let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, aggressive_quality);
                        if img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                            log_to_console(&format!("JPEG conversion: {} -> {} bytes", best_size, jpeg_output.len()));
                            best_output = jpeg_output;
                        }
                    }
                    
                    // Strategy 3: Try WebP conversion for modern browsers
                    if quality <= 80 {
                        if let Ok(webp_output) = webp::optimize_webp_with_config(data, quality, &self.config) {
                            if webp_output.len() < best_size {
                                log_to_console(&format!("WebP conversion: {} -> {} bytes", best_size, webp_output.len()));
                                best_output = webp_output;
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
                if let Ok(tiff_output) = tiff::optimize_tiff(data, quality) {
                    if tiff_output.len() < best_size {
                        best_output = tiff_output;
                    }
                }
            },
            PixieImageFormat::Svg => {
                if let Ok(svg_output) = svg::optimize_svg(data, quality, &self.config) {
                    if svg_output.len() < best_size {
                        best_output = svg_output;
                    }
                }
            },
            PixieImageFormat::Tga => {
                if let Ok(tga_output) = tga::optimize_tga(data, quality) {
                    if tga_output.len() < best_size {
                        best_output = tga_output;
                    }
                }
            },

            PixieImageFormat::Ico => {
                // ICO optimization using embedded image processing
                if let Ok(optimized) = crate::image::ico::optimize_ico(data, aggressive_quality, &self.config) {
                    best_output = optimized;
                } else {
                    best_output = data.to_vec();
                }
            },
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

    /// Fast path optimization for large images to avoid performance violations
    /// Skips complex processing and uses simplified optimization strategies
    #[cfg(feature = "image")]
    pub fn optimize_with_quality_fast_path(&self, data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
        // Debug logging - fast path entry point
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!("🚀 Fast path optimization: {} bytes, quality {}%", data.len(), quality);
            log_to_console(&msg);
        }
        
        // Detect the original format first
        let format = detect_image_format(data)?;
        
        // CRITICAL: Check for animations first - they need special handling even in fast path
        match format {
            PixieImageFormat::Gif => {
                // Check if it's an animated GIF
                #[cfg(target_arch = "wasm32")]
                log_to_console("🎬 Fast path: Detected GIF - checking for animation");
                
                if crate::image::gif::detect_animated_gif(data) {
                    #[cfg(target_arch = "wasm32")]
                    log_to_console("🎬 Fast path: Animated GIF detected - using full animation optimization");
                    
                    // Use full GIF optimization even in fast path for animations
                    return crate::image::gif::optimize_gif_rust(data, quality, &ImageOptConfig::default())
                        .map_err(|e| OptError::ProcessingError(format!("{}", e)));
                } else {
                    #[cfg(target_arch = "wasm32")]
                    log_to_console("🖼️ Fast path: Static GIF - converting to PNG for better compression");
                }
            },
            PixieImageFormat::WebP => {
                // Check if it's an animated WebP
                #[cfg(target_arch = "wasm32")]
                log_to_console("🎬 Fast path: Detected WebP - checking for animation");
                
                if crate::image::webp::detect_animated_webp(data) {
                    #[cfg(target_arch = "wasm32")]
                    log_to_console("🎬 Fast path: Animated WebP detected - using full animation optimization");
                    
                    // Use full WebP optimization even in fast path for animations
                    return crate::image::webp::optimize_webp_rust(data, quality)
                        .map_err(|e| OptError::ProcessingError(format!("{}", e)));
                } else {
                    #[cfg(target_arch = "wasm32")]
                    log_to_console("🖼️ Fast path: Static WebP - continuing with fast optimization");
                }
            },
            _ => {
                #[cfg(target_arch = "wasm32")]
                log_to_console("🖼️ Fast path: Non-animation format - using fast optimization");
            }
        }
        
        // For large files, use only the most efficient optimizations for non-animated content
        match format {
            PixieImageFormat::Jpeg => {
                // For JPEG, just re-encode with lower quality - fastest approach
                if let Ok(img) = load_from_memory(data) {
                    let mut output = Vec::new();
                    let target_quality = (quality as f32 * 0.8) as u8; // Be more aggressive
                    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, target_quality);
                    if img.write_with_encoder(encoder).is_ok() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let savings = ((data.len() - output.len()) as f64 / data.len() as f64) * 100.0;
                            let msg = format!("Fast JPEG optimization: {} -> {} bytes ({:.1}% reduction)", 
                                            data.len(), output.len(), savings);
                            log_to_console(&msg);
                        }
                        return Ok(output);
                    }
                }
            },
            PixieImageFormat::Png => {
                // For PNG, convert to JPEG for faster processing (static images only)
                if let Ok(img) = load_from_memory(data) {
                    let mut output = Vec::new();
                    let target_quality = (quality as f32 * 0.8) as u8;
                    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, target_quality);
                    if img.write_with_encoder(encoder).is_ok() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let savings = ((data.len() - output.len()) as f64 / data.len() as f64) * 100.0;
                            let msg = format!("Fast PNG->JPEG conversion: {} -> {} bytes ({:.1}% reduction)", 
                                            data.len(), output.len(), savings);
                            log_to_console(&msg);
                        }
                        return Ok(output);
                    }
                }
            },
            PixieImageFormat::WebP => {
                // For static WebP, convert to JPEG for faster processing
                if let Ok(img) = load_from_memory(data) {
                    let mut output = Vec::new();
                    let target_quality = (quality as f32 * 0.8) as u8;
                    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, target_quality);
                    if img.write_with_encoder(encoder).is_ok() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let savings = ((data.len() - output.len()) as f64 / data.len() as f64) * 100.0;
                            let msg = format!("Fast WebP->JPEG conversion: {} -> {} bytes ({:.1}% reduction)", 
                                            data.len(), output.len(), savings);
                            log_to_console(&msg);
                        }
                        return Ok(output);
                    }
                }
            },
            PixieImageFormat::Gif => {
                // For static GIF, convert to PNG for better compression
                if let Ok(img) = load_from_memory(data) {
                    let mut output = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut output);
                    if img.write_with_encoder(encoder).is_ok() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let savings = ((data.len() - output.len()) as f64 / data.len() as f64) * 100.0;
                            let msg = format!("Fast GIF->PNG conversion: {} -> {} bytes ({:.1}% reduction)", 
                                            data.len(), output.len(), savings);
                            log_to_console(&msg);
                        }
                        return Ok(output);
                    }
                }
            },
            _ => {
                // For other formats, just re-encode as JPEG with target quality
                if let Ok(img) = load_from_memory(data) {
                    let mut output = Vec::new();
                    let target_quality = (quality as f32 * 0.8) as u8;
                    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, target_quality);
                    if img.write_with_encoder(encoder).is_ok() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let savings = ((data.len() - output.len()) as f64 / data.len() as f64) * 100.0;
                            let msg = format!("Fast format conversion to JPEG: {} -> {} bytes ({:.1}% reduction)", 
                                            data.len(), output.len(), savings);
                            log_to_console(&msg);
                        }
                        return Ok(output);
                    }
                }
            }
        }
        
        // If all fast path optimizations fail, return minimal compression
        #[cfg(target_arch = "wasm32")]
        log_to_console("Fast path optimization fallback: returning original data");
        
        Ok(data.to_vec())
    }

    /// Fallback fast path optimization when image crate is not available
    #[cfg(not(feature = "image"))]
    pub fn optimize_with_quality_fast_path(&self, data: &[u8], _quality: u8) -> OptResult<Vec<u8>> {
        Ok(data.to_vec()) // Just return original data
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
