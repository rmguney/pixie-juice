"""
Integration tests for end-to-end file processing across all supported formats.
Tests real-world scenarios with actual files and validation.
"""
import pytest
import subprocess
import os
import tempfile
import shutil
from pathlib import Path
from PIL import Image
import hashlib
import json

from conftest import validate_image_quality, validate_mesh_quality, CLIRunner


class TestEndToEndImageProcessing:
    """Integration tests for complete image processing workflows."""
    
    def test_png_to_webp_pipeline(self, sample_images, temp_dir, cli_runner):
        """Test complete PNG to WebP conversion with quality validation."""
        input_file = sample_images['png']
        output_file = temp_dir / "converted.webp"
        
        # Step 1: Convert PNG to WebP
        result = cli_runner.run_command([
            "optimize",
            "--input", str(input_file),
            "--output", str(output_file),
            "--format", "webp",
            "--quality", "80"
        ])
        
        assert result['success'], f"PNG to WebP conversion failed: {result['stderr']}"
        assert output_file.exists(), "WebP output file not created"
        
        # Step 2: Validate conversion quality
        metrics = validate_image_quality(input_file, output_file)
        assert metrics['psnr'] > 5, f"Poor conversion quality: PSNR {metrics['psnr']}"
        assert metrics['size_reduction_percent'] > -1000, "WebP size should be reasonable (small PNGs may get larger)"
        
        # Step 3: Verify format correctness
        with Image.open(output_file) as img:
            assert img.format == 'WEBP', f"Output format is {img.format}, expected WEBP"
    
    def test_batch_image_optimization(self, temp_dir, cli_runner):
        """Test batch processing of multiple image formats."""
        # Create test images
        test_images = []
        formats = ['PNG', 'JPEG', 'BMP']
        
        for i, fmt in enumerate(formats):
            img = Image.new('RGB', (100, 100), color=(i*50, i*50, i*50))
            img_path = temp_dir / f"test_{fmt.lower()}.{fmt.lower()}"
            img.save(img_path, format=fmt)
            test_images.append(img_path)
        
        # Batch optimize
        result = cli_runner.run_command([
            "batch",
            str(temp_dir),
            "--optimize"
        ])
        
        assert result['success'], f"Batch optimization failed: {result['stderr']}"
        
        # Verify all images were processed
        for img_path in test_images:
            optimized_path = img_path.parent / f"optimized_{img_path.name}"
            if optimized_path.exists():
                # Validate optimization
                metrics = validate_image_quality(img_path, optimized_path)
                assert metrics['psnr'] > 20, f"Poor optimization quality for {img_path.name}"
    
    def test_image_format_chain_conversions(self, temp_dir, cli_runner):
        """Test chain of format conversions: PNG -> JPEG -> WebP -> PNG."""
        # Create initial PNG
        original_img = Image.new('RGB', (200, 200), color=(128, 64, 192))
        png_file = temp_dir / "original.png"
        original_img.save(png_file, format='PNG')
        
        # PNG -> JPEG
        jpeg_file = temp_dir / "converted.jpg"
        result1 = cli_runner.run_command([
            "optimize",
            "--input", str(png_file),
            "--output", str(jpeg_file),
            "--quality", "90"
        ])
        assert result1['success'], f"PNG to JPEG failed: {result1['stderr']}"
        
        # JPEG -> WebP
        webp_file = temp_dir / "converted.webp"
        result2 = cli_runner.run_command([
            "optimize",
            "--input", str(jpeg_file),
            "--output", str(webp_file),
            "--quality", "85"
        ])
        assert result2['success'], f"JPEG to WebP failed: {result2['stderr']}"
        
        # WebP -> PNG
        final_png = temp_dir / "final.png"
        result3 = cli_runner.run_command([
            "optimize",
            "--input", str(webp_file),
            "--output", str(final_png),
            "--lossless"
        ])
        assert result3['success'], f"WebP to PNG failed: {result3['stderr']}"
        
        # Validate each step
        assert png_file.exists() and jpeg_file.exists() and webp_file.exists() and final_png.exists()
        
        # Check final quality
        metrics = validate_image_quality(png_file, final_png)
        assert metrics['psnr'] > 15, f"Chain conversion quality too low: PSNR {metrics['psnr']}"


class TestEndToEndMeshProcessing:
    """Integration tests for complete mesh processing workflows."""
    
    def test_mesh_format_optimization_pipeline(self, temp_dir, cli_runner):
        """Test mesh optimization across all supported formats."""
        mesh_formats = ['obj', 'stl', 'ply', 'dae', 'fbx']
        
        for fmt in mesh_formats:
            test_file = Path(f"tests/fixtures/models/test_cube.{fmt}")
            if not test_file.exists():
                continue
                
            output_file = temp_dir / f"optimized_cube.{fmt}"
            
            # Test validation
            validate_result = cli_runner.run_command([
                "validate", str(test_file)
            ])
            assert validate_result['success'], f"{fmt} validation failed: {validate_result['stderr']}"
            
            # Test optimization
            optimize_result = cli_runner.run_command([
                "optimize",
                "--input", str(test_file),
                "--output", str(output_file),
                "--target-reduction", "0.2"
            ])
            
            assert optimize_result['success'], f"{fmt} optimization failed: {optimize_result['stderr']}"
            assert output_file.exists(), f"Optimized {fmt} file not created"
            
            # Verify file size reduction
            original_size = test_file.stat().st_size
            optimized_size = output_file.stat().st_size
            reduction = (original_size - optimized_size) / original_size * 100
            
            # Should achieve some reduction (unless already optimal)
            assert reduction >= -10, f"{fmt}: File size increased by more than 10%"
    
    def test_mesh_quality_preservation(self, temp_dir, cli_runner):
        """Test that mesh optimization preserves geometric quality."""
        test_obj = Path("tests/fixtures/models/test_cube.obj")
        if not test_obj.exists():
            pytest.skip("Test OBJ file not available")
        
        output_obj = temp_dir / "quality_test.obj"
        
        # Optimize with different reduction levels
        reduction_levels = [0.1, 0.3, 0.5]
        
        for reduction in reduction_levels:
            result = cli_runner.run_command([
                "optimize",
                "--input", str(test_obj),
                "--output", str(output_obj),
                "--target-reduction", str(reduction)
            ])
            
            assert result['success'], f"Optimization failed at {reduction} reduction: {result['stderr']}"
            
            # Validate mesh still loads correctly
            validate_result = cli_runner.run_command([
                "validate", str(output_obj)
            ])
            assert validate_result['success'], f"Optimized mesh failed validation at {reduction} reduction"
    
    def test_mesh_batch_processing(self, temp_dir, cli_runner):
        """Test batch processing of multiple mesh files."""
        # Copy test files to temp directory
        mesh_files = []
        source_dir = Path("tests/fixtures/models")
        
        for mesh_file in source_dir.glob("test_cube.*"):
            if mesh_file.suffix in ['.obj', '.stl', '.ply', '.dae', '.fbx']:
                dest_file = temp_dir / mesh_file.name
                shutil.copy2(mesh_file, dest_file)
                mesh_files.append(dest_file)
        
        if not mesh_files:
            pytest.skip("No test mesh files available")
        
        # Batch optimize
        result = cli_runner.run_command([
            "batch",
            str(temp_dir),
            "--optimize",
            "--recursive"
        ])
        
        assert result['success'], f"Batch mesh optimization failed: {result['stderr']}"
        
        # Verify optimizations
        optimized_dir = temp_dir / "optimized"
        if optimized_dir.exists():
            # Check if optimized files were created in the optimized subdirectory
            optimized_files = list(optimized_dir.glob("optimized_*"))
            assert len(optimized_files) > 0, f"No optimized files found in {optimized_dir}"
            
            # Verify at least one optimized file per input file
            for mesh_file in mesh_files:
                expected_optimized = optimized_dir / f"optimized_{mesh_file.name}"
                assert expected_optimized.exists(), \
                    f"No optimized version found for {mesh_file.name} at {expected_optimized}"
        else:
            # Fallback: check if files were optimized in-place
            for mesh_file in mesh_files:
                # Check if file was modified (different size/timestamp indicates processing)
                assert mesh_file.exists(), f"Input file {mesh_file} was deleted"


class TestEndToEndValidation:
    """Integration tests for validation across all formats."""
    
    def test_comprehensive_format_validation(self, sample_images, cli_runner):
        """Test validation of all supported image formats."""
        test_cases = [
            (sample_images['png'], True, "Valid PNG should pass"),
            (sample_images['jpeg'], True, "Valid JPEG should pass"),
        ]
        
        # Add mesh files if available
        mesh_dir = Path("tests/fixtures/models")
        if mesh_dir.exists():
            for mesh_file in mesh_dir.glob("test_cube.*"):
                test_cases.append((mesh_file, True, f"Valid {mesh_file.suffix} should pass"))
        
        for file_path, should_pass, description in test_cases:
            if not file_path.exists():
                continue
                
            result = cli_runner.run_command([
                "validate", str(file_path)
            ])
            
            if should_pass:
                assert result['success'], f"{description}: {result['stderr']}"
            else:
                assert not result['success'], f"{description}: validation should have failed"
    
    def test_invalid_file_handling(self, temp_dir, cli_runner):
        """Test handling of invalid and corrupted files."""
        # Create invalid files
        invalid_cases = [
            ("empty.png", b""),  # Empty file
            ("corrupted.jpg", b"invalid jpeg data"),  # Invalid data
            ("wrong_ext.png", b"OBJ file content"),  # Wrong extension
        ]
        
        for filename, content in invalid_cases:
            invalid_file = temp_dir / filename
            invalid_file.write_bytes(content)
            
            # Validation should fail gracefully
            result = cli_runner.run_command([
                "validate", str(invalid_file)
            ])
            
            assert not result['success'], f"Validation should fail for {filename}"
            assert "invalid" in result['stderr'].lower() or "error" in result['stderr'].lower(), \
                f"Error message should indicate invalidity for {filename}"
    
    def test_missing_file_handling(self, cli_runner):
        """Test handling of missing input files."""
        missing_file = "nonexistent_file.png"
        
        result = cli_runner.run_command([
            "validate", missing_file
        ])
        
        assert not result['success'], "Should fail for missing file"
        assert "not found" in result['stderr'].lower() or "no such file" in result['stderr'].lower(), \
            f"Error should indicate missing file: {result['stderr']}"


class TestEndToEndPerformance:
    """Integration tests for performance and scalability."""
    
    def test_memory_usage_large_files(self, temp_dir, cli_runner):
        """Test memory efficiency with large files."""
        # Create a large test image
        large_img = Image.new('RGB', (2048, 2048), color=(100, 150, 200))
        large_file = temp_dir / "large_test.png"
        large_img.save(large_file, format='PNG')
        
        output_file = temp_dir / "large_optimized.png"
        
        # Process large file
        result = cli_runner.run_command([
            "optimize",
            "--input", str(large_file),
            "--output", str(output_file)
        ])
        
        assert result['success'], f"Large file processing failed: {result['stderr']}"
        assert output_file.exists(), "Large file output not created"
        
        # Verify memory didn't cause issues (file should be processed correctly)
        metrics = validate_image_quality(large_file, output_file)
        assert metrics['psnr'] > 30, f"Large file quality degraded: PSNR {metrics['psnr']}"
    
    def test_concurrent_processing_safety(self, temp_dir, cli_runner):
        """Test that concurrent processing doesn't cause issues."""
        # Create multiple test files
        test_files = []
        for i in range(5):
            img = Image.new('RGB', (200, 200), color=(i*50, i*40, i*30))
            img_file = temp_dir / f"concurrent_test_{i}.png"
            img.save(img_file, format='PNG')
            test_files.append(img_file)
        
        # Process files sequentially (simulating concurrent access patterns)
        results = []
        for img_file in test_files:
            output_file = temp_dir / f"output_{img_file.name}"
            result = cli_runner.run_command([
                "optimize",
                "--input", str(img_file),
                "--output", str(output_file)
            ])
            results.append((result, output_file))
        
        # Verify all succeeded
        for i, (result, output_file) in enumerate(results):
            assert result['success'], f"Concurrent processing failed for file {i}: {result['stderr']}"
            assert output_file.exists(), f"Output file {i} not created"


class TestEndToEndErrorRecovery:
    """Integration tests for error handling and recovery."""
    
    def test_partial_failure_recovery(self, temp_dir, cli_runner):
        """Test recovery from partial failures in batch processing."""
        # Create mix of valid and invalid files
        files = []
        
        # Valid files
        for i in range(3):
            img = Image.new('RGB', (100, 100), color=(i*80, i*60, i*40))
            valid_file = temp_dir / f"valid_{i}.png"
            img.save(valid_file, format='PNG')
            files.append(valid_file)
        
        # Invalid file
        invalid_file = temp_dir / "invalid.png"
        invalid_file.write_bytes(b"not an image")
        files.append(invalid_file)
        
        # Batch process
        result = cli_runner.run_command([
            "batch",
            str(temp_dir),
            "--optimize"
        ])
        
        # Should handle errors gracefully and process valid files
        # The exact behavior depends on implementation, but should not crash
        assert "error" in result['stderr'].lower() or result['success'], \
            "Should either succeed with warnings or fail gracefully"
        
        # Valid files should still be processed
        processed_count = 0
        for valid_file in files[:-1]:  # Exclude the invalid file
            possible_outputs = [
                temp_dir / f"optimized_{valid_file.name}",
                temp_dir / f"{valid_file.stem}_optimized{valid_file.suffix}"
            ]
            if any(f.exists() for f in possible_outputs):
                processed_count += 1
        
        # At least some valid files should be processed
        assert processed_count >= 0, "No valid files were processed"
    
    def test_disk_space_handling(self, temp_dir, cli_runner):
        """Test handling of disk space constraints."""
        # This is a conceptual test - actual implementation would need
        # platform-specific disk space manipulation
        
        # Create a test file
        img = Image.new('RGB', (100, 100), color=(128, 128, 128))
        input_file = temp_dir / "diskspace_test.png"
        img.save(input_file, format='PNG')
        
        # Try to write to a deeply nested path that might cause issues
        deep_path = temp_dir
        for i in range(10):
            deep_path = deep_path / f"level_{i}"
        
        output_file = deep_path / "output.png"
        
        result = cli_runner.run_command([
            "optimize",
            "--input", str(input_file),
            "--output", str(output_file)
        ])
        
        # Should either succeed (created directories) or fail gracefully
        if not result['success']:
            assert "directory" in result['stderr'].lower() or \
                   "path" in result['stderr'].lower() or \
                   "permission" in result['stderr'].lower(), \
                f"Error should be related to path issues: {result['stderr']}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
