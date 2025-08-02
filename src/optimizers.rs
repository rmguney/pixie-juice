//! Main optimization entry points with hybrid Rust + C architecture
//! 
//! This module provides the main optimization interfaces that coordinate
//! between different format-specific optimizers and C hotspots.

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
fn get_current_time_ms() -> f64 {
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
            total_bytes_processed: 0,
            memory_peak_mb: 0.0,
            errors_count: 0,
            performance_target_violations: 0,
        }
    }
}

/// Global performance tracking (single-threaded WASM safe)
static mut PERF_STATS: PerformanceStats = PerformanceStats {
    images_processed: 0,
    meshes_processed: 0,
    avg_image_time_ms: 0.0,
    avg_mesh_time_ms: 0.0,
    max_image_time_ms: 0.0,
    max_mesh_time_ms: 0.0,
    total_bytes_processed: 0,
    memory_peak_mb: 0.0,
    errors_count: 0,
    performance_target_violations: 0,
};

/// CRITICAL Performance Targets (from TODO.md requirements)
const IMAGE_TARGET_MS: f64 = 100.0;  // <100ms for 1MB images (goal <50ms)
const MESH_TARGET_MS: f64 = 300.0;   // <300ms for 100k tris (goal <200ms)
const MEMORY_TARGET_MB: f64 = 100.0; // <100MB memory peak

/// Update performance statistics with timing and memory data
fn update_performance_stats(is_image: bool, elapsed_ms: f64, data_size: usize) {
    unsafe {
        PERF_STATS.total_bytes_processed += data_size as u64;
        
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
            
            // Check performance target compliance (CRITICAL requirement)
            let size_mb = data_size as f64 / 1_048_576.0;
            let target = if size_mb >= 1.0 { 
                IMAGE_TARGET_MS 
            } else { 
                IMAGE_TARGET_MS * 0.5 // Smaller files should be faster
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
            
            // Log memory violation
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::prelude::*;
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = console)]
                    fn warn(s: &str);
                }
                
                let msg = format!("MEMORY VIOLATION: Current usage {:.1}MB exceeds target {:.1}MB", 
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
    /// Create a new pixie optimizer with default configurations
    pub fn new() -> Self {
        Self {
            image_optimizer: ImageOptimizer::default(),
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
    pub fn optimize_auto(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let result = if let Ok(_format) = detect_image_format(data) {
            let image_result = self.image_optimizer.optimize_with_quality(data, quality);
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
        
        result
    }

    /// Optimize image data with performance monitoring
    pub fn optimize_image(&self, data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let result = self.image_optimizer.optimize_with_quality(data, quality);
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(true, elapsed, data_size);
        
        result
    }

    /// Optimize mesh data with performance monitoring
    pub fn optimize_mesh(&self, data: &[u8]) -> PixieResult<Vec<u8>> {
        let start_time = get_current_time_ms();
        let data_size = data.len();
        
        let result = self.mesh_optimizer.optimize(data);
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        result
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
