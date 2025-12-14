extern crate alloc;
use alloc::{vec::Vec, format, string::ToString};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};

use crate::types::{PixieResult, PixieError, ImageOptConfig, MeshOptConfig};
use crate::image::{ImageOptimizer, detect_image_format};
use crate::mesh::{MeshOptimizer, detect_mesh_format};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

pub fn get_current_time_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        now()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed) as f64
    }
}

#[cfg(target_arch = "wasm32")]
fn get_memory_usage_mb() -> f64 {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        type Performance;
        
        #[wasm_bindgen(js_namespace = ["performance", "memory"], js_name = usedJSHeapSize)]
        static USED_JS_HEAP_SIZE: JsValue;
    }
    
    if let Some(heap_size) = USED_JS_HEAP_SIZE.as_f64() {
        heap_size / 1_048_576.0
    } else {
        0.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub images_processed: u64,
    pub meshes_processed: u64,
    pub avg_image_time_ms: f64,
    pub avg_mesh_time_ms: f64,
    pub max_image_time_ms: f64,
    pub max_mesh_time_ms: f64,
    pub last_operation_time_ms: f64,
    pub total_bytes_processed: u64,
    pub memory_peak_mb: f64,
    pub errors_count: u64,
    pub performance_target_violations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOptConfig {
    pub preserve_metadata: bool,
    pub lossless_mode: bool,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            images_processed: 0,
            meshes_processed: 0,
            avg_image_time_ms: 0.0,
            avg_mesh_time_ms: 0.0,
            max_image_time_ms: 0.0,
            max_mesh_time_ms: 0.0,
            last_operation_time_ms: 0.0,
            total_bytes_processed: 0,
            memory_peak_mb: 0.0,
            errors_count: 0,
            performance_target_violations: 0,
        }
    }
}

impl Default for GlobalOptConfig {
    fn default() -> Self {
        Self {
            preserve_metadata: false,
            lossless_mode: false,
        }
    }
}

static IMAGES_PROCESSED: AtomicU64 = AtomicU64::new(0);
static MESHES_PROCESSED: AtomicU64 = AtomicU64::new(0);
pub static ERRORS_COUNT: AtomicU64 = AtomicU64::new(0);
static TOTAL_BYTES_PROCESSED: AtomicU64 = AtomicU64::new(0);
static PERFORMANCE_TARGET_VIOLATIONS: AtomicU32 = AtomicU32::new(0);

static PRESERVE_METADATA: AtomicBool = AtomicBool::new(false);
static LOSSLESS_MODE: AtomicBool = AtomicBool::new(false);

pub fn get_global_config() -> GlobalOptConfig {
    GlobalOptConfig {
        preserve_metadata: PRESERVE_METADATA.load(Ordering::Relaxed),
        lossless_mode: LOSSLESS_MODE.load(Ordering::Relaxed),
    }
}

pub fn set_preserve_metadata_global(enabled: bool) {
    PRESERVE_METADATA.store(enabled, Ordering::Relaxed);
}

pub fn set_lossless_mode_global(enabled: bool) {
    LOSSLESS_MODE.store(enabled, Ordering::Relaxed);
}

const IMAGE_TARGET_MS: f64 = 150.0;
const SMALL_FILE_TARGET_MS: f64 = 100.0;
#[cfg(target_arch = "wasm32")]
const MEMORY_TARGET_MB: f64 = 256.0; // 256MB memory target for WASM

pub fn update_performance_stats(is_image: bool, elapsed_ms: f64, data_size: usize) {
    TOTAL_BYTES_PROCESSED.fetch_add(data_size as u64, Ordering::Relaxed);
    
    if is_image {
        IMAGES_PROCESSED.fetch_add(1, Ordering::Relaxed);
    } else {
        MESHES_PROCESSED.fetch_add(1, Ordering::Relaxed);
    }
    
    let target_ms = if data_size < 512_000 { // <512KB
        SMALL_FILE_TARGET_MS
    } else {
        IMAGE_TARGET_MS
    };
    
    if elapsed_ms > target_ms {
        PERFORMANCE_TARGET_VIOLATIONS.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn get_performance_stats() -> PerformanceStats {
    PerformanceStats {
        images_processed: IMAGES_PROCESSED.load(Ordering::Relaxed),
        meshes_processed: MESHES_PROCESSED.load(Ordering::Relaxed),
        avg_image_time_ms: 0.0,
        avg_mesh_time_ms: 0.0,
        max_image_time_ms: 0.0,
        max_mesh_time_ms: 0.0,
        last_operation_time_ms: 0.0,
        total_bytes_processed: TOTAL_BYTES_PROCESSED.load(Ordering::Relaxed),
        memory_peak_mb: 0.0,
        errors_count: ERRORS_COUNT.load(Ordering::Relaxed),
        performance_target_violations: PERFORMANCE_TARGET_VIOLATIONS.load(Ordering::Relaxed) as u64,
    }
}

pub fn reset_performance_stats() {
    IMAGES_PROCESSED.store(0, Ordering::Relaxed);
    MESHES_PROCESSED.store(0, Ordering::Relaxed);
    ERRORS_COUNT.store(0, Ordering::Relaxed);
    TOTAL_BYTES_PROCESSED.store(0, Ordering::Relaxed);
    PERFORMANCE_TARGET_VIOLATIONS.store(0, Ordering::Relaxed);
}

pub fn check_performance_compliance() -> bool {
    let images = IMAGES_PROCESSED.load(Ordering::Relaxed);
    let meshes = MESHES_PROCESSED.load(Ordering::Relaxed);
    let total_ops = images + meshes;
    
    if total_ops == 0 {
        return true;
    }
    
    let violations = PERFORMANCE_TARGET_VIOLATIONS.load(Ordering::Relaxed) as u64;
    let violation_rate = violations as f64 / total_ops as f64;
    violation_rate < 0.05
}

pub struct PixieOptimizer {
    image_optimizer: ImageOptimizer,
    mesh_optimizer: MeshOptimizer,
}

impl PixieOptimizer {
    pub fn new() -> Self {
        Self {
            image_optimizer: ImageOptimizer::new(ImageOptConfig::with_global_settings()),
            mesh_optimizer: MeshOptimizer::default(),
        }
    }

    pub fn with_configs(image_config: ImageOptConfig, mesh_config: MeshOptConfig) -> Self {
        Self {
            image_optimizer: ImageOptimizer::new(image_config),
            mesh_optimizer: MeshOptimizer::new(mesh_config),
        }
    }

    pub fn optimize_auto(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        // FAST PATH: For large files (>100KB), use simpler optimization to avoid performance violations
        let use_fast_path = data_size > 100_000;
        
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = console)]
                fn log(s: &str);
            }
            log(&format!("🔍 File size check: {} bytes, fast path: {}", data_size, use_fast_path));
        }
        
        let base_result = if let Ok(_format) = detect_image_format(data) {
            let image_result = if use_fast_path {
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::prelude::*;
                    #[wasm_bindgen]
                    extern "C" {
                        #[wasm_bindgen(js_namespace = console)]
                        fn log(s: &str);
                    }
                    log(&format!("🚀 Using fast path for large file: {}KB", data_size / 1024));
                }
                self.image_optimizer.optimize_with_quality_fast_path(data, quality)
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::prelude::*;
                    #[wasm_bindgen]
                    extern "C" {
                        #[wasm_bindgen(js_namespace = console)]
                        fn log(s: &str);
                    }
                    log(&format!("📦 Using normal path for small file: {}KB", data_size / 1024));
                }
                self.image_optimizer.optimize_with_quality(data, quality)
            };
            let elapsed = get_current_time_ms() - start_time;
            update_performance_stats(true, elapsed, data_size);
            image_result
        } else if let Ok(_format) = detect_mesh_format(data) {
            let mesh_result = self.mesh_optimizer.optimize(data);
            let elapsed = get_current_time_ms() - start_time;
            update_performance_stats(false, elapsed, data_size);
            mesh_result
        } else {
            ERRORS_COUNT.fetch_add(1, Ordering::Relaxed);
            Err(PixieError::InvalidInput("Unknown file format".to_string()))
        };
        
        match base_result {
            Ok(optimized) => {
                #[cfg(c_hotspots_available)]
                {
                    if !use_fast_path && data.len() > 50_000 && quality < 70 {
                        if let Ok(compressed) = crate::c_hotspots::compress_data_c_hotspot(&optimized) {
                            if compressed.len() < optimized.len().saturating_mul(90).saturating_div(100) {
                                return Ok(compressed);
                            }
                        }
                    }
                }
                Ok(optimized)
            },
            Err(e) => Err(e)
        }
    }

    fn fast_path_image_optimize(&self, data: &[u8], quality: u8, _target_ms: f64) -> PixieResult<Vec<u8>> {
        let _start_time = get_current_time_ms();
        
        // Detect format for fast path decisions
        let format_result = detect_image_format(data);
        
        // Use format-specific fast optimizations
        let format_str = match format_result {
            Ok(fmt) => format!("{:?}", fmt),
            Err(_) => "Unknown".to_string(),
        };
        
        match format_str.as_str() {
            "PNG" => {
                if quality >= 80 {
                    self.image_optimizer.optimize_with_quality(data, quality)
                } else {
                    #[cfg(c_hotspots_available)]
                    {
                        match crate::c_hotspots::compress_data_c_hotspot(data) {
                            Ok(compressed) if compressed.len() < data.len() => Ok(compressed),
                            _ => self.image_optimizer.optimize_with_quality(data, quality)
                        }
                    }
                    #[cfg(not(c_hotspots_available))]
                    {
                        self.image_optimizer.optimize_with_quality(data, quality)
                    }
                }
            },
            "JPEG" => {
                self.image_optimizer.optimize_with_quality(data, quality.min(85))
            },
            "WebP" => {
                if data.len() > 1_048_576 && quality < 60 {
                    self.image_optimizer.optimize_with_quality(data, quality)
                } else {
                    Ok(data.to_vec())
                }
            },
            "TIFF" => {
                #[cfg(c_hotspots_available)]
                {
                    if let Ok(result) = crate::c_hotspots::strip_tiff_metadata_c_hotspot(data, false) {
                        if result.len() < data.len() {
                            return Ok(result);
                        }
                    }
                }
                self.image_optimizer.optimize_with_quality(data, quality)
            },
            _ => {
                self.image_optimizer.optimize_with_quality(data, quality)
            }
        }
    }

    pub fn optimize_image(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let target_ms = if data_size < 1_048_576 { SMALL_FILE_TARGET_MS } else { IMAGE_TARGET_MS };
        
        if data_size > 2_097_152 { // > 2MB files need aggressive optimization
            match self.fast_path_image_optimize(data, quality, target_ms) {
                Ok(result) => {
                    let elapsed = get_current_time_ms() - start_time;
                    update_performance_stats(true, elapsed, data_size);
                    return Ok(result);
                },
                Err(_) => {
                }
            }
        }
        
        let base_result = self.image_optimizer.optimize_with_quality(data, quality);
        let elapsed = get_current_time_ms() - start_time;
        
        if elapsed > target_ms * 0.8 {
            update_performance_stats(true, elapsed, data_size);
            return base_result;
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = console)]
                fn log(s: &str);
            }
            
            let msg = format!("Image optimization took {:.1}ms for {:.1}KB", elapsed, data_size as f64 / 1024.0);
            log(&msg);
        }
        
        update_performance_stats(true, elapsed, data_size);
        
        match base_result {
            Ok(optimized) => {
                let remaining_time = target_ms - elapsed;
                if remaining_time > 20.0 {
                    #[cfg(c_hotspots_available)]
                    {
                        if data.len() > 100_000 && quality < 70 {
                            if let Ok(compressed) = crate::c_hotspots::compress_data_c_hotspot(&optimized) {
                                if compressed.len() < optimized.len().saturating_mul(90).saturating_div(100) {
                                    return Ok(compressed);
                                }
                            }
                        }
                    }
                }
                Ok(optimized)
            },
            Err(e) => Err(e),
        }
    }

    pub fn optimize_mesh(&self, data: &[u8]) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let base_result = self.mesh_optimizer.optimize(data);
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        match base_result {
            Ok(optimized) => {
                #[cfg(c_hotspots_available)]
                {
                    if data.len() > 50_000 {
                        if let Ok(compressed) = crate::c_hotspots::compress_data_c_hotspot(&optimized) {
                            if compressed.len() < optimized.len().saturating_mul(85).saturating_div(100) {
                                return Ok(compressed);
                            }
                        }
                    }
                }
                Ok(optimized)
            },
            Err(e) => Err(e)
        }
    }

    pub fn optimize_with_c_hotspots(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let optimized = self.optimize_auto(data, quality)?;
        
        #[cfg(c_hotspots_available)]
        {
            if data.len() > 100_000 && quality < 70 {
                if let Ok(compressed) = crate::c_hotspots::compress_data_c_hotspot(&optimized) {
                    if compressed.len() < optimized.len().saturating_mul(90).saturating_div(100) {
                        let elapsed = get_current_time_ms() - start_time;
                        update_performance_stats(true, elapsed, data_size);
                        return Ok(compressed);
                    }
                }
            }
        }
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        Ok(optimized)
    }

    pub fn optimize_with_performance_target(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let result = {
            let images = IMAGES_PROCESSED.load(Ordering::Relaxed);
            let meshes = MESHES_PROCESSED.load(Ordering::Relaxed);
            let total_ops = images + meshes;
        
            if total_ops > 100 {
                self.optimize_fast_strategy(data, quality)
            } else {
                self.optimize_auto(data, quality)
            }
        };
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        result
    }

    fn optimize_fast_strategy(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        use crate::formats::{detect_image_format, detect_mesh_format};
        
        if let Ok(_format) = detect_image_format(data) {
            self.image_optimizer.optimize_with_quality(data, quality.max(70))
        } else if let Ok(_format) = detect_mesh_format(data) {
            self.mesh_optimizer.optimize(data)
        } else {
            ERRORS_COUNT.fetch_add(1, Ordering::Relaxed);
            Err(PixieError::InvalidInput("Unknown file format".to_string()))
        }
    }

    pub fn optimize_streaming(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        const CHUNK_THRESHOLD: usize = 1024 * 1024;
        
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        if data.len() < CHUNK_THRESHOLD {
            let result = self.optimize_auto(data, quality);
            let elapsed = get_current_time_ms() - start_time;
            update_performance_stats(true, elapsed, data_size);
            return result;
        }
        
        use crate::formats::{detect_image_format, detect_mesh_format, ImageFormat};
        
        let result = match detect_image_format(data) {
            Ok(ImageFormat::Jpeg) => self.optimize_jpeg_progressive_streaming(data, quality),
            Ok(ImageFormat::Png) => self.optimize_png_chunked_processing(data, quality),
            Ok(_) => self.optimize_auto(data, quality),
            Err(_) => {
                if detect_mesh_format(data).is_ok() {
                    self.optimize_mesh(data)
                } else {
                    ERRORS_COUNT.fetch_add(1, Ordering::Relaxed);
                    Err(PixieError::InvalidInput("Unknown format for streaming".to_string()))
                }
            }
        };
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        result
    }

    fn optimize_jpeg_progressive_streaming(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        use image::codecs::jpeg::JpegEncoder;
        
        let img = image::load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load JPEG: {}", e)))?;
        
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        
        let mut encoder_buffer = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut encoder_buffer, quality);
        
        encoder.encode(&rgb_img, width, height, image::ExtendedColorType::Rgb8)
            .map_err(|e| PixieError::ProcessingError(format!("Progressive JPEG encoding failed: {}", e)))?;
        
        Ok(encoder_buffer)
    }

    fn optimize_png_chunked_processing(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        self.image_optimizer.optimize_with_quality(data, quality.max(60))
    }
}

impl Default for PixieOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_optimize_auto(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_auto(data, quality)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_optimize_image(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_image(data, quality)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_optimize_mesh(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_mesh(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_get_memory_target_mb() -> f64 {
    MEMORY_TARGET_MB
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_get_performance_stats() -> JsValue {
    let stats = get_performance_stats();
    serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_reset_performance_stats() {
    reset_performance_stats();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_check_performance_compliance() -> bool {
    check_performance_compliance()
}
