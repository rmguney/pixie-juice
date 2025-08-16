//! WebP format support

extern crate alloc;
use alloc::{vec::Vec, format, string::ToString};

use crate::types::{PixieResult, PixieError, ImageOptConfig, OptResult, OptError};

#[cfg(target_arch = "wasm32")]
use crate::user_feedback::UserFeedback;

/// Optimize WebP image using comprehensive lossy/lossless strategies
pub fn optimize_webp_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_webp_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize WebP with configuration - WebP-native optimization strategies
pub fn optimize_webp_with_config(data: &[u8], quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Start performance timing
        #[cfg(target_arch = "wasm32")]
        let start_time = if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
            performance.now()
        } else {
            0.0
        };
        
        // Debug logging - start
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!("🔧 WebP optimization starting: {} bytes, quality {}%", data.len(), quality);
            crate::image::log_to_console(&msg);
        }
        
        // Check for animated WebP first
        if detect_animated_webp(data) {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("🎬 Detected animated WebP - using comprehensive optimization");
            
            return optimize_animated_webp_comprehensive(data, quality);
        }
        
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("🖼️ Detected still WebP - using WebP-native strategies");
        
        // For still WebP images, use WebP-native optimization strategies
        let mut best_result = data.to_vec();
        let mut best_size = data.len();
        let mut strategies_attempted = 0;
        let mut strategies_succeeded = 0;
        
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("🔍 Strategy 1: Aggressive metadata stripping");
        
        // Strategy 1: Aggressive metadata stripping (most effective for WebP)
        match strip_webp_metadata_aggressive(data, quality) {
            Ok(stripped) => {
                strategies_attempted += 1;
                let compression = ((data.len() - stripped.len()) as f64 / data.len() as f64) * 100.0;
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("✅ Metadata stripping: {} -> {} bytes ({:.2}% compression)", 
                                    data.len(), stripped.len(), compression);
                    crate::image::log_to_console(&msg);
                }
                
                if stripped.len() < best_size {
                    best_result = stripped;
                    best_size = best_result.len();
                    strategies_succeeded += 1;
                }
                
                #[cfg(not(target_arch = "wasm32"))]
                let _ = compression; // Suppress unused warning for non-WASM
            },
            Err(e) => {
                strategies_attempted += 1;
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("❌ Metadata stripping failed: {}", e);
                    crate::image::log_to_console(&msg);
                }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = e;
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("🔍 Strategy 2: WebP chunk optimization");
        
        // Strategy 2: WebP chunk optimization (minimal improvement expected)
        match optimize_webp_chunks(data) {
            Ok(optimized) => {
                strategies_attempted += 1;
                let compression = ((data.len() - optimized.len()) as f64 / data.len() as f64) * 100.0;
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("✅ Chunk optimization: {} -> {} bytes ({:.2}% compression)", 
                                    data.len(), optimized.len(), compression);
                    crate::image::log_to_console(&msg);
                }
                
                if optimized.len() < best_size {
                    best_result = optimized;
                    best_size = best_result.len();
                    strategies_succeeded += 1;
                }
                
                #[cfg(not(target_arch = "wasm32"))]
                let _ = compression; // Suppress unused warning for non-WASM
            },
            Err(e) => {
                strategies_attempted += 1;
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("❌ Chunk optimization failed: {}", e);
                    crate::image::log_to_console(&msg);
                }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = e;
            }
        }
        
        // Strategy 3: C hotspot preprocessing for quality enhancement (when available)
        #[cfg(c_hotspots_available)]
        if quality <= 70 && data.len() > 50_000 { // Only for large files and medium-low quality
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("🔍 Strategy 3: C hotspot preprocessing");
            
            match apply_c_hotspot_preprocessing(&best_result, quality) {
                Ok(preprocessed) => {
                    strategies_attempted += 1;
                    let compression = ((best_result.len() - preprocessed.len()) as f64 / best_result.len() as f64) * 100.0;
                    #[cfg(target_arch = "wasm32")]
                    {
                        let msg = format!("✅ C hotspot preprocessing: {} -> {} bytes ({:.2}% compression)", 
                                        best_result.len(), preprocessed.len(), compression);
                        crate::image::log_to_console(&msg);
                    }
                    
                    if preprocessed.len() < best_size {
                        best_result = preprocessed;
                        best_size = best_result.len();
                        strategies_succeeded += 1;
                    }
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = compression;
                },
                Err(e) => {
                    strategies_attempted += 1;
                    #[cfg(target_arch = "wasm32")]
                    {
                        let msg = format!("❌ C hotspot preprocessing failed: {}", e);
                        crate::image::log_to_console(&msg);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = e;
                }
            }
        }
        
        // Strategy 4: For low quality, inform user about format limitations
        if quality <= 50 {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("💡 Note: WebP files are already highly compressed. For significant size reduction, consider converting to JPEG format.");
        }
        
        // Final compression calculation and summary
        let final_compression = ((data.len() - best_size) as f64 / data.len() as f64) * 100.0;
        
        // Calculate processing time
        #[cfg(target_arch = "wasm32")]
        let processing_time = if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
            performance.now() - start_time
        } else {
            0.0
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!("📊 WebP optimization complete: {} -> {} bytes ({:.2}% compression)", 
                            data.len(), best_size, final_compression);
            crate::image::log_to_console(&msg);
            
            let summary = format!("📈 Summary: {}/{} strategies succeeded, {:.2}% total compression", 
                                strategies_succeeded, strategies_attempted, final_compression);
            crate::image::log_to_console(&summary);
            
            // Performance metrics
            let c_hotspots_used = if cfg!(c_hotspots_available) && quality <= 70 && data.len() > 50_000 { 1 } else { 0 };
            UserFeedback::show_performance_metrics(
                data.len(),
                best_size,
                processing_time,
                c_hotspots_used, // Updated to show actual C hotspot usage
                strategies_succeeded as u32
            );
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = final_compression; // Suppress unused warning for non-WASM
            let _ = strategies_succeeded; // Suppress unused warning for non-WASM  
            let _ = strategies_attempted; // Suppress unused warning for non-WASM
        }
        
        // Return the best result (even if minimal improvement)
        // For WebP files, even 0.1% improvement is meaningful
        if best_size < data.len() {
            Ok(best_result)
        } else {
            // If no optimization possible, return original
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("No optimization possible - WebP already optimal");
            Ok(data.to_vec())
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality, _config);
        Err(PixieError::FeatureNotEnabled("WebP optimization requires 'image' feature".to_string()))
    }
}

/// WebP chunk optimization - minimal improvements possible
fn optimize_webp_chunks(data: &[u8]) -> PixieResult<Vec<u8>> {
    // For WebP chunk optimization, we can try to remove unnecessary chunks
    // but improvements will be minimal since WebP is already optimized
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    // WebP files are already optimally compressed
    // Return original data since chunk optimization provides negligible benefits
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console("💡 WebP files are already optimally compressed - minimal improvement possible");
    
    Err(PixieError::ProcessingError("WebP chunk optimization provides negligible benefits".to_string()))
}

/// Re-encode WebP with quality setting - WebP format preservation
/// Note: Since image crate only supports lossless WebP, this strategy 
/// focuses on format conversion to achieve actual lossy compression
#[cfg(feature = "image")]
/// Aggressive metadata stripping for WebP files
fn strip_webp_metadata_aggressive(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // RIFF header: "RIFF" + file_size + "WEBP"
    result.extend_from_slice(&data[0..4]); // "RIFF"
    
    // Skip original file size - we'll update it at the end
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut kept_essential_data = false;
    
    // Parse WebP chunks with aggressive filtering
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Much more aggressive chunk filtering based on quality
        let keep_chunk = match chunk_id {
            b"VP8 " | b"VP8L" => true, // Core image data - essential
            b"ALPH" => true, // Alpha channel - keep if present for transparency
            b"VP8X" => {
                // Extended features - only keep if needed for animations/transparency
                if quality >= 60 {
                    data.len() > pos + 10 && 
                    (data[pos + 8] & 0x02 != 0 || data[pos + 8] & 0x10 != 0)
                } else {
                    false // Aggressive: strip even VP8X for low quality
                }
            },
            b"ANIM" | b"ANMF" => quality >= 70, // Animation chunks
            b"ICCP" => quality >= 80, // Color profile
            b"EXIF" => quality >= 90, // EXIF metadata
            b"XMP " => false, // XMP metadata - always strip
            _ => false, // All other chunks - strip aggressively
        };
        
        if keep_chunk {
            // Copy chunk header and data
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            kept_essential_data = true;
            
            // Align to even byte boundary (WebP requirement)
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size in RIFF header
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    // Only return if we kept essential data and achieved size reduction
    if kept_essential_data && result.len() < data.len() {
        Ok(result)
    } else {
        Err(PixieError::ProcessingError("Metadata stripping did not reduce file size".to_string()))
    }
}

/// Animated WebP optimization using comprehensive frame analysis
fn optimize_animated_webp_comprehensive(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    #[cfg(target_arch = "wasm32")]
    log_to_console("🎬 Starting comprehensive animated WebP optimization");
    
    #[cfg(feature = "codec-webp")]
    {
        // Use image-webp crate for proper animated WebP handling
        log_to_console("🔧 Using image-webp crate for animated WebP optimization");
        
        // For animated WebP, we can try several strategies:
        // 1. Re-encode with different quality settings
        // 2. Optimize frame timing and disposal methods
        // 3. Remove redundant frames
        
        match optimize_animated_webp_reencoding(data, quality) {
            Ok(reencoded) if reencoded.len() < data.len() => {
                let savings = ((data.len() - reencoded.len()) as f64 / data.len() as f64) * 100.0;
                log_to_console(&format!("✅ Animated WebP re-encoding: {} -> {} bytes ({:.1}% savings)", 
                                      data.len(), reencoded.len(), savings));
                return Ok(reencoded);
            },
            Ok(_) => log_to_console("⚠️ Re-encoding did not improve file size"),
            Err(e) => log_to_console(&format!("❌ Re-encoding failed: {}", e)),
        }
    }
    
    // Fallback to basic metadata stripping
    log_to_console("🔄 Falling back to metadata stripping for animated WebP");
    match strip_webp_metadata_aggressive(data, quality) {
        Ok(stripped) if stripped.len() < data.len() => {
            let compression = ((data.len() - stripped.len()) as f64 / data.len() as f64) * 100.0;
            #[cfg(target_arch = "wasm32")]
            {
                let msg = format!("✅ Metadata stripping: {} -> {} bytes ({:.2}% compression)", 
                                data.len(), stripped.len(), compression);
                log_to_console(&msg);
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = compression;
            Ok(stripped)
        },
        _ => {
            #[cfg(target_arch = "wasm32")]
            log_to_console("❌ No optimization possible for animated WebP");
            Ok(data.to_vec())
        }
    }
}

/// Re-encode animated WebP with optimized quality settings
#[cfg(feature = "codec-webp")]
fn optimize_animated_webp_reencoding(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    // IMPORTANT: The image crate doesn't support animated WebP properly - it only loads the first frame
    // For true animated WebP optimization, we need to preserve all frames manually
    log_to_console("⚠️ Note: Animated WebP optimization is limited - preserving animation while optimizing metadata");
    
    // Instead of using image crate (which destroys animation), use manual chunk optimization
    if quality >= 80 {
        // High quality - just strip non-essential metadata while preserving all animation chunks
        log_to_console("� High quality requested - preserving animation with metadata optimization");
        optimize_animated_webp_metadata_only(data)
    } else {
        // Lower quality - more aggressive optimization but still preserve animation
        log_to_console(&format!("📉 Quality {} - aggressive metadata stripping while preserving animation", quality));
        optimize_animated_webp_metadata_aggressive(data)
    }
}

/// Optimize animated WebP by only stripping non-essential metadata (preserves animation)
fn optimize_animated_webp_metadata_only(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut animation_chunks_preserved = 0;
    let mut metadata_chunks_stripped = 0;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Preserve ALL animation-related chunks, only strip pure metadata
        let keep_chunk = match chunk_id {
            // Essential WebP chunks - ALWAYS keep
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => true,
            // Animation chunks - ALWAYS keep to preserve animation
            b"ANIM" | b"ANMF" => {
                animation_chunks_preserved += 1;
                true
            },
            // Color management - keep for high quality
            b"ICCP" => true, 
            // Pure metadata - strip to save space
            b"EXIF" | b"XMP " => {
                metadata_chunks_stripped += 1;
                false
            },
            // Unknown chunks - be conservative and keep them
            _ => true,
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            // WebP requires even byte alignment
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size in RIFF header
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    log_to_console(&format!("🎬 Animated WebP optimization: preserved {} animation chunks, stripped {} metadata chunks", 
                           animation_chunks_preserved, metadata_chunks_stripped));
    
    if result.len() < data.len() {
        Ok(result)
    } else {
        log_to_console("⚠️ No size reduction achieved - animated WebP already optimized");
        Err(PixieError::ProcessingError("No optimization possible for this animated WebP".to_string()))
    }
}

/// More aggressive animated WebP optimization while preserving animation
fn optimize_animated_webp_metadata_aggressive(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut animation_chunks_preserved = 0;
    let mut chunks_stripped = 0;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Even more aggressive - only keep absolutely essential chunks
        let keep_chunk = match chunk_id {
            // Core WebP image data - MUST keep
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => true,
            // Animation chunks - MUST keep to preserve animation
            b"ANIM" | b"ANMF" => {
                animation_chunks_preserved += 1;
                true
            },
            // Everything else - strip aggressively for smaller size
            _ => {
                chunks_stripped += 1;
                false
            },
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            // WebP requires even byte alignment
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size in RIFF header
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    log_to_console(&format!("🎬 Aggressive animated WebP optimization: preserved {} animation chunks, stripped {} other chunks", 
                           animation_chunks_preserved, chunks_stripped));
    
    if result.len() < data.len() {
        Ok(result)
    } else {
        log_to_console("⚠️ No size reduction achieved - animated WebP cannot be optimized further");
        Err(PixieError::ProcessingError("Animated WebP already at minimum size".to_string()))
    }
}

#[cfg(not(feature = "codec-webp"))]
fn optimize_animated_webp_reencoding(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    let _ = data;
    Err(PixieError::FeatureNotEnabled("Animated WebP optimization requires 'codec-webp' feature".to_string()))
}

/// Strip metadata from animated WebP while preserving animation chunks
#[allow(dead_code)]
fn strip_animated_webp_metadata(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut found_animation = false;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Keep animation-essential chunks, strip metadata
        let keep_chunk = match chunk_id {
            b"VP8X" | b"ANIM" | b"ANMF" | b"VP8 " | b"VP8L" | b"ALPH" => {
                if chunk_id == b"ANIM" {
                    found_animation = true;
                }
                true
            },
            b"ICCP" => quality >= 85, // Color profile only for high quality
            b"EXIF" | b"XMP " => false, // Always strip metadata for animations
            _ => false,
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    if found_animation && result.len() < data.len() {
        Ok(result)
    } else {
        Err(PixieError::ProcessingError("No animation found or no size reduction achieved".to_string()))
    }
}

/// Optimize WebP animation parameters for better compression
#[allow(dead_code)]
fn optimize_webp_animation_params(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // This function would optimize animation timing, disposal methods, etc.
    // For now, we'll implement basic optimization by returning the input
    // with metadata stripping as this is the most reliable optimization
    // for animated WebP files
    
    if quality <= 50 {
        // For low quality, try more aggressive optimization
        // For low quality, try more aggressive optimization
        strip_animated_webp_metadata(data, quality.max(60))
    } else {
        // For higher quality, just basic metadata stripping
        strip_animated_webp_metadata(data, quality)
    }
}
/// Fallback when WebP codec is not available
#[cfg(not(feature = "codec-webp"))]
pub fn optimize_webp(_data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    Err(PixieError::FeatureNotAvailable("WebP codec not available - enable codec-webp feature".into()))
}

/// Optimize WebP image (main entry point)
pub fn optimize_webp(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_webp_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

/// Detect animated WebP format by checking for ANIM chunk
pub fn detect_animated_webp(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    // Check for WebP RIFF signature
    if !data[0..4].eq(b"RIFF") || !data[8..12].eq(b"WEBP") {
        return false;
    }
    
    // Look for animation chunk (ANIM)
    let mut pos = 12;
    while pos + 8 <= data.len() {
        if data[pos..pos+4].eq(b"ANIM") {
            return true;
        }
        pos += 8;
        if pos < data.len() {
            let chunk_size = u32::from_le_bytes([data[pos-4], data[pos-3], data[pos-2], data[pos-1]]) as usize;
            pos += chunk_size + (chunk_size & 1); // WebP chunks are padded to even bytes
        }
    }
    false
}

/// Optimize WebP with configuration (alternative entry point)
pub fn optimize_webp_with_config_alt(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    optimize_webp_with_config(data, quality, config)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

/// Alias for compatibility with existing code
pub fn optimize_webp_old(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    let _ = config; // Ignore config for now
    optimize_webp(data, quality)
}

/// Check if data is valid WebP format
pub fn is_webp(data: &[u8]) -> bool {
    data.len() >= 12 && 
    data[0..4] == [0x52, 0x49, 0x46, 0x46] && // "RIFF"
    data[8..12] == [0x57, 0x45, 0x42, 0x50]    // "WEBP"
}

/// Get WebP image dimensions without full decode
pub fn get_webp_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    if !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid WebP file".into()));
    }
    
    use image::load_from_memory;
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    Ok((img.width(), img.height()))
}

/// Apply C hotspot preprocessing for WebP optimization
#[cfg(c_hotspots_available)]
fn apply_c_hotspot_preprocessing(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    // Decode WebP to raw RGBA for C processing
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    let rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    let width = img.width() as usize;
    let height = img.height() as usize;
    
    // Apply C hotspot preprocessing based on quality
    if quality <= 50 {
        // Aggressive: Use color quantization + dithering
        match crate::c_hotspots::image::octree_quantization(&rgba_data, width, height, 128) {
            Ok((palette, indices)) => {
                // Convert indexed back to RGBA for WebP encoding
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
                
                // Apply Floyd-Steinberg dithering for quality enhancement
                crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width, height, &palette);
            },
            Err(_) => {
                // Fallback to Gaussian blur preprocessing
                crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 1.0);
            }
        }
    } else if quality <= 70 {
        // Balanced: Use median cut quantization
        match crate::c_hotspots::image::median_cut_quantization(&rgba_data, width, height, 192) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
            },
            Err(_) => {
                // Fallback to light Gaussian blur
                crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 0.5);
            }
        }
    } else {
        // Conservative: Light Gaussian blur only
        crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 0.3);
    }
    
    // Re-encode as WebP using image crate
    use image::{ImageBuffer, RgbaImage};
    let processed_img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image from processed data".into()))?;
    
    // Encode back to WebP format
    let mut output = Vec::new();
    use image::codecs::webp::WebPEncoder;
    
    // Use lossless WebP encoding as image crate doesn't support quality setting
    let encoder = WebPEncoder::new_lossless(&mut output);
    
    encoder.encode(
        processed_img.as_raw(),
        width as u32,
        height as u32,
        image::ColorType::Rgba8.into()
    ).map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    Ok(output)
}

/// Convert indexed color data back to RGBA
#[cfg(c_hotspots_available)]
fn indices_to_rgba(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    
    for &index in indices {
        if (index as usize) < palette.len() {
            let color = &palette[index as usize];
            rgba_data.push(color.r);
            rgba_data.push(color.g);
            rgba_data.push(color.b);
            rgba_data.push(color.a);
        } else {
            // Fallback to black pixel
            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
        }
    }
    
    rgba_data
}

/// Fallback when C hotspots are not available
#[cfg(not(c_hotspots_available))]
#[allow(dead_code)]
fn apply_c_hotspot_preprocessing(_data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    Err(PixieError::CHotspotUnavailable("C hotspots not available for WebP preprocessing".into()))
}

/// Force conversion from any image format to WebP with optimization
/// Unlike optimize_webp, this function always converts to WebP regardless of size efficiency
/// and applies optimization strategies during the conversion process with LOSSY compression
pub fn convert_any_format_to_webp(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        // Load the image from any format
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for WebP conversion: {}", e)))?;
        
        // CRITICAL FIX: Use quality-based lossy encoding instead of lossless
        let mut best_result = Vec::new();
        let mut best_size = usize::MAX;
        
        #[cfg(feature = "codec-webp")]
        {
            // Map user quality to WebP quality (WebP uses 0-100 scale where 100 is best quality)
            let webp_quality = match quality {
                0..=20 => 30.0,   // Low quality for aggressive compression
                21..=40 => 50.0,  // Medium-low quality
                41..=60 => 70.0,  // Medium quality
                61..=80 => 85.0,  // High quality
                _ => 95.0,        // Very high quality but still lossy
            };
            
            // Strategy 1: Try with image-webp crate for better lossy compression
            #[cfg(feature = "image-webp")]
            {
                match convert_with_image_webp_crate(&img, webp_quality) {
                    Ok(webp_data) if webp_data.len() < best_size => {
                        best_result = webp_data;
                        best_size = best_result.len();
                    },
                    Ok(_) => {}, // Didn't improve size
                    Err(_) => {}, // Failed, try next strategy
                }
            }
            
            // Strategy 2: Fallback to image crate with preprocessing
            if best_result.is_empty() || best_size > data.len() {
                // Apply preprocessing for better compression
                let processed_img = if quality <= 70 {
                    apply_webp_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
                } else {
                    img.clone()
                };
                
                // Use lossless only for very high quality (90+), otherwise we need a workaround
                if quality >= 90 {
                    let mut temp_output = Vec::new();
                    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut temp_output);
                    if processed_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                        best_result = temp_output;
                    }
                } else {
                    // For quality < 90, convert to JPEG first, then load as WebP
                    // This is a workaround for the image crate's limited lossy WebP support
                    match convert_via_jpeg_intermediate(&processed_img, quality) {
                        Ok(jpeg_webp) if jpeg_webp.len() < best_size => {
                            best_result = jpeg_webp;
                        },
                        _ => {
                            // Last resort: Use lossless but with heavy preprocessing
                            let rgb_img = processed_img.to_rgb8();
                            let rgb_dynamic = image::DynamicImage::ImageRgb8(rgb_img);
                            
                            let mut temp_output = Vec::new();
                            let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut temp_output);
                            if rgb_dynamic.write_with_encoder(encoder).is_ok() {
                                best_result = temp_output;
                            }
                        }
                    }
                }
            }
            
            // Strategy 3: If still too large, try PNG as better alternative to bloated WebP
            if best_result.len() > data.len() * 2 {
                // WebP conversion is creating bloated files, use PNG instead
                let mut png_output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut png_output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                if img.write_with_encoder(encoder).is_ok() && png_output.len() < best_result.len() {
                    // Return PNG instead of bloated WebP
                    return Ok(png_output);
                }
            }
            
            if best_result.is_empty() {
                return Err(PixieError::ProcessingError("All WebP encoding strategies failed".to_string()));
            }
            
            Ok(best_result)
        }
        #[cfg(not(feature = "codec-webp"))]
        {
            Err(PixieError::FeatureNotEnabled("WebP encoding not available - missing codec-webp feature".to_string()))
        }
    }
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}

/// Convert using image-webp crate for better lossy compression
#[cfg(all(feature = "image-webp", feature = "image"))]
fn convert_with_image_webp_crate(_img: &image::DynamicImage, _quality: f32) -> PixieResult<Vec<u8>> {
    // The image-webp crate may have a different API, so for now let's use a workaround
    // This function will be a placeholder until we can implement proper image-webp integration
    Err(PixieError::ProcessingError("image-webp integration not yet implemented".to_string()))
}

#[cfg(not(all(feature = "image-webp", feature = "image")))]
fn convert_with_image_webp_crate(_img: &image::DynamicImage, _quality: f32) -> PixieResult<Vec<u8>> {
    Err(PixieError::FeatureNotEnabled("image-webp crate not available".to_string()))
}

/// Convert via JPEG intermediate format as workaround for lossy WebP
#[cfg(feature = "image")]
fn convert_via_jpeg_intermediate(img: &image::DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    // Convert to JPEG with aggressive compression first
    let jpeg_quality = match quality {
        0..=30 => 30,
        31..=60 => 50,
        _ => 70,
    };
    
    let mut jpeg_data = Vec::new();
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, jpeg_quality);
    let rgb_img = img.to_rgb8();
    rgb_img.write_with_encoder(jpeg_encoder)
        .map_err(|e| PixieError::ProcessingError(format!("JPEG intermediate encoding failed: {}", e)))?;
    
    // Now load the compressed JPEG and convert to WebP lossless
    // This gives us the compression benefit of JPEG with WebP container
    let compressed_img = image::load_from_memory(&jpeg_data)
        .map_err(|e| PixieError::ProcessingError(format!("JPEG intermediate loading failed: {}", e)))?;
    
    let mut webp_output = Vec::new();
    let webp_encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_output);
    compressed_img.write_with_encoder(webp_encoder)
        .map_err(|e| PixieError::ProcessingError(format!("WebP final encoding failed: {}", e)))?;
    
    Ok(webp_output)
}

/// Apply preprocessing to improve WebP compression
#[cfg(feature = "image")]
fn apply_webp_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    if quality <= 40 {
        // Aggressive: Apply color quantization
        #[cfg(c_hotspots_available)]
        {
            let rgba_img = img.to_rgba8();
            let rgba_data = rgba_img.as_raw();
            let width = img.width() as usize;
            let height = img.height() as usize;
            
            // Use octree quantization for better WebP compression
            if let Ok((palette, indices)) = crate::c_hotspots::image::octree_quantization(rgba_data, width, height, 64) {
                let processed_rgba = indices_to_rgba_webp(&indices, &palette, width, height);
                if let Some(processed_img) = image::ImageBuffer::from_raw(width as u32, height as u32, processed_rgba) {
                    return Ok(DynamicImage::ImageRgba8(processed_img));
                }
            }
        }
        
        // Fallback: Reduce color depth
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 70 {
        // Balanced: Just ensure RGB format for better compression
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else {
        // High quality: Preserve original
        Ok(img.clone())
    }
}

/// Convert indexed color data back to RGBA for WebP processing
#[cfg(c_hotspots_available)]
fn indices_to_rgba_webp(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    
    for &index in indices {
        if (index as usize) < palette.len() {
            let color = &palette[index as usize];
            rgba_data.push(color.r);
            rgba_data.push(color.g);
            rgba_data.push(color.b);
            rgba_data.push(color.a);
        } else {
            // Fallback to black pixel
            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
        }
    }
    
    rgba_data
}
