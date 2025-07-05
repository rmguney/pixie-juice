#!/usr/bin/env python3
"""
Pixie Juice - Centralized Test Suite
Main CLI for running all format tests with organized inputs and outputs
"""

import os
import sys
import argparse
import subprocess
import tempfile
import json
import time
from pathlib import Path
from PIL import Image
import logging

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Project paths
PROJECT_ROOT = Path(__file__).parent.parent.absolute()
CLI_DEBUG_PATH = PROJECT_ROOT / "target" / "debug" / "pxjc.exe"
CLI_RELEASE_PATH = PROJECT_ROOT / "target" / "release" / "pxjc.exe"
INPUTS_DIR = Path(__file__).parent / "inputs"
OUTPUTS_DIR = Path(__file__).parent / "outputs"
IMAGES_DIR = INPUTS_DIR / "images"
MESHES_DIR = INPUTS_DIR / "meshes"

SUPPORTED_IMAGE_FORMATS = {
    'PNG': 'Lossless compression using oxipng (native) / image crate (WASM)',
    'JPEG': 'Quality-based optimization using jpeg-encoder with mozjpeg fallback',
    'WEBP': 'Full support using webp crate (native) / image crate fallback (WASM)',
    'BMP': 'Converts to PNG for optimal compression',
    'TIFF': 'Converts to PNG for optimal compression',
    'GIF': 'Re-encoding with optional color reduction using image crate'
}

SUPPORTED_MESH_FORMATS = {
    'OBJ': 'Complete loader/writer with mesh decimation and vertex welding',
    'PLY': 'Complete loader/writer with mesh decimation and vertex welding',
    'STL': 'Complete loader/writer with mesh decimation and vertex welding',
    'DAE': 'XML parsing with geometry extraction and optimization',
    'FBX': 'ASCII FBX parsing with geometry extraction and optimization',
    'GLTF': 'JSON format support with proper glTF writing',
    'USDZ': 'Universal Scene Description ZIP format with geometry extraction'
}

class TestResults:
    """Track test results across all formats"""
    def __init__(self):
        self.results = {
            'images': {},
            'meshes': {},
            'summary': {
                'total_tests': 0,
                'passed': 0,
                'failed': 0,
                'start_time': time.time()
            }
        }
    
    def add_result(self, category, format_name, passed, details):
        """Add test result"""
        self.results[category][format_name] = {
            'passed': passed,
            'details': details,
            'timestamp': time.time()
        }
        self.results['summary']['total_tests'] += 1
        if passed:
            self.results['summary']['passed'] += 1
        else:
            self.results['summary']['failed'] += 1
    
    def save_report(self):
        """Save detailed test report"""
        self.results['summary']['end_time'] = time.time()
        self.results['summary']['duration'] = self.results['summary']['end_time'] - self.results['summary']['start_time']
        
        report_file = OUTPUTS_DIR / f"test_report_{int(time.time())}.json"
        with open(report_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        
        print(f"\n📊 Detailed report saved to: {report_file}")
        return report_file
    
    def print_summary(self):
        """Print test summary"""
        total = self.results['summary']['total_tests']
        passed = self.results['summary']['passed']
        failed = self.results['summary']['failed']
        duration = self.results['summary'].get('duration', 0)
        
        print(f"\n{'='*60}")
        print(f"🏁 TEST SUMMARY")
        print(f"{'='*60}")
        print(f"Total Tests: {total}")
        print(f"✅ Passed: {passed}")
        print(f"❌ Failed: {failed}")
        print(f"⏱️  Duration: {duration:.2f}s")
        print(f"Success Rate: {(passed/total*100):.1f}%" if total > 0 else "No tests run")
        
        if failed > 0:
            print(f"\n❌ Failed tests:")
            for category in ['images', 'meshes']:
                for fmt, result in self.results[category].items():
                    if not result['passed']:
                        print(f"  - {category.upper()} {fmt}: {result['details']}")

class TestSuite:
    """Main test suite for Pixie Juice"""
    
    def __init__(self):
        self.results = TestResults()
        self.ensure_directories()
    
    def ensure_directories(self):
        """Ensure test directories exist"""
        for dir_path in [INPUTS_DIR, OUTPUTS_DIR, IMAGES_DIR, MESHES_DIR]:
            dir_path.mkdir(parents=True, exist_ok=True)
        logger.info(f"Test directories ready: {INPUTS_DIR}, {OUTPUTS_DIR}")
    
    def ensure_cli_built(self, use_release=False):
        """Ensure CLI is built"""
        cli_path = CLI_RELEASE_PATH if use_release else CLI_DEBUG_PATH
        
        if not cli_path.exists():
            logger.info(f"CLI not found at {cli_path}, building...")
            build_args = ["cargo", "build"]
            if use_release:
                build_args.append("--release")
            
            result = subprocess.run(
                build_args,
                cwd=PROJECT_ROOT,
                capture_output=True,
                text=True
            )
            if result.returncode != 0:
                logger.error(f"Failed to build CLI: {result.stderr}")
                sys.exit(1)
        logger.info(f"CLI ready: {cli_path}")
        return cli_path
    
    def create_test_image(self, format_type, size=(200, 150)):
        """Create test image for specific format"""
        img = Image.new('RGB', size, color='white')
        pixels = img.load()
        
        if pixels:
            for x in range(size[0]):
                for y in range(size[1]):
                    r = int(255 * (x / size[0]))
                    g = int(255 * (y / size[1]))
                    b = int(255 * ((x + y) % 256) / 255)
                    pixels[x, y] = (r, g, b)
        
        return img
    
    def create_test_cube_obj(self):
        """Create a simple test cube in OBJ format"""
        obj_content = '''# Simple test cube
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
        return obj_content
    
    def test_image_format(self, format_type, quality=30, cli_path=None):
        """Test optimization for a specific image format"""
        if cli_path is None:
            cli_path = CLI_DEBUG_PATH
        logger.info(f"Testing {format_type.upper()} format...")
        
        try:
            input_file = IMAGES_DIR / f"test_{format_type.lower()}.{format_type.lower()}"
            output_file = OUTPUTS_DIR / f"optimized_{format_type.lower()}.{format_type.lower()}"
            
            # Create test image if it doesn't exist
            if not input_file.exists():
                img = self.create_test_image(format_type)
                
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
                    img.save(input_file, 'TIFF')
                
                logger.info(f"Created test image: {input_file}")
            
            # Run CLI optimization
            cmd = [
                str(cli_path),
                "optimize",
                "--input", str(input_file),
                "--output", str(output_file),
                "--quality", str(quality)
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0 and output_file.exists():
                # Check size reduction
                input_size = input_file.stat().st_size
                output_size = output_file.stat().st_size
                reduction = ((input_size - output_size) / input_size) * 100
                
                details = f"Size: {input_size} → {output_size} bytes ({reduction:.1f}% reduction)"
                logger.info(f"✅ {format_type.upper()}: {details}")
                self.results.add_result('images', format_type.upper(), True, details)
                return True
            else:
                error_msg = result.stderr.strip() or "Unknown error"
                logger.error(f"❌ {format_type.upper()}: {error_msg}")
                self.results.add_result('images', format_type.upper(), False, error_msg)
                return False
                
        except Exception as e:
            error_msg = f"Exception: {str(e)}"
            logger.error(f"❌ {format_type.upper()}: {error_msg}")
            self.results.add_result('images', format_type.upper(), False, error_msg)
            return False
    
    def test_mesh_format(self, format_type, target_reduction=0.5, cli_path=None):
        """Test optimization for a specific mesh format"""
        if cli_path is None:
            cli_path = CLI_DEBUG_PATH
        logger.info(f"Testing {format_type.upper()} format...")
        
        try:
            input_file = MESHES_DIR / f"test_{format_type.lower()}.{format_type.lower()}"
            output_file = OUTPUTS_DIR / f"optimized_{format_type.lower()}.{format_type.lower()}"
            
            # Create test mesh if it doesn't exist
            if not input_file.exists():
                if format_type.upper() == 'OBJ':
                    with open(input_file, 'w') as f:
                        f.write(self.create_test_cube_obj())
                else:
                    # For other formats, try to convert from OBJ using CLI
                    obj_file = MESHES_DIR / "test_obj.obj"
                    if not obj_file.exists():
                        with open(obj_file, 'w') as f:
                            f.write(self.create_test_cube_obj())
                    
                    # Note: For real implementation, we'd need proper format conversion
                    # For now, skip creating other mesh formats programmatically
                    if format_type.upper() in ['PLY', 'STL', 'DAE', 'FBX', 'GLTF']:
                        logger.warning(f"Skipping {format_type.upper()} - no test file available")
                        self.results.add_result('meshes', format_type.upper(), False, "No test file available")
                        return False
                
                logger.info(f"Created test mesh: {input_file}")
            
            # Run CLI optimization
            cmd = [
                str(cli_path),
                "optimize",
                "--input", str(input_file),
                "--output", str(output_file),
                "--target-reduction", str(target_reduction)
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0 and output_file.exists():
                # Check size reduction
                input_size = input_file.stat().st_size
                output_size = output_file.stat().st_size
                reduction = ((input_size - output_size) / input_size) * 100
                
                details = f"Size: {input_size} → {output_size} bytes ({reduction:.1f}% reduction)"
                logger.info(f"✅ {format_type.upper()}: {details}")
                self.results.add_result('meshes', format_type.upper(), True, details)
                return True
            else:
                error_msg = result.stderr.strip() or "Unknown error"
                logger.error(f"❌ {format_type.upper()}: {error_msg}")
                self.results.add_result('meshes', format_type.upper(), False, error_msg)
                return False
                
        except Exception as e:
            error_msg = f"Exception: {str(e)}"
            logger.error(f"❌ {format_type.upper()}: {error_msg}")
            self.results.add_result('meshes', format_type.upper(), False, error_msg)
            return False
    
    def run_all_tests(self, cli_path, image_quality=30, mesh_reduction=0.5):
        """Run tests for all supported formats"""
        print(f"🚀 Starting Pixie Juice Test Suite")
        print(f"📂 Inputs: {INPUTS_DIR}")
        print(f"📂 Outputs: {OUTPUTS_DIR}")
        print(f"🔧 CLI: {cli_path}")
        print(f"{'='*60}")
        
        # Test image formats
        print(f"\n📸 TESTING IMAGE FORMATS")
        print(f"{'='*40}")
        for format_type in SUPPORTED_IMAGE_FORMATS:
            self.test_image_format(format_type, image_quality, cli_path)
        
        # Test mesh formats
        print(f"\n🎯 TESTING MESH FORMATS")
        print(f"{'='*40}")
        for format_type in SUPPORTED_MESH_FORMATS:
            self.test_mesh_format(format_type, mesh_reduction, cli_path)
        print(f"\n🎯 TESTING MESH FORMATS")
        print(f"{'='*40}")
        for format_type in SUPPORTED_MESH_FORMATS:
            self.test_mesh_format(format_type, mesh_reduction)
        
        # Generate report
        self.results.print_summary()
        report_file = self.results.save_report()
        
        return self.results.results['summary']['failed'] == 0

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='Pixie Juice Test Suite')
    parser.add_argument('--format', choices=list(SUPPORTED_IMAGE_FORMATS.keys()) + list(SUPPORTED_MESH_FORMATS.keys()),
                       help='Test specific format only')
    parser.add_argument('--image-quality', type=int, default=30, help='Image optimization quality (1-100)')
    parser.add_argument('--mesh-reduction', type=float, default=0.5, help='Mesh reduction target (0.0-1.0)')
    parser.add_argument('--list-formats', action='store_true', help='List all supported formats')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--release', action='store_true', help='Use release build instead of debug')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    if args.list_formats:
        print("📸 Supported Image Formats:")
        for fmt, desc in SUPPORTED_IMAGE_FORMATS.items():
            print(f"  {fmt}: {desc}")
        print("\n🎯 Supported Mesh Formats:")
        for fmt, desc in SUPPORTED_MESH_FORMATS.items():
            print(f"  {fmt}: {desc}")
        return 0
    
    suite = TestSuite()
    cli_path = suite.ensure_cli_built(use_release=getattr(args, 'release', False))
    
    if args.format:
        # Test single format
        if args.format in SUPPORTED_IMAGE_FORMATS:
            success = suite.test_image_format(args.format, args.image_quality, cli_path)
        else:
            success = suite.test_mesh_format(args.format, args.mesh_reduction, cli_path)
        
        suite.results.print_summary()
        return 0 if success else 1
    else:
        # Run all tests
        success = suite.run_all_tests(cli_path, args.image_quality, args.mesh_reduction)
        return 0 if success else 1

if __name__ == "__main__":
    sys.exit(main())
