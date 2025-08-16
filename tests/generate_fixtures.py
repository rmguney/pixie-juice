#!/usr/bin/env python3
"""
Comprehensive test fixture generator for Pixie Juice
Creates all required test files for both image and mesh optimization testing
"""

import os
import struct
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
from PIL.Image import Resampling
import numpy as np

# Configuration
FIXTURES_DIR = Path("fixtures")
IMAGES_DIR = FIXTURES_DIR / "images"
MESHES_DIR = FIXTURES_DIR / "meshes"

# Size definitions
SIZE_CONFIGS = {
    'tiny': (32, 32),
    'small': (128, 128),
    'medium': (512, 512),
    'large': (1024, 1024),
    'ultra': (2048, 2048)
}

# Quality settings for lossy formats
QUALITY_CONFIGS = {
    'low': 30,
    'medium': 75,
    'high': 95,
    'lossless': 100
}

class ImageFixtureGenerator:
    """Generates comprehensive image test fixtures"""
    
    def __init__(self):
        self.base_colors = [
            (255, 0, 0),    # Red
            (0, 255, 0),    # Green
            (0, 0, 255),    # Blue
            (255, 255, 0),  # Yellow
            (255, 0, 255),  # Magenta
            (0, 255, 255),  # Cyan
        ]
    
    def create_test_image(self, width, height, pattern='gradient', format_name='TEST'):
        """Create a test image with specified pattern"""
        img = Image.new('RGB', (width, height), 'white')
        draw = ImageDraw.Draw(img)
        
        if pattern == 'gradient':
            # Create gradient pattern
            for x in range(width):
                for y in range(height):
                    r = int(255 * x / max(width - 1, 1))
                    g = int(255 * y / max(height - 1, 1))
                    b = int(255 * (x + y) / max(width + height - 2, 1))
                    img.putpixel((x, y), (r, g, b))
        
        elif pattern == 'checkerboard':
            # Create checkerboard pattern
            square_size = max(8, min(width, height) // 16)
            for x in range(0, width, square_size):
                for y in range(0, height, square_size):
                    color = 'black' if (x // square_size + y // square_size) % 2 else 'white'
                    draw.rectangle([x, y, x + square_size, y + square_size], fill=color)
        
        elif pattern == 'geometric':
            # Create geometric shapes
            colors = self.base_colors
            num_shapes = min(10, max(3, (width * height) // 10000))
            
            for i in range(num_shapes):
                color = colors[i % len(colors)]
                x1, y1 = np.random.randint(0, width), np.random.randint(0, height)
                x2, y2 = np.random.randint(x1, width), np.random.randint(y1, height)
                
                if i % 3 == 0:
                    draw.rectangle([x1, y1, x2, y2], fill=color)
                elif i % 3 == 1:
                    draw.ellipse([x1, y1, x2, y2], fill=color)
                else:
                    draw.polygon([(x1, y1), (x2, y1), ((x1+x2)//2, y2)], fill=color)
        
        # Add format label
        try:
            # Try to add text (may fail if no font available)
            text = f"{format_name}\\n{width}x{height}"
            draw.text((10, 10), text, fill='black')
        except:
            pass  # Skip text if font not available
        
        return img
    
    def create_png_fixtures(self):
        """Create PNG test fixtures"""
        png_dir = IMAGES_DIR / 'png'
        png_dir.mkdir(parents=True, exist_ok=True)
        
        # Standard PNG files
        for size_name, (width, height) in SIZE_CONFIGS.items():
            if size_name == 'tiny':  # Skip tiny for PNG to reduce file count
                continue
                
            # Regular PNG
            img = self.create_test_image(width, height, 'gradient', 'PNG')
            img.save(png_dir / f'{size_name}_png.png', 'PNG', optimize=True)
            
            # PNG with transparency
            img_rgba = img.convert('RGBA')
            # Make a circular transparent area
            center_x, center_y = width // 2, height // 2
            radius = min(width, height) // 4
            
            for x in range(width):
                for y in range(height):
                    dist = ((x - center_x) ** 2 + (y - center_y) ** 2) ** 0.5
                    if dist < radius:
                        try:
                            pixel = img_rgba.getpixel((x, y))
                            if isinstance(pixel, tuple) and len(pixel) >= 3:
                                if len(pixel) == 4:
                                    r, g, b, a = pixel
                                else:
                                    r, g, b = pixel[:3]
                                    a = 255
                                img_rgba.putpixel((x, y), (r, g, b, int(255 * (dist / radius))))
                        except:
                            pass  # Skip problematic pixels
            
            img_rgba.save(png_dir / f'{size_name}_png_transparent.png', 'PNG')
            
            # Lossless PNG (same as regular for PNG)
            img.save(png_dir / f'{size_name}_png_lossless.png', 'PNG', optimize=True)
        
        print(f"SUCCESS: Created PNG fixtures in {png_dir}")
    
    def create_jpeg_fixtures(self):
        """Create JPEG test fixtures"""
        jpeg_dir = IMAGES_DIR / 'jpeg'
        jpeg_dir.mkdir(parents=True, exist_ok=True)
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            if size_name == 'tiny':
                continue
                
            img = self.create_test_image(width, height, 'geometric', 'JPEG')
            
            # Different quality levels
            for quality_name, quality_val in QUALITY_CONFIGS.items():
                if quality_name == 'lossless':  # JPEG doesn't have true lossless
                    continue
                    
                filename = f'{size_name}_jpeg_{quality_name}.jpg'
                img.save(jpeg_dir / filename, 'JPEG', quality=quality_val, optimize=True)
            
            # Standard JPEG (medium quality)
            img.save(jpeg_dir / f'{size_name}_jpeg.jpg', 'JPEG', quality=75, optimize=True)
        
        print(f"SUCCESS: Created JPEG fixtures in {jpeg_dir}")
    
    def create_webp_fixtures(self):
        """Create WebP test fixtures"""
        webp_dir = IMAGES_DIR / 'webp'
        webp_dir.mkdir(parents=True, exist_ok=True)
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            img = self.create_test_image(width, height, 'checkerboard', 'WebP')
            
            # Different quality levels
            for quality_name, quality_val in QUALITY_CONFIGS.items():
                if quality_name == 'lossless':
                    filename = f'{size_name}_webp_lossless.webp'
                    img.save(webp_dir / filename, 'WebP', lossless=True)
                else:
                    filename = f'{size_name}_webp_{quality_name}.webp'
                    img.save(webp_dir / filename, 'WebP', quality=quality_val)
            
            # Standard WebP  
            img.save(webp_dir / f'{size_name}_webp.webp', 'WebP', quality=80)
        
        print(f"SUCCESS: Created WebP fixtures in {webp_dir}")
    
    def create_gif_fixtures(self):
        """Create GIF test fixtures"""
        gif_dir = IMAGES_DIR / 'gif'
        gif_dir.mkdir(parents=True, exist_ok=True)
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            if size_name in ['ultra', 'large']:  # Skip large GIFs
                continue
                
            # Static GIF
            img = self.create_test_image(width, height, 'geometric', 'GIF')
            img_p = img.convert('P', colors=256)
            img_p.save(gif_dir / f'{size_name}_gif.gif', 'GIF', optimize=True)
            
            # Animated GIF
            frames = []
            for i in range(8):  # 8 frame animation
                frame_img = self.create_test_image(width, height, 'gradient', f'GIF-F{i}')
                # Rotate hue for animation effect
                frame_array = np.array(frame_img)
                frame_array = np.roll(frame_array, i * 10, axis=2)  # Simple color shift
                frame = Image.fromarray(frame_array)
                frames.append(frame.convert('P', colors=256))
            
            frames[0].save(
                gif_dir / f'{size_name}_gif_animated.gif',
                'GIF',
                save_all=True,
                append_images=frames[1:],
                duration=200,  # 200ms per frame
                loop=0,
                optimize=True
            )
        
        print(f"SUCCESS: Created GIF fixtures in {gif_dir}")
    
    def create_bmp_fixtures(self):
        """Create BMP test fixtures"""
        bmp_dir = IMAGES_DIR / 'bmp'
        bmp_dir.mkdir(parents=True, exist_ok=True)
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            if size_name == 'ultra':  # Skip ultra for BMP
                continue
                
            img = self.create_test_image(width, height, 'geometric', 'BMP')
            img.save(bmp_dir / f'{size_name}_bmp.bmp', 'BMP')
        
        print(f"SUCCESS: Created BMP fixtures in {bmp_dir}")
    
    def create_tiff_fixtures(self):
        """Create TIFF test fixtures"""
        tiff_dir = IMAGES_DIR / 'tiff'
        tiff_dir.mkdir(parents=True, exist_ok=True)
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            img = self.create_test_image(width, height, 'gradient', 'TIFF')
            
            # Standard TIFF
            img.save(tiff_dir / f'{size_name}_tiff.tiff', 'TIFF')
            img.save(tiff_dir / f'{size_name}_tif.tif', 'TIFF')  # Alternative extension
            
            # Compressed TIFF
            img.save(tiff_dir / f'{size_name}_tiff_lzw.tiff', 'TIFF', compression='lzw')
        
        print(f"SUCCESS: Created TIFF fixtures in {tiff_dir}")
    
    def create_ico_fixtures(self):
        """Create ICO test fixtures"""
        ico_dir = IMAGES_DIR / 'ico'
        ico_dir.mkdir(parents=True, exist_ok=True)
        
        # ICO files are typically small
        ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128)]
        
        for width, height in ico_sizes:
            img = self.create_test_image(width, height, 'geometric', 'ICO')
            img.save(ico_dir / f'{width}x{height}.ico', 'ICO', sizes=[(width, height)])
        
        # Multi-size ICO
        images = []
        for width, height in ico_sizes[:4]:  # Use first 4 sizes
            img = self.create_test_image(width, height, 'geometric', 'ICO')
            images.append(img)
        
        images[0].save(
            ico_dir / 'multi_size.ico',
            'ICO',
            sizes=[(img.width, img.height) for img in images],
            append_images=images[1:]
        )
        
        print(f"SUCCESS: Created ICO fixtures in {ico_dir}")
    
    def create_svg_fixtures(self):
        """Create SVG test fixtures"""
        svg_dir = IMAGES_DIR / 'svg'
        svg_dir.mkdir(parents=True, exist_ok=True)
        
        svg_templates = {
            'simple': '''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
    <rect width="{width}" height="{height}" fill="#f0f0f0"/>
    <circle cx="{cx}" cy="{cy}" r="{r}" fill="#ff0000"/>
    <text x="10" y="30" font-family="Arial" font-size="16">SVG {width}x{height}</text>
</svg>''',
            
            'complex': '''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
    <defs>
        <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style="stop-color:rgb(255,255,0);stop-opacity:1" />
            <stop offset="100%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
        </linearGradient>
    </defs>
    <rect width="{width}" height="{height}" fill="url(#grad1)"/>
    <circle cx="{cx}" cy="{cy}" r="{r}" fill="#0000ff" opacity="0.7"/>
    <polygon points="{poly_points}" fill="#00ff00" opacity="0.5"/>
    <text x="10" y="30" font-family="Arial" font-size="20" fill="white">Complex SVG</text>
</svg>'''
        }
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            cx, cy = width // 2, height // 2
            r = min(width, height) // 4
            poly_points = f"{width//4},{height//4} {3*width//4},{height//4} {width//2},{3*height//4}"
            
            for template_name, template in svg_templates.items():
                svg_content = template.format(
                    width=width, height=height, cx=cx, cy=cy, r=r,
                    poly_points=poly_points
                )
                
                filename = f'{size_name}_{template_name}.svg'
                with open(svg_dir / filename, 'w', encoding='utf-8') as f:
                    f.write(svg_content)
        
        print(f"SUCCESS: Created SVG fixtures in {svg_dir}")
    
    def create_tga_fixtures(self):
        """Create TGA test fixtures"""
        tga_dir = IMAGES_DIR / 'tga'
        tga_dir.mkdir(parents=True, exist_ok=True)
        
        for size_name, (width, height) in SIZE_CONFIGS.items():
            if size_name == 'ultra':  # Skip ultra for TGA
                continue
                
            # Create different bit depth variants
            img = self.create_test_image(width, height, 'gradient', 'TGA')
            
            # 24-bit TGA (standard)
            self._create_tga_file(tga_dir / f'{size_name}_tga_24bit.tga', img, 24)
            
            # 32-bit TGA (with alpha)
            img_rgba = img.convert('RGBA')
            self._create_tga_file(tga_dir / f'{size_name}_tga_32bit.tga', img_rgba, 32)
            
            # 16-bit TGA (reduced color)
            self._create_tga_file(tga_dir / f'{size_name}_tga_16bit.tga', img, 16)
            
            # Standard TGA
            self._create_tga_file(tga_dir / f'{size_name}_tga.tga', img, 24)
        
        print(f"SUCCESS: Created TGA fixtures in {tga_dir}")
    
    def _create_tga_file(self, filepath, img, bpp):
        """Create a TGA file with specified bit depth"""
        width, height = img.size
        
        # TGA header
        header = struct.pack('<BBBHHBHHHHBB',
            0,      # ID length
            0,      # Color map type
            2,      # Image type (uncompressed RGB)
            0, 0,   # Color map start, length
            0,      # Color map entry size
            0, 0,   # X, Y origin
            width, height,  # Width, height
            bpp,    # Bits per pixel
            0       # Image descriptor
        )
        
        # Convert image data
        if bpp == 32:
            pixels = list(img.convert('RGBA').getdata())
            image_data = bytearray()
            for r, g, b, a in pixels:
                image_data.extend([b, g, r, a])  # TGA uses BGRA
        elif bpp == 16:
            pixels = list(img.convert('RGB').getdata())
            image_data = bytearray()
            for r, g, b in pixels:
                # Convert to 16-bit RGB (5-5-5-1 format)
                r16 = (r >> 3) & 0x1F
                g16 = (g >> 3) & 0x1F
                b16 = (b >> 3) & 0x1F
                pixel16 = (r16 << 10) | (g16 << 5) | b16
                image_data.extend(struct.pack('<H', pixel16))
        else:  # 24-bit
            pixels = list(img.convert('RGB').getdata())
            image_data = bytearray()
            for r, g, b in pixels:
                image_data.extend([b, g, r])  # TGA uses BGR
        
        with open(filepath, 'wb') as f:
            f.write(header)
            f.write(image_data)

class MeshFixtureGenerator:
    """Generates comprehensive mesh test fixtures"""
    
    def create_obj_fixtures(self):
        """Create OBJ test fixtures"""
        obj_dir = MESHES_DIR / 'obj'
        obj_dir.mkdir(parents=True, exist_ok=True)
        
        # Simple cube
        cube_obj = '''# Simple Cube
v -1.0 -1.0  1.0
v  1.0 -1.0  1.0
v  1.0  1.0  1.0
v -1.0  1.0  1.0
v -1.0 -1.0 -1.0
v  1.0 -1.0 -1.0
v  1.0  1.0 -1.0
v -1.0  1.0 -1.0

f 1 2 3 4
f 8 7 6 5
f 4 3 7 8
f 5 1 4 8
f 5 6 2 1
f 2 6 7 3
'''
        with open(obj_dir / 'simple_cube.obj', 'w') as f:
            f.write(cube_obj)
        
        # Medium complexity mesh
        self._create_obj_sphere(obj_dir / 'medium_sphere.obj', subdivisions=2)
        
        # Complex mesh
        self._create_obj_sphere(obj_dir / 'complex_sphere.obj', subdivisions=4)
        
        # Large mesh
        self._create_obj_grid(obj_dir / 'large_grid.obj', size=50)
        
        print(f"SUCCESS: Created OBJ fixtures in {obj_dir}")
    
    def _create_obj_sphere(self, filepath, subdivisions=2):
        """Create an OBJ sphere with specified subdivision level"""
        vertices = []
        faces = []
        
        # Create icosphere vertices
        phi = (1 + np.sqrt(5)) / 2  # Golden ratio
        
        # Initial icosahedron vertices
        initial_vertices = [
            (-1, phi, 0), (1, phi, 0), (-1, -phi, 0), (1, -phi, 0),
            (0, -1, phi), (0, 1, phi), (0, -1, -phi), (0, 1, -phi),
            (phi, 0, -1), (phi, 0, 1), (-phi, 0, -1), (-phi, 0, 1)
        ]
        
        # Normalize vertices to unit sphere
        for v in initial_vertices:
            length = np.sqrt(sum(x*x for x in v))
            vertices.append((v[0]/length, v[1]/length, v[2]/length))
        
        # Initial faces (icosahedron)
        initial_faces = [
            (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
            (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
            (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
            (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)
        ]
        
        faces = initial_faces.copy()
        
        # Subdivide (simplified version)
        for _ in range(subdivisions):
            # This is a simplified subdivision
            new_faces = []
            for face in faces[:min(len(faces), 100)]:  # Limit for performance
                # Add original face
                new_faces.append(face)
                # Could add more sophisticated subdivision here
            faces = new_faces
        
        # Write OBJ file
        with open(filepath, 'w') as f:
            f.write(f"# Sphere with {subdivisions} subdivisions\\n")
            for v in vertices:
                f.write(f"v {v[0]:.6f} {v[1]:.6f} {v[2]:.6f}\\n")
            for face in faces:
                f.write(f"f {face[0]+1} {face[1]+1} {face[2]+1}\\n")
    
    def _create_obj_grid(self, filepath, size=10):
        """Create a grid mesh OBJ file"""
        with open(filepath, 'w') as f:
            f.write(f"# Grid mesh {size}x{size}\\n")
            
            # Generate vertices
            for y in range(size + 1):
                for x in range(size + 1):
                    f.write(f"v {x - size/2:.2f} 0.0 {y - size/2:.2f}\\n")
            
            # Generate faces
            for y in range(size):
                for x in range(size):
                    # Vertex indices (1-based)
                    v1 = y * (size + 1) + x + 1
                    v2 = v1 + 1
                    v3 = v1 + size + 1
                    v4 = v3 + 1
                    
                    # Two triangles per quad
                    f.write(f"f {v1} {v2} {v3}\\n")
                    f.write(f"f {v2} {v4} {v3}\\n")
    
    def create_stl_fixtures(self):
        """Create STL test fixtures"""
        stl_dir = MESHES_DIR / 'stl'
        stl_dir.mkdir(parents=True, exist_ok=True)
        
        # ASCII STL
        ascii_stl = '''solid SimpleCube
  facet normal 0.0 0.0 1.0
    outer loop
      vertex -1.0 -1.0 1.0
      vertex 1.0 -1.0 1.0
      vertex 1.0 1.0 1.0
    endloop
  endfacet
  facet normal 0.0 0.0 1.0
    outer loop
      vertex -1.0 -1.0 1.0
      vertex 1.0 1.0 1.0
      vertex -1.0 1.0 1.0
    endloop
  endfacet
endsolid SimpleCube
'''
        with open(stl_dir / 'simple_ascii.stl', 'w') as f:
            f.write(ascii_stl)
        
        # Binary STL
        self._create_binary_stl(stl_dir / 'simple_binary.stl')
        self._create_binary_stl(stl_dir / 'medium_binary.stl', triangles=100)
        self._create_binary_stl(stl_dir / 'large_binary.stl', triangles=1000)
        
        print(f"SUCCESS: Created STL fixtures in {stl_dir}")
    
    def _create_binary_stl(self, filepath, triangles=12):
        """Create a binary STL file"""
        with open(filepath, 'wb') as f:
            # 80-byte header
            header = b'Binary STL created by test fixture generator' + b'\\x00' * 37
            f.write(header)
            
            # Triangle count
            f.write(struct.pack('<I', triangles))
            
            # Generate triangles (simple cube-like shapes)
            for i in range(triangles):
                # Normal vector
                normal = (0.0, 0.0, 1.0)
                f.write(struct.pack('<fff', *normal))
                
                # Three vertices
                angle = 2 * np.pi * i / triangles
                for j in range(3):
                    vertex_angle = angle + j * 2 * np.pi / 3
                    x = np.cos(vertex_angle)
                    y = np.sin(vertex_angle)
                    z = 0.0 if i % 2 == 0 else 1.0
                    f.write(struct.pack('<fff', x, y, z))
                
                # Attribute byte count
                f.write(struct.pack('<H', 0))
    
    def create_ply_fixtures(self):
        """Create PLY test fixtures"""
        ply_dir = MESHES_DIR / 'ply'
        ply_dir.mkdir(parents=True, exist_ok=True)
        
        # Simple PLY
        simple_ply = '''ply
format ascii 1.0
element vertex 4
property float x
property float y
property float z
element face 2
property list uchar int vertex_indices
end_header
-1.0 -1.0 0.0
1.0 -1.0 0.0
1.0 1.0 0.0
-1.0 1.0 0.0
3 0 1 2
3 0 2 3
'''
        with open(ply_dir / 'simple_quad.ply', 'w') as f:
            f.write(simple_ply)
        
        # Binary PLY
        self._create_binary_ply(ply_dir / 'medium_binary.ply', vertices=100)
        self._create_binary_ply(ply_dir / 'large_binary.ply', vertices=1000)
        
        print(f"SUCCESS: Created PLY fixtures in {ply_dir}")
    
    def _create_binary_ply(self, filepath, vertices=100):
        """Create a binary PLY file"""
        faces = vertices - 2  # Approximate triangle count for a mesh
        
        header = f'''ply
format binary_little_endian 1.0
element vertex {vertices}
property float x
property float y
property float z
element face {faces}
property list uchar int vertex_indices
end_header
'''.encode('ascii')
        
        with open(filepath, 'wb') as f:
            f.write(header)
            
            # Write vertices
            for i in range(vertices):
                angle = 2 * np.pi * i / vertices
                x = np.cos(angle)
                y = np.sin(angle)
                z = np.random.uniform(-1, 1)
                f.write(struct.pack('<fff', x, y, z))
            
            # Write faces (triangles)
            for i in range(faces):
                # Triangle with 3 vertices
                f.write(struct.pack('<B', 3))  # Vertex count
                v1, v2, v3 = i, (i + 1) % vertices, (i + 2) % vertices
                f.write(struct.pack('<III', v1, v2, v3))
    
    def create_gltf_fixtures(self):
        """Create glTF test fixtures"""
        gltf_dir = MESHES_DIR / 'gltf'
        gltf_dir.mkdir(parents=True, exist_ok=True)
        
        # Simple glTF
        simple_gltf = {
            "asset": {"version": "2.0", "generator": "test-fixture-generator"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"mesh": 0}],
            "meshes": [{
                "primitives": [{
                    "attributes": {"POSITION": 0},
                    "indices": 1
                }]
            }],
            "accessors": [
                {
                    "bufferView": 0,
                    "componentType": 5126,
                    "count": 3,
                    "type": "VEC3",
                    "max": [1.0, 1.0, 0.0],
                    "min": [-1.0, -1.0, 0.0]
                },
                {
                    "bufferView": 1,
                    "componentType": 5123,
                    "count": 3,
                    "type": "SCALAR"
                }
            ],
            "bufferViews": [
                {"buffer": 0, "byteOffset": 0, "byteLength": 36},
                {"buffer": 0, "byteOffset": 36, "byteLength": 6}
            ],
            "buffers": [{"byteLength": 42, "uri": "simple_triangle.bin"}]
        }
        
        with open(gltf_dir / 'simple_triangle.gltf', 'w') as f:
            json.dump(simple_gltf, f, indent=2)
        
        # Create corresponding binary data
        with open(gltf_dir / 'simple_triangle.bin', 'wb') as f:
            # Vertex positions (3 vertices)
            vertices = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]]
            for vertex in vertices:
                f.write(struct.pack('<fff', *vertex))
            
            # Indices
            indices = [0, 1, 2]
            for index in indices:
                f.write(struct.pack('<H', index))
        
        print(f"SUCCESS: Created glTF fixtures in {gltf_dir}")
    
    def create_glb_fixtures(self):
        """Create GLB (binary glTF) test fixtures"""
        glb_dir = MESHES_DIR / 'glb'
        glb_dir.mkdir(parents=True, exist_ok=True)
        
        # Create a simple GLB file
        # This is a minimal implementation
        gltf_json = {
            "asset": {"version": "2.0"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"mesh": 0}],
            "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3"}],
            "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 36}],
            "buffers": [{"byteLength": 36}]
        }
        
        json_str = json.dumps(gltf_json, separators=(',', ':'))
        json_bytes = json_str.encode('utf-8')
        
        # Pad JSON to 4-byte boundary
        while len(json_bytes) % 4 != 0:
            json_bytes += b' '
        
        # Binary data (3 vertices)
        binary_data = struct.pack('<fffffffff', 
            -1.0, -1.0, 0.0,  # Vertex 1
             1.0, -1.0, 0.0,  # Vertex 2
             0.0,  1.0, 0.0   # Vertex 3
        )
        
        # Pad binary data to 4-byte boundary
        while len(binary_data) % 4 != 0:
            binary_data += b'\\x00'
        
        with open(glb_dir / 'simple_triangle.glb', 'wb') as f:
            # GLB header
            f.write(b'glTF')  # Magic
            f.write(struct.pack('<I', 2))  # Version
            f.write(struct.pack('<I', 12 + 8 + len(json_bytes) + 8 + len(binary_data)))  # Total length
            
            # JSON chunk
            f.write(struct.pack('<I', len(json_bytes)))  # Chunk length
            f.write(b'JSON')  # Chunk type
            f.write(json_bytes)
            
            # Binary chunk
            f.write(struct.pack('<I', len(binary_data)))  # Chunk length
            f.write(b'BIN\\x00')  # Chunk type
            f.write(binary_data)
        
        print(f"SUCCESS: Created GLB fixtures in {glb_dir}")
    
    def create_fbx_fixtures(self):
        """Create FBX test fixtures (placeholder)"""
        fbx_dir = MESHES_DIR / 'fbx'
        fbx_dir.mkdir(parents=True, exist_ok=True)
        
        # Create dummy FBX files (real FBX is very complex)
        fbx_header = b'Kaydara FBX Binary  \\x00\\x1a\\x00'
        
        sizes = ['simple', 'medium', 'complex']
        data_sizes = [1024, 4096, 16384]
        
        for size_name, data_size in zip(sizes, data_sizes):
            with open(fbx_dir / f'{size_name}_mesh.fbx', 'wb') as f:
                f.write(fbx_header)
                # Add dummy binary data
                f.write(b'\\x00\\x01\\x02\\x03' * (data_size // 4))
        
        print(f"SUCCESS: Created FBX placeholder fixtures in {fbx_dir}")


def main():
    """Main fixture generation function"""
    print("START: Comprehensive test fixture generation...")
    print(f"DIR: Fixtures directory: {FIXTURES_DIR.absolute()}")
    
    # Create directories
    IMAGES_DIR.mkdir(parents=True, exist_ok=True)
    MESHES_DIR.mkdir(parents=True, exist_ok=True)
    
    # Generate image fixtures
    print("\\nIMG: Generating image fixtures...")
    img_gen = ImageFixtureGenerator()
    
    img_gen.create_png_fixtures()
    img_gen.create_jpeg_fixtures()
    img_gen.create_webp_fixtures()
    img_gen.create_gif_fixtures()
    img_gen.create_bmp_fixtures()
    img_gen.create_tiff_fixtures()
    img_gen.create_ico_fixtures()
    img_gen.create_svg_fixtures()
    img_gen.create_tga_fixtures()
    
    # Generate mesh fixtures
    print("\\nMESH: Generating mesh fixtures...")
    mesh_gen = MeshFixtureGenerator()
    
    mesh_gen.create_obj_fixtures()
    mesh_gen.create_stl_fixtures()
    mesh_gen.create_ply_fixtures()
    mesh_gen.create_gltf_fixtures()
    mesh_gen.create_glb_fixtures()
    mesh_gen.create_fbx_fixtures()
    
    print("\nSUCCESS: All test fixtures generated successfully!")
    print(f"STAT: Image fixtures: {len(list(IMAGES_DIR.rglob('*.*')))} files")
    print(f"STAT: Mesh fixtures: {len(list(MESHES_DIR.rglob('*.*')))} files")
    print(f"STAT: Total fixtures: {len(list(FIXTURES_DIR.rglob('*.*')))} files")


if __name__ == "__main__":
    main()
