extern crate alloc;
use alloc::{string::String, format, string::ToString};

#[cfg(feature = "web-sys")]
use web_sys::console;

pub struct UserFeedback;

impl UserFeedback {
    pub fn show_welcome() {
        #[cfg(feature = "web-sys")]
        {
            console::log_1(&"Pixie Juice".into());
            console::log_1(&"Version 0.1.0 - Rust + C Hotspots for WebAssembly".into());
            console::log_1(&"Optimized for browser execution with maximum performance".into());
        }
        
        #[cfg(feature = "tracing")]
        tracing::info!("Pixie Juice initialized");
    }

    pub fn show_error(error: &str, suggestion: Option<&str>) {
        #[cfg(feature = "web-sys")]
        {
            console::error_1(&format!("Error: {}", error).into());
            if let Some(hint) = suggestion {
                console::warn_1(&format!("Hint: {}", hint).into());
            }
        }
        
        #[cfg(feature = "tracing")]
        {
            if let Some(hint) = suggestion {
                tracing::error!("Error: {} | Hint: {}", error, hint);
            } else {
                tracing::error!("Error: {}", error);
            }
        }
    }

    pub fn show_success(message: &str, stats: Option<&str>) {
        #[cfg(feature = "web-sys")]
        {
            console::log_1(&format!("Success: {}", message).into());
            if let Some(metrics) = stats {
                console::log_1(&format!("Stats: {}", metrics).into());
            }
        }
        
        #[cfg(feature = "tracing")]
        {
            if let Some(metrics) = stats {
                tracing::info!("Success: {} | Stats: {}", message, metrics);
            } else {
                tracing::info!("Success: {}", message);
            }
        }
    }

    pub fn show_warning(message: &str) {
        #[cfg(feature = "web-sys")]
        console::warn_1(&format!("Warning: {}", message).into());
        
        #[cfg(feature = "tracing")]
        tracing::warn!("Warning: {}", message);
    }

    pub fn show_info(message: &str) {
        #[cfg(feature = "web-sys")]
        console::log_1(&format!("Info: {}", message).into());
        
        #[cfg(feature = "tracing")]
        tracing::info!("Info: {}", message);
    }

    pub fn file_processing_error(filename: &str, error: &str) {
        Self::show_error(
            &format!("Failed to process file '{}': {}", filename, error),
            Some("Check file format and try again. Ensure the file is supported.")
        );
    }

    pub fn unsupported_format(format: &str, supported: &[&str]) {
        Self::show_error(
            &format!("Unsupported format: {}", format),
            Some(&format!(
                "Supported formats: {}. Check the file extension and format.",
                supported.join(", ")
            ))
        );
    }

    pub fn c_hotspot_status(enabled: bool, operation: &str) {
        if enabled {
            Self::show_info(&format!("C hotspot enabled for {}: Maximum performance", operation));
        } else {
            Self::show_warning(&format!("Using Rust fallback for {}: Good performance", operation));
        }
    }

    pub fn show_performance_metrics(
        input_size: usize,
        output_size: usize,
        processing_time_ms: f64,
        c_hotspots_used: u32,
        rust_fallbacks: u32,
    ) {
        let compression_ratio = if input_size > 0 {
            (output_size as f64 / input_size as f64 * 100.0).round()
        } else {
            100.0
        };
        
        let metrics = format!(
            "Size: {} → {} bytes ({:.1}% of original) | Time: {:.2}ms | C hotspots: {} | Rust: {}",
            Self::format_bytes(input_size),
            Self::format_bytes(output_size),
            compression_ratio,
            processing_time_ms,
            c_hotspots_used,
            rust_fallbacks
        );
        
        Self::show_success("Processing completed", Some(&metrics));
    }

    #[cfg(feature = "web-sys")]
    pub fn report_progress(operation: &str, progress: f32) {
        let progress_percent = (progress * 100.0).round() as u32;
        console::log_1(&format!("🔄 {}: {}%", operation, progress_percent).into());
        
        #[cfg(feature = "tracing")]
        tracing::info!("Progress - {}: {}%", operation, progress_percent);
    }

    fn format_bytes(bytes: usize) -> String {
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

    #[cfg(feature = "web-sys")]
    pub fn report_memory_usage() {
        if let Some(_performance) = web_sys::window().and_then(|w| w.performance()) {
            #[cfg(feature = "tracing")]
            tracing::debug!("Memory usage reporting available via Performance API");
        }
    }

    pub fn threading_status(enabled: bool, thread_count: usize) {
        if enabled {
            let count_str = if thread_count == 0 { "auto".to_string() } else { thread_count.to_string() };
            Self::show_info(&format!("Threading enabled: {} threads", count_str));
        } else {
            Self::show_info("Single-threaded mode");
        }
    }

    pub fn wasm_initialization_complete() {
        #[cfg(feature = "web-sys")]
        {
            console::log_1(&"WASM module loaded successfully".into());
            console::log_1(&"Ready for high-performance media processing".into());
        }
        
        #[cfg(feature = "tracing")]
        tracing::info!("WASM initialization complete - ready for processing");
    }

    pub fn feature_status(feature: &str, available: bool) {
        if available {
            Self::show_info(&format!("✅ {} available", feature));
        } else {
            Self::show_warning(&format!("❌ {} not available", feature));
        }
    }

    pub fn mesh_optimization_details(
        original_vertices: usize,
        original_faces: usize,
        optimized_vertices: usize,
        optimized_faces: usize,
        reduction_ratio: f64,
    ) {
        let vertex_reduction = if original_vertices > 0 {
            (1.0 - optimized_vertices as f64 / original_vertices as f64) * 100.0
        } else {
            0.0
        };
        let face_reduction = if original_faces > 0 {
            (1.0 - optimized_faces as f64 / original_faces as f64) * 100.0
        } else {
            0.0
        };
        
        let details = format!(
            "🔺 Mesh: {} → {} vertices ({:.1}% reduction) | {} → {} faces ({:.1}% reduction) | Target: {:.1}%",
            original_vertices, optimized_vertices, vertex_reduction,
            original_faces, optimized_faces, face_reduction,
            reduction_ratio * 100.0
        );
        
        Self::show_success("Mesh optimization completed", Some(&details));
    }

    pub fn batch_summary(
        total_files: usize,
        processed: usize,
        errors: usize,
        total_saved_bytes: i64,
    ) {
        let summary = if total_saved_bytes > 0 {
            format!(
                "Batch: {}/{} files processed, {} errors, {} saved",
                processed, total_files, errors, Self::format_bytes(total_saved_bytes as usize)
            )
        } else if total_saved_bytes < 0 {
            format!(
                "Batch: {}/{} files processed, {} errors, {} increased",
                processed, total_files, errors, Self::format_bytes((-total_saved_bytes) as usize)
            )
        } else {
            format!(
                "Batch: {}/{} files processed, {} errors",
                processed, total_files, errors
            )
        };
        
        Self::show_success("Batch processing completed", Some(&summary));
    }

    pub fn debug_mode_enabled() {
        Self::show_info("Debug mode enabled - verbose output active");
    }

    pub fn experimental_feature_warning(feature: &str) {
        Self::show_warning(&format!(
            "Using experimental feature: {}. Results may vary.",
            feature
        ));
    }
}
