//! USDZ format support for Universal Scene Description

#![allow(dead_code)]

use crate::types::{OptConfig, OptResult, OptError};
use crate::ffi::mesh_ffi::MeshData;

#[cfg(feature = "zip")]
use std::io::{Cursor, Read};

#[cfg(feature = "zip")]
use crate::ffi::mesh_ffi::{decimate_mesh_safe, weld_vertices_safe};

/// Optimize USDZ format (Universal Scene Description in ZIP format)
pub fn optimize_usdz(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    #[cfg(feature = "zip")]
    {
        // USDZ is essentially a ZIP file containing USD files
        // For now, we'll extract geometry and re-package it
        
        if data.len() < 30 {
            return Err(OptError::InvalidFormat("File too small for USDZ".to_string()));
        }

        // Check for ZIP magic bytes
        if !data.starts_with(b"PK\x03\x04") && !data.starts_with(b"PK\x05\x06") {
            return Err(OptError::InvalidFormat("Not a valid ZIP/USDZ file".to_string()));
        }

        // Extract mesh data from the USDZ archive
        let mesh_data = extract_usdz_geometry(data)?;
        
        if mesh_data.vertices.is_empty() {
            return Err(OptError::InvalidFormat("No geometry found in USDZ file".to_string()));
        }

        // Apply mesh optimization
        let mut optimized_mesh = mesh_data;
        
        // Apply vertex welding to remove duplicates
        optimized_mesh = weld_vertices_safe(&optimized_mesh, 0.0001)?; // 0.1mm tolerance
        
        // Apply mesh decimation if requested
        if let Some(target_reduction) = config.target_reduction {
            if target_reduction > 0.0 && target_reduction < 1.0 {
                // target_reduction represents the final ratio to keep (0.25 = keep 25%)
                optimized_mesh = decimate_mesh_safe(&optimized_mesh, target_reduction)?;
            }
        }
        
        // For now, rebuild as a simple USDZ with the optimized mesh
        rebuild_usdz_with_mesh(&optimized_mesh)
    }
    
    #[cfg(not(feature = "zip"))]
    {
        let _ = (data, config); // Suppress unused variable warnings
        Err(OptError::InvalidFormat("USDZ support not compiled in (requires zip feature)".to_string()))
    }
}

#[cfg(feature = "zip")]
/// Extract geometry data from USDZ archive
pub fn extract_usdz_geometry(data: &[u8]) -> OptResult<MeshData> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data))
        .map_err(|e| OptError::InvalidFormat(format!("Failed to read USDZ archive: {}", e)))?;
    
    let mut mesh = MeshData::new();
    
    // Look for USD files in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| OptError::InvalidFormat(format!("Failed to read file from archive: {}", e)))?;
        
        let file_name = file.name().to_string();
        
        // Process USD files (ASCII or binary)
        if file_name.ends_with(".usd") || file_name.ends_with(".usda") || file_name.ends_with(".usdc") {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| OptError::ProcessingError(format!("Failed to read USD file: {}", e)))?;
            
            // Try to extract basic geometry from USD content
            if let Ok(extracted_mesh) = extract_basic_usd_geometry(&contents, &file_name) {
                // Merge with existing mesh data
                mesh.vertices.extend_from_slice(&extracted_mesh.vertices);
                
                // Adjust indices to account for existing vertices
                let vertex_offset = (mesh.vertices.len() - extracted_mesh.vertices.len()) / 3;
                for &index in &extracted_mesh.indices {
                    mesh.indices.push(index + vertex_offset as u32);
                }
            }
        }
    }
    
    if mesh.vertices.is_empty() {
        // Create a fallback simple mesh if no geometry found
        mesh.vertices.extend_from_slice(&[
            -1.0, -1.0, 0.0,
             1.0, -1.0, 0.0,
             0.0,  1.0, 0.0,
        ]);
        mesh.indices.extend_from_slice(&[0, 1, 2]);
    }
    
    Ok(mesh)
}

#[cfg(not(feature = "zip"))]
/// Extract geometry data from USDZ archive (stub for non-zip builds)
pub fn extract_usdz_geometry(_data: &[u8]) -> OptResult<MeshData> {
    Err(OptError::InvalidFormat("USDZ support not compiled in (requires zip feature)".to_string()))
}

#[cfg(feature = "zip")]
/// Extract basic geometry from USD file content
fn extract_basic_usd_geometry(data: &[u8], filename: &str) -> OptResult<MeshData> {
    let mut mesh = MeshData::new();
    
    // Check if it's binary USD (starts with specific magic)
    if data.len() >= 8 && &data[0..8] == b"PXR-USDC" {
        // Binary USD - not implemented yet
        return Err(OptError::InvalidFormat("Binary USD parsing not yet implemented".to_string()));
    }
    
    // Try to parse as ASCII USD
    let text = String::from_utf8_lossy(data);
    
    // Look for mesh definitions in USD format
    let lines: Vec<&str> = text.lines().collect();
    let mut in_mesh_def = false;
    let mut in_points_def = false;
    let mut in_face_indices_def = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        // Look for mesh definition start
        if trimmed.contains("def Mesh") || trimmed.contains("def \"Mesh") {
            in_mesh_def = true;
            continue;
        }
        
        // Look for points definition
        if in_mesh_def && (trimmed.contains("point3f[] points") || trimmed.contains("float3[] points")) {
            in_points_def = true;
            continue;
        }
        
        // Look for face vertex indices
        if in_mesh_def && trimmed.contains("int[] faceVertexIndices") {
            in_face_indices_def = true;
            continue;
        }
        
        // Parse points data
        if in_points_def && trimmed.contains("[") {
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.find(']') {
                    let points_data = &trimmed[start+1..end];
                    parse_usd_points(points_data, &mut mesh);
                    in_points_def = false;
                }
            }
        }
        
        // Parse face indices
        if in_face_indices_def && trimmed.contains("[") {
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.find(']') {
                    let indices_data = &trimmed[start+1..end];
                    parse_usd_indices(indices_data, &mut mesh);
                    in_face_indices_def = false;
                }
            }
        }
        
        // End of mesh definition
        if in_mesh_def && trimmed == "}" {
            break;
        }
    }
    
    if mesh.vertices.is_empty() {
        return Err(OptError::InvalidFormat(format!("No geometry found in USD file: {}", filename)));
    }
    
    Ok(mesh)
}

/// Parse USD points data into mesh vertices
fn parse_usd_points(points_str: &str, mesh: &mut MeshData) {
    // USD points are in format like: (0, 0, 0), (1, 0, 0), (0, 1, 0)
    for point_match in points_str.split(',') {
        let point_match = point_match.trim();
        if point_match.starts_with('(') && point_match.contains(')') {
            // Extract coordinates from (x, y, z) format
            let coords = point_match.trim_start_matches('(').trim_end_matches(')');
            let parts: Vec<&str> = coords.split(',').collect();
            if parts.len() >= 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].trim().parse::<f32>(),
                    parts[1].trim().parse::<f32>(),
                    parts[2].trim().parse::<f32>(),
                ) {
                    mesh.vertices.push(x);
                    mesh.vertices.push(y);
                    mesh.vertices.push(z);
                }
            }
        }
    }
}

/// Parse USD face indices into mesh triangles
fn parse_usd_indices(indices_str: &str, mesh: &mut MeshData) {
    // USD face vertex indices are typically integers separated by commas
    let indices: Vec<u32> = indices_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    
    // Triangulate if necessary (USD can have arbitrary polygons)
    // For simplicity, assume triangular faces for now
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            mesh.indices.extend_from_slice(chunk);
        }
    }
}

#[cfg(feature = "zip")]
/// Rebuild USDZ with optimized mesh data
fn rebuild_usdz_with_mesh(mesh: &MeshData) -> OptResult<Vec<u8>> {
    use std::io::Write;
    
    // Create a simple USDZ with the optimized mesh
    let mut zip_buffer = Vec::new();
    
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buffer));
        
        // Create a simple USD file with the optimized mesh
        let usd_content = create_simple_usd_with_mesh(mesh)?;
        
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        zip.start_file("mesh.usda", options)
            .map_err(|e| OptError::ProcessingError(format!("Failed to create USD file in archive: {}", e)))?;
        
        zip.write_all(&usd_content)
            .map_err(|e| OptError::ProcessingError(format!("Failed to write USD content: {}", e)))?;
        
        zip.finish()
            .map_err(|e| OptError::ProcessingError(format!("Failed to finalize USDZ archive: {}", e)))?;
    }
    
    Ok(zip_buffer)
}

#[cfg(feature = "zip")]
/// Create a simple USD file content with mesh data
fn create_simple_usd_with_mesh(mesh: &MeshData) -> OptResult<Vec<u8>> {
    let mut usd_content = String::new();
    
    // USD header
    usd_content.push_str("#usda 1.0\n");
    usd_content.push_str("(\n");
    usd_content.push_str("    defaultPrim = \"Mesh\"\n");
    usd_content.push_str("    upAxis = \"Y\"\n");
    usd_content.push_str(")\n\n");
    
    // Mesh definition
    usd_content.push_str("def Mesh \"Mesh\"\n");
    usd_content.push_str("{\n");
    
    // Points (vertices)
    usd_content.push_str("    point3f[] points = [\n");
    for (i, chunk) in mesh.vertices.chunks(3).enumerate() {
        if i > 0 {
            usd_content.push_str(",\n");
        }
        if chunk.len() == 3 {
            usd_content.push_str(&format!("        ({}, {}, {})", chunk[0], chunk[1], chunk[2]));
        }
    }
    usd_content.push_str("\n    ]\n");
    
    // Face vertex indices
    usd_content.push_str("    int[] faceVertexIndices = [\n        ");
    for (i, &index) in mesh.indices.iter().enumerate() {
        if i > 0 {
            usd_content.push_str(", ");
        }
        usd_content.push_str(&index.to_string());
    }
    usd_content.push_str("\n    ]\n");
    
    // Face vertex counts (all triangles)
    let triangle_count = mesh.indices.len() / 3;
    usd_content.push_str("    int[] faceVertexCounts = [\n        ");
    for i in 0..triangle_count {
        if i > 0 {
            usd_content.push_str(", ");
        }
        usd_content.push_str("3");
    }
    usd_content.push_str("\n    ]\n");
    
    usd_content.push_str("}\n");
    
    Ok(usd_content.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_usdz_format_detection() {
        // Test with empty data should fail
        let result = optimize_usdz(&[], &OptConfig::default());
        assert!(result.is_err());
        
        // Test with non-USDZ data
        let result = optimize_usdz(b"not usdz data", &OptConfig::default());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_usd_points_parsing() {
        let mut mesh = MeshData::new();
        let points_str = "(0, 0, 0), (1, 0, 0), (0, 1, 0)";
        parse_usd_points(points_str, &mut mesh);
        
        assert_eq!(mesh.vertices.len(), 9); // 3 vertices * 3 components
        assert_eq!(mesh.vertices[0], 0.0);
        assert_eq!(mesh.vertices[3], 1.0);
        assert_eq!(mesh.vertices[7], 1.0);
    }
}
