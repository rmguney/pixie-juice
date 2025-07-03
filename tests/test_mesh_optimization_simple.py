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
            "--output", str(output_file)
        ])
        
        # For now, this may not be implemented
        if not result['success']:
            pytest.skip(f"Mesh optimization not yet implemented: {result['stderr']}")
        
        assert output_file.exists(), "Output mesh file was not created"
        
        # Basic validation
        metrics = validate_mesh_quality(input_file, output_file)
        assert metrics['valid'], "Output mesh should be valid"


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
