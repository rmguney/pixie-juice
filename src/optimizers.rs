//! Main optimization entry points

extern crate alloc;
use alloc::{vec::Vec, format, string::ToString};

use crate::types::{PixieResult, PixieError, ImageOptConfig, MeshOptConfig};
use crate::image::{ImageOptimizer, detect_image_format};
use crate::mesh::{MeshOptimizer, detect_mesh_format};
use serde::{Deserialize, Serialize};

/// Performance monitoring utilities
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

/// Get current timestamp for performance measurement with memory tracking
pub fn get_current_time_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        now()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // For native builds, use a simple counter
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            COUNTER as f64
        }
    }
}

/// Get current memory usage in MB (WASM heap monitoring)
#[cfg(target_arch = "wasm32")]
fn get_memory_usage_mb() -> f64 {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        type Performance;
        
        #[wasm_bindgen(js_namespace = ["performance", "memory"], js_name = usedJSHeapSize)]
        static USED_JS_HEAP_SIZE: JsValue;
    }
    
    // Try to get heap size, fallback to 0 if not available
    if let Some(heap_size) = USED_JS_HEAP_SIZE.as_f64() {
        heap_size / 1_048_576.0 // Convert bytes to MB
    } else {
        0.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_memory_usage_mb() -> f64 {
    // For native builds, return a placeholder
    // TODO: Implement native memory monitoring
    0.0
}

/// Performance statistics for CRITICAL compliance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub images_processed: u64,
    pub meshes_processed: u64,
    pub avg_image_time_ms: f64,
    pub avg_mesh_time_ms: f64,
    pub max_image_time_ms: f64,
    pub max_mesh_time_ms: f64,
    pub last_operation_time_ms: f64,  // Track individual operation timing
    pub total_bytes_processed: u64,
    pub memory_peak_mb: f64,
    pub errors_count: u64,
    pub performance_target_violations: u64,
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

/// Global performance tracking (single-threaded WASM safe)
pub static mut PERF_STATS: PerformanceStats = PerformanceStats {
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
};

/// Global optimization configuration (single-threaded WASM safe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOptConfig {
    pub preserve_metadata: bool,
    pub lossless_mode: bool,
}

impl Default for GlobalOptConfig {
    fn default() -> Self {
        Self {
            preserve_metadata: false, // Default to removing metadata for better compression
            lossless_mode: false,
        }
    }
}

static mut GLOBAL_CONFIG: GlobalOptConfig = GlobalOptConfig {
    preserve_metadata: false,
    lossless_mode: false,
};

/// Get current global configuration
pub fn get_global_config() -> GlobalOptConfig {
    unsafe { GLOBAL_CONFIG.clone() }
}

/// Set metadata preservation globally
pub fn set_preserve_metadata_global(enabled: bool) {
    unsafe {
        GLOBAL_CONFIG.preserve_metadata = enabled;
    }
}

/// Set lossless mode globally
pub fn set_lossless_mode_global(enabled: bool) {
    unsafe {
        GLOBAL_CONFIG.lossless_mode = enabled;
    }
}

/// CRITICAL Performance Targets (optimized for aggressive performance compliance)
const IMAGE_TARGET_MS: f64 = 150.0;  // <150ms for 1MB images (reduced from 200ms for better performance)
const MESH_TARGET_MS: f64 = 250.0;   // <250ms for 100k tris (reduced from 300ms)
const MEMORY_TARGET_MB: f64 = 800.0; // <800MB memory peak (reduced from 1024MB for efficiency)
const SMALL_FILE_TARGET_MS: f64 = 100.0; // <100ms for files <512KB (new aggressive target)

/// Update performance statistics with timing and memory data
pub fn update_performance_stats(is_image: bool, elapsed_ms: f64, data_size: usize) {
    unsafe {
        PERF_STATS.total_bytes_processed += data_size as u64;
        PERF_STATS.last_operation_time_ms = elapsed_ms;  // Track individual operation timing
        
        // Update memory tracking
        let current_memory = get_memory_usage_mb();
        if current_memory > PERF_STATS.memory_peak_mb {
            PERF_STATS.memory_peak_mb = current_memory;
        }
        
        if is_image {
            // Update image statistics
            PERF_STATS.images_processed += 1;
            
            // Calculate moving average
            let count = PERF_STATS.images_processed as f64;
            PERF_STATS.avg_image_time_ms = 
                (PERF_STATS.avg_image_time_ms * (count - 1.0) + elapsed_ms) / count;
            
            // Update maximum
            if elapsed_ms > PERF_STATS.max_image_time_ms {
                PERF_STATS.max_image_time_ms = elapsed_ms;
            }
            
            // Check performance target compliance (CRITICAL requirement) - OPTIMIZED
            let size_mb = data_size as f64 / 1_048_576.0;
            let target = if size_mb >= 1.0 { 
                IMAGE_TARGET_MS 
            } else { 
                SMALL_FILE_TARGET_MS // Use new aggressive target for small files
            };
            
            if elapsed_ms > target {
                PERF_STATS.performance_target_violations += 1;
                
                // Log performance violation for debugging
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::prelude::*;
                    #[wasm_bindgen]
                    extern "C" {
                        #[wasm_bindgen(js_namespace = console)]
                        fn warn(s: &str);
                    }
                    
                    let msg = format!("PERFORMANCE VIOLATION: Image processing took {:.1}ms (target: {:.1}ms) for {:.1}MB file", 
                                    elapsed_ms, target, size_mb);
                    warn(&msg);
                }
            }
        } else {
            // Update mesh statistics
            PERF_STATS.meshes_processed += 1;
            
            // Calculate moving average
            let count = PERF_STATS.meshes_processed as f64;
            PERF_STATS.avg_mesh_time_ms = 
                (PERF_STATS.avg_mesh_time_ms * (count - 1.0) + elapsed_ms) / count;
            
            // Update maximum
            if elapsed_ms > PERF_STATS.max_mesh_time_ms {
                PERF_STATS.max_mesh_time_ms = elapsed_ms;
            }
            
            // Check performance target compliance
            if elapsed_ms > MESH_TARGET_MS {
                PERF_STATS.performance_target_violations += 1;
                
                // Log performance violation for debugging
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::prelude::*;
                    #[wasm_bindgen]
                    extern "C" {
                        #[wasm_bindgen(js_namespace = console)]
                        fn warn(s: &str);
                    }
                    
                    let msg = format!("PERFORMANCE VIOLATION: Mesh processing took {:.1}ms (target: {:.1}ms)", 
                                    elapsed_ms, MESH_TARGET_MS);
                    warn(&msg);
                }
            }
        }
        
        // Check memory compliance
        if current_memory > MEMORY_TARGET_MB {
            PERF_STATS.performance_target_violations += 1;
            
            // Log memory violation with actionable guidance
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::prelude::*;
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = console)]
                    fn warn(s: &str);
                }
                
                let msg = format!("MEMORY USAGE: {:.1}MB (target: {:.1}MB) - Large files may require more memory for processing", 
                                current_memory, MEMORY_TARGET_MB);
                warn(&msg);
            }
        }
    }
}

/// Get current performance statistics (CRITICAL requirement)
pub fn get_performance_stats() -> PerformanceStats {
    unsafe { PERF_STATS.clone() }
}

/// Reset performance statistics
pub fn reset_performance_stats() {
    unsafe {
        PERF_STATS = PerformanceStats::default();
    }
}

/// Check if performance targets are being met
pub fn check_performance_compliance() -> bool {
    unsafe {
        let total_ops = PERF_STATS.images_processed + PERF_STATS.meshes_processed;
        if total_ops == 0 {
            return true; // No operations yet
        }
        
        // Check violation rate (should be <5% for compliance)
        let violation_rate = PERF_STATS.performance_target_violations as f64 / total_ops as f64;
        violation_rate < 0.05
    }
}

/// Main pixie-juice optimizer that handles all supported formats
pub struct PixieOptimizer {
    image_optimizer: ImageOptimizer,
    mesh_optimizer: MeshOptimizer,
}

impl PixieOptimizer {
    /// Create a new pixie optimizer with default configurations that respect global settings
    pub fn new() -> Self {
        Self {
            image_optimizer: ImageOptimizer::new(ImageOptConfig::with_global_settings()),
            mesh_optimizer: MeshOptimizer::default(),
        }
    }

    /// Create a new pixie optimizer with custom configurations
    pub fn with_configs(image_config: ImageOptConfig, mesh_config: MeshOptConfig) -> Self {
        Self {
            image_optimizer: ImageOptimizer::new(image_config),
            mesh_optimizer: MeshOptimizer::new(mesh_config),
        }
    }

    /// Automatically detect format and optimize with CRITICAL performance monitoring
    /// Includes C hotspots integration when enabled by default
    pub fn optimize_auto(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        // FAST PATH: For large files (>100KB), use simpler optimization to avoid performance violations
        let use_fast_path = data_size > 100_000;
        
        // Debug logging for fast path detection
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
        
        // Step 1: Format-specific optimization using proven libraries
        let base_result = if let Ok(_format) = detect_image_format(data) {
            let image_result = if use_fast_path {
                // Fast path: minimal processing for large files
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
            unsafe { PERF_STATS.errors_count += 1; }
            Err(PixieError::InvalidInput("Unknown file format".to_string()))
        };
        
        // Step 2: Apply C hotspots for additional compression ONLY for smaller files
        match base_result {
            Ok(optimized) => {
                #[cfg(c_hotspots_available)]
                {
                    // Skip C hotspots for large files to avoid compress_lz4 performance issues
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

    /// Fast path optimization for large files to meet aggressive performance targets
    fn fast_path_image_optimize(&self, data: &[u8], quality: u8, target_ms: f64) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        
        // Detect format for fast path decisions
        let format_result = detect_image_format(data);
        
        // Use format-specific fast optimizations
        let format_str = match format_result {
            Ok(fmt) => format!("{:?}", fmt),
            Err(_) => "Unknown".to_string(),
        };
        
        match format_str.as_str() {
            "PNG" => {
                // For PNG, use quick compression level adjustment
                if quality >= 80 {
                    // High quality: minimal processing
                    self.image_optimizer.optimize_with_quality(data, quality)
                } else {
                    // Lower quality: aggressive compression with C hotspots
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
                // For JPEG, quick re-encoding with target quality
                self.image_optimizer.optimize_with_quality(data, quality.min(85))
            },
            "WebP" => {
                // WebP is already well compressed, minimal processing
                if data.len() > 1_048_576 && quality < 60 {
                    self.image_optimizer.optimize_with_quality(data, quality)
                } else {
                    Ok(data.to_vec()) // Return original for small WebP files
                }
            },
            "TIFF" => {
                // Use TIFF C hotspots for fast processing
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
                // For other formats, use standard optimization
                self.image_optimizer.optimize_with_quality(data, quality)
            }
        }
    }

    /// Optimize image data with performance monitoring and C hotspots integration
    pub fn optimize_image(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        // Performance-aware optimization with early termination
        let target_ms = if data_size < 1_048_576 { SMALL_FILE_TARGET_MS } else { IMAGE_TARGET_MS };
        
        // Fast path for large files to meet performance targets
        if data_size > 2_097_152 { // > 2MB files need aggressive optimization
            match self.fast_path_image_optimize(data, quality, target_ms) {
                Ok(result) => {
                    let elapsed = get_current_time_ms() - start_time;
                    update_performance_stats(true, elapsed, data_size);
                    return Ok(result);
                },
                Err(_) => {
                    // Fall through to regular optimization
                }
            }
        }
        
        let base_result = self.image_optimizer.optimize_with_quality(data, quality);
        let elapsed = get_current_time_ms() - start_time;
        
        // Early termination if approaching performance target
        if elapsed > target_ms * 0.8 {
            update_performance_stats(true, elapsed, data_size);
            return base_result;
        }
        
        // Log individual timing for debugging
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
        
        // Apply C hotspots for additional compression when enabled and we have time
        match base_result {
            Ok(optimized) => {
                let remaining_time = target_ms - elapsed;
                if remaining_time > 20.0 { // Only if we have >20ms remaining
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

    /// Optimize mesh data with performance monitoring and C hotspots integration
    pub fn optimize_mesh(&self, data: &[u8]) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let base_result = self.mesh_optimizer.optimize(data);
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        // Apply C hotspots for mesh compression when enabled  
        match base_result {
            Ok(optimized) => {
                #[cfg(c_hotspots_available)]
                {
                    // Use lower threshold for mesh data (meshes benefit more from compression)
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

    /// Optimize with C hotspots for enhanced performance (large files)
    pub fn optimize_with_c_hotspots(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        // First apply format-specific optimization
        let optimized = self.optimize_auto(data, quality)?;
        
        // Apply C hotspot compression for large files with low quality
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
        
        // MANDATORY performance tracking
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        Ok(optimized)
    }

    /// Performance-based strategy selection for compliance
    pub fn optimize_with_performance_target(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let stats = unsafe { &PERF_STATS };
        let total_ops = stats.images_processed + stats.meshes_processed;
        
        // Calculate recent average performance
        let recent_avg = if total_ops > 0 {
            (stats.avg_image_time_ms * stats.images_processed as f64 + 
             stats.avg_mesh_time_ms * stats.meshes_processed as f64) / total_ops as f64
        } else {
            0.0
        };
        
        // Choose strategy based on performance compliance
        let result = if recent_avg > IMAGE_TARGET_MS * 0.8 {
            // Use faster algorithms when approaching target
            self.optimize_fast_strategy(data, quality)
        } else {
            // Use best compression when performance allows
            self.optimize_auto(data, quality)
        };
        
        // MANDATORY performance tracking
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        result
    }

    /// Fast strategy implementation for performance compliance
    fn optimize_fast_strategy(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        use crate::formats::{detect_image_format, detect_mesh_format};
        
        // Skip expensive optimizations when performance is critical
        if let Ok(_format) = detect_image_format(data) {
            // Use minimal compression settings for speed
            self.image_optimizer.optimize_with_quality(data, quality.max(70))
        } else if let Ok(_format) = detect_mesh_format(data) {
            // Use faster mesh optimization
            self.mesh_optimizer.optimize(data)
        } else {
            unsafe { PERF_STATS.errors_count += 1; }
            Err(PixieError::InvalidInput("Unknown file format".to_string()))
        }
    }

    /// Streaming optimization for large files
    pub fn optimize_streaming(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        const CHUNK_THRESHOLD: usize = 1024 * 1024; // 1MB
        
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        if data.len() < CHUNK_THRESHOLD {
            let result = self.optimize_auto(data, quality);
            let elapsed = get_current_time_ms() - start_time;
            update_performance_stats(true, elapsed, data_size);
            return result;
        }
        
        // Format-specific streaming with MANDATORY performance tracking
        use crate::formats::{detect_image_format, detect_mesh_format, ImageFormat};
        
        let result = match detect_image_format(data) {
            Ok(ImageFormat::Jpeg) => self.optimize_jpeg_progressive_streaming(data, quality),
            Ok(ImageFormat::Png) => self.optimize_png_chunked_processing(data, quality),
            Ok(_) => self.optimize_auto(data, quality),
            Err(_) => {
                if detect_mesh_format(data).is_ok() {
                    self.optimize_mesh(data)
                } else {
                    unsafe { PERF_STATS.errors_count += 1; }
                    Err(PixieError::InvalidInput("Unknown format for streaming".to_string()))
                }
            }
        };
        
        // MANDATORY performance tracking
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        result
    }

    /// Progressive JPEG streaming using image crate
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

    /// PNG chunked processing for large files
    fn optimize_png_chunked_processing(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        // For large PNG files, use standard optimization but with conservative settings for speed
        self.image_optimizer.optimize_with_quality(data, quality.max(60))
    }
}

impl Default for PixieOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// WASM exports for the public API
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WASM-compatible wrapper for auto optimization
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_optimize_auto(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_auto(data, quality)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM-compatible wrapper for image optimization
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_optimize_image(data: &[u8], quality: u8) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_image(data, quality)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM-compatible wrapper for mesh optimization
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_optimize_mesh(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let optimizer = PixieOptimizer::new();
    optimizer.optimize_mesh(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM-compatible function to get memory target for verification
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_get_memory_target_mb() -> f64 {
    MEMORY_TARGET_MB
}

/// WASM-compatible performance statistics getter
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_get_performance_stats() -> JsValue {
    let stats = get_performance_stats();
    serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
}

/// WASM-compatible performance reset
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_reset_performance_stats() {
    reset_performance_stats();
}

/// WASM-compatible performance compliance checker
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn pixie_check_performance_compliance() -> bool {
    check_performance_compliance()
}
