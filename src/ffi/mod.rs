//! FFI module - placeholder for C integration

extern crate alloc;
use alloc::{vec::Vec, string::String, string::ToString};

// Placeholder - FFI integration will be implemented after basic Rust implementation works

#[derive(Debug, Clone)]
pub struct CodecInfo {
    pub version: String,
    pub features: Vec<String>,
    pub c_hotspots_enabled: bool,
    pub wasm_target: bool,
    pub threading_available: bool,
    pub codecs_available: Vec<String>,
}

/// Get optimization information (placeholder)
pub fn get_optimization_info() -> CodecInfo {
    CodecInfo {
        version: "0.1.0".to_string(),
        features: Vec::new(),
        c_hotspots_enabled: false,
        wasm_target: true,
        threading_available: false,
        codecs_available: Vec::new(),
    }
}
