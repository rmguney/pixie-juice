//! Pixie Juice CLI - Main entry point

mod args;

use anyhow::{Context, Result};
use args::{Args, Commands, MeshOptions, VideoOptions, OutputFormat};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rust_core::{ImageOptimizer, MeshOptimizer, OptConfig};
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    match args.command {
        Commands::Optimize { 
            input, 
            output, 
            format,
            quality,
            compression,
            lossless,
            preserve_image_metadata,
            fast_mode,
            reduce_colors,
            target_reduction,
            max_width,
            max_height,
            mesh_opts,
            video_opts,
        } => {
            optimize_single_file(
                &input, 
                &output, 
                format,
                quality,
                compression,
                lossless,
                preserve_image_metadata,
                fast_mode,
                reduce_colors,
                target_reduction,
                max_width,
                max_height,
                &mesh_opts,
                &video_opts,
            )
        }
        Commands::Batch { 
            input_dir, 
            output_dir, 
            pattern,
            recursive,
            optimize,
        } => {
            batch_process(
                &input_dir,
                output_dir.as_deref(),
                &pattern,
                recursive,
                optimize,
            )
        }
        Commands::Validate { file } => {
            validate_file(&file, None)
        }
    }
}

fn optimize_single_file(
    input: &Path,
    output: &Path,
    format: Option<OutputFormat>,
    quality: Option<u8>,
    compression: Option<u8>,
    lossless: bool,
    preserve_image_metadata: bool,
    fast_mode: bool,
    reduce_colors: bool,
    target_reduction: Option<f32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    mesh_opts: &MeshOptions,
    video_opts: &VideoOptions,
) -> Result<()> {
    let start_time = Instant::now();
    
    // Check input file exists
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }
    
    // Create output directory if needed
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }
    
    // Read input file
    let input_data = fs::read(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;
    
    let original_size = input_data.len();
    
    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);
    pb.set_message(format!("Processing {}...", input.file_name().unwrap().to_string_lossy()));
    
    // Determine file type and process accordingly
    let output_data = if is_image_file(input) {
        pb.set_message("Optimizing image...");
        optimize_image(
            &input_data,
            format.as_ref(),
            quality,
            compression,
            lossless,
            preserve_image_metadata,
            fast_mode,
            reduce_colors,
            target_reduction,
            max_width,
            max_height,
        )?
    } else if is_mesh_file(input) {
        pb.set_message("Optimizing mesh...");
        optimize_mesh(&input_data, format.as_ref(), mesh_opts)?
    } else if is_video_file(input) {
        pb.set_message("Optimizing video...");
        // Video optimization works directly with files
        optimize_video(input, output, format.as_ref(), video_opts)?;
        pb.finish_with_message("Video optimized!");
        return Ok(()); // Early return since video was handled directly
    } else {
        pb.finish_with_message("Unknown file type, copying...");
        input_data
    };
    
    // Write output file
    fs::write(output, &output_data)
        .with_context(|| format!("Failed to write output file: {}", output.display()))?;
    
    let duration = start_time.elapsed();
    let new_size = output_data.len();
    let savings = if original_size > new_size {
        (original_size - new_size) as f64 / original_size as f64 * 100.0
    } else {
        0.0
    };
    
    pb.finish_with_message(format!(
        "✅ Processed {} -> {} ({:.1}% smaller) in {:.2}s",
        format_size(original_size),
        format_size(new_size),
        savings,
        duration.as_secs_f64()
    ));
    
    Ok(())
}

fn optimize_image(
    data: &[u8],
    _format: Option<&OutputFormat>,
    quality: Option<u8>,
    compression: Option<u8>,
    lossless: bool,
    preserve_image_metadata: bool,
    fast_mode: bool,
    reduce_colors: bool,
    target_reduction: Option<f32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<Vec<u8>> {
    let config = OptConfig {
        quality,
        compression_level: compression,
        lossless: Some(lossless),
        preserve_metadata: Some(preserve_image_metadata),
        fast_mode: Some(fast_mode),
        reduce_colors: Some(reduce_colors),
        target_reduction,
        max_width,
        max_height,
    };
    
    let optimizer = ImageOptimizer::new();
    optimizer.optimize(data, &config)
        .map_err(|e| anyhow::anyhow!("Image optimization failed: {}", e))
}

fn optimize_mesh(
    data: &[u8],
    _format: Option<&OutputFormat>,
    mesh_opts: &MeshOptions,
) -> Result<Vec<u8>> {
    let config = OptConfig {
        quality: None,
        compression_level: None,
        lossless: Some(false),
        preserve_metadata: Some(false),
        fast_mode: Some(false),
        reduce_colors: Some(false),
        target_reduction: mesh_opts.reduce,
        max_width: None,
        max_height: None,
    };

    let optimizer = MeshOptimizer::new();
    
    if mesh_opts.deduplicate {
        let deduplicated = optimizer.deduplicate_vertices(data)
            .map_err(|e| anyhow::anyhow!("Mesh deduplication failed: {}", e))?;
        
        if let Some(reduce_ratio) = mesh_opts.reduce {
            optimizer.reduce_triangles(&deduplicated, reduce_ratio)
                .map_err(|e| anyhow::anyhow!("Mesh reduction failed: {}", e))
        } else {
            Ok(deduplicated)
        }
    } else if let Some(reduce_ratio) = mesh_opts.reduce {
        optimizer.reduce_triangles(data, reduce_ratio)
            .map_err(|e| anyhow::anyhow!("Mesh reduction failed: {}", e))
    } else {
        // Use general optimize method
        optimizer.optimize(data, &config)
            .map_err(|e| anyhow::anyhow!("Mesh optimization failed: {}", e))
    }
}

fn optimize_video(
    input_path: &Path,
    output_path: &Path,
    _format: Option<&OutputFormat>,
    video_opts: &VideoOptions,
) -> Result<()> {
    // Convert video options to new format
    let quality = video_opts.crf.map(|crf| {
        // Convert CRF (0-51, lower is better) to quality (0.0-1.0, higher is better)
        1.0 - (crf as f32 / 51.0)
    }).unwrap_or(0.85); // Default quality if no CRF specified
    
    let options = rust_core::common::OptimizationOptions {
        quality,
        target_size_kb: None,
        preserve_metadata: video_opts.preserve_video_metadata,
        progressive: false,
        optimization_level: if video_opts.web_optimize { 9 } else { 6 },
    };
    
    // TODO: Handle trim option when implementing video processing
    if video_opts.trim.is_some() {
        log::warn!("Video trimming not yet implemented: {:?}", video_opts.trim);
    }
    
    // Use new video module for optimization
    rust_core::video::optimize_video(input_path, output_path, &options)
        .map_err(|e| anyhow::anyhow!("Video optimization failed: {}", e))?;
    
    Ok(())
}

fn batch_process(
    input_dir: &Path,
    output_dir: Option<&Path>,
    pattern: &str,
    recursive: bool,
    optimize: bool,
) -> Result<()> {
    if !input_dir.exists() {
        anyhow::bail!("Input directory not found: {}", input_dir.display());
    }
    
    let default_output_dir = input_dir.join("optimized");
    let output_dir = output_dir.unwrap_or(&default_output_dir);
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    
    // Create glob pattern for file search
    let glob_pattern = if recursive {
        input_dir.join("**").join(pattern)
    } else {
        input_dir.join(pattern)
    };
    
    println!("🔍 Scanning {} for files matching '{}'...", input_dir.display(), pattern);
    
    let mut files_to_process = Vec::new();
    if let Ok(entries) = glob::glob(&glob_pattern.to_string_lossy()) {
        for entry in entries.flatten() {
            if entry.is_file() {
                files_to_process.push(entry);
            }
        }
    }
    
    if files_to_process.is_empty() {
        println!("No files found to process.");
        return Ok(());
    }
    
    println!("📦 Processing {} files...", files_to_process.len());
    
    let pb = ProgressBar::new(files_to_process.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")?);
    
    let mut processed = 0;
    let mut errors = 0;
    
    // Create default options for batch processing
    let default_mesh_opts = MeshOptions {
        deduplicate: false,
        tolerance: 0.001,
        reduce: None,
        simplify: false,
    };
    
    let default_video_opts = VideoOptions {
        crf: None,
        trim: None,
        preserve_video_metadata: false,
        web_optimize: false,
    };
    
    for input_file in files_to_process {
        let output_file = if optimize {
            output_dir.join(format!("optimized_{}", input_file.file_name().unwrap().to_string_lossy()))
        } else {
            output_dir.join(input_file.file_name().unwrap())
        };
        
        match optimize_single_file(
            &input_file,
            &output_file,
            None, // Auto-detect format
            None, // Default quality
            None, // Default compression
            false, // Not lossless by default
            false, // Don't preserve metadata by default
            false, // Not fast mode by default
            false, // Don't reduce colors by default
            None, // No target reduction
            None, // No max width
            None, // No max height
            &default_mesh_opts,
            &default_video_opts,
        ) {
            Ok(_) => {
                processed += 1;
                pb.set_message(format!("✅ {}", input_file.file_name().unwrap().to_string_lossy()));
            }
            Err(e) => {
                errors += 1;
                pb.set_message(format!("❌ {}: {}", input_file.file_name().unwrap().to_string_lossy(), e));
            }
        }
        pb.inc(1);
    }
    
    pb.finish_with_message(format!("📋 Batch complete: {} processed, {} errors", processed, errors));
    
    Ok(())
}

fn validate_file(input: &Path, expected_format: Option<&str>) -> Result<()> {
    if !input.exists() {
        anyhow::bail!("File not found: {}", input.display());
    }
    
    let data = fs::read(input)
        .with_context(|| format!("Failed to read file: {}", input.display()))?;
    
    if is_image_file(input) {
        match rust_core::image::detect_image_format(&data) {
            Ok(format) => {
                println!("✅ Valid image: {} format", format.extension().to_uppercase());
                if let Some(expected) = expected_format {
                    if format.extension() != expected.to_lowercase() {
                        println!("⚠️  Expected {}, found {}", expected.to_uppercase(), format.extension().to_uppercase());
                    }
                }
            }
            Err(e) => {
                println!("❌ Invalid image: {}", e);
                return Err(anyhow::anyhow!("Image validation failed"));
            }
        }
    } else if is_mesh_file(input) {
        println!("✅ Mesh file detected: {}", input.extension().unwrap().to_string_lossy().to_uppercase());
    } else if is_video_file(input) {
        println!("✅ Video file detected: {}", input.extension().unwrap().to_string_lossy().to_uppercase());
    } else {
        println!("❓ Unknown file type: {}", input.extension().unwrap_or_default().to_string_lossy());
    }
    
    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_string_lossy().to_lowercase().as_str(), 
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" | "tif")
    } else {
        false
    }
}

fn is_mesh_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_string_lossy().to_lowercase().as_str(), 
                "obj" | "ply" | "stl" | "gltf" | "glb")
    } else {
        false
    }
}

fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_string_lossy().to_lowercase().as_str(), 
                "mp4" | "m4v" | "webm")
    } else {
        false
    }
}

fn format_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
