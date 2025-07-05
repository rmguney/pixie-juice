"""
Test alpha channel preservation in PNG optimization.
"""

import pytest
import numpy as np
from PIL import Image
from pathlib import Path


def create_test_png_with_alpha(size=(100, 100)):
    """Create a test PNG image with alpha channel."""
    # Create RGBA image with varying alpha values
    img = Image.new('RGBA', size, (255, 255, 255, 0))  # Transparent white background
    
    # Add some colored areas with different alpha values
    for x in range(size[0]):
        for y in range(size[1]):
            if x < size[0] // 3:
                # Red area with full opacity
                img.putpixel((x, y), (255, 0, 0, 255))
            elif x < 2 * size[0] // 3:
                # Green area with 50% opacity
                img.putpixel((x, y), (0, 255, 0, 128))
            else:
                # Blue area with varying opacity based on y position
                alpha = int(255 * y / size[1])
                img.putpixel((x, y), (0, 0, 255, alpha))
    
    return img


def test_png_alpha_preservation_cli(temp_dir, cli_runner):
    """Test that PNG optimization preserves alpha channels via CLI."""
    # Create test image with alpha
    test_img = create_test_png_with_alpha()
    input_file = temp_dir / "test_alpha.png"
    output_file = temp_dir / "test_alpha_optimized.png"
    
    # Save test image
    test_img.save(input_file)
    
    # Get original alpha channel
    original_alpha = np.array(test_img)[:, :, 3]  # Alpha channel
    
    # Optimize via CLI
    result = cli_runner.run_command([
        "optimize",
        "--input", str(input_file),
        "--output", str(output_file),
        "--lossless"
    ])
    
    assert result['success'], f"CLI failed: {result['stderr']}"
    assert output_file.exists(), "Output file was not created"
    
    # Load optimized image and check alpha channel
    optimized_img = Image.open(output_file)
    assert optimized_img.mode == 'RGBA', f"Expected RGBA mode, got {optimized_img.mode}"
    
    optimized_alpha = np.array(optimized_img)[:, :, 3]  # Alpha channel
    
    # Alpha channels should be identical for lossless optimization
    np.testing.assert_array_equal(original_alpha, optimized_alpha, 
                                  "Alpha channel was modified during lossless optimization")


def test_png_alpha_preservation_precise_values(temp_dir, cli_runner):
    """Test that specific alpha values are preserved exactly."""
    # Create image with specific alpha values we want to test
    img = Image.new('RGBA', (10, 10), (0, 0, 0, 0))
    
    # Set specific alpha values that should be preserved
    test_alphas = [0, 1, 63, 127, 191, 254, 255]
    for i, alpha in enumerate(test_alphas):
        if i < 10:
            img.putpixel((i, 0), (255, 255, 255, alpha))
    
    input_file = temp_dir / "alpha_precise.png"
    output_file = temp_dir / "alpha_precise_optimized.png"
    
    img.save(input_file)
    
    # Optimize
    result = cli_runner.run_command([
        "optimize",
        "--input", str(input_file),
        "--output", str(output_file),
        "--lossless"
    ])
    
    assert result['success'], f"CLI failed: {result['stderr']}"
    
    # Check that specific alpha values are preserved
    optimized_img = Image.open(output_file)
    for i, expected_alpha in enumerate(test_alphas):
        if i < 10:
            pixel = optimized_img.getpixel((i, 0))
            if isinstance(pixel, tuple) and len(pixel) >= 4:
                actual_alpha = pixel[3]
                assert actual_alpha == expected_alpha, \
                    f"Alpha value at position {i} changed from {expected_alpha} to {actual_alpha}"


def test_png_alpha_metadata_interaction(temp_dir, cli_runner):
    """Test that alpha preservation works correctly with metadata options."""
    test_img = create_test_png_with_alpha()
    input_file = temp_dir / "test_alpha_meta.png"
    
    # Save with some metadata (via PIL this is limited, but we test the flag)
    test_img.save(input_file)
    original_alpha = np.array(test_img)[:, :, 3]
    
    # Test with metadata preservation
    output_with_meta = temp_dir / "alpha_with_meta.png"
    result = cli_runner.run_command([
        "optimize",
        "--input", str(input_file),
        "--output", str(output_with_meta),
        "--preserve-image-metadata",
        "--lossless"
    ])
    
    assert result['success'], f"CLI failed: {result['stderr']}"
    
    # Test without metadata preservation
    output_no_meta = temp_dir / "alpha_no_meta.png"
    result = cli_runner.run_command([
        "optimize",
        "--input", str(input_file),
        "--output", str(output_no_meta),
        "--lossless"
    ])
    
    assert result['success'], f"CLI failed: {result['stderr']}"
    
    # Both should preserve alpha channels regardless of metadata setting
    img_with_meta = Image.open(output_with_meta)
    img_no_meta = Image.open(output_no_meta)
    
    alpha_with_meta = np.array(img_with_meta)[:, :, 3]
    alpha_no_meta = np.array(img_no_meta)[:, :, 3]
    
    np.testing.assert_array_equal(original_alpha, alpha_with_meta,
                                  "Alpha channel changed with metadata preservation")
    np.testing.assert_array_equal(original_alpha, alpha_no_meta,
                                  "Alpha channel changed without metadata preservation")


def test_png_size_reduction_with_alpha(temp_dir, cli_runner):
    """Test that PNG optimization actually reduces file size while preserving alpha."""
    test_img = create_test_png_with_alpha(size=(200, 200))  # Larger image for better compression
    input_file = temp_dir / "large_alpha.png"
    output_file = temp_dir / "large_alpha_optimized.png"
    
    # Save with minimal compression to ensure room for optimization
    test_img.save(input_file, compress_level=0)
    
    original_size = input_file.stat().st_size
    
    # Optimize
    result = cli_runner.run_command([
        "optimize",
        "--input", str(input_file),
        "--output", str(output_file),
        "--lossless"
    ])
    
    assert result['success'], f"CLI failed: {result['stderr']}"
    
    optimized_size = output_file.stat().st_size
    
    # Should reduce size while preserving alpha
    assert optimized_size < original_size, \
        f"PNG optimization should reduce size: {original_size} -> {optimized_size}"
    
    # Verify alpha is still preserved
    original_alpha = np.array(test_img)[:, :, 3]
    optimized_img = Image.open(output_file)
    optimized_alpha = np.array(optimized_img)[:, :, 3]
    
    np.testing.assert_array_equal(original_alpha, optimized_alpha,
                                  "Alpha channel was modified during size optimization")
