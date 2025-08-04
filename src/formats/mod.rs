//! Format handling for different file types

pub mod image;
pub mod mesh;

pub use image::ImageFormat;
pub use mesh::MeshFormat;

// Re-export detection functions for convenience
pub use image::detect_image_format;
pub use crate::mesh::detect_mesh_format;
