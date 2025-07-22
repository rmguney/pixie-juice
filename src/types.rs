//! Common types for the core crate

/// Result type for optimization operations
pub type OptResult<T> = Result<T, OptError>;

/// Error types for optimization operations
#[derive(Debug, thiserror::Error)]
pub enum OptError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Format error: {0}")]
    FormatError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("XML error: {0}")]
    XmlError(String),
}

impl From<quick_xml::Error> for OptError {
    fn from(err: quick_xml::Error) -> Self {
        OptError::XmlError(err.to_string())
    }
}

/// Configuration for optimization operations
#[derive(Debug, Clone)]
pub struct OptConfig {
    /// Quality setting (0-100 for lossy formats)
    pub quality: Option<u8>,
    
    /// Compression level (format-specific)
    pub compression_level: Option<u8>,
    
    /// Whether to use lossless compression when available
    pub lossless: Option<bool>,
    
    /// Whether to preserve metadata (EXIF, etc.)
    pub preserve_metadata: Option<bool>,
    
    /// Fast mode (less optimization, faster processing)
    pub fast_mode: Option<bool>,
    
    /// Reduce color palette for GIF/PNG
    pub reduce_colors: Option<bool>,
    
    /// Target file size reduction ratio (0.0-1.0)
    pub target_reduction: Option<f32>,
    
    /// Whether to preserve alpha channels even if fully opaque
    pub preserve_alpha: Option<bool>,
    
    /// Maximum output dimensions (for resizing)
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
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
            preserve_alpha: Some(true),  // Default to preserving alpha channels
            max_width: None,
            max_height: None,
        }
    }
}
