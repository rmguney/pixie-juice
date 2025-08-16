#!/usr/bin/env python3
"""
Pytest configuration for Pixie Juice web component testing
"""

import pytest
import pytest_asyncio
import asyncio
import os
import time
from pathlib import Path
from playwright.async_api import async_playwright, Browser, BrowserContext, Page
import subprocess
import signal
import requests


@pytest_asyncio.fixture(scope="session")
async def browser():
    """Launch browser for the entire test session."""
    async with async_playwright() as p:
        browser = await p.chromium.launch(
            headless=False,  # Set to True for CI/CD
            args=['--disable-web-security', '--disable-features=VizDisplayCompositor']
        )
        yield browser
        await browser.close()


@pytest_asyncio.fixture(scope="session")
async def dev_server():
    """Get development server URL, preferring existing server."""
    # Check for existing servers first
    for port in [3001, 3000]:
        try:
            url = f"http://localhost:{port}"
            response = requests.get(url, timeout=3)
            if response.status_code == 200:
                print(f"✓ Using existing development server at {url}")
                yield url
                return
        except:
            continue
    
    # If no server found, provide default URL and let user start manually
    print("⚠️  No development server found")
    print("💡 Please start the server manually: cd web; npm run dev")
    print("🔄 Using default URL for testing...")
    yield "http://localhost:3000"


@pytest_asyncio.fixture
async def context(browser: Browser):
    """Create a new browser context for each test."""
    context = await browser.new_context(
        viewport={'width': 1280, 'height': 720},
        permissions=['clipboard-read', 'clipboard-write']
    )
    yield context
    await context.close()


@pytest_asyncio.fixture
async def page(context: BrowserContext, dev_server):
    """Create a new page for each test."""
    page = await context.new_page()
    
    # Set reasonable timeout
    page.set_default_timeout(15000)
    
    # Navigate to the app with wait for load
    await page.goto(dev_server, wait_until='domcontentloaded')
    
    # Wait for React to render
    await page.wait_for_timeout(3000)
    
    yield page
    await page.close()


@pytest.fixture(scope="session")
def test_files():
    """Provide paths to test files organized by format."""
    fixtures_dir = Path(__file__).parent / "fixtures"
    
    # Use generated test files for comprehensive coverage
    images_dir = fixtures_dir / "images"
    meshes_dir = fixtures_dir / "meshes"
    
    return {
        "images": {
            "small_png": images_dir / "png" / "small_png.png",
            "medium_png": images_dir / "png" / "medium_png.png",
            "large_png": images_dir / "png" / "large_png.png",
            "small_jpg": images_dir / "jpeg" / "small_jpeg.jpg",
            "medium_jpg": images_dir / "jpeg" / "medium_jpeg.jpg",
            "large_jpg": images_dir / "jpeg" / "large_jpeg.jpg",
            "small_webp": images_dir / "webp" / "small_webp.webp",
            "medium_webp": images_dir / "webp" / "medium_webp.webp",
            "large_webp": images_dir / "webp" / "large_webp.webp",
            "small_gif": images_dir / "gif" / "small_gif.gif",
            "small_bmp": images_dir / "bmp" / "small_bmp.bmp",
            "small_tiff": images_dir / "tiff" / "small_tiff.tiff",
            "small_ico": images_dir / "ico" / "32x32.ico",
            "small_svg": images_dir / "svg" / "small_simple.svg",
            "small_tga": images_dir / "tga" / "small_tga.tga",
        },
        
        "meshes": {
            "simple_obj": meshes_dir / "obj" / "simple_cube.obj",
            "simple_ply": meshes_dir / "ply" / "simple_quad.ply",
            "simple_stl": meshes_dir / "stl" / "simple_binary.stl",
            "simple_gltf": meshes_dir / "gltf" / "simple_triangle.gltf",
            "simple_glb": meshes_dir / "glb" / "simple_triangle.glb",
        },
        
        "sizes": {
            "small": images_dir / "jpeg" / "small_jpeg.jpg",
            "medium": images_dir / "jpeg" / "medium_jpeg.jpg", 
            "large": images_dir / "jpeg" / "large_jpeg.jpg",
        }
    }


@pytest.fixture
def performance_thresholds():
    """Performance thresholds matching Pixie Juice requirements."""
    return {
        "image_target_ms": 100.0,  # <100ms for 1MB images
        "mesh_target_ms": 300.0,   # <300ms for 100k triangles
        "memory_target_mb": 100.0, # <100MB memory peak
        "violation_rate_max": 0.05, # <5% violation rate
        "compression_ratio_min": {
            "png": 0.10,  # At least 10% savings
            "jpg": 0.05,  # At least 5% savings
            "webp": 0.15, # At least 15% savings
            "gif": 0.20,  # At least 20% savings
        }
    }
