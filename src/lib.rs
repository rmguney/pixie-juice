//! Pixie Juice - High-performance WASM-only media processing library
//! 
//! This library provides optimized image and mesh processing capabilities
//! designed specifically for WebAssembly execution in modern browsers.

#![no_std]

extern crate alloc;
use alloc::{vec::Vec, string::String, format, string::ToString};

use wasm_bindgen::prelude::*;

// Use WASM-compatible allocator
#[cfg(feature = "dlmalloc")]
extern crate dlmalloc;

#[global_allocator]
#[cfg(feature = "dlmalloc")]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

// Import modules following the architecture pattern
pub mod config;
pub mod types;
pub mod optimizers;
pub mod formats;
pub mod image;
pub mod mesh;
pub mod ffi;
pub mod common;
pub mod user_feedback;
pub mod c_hotspots;
pub mod benchmarks;

// Re-export main types for easier API usage
pub use config::*;
pub use types::*;
pub use optimizers::*;
pub use benchmarks::*;

// WASM utilities
mod wasm_utils {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
        
        #[wasm_bindgen(js_namespace = performance)]
        fn now() -> f64;
    }
    
    pub fn log_message(msg: &str) {
        unsafe {
            log(msg);
        }
    }
    
    pub fn get_timestamp() -> f64 {
        now()
    }
}

// Set up panic hook for better error messages in WASM
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(not(feature = "console_error_panic_hook"))]
pub fn set_panic_hook() {
    // No-op when panic hook feature is disabled
}

// WASM initialization function
#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
    
    #[cfg(feature = "tracing")]
    {
        tracing_wasm::set_as_global_default();
    }
}

// Main WASM exports for image optimization
#[wasm_bindgen]
pub fn optimize_image(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_image(data, quality)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

// Main WASM exports for mesh optimization  
#[wasm_bindgen]
pub fn optimize_mesh(data: &[u8], target_ratio: Option<f32>) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    let _target_faces = target_ratio.map(|ratio| (1000.0 * ratio) as u32); // Convert ratio to face count
    optimizer.optimize_mesh(data)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

// Auto-detect format and optimize
#[wasm_bindgen]
pub fn optimize_auto(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_auto(data, quality)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

// Get library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Detect file format
#[wasm_bindgen]
pub fn detect_format(data: &[u8]) -> String {
    use formats::*;
    
    if let Ok(format) = detect_image_format(data) {
        format!("image:{:?}", format)
    } else if let Ok(format) = detect_mesh_format(data) {
        format!("mesh:{:?}", format)
    } else {
        "unknown".to_string()
    }
}

// Performance monitoring
#[wasm_bindgen]
pub fn get_performance_metrics() -> JsValue {
    serde_wasm_bindgen::to_value(&optimizers::get_performance_stats())
        .unwrap_or(JsValue::NULL)
}

// Reset performance statistics
#[wasm_bindgen]
pub fn reset_performance_stats() {
    optimizers::reset_performance_stats();
}

// Check performance compliance
#[wasm_bindgen]
pub fn check_performance_compliance() -> bool {
    optimizers::check_performance_compliance()
}

// Specific format optimization functions
#[wasm_bindgen]
pub fn optimize_png(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    crate::image::png::optimize_png_rust(data, quality)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_jpeg(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::jpeg::optimize_jpeg(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_webp(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    crate::image::webp::optimize_webp(data, quality)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_gif(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::gif::optimize_gif_rust(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

// Phase 3: Advanced Image Formats (commented out for now)
/*
#[wasm_bindgen]
pub fn optimize_avif(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::avif::optimize_avif(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_heic(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::heic::optimize_heic(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}
*/

/*
#[wasm_bindgen]
pub fn optimize_svg(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::svg::optimize_svg(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}
*/

/*
#[wasm_bindgen]
pub fn optimize_pdf(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::pdf::optimize_pdf(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}
*/

#[wasm_bindgen]
pub fn optimize_ico(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::ico::optimize_ico(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

/*
#[wasm_bindgen]
pub fn optimize_tiff(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    crate::image::tiff::optimize_tiff(data, quality)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_hdr(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::hdr::optimize_hdr(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_exr(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::hdr::optimize_exr(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}
*/

// Format detection functions
#[wasm_bindgen]
pub fn is_webp(data: &[u8]) -> bool {
    crate::image::webp::is_webp(data)
}

#[wasm_bindgen]
pub fn is_gif(data: &[u8]) -> bool {
    crate::image::gif::is_gif(data)
}

/*
#[wasm_bindgen]
pub fn is_heic(data: &[u8]) -> bool {
    crate::image::heic::is_heic(data)
}
*/

/*
#[wasm_bindgen]
pub fn is_svg(data: &[u8]) -> bool {
    crate::image::svg::is_svg(data)
}
*/

/*
#[wasm_bindgen]
pub fn is_pdf(data: &[u8]) -> bool {
    crate::image::pdf::is_pdf(data)
}
*/

#[wasm_bindgen]
pub fn is_ico(data: &[u8]) -> bool {
    crate::image::ico::is_ico(data)
}

/*
#[wasm_bindgen]
pub fn is_hdr(data: &[u8]) -> bool {
    crate::image::hdr::is_hdr(data)
}

#[wasm_bindgen]
pub fn is_exr(data: &[u8]) -> bool {
    crate::image::hdr::is_exr(data)
}

#[wasm_bindgen]
pub fn is_tiff(data: &[u8]) -> bool {
    crate::image::tiff::is_tiff(data)
}
*/

// Conversion functions
#[wasm_bindgen]
pub fn convert_to_webp(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    
    // First detect the format, then convert
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::webp::optimize_webp(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for WebP conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_png(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Convert any image format to PNG (lossless)
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::png::optimize_png_rust(data, 100)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for PNG conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_jpeg(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    
    // Convert any image format to JPEG
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::jpeg::optimize_jpeg(data, quality, &config)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for JPEG conversion"))
    }
}

// Advanced configuration functions
#[wasm_bindgen]
pub fn set_lossless_mode(enabled: bool) -> JsValue {
    serde_wasm_bindgen::to_value(&format!("Lossless mode: {}", enabled))
        .unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn set_preserve_metadata(enabled: bool) -> JsValue {
    serde_wasm_bindgen::to_value(&format!("Preserve metadata: {}", enabled))
        .unwrap_or(JsValue::NULL)
}
