//! Mesh validation functionality

use crate::ffi::mesh_ffi::MeshData;
use crate::types::{OptError, OptResult};

/// Validation report containing errors and warnings
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }
}

/// Mesh validator for checking mesh integrity
pub struct MeshValidator;

impl MeshValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate mesh data and return a report
    pub fn validate(&self, mesh: &MeshData) -> OptResult<ValidationReport> {
        let mut report = ValidationReport::new();

        // Basic validation
        self.check_vertex_data(mesh, &mut report)?;
        self.check_index_data(mesh, &mut report)?;

        // Set validity based on errors
        report.is_valid = report.errors.is_empty();

        Ok(report)
    }

    fn check_vertex_data(&self, mesh: &MeshData, report: &mut ValidationReport) -> OptResult<()> {
        if mesh.vertices.is_empty() {
            report.errors.push("Mesh has no vertices".to_string());
            return Ok(());
        }

        if mesh.vertices.len() % 3 != 0 {
            report.errors.push(format!(
                "Vertex count {} is not divisible by 3",
                mesh.vertices.len()
            ));
        }

        // Check for NaN or infinite values
        for (i, &vertex) in mesh.vertices.iter().enumerate() {
            if !vertex.is_finite() {
                report.errors.push(format!("Invalid vertex value at index {}: {}", i, vertex));
                break; // Only report first occurrence
            }
        }

        Ok(())
    }

    fn check_index_data(&self, mesh: &MeshData, report: &mut ValidationReport) -> OptResult<()> {
        if mesh.indices.is_empty() {
            report.warnings.push("Mesh has no indices (point cloud?)".to_string());
            return Ok(());
        }

        if mesh.indices.len() % 3 != 0 {
            report.errors.push(format!(
                "Index count {} is not divisible by 3",
                mesh.indices.len()
            ));
        }

        let vertex_count = mesh.vertex_count() as u32;

        // Check for out-of-bounds indices
        for (i, &index) in mesh.indices.iter().enumerate() {
            if index >= vertex_count {
                report.errors.push(format!(
                    "Index {} at position {} is out of bounds (vertex count: {})",
                    index, i, vertex_count
                ));
                break; // Only report first occurrence
            }
        }

        // Check for degenerate triangles
        let mut degenerate_count = 0;
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                if chunk[0] == chunk[1] || chunk[1] == chunk[2] || chunk[0] == chunk[2] {
                    degenerate_count += 1;
                }
            }
        }

        if degenerate_count > 0 {
            report.warnings.push(format!("{} degenerate triangles found", degenerate_count));
        }

        Ok(())
    }
}

impl Default for MeshValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mesh() {
        let validator = MeshValidator::new();
        let mesh = MeshData {
            vertices: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            indices: vec![0, 1, 2],
        };

        let report = validator.validate(&mesh).unwrap();
        assert!(report.is_valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_invalid_mesh_vertex_count() {
        let validator = MeshValidator::new();
        let mesh = MeshData {
            vertices: vec![0.0, 0.0], // Only 2 values, not divisible by 3
            indices: vec![0],
        };

        let report = validator.validate(&mesh).unwrap();
        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_out_of_bounds_index() {
        let validator = MeshValidator::new();
        let mesh = MeshData {
            vertices: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            indices: vec![0, 1, 5], // Index 5 is out of bounds (only 3 vertices)
        };

        let report = validator.validate(&mesh).unwrap();
        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
    }
}
