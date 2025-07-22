use std::io::{self, Write};

// Simple implementation without colored output for now
pub struct UserFeedback;

impl UserFeedback {
    /// Display a welcoming message with version info
    pub fn show_welcome() {
        println!("{}", "✨ Pixie Juice - Image/Mesh/Video Optimizer");
        println!("{}", "Version 0.1.0 - Powered by Rust + C Hotspots");
        println!();
    }

    /// Display helpful error messages with suggestions
    pub fn show_error(error: &str, suggestion: Option<&str>) {
        eprintln!("{} {}", "❌ Error:", error);
        if let Some(hint) = suggestion {
            eprintln!("{} {}", "💡 Hint:", hint);
        }
        eprintln!();
    }

    /// Display success messages with metrics
    pub fn show_success(message: &str, stats: Option<&str>) {
        println!("{} {}", "✅ Success:", message);
        if let Some(metrics) = stats {
            println!("{} {}", "📊 Stats:", metrics);
        }
        println!();
    }

    /// Display warnings that don't halt execution
    pub fn show_warning(message: &str) {
        println!("{} {}", "⚠️  Warning:", message);
    }

    /// Display information messages
    pub fn show_info(message: &str) {
        println!("{} {}", "ℹ️  Info:", message);
    }

    /// Enhanced file not found error with helpful suggestions
    pub fn file_not_found(path: &str) {
        Self::show_error(
            &format!("Input file not found: {}", path),
            Some("Check the file path and ensure the file exists. Use absolute paths if needed.")
        );
    }

    /// Enhanced unsupported format error with format list
    pub fn unsupported_format(format: &str, supported: &[&str]) {
        Self::show_error(
            &format!("Unsupported format: {}", format),
            Some(&format!(
                "Supported formats: {}. Use --format to specify output format.",
                supported.join(", ")
            ))
        );
    }

    /// Enhanced permission error with helpful suggestions
    pub fn permission_denied(path: &str) {
        Self::show_error(
            &format!("Permission denied: {}", path),
            Some("Check file permissions or try running with elevated privileges.")
        );
    }

    /// Enhanced out of memory error with suggestions
    pub fn out_of_memory(file_size: Option<u64>) {
        let suggestion = if let Some(size) = file_size {
            if size > 100_000_000 { // > 100MB
                "Try using --fast-mode or --max-width/--max-height to reduce memory usage for large files."
            } else {
                "Close other applications to free up memory or process smaller files."
            }
        } else {
            "Try processing smaller files or use --fast-mode for reduced memory usage."
        };

        Self::show_error(
            "Insufficient memory to process file",
            Some(suggestion)
        );
    }

    /// Enhanced invalid arguments error
    pub fn invalid_arguments(message: &str) {
        Self::show_error(
            &format!("Invalid arguments: {}", message),
            Some("Use --help to see available options and examples.")
        );
    }

    /// Show processing progress with spinner and ETA
    pub fn show_progress(operation: &str, progress: f32, eta: Option<&str>) {
        let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner_idx = (progress * 10.0) as usize % spinner_chars.len();
        let bar_width = 30;
        let filled = (progress * bar_width as f32) as usize;
        let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        
        let eta_text = eta.map(|e| format!(" ETA: {}", e)).unwrap_or_default();
        
        print!("\r{} {} [{}] {:.1}%{}", 
               spinner_chars[spinner_idx],
               operation,
               bar,
               progress * 100.0,
               eta_text
        );
        io::stdout().flush().unwrap();
    }

    /// Clear progress line and show completion
    pub fn finish_progress(operation: &str, duration: &str) {
        print!("\r{}{}", " ".repeat(80), "\r"); // Clear line
        println!("{} {} ({})", 
                "✅", 
                operation, 
                duration
        );
    }

    /// Show file processing results with before/after comparison
    pub fn show_file_results(
        input_path: &str,
        output_path: &str,
        original_size: u64,
        optimized_size: u64,
        duration: &str
    ) {
        let reduction = if original_size > 0 {
            ((original_size - optimized_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        println!("{}", "📁 File Processing Results");
        println!("   Input:     {}", input_path);
        println!("   Output:    {}", output_path);
        println!("   Original:  {}", format_file_size(original_size));
        println!("   Optimized: {}", format_file_size(optimized_size));
        
        if reduction > 0.0 {
            println!("   Saved:     {:.1}% ({} smaller)", 
                    reduction, 
                    format_file_size(original_size - optimized_size)
            );
        } else if reduction < 0.0 {
            println!("   Increased: {:.1}% ({} larger)", 
                    -reduction, 
                    format_file_size(optimized_size - original_size)
            );
        } else {
            println!("   Size:      {} (unchanged)", "No change");
        }
        
        println!("   Duration:  {}", duration);
        println!();
    }

    /// Show batch processing summary
    pub fn show_batch_summary(
        total_files: usize,
        successful: usize,
        failed: usize,
        total_saved: u64,
        duration: &str
    ) {
        println!("{}", "📊 Batch Processing Summary");
        println!("   Total files:     {}", total_files);
        println!("   Successful:      {}", successful.to_string());
        if failed > 0 {
            println!("   Failed:          {}", failed.to_string());
        }
        if total_saved > 0 {
            println!("   Total saved:     {}", format_file_size(total_saved));
        }
        println!("   Total duration:  {}", duration);
        println!();
    }

    /// Show helpful usage examples
    pub fn show_examples() {
        println!("{}", "💡 Usage Examples");
        println!();
        println!("  {}", "Image Optimization:");
        println!("    pxjc optimize -i photo.jpg -o optimized.jpg -q 85");
        println!("    pxjc optimize -i image.png -o image.webp --format webp");
        println!();
        println!("  {}", "Mesh Processing:");
        println!("    pxjc optimize -i model.ply -o optimized.ply --reduce 0.5");
        println!("    pxjc optimize -i mesh.obj -o mesh.ply --deduplicate");
        println!();
        println!("  {}", "Batch Processing:");
        println!("    pxjc batch --input-dir ./images --output-dir ./optimized");
        println!("    pxjc batch -i ./photos -o ./compressed --format jpeg -q 80");
        println!();
    }

    /// Show performance tips based on file type and size
    pub fn show_performance_tips(file_size: u64, file_type: &str) {
        if file_size > 50_000_000 { // > 50MB
            Self::show_info("Large file detected. Consider these performance tips:");
            println!("   • Use --fast-mode for quicker processing");
            println!("   • Try --max-width/--max-height to resize and reduce memory usage");
            if file_type.starts_with("image/") {
                println!("   • Consider JPEG or WebP format for better compression");
            }
            println!();
        }
    }

    /// Show format-specific optimization suggestions
    pub fn show_format_suggestions(input_format: &str, output_format: &str) {
        match (input_format, output_format) {
            ("png", "jpeg") => {
                Self::show_info("Converting PNG to JPEG will remove transparency. Use --quality 85-95 for photos.");
            }
            ("jpeg", "png") => {
                Self::show_info("Converting JPEG to PNG may increase file size but preserves quality.");
            }
            ("png", "webp") | ("jpeg", "webp") => {
                Self::show_info("WebP provides excellent compression. Try --quality 80 for good balance.");
            }
            (input, output) if input == output => {
                Self::show_info("Optimizing same format. Lossless compression will be applied where possible.");
            }
            _ => {}
        }
    }
}

/// Format file sizes in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else {
        let minutes = (seconds / 60.0) as u32;
        let remaining_seconds = seconds % 60.0;
        format!("{}m {:.1}s", minutes, remaining_seconds)
    }
}
