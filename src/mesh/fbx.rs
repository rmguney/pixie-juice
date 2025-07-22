//! FBX format support with basic ASCII FBX parsing and binary detection

use crate::types::{OptConfig, OptError, OptResult};
use crate::ffi::mesh_ffi::{MeshData, decimate_mesh_safe, weld_vertices_safe};

/// Optimize FBX format
pub fn optimize_fbx(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Check if it's binary or ASCII FBX
    if data.len() < 20 {
        return Err(OptError::InvalidInput("File too small for FBX".to_string()));
    }
    
    if data.starts_with(b"Kaydara FBX Binary") {
        // Binary FBX - currently unsupported for optimization
        log::warn!("Binary FBX optimization not yet implemented - returning original data");
        Ok(data.to_vec())
    } else {
        // Try to parse as ASCII FBX
        let text = String::from_utf8_lossy(data);
        if text.contains("FBXHeaderExtension") || text.contains("FBXVersion") {
            optimize_ascii_fbx(data, config)
        } else {
            Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
        }
    }
}

/// Optimize ASCII FBX format
fn optimize_ascii_fbx(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    let text = String::from_utf8_lossy(data);
    
    // Extract mesh data from ASCII FBX
    let mesh_data = extract_ascii_fbx_geometry(&text)?;
    
    if mesh_data.vertices.is_empty() {
        return Err(OptError::InvalidFormat("No geometry found in FBX file".to_string()));
    }
    
    // Apply mesh optimization
    let mut optimized_mesh = mesh_data;
    
    // Apply vertex welding to remove duplicates
    optimized_mesh = weld_vertices_safe(&optimized_mesh, 0.0001)?; // 0.1mm tolerance
    
    // Apply mesh decimation if requested
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction > 0.0 && target_reduction < 1.0 {
            let target_ratio = 1.0 - target_reduction; // Convert reduction to keep ratio
            optimized_mesh = decimate_mesh_safe(&optimized_mesh, target_ratio)?;
        }
    }
    
    // Rebuild ASCII FBX with optimized geometry
    rebuild_ascii_fbx_with_mesh(&text, &optimized_mesh)
}

/// Extract geometry data from ASCII FBX
pub fn extract_ascii_fbx_geometry(text: &str) -> OptResult<MeshData> {
    let mut mesh = MeshData::new();
    let lines: Vec<&str> = text.lines().collect();
    
    let mut in_vertices_section = false;
    let mut in_polygon_index_section = false;
    
    for line in &lines {
        let trimmed = line.trim();
        
        // Look for Vertices property start
        if trimmed.starts_with("Vertices:") {
            in_vertices_section = true;
            continue;
        }
        
        // Look for PolygonVertexIndex property start
        if trimmed.starts_with("PolygonVertexIndex:") {
            in_polygon_index_section = true;
            continue;
        }
        
        // Process vertices data
        if in_vertices_section {
            if trimmed.starts_with("a:") {
                // Extract vertex data after "a:"
                let vertex_data = &trimmed[2..]; // Skip "a:"
                
                // Extract numbers from the vertex data
                let numbers: Vec<f32> = vertex_data
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                
                mesh.vertices = numbers;
                in_vertices_section = false;
            } else if trimmed == "}" {
                in_vertices_section = false;
            }
        }
        
        // Process polygon index data
        if in_polygon_index_section {
            if trimmed.starts_with("a:") {
                // Extract index data after "a:"
                let index_data = &trimmed[2..]; // Skip "a:"
                
                // Extract integer indices
                let indices: Vec<i32> = index_data
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                
                // Convert FBX polygon indices to triangle indices
                // FBX uses negative indices to mark end of polygon
                let mut current_polygon = Vec::new();
                
                for &idx in &indices {
                    if idx < 0 {
                        // End of polygon (negative index means bitwise NOT of actual index)
                        current_polygon.push((!idx) as u32);
                        
                        // Triangulate polygon if it has 3+ vertices
                        if current_polygon.len() >= 3 {
                            // Fan triangulation
                            for i in 1..(current_polygon.len() - 1) {
                                mesh.indices.push(current_polygon[0]);
                                mesh.indices.push(current_polygon[i]);
                                mesh.indices.push(current_polygon[i + 1]);
                            }
                        }
                        current_polygon.clear();
                    } else {
                        current_polygon.push(idx as u32);
                    }
                }
                
                in_polygon_index_section = false;
            } else if trimmed == "}" {
                in_polygon_index_section = false;
            }
        }
    }
    
    Ok(mesh)
}

/// Rebuild ASCII FBX with optimized mesh data
fn rebuild_ascii_fbx_with_mesh(original_text: &str, mesh: &MeshData) -> OptResult<Vec<u8>> {
    let lines: Vec<&str> = original_text.lines().collect();
    let mut result = String::new();
    let mut skip_until_next_property = false;
    
    for line in &lines {
        let trimmed = line.trim();
        
        if skip_until_next_property {
            // Skip lines until we reach a new property or close brace
            if trimmed.starts_with('}') || (trimmed.contains(':') && !trimmed.starts_with("a:")) {
                skip_until_next_property = false;
                result.push_str(line);
                result.push('\n');
            }
            continue;
        }
        
        if trimmed.starts_with("Vertices:") {
            // Replace with optimized vertices
            result.push_str("        Vertices: *");
            result.push_str(&mesh.vertices.len().to_string());
            result.push_str(" {\n");
            result.push_str("            a: ");
            
            // Write vertex data
            for (i, vertex) in mesh.vertices.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                result.push_str(&vertex.to_string());
            }
            result.push_str("\n        } \n");
            skip_until_next_property = true;
            continue;
        }
        
        if trimmed.starts_with("PolygonVertexIndex:") {
            // Replace with optimized indices
            result.push_str("        PolygonVertexIndex: *");
            result.push_str(&mesh.indices.len().to_string());
            result.push_str(" {\n");
            result.push_str("            a: ");
            
            // Write triangle indices (convert back to FBX polygon format)
            for (i, chunk) in mesh.indices.chunks(3).enumerate() {
                if i > 0 {
                    result.push(',');
                }
                if chunk.len() == 3 {
                    result.push_str(&chunk[0].to_string());
                    result.push(',');
                    result.push_str(&chunk[1].to_string());
                    result.push(',');
                    // FBX uses negative index to mark end of polygon (bitwise NOT)
                    result.push_str(&format!("{}", !(chunk[2] as i32)));
                }
            }
            result.push_str("\n        } \n");
            skip_until_next_property = true;
            continue;
        }
        
        result.push_str(line);
        result.push('\n');
    }
    
    Ok(result.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fbx_binary_detection() {
        let config = OptConfig::default();
        
        // Test with FBX binary header
        let fbx_header = b"Kaydara FBX Binary  \x00\x1a\x00";
        let result = optimize_fbx(fbx_header, &config);
        assert!(result.is_ok());
        
        // Should return original data for binary FBX
        let output = result.unwrap();
        assert_eq!(output, fbx_header);
    }
    
    #[test]
    fn test_fbx_ascii_optimization() {
        let config = OptConfig::default();
        
        // Create minimal ASCII FBX content
        let fbx_content = r#"; FBX 7.4.0 project file
FBXHeaderExtension:  {
    FBXHeaderVersion: 1003
    FBXVersion: 7400
}

Objects:  {
    Geometry: 1234567890, "Geometry::Cube", "Mesh" {
        Vertices: *24 {
            a: -1.0,-1.0,1.0,1.0,-1.0,1.0,1.0,1.0,1.0,-1.0,1.0,1.0,-1.0,-1.0,-1.0,1.0,-1.0,-1.0,1.0,1.0,-1.0,-1.0,1.0,-1.0
        } 
        PolygonVertexIndex: *36 {
            a: 0,1,2,-4,4,7,6,-6,0,4,5,-2,2,6,7,-4,0,3,7,-5,5,6,2,-2
        } 
    }
}
"#;
        
        let result = optimize_fbx(fbx_content.as_bytes(), &config);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("FBXHeaderExtension"));
        assert!(output_str.contains("Vertices:"));
        assert!(output_str.contains("PolygonVertexIndex:"));
    }
    
    #[test]
    fn test_fbx_invalid_format() {
        let config = OptConfig::default();
        
        // Test with empty data should fail
        let result = optimize_fbx(&[], &config);
        assert!(result.is_err());
        
        // Test with non-FBX data
        let result = optimize_fbx(b"not fbx data", &config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_ascii_fbx_geometry() {
        let fbx_content = r#"
        Vertices: *9 {
            a: 0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0,0.0
        } 
        PolygonVertexIndex: *3 {
            a: 0,1,-3
        } 
        "#;
        
        let result = extract_ascii_fbx_geometry(fbx_content);
        assert!(result.is_ok());
        
        let mesh = result.unwrap();
        assert_eq!(mesh.vertices.len(), 9);  // 3 vertices * 3 components
        assert_eq!(mesh.indices.len(), 3);   // 1 triangle * 3 indices
    }
}
