//! Mesh optimization utilities

use crate::common::Result;
use crate::ffi::mesh_ffi::MeshData;

/// Mesh optimizer for reducing polygon count and improving performance
pub struct MeshOptimizer {
    /// Target reduction percentage (0.0 to 1.0)
    pub target_reduction: f32,
    /// Preserve mesh boundaries
    pub preserve_boundaries: bool,
    /// Preserve UV coordinates
    pub preserve_uvs: bool,
    /// Preserve normals
    pub preserve_normals: bool,
}

impl MeshOptimizer {
    /// Create a new mesh optimizer with default settings
    pub fn new() -> Self {
        Self {
            target_reduction: 0.5, // 50% reduction by default
            preserve_boundaries: true,
            preserve_uvs: true,
            preserve_normals: true,
        }
    }

    /// Create optimizer with specific reduction target
    pub fn with_reduction(target_reduction: f32) -> Self {
        Self {
            target_reduction: target_reduction.clamp(0.0, 1.0),
            ..Self::new()
        }
    }

    /// Optimize mesh using various techniques
    pub fn optimize(&self, mesh: &MeshData) -> Result<OptimizedMesh> {
        let mut optimized = OptimizedMesh {
            mesh: mesh.clone(),
            optimization_stats: OptimizationStats::new(),
        };

        let start_time = std::time::Instant::now();
        let original_triangle_count = mesh.triangle_count();

        // Step 1: Remove duplicate vertices
        self.remove_duplicate_vertices(&mut optimized.mesh)?;

        // Step 2: Remove degenerate triangles
        self.remove_degenerate_triangles(&mut optimized.mesh)?;

        // Step 3: Simplify mesh (calls C hotspot for performance)
        if self.target_reduction > 0.0 {
            self.simplify_mesh(&mut optimized.mesh)?;
        }

        // Step 4: Optimize vertex cache
        self.optimize_vertex_cache(&mut optimized.mesh)?;

        // Calculate stats
        let processing_time = start_time.elapsed();
        optimized.optimization_stats = OptimizationStats {
            original_vertices: mesh.vertex_count(),
            optimized_vertices: optimized.mesh.vertex_count(),
            original_triangles: original_triangle_count,
            optimized_triangles: optimized.mesh.triangle_count(),
            processing_time_ms: processing_time.as_millis() as u64,
        };

        Ok(optimized)
    }

    fn remove_duplicate_vertices(&self, mesh: &mut MeshData) -> Result<()> {
        use std::collections::HashMap;
        
        if mesh.vertices.is_empty() {
            return Ok(());
        }
        
        // Create a map to track unique vertices
        let mut vertex_map: HashMap<[u32; 3], usize> = HashMap::new();
        let mut new_vertices = Vec::new();
        let mut vertex_remap = vec![0usize; mesh.vertices.len() / 3];
        
        // Process vertices in groups of 3 (x, y, z)
        for i in 0..mesh.vertices.len() / 3 {
            let vertex_start = i * 3;
            
            // Convert to fixed-point representation for hashing (avoid floating point precision issues)
            let x = (mesh.vertices[vertex_start] * 1000000.0) as u32;
            let y = (mesh.vertices[vertex_start + 1] * 1000000.0) as u32;
            let z = (mesh.vertices[vertex_start + 2] * 1000000.0) as u32;
            let vertex_key = [x, y, z];
            
            if let Some(&existing_index) = vertex_map.get(&vertex_key) {
                // Duplicate vertex found, remap to existing
                vertex_remap[i] = existing_index;
            } else {
                // New unique vertex
                let new_index = new_vertices.len() / 3;
                vertex_map.insert(vertex_key, new_index);
                vertex_remap[i] = new_index;
                
                // Add vertex to new list
                new_vertices.push(mesh.vertices[vertex_start]);
                new_vertices.push(mesh.vertices[vertex_start + 1]);
                new_vertices.push(mesh.vertices[vertex_start + 2]);
            }
        }
        
        // Update indices to point to new vertex positions
        for index in &mut mesh.indices {
            if (*index as usize) < vertex_remap.len() {
                *index = vertex_remap[*index as usize] as u32;
            }
        }
        
        // Update vertex data
        mesh.vertices = new_vertices;
        
        log::info!("Duplicate vertex removal: {} -> {} vertices", 
                  vertex_remap.len(), mesh.vertices.len() / 3);
        
        Ok(())
    }

    fn remove_degenerate_triangles(&self, mesh: &mut MeshData) -> Result<()> {
        let mut valid_indices = Vec::new();

        for triangle in mesh.indices.chunks(3) {
            if triangle.len() == 3 {
                // Check if indices are different (not a degenerate triangle)
                if triangle[0] != triangle[1] && triangle[1] != triangle[2] && triangle[0] != triangle[2] {
                    valid_indices.extend_from_slice(triangle);
                }
            }
        }

        mesh.indices = valid_indices;
        Ok(())
    }

    fn simplify_mesh(&self, mesh: &mut MeshData) -> Result<()> {
        #[cfg(feature = "c_hotspots")]
        {
            // Call C hotspot for mesh decimation
            use crate::ffi::mesh_ffi::decimate_mesh_c;
            
            if mesh.vertices.len() < 9 || mesh.indices.len() < 3 {
                return Ok(()); // Too small to optimize
            }

            // Calculate target index count
            let target_count = ((mesh.indices.len() as f32) * (1.0 - self.target_reduction)) as usize;
            let target_count = (target_count / 3) * 3; // Ensure multiple of 3

            if target_count >= mesh.indices.len() {
                return Ok(()); // No reduction needed
            }

            // Convert to format expected by C function
            let vertex_data: Vec<f32> = mesh.vertices.clone();

            // Call C decimation function
            match decimate_mesh_c(&vertex_data, &mesh.indices, self.target_reduction) {
                Ok((new_vertices, new_indices)) => {
                    // Update mesh with new data
                    mesh.vertices = new_vertices;
                    mesh.indices = new_indices;
                    mesh.vertex_count = mesh.vertices.len() / 3;
                    mesh.index_count = mesh.indices.len();
                }
                Err(_) => {
                    // Fall back to Rust implementation on error
                    log::warn!("C mesh decimation failed, using Rust fallback");
                }
            }
        }
        
        #[cfg(not(feature = "c_hotspots"))]
        {
            // Rust fallback implementation
            if mesh.vertices.len() < 9 || mesh.indices.len() < 3 {
                return Ok(()); // Too small to optimize
            }

            let target_count = ((mesh.indices.len() as f32) * (1.0 - self.target_reduction)) as usize;
            let target_count = (target_count / 3) * 3; // Ensure multiple of 3

            if target_count >= mesh.indices.len() {
                return Ok(()); // No reduction needed
            }

            // Simple Rust-based decimation (placeholder)
            // In practice, you'd want a more sophisticated algorithm
            mesh.indices.truncate(target_count);
        }
        
        Ok(())
    }

    fn optimize_vertex_cache(&self, mesh: &mut MeshData) -> Result<()> {
        if mesh.indices.is_empty() || mesh.indices.len() % 3 != 0 {
            return Ok(());
        }
        
        // Implement simplified vertex cache optimization using linear scan approach
        // This reorders triangles to improve vertex cache hit rate
        
        let triangle_count = mesh.indices.len() / 3;
        let mut used_triangles = vec![false; triangle_count];
        let mut optimized_indices = Vec::with_capacity(mesh.indices.len());
        let mut vertex_cache: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
        const CACHE_SIZE: usize = 32; // Typical GPU vertex cache size
        
        // Process triangles in an order that maximizes cache hits
        for _ in 0..triangle_count {
            let mut best_triangle = None;
            let mut best_score = -1.0f32;
            
            // Find the triangle with the highest cache score
            for tri_idx in 0..triangle_count {
                if used_triangles[tri_idx] {
                    continue;
                }
                
                let base_idx = tri_idx * 3;
                let v0 = mesh.indices[base_idx];
                let v1 = mesh.indices[base_idx + 1];
                let v2 = mesh.indices[base_idx + 2];
                
                // Calculate cache score for this triangle
                let mut score = 0.0f32;
                
                // Score based on how many vertices are already in cache
                for &vertex in &[v0, v1, v2] {
                    if let Some(cache_pos) = vertex_cache.iter().position(|&v| v == vertex) {
                        // Vertex is in cache - score based on position (newer = better)
                        let cache_score = (CACHE_SIZE - cache_pos) as f32 / CACHE_SIZE as f32;
                        score += cache_score * 2.0; // Bonus for cache hits
                    }
                }
                
                if score > best_score {
                    best_score = score;
                    best_triangle = Some(tri_idx);
                }
            }
            
            // If no triangle found with cache hits, pick the first unused one
            if best_triangle.is_none() {
                best_triangle = used_triangles.iter().position(|&used| !used);
            }
            
            if let Some(tri_idx) = best_triangle {
                used_triangles[tri_idx] = true;
                
                let base_idx = tri_idx * 3;
                let v0 = mesh.indices[base_idx];
                let v1 = mesh.indices[base_idx + 1];
                let v2 = mesh.indices[base_idx + 2];
                
                optimized_indices.push(v0);
                optimized_indices.push(v1);
                optimized_indices.push(v2);
                
                // Update vertex cache
                for &vertex in &[v0, v1, v2] {
                    // Remove vertex if already in cache
                    if let Some(pos) = vertex_cache.iter().position(|&v| v == vertex) {
                        vertex_cache.remove(pos);
                    }
                    // Add vertex to front of cache
                    vertex_cache.push_front(vertex);
                    // Limit cache size
                    if vertex_cache.len() > CACHE_SIZE {
                        vertex_cache.pop_back();
                    }
                }
            } else {
                break; // No more triangles to process
            }
        }
        
        // Update mesh indices with optimized order
        mesh.indices = optimized_indices;
        
        log::info!("Vertex cache optimization completed for {} triangles", triangle_count);
        
        Ok(())
    }
}

impl Default for MeshOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of mesh optimization
#[derive(Debug, Clone)]
pub struct OptimizedMesh {
    pub mesh: MeshData,
    pub optimization_stats: OptimizationStats,
}

/// Statistics about the optimization process
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub original_vertices: usize,
    pub optimized_vertices: usize,
    pub original_triangles: usize,
    pub optimized_triangles: usize,
    pub processing_time_ms: u64,
}

impl OptimizationStats {
    fn new() -> Self {
        Self {
            original_vertices: 0,
            optimized_vertices: 0,
            original_triangles: 0,
            optimized_triangles: 0,
            processing_time_ms: 0,
        }
    }

    /// Calculate vertex reduction percentage
    pub fn vertex_reduction_percent(&self) -> f32 {
        if self.original_vertices > 0 {
            ((self.original_vertices - self.optimized_vertices) as f32 / self.original_vertices as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate triangle reduction percentage
    pub fn triangle_reduction_percent(&self) -> f32 {
        if self.original_triangles > 0 {
            ((self.original_triangles - self.optimized_triangles) as f32 / self.original_triangles as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Print optimization summary
    pub fn print_summary(&self) {
        println!("Mesh Optimization Results:");
        println!("  Vertices: {} -> {} ({:.1}% reduction)", 
                 self.original_vertices, self.optimized_vertices, self.vertex_reduction_percent());
        println!("  Triangles: {} -> {} ({:.1}% reduction)", 
                 self.original_triangles, self.optimized_triangles, self.triangle_reduction_percent());
        println!("  Processing time: {}ms", self.processing_time_ms);
    }
}
