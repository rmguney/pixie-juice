//! AVIF format support with pure Rust fallback
//! 
//! This module provides AVIF format detection and basic optimization.
//! Advanced AVIF support requires C libraries not compatible with WASM.

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, PixieError, ImageOptConfig};

/// Optimize AVIF data with fallback approach for WASM compatibility
pub fn optimize_avif(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // First validate that this is actually AVIF
    if !is_avif_format(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid AVIF file".to_string()));
    }
    
    // For lossless mode with high quality, preserve original
    if config.lossless && quality > 90 {
        return Ok(data.to_vec());
    }
    
    // AVIF is already a highly optimized format
    // Real optimization would require C libraries that aren't WASM compatible
    // For now, implement conversion to more widely supported formats
    convert_avif_to_compressed(data, quality)
}

/// Check if data is AVIF format using magic bytes
fn is_avif_format(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    // AVIF signature: ftypavif
    data.len() >= 12 && 
    data[4..8] == [0x66, 0x74, 0x79, 0x70] && // "ftyp"
    data[8..12] == [0x61, 0x76, 0x69, 0x66]   // "avif"
}

/// Convert AVIF to more compressed format (JPEG/WebP) if beneficial
pub fn convert_avif_to_compressed(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // Since we can't decode AVIF without C dependencies in WASM,
    // we'll use a conservative approach for optimization
    
    // Check file size - if it's small enough, it's probably already well optimized
    if data.len() < 50_000 {
        return Ok(data.to_vec());
    }
    
    // For larger AVIF files, suggest conversion to JPEG for compatibility
    // but return original since we can't actually decode without C libraries
    
    // TODO: Implement pure Rust AVIF decoder when available
    // For now, return original data to preserve functionality
    let _ = quality; // Suppress unused warning
    
    // Log optimization attempt for debugging
    #[cfg(target_arch = "wasm32")]
    {
        log_to_console(&format!("AVIF optimization: {} bytes (preserved - no pure Rust decoder available)", data.len()));
    }
    
    Ok(data.to_vec())
}

/// Detect if AVIF has transparency for optimization decisions  
pub fn avif_has_transparency(_data: &[u8]) -> bool {
    // Without being able to decode AVIF, conservatively assume transparency
    // This ensures we don't make wrong optimization decisions
    true
}

/// Helper function for console logging in WASM
#[cfg(target_arch = "wasm32")]
fn log_to_console(msg: &str) {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }
    unsafe { log(msg); }
}

/// Placeholder for native builds
#[cfg(not(target_arch = "wasm32"))]
fn log_to_console(_msg: &str) {
    // No-op for native builds
}