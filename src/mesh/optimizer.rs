extern crate alloc;
use alloc::{vec, vec::Vec, string::ToString};
use crate::types::{PixieResult, PixieError, MeshOptConfig};
use crate::optimizers::{get_current_time_ms, update_performance_stats};

pub struct MeshOptimizerCore;

impl MeshOptimizerCore {
    pub fn decimate_mesh_qem(
        vertices: &[f32], 
        indices: &[u32], 
        target_ratio: f32,
        config: &MeshOptConfig
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        #[cfg(c_hotspots_available)]
        {
            if data_size > 200_000 {
            }
        }
        
        let result = match config.simplification_algorithm {
            crate::types::SimplificationAlgorithm::QuadricErrorMetrics => {
                decimate_qem_rust(vertices, indices, target_ratio, config)
            },
            crate::types::SimplificationAlgorithm::EdgeCollapse => {
                decimate_edge_collapse_rust(vertices, indices, target_ratio, config)
            },
            crate::types::SimplificationAlgorithm::VertexClustering => {
                decimate_vertex_clustering_rust(vertices, indices, target_ratio, config)
            },
        };
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        match result {
            Ok(decimated) => Ok(decimated),
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn weld_vertices(
        vertices: &[f32], 
        indices: &[u32], 
        tolerance: f32
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        let result = weld_vertices_spatial_hash(vertices, indices, tolerance);
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        result
    }

    pub fn optimize_vertex_cache(
        vertices: &[f32], 
        indices: &[u32]
    ) -> PixieResult<Vec<u32>> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        let vertex_count = vertices.len() / 3;
        let result = optimize_vertex_cache_forsyth(vertex_count, indices, false);
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        result
    }
}

#[cfg(c_hotspots_available)]
fn apply_c_mesh_decimation(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    use crate::c_hotspots::decimate_mesh_qem;
    
    unsafe {
        let result = decimate_mesh_qem(
            vertices.as_ptr(),
            vertices.len(),
            indices.as_ptr(),
            indices.len(),
            target_ratio
        );
        
        if result.success != 0 {
            let new_vertices = Vec::from_raw_parts(
                result.vertices,
                result.vertex_count,
                result.vertex_count
            );
            let new_indices = Vec::from_raw_parts(
                result.indices,
                result.index_count,
                result.index_count
            );
            Ok((new_vertices, new_indices))
        } else {
            Err(PixieError::CHotspotError("Mesh decimation failed".to_string()))
        }
    }
}

fn decimate_qem_rust(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    config: &MeshOptConfig
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    if vertices.len() % 3 != 0 {
        return Err(PixieError::MeshOptimizationFailed(
            "Invalid vertex data: must be multiples of 3".to_string()
        ));
    }
    
    if indices.len() % 3 != 0 {
        return Err(PixieError::MeshOptimizationFailed(
            "Invalid index data: must be multiples of 3".to_string()
        ));
    }
    
    let target_triangle_count = ((indices.len() / 3) as f32 * target_ratio) as usize;
    
    if config.preserve_topology {
        let _decimation_step = if target_ratio > 0.5 { 2 } else { 3 };
        let mut new_indices = Vec::new();
        
        for chunk in indices.chunks(3) {
            if new_indices.len() / 3 < target_triangle_count {
                new_indices.extend_from_slice(chunk);
            }
        }
        
        Ok((vertices.to_vec(), new_indices))
    } else {
        let mut new_indices = Vec::new();
        let step = indices.len() / 3 / target_triangle_count.max(1);
        
        for i in (0..indices.len()).step_by(step * 3) {
            if i + 2 < indices.len() {
                new_indices.push(indices[i]);
                new_indices.push(indices[i + 1]);
                new_indices.push(indices[i + 2]);
            }
        }
        
        Ok((vertices.to_vec(), new_indices))
    }
}

fn decimate_edge_collapse_rust(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    _config: &MeshOptConfig
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    let target_count = ((indices.len() / 3) as f32 * target_ratio) as usize * 3;
    let mut new_indices = indices.to_vec();
    new_indices.truncate(target_count);
    
    Ok((vertices.to_vec(), new_indices))
}

fn decimate_vertex_clustering_rust(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    _config: &MeshOptConfig
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    let target_count = ((indices.len() / 3) as f32 * target_ratio) as usize * 3;
    let mut new_indices = indices.to_vec();
    new_indices.truncate(target_count);
    
    Ok((vertices.to_vec(), new_indices))
}

fn weld_vertices_spatial_hash(
    vertices: &[f32], 
    indices: &[u32], 
    tolerance: f32
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    use alloc::collections::BTreeMap;
    
    let mut vertex_map = BTreeMap::new();
    let mut new_vertices = Vec::new();
    let mut new_indices = Vec::new();
    
    let inv_tolerance = 1.0 / tolerance;
    
    for i in 0..vertices.len() / 3 {
        let x = vertices[i * 3];
        let y = vertices[i * 3 + 1];
        let z = vertices[i * 3 + 2];
        
        let hash_x = (x * inv_tolerance) as i32;
        let hash_y = (y * inv_tolerance) as i32;
        let hash_z = (z * inv_tolerance) as i32;
        let hash_key = (hash_x, hash_y, hash_z);
        
        if let Some(&existing_index) = vertex_map.get(&hash_key) {
            for &index in indices.iter() {
                if index == i as u32 {
                    new_indices.push(existing_index);
                } else {
                    new_indices.push(index);
                }
            }
        } else {
            let new_index = new_vertices.len() as u32 / 3;
            vertex_map.insert(hash_key, new_index);
            new_vertices.push(x);
            new_vertices.push(y);
            new_vertices.push(z);
        }
    }
    
    Ok((new_vertices, new_indices))
}

#[derive(Clone, Copy)]
struct ForsythVertex {
    cache_pos: i32,
    active_tris: u32,
    score: f32,
}

fn forsyth_vertex_score(cache_pos: i32, active_tris: u32, cache_size: u32) -> f32 {
    if active_tris == 0 {
        return -1.0;
    }

    const CACHE_DECAY_POWER: f32 = 1.5;
    const LAST_TRI_SCORE: f32 = 0.75;
    const VALENCE_BOOST_SCALE: f32 = 2.0;
    const VALENCE_BOOST_POWER: f32 = 0.5;

    let mut score = if cache_pos < 0 {
        0.0
    } else if cache_pos < 3 {
        LAST_TRI_SCORE
    } else {
        let cache_size = cache_size.max(4);
        let scaler = 1.0 / (cache_size as f32 - 3.0);
        let v = (1.0 - ((cache_pos as f32 - 3.0) * scaler)).max(0.0);
        v.powf(CACHE_DECAY_POWER)
    };

    score += VALENCE_BOOST_SCALE * (active_tris as f32).powf(-VALENCE_BOOST_POWER);
    score
}

pub fn optimize_vertex_cache_forsyth(
    vertex_count: usize,
    indices: &[u32],
    use_c_hotspots: bool,
) -> PixieResult<Vec<u32>> {
    if indices.len() % 3 != 0 {
        return Err(PixieError::InvalidInput(
            "Indices must be triangle list".to_string(),
        ));
    }

    if vertex_count == 0 {
        return Ok(indices.to_vec());
    }

    if use_c_hotspots {
        #[cfg(c_hotspots_available)]
        {
            if let Ok(out) = crate::c_hotspots::vertex_cache::optimize_indices_forsyth(
                vertex_count,
                indices,
                true,
            ) {
                return Ok(out);
            }
        }
    }

    optimize_vertex_cache_forsyth_rust(vertex_count, indices, 32)
}

fn optimize_vertex_cache_forsyth_rust(
    vertex_count: usize,
    indices: &[u32],
    cache_size: u32,
) -> PixieResult<Vec<u32>> {
    let tri_count = indices.len() / 3;
    if tri_count == 0 {
        return Ok(Vec::new());
    }

    let cache_size = cache_size.clamp(4, 64);

    let mut vertices = vec![ForsythVertex { cache_pos: -1, active_tris: 0, score: 0.0 }; vertex_count];
    let mut tri_emitted = vec![false; tri_count];
    let mut tri_verts = vec![[0u32; 3]; tri_count];

    for (t, tri) in indices.chunks_exact(3).enumerate() {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a >= vertex_count || b >= vertex_count || c >= vertex_count {
            return Err(PixieError::InvalidInput("Index out of range".to_string()));
        }
        tri_verts[t] = [tri[0], tri[1], tri[2]];
        vertices[a].active_tris += 1;
        vertices[b].active_tris += 1;
        vertices[c].active_tris += 1;
    }

    let mut offsets = vec![0u32; vertex_count + 1];
    let mut sum = 0u32;
    for v in 0..vertex_count {
        offsets[v] = sum;
        sum += vertices[v].active_tris;
    }
    offsets[vertex_count] = sum;

    let mut adjacency = vec![0u32; sum as usize];
    let mut cursor = offsets[..vertex_count].to_vec();
    for (t, tri) in tri_verts.iter().enumerate() {
        for &v in tri {
            let vi = v as usize;
            let pos = cursor[vi] as usize;
            adjacency[pos] = t as u32;
            cursor[vi] += 1;
        }
    }

    for v in 0..vertex_count {
        vertices[v].score = forsyth_vertex_score(-1, vertices[v].active_tris, cache_size);
    }

    let mut tri_scores = vec![0.0f32; tri_count];
    for t in 0..tri_count {
        let [a, b, c] = tri_verts[t];
        tri_scores[t] = vertices[a as usize].score + vertices[b as usize].score + vertices[c as usize].score;
    }

    let mut heap = (0..tri_count as u32).collect::<Vec<u32>>();
    let mut heap_pos = vec![0u32; tri_count];
    for (i, &t) in heap.iter().enumerate() {
        heap_pos[t as usize] = i as u32;
    }

    let mut heap_swap = |heap: &mut [u32], heap_pos: &mut [u32], a: usize, b: usize| {
        let ta = heap[a];
        let tb = heap[b];
        heap[a] = tb;
        heap[b] = ta;
        heap_pos[ta as usize] = b as u32;
        heap_pos[tb as usize] = a as u32;
    };

    let mut sift_up = |heap: &mut [u32], heap_pos: &mut [u32], tri_scores: &[f32], mut idx: usize| {
        while idx > 0 {
            let parent = (idx - 1) >> 1;
            let t = heap[idx] as usize;
            let p = heap[parent] as usize;
            if tri_scores[t] <= tri_scores[p] {
                break;
            }
            heap_swap(heap, heap_pos, idx, parent);
            idx = parent;
        }
    };

    let mut sift_down = |heap: &mut [u32], heap_pos: &mut [u32], tri_scores: &[f32], mut idx: usize, size: usize| {
        loop {
            let left = idx * 2 + 1;
            if left >= size {
                break;
            }
            let right = left + 1;
            let mut best = left;
            if right < size {
                let tl = heap[left] as usize;
                let tr = heap[right] as usize;
                if tri_scores[tr] > tri_scores[tl] {
                    best = right;
                }
            }
            let ti = heap[idx] as usize;
            let tb = heap[best] as usize;
            if tri_scores[ti] >= tri_scores[tb] {
                break;
            }
            heap_swap(heap, heap_pos, idx, best);
            idx = best;
        }
    };

    let mut heap_size = heap.len();
    for i in (0..=(heap_size / 2)).rev() {
        sift_down(&mut heap, &mut heap_pos, &tri_scores, i, heap_size);
        if i == 0 {
            break;
        }
    }

    let mut cache = vec![u32::MAX; cache_size as usize];
    let mut touched = vec![false; vertex_count];
    let mut out = Vec::with_capacity(indices.len());

    for _ in 0..tri_count {
        if heap_size == 0 {
            break;
        }

        let tri = heap[0] as usize;
        heap_size -= 1;
        if heap_size > 0 {
            heap[0] = heap[heap_size];
            heap_pos[heap[0] as usize] = 0;
            sift_down(&mut heap, &mut heap_pos, &tri_scores, 0, heap_size);
        }
        heap_pos[tri] = u32::MAX;

        if tri_emitted[tri] {
            continue;
        }
        tri_emitted[tri] = true;

        let [a, b, c] = tri_verts[tri];
        out.extend_from_slice(&[a, b, c]);

        for &v in &[a, b, c] {
            let vi = v as usize;
            if vertices[vi].active_tris > 0 {
                vertices[vi].active_tris -= 1;
            }
        }

        for &v in &[a, b, c] {
            let mut found = None;
            for (i, &cv) in cache.iter().enumerate() {
                if cv == v {
                    found = Some(i);
                    break;
                }
            }
            if let Some(pos) = found {
                for i in (1..=pos).rev() {
                    cache[i] = cache[i - 1];
                }
                cache[0] = v;
            } else {
                for i in (1..cache.len()).rev() {
                    cache[i] = cache[i - 1];
                }
                cache[0] = v;
            }
        }

        for v in 0..vertex_count {
            vertices[v].cache_pos = -1;
        }
        for (i, &v) in cache.iter().enumerate() {
            if v == u32::MAX {
                continue;
            }
            vertices[v as usize].cache_pos = i as i32;
            touched[v as usize] = true;
        }
        for &v in &[a, b, c] {
            touched[v as usize] = true;
        }

        for v in 0..vertex_count {
            if !touched[v] {
                continue;
            }
            vertices[v].score = forsyth_vertex_score(vertices[v].cache_pos, vertices[v].active_tris, cache_size);

            let start = offsets[v] as usize;
            let end = offsets[v + 1] as usize;
            for &t in &adjacency[start..end] {
                let ti = t as usize;
                if tri_emitted[ti] {
                    continue;
                }
                let [va, vb, vc] = tri_verts[ti];
                tri_scores[ti] = vertices[va as usize].score + vertices[vb as usize].score + vertices[vc as usize].score;
                let hp = heap_pos[ti];
                if hp != u32::MAX {
                    let hp = hp as usize;
                    sift_up(&mut heap, &mut heap_pos, &tri_scores, hp);
                    sift_down(&mut heap, &mut heap_pos, &tri_scores, hp, heap_size);
                }
            }

            touched[v] = false;
        }
    }

    Ok(out)
}
