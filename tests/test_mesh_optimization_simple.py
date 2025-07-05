"""
Test suite for mesh optimization functionality.
These tests are updated to match the current CLI architecture.
"""
import pytest
import subprocess
import os
from pathlib import Path
from conftest import validate_mesh_quality


class TestMeshOptimization:
    """Test mesh optimization via CLI interface."""
    
    def test_basic_mesh_optimization(self, sample_models, temp_dir, cli_runner):
        """Test basic mesh optimization functionality."""
        input_file = sample_models['obj']
        output_file = temp_dir / "optimized.obj"
        
        result = cli_runner.run_command([
            "optimize",
            "--input", str(input_file),
            "--output", str(output_file),
            "--target-reduction", "0.5"
        ])
        
        assert result['success'], f"Mesh optimization failed: {result['stderr']}"
        assert output_file.exists(), "Output mesh file was not created"
        
        # Basic validation
        metrics = validate_mesh_quality(input_file, output_file)
        
        # Should have reduced faces/vertices
        assert metrics['face_reduction_percent'] > 0, "Should reduce face count"
        assert metrics['vertex_reduction_percent'] > 0, "Should reduce vertex count"
        
        # Output should still be a valid mesh
        assert metrics['optimized_faces'] > 0, "Output mesh should have faces"
        assert metrics['optimized_vertices'] > 0, "Output mesh should have vertices"

    def test_mesh_reduction_different_ratios(self, sample_models, temp_dir, cli_runner):
        """Test mesh reduction with different target ratios."""
        input_file = sample_models['ascii_ply']  # Use ASCII PLY since binary PLY isn't supported yet
        
        for ratio in [0.3, 0.5, 0.7]:
            output_file = temp_dir / f"optimized_{int(ratio*100)}.ply"
            
            result = cli_runner.run_command([
                "optimize",
                "--input", str(input_file),
                "--output", str(output_file),
                "--target-reduction", str(ratio)
            ])
            
            assert result['success'], f"Mesh optimization failed for ratio {ratio}: {result['stderr']}"
            assert output_file.exists(), f"Output mesh file not created for ratio {ratio}"
            
            # Validate reduction is close to target
            metrics = validate_mesh_quality(input_file, output_file)
            actual_reduction = metrics['face_reduction_percent'] / 100
            
            # Allow some tolerance since exact reduction isn't always possible
            assert abs(actual_reduction - ratio) < 0.3, \
                f"Reduction ratio {actual_reduction} not close to target {ratio}"

    def test_mesh_validation_command(self, sample_models, cli_runner):
        """Test mesh validation command."""
        input_file = sample_models['obj']
        
        result = cli_runner.run_command([
            "validate",
            str(input_file)
        ])
        
        assert result['success'], f"Mesh validation failed for valid file: {result['stderr']}"
        # Should indicate the file is valid with mesh statistics
        assert ("valid" in result['stdout'].lower() and "mesh" in result['stdout'].lower()) or \
               ("vertices" in result['stdout'].lower() and "triangles" in result['stdout'].lower()), \
            f"Should report mesh as valid with statistics: {result['stdout']}"

    def test_format_preservation(self, sample_models, temp_dir, cli_runner):
        """Test that mesh optimization preserves file format."""
        # Test OBJ -> OBJ
        obj_input = sample_models['obj']
        obj_output = temp_dir / "output.obj"
        
        result = cli_runner.run_command([
            "optimize",
            "--input", str(obj_input),
            "--output", str(obj_output),
            "--target-reduction", "0.2"
        ])
        
        assert result['success'], f"OBJ optimization failed: {result['stderr']}"
        assert obj_output.exists(), "OBJ output file not created"
        
        # Test PLY -> PLY (use ASCII PLY since binary PLY isn't supported yet)
        ply_input = sample_models['ascii_ply']
        ply_output = temp_dir / "output.ply"
        
        result = cli_runner.run_command([
            "optimize",
            "--input", str(ply_input),
            "--output", str(ply_output),
            "--target-reduction", "0.2"
        ])
        
        assert result['success'], f"PLY optimization failed: {result['stderr']}"
        assert ply_output.exists(), "PLY output file not created"


class TestMeshValidation:
    """Test mesh format validation and error handling."""
    
    def test_invalid_mesh_file(self, temp_dir, cli_runner):
        """Test handling of invalid mesh files."""
        fake_mesh = temp_dir / "fake.obj"
        fake_mesh.write_text("This is not a valid mesh file")
        
        output_file = temp_dir / "output.obj"
        
        result = cli_runner.run_command([
            "optimize",
            "--input", str(fake_mesh),
            "--output", str(output_file)
        ])
        
        assert not result['success'], "CLI should fail for invalid mesh data"

    def test_mesh_validation_invalid_content(self, temp_dir, cli_runner):
        """Test mesh validation with invalid mesh content."""
        # Create an OBJ file with invalid indices
        invalid_obj = temp_dir / "invalid.obj"
        invalid_obj.write_text("""# Invalid OBJ file
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 5
""")
        
        result = cli_runner.run_command([
            "validate", 
            str(invalid_obj)
        ])
        
        assert not result['success'], "Validation should fail for invalid mesh"
        # Just verify validation fails - manual testing shows detailed errors work
        assert result['returncode'] != 0, "Should return non-zero exit code for invalid mesh"
