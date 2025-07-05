"""
Shared test utilities and fixtures for pixel-squish test suite.
"""
import os
import tempfile
import shutil
from pathlib import Path
from typing import Generator, Dict, Any, Optional
import pytest
import numpy as np
from PIL import Image
import trimesh


@pytest.fixture(scope="session")
def fixtures_dir() -> Path:
    """Return the path to the test fixtures directory."""
    return Path(__file__).parent / "fixtures"


@pytest.fixture(scope="session")
def images_dir(fixtures_dir: Path) -> Path:
    """Return the path to test image fixtures."""
    return fixtures_dir / "images"


@pytest.fixture(scope="session")
def models_dir(fixtures_dir: Path) -> Path:
    """Return the path to test model fixtures."""
    return fixtures_dir / "models"


@pytest.fixture(scope="session")
def videos_dir(fixtures_dir: Path) -> Path:
    """Return the path to test video fixtures."""
    return fixtures_dir / "videos"


@pytest.fixture
def temp_dir() -> Generator[Path, None, None]:
    """Create a temporary directory for test outputs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture(scope="session")
def sample_images(images_dir: Path) -> Dict[str, Path]:
    """Create sample test images if they don't exist."""
    images = {}
    
    # Create a simple PNG test image
    png_path = images_dir / "test_image.png"
    if not png_path.exists():
        img = Image.new('RGB', (256, 256), color='red')
        # Add some detail to make compression meaningful
        pixels = np.array(img)
        pixels[50:200, 50:200] = [0, 255, 0]  # Green square
        pixels[100:150, 100:150] = [0, 0, 255]  # Blue square
        Image.fromarray(pixels).save(png_path, 'PNG')
    images['png'] = png_path
    
    # Create a JPEG version
    jpeg_path = images_dir / "test_image.jpg"
    if not jpeg_path.exists():
        img = Image.open(png_path)
        img.save(jpeg_path, 'JPEG', quality=90)
    images['jpeg'] = jpeg_path
    
    # Create a larger image for compression testing
    large_png_path = images_dir / "large_test.png"
    if not large_png_path.exists():
        img = Image.new('RGB', (1024, 1024), color='white')
        # Add noise pattern
        pixels = np.array(img)
        noise = np.random.randint(0, 50, (1024, 1024, 3))
        pixels = np.clip(pixels.astype(int) + noise, 0, 255).astype(np.uint8)
        Image.fromarray(pixels).save(large_png_path, 'PNG')
    images['large_png'] = large_png_path
    
    return images


@pytest.fixture(scope="session")
def sample_models(models_dir: Path) -> Dict[str, Path]:
    """Create sample test 3D models if they don't exist."""
    models = {}
    
    # Create a simple cube OBJ file
    obj_path = models_dir / "test_cube.obj"
    if not obj_path.exists():
        mesh = trimesh.creation.box(extents=[2, 2, 2])
        mesh.export(str(obj_path))
    models['obj'] = obj_path
    
    # Create a simple PLY file
    ply_path = models_dir / "test_sphere.ply"
    if not ply_path.exists():
        mesh = trimesh.creation.icosphere(subdivisions=2)
        mesh.export(str(ply_path))
    models['ply'] = ply_path
    
    # Create an ASCII PLY file (since our loader only supports ASCII currently)
    ascii_ply_path = models_dir / "test_cube_ascii.ply"
    if not ascii_ply_path.exists():
        # Create a simple cube in ASCII PLY format
        ply_content = """ply
format ascii 1.0
comment Created by test suite
element vertex 8
property float x
property float y
property float z
element face 12
property list uchar int vertex_indices
end_header
-1.0 -1.0 -1.0
1.0 -1.0 -1.0
1.0 1.0 -1.0
-1.0 1.0 -1.0
-1.0 -1.0 1.0
1.0 -1.0 1.0
1.0 1.0 1.0
-1.0 1.0 1.0
3 0 1 2
3 0 2 3
3 4 7 6
3 4 6 5
3 0 4 5
3 0 5 1
3 2 6 7
3 2 7 3
3 0 3 7
3 0 7 4
3 1 5 6
3 1 6 2"""
        ascii_ply_path.write_text(ply_content)
    models['ascii_ply'] = ascii_ply_path
    
    return models


def validate_image_quality(original_path: Path, optimized_path: Path) -> Dict[str, Any]:
    """
    Validate that an optimized image maintains acceptable quality.
    Returns metrics for comparison.
    """
    original_img = Image.open(original_path)
    optimized_img = Image.open(optimized_path)
    
    # Ensure both images are in the same mode for comparison
    if original_img.mode != optimized_img.mode:
        # Convert both to RGB for consistent comparison
        if original_img.mode == 'RGBA':
            original_img = original_img.convert('RGB')
        if optimized_img.mode == 'RGBA':
            optimized_img = optimized_img.convert('RGB')
        elif original_img.mode == 'RGB' and optimized_img.mode in ['L', 'P']:
            optimized_img = optimized_img.convert('RGB')
        elif optimized_img.mode == 'RGB' and original_img.mode in ['L', 'P']:
            original_img = original_img.convert('RGB')
        elif original_img.mode in ['L', 'P'] and optimized_img.mode in ['L', 'P']:
            # Both grayscale-like, convert to L
            original_img = original_img.convert('L')
            optimized_img = optimized_img.convert('L')
    
    # Resize if dimensions don't match (for format conversions)
    if original_img.size != optimized_img.size:
        optimized_img = optimized_img.resize(original_img.size, Image.Resampling.LANCZOS)
    
    original = np.array(original_img)
    optimized = np.array(optimized_img)
    
    # Ensure same number of dimensions
    if len(original.shape) != len(optimized.shape):
        if len(original.shape) == 3 and len(optimized.shape) == 2:
            # Original is RGB, optimized is grayscale
            optimized = np.stack([optimized] * 3, axis=-1)
        elif len(original.shape) == 2 and len(optimized.shape) == 3:
            # Original is grayscale, optimized is RGB - convert to grayscale
            optimized = np.mean(optimized, axis=-1)
    
    # Calculate PSNR (Peak Signal-to-Noise Ratio)
    mse = np.mean((original.astype(float) - optimized.astype(float)) ** 2)
    if mse == 0:
        psnr = float('inf')
    else:
        max_pixel_value = 255.0
        psnr = 20 * np.log10(max_pixel_value / np.sqrt(mse))
    
    # Calculate file size reduction
    original_size = original_path.stat().st_size
    optimized_size = optimized_path.stat().st_size
    size_reduction = (original_size - optimized_size) / original_size * 100
    
    return {
        'psnr': psnr,
        'size_reduction_percent': size_reduction,
        'original_size': original_size,
        'optimized_size': optimized_size,
        'dimensions_match': original_img.size == optimized_img.size,
        'original_mode': original_img.mode,
        'optimized_mode': optimized_img.mode
    }


def validate_mesh_quality(original_path: Path, optimized_path: Path) -> Dict[str, Any]:
    """
    Validate that an optimized mesh maintains acceptable quality.
    Returns metrics for comparison.
    """
    original = trimesh.load(str(original_path))
    optimized = trimesh.load(str(optimized_path))
    
    # Handle Scene objects by getting the main geometry
    if hasattr(original, 'geometry') and len(original.geometry) > 0:
        original = list(original.geometry.values())[0]
    if hasattr(optimized, 'geometry') and len(optimized.geometry) > 0:
        optimized = list(optimized.geometry.values())[0]
    
    # Check if we have valid mesh objects
    if not hasattr(original, 'vertices') or not hasattr(optimized, 'vertices'):
        return {
            'vertex_reduction_percent': 0,
            'face_reduction_percent': 0,
            'original_vertices': 0,
            'optimized_vertices': 0,
            'original_faces': 0,
            'optimized_faces': 0,
            'maintains_watertight': True,
            'volume_ratio': 1.0
        }
    
    # Calculate vertex and face reduction
    vertex_reduction = (len(original.vertices) - len(optimized.vertices)) / len(original.vertices) * 100
    face_reduction = (len(original.faces) - len(optimized.faces)) / len(original.faces) * 100
    
    # Check if mesh is still watertight (if original was)
    original_watertight = getattr(original, 'is_watertight', False)
    optimized_watertight = getattr(optimized, 'is_watertight', False)
    
    # Volume calculation with error handling
    original_volume = getattr(original, 'volume', 0)
    optimized_volume = getattr(optimized, 'volume', 0)
    volume_ratio = optimized_volume / original_volume if original_volume > 0 else 1.0
    
    return {
        'vertex_reduction_percent': vertex_reduction,
        'face_reduction_percent': face_reduction,
        'original_vertices': len(original.vertices),
        'optimized_vertices': len(optimized.vertices),
        'original_faces': len(original.faces),
        'optimized_faces': len(optimized.faces),
        'maintains_watertight': not original_watertight or optimized_watertight,
        'volume_ratio': volume_ratio
    }


class CLIRunner:
    """Helper class for running CLI commands in tests."""
    
    def __init__(self, cli_executable: Optional[str] = None):
        # Use built Rust CLI binary
        import os
        import platform
        exe_suffix = ".exe" if platform.system() == "Windows" else ""
        rust_cli_path = Path(__file__).parent.parent / "target" / "release" / f"pxjc{exe_suffix}"
        if rust_cli_path.exists():
            self.cli_executable = [str(rust_cli_path)]
        else:
            # Fallback to debug build
            debug_cli_path = Path(__file__).parent.parent / "target" / "debug" / f"pxjc{exe_suffix}"
            if debug_cli_path.exists():
                self.cli_executable = [str(debug_cli_path)]
            else:
                self.cli_executable = None
    
    def run_command(self, args: list, cwd: Optional[Path] = None) -> Dict[str, Any]:
        """Run a CLI command and return result."""
        import subprocess
        
        if cwd is None:
            cwd = Path(__file__).parent.parent  # Project root
        
        if self.cli_executable is None:
            return {
                'returncode': -1,
                'stdout': '',
                'stderr': 'CLI executable not found',
                'success': False
            }

        # Prepend 'optimize' subcommand if arguments start with --input
        if args and args[0] == '--input':
            cmd = self.cli_executable + ['optimize'] + args
        elif args and args[0] == '--validate':
            cmd = self.cli_executable + ['validate'] + args[1:]  # Remove --validate flag, use validate subcommand
        else:
            cmd = self.cli_executable + args
            
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                cwd=cwd,
                timeout=30
            )
            return {
                'returncode': result.returncode,
                'stdout': result.stdout,
                'stderr': result.stderr,
                'success': result.returncode == 0
            }
        except subprocess.TimeoutExpired:
            return {
                'returncode': -1,
                'stdout': '',
                'stderr': 'Command timed out',
                'success': False
            }
        except FileNotFoundError:
            return {
                'returncode': -1,
                'stdout': '',
                'stderr': f'CLI executable not found',
                'success': False
            }


@pytest.fixture
def cli_runner() -> CLIRunner:
    """Provide a CLI runner for integration tests."""
    return CLIRunner()
