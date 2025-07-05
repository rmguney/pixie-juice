#!/usr/bin/env python3

"""
Test all supported formats to see what's actually working
"""

import os
import sys
import subprocess
import tempfile
from PIL import Image

def create_test_image(format_type):
    """Create test image for specific format"""
    img = Image.new('RGB', (200, 150), color='white')
    pixels = img.load()
    
    if pixels:
        for x in range(200):
            for y in range(150):
                r = int(255 * (x / 200))
                g = int(255 * (y / 150))
                b = int(255 * ((x + y) % 256) / 255)
                pixels[x, y] = (r, g, b)
    
    return img

def test_format(format_type, quality=30):
    """Test optimization for a specific format"""
    print(f"\n=== Testing {format_type.upper()} ===")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = os.path.join(tmpdir, f"test.{format_type.lower()}")
        output_file = os.path.join(tmpdir, f"optimized.{format_type.lower()}")
        
        try:
            img = create_test_image(format_type)
            
            # Save with format-specific settings
            if format_type.upper() == 'PNG':
                img.save(input_file, 'PNG', optimize=False, compress_level=0)
            elif format_type.upper() == 'JPEG':
                img.save(input_file, 'JPEG', quality=100, optimize=False)
            elif format_type.upper() == 'WEBP':
                img.save(input_file, 'WEBP', quality=100, optimize=False)
            elif format_type.upper() == 'GIF':
                img.save(input_file, 'GIF', optimize=False)
            elif format_type.upper() == 'BMP':
                img.save(input_file, 'BMP')
            elif format_type.upper() == 'TIFF':
                img.save(input_file, 'TIFF', compression='raw')
            else:
                print(f"❓ Unknown image format: {format_type}")
                return False
                
        except Exception as e:
            print(f"❌ Could not create {format_type} test file: {e}")
            return False
        
        if not os.path.exists(input_file):
            print(f"❌ Test file not created for {format_type}")
            return False
            
        original_size = os.path.getsize(input_file)
        print(f"Original size: {original_size} bytes")
        
        # Test CLI optimization
        cmd = [
            r"target\debug\pxjc.exe", "optimize",
            "--input", input_file,
            "--output", output_file,
            "--quality", str(quality),
            "--target-reduction", "0.5"
        ]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
            
            if result.returncode == 0:
                if os.path.exists(output_file):
                    optimized_size = os.path.getsize(output_file)
                    reduction = ((original_size - optimized_size) / original_size) * 100
                    
                    print(f"Optimized size: {optimized_size} bytes")
                    print(f"Reduction: {reduction:.1f}%")
                    
                    if reduction >= 10:  # Lower threshold for initial testing
                        print(f"✅ {format_type.upper()} WORKING")
                        return True
                    else:
                        print(f"⚠️  {format_type.upper()} MINIMAL REDUCTION")
                        return False
                else:
                    print(f"❌ {format_type.upper()} NO OUTPUT FILE")
                    return False
            else:
                print(f"❌ {format_type.upper()} COMMAND FAILED")
                if result.stderr.strip():
                    print(f"Error: {result.stderr.strip()}")
                return False
                
        except subprocess.TimeoutExpired:
            print(f"❌ {format_type.upper()} TIMEOUT")
            return False
        except Exception as e:
            print(f"❌ {format_type.upper()} EXCEPTION: {e}")
            return False

def create_test_mesh(format_type):
    """Create a simple test mesh"""
    content = """# Simple test cube
v -1.0 -1.0  1.0
v  1.0 -1.0  1.0
v  1.0  1.0  1.0
v -1.0  1.0  1.0
v -1.0 -1.0 -1.0
v  1.0 -1.0 -1.0
v  1.0  1.0 -1.0
v -1.0  1.0 -1.0

f 1 2 3 4
f 2 6 7 3
f 6 5 8 7
f 5 1 4 8
f 4 3 7 8
f 1 5 6 2
"""
    return content

def test_mesh_format(format_type):
    """Test mesh format optimization"""
    print(f"\n=== Testing {format_type.upper()} (Mesh) ===")
    
    if format_type.upper() != 'OBJ':
        print(f"❓ Skipping {format_type} - only OBJ mesh creation implemented")
        return False
    
    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = os.path.join(tmpdir, f"test.{format_type.lower()}")
        output_file = os.path.join(tmpdir, f"optimized.{format_type.lower()}")
        
        # Create test mesh
        with open(input_file, 'w') as f:
            f.write(create_test_mesh(format_type))
        
        original_size = os.path.getsize(input_file)
        print(f"Original size: {original_size} bytes")
        
        cmd = [
            r"target\debug\pxjc.exe", "optimize",
            "--input", input_file,
            "--output", output_file,
            "--reduce", "0.3"
        ]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
            
            if result.returncode == 0 and os.path.exists(output_file):
                optimized_size = os.path.getsize(output_file)
                reduction = ((original_size - optimized_size) / original_size) * 100
                
                print(f"Optimized size: {optimized_size} bytes")
                print(f"Reduction: {reduction:.1f}%")
                
                print(f"✅ {format_type.upper()} WORKING")
                return True
            else:
                print(f"❌ {format_type.upper()} FAILED")
                if result.stderr.strip():
                    print(f"Error: {result.stderr.strip()}")
                return False
                
        except Exception as e:
            print(f"❌ {format_type.upper()} EXCEPTION: {e}")
            return False

def main():
    os.chdir(r"c:\code\pixie-juice")
    
    print("🔍 COMPREHENSIVE FORMAT SUPPORT TEST")
    print("=" * 60)
    
    # Test image formats
    image_formats = ['PNG', 'JPEG', 'WEBP', 'GIF', 'BMP', 'TIFF']
    image_results = []
    
    for fmt in image_formats:
        image_results.append((fmt, test_format(fmt)))
    
    # Test mesh formats  
    mesh_formats = ['OBJ']  # Start with OBJ, add others if working
    mesh_results = []
    
    for fmt in mesh_formats:
        mesh_results.append((fmt, test_mesh_format(fmt)))
    
    # Summary
    print(f"\n{'=' * 60}")
    print("📊 FORMAT SUPPORT SUMMARY")
    print(f"{'=' * 60}")
    
    print("\n🖼️  IMAGE FORMATS:")
    for fmt, result in image_results:
        status = "✅ WORKING" if result else "❌ BROKEN"
        print(f"  {fmt:6}: {status}")
    
    print("\n🎲 MESH FORMATS:")
    for fmt, result in mesh_results:
        status = "✅ WORKING" if result else "❌ BROKEN"
        print(f"  {fmt:6}: {status}")
    
    total_working = sum(r for _, r in image_results + mesh_results)
    total_formats = len(image_results + mesh_results)
    
    print(f"\nTotal working: {total_working}/{total_formats}")
    
    if total_working < total_formats:
        print("⚠️  Some formats need implementation!")
        return False
    else:
        print("🎉 All tested formats working!")
        return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
