/// FFI bindings for mathematical operations C hotspots
/// Provides SIMD-optimized vector, matrix, and quaternion operations

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub m: [f32; 16], // Column-major order
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// Conditionally compile C FFI declarations only when c_hotspots feature is enabled
#[cfg(feature = "c_hotspots")]
extern "C" {
    // SIMD-optimized vector operations
    fn vec3_add_simd(a: *const Vec3, b: *const Vec3, result: *mut Vec3, count: usize);
    fn vec3_sub_simd(a: *const Vec3, b: *const Vec3, result: *mut Vec3, count: usize);
    fn vec3_mul_scalar_simd(vectors: *const Vec3, scalar: f32, result: *mut Vec3, count: usize);
    fn vec3_dot_simd(a: *const Vec3, b: *const Vec3, result: *mut f32, count: usize);
    fn vec3_cross_simd(a: *const Vec3, b: *const Vec3, result: *mut Vec3, count: usize);
    fn vec3_normalize_simd(vectors: *mut Vec3, count: usize);
    
    // Matrix operations
    fn mat4_identity(matrix: *mut Mat4);
    fn mat4_multiply(a: *const Mat4, b: *const Mat4, result: *mut Mat4);
    fn mat4_multiply_simd(matrices_a: *const Mat4, matrices_b: *const Mat4, results: *mut Mat4, count: usize);
    fn mat4_transpose(matrix: *mut Mat4);
    fn mat4_inverse(matrix: *mut Mat4) -> bool;
    fn mat4_determinant(matrix: *const Mat4) -> f32;
    
    // Transform operations
    fn transform_points(matrix: *const Mat4, points: *const Vec3, result: *mut Vec3, count: usize);
    fn transform_vectors(matrix: *const Mat4, vectors: *const Vec3, result: *mut Vec3, count: usize);
    fn transform_points_batch(matrices: *const Mat4, points: *const Vec3, results: *mut Vec3, matrix_count: usize, point_count: usize);
    fn transform_vectors_batch(matrices: *const Mat4, vectors: *const Vec3, results: *mut Vec3, matrix_count: usize, vector_count: usize);
    
    // Quaternion operations
    fn quat_multiply(a: *const Quat, b: *const Quat, result: *mut Quat);
    fn quat_normalize(quat: *mut Quat);
    fn quat_conjugate(quat: *const Quat, result: *mut Quat);
    fn quat_slerp(a: *const Quat, b: *const Quat, t: f32, result: *mut Quat);
    fn quat_slerp_batch(quats_a: *const Quat, quats_b: *const Quat, t_values: *const f32, results: *mut Quat, count: usize);
    fn quat_to_matrix(quat: *const Quat, matrix: *mut Mat4);
    fn matrix_to_quat(matrix: *const Mat4, quat: *mut Quat);
}

/// Safe wrapper for batch vector addition
pub fn vec3_add_batch(a: &[Vec3], b: &[Vec3], result: &mut [Vec3]) -> bool {
    if a.len() != b.len() || a.len() != result.len() || a.is_empty() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        vec3_add_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), a.len());
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for i in 0..a.len() {
            result[i] = Vec3 {
                x: a[i].x + b[i].x,
                y: a[i].y + b[i].y,
                z: a[i].z + b[i].z,
            };
        }
    }
    
    true
}

/// Safe wrapper for batch vector subtraction
pub fn vec3_sub_batch(a: &[Vec3], b: &[Vec3], result: &mut [Vec3]) -> bool {
    if a.len() != b.len() || a.len() != result.len() || a.is_empty() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        vec3_sub_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), a.len());
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for i in 0..a.len() {
            result[i] = Vec3 {
                x: a[i].x - b[i].x,
                y: a[i].y - b[i].y,
                z: a[i].z - b[i].z,
            };
        }
    }
    
    true
}

/// Safe wrapper for batch scalar multiplication
pub fn vec3_mul_scalar_batch(vectors: &[Vec3], scalar: f32, result: &mut [Vec3]) -> bool {
    if vectors.len() != result.len() || vectors.is_empty() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        vec3_mul_scalar_simd(vectors.as_ptr(), scalar, result.as_mut_ptr(), vectors.len());
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for i in 0..vectors.len() {
            result[i] = Vec3 {
                x: vectors[i].x * scalar,
                y: vectors[i].y * scalar,
                z: vectors[i].z * scalar,
            };
        }
    }
    
    true
}

/// Safe wrapper for batch dot product
pub fn vec3_dot_batch(a: &[Vec3], b: &[Vec3], result: &mut [f32]) -> bool {
    if a.len() != b.len() || a.len() != result.len() || a.is_empty() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        vec3_dot_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), a.len());
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for i in 0..a.len() {
            result[i] = a[i].x * b[i].x + a[i].y * b[i].y + a[i].z * b[i].z;
        }
    }
    
    true
}

/// Safe wrapper for batch cross product
pub fn vec3_cross_batch(a: &[Vec3], b: &[Vec3], result: &mut [Vec3]) -> bool {
    if a.len() != b.len() || a.len() != result.len() || a.is_empty() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        vec3_cross_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), a.len());
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for i in 0..a.len() {
            result[i] = Vec3 {
                x: a[i].y * b[i].z - a[i].z * b[i].y,
                y: a[i].z * b[i].x - a[i].x * b[i].z,
                z: a[i].x * b[i].y - a[i].y * b[i].x,
            };
        }
    }
    
    true
}

/// Safe wrapper for batch matrix multiplication
pub fn mat4_multiply_batch(a: &[Mat4], b: &[Mat4], result: &mut [Mat4]) -> bool {
    if a.len() != b.len() || a.len() != result.len() || a.is_empty() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        mat4_multiply_simd(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), a.len());
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation - basic matrix multiplication
        for i in 0..a.len() {
            // Simple 4x4 matrix multiplication
            let mut m = [0.0f32; 16];
            for row in 0..4 {
                for col in 0..4 {
                    for k in 0..4 {
                        m[row * 4 + col] += a[i].m[row * 4 + k] * b[i].m[k * 4 + col];
                    }
                }
            }
            result[i] = Mat4 { m };
        }
    }
    
    true
}

/// Safe wrapper for batch point transformation
pub fn transform_points_batch(matrices: &[Mat4], points: &[Vec3], results: &mut [Vec3]) -> bool {
    if matrices.is_empty() || points.is_empty() || results.len() != matrices.len() * points.len() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        transform_points_batch(
            matrices.as_ptr(),
            points.as_ptr(),
            results.as_mut_ptr(),
            matrices.len(),
            points.len(),
        );
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for (mat_idx, matrix) in matrices.iter().enumerate() {
            for (pt_idx, point) in points.iter().enumerate() {
                let result_idx = mat_idx * points.len() + pt_idx;
                if result_idx < results.len() {
                    // Simple matrix-vector multiplication (assuming w=1 for points)
                    results[result_idx] = Vec3 {
                        x: matrix.m[0] * point.x + matrix.m[4] * point.y + matrix.m[8] * point.z + matrix.m[12],
                        y: matrix.m[1] * point.x + matrix.m[5] * point.y + matrix.m[9] * point.z + matrix.m[13],
                        z: matrix.m[2] * point.x + matrix.m[6] * point.y + matrix.m[10] * point.z + matrix.m[14],
                    };
                }
            }
        }
    }
    
    true
}

/// Safe wrapper for batch vector transformation
pub fn transform_vectors_batch(matrices: &[Mat4], vectors: &[Vec3], results: &mut [Vec3]) -> bool {
    if matrices.is_empty() || vectors.is_empty() || results.len() != matrices.len() * vectors.len() {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        transform_vectors_batch(
            matrices.as_ptr(),
            vectors.as_ptr(),
            results.as_mut_ptr(),
            matrices.len(),
            vectors.len(),
        );
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation
        for (mat_idx, matrix) in matrices.iter().enumerate() {
            for (vec_idx, vector) in vectors.iter().enumerate() {
                let result_idx = mat_idx * vectors.len() + vec_idx;
                if result_idx < results.len() {
                    // Simple matrix-vector multiplication (assuming w=0 for vectors)
                    results[result_idx] = Vec3 {
                        x: matrix.m[0] * vector.x + matrix.m[4] * vector.y + matrix.m[8] * vector.z,
                        y: matrix.m[1] * vector.x + matrix.m[5] * vector.y + matrix.m[9] * vector.z,
                        z: matrix.m[2] * vector.x + matrix.m[6] * vector.y + matrix.m[10] * vector.z,
                    };
                }
            }
        }
    }
    
    true
}

/// Safe wrapper for batch quaternion slerp
pub fn quat_slerp_batch_safe(
    quats_a: &[Quat],
    quats_b: &[Quat],
    t_values: &[f32],
    results: &mut [Quat],
) -> bool {
    if quats_a.len() != quats_b.len()
        || quats_a.len() != t_values.len()
        || quats_a.len() != results.len()
        || quats_a.is_empty()
    {
        return false;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        quat_slerp_batch(
            quats_a.as_ptr(),
            quats_b.as_ptr(),
            t_values.as_ptr(),
            results.as_mut_ptr(),
            quats_a.len(),
        );
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust fallback implementation - simple linear interpolation
        for i in 0..quats_a.len() {
            let t = t_values[i].clamp(0.0, 1.0);
            let inv_t = 1.0 - t;
            results[i] = Quat {
                x: inv_t * quats_a[i].x + t * quats_b[i].x,
                y: inv_t * quats_a[i].y + t * quats_b[i].y,
                z: inv_t * quats_a[i].z + t * quats_b[i].z,
                w: inv_t * quats_a[i].w + t * quats_b[i].w,
            };
            // Normalize the result
            let len = (results[i].x * results[i].x + results[i].y * results[i].y + 
                      results[i].z * results[i].z + results[i].w * results[i].w).sqrt();
            if len > 0.0 {
                results[i].x /= len;
                results[i].y /= len;
                results[i].z /= len;
                results[i].w /= len;
            }
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vec3_operations() {
        let a = vec![Vec3 { x: 1.0, y: 2.0, z: 3.0 }];
        let b = vec![Vec3 { x: 4.0, y: 5.0, z: 6.0 }];
        let mut result = vec![Vec3 { x: 0.0, y: 0.0, z: 0.0 }];
        
        assert!(vec3_add_batch(&a, &b, &mut result));
        assert_eq!(result[0], Vec3 { x: 5.0, y: 7.0, z: 9.0 });
        
        assert!(vec3_sub_batch(&b, &a, &mut result));
        assert_eq!(result[0], Vec3 { x: 3.0, y: 3.0, z: 3.0 });
        
        assert!(vec3_mul_scalar_batch(&a, 2.0, &mut result));
        assert_eq!(result[0], Vec3 { x: 2.0, y: 4.0, z: 6.0 });
        
        let mut dot_result = vec![0.0f32];
        assert!(vec3_dot_batch(&a, &b, &mut dot_result));
        assert_eq!(dot_result[0], 32.0); // 1*4 + 2*5 + 3*6 = 32
        
        assert!(vec3_cross_batch(&a, &b, &mut result));
        assert_eq!(result[0], Vec3 { x: -3.0, y: 6.0, z: -3.0 });
    }
    
    #[test]
    fn test_quaternion_slerp() {
        let quats_a = vec![Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }];
        let quats_b = vec![Quat { x: 1.0, y: 0.0, z: 0.0, w: 0.0 }];
        let t_values = vec![0.5];
        let mut results = vec![Quat { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }];
        
        assert!(quat_slerp_batch_safe(&quats_a, &quats_b, &t_values, &mut results));
        // Result should be normalized
        let len = (results[0].x * results[0].x + results[0].y * results[0].y + 
                  results[0].z * results[0].z + results[0].w * results[0].w).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }
}
