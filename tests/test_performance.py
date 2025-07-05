#!/usr/bin/env python3
"""
Simple performance test for C hotspots
"""
import subprocess
import time
import tempfile
from pathlib import Path
from PIL import Image
import numpy as np

def create_large_test_image(path: Path, size=(1024, 1024)):
    """Create a large complex test image"""
    print(f"Creating large test image {size[0]}x{size[1]}...")
    
    # Create complex pattern
    img_array = np.zeros((size[1], size[0], 3), dtype=np.uint8)
    
    # Generate complex gradient + noise pattern
    for y in range(size[1]):
        for x in range(size[0]):
            img_array[y, x, 0] = (x * 255) // size[0]  # Red gradient
            img_array[y, x, 1] = (y * 255) // size[1]  # Green gradient  
            img_array[y, x, 2] = ((x + y) * 255) // (size[0] + size[1])  # Blue diagonal
    
    # Add noise for complexity
    noise = np.random.randint(0, 30, size=(size[1], size[0], 3), dtype=np.uint8)
    img_array = np.clip(img_array.astype(np.int16) + noise, 0, 255).astype(np.uint8)
    
    img = Image.fromarray(img_array)
    img.save(path)
    print(f"Created {path} ({path.stat().st_size / 1024:.1f} KB)")
    return path

def test_optimization_performance():
    """Test optimization performance with timing"""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)
        
        # Create large test image
        input_file = tmpdir / "large_test.png"
        output_file = tmpdir / "optimized.png"
        create_large_test_image(input_file)
        
        # Test PNG optimization performance
        cli_path = Path("target/release/pxjc.exe")
        cmd = [
            str(cli_path), "optimize",
            "--input", str(input_file),
            "--output", str(output_file),
            "--format", "png",
            "--lossless"
        ]
        
        print(f"\n=== Running PNG Optimization Performance Test ===")
        print(f"Command: {' '.join(cmd)}")
        
        start_time = time.time()
        result = subprocess.run(cmd, capture_output=True, text=True)
        end_time = time.time()
        
        duration = end_time - start_time
        
        print(f"Optimization completed in {duration:.2f} seconds")
        print(f"Return code: {result.returncode}")
        
        if result.stdout:
            print(f"STDOUT: {result.stdout}")
        if result.stderr:
            print(f"STDERR: {result.stderr}")
        
        if result.returncode == 0 and output_file.exists():
            original_size = input_file.stat().st_size
            optimized_size = output_file.stat().st_size
            compression_ratio = (original_size - optimized_size) / original_size * 100
            
            print(f"✅ Success!")
            print(f"Original size: {original_size / 1024:.1f} KB")
            print(f"Optimized size: {optimized_size / 1024:.1f} KB") 
            print(f"Compression: {compression_ratio:.1f}%")
            print(f"Performance: {original_size / 1024 / duration:.1f} KB/s")
            
            return True
        else:
            print(f"❌ Optimization failed")
            return False

if __name__ == "__main__":
    print("=== C Hotspot Performance Test ===")
    success = test_optimization_performance()
    if success:
        print("🎉 Performance test passed!")
    else:
        print("💥 Performance test failed!")
