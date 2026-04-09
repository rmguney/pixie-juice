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
        config: &MeshOptConfig,
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;

        if vertices.len() % 3 != 0 {
            return Err(PixieError::MeshOptimizationFailed("vertex array length must be a multiple of 3".to_string()));
        }
        if indices.len() % 3 != 0 {
            return Err(PixieError::MeshOptimizationFailed("index array length must be a multiple of 3".to_string()));
        }
        let target_ratio = target_ratio.clamp(0.01, 1.0);

        let result = match config.simplification_algorithm {
            crate::types::SimplificationAlgorithm::QuadricErrorMetrics => {
                decimate_qem(vertices, indices, target_ratio)
            }
            crate::types::SimplificationAlgorithm::EdgeCollapse => {
                decimate_edge_collapse(vertices, indices, target_ratio)
            }
            crate::types::SimplificationAlgorithm::VertexClustering => {
                decimate_vertex_clustering(vertices, indices, target_ratio)
            }
        };

        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        result
    }

    pub fn weld_vertices(
        vertices: &[f32],
        indices: &[u32],
        tolerance: f32,
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
        indices: &[u32],
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

fn weld_vertices_spatial_hash(
    vertices: &[f32],
    indices: &[u32],
    tolerance: f32,
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    use alloc::collections::BTreeMap;

    if vertices.len() % 3 != 0 {
        return Err(PixieError::MeshOptimizationFailed("vertex array length must be a multiple of 3".to_string()));
    }
    let tolerance = if tolerance > 0.0 { tolerance } else { 1e-6 };
    let inv_tolerance = 1.0 / tolerance;
    let original_vertex_count = vertices.len() / 3;

    let mut bucket: BTreeMap<(i32, i32, i32), u32> = BTreeMap::new();
    let mut new_vertices: Vec<f32> = Vec::with_capacity(vertices.len());
    let mut old_to_new: Vec<u32> = Vec::with_capacity(original_vertex_count);

    for i in 0..original_vertex_count {
        let x = vertices[i * 3];
        let y = vertices[i * 3 + 1];
        let z = vertices[i * 3 + 2];
        let key = (
            (x * inv_tolerance).round() as i32,
            (y * inv_tolerance).round() as i32,
            (z * inv_tolerance).round() as i32,
        );
        if let Some(&existing) = bucket.get(&key) {
            old_to_new.push(existing);
        } else {
            let new_idx = (new_vertices.len() / 3) as u32;
            bucket.insert(key, new_idx);
            new_vertices.push(x);
            new_vertices.push(y);
            new_vertices.push(z);
            old_to_new.push(new_idx);
        }
    }

    let mut new_indices: Vec<u32> = Vec::with_capacity(indices.len());
    for tri in indices.chunks_exact(3) {
        let a = *old_to_new.get(tri[0] as usize).ok_or_else(|| {
            PixieError::MeshOptimizationFailed("index out of range during weld".to_string())
        })?;
        let b = *old_to_new.get(tri[1] as usize).ok_or_else(|| {
            PixieError::MeshOptimizationFailed("index out of range during weld".to_string())
        })?;
        let c = *old_to_new.get(tri[2] as usize).ok_or_else(|| {
            PixieError::MeshOptimizationFailed("index out of range during weld".to_string())
        })?;
        if a != b && b != c && a != c {
            new_indices.push(a);
            new_indices.push(b);
            new_indices.push(c);
        }
    }

    Ok((new_vertices, new_indices))
}

fn bounding_box(vertices: &[f32]) -> Option<([f32; 3], [f32; 3])> {
    if vertices.len() < 3 {
        return None;
    }
    let mut min = [vertices[0], vertices[1], vertices[2]];
    let mut max = min;
    for chunk in vertices.chunks_exact(3) {
        for d in 0..3 {
            if chunk[d] < min[d] { min[d] = chunk[d]; }
            if chunk[d] > max[d] { max[d] = chunk[d]; }
        }
    }
    Some((min, max))
}

fn decimate_vertex_clustering(
    vertices: &[f32],
    indices: &[u32],
    target_ratio: f32,
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    use alloc::collections::BTreeMap;

    let (min, max) = bounding_box(vertices)
        .ok_or_else(|| PixieError::MeshOptimizationFailed("empty mesh".to_string()))?;

    let original_vertex_count = vertices.len() / 3;
    if original_vertex_count == 0 {
        return Ok((Vec::new(), Vec::new()));
    }
    let target_vertex_count = ((original_vertex_count as f32 * target_ratio).ceil() as usize).max(4);

    let grid_resolution = grid_resolution_for_target(target_vertex_count);
    let extents = [
        (max[0] - min[0]).max(1e-6),
        (max[1] - min[1]).max(1e-6),
        (max[2] - min[2]).max(1e-6),
    ];
    let inv_step = [
        grid_resolution as f32 / extents[0],
        grid_resolution as f32 / extents[1],
        grid_resolution as f32 / extents[2],
    ];

    let mut cluster_accum: BTreeMap<(u32, u32, u32), ([f32; 3], u32, u32)> = BTreeMap::new();
    let mut cluster_index: BTreeMap<(u32, u32, u32), u32> = BTreeMap::new();
    let mut old_to_cluster_key: Vec<(u32, u32, u32)> = Vec::with_capacity(original_vertex_count);

    for i in 0..original_vertex_count {
        let p = [vertices[i * 3], vertices[i * 3 + 1], vertices[i * 3 + 2]];
        let key = (
            (((p[0] - min[0]) * inv_step[0]) as u32).min(grid_resolution.saturating_sub(1)),
            (((p[1] - min[1]) * inv_step[1]) as u32).min(grid_resolution.saturating_sub(1)),
            (((p[2] - min[2]) * inv_step[2]) as u32).min(grid_resolution.saturating_sub(1)),
        );
        let entry = cluster_accum.entry(key).or_insert(([0.0; 3], 0, 0));
        entry.0[0] += p[0];
        entry.0[1] += p[1];
        entry.0[2] += p[2];
        entry.1 += 1;
        old_to_cluster_key.push(key);
    }

    let mut new_vertices: Vec<f32> = Vec::with_capacity(cluster_accum.len() * 3);
    for (key, entry) in cluster_accum.iter_mut() {
        let count = entry.1.max(1) as f32;
        let idx = (new_vertices.len() / 3) as u32;
        new_vertices.push(entry.0[0] / count);
        new_vertices.push(entry.0[1] / count);
        new_vertices.push(entry.0[2] / count);
        entry.2 = idx;
        cluster_index.insert(*key, idx);
    }

    let mut new_indices: Vec<u32> = Vec::with_capacity(indices.len());
    for tri in indices.chunks_exact(3) {
        let ka = *old_to_cluster_key.get(tri[0] as usize)
            .ok_or_else(|| PixieError::MeshOptimizationFailed("index out of range during clustering".to_string()))?;
        let kb = *old_to_cluster_key.get(tri[1] as usize)
            .ok_or_else(|| PixieError::MeshOptimizationFailed("index out of range during clustering".to_string()))?;
        let kc = *old_to_cluster_key.get(tri[2] as usize)
            .ok_or_else(|| PixieError::MeshOptimizationFailed("index out of range during clustering".to_string()))?;
        let a = *cluster_index.get(&ka).unwrap();
        let b = *cluster_index.get(&kb).unwrap();
        let c = *cluster_index.get(&kc).unwrap();
        if a != b && b != c && a != c {
            new_indices.push(a);
            new_indices.push(b);
            new_indices.push(c);
        }
    }

    Ok((new_vertices, new_indices))
}

fn grid_resolution_for_target(target_vertex_count: usize) -> u32 {
    let cube_root = (target_vertex_count as f64).cbrt();
    let r = cube_root.ceil() as u32;
    r.max(2)
}

fn decimate_edge_collapse(
    vertices: &[f32],
    indices: &[u32],
    target_ratio: f32,
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    let original_triangle_count = indices.len() / 3;
    if original_triangle_count == 0 {
        return Ok((Vec::new(), Vec::new()));
    }
    let target_triangle_count = ((original_triangle_count as f32 * target_ratio).ceil() as usize).max(1);

    let original_vertex_count = vertices.len() / 3;
    if original_vertex_count == 0 {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut working_vertices: Vec<[f32; 3]> = Vec::with_capacity(original_vertex_count);
    for chunk in vertices.chunks_exact(3) {
        working_vertices.push([chunk[0], chunk[1], chunk[2]]);
    }

    let mut working_triangles: Vec<[u32; 3]> = Vec::with_capacity(original_triangle_count);
    for tri in indices.chunks_exact(3) {
        working_triangles.push([tri[0], tri[1], tri[2]]);
    }

    let mut edge_pairs: Vec<(f32, u32, u32)> = Vec::new();
    for tri in &working_triangles {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for &(a, b) in &edges {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            if lo == hi { continue; }
            let pa = working_vertices[lo as usize];
            let pb = working_vertices[hi as usize];
            let dx = pa[0] - pb[0];
            let dy = pa[1] - pb[1];
            let dz = pa[2] - pb[2];
            let len_sq = dx * dx + dy * dy + dz * dz;
            edge_pairs.push((len_sq, lo, hi));
        }
    }

    edge_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    edge_pairs.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2);

    let mut remap: Vec<u32> = (0..working_vertices.len() as u32).collect();
    fn root(remap: &mut [u32], mut x: u32) -> u32 {
        while remap[x as usize] != x {
            let parent = remap[x as usize];
            remap[x as usize] = remap[parent as usize];
            x = remap[x as usize];
        }
        x
    }

    let mut remaining_triangles = original_triangle_count;
    let mut edge_idx = 0;
    while remaining_triangles > target_triangle_count && edge_idx < edge_pairs.len() {
        let (_, lo, hi) = edge_pairs[edge_idx];
        edge_idx += 1;
        let ra = root(&mut remap, lo);
        let rb = root(&mut remap, hi);
        if ra == rb {
            continue;
        }
        let pa = working_vertices[ra as usize];
        let pb = working_vertices[rb as usize];
        working_vertices[ra as usize] = [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ];
        remap[rb as usize] = ra;

        let mut degenerate = 0usize;
        for tri in &working_triangles {
            let a = root(&mut remap, tri[0]);
            let b = root(&mut remap, tri[1]);
            let c = root(&mut remap, tri[2]);
            if a == b || b == c || a == c {
                degenerate += 1;
            }
        }
        remaining_triangles = original_triangle_count - degenerate;
    }

    finalize_after_remap(&working_vertices, &working_triangles, &mut remap)
}

fn finalize_after_remap(
    working_vertices: &[[f32; 3]],
    working_triangles: &[[u32; 3]],
    remap: &mut [u32],
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    fn root(remap: &mut [u32], mut x: u32) -> u32 {
        while remap[x as usize] != x {
            let parent = remap[x as usize];
            remap[x as usize] = remap[parent as usize];
            x = remap[x as usize];
        }
        x
    }

    let mut compact_index: Vec<i32> = vec![-1; working_vertices.len()];
    let mut new_vertices: Vec<f32> = Vec::new();
    for tri in working_triangles {
        for &orig in tri {
            let r = root(remap, orig) as usize;
            if compact_index[r] < 0 {
                compact_index[r] = (new_vertices.len() / 3) as i32;
                new_vertices.extend_from_slice(&working_vertices[r]);
            }
        }
    }

    let mut new_indices: Vec<u32> = Vec::new();
    for tri in working_triangles {
        let a = compact_index[root(remap, tri[0]) as usize];
        let b = compact_index[root(remap, tri[1]) as usize];
        let c = compact_index[root(remap, tri[2]) as usize];
        if a < 0 || b < 0 || c < 0 {
            continue;
        }
        let (au, bu, cu) = (a as u32, b as u32, c as u32);
        if au != bu && bu != cu && au != cu {
            new_indices.push(au);
            new_indices.push(bu);
            new_indices.push(cu);
        }
    }

    Ok((new_vertices, new_indices))
}

#[derive(Clone, Copy, Default)]
struct Quadric {
    m: [f64; 10],
}

impl Quadric {
    fn from_plane(a: f64, b: f64, c: f64, d: f64) -> Self {
        Self {
            m: [
                a * a, a * b, a * c, a * d,
                b * b, b * c, b * d,
                c * c, c * d,
                d * d,
            ],
        }
    }

    fn add(&self, other: &Self) -> Self {
        let mut out = Self::default();
        for i in 0..10 {
            out.m[i] = self.m[i] + other.m[i];
        }
        out
    }

    fn eval(&self, x: f64, y: f64, z: f64) -> f64 {
        let m = &self.m;
        m[0] * x * x + 2.0 * m[1] * x * y + 2.0 * m[2] * x * z + 2.0 * m[3] * x
            + m[4] * y * y + 2.0 * m[5] * y * z + 2.0 * m[6] * y
            + m[7] * z * z + 2.0 * m[8] * z
            + m[9]
    }
}

fn plane_for_triangle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> Option<(f64, f64, f64, f64)> {
    let ex = (p1[0] - p0[0]) as f64;
    let ey = (p1[1] - p0[1]) as f64;
    let ez = (p1[2] - p0[2]) as f64;
    let fx = (p2[0] - p0[0]) as f64;
    let fy = (p2[1] - p0[1]) as f64;
    let fz = (p2[2] - p0[2]) as f64;
    let nx = ey * fz - ez * fy;
    let ny = ez * fx - ex * fz;
    let nz = ex * fy - ey * fx;
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-12 {
        return None;
    }
    let inv = 1.0 / len;
    let a = nx * inv;
    let b = ny * inv;
    let c = nz * inv;
    let d = -(a * p0[0] as f64 + b * p0[1] as f64 + c * p0[2] as f64);
    Some((a, b, c, d))
}

fn decimate_qem(
    vertices: &[f32],
    indices: &[u32],
    target_ratio: f32,
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    let original_triangle_count = indices.len() / 3;
    let original_vertex_count = vertices.len() / 3;
    if original_triangle_count == 0 || original_vertex_count == 0 {
        return Ok((Vec::new(), Vec::new()));
    }
    let target_triangle_count = ((original_triangle_count as f32 * target_ratio).ceil() as usize).max(1);

    let mut working_vertices: Vec<[f32; 3]> = Vec::with_capacity(original_vertex_count);
    for chunk in vertices.chunks_exact(3) {
        working_vertices.push([chunk[0], chunk[1], chunk[2]]);
    }
    let mut working_triangles: Vec<[u32; 3]> = Vec::with_capacity(original_triangle_count);
    for tri in indices.chunks_exact(3) {
        working_triangles.push([tri[0], tri[1], tri[2]]);
    }

    let mut quadrics: Vec<Quadric> = vec![Quadric::default(); original_vertex_count];
    for tri in &working_triangles {
        let p0 = working_vertices[tri[0] as usize];
        let p1 = working_vertices[tri[1] as usize];
        let p2 = working_vertices[tri[2] as usize];
        if let Some((a, b, c, d)) = plane_for_triangle(p0, p1, p2) {
            let q = Quadric::from_plane(a, b, c, d);
            quadrics[tri[0] as usize] = quadrics[tri[0] as usize].add(&q);
            quadrics[tri[1] as usize] = quadrics[tri[1] as usize].add(&q);
            quadrics[tri[2] as usize] = quadrics[tri[2] as usize].add(&q);
        }
    }

    let mut edge_costs: Vec<(f64, u32, u32, [f32; 3])> = Vec::new();
    for tri in &working_triangles {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for &(a, b) in &edges {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            if lo == hi { continue; }
            let combined = quadrics[lo as usize].add(&quadrics[hi as usize]);
            let pa = working_vertices[lo as usize];
            let pb = working_vertices[hi as usize];
            let midpoint = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let cost = combined.eval(midpoint[0] as f64, midpoint[1] as f64, midpoint[2] as f64).max(0.0);
            edge_costs.push((cost, lo, hi, midpoint));
        }
    }

    edge_costs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    edge_costs.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2);

    let mut remap: Vec<u32> = (0..working_vertices.len() as u32).collect();
    fn root(remap: &mut [u32], mut x: u32) -> u32 {
        while remap[x as usize] != x {
            let parent = remap[x as usize];
            remap[x as usize] = remap[parent as usize];
            x = remap[x as usize];
        }
        x
    }

    let mut remaining_triangles = original_triangle_count;
    let mut edge_idx = 0;
    while remaining_triangles > target_triangle_count && edge_idx < edge_costs.len() {
        let (_, lo, hi, midpoint) = edge_costs[edge_idx];
        edge_idx += 1;
        let ra = root(&mut remap, lo);
        let rb = root(&mut remap, hi);
        if ra == rb {
            continue;
        }
        working_vertices[ra as usize] = midpoint;
        let combined = quadrics[ra as usize].add(&quadrics[rb as usize]);
        quadrics[ra as usize] = combined;
        remap[rb as usize] = ra;

        let mut degenerate = 0usize;
        for tri in &working_triangles {
            let a = root(&mut remap, tri[0]);
            let b = root(&mut remap, tri[1]);
            let c = root(&mut remap, tri[2]);
            if a == b || b == c || a == c {
                degenerate += 1;
            }
        }
        remaining_triangles = original_triangle_count - degenerate;
    }

    finalize_after_remap(&working_vertices, &working_triangles, &mut remap)
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
        return Err(PixieError::InvalidInput("Indices must be triangle list".to_string()));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_mesh() -> (Vec<f32>, Vec<u32>) {
        let vertices: Vec<f32> = vec![
            0.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 1.0, 0.0,  0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,  1.0, 0.0, 1.0,  1.0, 1.0, 1.0,  0.0, 1.0, 1.0,
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3,
            4, 6, 5, 4, 7, 6,
            0, 4, 5, 0, 5, 1,
            2, 6, 7, 2, 7, 3,
            0, 3, 7, 0, 7, 4,
            1, 5, 6, 1, 6, 2,
        ];
        (vertices, indices)
    }

    #[test]
    fn test_weld_dedupes_coincident_vertices() {
        let vertices = vec![
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
        ];
        let indices = vec![0, 2, 3, 1, 2, 3];
        let (new_verts, new_indices) = weld_vertices_spatial_hash(&vertices, &indices, 1e-4).unwrap();
        assert_eq!(new_verts.len() / 3, 3);
        assert_eq!(new_indices.len(), 3);
    }

    #[test]
    fn test_vertex_clustering_reduces() {
        let (verts, indices) = cube_mesh();
        let (new_verts, new_indices) = decimate_vertex_clustering(&verts, &indices, 0.5).unwrap();
        assert!(new_verts.len() / 3 <= verts.len() / 3);
        assert!(new_indices.len() % 3 == 0);
    }

    #[test]
    fn test_qem_reduces() {
        let (verts, indices) = cube_mesh();
        let (_new_verts, new_indices) = decimate_qem(&verts, &indices, 0.5).unwrap();
        assert!(new_indices.len() % 3 == 0);
        assert!(new_indices.len() / 3 <= indices.len() / 3);
    }

    #[test]
    fn test_edge_collapse_reduces() {
        let (verts, indices) = cube_mesh();
        let (_new_verts, new_indices) = decimate_edge_collapse(&verts, &indices, 0.5).unwrap();
        assert!(new_indices.len() % 3 == 0);
        assert!(new_indices.len() / 3 <= indices.len() / 3);
    }
}
