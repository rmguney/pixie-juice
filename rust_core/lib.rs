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

// Conditional modules based on target platform
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod user_feedback;

#[cfg(target_arch = "wasm32")]
#[path = "config_wasm.rs"]
pub mod config;
#[cfg(target_arch = "wasm32")]
#[path = "user_feedback_wasm.rs"]
pub mod user_feedback;

// Re-export types for convenience
pub use types::{OptConfig, OptError, OptResult};
pub use common::{OptimizationOptions, ProcessingResult};
pub use config::PixieConfig;
pub use user_feedback::UserFeedback;

// Re-export optimizer structs
pub use mesh::MeshOptimizer;
pub use optimizers::VideoOptimizer;

// Re-export new format-based optimizers
pub use image::ImageOptimizer;
