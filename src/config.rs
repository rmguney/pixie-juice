use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[cfg(feature = "dirs")]
use dirs;

/// Configuration structure for Pixie Juice
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
    /// Default output format ("auto", "png", "jpeg", "webp", etc.)
    pub default_format: String,
    /// Default quality for lossy formats (1-100)
    pub default_quality: u8,
    /// Default compression level (1-9)
    pub default_compression: u8,
    /// Preserve metadata by default
    pub preserve_metadata: bool,
    /// Use lossless compression when available
    pub prefer_lossless: bool,
    /// Automatically reduce colors for PNG/GIF
    pub auto_reduce_colors: bool,
    /// Maximum dimensions for automatic resizing
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Default output format ("auto", "ply", "obj", "stl")
    pub default_format: String,
    /// Default vertex deduplication tolerance
    pub default_tolerance: f64,
    /// Default triangle reduction ratio (0.0-1.0)
    pub default_reduction: Option<f64>,
    /// Enable vertex deduplication by default
    pub auto_deduplicate: bool,
    /// Enable automatic mesh simplification
    pub auto_simplify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Default output format ("auto", "mp4", "webm")
    pub default_format: String,
    /// Default CRF quality (0-51, lower = better)
    pub default_crf: u8,
    /// Preserve video metadata by default
    pub preserve_metadata: bool,
    /// Optimize for web streaming by default
    pub web_optimize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Use fast mode by default (less optimization, faster processing)
    pub fast_mode: bool,
    /// Number of threads for parallel processing (0 = auto)
    pub num_threads: usize,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Enable memory-mapped file I/O for large files
    pub use_memory_mapping: bool,
    /// Memory limit in MB (0 = no limit)
    pub memory_limit_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Use colored output in terminal
    pub use_colors: bool,
    /// Show progress bars for long operations
    pub show_progress: bool,
    /// Verbosity level (0=quiet, 1=normal, 2=verbose, 3=debug)
    pub verbosity: u8,
    /// Show performance tips and suggestions
    pub show_tips: bool,
    /// Show file processing statistics
    pub show_stats: bool,
}

impl Default for PixieConfig {
    fn default() -> Self {
        Self {
            image: ImageConfig {
                default_format: "auto".to_string(),
                default_quality: 85,
                default_compression: 6,
                preserve_metadata: true,
                prefer_lossless: false,
                auto_reduce_colors: false,
                max_width: None,
                max_height: None,
            },
            mesh: MeshConfig {
                default_format: "auto".to_string(),
                default_tolerance: 0.001,
                default_reduction: None,
                auto_deduplicate: true,
                auto_simplify: false,
            },
            video: VideoConfig {
                default_format: "auto".to_string(),
                default_crf: 23,
                preserve_metadata: true,
                web_optimize: false,
            },
            performance: PerformanceConfig {
                fast_mode: false,
                num_threads: 0, // Auto-detect
                enable_simd: true,
                use_memory_mapping: true,
                memory_limit_mb: 0, // No limit
            },
            ui: UiConfig {
                use_colors: true,
                show_progress: true,
                verbosity: 1,
                show_tips: true,
                show_stats: true,
            },
        }
    }
}

impl PixieConfig {
    /// Load configuration from file, create default if not exists
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: PixieConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
        Ok(())
    }

    /// Get the configuration file path
    #[cfg(feature = "dirs")]
    pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?;
        
        Ok(config_dir.join("pixie-juice").join("config.toml"))
    }
    
    /// Get the configuration file path (WASM fallback)
    #[cfg(not(feature = "dirs"))]
    pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // In WASM, there's no filesystem access for config files
        Err("Configuration files not supported in WASM builds".into())
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: PixieConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Create a project-specific configuration
    pub fn create_project_config<P: AsRef<std::path::Path>>(
        project_path: P,
        config: &PixieConfig
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_file = project_path.as_ref().join("pixie-juice.toml");
        let content = toml::to_string_pretty(config)?;
        fs::write(config_file, content)?;
        Ok(())
    }

    /// Load project-specific configuration if it exists
    pub fn load_project_config<P: AsRef<std::path::Path>>(
        project_path: P
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let config_file = project_path.as_ref().join("pixie-juice.toml");
        
        if config_file.exists() {
            let content = fs::read_to_string(config_file)?;
            let config: PixieConfig = toml::from_str(&content)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Merge project config with global config (project takes precedence)
    pub fn merge_with_project(&self, project_config: &PixieConfig) -> PixieConfig {
        let mut merged = self.clone();
        
        // Merge image settings
        if project_config.image.default_format != "auto" {
            merged.image.default_format = project_config.image.default_format.clone();
        }
        if project_config.image.default_quality != 85 { // Not default
            merged.image.default_quality = project_config.image.default_quality;
        }
        // ... (merge other settings as needed)
        
        merged
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        // Validate image quality
        if self.image.default_quality == 0 || self.image.default_quality > 100 {
            return Err("Image quality must be between 1 and 100".to_string());
        }

        // Validate compression level
        if self.image.default_compression == 0 || self.image.default_compression > 9 {
            return Err("Compression level must be between 1 and 9".to_string());
        }

        // Validate mesh tolerance
        if self.mesh.default_tolerance < 0.0 {
            return Err("Mesh tolerance must be positive".to_string());
        }

        // Validate video CRF
        if self.video.default_crf > 51 {
            return Err("Video CRF must be between 0 and 51".to_string());
        }

        // Validate verbosity
        if self.ui.verbosity > 3 {
            return Err("Verbosity level must be between 0 and 3".to_string());
        }

        Ok(())
    }

    /// Get configuration summary for display
    pub fn summary(&self) -> String {
        format!(
            "Pixie Juice Configuration Summary:
  Image: {} format, {}% quality, {} compression
  Mesh: {} format, {:.3} tolerance, {} deduplication
  Video: {} format, {} CRF, {} metadata
  Performance: {} threads, {} SIMD, {} fast mode
  UI: verbosity {}, {} colors, {} progress",
            self.image.default_format,
            self.image.default_quality,
            self.image.default_compression,
            self.mesh.default_format,
            self.mesh.default_tolerance,
            if self.mesh.auto_deduplicate { "auto" } else { "manual" },
            self.video.default_format,
            self.video.default_crf,
            if self.video.preserve_metadata { "preserve" } else { "strip" },
            if self.performance.num_threads == 0 { "auto".to_string() } else { self.performance.num_threads.to_string() },
            if self.performance.enable_simd { "enabled" } else { "disabled" },
            if self.performance.fast_mode { "enabled" } else { "disabled" },
            self.ui.verbosity,
            if self.ui.use_colors { "enabled" } else { "disabled" },
            if self.ui.show_progress { "enabled" } else { "disabled" }
        )
    }
}

/// Configuration management commands
pub struct ConfigManager;

impl ConfigManager {
    /// Initialize configuration (create default if not exists)
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let config = PixieConfig::load()?;
        println!("Configuration initialized at: {:?}", PixieConfig::get_config_path()?);
        println!("{}", config.summary());
        Ok(())
    }

    /// Show current configuration
    pub fn show() -> Result<(), Box<dyn std::error::Error>> {
        let config = PixieConfig::load()?;
        println!("{}", config.summary());
        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset() -> Result<(), Box<dyn std::error::Error>> {
        let default_config = PixieConfig::default();
        default_config.save()?;
        println!("Configuration reset to defaults");
        Ok(())
    }

    /// Set a configuration value
    pub fn set(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = PixieConfig::load()?;
        
        match key {
            "image.quality" => {
                config.image.default_quality = value.parse()
                    .map_err(|_| "Invalid quality value (must be 1-100)")?;
            }
            "image.format" => {
                config.image.default_format = value.to_string();
            }
            "mesh.tolerance" => {
                config.mesh.default_tolerance = value.parse()
                    .map_err(|_| "Invalid tolerance value (must be positive number)")?;
            }
            "performance.threads" => {
                config.performance.num_threads = value.parse()
                    .map_err(|_| "Invalid thread count (must be number)")?;
            }
            "ui.verbosity" => {
                config.ui.verbosity = value.parse()
                    .map_err(|_| "Invalid verbosity level (must be 0-3)")?;
            }
            _ => return Err(format!("Unknown configuration key: {}", key).into()),
        }

        config.validate()?;
        config.save()?;
        println!("Configuration updated: {} = {}", key, value);
        
        Ok(())
    }

    /// Create a project configuration template
    pub fn create_project<P: AsRef<std::path::Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
        let config = PixieConfig::default();
        PixieConfig::create_project_config(&path, &config)?;
        println!("Project configuration created at: {:?}", path.as_ref().join("pixie-juice.toml"));
        Ok(())
    }
}
