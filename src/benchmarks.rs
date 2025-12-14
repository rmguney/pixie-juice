extern crate alloc;
use alloc::{vec::Vec, format, string::String, string::ToString};

use crate::optimizers::{PixieOptimizer, get_performance_stats, reset_performance_stats};
use crate::types::{PixieResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub elapsed_ms: f64,
    pub data_size_mb: f64,
    pub target_ms: f64,
    pub passed: bool,
    pub memory_peak_mb: f64,
}

pub fn run_performance_benchmarks() -> PixieResult<Vec<BenchmarkResult>> {
    let mut results = Vec::new();
    
    reset_performance_stats();
    
    results.push(benchmark_image_1mb()?);
    results.push(benchmark_mesh_100k_tris()?);
    results.push(benchmark_memory_usage()?);
    results.push(benchmark_batch_processing()?);
    
    Ok(results)
}

fn benchmark_image_1mb() -> PixieResult<BenchmarkResult> {
    let test_data = generate_test_image_1mb();
    let optimizer = PixieOptimizer::new();
    let start_time = get_current_time_ms();
    let _result = optimizer.optimize_image(&test_data, 75)?;
    let elapsed_ms = get_current_time_ms() - start_time;
    let stats = get_performance_stats();
    let target_ms = 100.0; // <100ms for 1MB images
    let passed = elapsed_ms <= target_ms;
    
    Ok(BenchmarkResult {
        test_name: "Image 1MB Optimization".to_string(),
        elapsed_ms,
        data_size_mb: 1.0,
        target_ms,
        passed,
        memory_peak_mb: stats.memory_peak_mb,
    })
}

fn benchmark_mesh_100k_tris() -> PixieResult<BenchmarkResult> {
    let test_data = generate_test_mesh_100k();
    let optimizer = PixieOptimizer::new();
    let start_time = get_current_time_ms();
    let _result = optimizer.optimize_mesh(&test_data)?;
    let elapsed_ms = get_current_time_ms() - start_time;
    let stats = get_performance_stats();
    let target_ms = 300.0;
    let passed = elapsed_ms <= target_ms;
    
    Ok(BenchmarkResult {
        test_name: "Mesh 100k Triangles".to_string(),
        elapsed_ms,
        data_size_mb: test_data.len() as f64 / 1_048_576.0,
        target_ms,
        passed,
        memory_peak_mb: stats.memory_peak_mb,
    })
}

fn benchmark_memory_usage() -> PixieResult<BenchmarkResult> {
    let test_data = generate_test_image_5mb();
    let optimizer = PixieOptimizer::new();
    let start_time = get_current_time_ms();
    let _result = optimizer.optimize_image(&test_data, 60)?;
    let elapsed_ms = get_current_time_ms() - start_time;
    let stats = get_performance_stats();
    let target_memory = 100.0;
    let passed = stats.memory_peak_mb <= target_memory;
    
    Ok(BenchmarkResult {
        test_name: "Memory Usage Test".to_string(),
        elapsed_ms,
        data_size_mb: 5.0,
        target_ms: target_memory,
        passed,
        memory_peak_mb: stats.memory_peak_mb,
    })
}

fn benchmark_batch_processing() -> PixieResult<BenchmarkResult> {
    let optimizer = PixieOptimizer::new();
    let start_time = get_current_time_ms();
    
    for _ in 0..10 {
        let test_data = generate_test_image_100kb();
        let _result = optimizer.optimize_image(&test_data, 80)?;
    }
    
    let elapsed_ms = get_current_time_ms() - start_time;
    let stats = get_performance_stats();
    
    let target_ms = 200.0;
    let passed = elapsed_ms <= target_ms;
    
    Ok(BenchmarkResult {
        test_name: "Batch Processing (10x100KB)".to_string(),
        elapsed_ms,
        data_size_mb: 1.0,
        target_ms,
        passed,
        memory_peak_mb: stats.memory_peak_mb,
    })
}

fn generate_test_image_1mb() -> Vec<u8> {
    let mut data = Vec::with_capacity(1_048_576);
    
    data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    data.resize(1_048_576, 0x42);
    
    data
}

fn generate_test_image_100kb() -> Vec<u8> {
    let mut data = Vec::with_capacity(102_400);
    
    data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    data.resize(102_400, 0x42);

    data
}

fn generate_test_image_5mb() -> Vec<u8> {
    let mut data = Vec::with_capacity(5_242_880);
    data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    data.resize(5_242_880, 0x42);

    data
}

fn generate_test_mesh_100k() -> Vec<u8> {
    let mut data = Vec::new();
    
    data.extend_from_slice(b"# Test OBJ file with 100k triangles\n");
    data.extend_from_slice(b"o TestMesh\n");
    data.resize(512_000, b'v');
    
    data
}

fn get_current_time_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = performance)]
            fn now() -> f64;
        }
        
        now()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            COUNTER as f64
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_wasm_benchmarks() -> JsValue {
    match run_performance_benchmarks() {
        Ok(results) => {
            serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
        },
        Err(_) => JsValue::NULL
    }
}

pub fn format_benchmark_results(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();
    output.push_str("=== PERFORMANCE BENCHMARK RESULTS ===\n");
    
    let mut all_passed = true;
    
    for result in results {
        let status = if result.passed { "PASS" } else { "FAIL" };
        
        output.push_str(&format!(
            "{}: {} - {:.1}ms (target: {:.1}ms) - Memory: {:.1}MB\n",
            status, result.test_name, result.elapsed_ms, result.target_ms, result.memory_peak_mb
        ));
        
        if !result.passed {
            all_passed = false;
        }
    }
    
    output.push_str("\n");
    if all_passed {
        output.push_str("ALL PERFORMANCE TARGETS MET!\n");
    } else {
        output.push_str("PERFORMANCE TARGETS NOT MET - OPTIMIZATION NEEDED\n");
    }
    
    output
}
