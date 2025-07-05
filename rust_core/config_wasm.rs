//! Simple configuration for WASM builds (no file system access)

use serde::{Deserialize, Serialize};

/// Configuration structure for Pixie Juice WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixieConfig {
    /// Default image optimization settings
    pub image: ImageConfig,
    /// Default mesh optimization settings  
    pub mesh: MeshConfig,
    /// Default video optimization settings
    pub video: VideoConfig,
    /// Performance and behavior settings
    pub performance: PerformanceConfig,
    /// User interface preferences
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,
    /// PNG compression level (0-9)
    pub png_compression: u8,
    /// WebP quality (1-100)
    pub webp_quality: u8,
    /// Whether to preserve metadata
    pub preserve_metadata: bool,
    /// Whether to use lossless compression when possible
    pub prefer_lossless: bool,
    /// Whether to reduce color palette
    pub reduce_colors: bool,
    /// Maximum colors for palette reduction
    pub max_colors: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Target triangle reduction ratio (0.0-1.0)
    pub decimation_ratio: f32,
    /// Whether to preserve UV coordinates
    pub preserve_uvs: bool,
    /// Whether to preserve vertex colors
    pub preserve_colors: bool,
    /// Whether to preserve vertex normals
    pub preserve_normals: bool,
    /// Whether to weld vertices
    pub weld_vertices: bool,
    /// Vertex welding threshold
    pub weld_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Target bitrate for video compression
    pub target_bitrate: u32,
    /// Video quality preset
    pub quality_preset: String,
    /// Whether to preserve audio
    pub preserve_audio: bool,
    /// Target frame rate (0 = preserve original)
    pub target_fps: u32,
    /// Target resolution scaling (1.0 = original)
    pub resolution_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of threads to use (0 = auto)
    pub thread_count: usize,
    /// Whether to enable C hotspots
    pub use_c_hotspots: bool,
    /// Memory limit for large files (MB)
    pub memory_limit_mb: u32,
    /// Whether to use streaming for large files
    pub use_streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Whether to show progress bars
    pub show_progress: bool,
    /// Whether to show detailed statistics
    pub show_stats: bool,
    /// Whether to use colored output
    pub use_colors: bool,
    /// Verbosity level (0-3)
    pub verbosity: u8,
}

impl Default for PixieConfig {
    fn default() -> Self {
        Self {
            image: ImageConfig::default(),
            mesh: MeshConfig::default(),
            video: VideoConfig::default(),
            performance: PerformanceConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            jpeg_quality: 85,
            png_compression: 6,
            webp_quality: 80,
            preserve_metadata: true,
            prefer_lossless: false,
            reduce_colors: false,
            max_colors: 256,
        }
    }
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            decimation_ratio: 0.1,
            preserve_uvs: true,
            preserve_colors: true,
            preserve_normals: true,
            weld_vertices: true,
            weld_threshold: 0.001,
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            target_bitrate: 1000,
            quality_preset: "medium".to_string(),
            preserve_audio: true,
            target_fps: 0,
            resolution_scale: 1.0,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_count: 0,
            use_c_hotspots: true,
            memory_limit_mb: 512,
            use_streaming: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_progress: true,
            show_stats: true,
            use_colors: false, // No colors in WASM
            verbosity: 1,
        }
    }
}

impl PixieConfig {
    /// Create a new config with WASM-appropriate defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// WASM builds don't support file loading/saving
    pub fn load_from_file(_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

    /// WASM builds don't support file saving
    pub fn save_to_file(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Load from JSON string (for WASM localStorage integration)
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Save to JSON string (for WASM localStorage integration)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
