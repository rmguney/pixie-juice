/// Common types and utilities shared across format modules

/// Error types for the pixel-squish library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Processing error: {0}")]
    Processing(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("C FFI error: {0}")]
    Ffi(String),
}

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, Error>;

/// Optimization options for all file types
#[derive(Debug, Clone)]
pub struct OptimizationOptions {
    /// Quality level (0.0 to 1.0, where 1.0 is highest quality)
    pub quality: f32,
    /// Target file size in KB (optional)
    pub target_size_kb: Option<u32>,
    /// Whether to preserve metadata
    pub preserve_metadata: bool,
    /// Progressive/interlaced encoding
    pub progressive: bool,
    /// Custom optimization level (0-9, where 9 is most aggressive)
    pub optimization_level: u8,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            quality: 0.85,
            target_size_kb: None,
            preserve_metadata: false,
            progressive: false,
            optimization_level: 6,
        }
    }
}

/// Result type used across all processing modules
pub type ProcessingResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Common metadata structure for file information
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_size: u64,
    pub format: String,
    pub dimensions: Option<(u32, u32)>, // width, height for images/videos
    pub duration: Option<f64>, // duration in seconds for videos
}

/// Processing statistics for optimization results
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub original_size: u64,
    pub optimized_size: u64,
    pub compression_ratio: f64,
    pub processing_time_ms: u64,
}

impl ProcessingStats {
    pub fn new(original_size: u64, optimized_size: u64, processing_time_ms: u64) -> Self {
        let compression_ratio = if original_size > 0 {
            optimized_size as f64 / original_size as f64
        } else {
            1.0
        };
        
        Self {
            original_size,
            optimized_size,
            compression_ratio,
            processing_time_ms,
        }
    }
    
    pub fn size_reduction_percent(&self) -> f64 {
        if self.original_size > 0 {
            ((self.original_size as f64 - self.optimized_size as f64) / self.original_size as f64) * 100.0
        } else {
            0.0
        }
    }
}
