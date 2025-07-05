#!/usr/bin/env python3

"""
Final validation test for Pixie Juice aggressive optimization improvements.
Tests that all optimization modes produce significant size reduction (≥40%).
"""

import os
import sys
import subprocess
import tempfile
from pathlib import Path
from PIL import Image
import shutil

def run_command(cmd, input_file=None):
    """Run a command and return success, output, error."""
    try:
        if input_file:
            with open(input_file, 'rb') as f:
                result = subprocess.run(cmd, input=f.read(), capture_output=True, shell=True)
        else:
            result = subprocess.run(cmd, capture_output=True, shell=True, text=True)
        return result.returncode == 0, result.stdout, result.stderr
    except Exception as e:
        return False, "", str(e)

def get_file_size(filepath):
    """Get file size in bytes."""
    return os.path.getsize(filepath) if os.path.exists(filepath) else 0

def create_test_image(filepath, format_type, size=(800, 600)):
    """Create a test image for optimization."""
    # Create a complex image with gradients and details to ensure compression opportunities
    img = Image.new('RGB', size, color='white')
    pixels = img.load()
    
    # Create a complex pattern with gradients and noise
    for x in range(size[0]):
        for y in range(size[1]):
            r = int(255 * (x / size[0]))
            g = int(255 * (y / size[1]))
            b = int(255 * ((x + y) % 256) / 255)
            pixels[x, y] = (r, g, b)
    
    if format_type.upper() == 'PNG':
        img.save(filepath, 'PNG', optimize=False, compress_level=0)
    elif format_type.upper() == 'JPEG':
        img.save(filepath, 'JPEG', quality=100, optimize=False)
    elif format_type.upper() == 'WEBP':
        img.save(filepath, 'WEBP', quality=100, optimize=False)

def create_test_mesh(filepath):
    """Create a test OBJ mesh file with many vertices."""
    with open(filepath, 'w') as f:
        f.write("# Test mesh with many vertices for decimation\n")
        
        # Create a grid of vertices (100x100 = 10,000 vertices)
        for i in range(100):
            for j in range(100):
                x = i * 0.1
                y = j * 0.1
                z = 0.05 * (i + j)  # Add some variation
                f.write(f"v {x} {y} {z}\n")
        
        # Create faces (triangles)
        for i in range(99):
            for j in range(99):
                # Current vertex index (1-based for OBJ)
                v1 = i * 100 + j + 1
                v2 = v1 + 1
                v3 = v1 + 100
                v4 = v3 + 1
                
                # Two triangles per quad
                f.write(f"f {v1} {v2} {v3}\n")
                f.write(f"f {v2} {v4} {v3}\n")

def test_image_optimization(format_type):
    """Test image optimization for a specific format."""
    print(f"\n=== Testing {format_type} Optimization ===")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test image
        input_file = os.path.join(tmpdir, f"test.{format_type.lower()}")
        output_file = os.path.join(tmpdir, f"optimized.{format_type.lower()}")
        
        create_test_image(input_file, format_type)
        original_size = get_file_size(input_file)
        print(f"Original {format_type} size: {original_size} bytes")
        
        # Run optimization with aggressive settings
        cmd = [
            r"target\debug\pxjc.exe", "optimize",
            "--input", input_file,
            "--output", output_file,
            "--quality", "20",  # Low quality for aggressive compression
            "--target-reduction", "0.5"  # Target 50% reduction (0.0-1.0 range)
        ]
        
        success, stdout, stderr = run_command(cmd)
        
        if not success:
            print(f"❌ {format_type} optimization failed:")
            print(f"stdout: {stdout}")
            print(f"stderr: {stderr}")
            return False
        
        if os.path.exists(output_file):
            optimized_size = get_file_size(output_file)
            reduction = ((original_size - optimized_size) / original_size) * 100
            print(f"Optimized {format_type} size: {optimized_size} bytes")
            print(f"Size reduction: {reduction:.1f}%")
            
            if reduction >= 40:
                print(f"✅ {format_type} optimization successful (≥40% reduction)")
                return True
            else:
                print(f"❌ {format_type} optimization insufficient (<40% reduction)")
                return False
        else:
            print(f"❌ {format_type} output file not created")
            return False

def test_mesh_optimization():
    """Test mesh optimization."""
    print(f"\n=== Testing Mesh Optimization ===")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test mesh
        input_file = os.path.join(tmpdir, "test.obj")
        output_file = os.path.join(tmpdir, "optimized.obj")
        
        create_test_mesh(input_file)
        original_size = get_file_size(input_file)
        print(f"Original mesh size: {original_size} bytes")
        
        # Count original vertices and faces
        with open(input_file, 'r') as f:
            content = f.read()
            original_vertices = content.count('\nv ')
            original_faces = content.count('\nf ')
        print(f"Original mesh: {original_vertices} vertices, {original_faces} faces")
        
        # Run mesh optimization with aggressive reduction
        cmd = [
            r"target\debug\pxjc.exe", "optimize",
            "--input", input_file,
            "--output", output_file,
            "--target-reduction", "0.6",  # Target 60% reduction for mesh (0.0-1.0 range)
            "--reduce", "0.6"  # Triangle reduction ratio
        ]
        
        success, stdout, stderr = run_command(cmd)
        
        if not success:
            print(f"❌ Mesh optimization failed:")
            print(f"stdout: {stdout}")
            print(f"stderr: {stderr}")
            return False
        
        if os.path.exists(output_file):
            optimized_size = get_file_size(output_file)
            
            # Count optimized vertices and faces
            with open(output_file, 'r') as f:
                content = f.read()
                optimized_vertices = content.count('\nv ')
                optimized_faces = content.count('\nf ')
            
            vertex_reduction = ((original_vertices - optimized_vertices) / original_vertices) * 100
            face_reduction = ((original_faces - optimized_faces) / original_faces) * 100
            size_reduction = ((original_size - optimized_size) / original_size) * 100
            
            print(f"Optimized mesh: {optimized_vertices} vertices, {optimized_faces} faces")
            print(f"Vertex reduction: {vertex_reduction:.1f}%")
            print(f"Face reduction: {face_reduction:.1f}%")
            print(f"File size reduction: {size_reduction:.1f}%")
            
            if vertex_reduction >= 40 and face_reduction >= 40:
                print(f"✅ Mesh optimization successful (≥40% reduction in vertices and faces)")
                return True
            else:
                print(f"❌ Mesh optimization insufficient (<40% reduction)")
                return False
        else:
            print(f"❌ Mesh output file not created")
            return False

def main():
    """Run all validation tests."""
    print("🔍 Final Validation: Testing Aggressive Optimization Logic")
    print("=" * 60)
    
    # Change to project directory
    os.chdir(r"c:\code\pixie-juice")
    
    # Build the CLI first
    print("Building CLI...")
    if not os.path.exists(r"target\debug\pxjc.exe"):
        print("❌ CLI binary not found. Please build it first with 'cargo build --package cli'")
        return False
    
    print("✅ CLI binary found")
    
    # Test all formats
    results = []
    
    # Test image formats
    for format_type in ['PNG', 'JPEG', 'WEBP']:
        results.append(test_image_optimization(format_type))
    
    # Test mesh optimization
    results.append(test_mesh_optimization())
    
    # Summary
    print(f"\n{'=' * 60}")
    print("📊 FINAL VALIDATION SUMMARY")
    print(f"{'=' * 60}")
    
    passed = sum(results)
    total = len(results)
    
    formats = ['PNG', 'JPEG', 'WEBP', 'Mesh']
    for i, (format_name, result) in enumerate(zip(formats, results)):
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{format_name:8} optimization: {status}")
    
    print(f"\nOverall: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 ALL TESTS PASSED! Aggressive optimization is working correctly.")
        return True
    else:
        print("⚠️  Some tests failed. Optimization needs further improvement.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
