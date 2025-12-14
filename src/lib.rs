#![no_std]

extern crate alloc;
use alloc::{vec::Vec, string::String, format, string::ToString};

use wasm_bindgen::prelude::*;
use core::sync::atomic::{AtomicBool, Ordering};

static TRACING_INIT: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "dlmalloc")]
extern crate dlmalloc;

#[global_allocator]
#[cfg(feature = "dlmalloc")]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

pub mod config;
pub mod types;
pub mod optimizers;
pub mod formats;
pub mod image;
pub mod mesh;
pub mod common;
pub mod user_feedback;
pub mod c_hotspots;
pub mod benchmarks;

pub use config::*;
pub use types::*;
pub use optimizers::*;
pub use benchmarks::*;

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
        log(msg);
    }
}

#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(not(feature = "console_error_panic_hook"))]
pub fn set_panic_hook() {
}

#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
    
    #[cfg(feature = "tracing")]
    {
        if !TRACING_INIT.swap(true, Ordering::Relaxed) {
            tracing_wasm::set_as_global_default();
        }
    }
}

#[wasm_bindgen]
pub fn optimize_image(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if data.is_empty() {
        return Err(JsValue::from_str("Input data is empty"));
    }
    
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_image(data, quality)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_mesh(data: &[u8], target_ratio: Option<f32>) -> Result<Vec<u8>, JsValue> {
    if data.is_empty() {
        return Err(JsValue::from_str("Input data is empty"));
    }
    
    let optimizer = PixieOptimizer::new();
    let _target_faces = target_ratio.map(|ratio| (1000.0 * ratio) as u32);
    optimizer.optimize_mesh(data)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_auto(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if data.is_empty() {
        return Err(JsValue::from_str("Input data is empty"));
    }
    
    wasm_utils::log_message(&format!("🧪 DEBUG: optimize_auto called with {} bytes, quality {}", data.len(), quality));
    
    // CRITICAL: Size-based bypass to prevent compress_lz4 errors on large files
    let data_size = data.len();
    if data_size > 10_485_760 {
        wasm_utils::log_message(&format!("🚀 LARGE FILE BYPASS: {}MB - using safe path to avoid compress_lz4", data_size / 1_048_576));
        
        let optimizer = PixieOptimizer::new();
        optimizer.optimize_image(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        wasm_utils::log_message(&format!("📦 NORMAL PATH: {}MB - using optimize_auto", data_size / 1_048_576));
        
        let optimizer = PixieOptimizer::new();
        optimizer.optimize_auto(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn build_timestamp() -> String {
    "2024-12-29T23:35:00Z-webp-chunk-fix".to_string()
}

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

#[wasm_bindgen]
pub fn get_performance_metrics() -> JsValue {
    serde_wasm_bindgen::to_value(&optimizers::get_performance_stats())
        .unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn reset_performance_stats() {
    optimizers::reset_performance_stats();
}

#[wasm_bindgen]
pub fn check_performance_compliance() -> bool {
    optimizers::check_performance_compliance()
}

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

#[wasm_bindgen]
pub fn optimize_ico(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    crate::image::ico::optimize_ico(data, quality, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_tga(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    crate::image::tga::optimize_tga_entry(data, quality)
}

#[wasm_bindgen]
pub fn is_webp(data: &[u8]) -> bool {
    crate::image::webp::is_webp(data)
}

#[wasm_bindgen]
pub fn is_gif(data: &[u8]) -> bool {
    crate::image::gif::is_gif(data)
}

#[wasm_bindgen]
pub fn is_ico(data: &[u8]) -> bool {
    crate::image::ico::is_ico(data)
}

#[wasm_bindgen]
pub fn is_tga(data: &[u8]) -> bool {
    crate::image::tga::is_tga(data)
}

#[wasm_bindgen]
pub fn convert_to_webp(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::webp::convert_any_format_to_webp(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for WebP conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_png(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::png::convert_any_format_to_png(data)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for PNG conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_jpeg(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::jpeg::convert_any_format_to_jpeg(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for JPEG conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_bmp(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::bmp::convert_any_format_to_bmp(data)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for BMP conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_gif(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::gif::convert_any_format_to_gif(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for GIF conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_ico(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::ico::optimize_ico(data, quality, &config)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for ICO conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_tiff(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::tiff::optimize_tiff(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for TIFF conversion"))
    }
}

#[wasm_bindgen]
pub fn strip_tiff_metadata_simd(data: &[u8], preserve_icc: bool) -> Result<Vec<u8>, JsValue> {
    crate::c_hotspots::strip_tiff_metadata_c_hotspot(data, preserve_icc)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn convert_to_svg(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    use crate::types::ImageOptConfig;
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::svg::optimize_svg(data, quality, &config)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for SVG conversion"))
    }
}

#[wasm_bindgen]
pub fn convert_to_tga(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    if let Ok(_) = formats::detect_image_format(data) {
        crate::image::tga::convert_any_format_to_tga(data, quality)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    } else {
        Err(JsValue::from_str("Unsupported image format for TGA conversion"))
    }
}

#[wasm_bindgen]
pub fn set_lossless_mode(enabled: bool) -> JsValue {
    optimizers::set_lossless_mode_global(enabled);
    serde_wasm_bindgen::to_value(&format!("Lossless mode: {}", enabled))
        .unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn set_preserve_metadata(enabled: bool) -> JsValue {
    optimizers::set_preserve_metadata_global(enabled);
    serde_wasm_bindgen::to_value(&format!("Preserve metadata: {}", enabled))
        .unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn optimize_obj(data: &[u8], reduction_ratio: f32) -> Result<Vec<u8>, JsValue> {
    use crate::types::MeshOptConfig;
    let mut config = MeshOptConfig::default();
    config.target_ratio = reduction_ratio;
    crate::mesh::obj::optimize_obj(data, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_gltf(data: &[u8], reduction_ratio: f32) -> Result<Vec<u8>, JsValue> {
    use crate::types::MeshOptConfig;
    let mut config = MeshOptConfig::default();
    config.target_ratio = reduction_ratio;
    crate::mesh::gltf::optimize_gltf(data, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_stl(data: &[u8], reduction_ratio: f32) -> Result<Vec<u8>, JsValue> {
    use crate::types::MeshOptConfig;
    let mut config = MeshOptConfig::default();
    config.target_ratio = reduction_ratio;
    crate::mesh::stl::optimize_stl(data, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_fbx(data: &[u8], reduction_ratio: f32) -> Result<Vec<u8>, JsValue> {
    use crate::types::MeshOptConfig;
    let mut config = MeshOptConfig::default();
    config.target_ratio = reduction_ratio;
    crate::mesh::fbx::optimize_fbx(data, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn optimize_ply(data: &[u8], reduction_ratio: f32) -> Result<Vec<u8>, JsValue> {
    use crate::types::MeshOptConfig;
    let mut config = MeshOptConfig::default();
    config.target_ratio = reduction_ratio;
    crate::mesh::ply::optimize_ply(data, &config)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn is_obj(data: &[u8]) -> bool {
    matches!(crate::mesh::detect_mesh_format(data), Ok(crate::formats::MeshFormat::Obj))
}

#[wasm_bindgen]
pub fn is_gltf(data: &[u8]) -> bool {
    matches!(crate::mesh::detect_mesh_format(data), Ok(crate::formats::MeshFormat::Gltf | crate::formats::MeshFormat::Glb))
}

#[wasm_bindgen]
pub fn is_stl(data: &[u8]) -> bool {
    crate::mesh::stl::is_stl(data)
}

#[wasm_bindgen]
pub fn is_fbx(data: &[u8]) -> bool {
    matches!(crate::mesh::detect_mesh_format(data), Ok(crate::formats::MeshFormat::Fbx))
}

#[wasm_bindgen]
pub fn is_ply(data: &[u8]) -> bool {
    matches!(crate::mesh::detect_mesh_format(data), Ok(crate::formats::MeshFormat::Ply))
}
