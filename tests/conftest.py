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
    
    return models


def validate_image_quality(original_path: Path, optimized_path: Path) -> Dict[str, Any]:
    """
    Validate that an optimized image maintains acceptable quality.
    Returns metrics for comparison.
    """
    original = np.array(Image.open(original_path))
    optimized = np.array(Image.open(optimized_path))
    
    # Calculate PSNR (Peak Signal-to-Noise Ratio)
    mse = np.mean((original.astype(float) - optimized.astype(float)) ** 2)
    if mse == 0:
        psnr = float('inf')
    else:
        psnr = 20 * np.log10(255.0 / np.sqrt(mse))
    
    # Calculate file size reduction
    original_size = original_path.stat().st_size
    optimized_size = optimized_path.stat().st_size
    size_reduction = (original_size - optimized_size) / original_size * 100
    
    return {
        'psnr': psnr,
        'size_reduction_percent': size_reduction,
        'original_size': original_size,
        'optimized_size': optimized_size,
        'dimensions_match': original.shape == optimized.shape
    }


def validate_mesh_quality(original_path: Path, optimized_path: Path) -> Dict[str, Any]:
    """
    Validate that an optimized mesh maintains acceptable quality.
    Returns metrics for comparison.
    """
    original = trimesh.load(str(original_path))
    optimized = trimesh.load(str(optimized_path))
    
    # Calculate vertex and face reduction
    vertex_reduction = (len(original.vertices) - len(optimized.vertices)) / len(original.vertices) * 100
    face_reduction = (len(original.faces) - len(optimized.faces)) / len(original.faces) * 100
    
    # Check if mesh is still watertight (if original was)
    original_watertight = original.is_watertight
    optimized_watertight = optimized.is_watertight
    
    return {
        'vertex_reduction_percent': vertex_reduction,
        'face_reduction_percent': face_reduction,
        'original_vertices': len(original.vertices),
        'optimized_vertices': len(optimized.vertices),
        'original_faces': len(original.faces),
        'optimized_faces': len(optimized.faces),
        'maintains_watertight': not original_watertight or optimized_watertight,
        'volume_ratio': optimized.volume / original.volume if original.volume > 0 else 1.0
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
