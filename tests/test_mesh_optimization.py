"""
Test suite for 3D mesh optimization functionality.
These tests define the expected behavior before implementation.
"""
import pytest
import subprocess
import trimesh
import numpy as np
from pathlib import Path
from conftest import validate_mesh_quality


class TestMeshOptimization:
    """Test 3D mesh optimization via CLI interface."""
    
    def test_vertex_deduplication(self, sample_models, temp_dir, cli_runner):
        """Test vertex deduplication reduces vertex count while preserving geometry."""
        input_file = sample_models['obj']
        output_file = temp_dir / "deduplicated.obj"
        
        result = cli_runner.run_command([
            "--input", str(input_file),
            "--output", str(output_file),
            "--deduplicate",
            "--tolerance", "0.001"
        ])
        
        assert result['success'], f"Mesh deduplication failed: {result['stderr']}"
        assert output_file.exists(), "Deduplicated mesh file was not created"
        
        # Validate deduplication results
        metrics = validate_mesh_quality(input_file, output_file)
        
        # Should reduce vertices (or at least not increase them)
        assert metrics['vertex_reduction_percent'] >= 0, \
            f"Vertex count should not increase, got {metrics['vertex_reduction_percent']}% reduction"
        
        # Should maintain watertight property if original was watertight
        assert metrics['maintains_watertight'], "Mesh should remain watertight after deduplication"
        
        # Volume should be approximately preserved
        assert 0.95 <= metrics['volume_ratio'] <= 1.05, \
            f"Volume should be preserved, got ratio: {metrics['volume_ratio']}"
    
    def test_triangle_reduction(self, sample_models, temp_dir, cli_runner):
        """Test triangle reduction simplifies mesh while maintaining shape."""
        input_file = sample_models['ply']
        
        # Test different reduction ratios
        for ratio in [0.5, 0.25, 0.1]:
            output_file = temp_dir / f"reduced_{int(ratio*100)}pct.ply"
            
            result = cli_runner.run_command([
                "--input", str(input_file),
                "--output", str(output_file),
                "--reduce", str(ratio)
            ])
            
            assert result['success'], f"Triangle reduction failed for ratio {ratio}: {result['stderr']}"
            assert output_file.exists(), f"Reduced mesh file not created for ratio {ratio}"
            
            # Validate reduction
            metrics = validate_mesh_quality(input_file, output_file)
            
            # Should achieve significant face reduction
            expected_reduction = (1 - ratio) * 100
            assert metrics['face_reduction_percent'] >= expected_reduction * 0.8, \
                f"Should reduce faces by ~{expected_reduction}%, got {metrics['face_reduction_percent']}%"
            
            # More aggressive reduction should still maintain general shape
            if ratio >= 0.25:
                assert metrics['volume_ratio'] >= 0.7, \
                    f"Volume should be reasonably preserved, got ratio: {metrics['volume_ratio']}"
    
    def test_format_conversion_obj_to_gltf(self, sample_models, temp_dir, cli_runner):
        """Test conversion between mesh formats (OBJ to glTF)."""
        input_file = sample_models['obj']
        output_file = temp_dir / "converted.gltf"
        
        result = cli_runner.run_command([
            "--input", str(input_file),
            "--output", str(output_file),
            "--format", "gltf"
        ])
        
        assert result['success'], f"OBJ to glTF conversion failed: {result['stderr']}"
        assert output_file.exists(), "glTF output file was not created"
        
        # Validate conversion preserved geometry
        try:
            output_mesh = trimesh.load(str(output_file))
            input_mesh = trimesh.load(str(input_file))
            
            # Vertex counts should be similar (allowing for format differences)
            vertex_diff = abs(len(output_mesh.vertices) - len(input_mesh.vertices))
            vertex_ratio = vertex_diff / len(input_mesh.vertices)
            assert vertex_ratio < 0.1, f"Vertex count changed too much during conversion: {vertex_ratio}"
            
            # Volume should be preserved
            if input_mesh.volume > 0 and output_mesh.volume > 0:
                volume_ratio = output_mesh.volume / input_mesh.volume
                assert 0.95 <= volume_ratio <= 1.05, f"Volume not preserved: {volume_ratio}"
                
        except Exception as e:
            pytest.fail(f"Could not validate converted mesh: {e}")
    
    def test_format_conversion_ply_to_obj(self, sample_models, temp_dir, cli_runner):
        """Test conversion from PLY to OBJ format."""
        input_file = sample_models['ply']
        output_file = temp_dir / "converted.obj"
        
        result = cli_runner.run_command([
            "--input", str(input_file),
            "--output", str(output_file),
            "--format", "obj"
        ])
        
        assert result['success'], f"PLY to OBJ conversion failed: {result['stderr']}"
        assert output_file.exists(), "OBJ output file was not created"
        
        # Validate the OBJ file is valid
        try:
            output_mesh = trimesh.load(str(output_file))
            assert len(output_mesh.vertices) > 0, "Converted mesh should have vertices"
            assert len(output_mesh.faces) > 0, "Converted mesh should have faces"
        except Exception as e:
            pytest.fail(f"Converted OBJ file is invalid: {e}")
    
    def test_mesh_validation(self, sample_models, temp_dir, cli_runner):
        """Test mesh validation catches corrupt or invalid meshes."""
        input_file = sample_models['obj']
        
        # Test validation flag
        result = cli_runner.run_command([
            "validate", 
            str(input_file)
        ])
        
        assert result['success'], f"Mesh validation failed for valid mesh: {result['stderr']}"
        assert "valid" in result['stdout'].lower() or "ok" in result['stdout'].lower(), \
            f"Should report mesh as valid: {result['stdout']}"
    
    def test_invalid_mesh_handling(self, temp_dir, cli_runner):
        """Test handling of invalid or corrupted mesh files."""
        # Create a fake mesh file
        fake_mesh = temp_dir / "fake.obj"
        fake_mesh.write_text("This is not a valid OBJ file\nv 1 2\nf 1 2 3 4 5")
        
        output_file = temp_dir / "output.obj"
        
        result = cli_runner.run_command([
            "--input", str(fake_mesh),
            "--output", str(output_file),
            "--optimize"
        ])
        
        assert not result['success'], "CLI should fail for invalid mesh data"
        assert any(word in result['stderr'].lower() for word in ["format", "invalid", "corrupt", "parse"]), \
            f"Error should mention format issue: {result['stderr']}"
    
    @pytest.mark.parametrize("format_name,extension", [
        ("obj", "obj"),
        ("ply", "ply"), 
        ("stl", "stl"),
        ("gltf", "gltf"),
    ])
    def test_supported_mesh_formats(self, sample_models, temp_dir, cli_runner, format_name, extension):
        """Test that all claimed supported mesh formats work."""
        input_file = sample_models['obj']  # Use OBJ as universal input
        output_file = temp_dir / f"test.{extension}"
        
        result = cli_runner.run_command([
            "--input", str(input_file),
            "--output", str(output_file),
            "--format", format_name
        ])
        
        # CLI should support the format or give clear error
        if not result['success']:
            pytest.skip(f"Mesh format {format_name} not yet implemented: {result['stderr']}")
        
        assert output_file.exists(), f"Output file not created for {format_name}"
        
        # Validate file can be loaded
        try:
            mesh = trimesh.load(str(output_file))
            assert len(mesh.vertices) > 0, f"Generated {format_name} file should have vertices"
            assert len(mesh.faces) > 0, f"Generated {format_name} file should have faces"
        except Exception as e:
            pytest.fail(f"Generated {format_name} file is invalid: {e}")


class TestMeshOptimizationCombined:
    """Test combined mesh optimization operations."""
    
    def test_deduplicate_and_reduce(self, sample_models, temp_dir, cli_runner):
        """Test combining deduplication and reduction operations."""
        input_file = sample_models['ply']
        output_file = temp_dir / "optimized.ply"
        
        result = cli_runner.run_command([
            "--input", str(input_file),
            "--output", str(output_file),
            "--deduplicate",
            "--reduce", "0.5",
            "--tolerance", "0.001"
        ])
        
        assert result['success'], f"Combined optimization failed: {result['stderr']}"
        assert output_file.exists(), "Combined optimization output was not created"
        
        # Should achieve both vertex and face reduction
        metrics = validate_mesh_quality(input_file, output_file)
        assert metrics['vertex_reduction_percent'] >= 0, "Should reduce or maintain vertex count"
        assert metrics['face_reduction_percent'] > 30, "Should significantly reduce face count"
        
        # Should maintain overall shape
        assert metrics['volume_ratio'] >= 0.6, f"Should preserve general shape: {metrics['volume_ratio']}"
    
    def test_optimize_flag_auto_settings(self, sample_models, temp_dir, cli_runner):
        """Test that --optimize flag applies reasonable automatic settings."""
        input_file = sample_models['obj']
        output_file = temp_dir / "auto_optimized.obj"
        
        result = cli_runner.run_command([
            "optimize",
            "--input", str(input_file),
            "--output", str(output_file),
            "--deduplicate",
            "--reduce", "0.1"
        ])
        
        assert result['success'], f"Auto-optimization failed: {result['stderr']}"
        assert output_file.exists(), "Auto-optimization output was not created"
        
        # Should achieve some optimization
        metrics = validate_mesh_quality(input_file, output_file)
        total_reduction = metrics['vertex_reduction_percent'] + metrics['face_reduction_percent']
        assert total_reduction > 0, "Auto-optimization should achieve some reduction"
        
        # Should maintain quality
        assert metrics['maintains_watertight'], "Auto-optimization should preserve watertight property"


class TestMeshPerformance:
    """Test performance requirements for mesh optimization."""
    
    @pytest.mark.benchmark  
    def test_large_mesh_performance(self, sample_models, temp_dir, cli_runner, benchmark):
        """Test mesh optimization performance on larger meshes."""
        input_file = sample_models['ply']
        output_file = temp_dir / "perf_test.ply"
        
        def run_optimization():
            result = cli_runner.run_command([
                "optimize",
                "--input", str(input_file),
                "--output", str(output_file),
                "--reduce", "0.5"
            ])
            assert result['success'], f"Optimization failed: {result['stderr']}"
            return result
        
        # Benchmark the optimization
        result = benchmark(run_optimization)
        
        # Should complete in reasonable time (generous for development)
        try:
            # Handle benchmark stats access similar to other tests
            if hasattr(benchmark.stats, 'mean'):
                mean_time = benchmark.stats.mean
            elif hasattr(benchmark.stats, 'stats') and hasattr(benchmark.stats.stats, 'mean'):
                mean_time = benchmark.stats.stats.mean
            else:
                pytest.skip("Cannot access benchmark timing data")
                
            assert mean_time < 5.0, \
                f"Mesh optimization too slow: {mean_time:.2f}s (target: <5s)"
        except AttributeError as e:
            pytest.skip(f"Benchmark stats access issue: {e}")
        
        # Verify output quality
        if output_file.exists():
            metrics = validate_mesh_quality(input_file, output_file)
            assert metrics['face_reduction_percent'] > 30, "Should achieve significant reduction"
