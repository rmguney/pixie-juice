extern crate alloc;
use alloc::{string::String, format, string::ToString};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::types::{ImageOptConfig, MeshOptConfig, SimplificationAlgorithm, ColorSpace};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PixieConfig {
    image: ImageConfig,
    mesh: MeshConfig,
    performance: PerformanceConfig,
    ui: UiConfig,
    threading: ThreadingConfig,
}

#[wasm_bindgen]
impl PixieConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    
    #[wasm_bindgen(getter)]
    pub fn use_c_hotspots(&self) -> bool {
        self.performance.use_c_hotspots
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_use_c_hotspots(&mut self, value: bool) {
        self.performance.use_c_hotspots = value;
    }
    
    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> u8 {
        self.image.jpeg_quality
    }
    
    #[wasm_bindgen(setter)]  
    pub fn set_quality(&mut self, quality: u8) {
        self.image.jpeg_quality = quality.clamp(1, 100);
        self.image.webp_quality = quality.clamp(1, 100);
    }
    
    #[wasm_bindgen(getter)]
    pub fn enable_threading(&self) -> bool {
        self.threading.enable_threads
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_enable_threading(&mut self, value: bool) {
        self.threading.enable_threads = value;
    }
    
    pub fn to_image_config(&self) -> ImageOptConfig {
        ImageOptConfig {
            quality: self.image.jpeg_quality,
            lossless: self.image.prefer_lossless,
            preserve_metadata: self.image.preserve_metadata,
            optimize_colors: self.image.reduce_colors,
            max_colors: Some(self.image.max_colors),
            use_c_hotspots: self.performance.use_c_hotspots,
            enable_simd: self.performance.enable_simd,
            compression_level: Some(self.image.png_compression),
            fast_mode: self.performance.fast_mode,
            preserve_alpha: true,
            max_width: self.image.max_width,
            max_height: self.image.max_height,
            target_reduction: None,
        }
    }
    
    pub fn to_mesh_config(&self) -> MeshOptConfig {
        MeshOptConfig {
            target_ratio: self.mesh.decimation_ratio,
            preserve_topology: true,
            weld_vertices: self.mesh.weld_vertices,
            vertex_tolerance: self.mesh.weld_threshold,
            simplification_algorithm: self.mesh.simplification_algorithm,
            use_c_hotspots: self.performance.use_c_hotspots,
            generate_normals: !self.mesh.preserve_normals,
            optimize_vertex_cache: true,
            preserve_uv_seams: self.mesh.preserve_uvs,
            preserve_boundaries: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub jpeg_quality: u8,
    pub png_compression: u8,
    pub webp_quality: u8,
    pub preserve_metadata: bool,
    pub prefer_lossless: bool,
    pub reduce_colors: bool,
    pub max_colors: u16,
    pub color_space: ColorSpace,
    pub progressive_jpeg: bool,
    pub optimize_huffman: bool,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    pub decimation_ratio: f32,
    pub preserve_uvs: bool,
    pub preserve_colors: bool,
    pub preserve_normals: bool,
    pub weld_vertices: bool,
    pub weld_threshold: f32,
    pub simplification_algorithm: SimplificationAlgorithm,
    pub generate_lod: bool,
    pub lod_levels: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub use_c_hotspots: bool,
    pub enable_simd: bool,
    pub fast_mode: bool,
    pub memory_limit_mb: u32,
    pub use_streaming: bool,
    pub cache_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_progress: bool,
    pub show_stats: bool,
    pub use_colors: bool,
    pub verbosity: u8,
    pub show_performance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadingConfig {
    pub enable_threads: bool,
    pub thread_count: usize,
    pub use_thread_pool: bool,
    pub work_stealing: bool,
}

impl Default for PixieConfig {
    fn default() -> Self {
        Self {
            image: ImageConfig {
                jpeg_quality: 85,
                png_compression: 6,
                webp_quality: 80,
                preserve_metadata: false,
                prefer_lossless: false,
                reduce_colors: true,
                max_colors: 256,
                color_space: ColorSpace::RGB,
                progressive_jpeg: true,
                optimize_huffman: true,
                max_width: None,
                max_height: None,
            },
            mesh: MeshConfig {
                decimation_ratio: 0.5,
                preserve_uvs: true,
                preserve_colors: true,
                preserve_normals: true,
                weld_vertices: true,
                weld_threshold: 1e-6,
                simplification_algorithm: SimplificationAlgorithm::QuadricErrorMetrics,
                generate_lod: false,
                lod_levels: 3,
            },
            performance: PerformanceConfig {
                use_c_hotspots: true,
                enable_simd: true,
                fast_mode: false,
                memory_limit_mb: 512,
                use_streaming: true,
                cache_size_mb: 64,
            },
            ui: UiConfig {
                show_progress: true,
                show_stats: true,
                use_colors: false,
                verbosity: 1,
                show_performance: true,
            },
            threading: ThreadingConfig {
                enable_threads: true,
                thread_count: 0,
                use_thread_pool: true,
                work_stealing: true,
            },
        }
    }
}

impl PixieConfig {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.image.jpeg_quality == 0 || self.image.jpeg_quality > 100 {
            return Err("JPEG quality must be between 1 and 100".to_string());
        }

        if self.image.webp_quality == 0 || self.image.webp_quality > 100 {
            return Err("WebP quality must be between 1 and 100".to_string());
        }

        if self.image.png_compression > 9 {
            return Err("PNG compression level must be between 0 and 9".to_string());
        }

        if self.mesh.decimation_ratio < 0.0 || self.mesh.decimation_ratio > 1.0 {
            return Err("Mesh decimation ratio must be between 0.0 and 1.0".to_string());
        }

        if self.mesh.weld_threshold < 0.0 {
            return Err("Mesh weld threshold must be positive".to_string());
        }

        if self.ui.verbosity > 3 {
            return Err("Verbosity level must be between 0 and 3".to_string());
        }

        Ok(())
    }

    pub fn summary(&self) -> String {
        format!(
            "Pixie Juice WASM Configuration:
  Image: JPEG {}%, PNG {}, WebP {}%, {} hotspots
  Mesh: {:.1}% reduction, {} algorithm, {} hotspots
  Performance: {} threads, {} SIMD, {} fast mode, {} hotspots
  UI: verbosity {}, {} progress, {} performance metrics",
            self.image.jpeg_quality,
            self.image.png_compression,
            self.image.webp_quality,
            if self.performance.use_c_hotspots { "C" } else { "Rust" },
            self.mesh.decimation_ratio * 100.0,
            match self.mesh.simplification_algorithm {
                SimplificationAlgorithm::QuadricErrorMetrics => "QEM",
                SimplificationAlgorithm::EdgeCollapse => "Edge",
                SimplificationAlgorithm::VertexClustering => "Cluster",
            },
            if self.performance.use_c_hotspots { "C" } else { "Rust" },
            if self.threading.thread_count == 0 { "auto".to_string() } else { self.threading.thread_count.to_string() },
            if self.performance.enable_simd { "enabled" } else { "disabled" },
            if self.performance.fast_mode { "enabled" } else { "disabled" },
            if self.performance.use_c_hotspots { "C" } else { "Rust" },
            self.ui.verbosity,
            if self.ui.show_progress { "enabled" } else { "disabled" },
            if self.ui.show_performance { "enabled" } else { "disabled" }
        )
    }
}


