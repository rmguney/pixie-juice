use std::fs;

fn main() {
    let data = fs::read("tests/fixtures/models/test_cube.dae").expect("Failed to read DAE file");
    
    // Parse the DAE file and show debug info
    let text = String::from_utf8_lossy(&data);
    println!("DAE file contents preview:");
    println!("{}", &text[..text.len().min(500)]);
    
    // Try to extract geometry
    match rust_core::mesh::dae::extract_dae_geometry(&data) {
        Ok(mesh) => {
            println!("\nExtracted mesh:");
            println!("Vertices: {} floats", mesh.vertices.len());
            println!("Indices: {} indices", mesh.indices.len());
            println!("Vertex count: {}", mesh.vertices.len() / 3);
            println!("Triangle count: {}", mesh.indices.len() / 3);
            
            if !mesh.vertices.is_empty() {
                println!("First few vertices:");
                for chunk in mesh.vertices.chunks(3).take(3) {
                    if chunk.len() == 3 {
                        println!("  [{}, {}, {}]", chunk[0], chunk[1], chunk[2]);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to extract geometry: {:?}", e);
        }
    }
}
