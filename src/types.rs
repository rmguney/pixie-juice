//! Core types and data structures

extern crate alloc;
use alloc::{vec::Vec, string::String, format};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use core::fmt;

/// Main result type for Pixie Juice operations
pub type PixieResult<T> = Result<T, PixieError>;

/// Legacy alias for backward compatibility
pub type OptResult<T> = PixieResult<T>;

/// Comprehensive error type for all Pixie Juice operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PixieError {
    /// Image processing errors
    InvalidImageFormat(String),
    ImageDecodingFailed(String),
    ImageEncodingFailed(String),
    UnsupportedImageFeature(String),
    
    /// Mesh processing errors
    InvalidMeshFormat(String),
    MeshLoadingFailed(String),
    MeshOptimizationFailed(String),
    GeometryValidationFailed(String),
    
    /// Compression errors
    CompressionFailed(String),
    DecompressionFailed(String),
    InvalidCompressionLevel(u8),
    
    /// Memory and I/O errors
    InsufficientMemory(String),
    BufferOverflow(String),
    InvalidBufferSize(usize),
    IoError(String),
    
    /// Configuration errors
    InvalidConfiguration(String),
    FeatureNotAvailable(String),
    FeatureNotEnabled(String),
    UnsupportedFormat(String),
    
    /// C hotspot errors
    CHotspotFailed(String),
    CHotspotUnavailable(String),
    CHotspotError(String),
    
    /// Threading errors
    ThreadingError(String),
    
    /// Generic processing error
    ProcessingError(String),
    
    /// Legacy error types for compatibility
    InvalidInput(String),
    InvalidFormat(String),
    OptimizationFailed(String),
    Memory(String),
    FormatError(String),
    XmlError(String),
    
    /// WebAssembly errors
    WebAssemblyError(String),
}

impl From<wasm_bindgen::JsValue> for PixieError {
    fn from(err: wasm_bindgen::JsValue) -> Self {
        PixieError::WebAssemblyError(format!("JavaScript error: {:?}", err))
    }
}

impl fmt::Display for PixieError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixieError::InvalidImageFormat(msg) => write!(f, "Invalid image format: {}", msg),
            PixieError::ImageDecodingFailed(msg) => write!(f, "Image decoding failed: {}", msg),
            PixieError::ImageEncodingFailed(msg) => write!(f, "Image encoding failed: {}", msg),
            PixieError::UnsupportedImageFeature(msg) => write!(f, "Unsupported image feature: {}", msg),
            PixieError::InvalidMeshFormat(msg) => write!(f, "Invalid mesh format: {}", msg),
            PixieError::MeshLoadingFailed(msg) => write!(f, "Mesh loading failed: {}", msg),
            PixieError::MeshOptimizationFailed(msg) => write!(f, "Mesh optimization failed: {}", msg),
            PixieError::GeometryValidationFailed(msg) => write!(f, "Geometry validation failed: {}", msg),
            PixieError::CompressionFailed(msg) => write!(f, "Compression failed: {}", msg),
            PixieError::DecompressionFailed(msg) => write!(f, "Decompression failed: {}", msg),
            PixieError::InvalidCompressionLevel(level) => write!(f, "Invalid compression level: {}", level),
            PixieError::InsufficientMemory(msg) => write!(f, "Insufficient memory: {}", msg),
            PixieError::BufferOverflow(msg) => write!(f, "Buffer overflow: {}", msg),
            PixieError::InvalidBufferSize(size) => write!(f, "Invalid buffer size: {}", size),
            PixieError::IoError(msg) => write!(f, "I/O error: {}", msg),
            PixieError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            PixieError::FeatureNotAvailable(msg) => write!(f, "Feature not available: {}", msg),
            PixieError::FeatureNotEnabled(msg) => write!(f, "Feature not enabled: {}", msg),
            PixieError::CHotspotFailed(msg) => write!(f, "C hotspot failed: {}", msg),
            PixieError::CHotspotUnavailable(msg) => write!(f, "C hotspot unavailable: {}", msg),
            PixieError::CHotspotError(msg) => write!(f, "C hotspot error: {}", msg),
            PixieError::ThreadingError(msg) => write!(f, "Threading error: {}", msg),
            PixieError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            PixieError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            PixieError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            PixieError::OptimizationFailed(msg) => write!(f, "Optimization failed: {}", msg),
            PixieError::Memory(msg) => write!(f, "Memory error: {}", msg),
            PixieError::FormatError(msg) => write!(f, "Format error: {}", msg),
            PixieError::XmlError(msg) => write!(f, "XML error: {}", msg),
            PixieError::WebAssemblyError(msg) => write!(f, "WebAssembly error: {}", msg),
            PixieError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
        }
    }
}

// Legacy error type alias
pub type OptError = PixieError;

/// Configuration for image optimization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct ImageOptConfig {
    pub quality: u8,
    pub lossless: bool,
    pub preserve_metadata: bool,
    pub optimize_colors: bool,
    pub max_colors: Option<u16>,
    pub use_c_hotspots: bool,
    pub enable_simd: bool,
    pub compression_level: Option<u8>,
    pub fast_mode: bool,
    pub preserve_alpha: bool,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub target_reduction: Option<f32>,
}

impl Default for ImageOptConfig {
    fn default() -> Self {
        // Get global configuration for metadata preservation
        let global_config = crate::optimizers::get_global_config();
        
        Self {
            quality: 85,
            lossless: global_config.lossless_mode,
            preserve_metadata: global_config.preserve_metadata,
            optimize_colors: true,
            max_colors: Some(256),
            use_c_hotspots: true,
            enable_simd: true,
            compression_level: Some(6),
            fast_mode: false,
            preserve_alpha: true,
            max_width: None,
            max_height: None,
            target_reduction: None,
        }
    }
}

impl ImageOptConfig {
    /// Create a new ImageOptConfig with current global settings applied
    pub fn with_global_settings() -> Self {
        Self::default()
    }
    
    /// Create a new ImageOptConfig with custom quality but respecting global settings
    pub fn with_quality(quality: u8) -> Self {
        let mut config = Self::default();
        config.quality = quality.clamp(1, 100);
        config
    }
}

/// Configuration for mesh optimization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct MeshOptConfig {
    pub target_ratio: f32,
    pub preserve_topology: bool,
    pub weld_vertices: bool,
    pub vertex_tolerance: f32,
    pub simplification_algorithm: SimplificationAlgorithm,
    pub use_c_hotspots: bool,
    pub generate_normals: bool,
    pub optimize_vertex_cache: bool,
    pub preserve_uv_seams: bool,
    pub preserve_boundaries: bool,
}

impl Default for MeshOptConfig {
    fn default() -> Self {
        Self {
            target_ratio: 0.5,
            preserve_topology: true,
            weld_vertices: true,
            vertex_tolerance: 1e-6,
            simplification_algorithm: SimplificationAlgorithm::QuadricErrorMetrics,
            use_c_hotspots: true,
            generate_normals: false,
            optimize_vertex_cache: true,
            preserve_uv_seams: true,
            preserve_boundaries: true,
        }
    }
}

/// Mesh simplification algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum SimplificationAlgorithm {
    /// Quadric Error Metrics - highest quality
    QuadricErrorMetrics,
    /// Edge collapse - good balance of speed and quality
    EdgeCollapse,
    /// Vertex clustering - fastest but lower quality
    VertexClustering,
}

/// Color representation for C FFI compatibility
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color32 {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }
}

/// Legacy configuration type for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptConfig {
    // Legacy flat structure for backward compatibility
    pub quality: Option<u8>,
    pub compression_level: Option<u8>,
    pub lossless: Option<bool>,
    pub preserve_metadata: Option<bool>,
    pub fast_mode: Option<bool>,
    pub reduce_colors: Option<bool>,
    pub target_reduction: Option<f32>,
    pub preserve_alpha: Option<bool>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    
    // New structured configs
    pub image: ImageOptConfig,
    pub mesh: MeshOptConfig,
    pub enable_threading: bool,
    pub use_zero_copy: bool,
}

impl Default for OptConfig {
    fn default() -> Self {
        Self {
            quality: Some(85),
            compression_level: Some(6),
            lossless: Some(false),
            preserve_metadata: Some(false),
            fast_mode: Some(false),
            reduce_colors: Some(false),
            target_reduction: None,
            preserve_alpha: Some(true),
            max_width: None,
            max_height: None,
            image: ImageOptConfig::default(),
            mesh: MeshOptConfig::default(),
            enable_threading: true,
            use_zero_copy: false,
        }
    }
}

/// Processing statistics for performance monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub operations_count: u64,
    pub total_input_bytes: u64,
    pub total_output_bytes: u64,
    pub total_processing_time_ms: f64,
    pub c_hotspots_used: u64,
    pub rust_fallbacks_used: u64,
    pub memory_peak_usage_bytes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ProcessingStats {
    pub fn compression_ratio(&self) -> f64 {
        if self.total_input_bytes > 0 {
            self.total_output_bytes as f64 / self.total_input_bytes as f64
        } else {
            1.0
        }
    }
    
    pub fn average_processing_time(&self) -> f64 {
        if self.operations_count > 0 {
            self.total_processing_time_ms / self.operations_count as f64
        } else {
            0.0
        }
    }
    
    pub fn cache_hit_ratio(&self) -> f64 {
        let total_cache_operations = self.cache_hits + self.cache_misses;
        if total_cache_operations > 0 {
            self.cache_hits as f64 / total_cache_operations as f64
        } else {
            0.0
        }
    }
}

/// Image information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub format: String,
    pub has_alpha: bool,
    pub color_space: ColorSpace,
    pub compression: Option<String>,
    pub file_size: Option<usize>,
}

/// Color space enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum ColorSpace {
    RGB,
    RGBA,
    Grayscale,
    GrayscaleAlpha,
    CMYK,
    YUV,
    HSV,
    LAB,
}

/// Mesh information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshInfo {
    pub vertex_count: usize,
    pub face_count: usize,
    pub triangle_count: usize,
    pub has_normals: bool,
    pub has_texcoords: bool,
    pub has_colors: bool,
    pub bounding_box: BoundingBox,
    pub format: String,
    pub file_size: Option<usize>,
}

/// 3D bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BoundingBox {
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
    
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
    
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size[0] * size[1] * size[2]
    }
}

/// Buffer management for zero-copy operations
#[derive(Debug)]
pub struct ZeroCopyBuffer {
    data: Vec<u8>,
    readonly: bool,
}

impl ZeroCopyBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            readonly: false,
        }
    }
    
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            readonly: true,
        }
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        if self.readonly {
            None
        } else {
            Some(&mut self.data)
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            channels: 0,
            bit_depth: 0,
            format: String::new(),
            has_alpha: false,
            color_space: ColorSpace::RGB,
            compression: None,
            file_size: None,
        }
    }
}

impl Default for MeshInfo {
    fn default() -> Self {
        Self {
            vertex_count: 0,
            face_count: 0,
            triangle_count: 0,
            has_normals: false,
            has_texcoords: false,
            has_colors: false,
            bounding_box: BoundingBox::default(),
            format: String::new(),
            file_size: None,
        }
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: [0.0, 0.0, 0.0],
            max: [0.0, 0.0, 0.0],
        }
    }
}

// Legacy type aliases for backward compatibility
pub type ImageOptimizer = ();
pub type MeshOptimizer = ();
pub type VideoOptimizer = ();
