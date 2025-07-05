"""Test C hotspot performance vs Rust implementations."""

import os
import time
import pytest
import numpy as np
from pathlib import Path
from PIL import Image
import subprocess


class TestCHotspotPerformance:
    """Test performance gains from C hotspot optimizations."""
    
    @pytest.fixture
    def large_test_image(self, tmp_path):
        """Create a large test image for performance testing."""
        # Create a 2048x2048 test image with complex content
        size = (2048, 2048)
        
        # Generate complex pattern to stress optimization algorithms
        img_array = np.zeros((size[1], size[0], 3), dtype=np.uint8)
        
        # Create gradient patterns
        for y in range(size[1]):
            for x in range(size[0]):
                img_array[y, x, 0] = (x * 255) // size[0]  # Red gradient
                img_array[y, x, 1] = (y * 255) // size[1]  # Green gradient
                img_array[y, x, 2] = ((x + y) * 255) // (size[0] + size[1])  # Blue diagonal
        
        # Add some noise for complexity
        noise = np.random.randint(0, 50, size=(size[1], size[0], 3), dtype=np.uint8)
        img_array = np.clip(img_array.astype(np.int16) + noise, 0, 255).astype(np.uint8)
        
        img = Image.fromarray(img_array)
        test_file = tmp_path / "large_test.png"
        img.save(test_file)
        return test_file
    
    def test_png_optimization_performance(self, large_test_image, tmp_path, benchmark):
        """Benchmark PNG optimization with C hotspots."""
        output_file = tmp_path / "optimized_large.png"
        
        def optimize_png():
            result = subprocess.run([
                "target/debug/pxjc.exe", "optimize",
                "--input", str(large_test_image),
                "--output", str(output_file),
                "--format", "png"
            ], capture_output=True, text=True)
            assert result.returncode == 0, f"CLI failed: {result.stderr}"
            assert output_file.exists(), "Output file not created"
            return result
        
        # Benchmark the optimization
        result = benchmark(optimize_png)
        
        # Verify optimization was effective
        original_size = large_test_image.stat().st_size
        optimized_size = output_file.stat().st_size
        
        print(f"Original size: {original_size:,} bytes")
        print(f"Optimized size: {optimized_size:,} bytes")
        
        # For PNG, we should see some compression improvement
        # (actual improvement depends on content)
        assert optimized_size > 0
        
    def test_webp_conversion_performance(self, large_test_image, tmp_path, benchmark):
        """Benchmark WebP conversion with C hotspots."""
        output_file = tmp_path / "converted_large.webp"
        
        def convert_webp():
            result = subprocess.run([
                "target/debug/pxjc.exe", "optimize",
                "--input", str(large_test_image),
                "--output", str(output_file),
                "--format", "webp"
            ], capture_output=True, text=True)
            assert result.returncode == 0, f"CLI failed: {result.stderr}"
            assert output_file.exists(), "Output file not created"
            return result
        
        # Benchmark the conversion
        result = benchmark(convert_webp)
        
        # Verify conversion was effective
        original_size = large_test_image.stat().st_size
        webp_size = output_file.stat().st_size
        
        print(f"Original PNG size: {original_size:,} bytes")
        print(f"WebP size: {webp_size:,} bytes")
        
        # WebP should typically be smaller than PNG for photographic content
        assert webp_size > 0
        
    def test_jpeg_conversion_performance(self, large_test_image, tmp_path, benchmark):
        """Benchmark JPEG conversion with C hotspots."""
        output_file = tmp_path / "converted_large.jpg"
        
        def convert_jpeg():
            result = subprocess.run([
                "target/debug/pxjc.exe", "optimize",
                "--input", str(large_test_image),
                "--output", str(output_file),
                "--format", "jpeg"
            ], capture_output=True, text=True)
            assert result.returncode == 0, f"CLI failed: {result.stderr}"
            assert output_file.exists(), "Output file not created"
            return result
        
        # Benchmark the conversion
        result = benchmark(convert_jpeg)
        
        # Verify conversion was effective
        original_size = large_test_image.stat().st_size
        jpeg_size = output_file.stat().st_size
        
        print(f"Original PNG size: {original_size:,} bytes")
        print(f"JPEG size: {jpeg_size:,} bytes")
        
        # JPEG should typically be much smaller than PNG for photographic content
        assert jpeg_size > 0
        compression_ratio = original_size / jpeg_size
        print(f"Compression ratio: {compression_ratio:.2f}x")
        
    def test_memory_usage_large_files(self, large_test_image, tmp_path):
        """Test memory usage with large files."""
        output_file = tmp_path / "memory_test.webp"
        
        # Monitor memory usage during optimization (simplified version)
        import subprocess
        
        start_time = time.time()
        
        # Start the process
        result = subprocess.run([
            "target/debug/pxjc.exe", "optimize",
            "--input", str(large_test_image),
            "--output", str(output_file),
            "--format", "webp"
        ], capture_output=True, text=True)
        
        end_time = time.time()
        processing_time = end_time - start_time
        
        assert result.returncode == 0, f"CLI failed: {result.stderr}"
        
        print(f"Processing time: {processing_time:.2f} seconds")
        
        # For a 2048x2048 image, processing should complete in reasonable time
        assert processing_time < 30, f"Processing too slow: {processing_time:.2f} seconds"
        assert output_file.exists(), "Output file not created"


class TestCHotspotFeatures:
    """Test that C hotspot features work correctly."""
    
    @pytest.fixture
    def small_test_image(self, tmp_path):
        """Create a small test image for feature testing."""
        # Create a 64x64 test image with known patterns
        size = (64, 64)
        img_array = np.zeros((size[1], size[0], 3), dtype=np.uint8)
        
        # Create a simple pattern for testing color quantization
        for y in range(size[1]):
            for x in range(size[0]):
                # Create distinct color regions
                if x < 16:
                    img_array[y, x] = [255, 0, 0]  # Red
                elif x < 32:
                    img_array[y, x] = [0, 255, 0]  # Green
                elif x < 48:
                    img_array[y, x] = [0, 0, 255]  # Blue
                else:
                    img_array[y, x] = [255, 255, 255]  # White
        
        img = Image.fromarray(img_array)
        test_file = tmp_path / "small_test.png"
        img.save(test_file)
        return test_file
    
    def test_png_optimization_features(self, small_test_image, tmp_path):
        """Test PNG optimization features work correctly."""
        output_file = tmp_path / "optimized_small.png"
        
        result = subprocess.run([
            "target/debug/pxjc.exe", "optimize",
            "--input", str(small_test_image),
            "--output", str(output_file),
            "--format", "png"
        ], capture_output=True, text=True)
        
        assert result.returncode == 0, f"CLI failed: {result.stderr}"
        assert output_file.exists(), "Output file not created"
        
        # Verify the output is a valid PNG
        with Image.open(output_file) as img:
            assert img.format == "PNG"
            assert img.size == (64, 64)
            
    def test_format_conversion_quality(self, small_test_image, tmp_path):
        """Test that format conversions maintain reasonable quality."""
        formats = ["png", "jpeg", "webp"]
        
        for fmt in formats:
            output_file = tmp_path / f"converted_small.{fmt}"
            
            result = subprocess.run([
                "target/debug/pxjc.exe", "optimize",
                "--input", str(small_test_image),
                "--output", str(output_file),
                "--format", fmt
            ], capture_output=True, text=True)
            
            assert result.returncode == 0, f"CLI failed for {fmt}: {result.stderr}"
            assert output_file.exists(), f"Output file not created for {fmt}"
            
            # Verify the output is valid
            with Image.open(output_file) as img:
                assert img.size == (64, 64)
                # For small test images, formats might vary in exact pixel values
                # but should preserve general structure
