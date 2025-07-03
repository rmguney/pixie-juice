//! Command line argument parsing for pxjc CLI

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pxjc")]
#[command(about = "Pixie Juice - Image/Mesh/Video Optimizer")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Optimize a single file
    Optimize {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output format (auto-detected if not specified)
        #[arg(short, long)]
        format: Option<OutputFormat>,
        
        /// Quality setting (1-100 for lossy formats)
        #[arg(short, long)]
        quality: Option<u8>,
        
        /// Compression level (1-9)
        #[arg(short, long)]
        compression: Option<u8>,
        
        /// Use lossless compression when available
        #[arg(long)]
        lossless: bool,
        
        /// Preserve metadata (EXIF, etc.)
        #[arg(long)]
        preserve_image_metadata: bool,
        
        /// Fast mode (less optimization, faster processing)
        #[arg(long)]
        fast_mode: bool,
        
        /// Reduce color palette for GIF/PNG
        #[arg(long)]
        reduce_colors: bool,
        
        /// Target file size reduction ratio (0.0-1.0)
        #[arg(long)]
        target_reduction: Option<f32>,
        
        /// Maximum output width (for resizing)
        #[arg(long)]
        max_width: Option<u32>,
        
        /// Maximum output height (for resizing)
        #[arg(long)]
        max_height: Option<u32>,
        
        /// Mesh optimization options
        #[command(flatten)]
        mesh_opts: MeshOptions,
        
        /// Video optimization options  
        #[command(flatten)]
        video_opts: VideoOptions,
    },
    
    /// Batch process directory
    Batch {
        /// Input directory
        input_dir: PathBuf,
        
        /// Output directory
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        
        /// File pattern to match (e.g., "*.png")
        #[arg(short, long, default_value = "*")]
        pattern: String,
        
        /// Recursive processing
        #[arg(short, long)]
        recursive: bool,
        
        /// Apply optimizations
        #[arg(long)]
        optimize: bool,
    },
    
    /// Validate file format
    Validate {
        /// File to validate
        file: PathBuf,
    },
}

#[derive(clap::Args)]
pub struct MeshOptions {
    /// Remove duplicate vertices
    #[arg(long)]
    pub deduplicate: bool,
    
    /// Vertex deduplication tolerance
    #[arg(long, default_value = "0.001")]
    pub tolerance: f32,
    
    /// Triangle reduction ratio (0.0-1.0)
    #[arg(long)]
    pub reduce: Option<f32>,
    
    /// Simplify mesh automatically
    #[arg(long)]
    pub simplify: bool,
}

#[derive(clap::Args)]
pub struct VideoOptions {
    /// CRF quality setting (0-51, lower = better quality)
    #[arg(long)]
    pub crf: Option<u8>,
    
    /// Trim video (format: start-end, e.g., "00:10-00:30")
    #[arg(long)]
    pub trim: Option<String>,
    
    /// Preserve video metadata
    #[arg(long)]
    pub preserve_video_metadata: bool,
    
    /// Optimize for web streaming
    #[arg(long)]
    pub web_optimize: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    // Images
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Tiff,
    
    // Meshes
    Obj,
    Ply,
    Stl,
    Gltf,
    Glb,
    
    // Videos
    Mp4,
    Webm,
}
