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
        # Image formats - using generated files
        "images": {
            "png": images_dir / "sample.png",
            "jpg": images_dir / "sample.jpg", 
            "jpeg": images_dir / "large.jpg",
            "webp": images_dir / "test.webp",
            "gif": images_dir / "test.gif",
            "animated_gif": images_dir / "animated_test.gif",
            "small_png": images_dir / "small_png.png",
            "medium_png": images_dir / "medium_png.png",
            "large_png": images_dir / "large_png.png",
            "small_jpg": images_dir / "small_jpg.jpg",
            "medium_jpg": images_dir / "medium_jpg.jpg",
            "large_jpg": images_dir / "large_jpg.jpg",
            "small_webp": images_dir / "small_webp.webp",
            "medium_webp": images_dir / "medium_webp.webp", 
            "svg": images_dir / "test.svg",
            "ico": images_dir / "test.ico",
            "bmp": images_dir / "small_bmp.bmp",
            # "tiff": images_dir / "small_tiff.tiff",
        },
        
        "meshes": {
            "obj": meshes_dir / "cube.obj",
            "ply": meshes_dir / "tetrahedron.ply",
            # "stl": meshes_dir / "triangle.stl",
        },
        
        # Test file sizes for performance testing
        "sizes": {
            "small": images_dir / "small_jpg.jpg",
            "medium": images_dir / "medium_jpg.jpg", 
            "large": images_dir / "large_jpg.jpg",
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
