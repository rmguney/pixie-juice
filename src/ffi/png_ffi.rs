/*
 * PNG Optimization FFI - Rust interface to C PNG optimization
 */

use crate::types::{OptConfig, OptError, OptResult};

#[cfg(c_hotspots_available)]
mod c_bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(c_hotspots_available)]
use c_bindings::{PngOptConfig, PngOptResult, png_optimize_c, png_opt_result_free, png_has_alpha_channel};

#[cfg(feature = "wasm")]
macro_rules! wasm_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}

#[cfg(not(feature = "wasm"))]
macro_rules! wasm_log {
    ($($t:tt)*) => {
        println!($($t)*);
    };
}

pub fn optimize_png_with_c(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        wasm_log!("🔧 Using C PNG optimization with SIMD acceleration");
        
        // Check if PNG has alpha channel before optimization
        let has_alpha = unsafe { png_has_alpha_channel(data.as_ptr(), data.len()) };
        wasm_log!("🔍 PNG alpha channel detection: {}", if has_alpha != 0 { "HAS ALPHA" } else { "NO ALPHA" });
        
        let c_config = PngOptConfig {
            compress_level: (config.quality.unwrap_or(85) / 10) as i32, // Convert 0-100 to 0-9
            reduce_colors: if config.reduce_colors.unwrap_or(false) { 1 } else { 0 },
            max_colors: 256,
            strip_metadata: 1, // Always strip metadata for smaller files
            optimize_filters: 1, // Always optimize filters
        };
        
        wasm_log!("📊 C PNG config: compress_level={}, reduce_colors={}, max_colors={}, alpha_detected={}", 
                  c_config.compress_level, c_config.reduce_colors, c_config.max_colors, has_alpha != 0);
        
        let result = unsafe { 
            png_optimize_c(data.as_ptr(), data.len(), &c_config as *const PngOptConfig)
        };
        
        if result.error_code == 0 && !result.output_data.is_null() && result.output_size > 0 {
            // Copy the optimized data to a Rust Vec
            let optimized_data = unsafe {
                std::slice::from_raw_parts(result.output_data, result.output_size).to_vec()
            };
            
            // Free the C-allocated memory
            let mut result_mut = result;
            unsafe { png_opt_result_free(&mut result_mut as *mut PngOptResult); }
            
            let reduction = 1.0 - (optimized_data.len() as f64 / data.len() as f64);
            wasm_log!("✅ C PNG optimization successful: {} → {} bytes ({:.1}% reduction)", 
                      data.len(), optimized_data.len(), reduction * 100.0);
            
            Ok(optimized_data)
        } else {
            // Extract error message
            let error_msg = if !result.error_message.is_empty() {
                let c_str = std::ffi::CStr::from_bytes_until_nul(&result.error_message)
                    .unwrap_or(std::ffi::CStr::from_bytes_with_nul(b"Unknown error\0").unwrap());
                c_str.to_string_lossy().to_string()
            } else {
                format!("C PNG optimization failed with code {}", result.error_code)
            };
            
            wasm_log!("❌ C PNG optimization failed: {}", error_msg);
            
            // Free any allocated memory
            if !result.output_data.is_null() {
                let mut result_mut = result;
                unsafe { png_opt_result_free(&mut result_mut as *mut PngOptResult); }
            }
            
            Err(OptError::ProcessingError(error_msg))
        }
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        wasm_log!("⚠️ C PNG optimization not available, falling back to Rust implementation");
        Err(OptError::ProcessingError("C hotspots not available".to_string()))
    }
}

// Test function to verify C PNG optimization is working
pub fn test_c_png_optimization() -> bool {
    #[cfg(c_hotspots_available)]
    {
        // Create a minimal PNG test case
        let test_png = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            b'I', b'H', b'D', b'R', // IHDR type
            0x00, 0x00, 0x00, 0x01, // Width: 1
            0x00, 0x00, 0x00, 0x01, // Height: 1
            0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
            0x90, 0x77, 0x53, 0xDE, // CRC
            0x00, 0x00, 0x00, 0x00, // IEND length
            b'I', b'E', b'N', b'D', // IEND type
            0xAE, 0x42, 0x60, 0x82, // IEND CRC
        ];
        
        let config = OptConfig {
            quality: Some(80),
            reduce_colors: Some(true),
            lossless: Some(false),
            compression_level: Some(6),
            fast_mode: Some(false),
            max_width: Some(4096),
            max_height: Some(4096),
            target_reduction: Some(0.2),
            preserve_metadata: Some(false),
            preserve_alpha: Some(true),
        };
        
        match optimize_png_with_c(&test_png, &config) {
            Ok(_) => {
                wasm_log!("✅ C PNG optimization test passed");
                true
            }
            Err(e) => {
                wasm_log!("❌ C PNG optimization test failed: {:?}", e);
                false
            }
        }
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        false
    }
}
