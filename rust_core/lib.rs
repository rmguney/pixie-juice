//! Pixie Juice Core Library
//! 
//! This crate provides Rust-first optimization logic for both CLI and WASM targets.
//! It uses proven Rust crates for most processing with C hotspots only where needed.

pub mod common;
pub mod ffi;
pub mod formats;
pub mod image;
pub mod mesh;
pub mod optimizers;
pub mod types;
pub mod video;

// Re-export types for convenience
pub use types::{OptConfig, OptError, OptResult};
pub use common::{OptimizationOptions, ProcessingResult};

// Re-export optimizer structs (legacy compatibility)
pub use optimizers::{MeshOptimizer, VideoOptimizer};

// Re-export new format-based optimizers
pub use image::ImageOptimizer;
