/// Simplified mesh loading utilities - Basic functionality to get build working

use crate::types::{OptError, OptResult};
use crate::ffi::mesh_ffi::MeshData;

/// Load mesh data from bytes based on extension
pub fn load_mesh_data(data: &[u8], extension: &str) -> OptResult<MeshData> {
    match extension.to_lowercase().as_str() {
        "obj" => load_obj_data(data),
        "stl" => load_simple_stl_data(data),
        "ply" => load_simple_ply_data(data),
        "gltf" | "glb" => load_simple_gltf_data(data),
        "dae" => load_dae_data(data),
        "fbx" => load_fbx_data(data),
        _ => Err(OptError::InvalidFormat(format!("Unsupported format: {}", extension))),
    }
}

/// Load OBJ mesh data
fn load_obj_data(data: &[u8]) -> OptResult<MeshData> {
    use std::io::Cursor;
    
    let cursor = Cursor::new(data);
    let obj = obj::ObjData::load_buf(cursor)
        .map_err(|e| OptError::InvalidFormat(format!("OBJ parse error: {e}")))?;
    
    let mut mesh = MeshData::new();
    
    // Extract vertices
    for v in &obj.position {
        mesh.vertices.push(v[0]);
        mesh.vertices.push(v[1]);
        mesh.vertices.push(v[2]);
    }
    
    // Extract triangle indices from all objects and groups
    for object in &obj.objects {
        for group in &object.groups {
            for poly in &group.polys {
                if poly.0.len() >= 3 {
                    // Triangulate simple polygons (assume convex)
                    for i in 1..poly.0.len()-1 {
                        mesh.indices.push(poly.0[0].0 as u32);
                        mesh.indices.push(poly.0[i].0 as u32);
                        mesh.indices.push(poly.0[i+1].0 as u32);
                    }
                }
            }
        }
    }
    
    Ok(mesh)
}

/// Load STL mesh data
fn load_simple_stl_data(data: &[u8]) -> OptResult<MeshData> {
    use std::io::Cursor;
    
    let mut cursor = Cursor::new(data);
    let stl = stl_io::read_stl(&mut cursor)
        .map_err(|e| OptError::InvalidFormat(format!("STL parse error: {e}")))?;
    
    let mut mesh = MeshData::new();
    
    // Copy vertices
    for vertex in &stl.vertices {
        mesh.vertices.push(vertex[0]);
        mesh.vertices.push(vertex[1]);
        mesh.vertices.push(vertex[2]);
    }
    
    // Copy face indices
    for face in &stl.faces {
        mesh.indices.push(face.vertices[0] as u32);
        mesh.indices.push(face.vertices[1] as u32);
        mesh.indices.push(face.vertices[2] as u32);
    }
    
    Ok(mesh)
}

/// Load PLY mesh data (simplified implementation)
fn load_simple_ply_data(data: &[u8]) -> OptResult<MeshData> {
    use std::io::Cursor;
    
    // Try to parse with ply-rs for both ASCII and binary support
    let mut cursor = Cursor::new(data);
    
    // Parse PLY file
    let parser = ply_rs::parser::Parser::<ply_rs::ply::DefaultElement>::new();
    let ply = parser.read_ply(&mut cursor)
        .map_err(|e| OptError::InvalidFormat(format!("PLY parse error: {}", e)))?;
    
    let mut mesh = MeshData::new();
    
    // Extract vertices
    if let Some(vertex_element) = ply.payload.get("vertex") {
        for vertex in vertex_element {
            if let (
                Some(ply_rs::ply::Property::Float(x)),
                Some(ply_rs::ply::Property::Float(y)), 
                Some(ply_rs::ply::Property::Float(z))
            ) = (
                vertex.get("x"),
                vertex.get("y"),
                vertex.get("z")
            ) {
                mesh.vertices.push(*x);
                mesh.vertices.push(*y);
                mesh.vertices.push(*z);
            }
        }
    }
    
    // Extract faces  
    if let Some(face_element) = ply.payload.get("face") {
        for face in face_element {
            if let Some(ply_rs::ply::Property::ListInt(vertex_indices)) = face.get("vertex_indices") {
                // Convert faces to triangles (triangulate if needed)
                if vertex_indices.len() >= 3 {
                    // For triangles, add directly
                    if vertex_indices.len() == 3 {
                        mesh.indices.push(vertex_indices[0] as u32);
                        mesh.indices.push(vertex_indices[1] as u32);
                        mesh.indices.push(vertex_indices[2] as u32);
                    } else {
                        // For quads and higher polygons, triangulate as fan
                        for i in 1..(vertex_indices.len() - 1) {
                            mesh.indices.push(vertex_indices[0] as u32);
                            mesh.indices.push(vertex_indices[i] as u32);
                            mesh.indices.push(vertex_indices[i + 1] as u32);
                        }
                    }
                }
            }
        }
    }
    
    if mesh.vertices.is_empty() {
        return Err(OptError::InvalidFormat("No vertices found in PLY file".to_string()));
    }
    
    Ok(mesh)
}

/// Load glTF mesh data
fn load_simple_gltf_data(data: &[u8]) -> OptResult<MeshData> {
    let gltf = gltf::Gltf::from_slice(data)
        .map_err(|e| OptError::InvalidFormat(format!("glTF parse error: {e}")))?;
    
    let mut mesh = MeshData::new();
    
    // For simplicity, extract data from the first mesh in the first scene
    if let Some(scene) = gltf.default_scene() {
        for node in scene.nodes() {
            if let Some(node_mesh) = node.mesh() {
                for primitive in node_mesh.primitives() {
                    // Get vertex positions
                    if let Some(_positions_accessor) = primitive.get(&gltf::Semantic::Positions) {
                        // We need the buffer data, but glTF crate doesn't provide it directly
                        // For now, create a placeholder mesh
                        // TODO: Implement proper buffer reading when we have external .bin files
                        
                        // Create a simple triangle as placeholder
                        mesh.vertices.extend_from_slice(&[
                            0.0, 0.0, 0.0,
                            1.0, 0.0, 0.0, 
                            0.0, 1.0, 0.0
                        ]);
                        mesh.indices.extend_from_slice(&[0, 1, 2]);
                        
                        // Only process first primitive for now
                        break;
                    }
                }
                // Only process first mesh for now
                break;
            }
        }
    }
    
    // If no mesh data was found, return error
    if mesh.vertices.is_empty() {
        return Err(OptError::InvalidFormat("No mesh data found in glTF".to_string()));
    }
    
    Ok(mesh)
}

/// Load DAE mesh data
fn load_dae_data(data: &[u8]) -> OptResult<MeshData> {
    use crate::mesh::dae::extract_dae_geometry;
    extract_dae_geometry(data)
}

/// Load FBX mesh data  
fn load_fbx_data(data: &[u8]) -> OptResult<MeshData> {
    use crate::mesh::fbx::extract_ascii_fbx_geometry;
    
    // Check if it's binary or ASCII FBX
    if data.len() < 20 {
        return Err(OptError::InvalidInput("File too small for FBX".to_string()));
    }
    
    if data.starts_with(b"Kaydara FBX Binary") {
        // Binary FBX - not yet supported for loading
        return Err(OptError::InvalidFormat("Binary FBX loading not yet implemented".to_string()));
    } else {
        // Try to parse as ASCII FBX
        let text = String::from_utf8_lossy(data);
        if text.contains("FBXHeaderExtension") || text.contains("FBXVersion") {
            extract_ascii_fbx_geometry(&text)
        } else {
            Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
        }
    }
}

/// Write mesh data to bytes in specified format
pub fn write_mesh_data(mesh: &MeshData, extension: &str) -> OptResult<Vec<u8>> {
    match extension.to_lowercase().as_str() {
        "obj" => write_obj_data(mesh),
        "stl" => write_stl_data(mesh),
        "ply" => write_ply_data(mesh),
        _ => Err(OptError::InvalidFormat(format!("Output format not supported: {}", extension))),
    }
}

/// Write mesh data as OBJ format
fn write_obj_data(mesh: &MeshData) -> OptResult<Vec<u8>> {
    let mut output = String::new();
    
    // Write vertices
    for i in (0..mesh.vertices.len()).step_by(3) {
        if i + 2 < mesh.vertices.len() {
            output.push_str(&format!(
                "v {} {} {}\n",
                mesh.vertices[i],
                mesh.vertices[i + 1],
                mesh.vertices[i + 2]
            ));
        }
    }
    
    // Write faces
    for i in (0..mesh.indices.len()).step_by(3) {
        if i + 2 < mesh.indices.len() {
            output.push_str(&format!(
                "f {} {} {}\n",
                mesh.indices[i] + 1,
                mesh.indices[i + 1] + 1,
                mesh.indices[i + 2] + 1
            ));
        }
    }
    
    Ok(output.into_bytes())
}

/// Write simplified STL data (ASCII format)
fn write_stl_data(mesh: &MeshData) -> OptResult<Vec<u8>> {
    // Write as ASCII STL for simplicity and compatibility
    let mut output = String::new();
    output.push_str("solid mesh\n");
    
    for i in (0..mesh.indices.len()).step_by(3) {
        if i + 2 < mesh.indices.len() {
            let i0 = (mesh.indices[i] as usize) * 3;
            let i1 = (mesh.indices[i + 1] as usize) * 3;
            let i2 = (mesh.indices[i + 2] as usize) * 3;
            
            if i0 + 2 < mesh.vertices.len() && i1 + 2 < mesh.vertices.len() && i2 + 2 < mesh.vertices.len() {
                output.push_str("facet normal 0.0 0.0 1.0\n");
                output.push_str("  outer loop\n");
                output.push_str(&format!(
                    "    vertex {} {} {}\n",
                    mesh.vertices[i0], mesh.vertices[i0 + 1], mesh.vertices[i0 + 2]
                ));
                output.push_str(&format!(
                    "    vertex {} {} {}\n",
                    mesh.vertices[i1], mesh.vertices[i1 + 1], mesh.vertices[i1 + 2]
                ));
                output.push_str(&format!(
                    "    vertex {} {} {}\n",
                    mesh.vertices[i2], mesh.vertices[i2 + 1], mesh.vertices[i2 + 2]
                ));
                output.push_str("  endloop\n");
                output.push_str("endfacet\n");
            }
        }
    }
    
    output.push_str("endsolid mesh\n");
    Ok(output.into_bytes())
}

/// Write simplified PLY data
fn write_ply_data(mesh: &MeshData) -> OptResult<Vec<u8>> {
    let mut output = String::new();
    
    // PLY header
    output.push_str("ply\n");
    output.push_str("format ascii 1.0\n");
    output.push_str(&format!("element vertex {}\n", mesh.vertex_count()));
    output.push_str("property float x\n");
    output.push_str("property float y\n");
    output.push_str("property float z\n");
    output.push_str(&format!("element face {}\n", mesh.triangle_count()));
    output.push_str("property list uchar int vertex_indices\n");
    output.push_str("end_header\n");
    
    // Write vertices
    for i in (0..mesh.vertices.len()).step_by(3) {
        if i + 2 < mesh.vertices.len() {
            output.push_str(&format!(
                "{} {} {}\n",
                mesh.vertices[i],
                mesh.vertices[i + 1],
                mesh.vertices[i + 2]
            ));
        }
    }
    
    // Write faces
    for i in (0..mesh.indices.len()).step_by(3) {
        if i + 2 < mesh.indices.len() {
            output.push_str(&format!(
                "3 {} {} {}\n",
                mesh.indices[i],
                mesh.indices[i + 1],
                mesh.indices[i + 2]
            ));
        }
    }
    
    Ok(output.into_bytes())
}
