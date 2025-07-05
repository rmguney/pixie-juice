//! DAE/Collada format support with real geometry extraction

use crate::types::{OptConfig, OptError, OptResult};
use crate::ffi::mesh_ffi::{MeshData, decimate_mesh_safe, weld_vertices_safe};
use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use std::io::Cursor;
use std::collections::HashMap;

/// Optimize DAE/Collada format with real geometry processing
pub fn optimize_dae(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Validate it's XML with COLLADA namespace
    let text = String::from_utf8_lossy(data);
    
    if !text.contains("<?xml") {
        return Err(OptError::InvalidFormat("Not a valid XML file".to_string()));
    }
    
    if !text.contains("COLLADA") {
        return Err(OptError::InvalidFormat("Not a valid COLLADA/DAE file".to_string()));
    }
    
    // Extract mesh data from the COLLADA XML
    let mesh_data = extract_dae_geometry(data)?;
    
    if mesh_data.vertices.is_empty() {
        return Err(OptError::InvalidFormat("No geometry found in DAE file".to_string()));
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
    
    // Rebuild the COLLADA XML with optimized geometry
    rebuild_dae_with_mesh(data, &optimized_mesh)
}

/// Extract geometry data from COLLADA XML
pub fn extract_dae_geometry(data: &[u8]) -> OptResult<MeshData> {
    let mut reader = Reader::from_reader(Cursor::new(data));
    reader.trim_text(true);
    
    let mut mesh = MeshData::new();
    let mut buf = Vec::new();
    let mut sources: HashMap<String, Vec<f32>> = HashMap::new();
    let mut vertices_sources: HashMap<String, String> = HashMap::new(); // vertices id -> source id mapping
    let mut triangle_inputs: HashMap<String, String> = HashMap::new(); // triangles semantic -> source mapping
    let mut in_float_array = false;
    let mut current_source_id = String::new();
    let mut in_vertices = false;
    let mut current_vertices_id = String::new();
    let mut in_triangles = false;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"source" => {
                        current_source_id = extract_attribute(e, "id").unwrap_or_default();
                    }
                    b"float_array" => {
                        in_float_array = true;
                    }
                    b"vertices" => {
                        in_vertices = true;
                        current_vertices_id = extract_attribute(e, "id").unwrap_or_default();
                    }
                    b"triangles" => {
                        in_triangles = true;
                    }
                    b"input" => {
                        if let (Some(semantic), Some(source)) = (
                            extract_attribute(e, "semantic"),
                            extract_attribute(e, "source")
                        ) {
                            let source_clean = source.trim_start_matches('#');
                            
                            if in_vertices && semantic == "POSITION" {
                                // Map vertices element to its position source
                                vertices_sources.insert(current_vertices_id.clone(), source_clean.to_string());
                            } else if in_triangles {
                                // Map triangle inputs
                                triangle_inputs.insert(semantic.clone(), source_clean.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing tags like <input .../> 
                match e.name().as_ref() {
                    b"input" => {
                        if let (Some(semantic), Some(source)) = (
                            extract_attribute(e, "semantic"),
                            extract_attribute(e, "source")
                        ) {
                            let source_clean = source.trim_start_matches('#');
                            
                            if in_vertices && semantic == "POSITION" {
                                // Map vertices element to its position source
                                vertices_sources.insert(current_vertices_id.clone(), source_clean.to_string());
                            } else if in_triangles {
                                // Map triangle inputs
                                triangle_inputs.insert(semantic.clone(), source_clean.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_float_array && !current_source_id.is_empty() {
                    let text = e.unescape().unwrap_or_default();
                    let floats: Vec<f32> = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    sources.insert(current_source_id.clone(), floats.clone());
                    println!("DAE: Added {} floats to source {}", floats.len(), current_source_id);
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"float_array" => {
                        in_float_array = false;
                        println!("DAE: Exiting float_array");
                    }
                    b"source" => {
                        current_source_id.clear();
                    }
                    b"vertices" => {
                        in_vertices = false;
                        current_vertices_id.clear();
                        println!("DAE: Exiting vertices");
                    }
                    b"triangles" => {
                        in_triangles = false;
                        println!("DAE: Exiting triangles");
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OptError::InvalidFormat(format!("XML parse error: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    
    
    // Extract vertex positions
    // Check if triangles reference a vertices element
    if let Some(vertices_id) = triangle_inputs.get("VERTEX") {
        if let Some(source_id) = vertices_sources.get(vertices_id) {
            if let Some(position_data) = sources.get(source_id) {
                mesh.vertices = position_data.clone();
            }
        }
    }
    
    // If no vertices found, try direct POSITION reference
    if mesh.vertices.is_empty() {
        if let Some(position_source_id) = triangle_inputs.get("POSITION") {
            if let Some(position_data) = sources.get(position_source_id) {
                mesh.vertices = position_data.clone();
            }
        }
    }
    
    // Extract triangles (simplified - assumes triangulated geometry)
    let mut reader = Reader::from_reader(Cursor::new(data));
    reader.trim_text(true);
    buf.clear();
    let mut in_triangles_p = false;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"p" {
                    in_triangles_p = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_triangles_p {
                    let text = e.unescape().unwrap_or_default();
                    let indices: Vec<u32> = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    
                    // For triangles, take every vertex index (assuming simple format)
                    for chunk in indices.chunks(3) {
                        if chunk.len() == 3 {
                            mesh.indices.extend_from_slice(chunk);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"p" {
                    in_triangles_p = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    
    Ok(mesh)
}

/// Rebuild COLLADA XML with optimized mesh data
fn rebuild_dae_with_mesh(original_data: &[u8], mesh: &MeshData) -> OptResult<Vec<u8>> {
    let mut reader = Reader::from_reader(Cursor::new(original_data));
    reader.trim_text(true);
    
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut skip_float_array = false;
    let mut skip_triangles = false;
    let mut in_geometry = false;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"library_geometries" => {
                        in_geometry = true;
                        writer.write_event(Event::Start(e.clone()))?;
                    }
                    b"float_array" if in_geometry => {
                        skip_float_array = true;
                        
                        // Write new float array with optimized vertices
                        let mut new_elem = BytesStart::new("float_array");
                        if let Some(id) = extract_attribute(e, "id") {
                            new_elem.push_attribute(("id", id.as_str()));
                        }
                        new_elem.push_attribute(("count", mesh.vertices.len().to_string().as_str()));
                        
                        writer.write_event(Event::Start(new_elem))?;
                        
                        // Write vertex data
                        let vertex_text = mesh.vertices
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(" ");
                        writer.write_event(Event::Text(BytesText::new(&vertex_text)))?;
                    }
                    b"triangles" if in_geometry => {
                        skip_triangles = true;
                        
                        // Write new triangles with optimized indices
                        let mut new_elem = BytesStart::new("triangles");
                        new_elem.push_attribute(("count", (mesh.indices.len() / 3).to_string().as_str()));
                        
                        writer.write_event(Event::Start(new_elem))?;
                        
                        // Write simplified input element
                        let mut input_elem = BytesStart::new("input");
                        input_elem.push_attribute(("semantic", "VERTEX"));
                        input_elem.push_attribute(("source", "#vertices"));
                        input_elem.push_attribute(("offset", "0"));
                        writer.write_event(Event::Start(input_elem))?;
                        writer.write_event(Event::End(BytesEnd::new("input")))?;
                        
                        // Write p element with new indices
                        writer.write_event(Event::Start(BytesStart::new("p")))?;
                        let index_text = mesh.indices
                            .iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<_>>()
                            .join(" ");
                        writer.write_event(Event::Text(BytesText::new(&index_text)))?;
                        writer.write_event(Event::End(BytesEnd::new("p")))?;
                    }
                    _ => {
                        if !skip_float_array && !skip_triangles {
                            writer.write_event(Event::Start(e.clone()))?;
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"library_geometries" => {
                        in_geometry = false;
                        writer.write_event(Event::End(e.clone()))?;
                    }
                    b"float_array" if skip_float_array => {
                        skip_float_array = false;
                        writer.write_event(Event::End(e.clone()))?;
                    }
                    b"triangles" if skip_triangles => {
                        skip_triangles = false;
                        writer.write_event(Event::End(e.clone()))?;
                    }
                    _ => {
                        if !skip_float_array && !skip_triangles {
                            writer.write_event(Event::End(e.clone()))?;
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if !skip_float_array && !skip_triangles {
                    writer.write_event(Event::Text(e))?;
                }
            }
            Ok(event) => {
                if !skip_float_array && !skip_triangles {
                    writer.write_event(event)?;
                }
            }
            Err(e) => return Err(OptError::InvalidFormat(format!("XML parse error: {}", e))),
        }
        buf.clear();
    }
    
    let result = writer.into_inner().into_inner();
    Ok(result)
}

/// Extract attribute value from XML element
fn extract_attribute(elem: &BytesStart, attr_name: &str) -> Option<String> {
    elem.attributes()
        .filter_map(|a| a.ok())
        .find(|attr| attr.key.as_ref() == attr_name.as_bytes())
        .and_then(|attr| String::from_utf8(attr.value.to_vec()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dae_optimization() {
        let config = OptConfig::default();
        
        let dae_data = "<?xml version=\"1.0\"?>
<COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\">
  <library_geometries>
    <geometry id=\"simple-triangle\">
      <mesh>
        <source id=\"positions\">
          <float_array id=\"positions-array\" count=\"9\">
            0.0 0.0 0.0 1.0 0.0 0.0 0.0 1.0 0.0
          </float_array>
        </source>
        <vertices id=\"vertices\">
          <input semantic=\"POSITION\" source=\"#positions\"/>
        </vertices>
        <triangles count=\"1\">
          <input semantic=\"VERTEX\" source=\"#vertices\" offset=\"0\"/>
          <p>0 1 2</p>
        </triangles>
      </mesh>
    </geometry>
  </library_geometries>
</COLLADA>";
        
        // Debug: first test geometry extraction
        let mesh_result = extract_dae_geometry(dae_data.as_bytes());
        match &mesh_result {
            Ok(mesh) => {
                println!("Extracted mesh: {} vertices, {} indices", mesh.vertices.len(), mesh.indices.len());
            }
            Err(e) => {
                println!("Failed to extract mesh: {:?}", e);
            }
        }
        
        let result = optimize_dae(dae_data.as_bytes(), &config);
        if let Err(ref e) = result {
            println!("Optimization failed: {:?}", e);
        }
        assert!(result.is_ok());
        
        // The result should still be valid XML
        let output = result.unwrap();
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("<?xml"));
        assert!(output_str.contains("COLLADA"));
    }
    
    #[test]
    fn test_dae_invalid_format() {
        let config = OptConfig::default();
        
        // Test with non-XML data
        let result = optimize_dae(b"not xml", &config);
        assert!(result.is_err());
        
        // Test with XML but not COLLADA
        let xml_data = b"<?xml version=\"1.0\"?><root></root>";
        let result = optimize_dae(xml_data, &config);
        assert!(result.is_err());
    }
}
