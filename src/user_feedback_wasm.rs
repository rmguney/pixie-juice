//! Simple user feedback for WASM builds (no colored output)

use std::io::{self, Write};

/// Simple error messages and user feedback for WASM
pub struct UserFeedback;

impl UserFeedback {
    /// Display a welcoming message with version info
    pub fn show_welcome() {
        println!("✨ Pixie Juice - Image/Mesh/Video Optimizer");
        println!("Version 0.1.0 - WASM Build");
        println!();
    }

    /// Display helpful error messages
    pub fn show_error(error: &str, suggestion: Option<&str>) {
        eprintln!("❌ Error: {}", error);
        if let Some(hint) = suggestion {
            eprintln!("💡 Hint: {}", hint);
        }
        eprintln!();
    }

    /// Display success messages
    pub fn show_success(message: &str, metrics: Option<&str>) {
        println!("✅ Success: {}", message);
        if let Some(metrics) = metrics {
            println!("📊 Stats: {}", metrics);
        }
    }

    /// Display warning messages
    pub fn show_warning(message: &str) {
        println!("⚠️  Warning: {}", message);
    }

    /// Display informational messages
    pub fn show_info(message: &str) {
        println!("ℹ️  Info: {}", message);
    }

    /// Simple progress reporting for WASM
    pub fn show_progress(_current: usize, _total: usize, _message: &str, _start_time: std::time::Instant) {
        // Simplified progress for WASM - just log the message
        // In a real WASM app, this would use JS callbacks for progress updates
    }

    /// Show final results for WASM
    pub fn show_results(
        input_path: &str,
        output_path: &str,
        original_size: u64,
        optimized_size: u64,
        duration: &str,
    ) {
        println!("📁 File Processing Results");
        println!("   Input:     {}", input_path);
        println!("   Output:    {}", output_path);
        println!("   Original:  {}", format_file_size(original_size));
        println!("   Optimized: {}", format_file_size(optimized_size));
        
        if optimized_size < original_size {
            let savings = original_size - optimized_size;
            println!("   Saved:     {} ({:.1}% reduction)", 
                format_file_size(savings),
                (savings as f64 / original_size as f64) * 100.0
            );
        } else if optimized_size > original_size {
            let increase = optimized_size - original_size;
            println!("   Increase:  {}", format_file_size(increase));
        } else {
            println!("   Size:      No change");
        }
        
        println!("   Duration:  {}", duration);
        println!();
    }

    /// Show batch processing summary for WASM
    pub fn show_batch_summary(
        successful: usize,
        failed: usize,
        total_saved: u64,
        duration: &str,
    ) {
        println!("📊 Batch Processing Summary");
        println!("   Files processed: {}", successful + failed);
        println!("   Successful:      {}", successful);
        if failed > 0 {
            println!("   Failed:          {}", failed);
        }
        if total_saved > 0 {
            println!("   Total saved:     {}", format_file_size(total_saved));
        }
        println!("   Total duration:  {}", duration);
        println!();
    }

    /// Show usage examples for WASM
    pub fn show_usage_examples() {
        println!("💡 Usage Examples");
        println!();
        println!("  Image Optimization:");
        println!("    - Upload PNG/JPEG/WebP files via drag & drop");
        println!("    - Adjust quality settings in the sidebar");
        println!();
        println!("  Mesh Processing:");
        println!("    - Upload OBJ/PLY/STL files");
        println!("    - Configure decimation settings");
        println!();
        println!("  Batch Processing:");
        println!("    - Select multiple files at once");
        println!("    - Process all with same settings");
        println!();
    }
}

/// Format file sizes in human-readable format
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
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
