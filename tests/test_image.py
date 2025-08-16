#!/usr/bin/env python3
"""
Comprehensive test that runs the full functionality suite with logging
"""

import asyncio
import json
import time
import re
from pathlib import Path
from datetime import datetime
from playwright.async_api import async_playwright
import pytest


class ConsoleLogger:
    """Cap    print(f"TEST: Test File Validation:")
    print(f"   SUCCESS: Available: {len(available_cases)} files")
    print(f"   MISSING: Missing: {len(missing_files)} files")e and log browser console messages during testing."""
    
    def __init__(self):
        self.logs = []
        self.messages = []
        self.errors = []
        self.warnings = []
        
    def handle_console(self, msg):
        """Handle console messages from the browser."""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        message_text = msg.text
        message_type = msg.type
        
        log_entry = {
            "timestamp": timestamp,
            "type": message_type,
            "message": message_text,
            "location": msg.location if hasattr(msg, 'location') else None
        }
        
        self.logs.append(log_entry)
        self.messages.append(log_entry)
        
        # Categorize messages
        if message_type == 'error':
            self.errors.append(log_entry)
        elif message_type == 'warning':
            self.warnings.append(log_entry)
            
        # Real-time console output with formatting
        prefix = {
            'log': '[LOG]',
            'info': '[INFO]',
            'warn': '[WARN]',
            'error': '[ERROR]',
            'debug': '[DEBUG]'
        }.get(message_type, '[MSG]')
        
        print(f"   {prefix} [{timestamp}] {message_type.upper()}: {message_text}")
        
        # Special handling for WASM-related messages
        if 'wasm' in message_text.lower() or 'pixie' in message_text.lower():
            print(f"      WASM/Pixie: {message_text}")
            
        # Special handling for optimization messages
        if any(keyword in message_text.lower() for keyword in ['optimization', 'compress', 'image', 'jpeg', 'png', 'webp']):
            print(f"      Optimization: {message_text}")
    
    def handle_page_error(self, error):
        """Handle page errors."""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        error_entry = {
            "timestamp": timestamp,
            "type": "page_error",
            "message": str(error),
            "location": None
        }
        
        self.errors.append(error_entry)
        self.logs.append(error_entry)
        print(f"   PAGE ERROR [{timestamp}]: {error}")
    
    def get_summary(self):
        """Get a summary of console activity."""
        return {
            "total_messages": len(self.logs),
            "error_count": len(self.errors),
            "warning_count": len(self.warnings),
            "info_count": len([m for m in self.logs if m['type'] in ['log', 'info', 'debug']]),
            "error_messages": [e["message"] for e in self.errors[-5:]],  # Last 5 errors
            "warning_messages": [w["message"] for w in self.warnings[-5:]]  # Last 5 warnings
        }


async def setup_page_logging(page, console_logger):
    """Set up comprehensive page logging including console, network, and errors."""
    
    # Console message logging
    page.on("console", console_logger.handle_console)
    
    # Page error logging
    page.on("pageerror", console_logger.handle_page_error)
    
    # Network request logging (optional - can be verbose)
    def handle_request(request):
        if 'wasm' in request.url.lower() or 'pixie' in request.url.lower():
            print(f"   NET: {request.method} {request.url}")
    
    page.on("request", handle_request)
    
    return console_logger


async def wait_for_result_panel(page, timeout_ms=20000) -> bool:
    """Wait for the result panel to become visible after processing completion."""
    try:
        # Look for specific result panel selectors based on actual UI structure
        result_selectors = [
            # Primary indicators - download buttons and links
            'button:has-text("Download"):visible',
            'a[download]:visible',
            
            # Results panel structure from ResultsPanel.js
            'div:has(button:has-text("Download")):visible',
            'div:has-text("optimized"):visible',
            
            # Specific UI elements from the component
            '.space-y-2:has(button:has-text("Download"))',
            'div:has-text("→"):visible',  # File size arrows
            
            # Fallback selectors
            '[data-testid="result-panel"]',
            '.result-panel',
            '[data-testid="optimization-result"]',
            '.optimization-result',
        ]
        
        print(f"   Waiting for result panel to appear...")
        
        for selector in result_selectors:
            try:
                await page.locator(selector).wait_for(state="visible", timeout=timeout_ms)
                print(f"   Result panel found: {selector}")
                return True
            except:
                continue
        
        print(f"   WARN: No result panel found within {timeout_ms/1000}s")
        return False
        
    except Exception as e:
        print(f"   Error waiting for result panel: {e}")
        return False


async def debug_ui_content(page) -> None:
    """Debug helper to understand what's actually in the UI."""
    try:
        print("\n DEBUG: UI Content Analysis")
        print("-" * 40)
        
        # Check for various elements
        elements_to_check = [
            ('File inputs', 'input[type="file"]'),
            ('Download buttons', 'button:has-text("Download")'),
            ('Download links', 'a[download]'),
            ('Result panels', 'div:has(button:has-text("Download"))'),
            ('Percentage text', 'div:has-text("%")'),
            ('Arrow text', 'div:has-text("→")'),
        ]
        
        for name, selector in elements_to_check:
            count = await page.locator(selector).count()
            print(f"   {name}: {count} found")
            
            if count > 0 and count <= 3:  # Show text for small counts
                for i in range(count):
                    try:
                        text = await page.locator(selector).nth(i).text_content()
                        if text and len(text.strip()) > 0:
                            print(f"     [{i}]: {text.strip()[:100]}")
                    except:
                        pass
        
        print("-" * 40)
        
    except Exception as e:
        print(f"   Debug failed: {e}")


async def extract_metrics_from_result_panel(page) -> dict:
    """Extract file size reduction metrics from the visible result panel."""
    try:
        # Wait for content to load and become stable
        await page.wait_for_timeout(1000)
        
        print(f"   EXTRACT: Extracting metrics from result panel...")
        
        results = {
            "compression_percentage": None,
            "download_available": False,
            "original_size": None,
            "optimized_size": None,
            "file_sizes_text": None
        }
        
        # Check for download availability first (most reliable indicator)
        download_links = await page.locator('a[download]').count()
        download_buttons = await page.locator('button:has-text("Download")').count()
        results["download_available"] = download_links > 0 or download_buttons > 0
        
        if results["download_available"]:
            print(f"   SUCCESS: Download available: {download_links} links, {download_buttons} buttons")
        
        # Look for file size information in the results panel
        # The UI shows: "123 KB → 89 KB (-27.6%)"
        results_panel = page.locator('.space-y-2, [class*="result"], .border:has(button:has-text("Download"))')
        
        if await results_panel.count() > 0:
            panel_text = await results_panel.first.text_content()
            print(f"   Results panel text: {panel_text[:200]}...")
            
            # Extract file size information using multiple patterns
            size_patterns = [
                # Pattern: "123 KB → 89 KB (-27.6%)" or "123 KB → 89 KB (+27.6%)"
                r'(\d+(?:\.\d+)?)\s*([KMGT]?B)\s*→\s*(\d+(?:\.\d+)?)\s*([KMGT]?B)\s*\(([+-]?\d+(?:\.\d+)?)%\)',
                # Pattern: "(+15.5%)" or "(-25.3%)"
                r'\(([+-]?\d+(?:\.\d+)?)%\)',
                # Pattern: "15.5% smaller" or "25.3% reduction"
                r'(\d+(?:\.\d+)?)%\s*(?:smaller|reduction|saved|compressed)',
            ]
            
            for pattern in size_patterns:
                matches = re.findall(pattern, panel_text, re.IGNORECASE)
                if matches:
                    print(f"   Found matches with pattern: {pattern[:50]}...")
                    print(f"   Matches: {matches}")
                    
                    if len(matches[0]) == 5:  # Full size pattern match
                        orig_size, orig_unit, opt_size, opt_unit, percentage = matches[0]
                        results["original_size"] = f"{orig_size} {orig_unit}"
                        results["optimized_size"] = f"{opt_size} {opt_unit}"
                        results["compression_percentage"] = abs(float(percentage))
                        results["file_sizes_text"] = f"{orig_size} {orig_unit} → {opt_size} {opt_unit}"
                        print(f"   Extracted: {results['file_sizes_text']} ({percentage}%)")
                        break
                    elif isinstance(matches[0], str):  # Simple percentage match
                        percentage = matches[0].replace('+', '').replace('-', '')
                        results["compression_percentage"] = abs(float(percentage))
                        print(f"   Extracted compression: {percentage}%")
                        break
                    else:
                        percentage = matches[0]
                        results["compression_percentage"] = abs(float(percentage))
                        print(f"   Extracted compression: {percentage}%")
                        break
        
        # Alternative: Look for summary section with total savings
        summary_section = page.locator('div:has-text("optimized"), div:has-text("file"), .bg-neutral-900')
        if await summary_section.count() > 0 and not results["compression_percentage"]:
            summary_text = await summary_section.first.text_content()
            print(f"   Summary text: {summary_text}")
            
            # Look for percentage in summary
            summary_matches = re.findall(r'[(-](\d+(?:\.\d+)?)%[)]', summary_text)
            if summary_matches:
                results["compression_percentage"] = float(summary_matches[0])
                print(f"   Found summary compression: {results['compression_percentage']}%")
        
        # Final fallback: Get all text and search broadly
        if not results["compression_percentage"]:
            all_text = await page.text_content('body')
            
            # Look for any percentage that might indicate compression
            all_percentages = re.findall(r'[(-](\d+(?:\.\d+)?)%[)]', all_text)
            if all_percentages:
                # Take the first reasonable percentage (between 1% and 90%)
                for pct in all_percentages:
                    pct_val = float(pct)
                    if 1.0 <= pct_val <= 90.0:
                        results["compression_percentage"] = pct_val
                        print(f"   WARN: Fallback compression detected: {pct_val}%")
                        break
        
        return results
        
    except Exception as e:
        print(f"   Error extracting metrics: {e}")
        return {
            "compression_percentage": None,
            "download_available": False,
            "original_size": None,
            "optimized_size": None,
            "file_sizes_text": None
        }


async def validate_test_files(test_cases) -> list:
    """Validate that test files exist and return available test cases."""
    available_cases = []
    missing_files = []
    
    for format_name, fixture_path, expected_compression in test_cases:
        fixture_file = Path(fixture_path)
        if fixture_file.exists():
            available_cases.append((format_name, fixture_path, expected_compression))
        else:
            missing_files.append((format_name, fixture_path))
    
    print(f"   Test File Validation:")
    print(f"   Available: {len(available_cases)} files")
    print(f"   Missing: {len(missing_files)} files")
    
    if missing_files and len(missing_files) <= 5:  # Show first 5 missing files
        print(f"   NOTE: Missing files:")
        for format_name, path in missing_files[:5]:
            print(f"      - {format_name}: {path}")
    
    return available_cases


async def run_comprehensive_test():
    """Run comprehensive testing with all metrics and logging."""
    
    test_results = {
        "test_date": time.strftime("%Y-%m-%d %H:%M:%S"),
        "formats_tested": {},
        "performance_metrics": {},
        "compression_results": {},
        "issues_found": []
    }
    
    print("TEST: Pixie Juice Comprehensive Testing Suite")
    print("=" * 60)
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=False, slow_mo=200)   # need to switch to headless for async tests sometime
        page = await browser.new_page()
        
        # Initialize console logging
        console_logger = ConsoleLogger()
        await setup_page_logging(page, console_logger)
        print("LOG: Browser console logging initialized")
        
        # Set longer timeout for stability
        page.set_default_timeout(30000)  # 30 seconds
        
        try:
            # Navigate to app
            print("NET: Loading application...")
            await page.goto("http://localhost:3000")
            await page.wait_for_timeout(5000)  # Longer initial wait
            
            print("CHECK: Checking if application loaded properly...")
            # Verify the page loaded
            title = await page.title()
            print(f"   Page title: {title}")
            
            # Look for file input to confirm app is ready
            file_input = page.locator('input[type="file"]')
            if await file_input.count() > 0:
                print("   SUCCESS: File input found - app is ready")
            else:
                print("   WARN: No file input found - app may not be ready")
                # Wait a bit more
                await page.wait_for_timeout(3000)
            
            # Test comprehensive format coverage with different sizes
            test_cases = [
                # PNG formats - using generated fixtures
                ("PNG_Small", "fixtures/images/png/small_png.png", 15),
                ("PNG_Medium", "fixtures/images/png/medium_png.png", 20),
                ("PNG_Large", "fixtures/images/png/large_png.png", 25),
                ("PNG_Ultra", "fixtures/images/png/ultra_png.png", 30),
                ("PNG_Transparent_Small", "fixtures/images/png/small_png_transparent.png", 10),
                ("PNG_Transparent_Medium", "fixtures/images/png/medium_png_transparent.png", 15),
                ("PNG_Lossless_Small", "fixtures/images/png/small_png_lossless.png", 10),
                ("PNG_Lossless_Medium", "fixtures/images/png/medium_png_lossless.png", 15),
                ("PNG_Lossless_Large", "fixtures/images/png/large_png_lossless.png", 20),
                
                # JPEG formats - using generated fixtures
                ("JPEG_Small", "fixtures/images/jpeg/small_jpeg.jpg", 10),
                ("JPEG_Medium", "fixtures/images/jpeg/medium_jpeg.jpg", 15),
                ("JPEG_Large", "fixtures/images/jpeg/large_jpeg.jpg", 20),
                ("JPEG_Low_Small", "fixtures/images/jpeg/small_jpeg_low.jpg", 5),
                ("JPEG_Low_Medium", "fixtures/images/jpeg/medium_jpeg_low.jpg", 8),
                ("JPEG_Low_Large", "fixtures/images/jpeg/large_jpeg_low.jpg", 10),
                ("JPEG_High_Small", "fixtures/images/jpeg/small_jpeg_high.jpg", 15),
                ("JPEG_High_Medium", "fixtures/images/jpeg/medium_jpeg_high.jpg", 20),
                ("JPEG_High_Large", "fixtures/images/jpeg/large_jpeg_high.jpg", 25),
                ("JPEG_Ultra", "fixtures/images/jpeg/ultra_jpeg.jpg", 25),
                
                # WebP formats - using generated fixtures
                ("WebP_Small", "fixtures/images/webp/small_webp.webp", 15),
                ("WebP_Medium", "fixtures/images/webp/medium_webp.webp", 20),
                ("WebP_Large", "fixtures/images/webp/large_webp.webp", 25),
                ("WebP_Low_Small", "fixtures/images/webp/small_webp_low.webp", 8),
                ("WebP_Low_Medium", "fixtures/images/webp/medium_webp_low.webp", 10),
                ("WebP_High_Large", "fixtures/images/webp/large_webp_high.webp", 30),
                ("WebP_Lossless_Small", "fixtures/images/webp/small_webp_lossless.webp", 12),
                ("WebP_Lossless_Medium", "fixtures/images/webp/medium_webp_lossless.webp", 18),
                ("WebP_Ultra", "fixtures/images/webp/ultra_webp.webp", 35),
                
                # GIF formats - using generated fixtures
                ("GIF_Small", "fixtures/images/gif/small_gif.gif", 20),
                ("GIF_Medium", "fixtures/images/gif/medium_gif.gif", 25),
                ("GIF_Animated_Small", "fixtures/images/gif/small_gif_animated.gif", 30),
                ("GIF_Animated_Medium", "fixtures/images/gif/medium_gif_animated.gif", 40),
                
                # BMP formats - using generated fixtures
                ("BMP_Small", "fixtures/images/bmp/small_bmp.bmp", 25),
                ("BMP_Medium", "fixtures/images/bmp/medium_bmp.bmp", 30),
                ("BMP_Large", "fixtures/images/bmp/large_bmp.bmp", 35),
                
                # TIFF formats - using generated fixtures
                ("TIFF_Small", "fixtures/images/tiff/small_tiff.tiff", 20),
                ("TIFF_Medium", "fixtures/images/tiff/medium_tiff.tiff", 25),
                ("TIFF_Large", "fixtures/images/tiff/large_tiff.tiff", 30),
                ("TIF_Small", "fixtures/images/tiff/small_tif.tif", 20),
                ("TIFF_LZW_Medium", "fixtures/images/tiff/medium_tiff_lzw.tiff", 15),
                
                # ICO formats - using generated fixtures
                ("ICO_16x16", "fixtures/images/ico/16x16.ico", 15),
                ("ICO_32x32", "fixtures/images/ico/32x32.ico", 18),
                ("ICO_64x64", "fixtures/images/ico/64x64.ico", 22),
                ("ICO_Multi", "fixtures/images/ico/multi_size.ico", 25),
                
                # SVG formats - using generated fixtures
                ("SVG_Simple_Small", "fixtures/images/svg/small_simple.svg", 30),
                ("SVG_Simple_Medium", "fixtures/images/svg/medium_simple.svg", 35),
                ("SVG_Complex_Small", "fixtures/images/svg/small_complex.svg", 40),
                ("SVG_Complex_Large", "fixtures/images/svg/large_complex.svg", 50),
                
                # TGA formats - using generated fixtures (NEW FORMAT SUPPORT)
                ("TGA_Small", "fixtures/images/tga/small_tga.tga", 20),
                ("TGA_Medium", "fixtures/images/tga/medium_tga.tga", 25),
                ("TGA_Large", "fixtures/images/tga/large_tga.tga", 30),
                ("TGA_24bit_Small", "fixtures/images/tga/small_tga_24bit.tga", 18),
                ("TGA_32bit_Medium", "fixtures/images/tga/medium_tga_32bit.tga", 22),
                ("TGA_16bit_Small", "fixtures/images/tga/small_tga_16bit.tga", 15),
                ("TGA_24bit", "fixtures/images/tga/test_24bit.tga", 25),
                ("TGA_32bit", "fixtures/images/tga/test_32bit.tga", 30),
                ("TGA_Compressed", "fixtures/images/tga/compressed.tga", 28),
                ("TGA_Uncompressed", "fixtures/images/tga/uncompressed.tga", 32),
                
                # Other formats
                ("ICO", "fixtures/images/test.ico", 20),
                ("SVG", "fixtures/images/test.svg", 10),
                
                # Sample formats
                ("Sample_JPG", "fixtures/images/sample.jpg", 15),
                ("Sample_PNG", "fixtures/images/sample.png", 20),
            ]
            
            # Validate test files and get available cases
            available_test_cases = await validate_test_files(test_cases)
            
            if not available_test_cases:
                print("ERROR: No test files available!")
                return test_results
            
            print(f"\nTEST: Running tests on {len(available_test_cases)} available files...")
            
            for format_name, fixture_path, expected_compression in available_test_cases:
                fixture_file = Path(fixture_path)
                
                print(f"\nTEST: Testing {format_name} format...")
                
                # Reset console logger for this test
                console_logger.messages.clear()
                console_logger.errors.clear()
                console_logger.warnings.clear()
                
                try:
                    # Get original file size
                    original_size = fixture_file.stat().st_size
                    print(f"   SIZE: Original size: {original_size:,} bytes")
                    
                    # Track console activity before file upload
                    pre_upload_console_count = len(console_logger.logs)
                    print(f"   LOG: Console messages before upload: {pre_upload_console_count}")
                    
                    # Upload file
                    print(f"   UPLOAD: Uploading {fixture_file.name}...")
                    file_input = page.locator('input[type="file"]')
                    await file_input.set_input_files(str(fixture_file))
                    await page.wait_for_timeout(1000)  # Reduced from 2000ms
                    
                    # Check console activity after upload
                    post_upload_console_count = len(console_logger.logs)
                    if post_upload_console_count > pre_upload_console_count:
                        print(f"   CONSOLE: Console activity during upload: {post_upload_console_count - pre_upload_console_count} new messages")
                    
                    # Set quality
                    quality_slider = page.locator('input[type="range"]')
                    if await quality_slider.count() > 0:
                        await quality_slider.fill('75')
                        await page.wait_for_timeout(300)  # Reduced from 500ms
                    
                    # Click optimize
                    optimize_btn = page.locator('button:has-text("Optimize")')
                    if await optimize_btn.count() > 0:
                        print("   START: Starting optimization...")
                        
                        # Track console activity during optimization
                        pre_optimization_console_count = len(console_logger.logs)
                        
                        await optimize_btn.click()
                        
                        # Wait for result panel to appear (reliable completion indicator)
                        result_panel_visible = await wait_for_result_panel(page, 20000)  # 20 seconds max
                        
                        # Check console activity during optimization
                        post_optimization_console_count = len(console_logger.logs)
                        if post_optimization_console_count > pre_optimization_console_count:
                            print(f"   CONSOLE: Console activity during optimization: {post_optimization_console_count - pre_optimization_console_count} new messages")
                        
                        if not result_panel_visible:
                            # Debug UI content when result panel is not found
                            await debug_ui_content(page)
                        
                        # Extract metrics from result panel
                        metrics = await extract_metrics_from_result_panel(page)
                        
                        compression_ratio = metrics.get("compression_percentage")
                        download_available = metrics.get("download_available", False)
                        original_size_text = metrics.get("original_size")
                        optimized_size_text = metrics.get("optimized_size")
                        file_sizes_text = metrics.get("file_sizes_text")
                        
                        # Success criteria - result panel visible and download available
                        success = result_panel_visible and download_available
                        
                        # Store results with enhanced metrics
                        test_results["formats_tested"][format_name] = {
                            "success": success,
                            "original_size_bytes": original_size,
                            "compression_ratio_percent": compression_ratio,
                            "result_panel_visible": result_panel_visible,
                            "download_available": download_available,
                            "quality_setting": 75,
                            "original_size_text": original_size_text,
                            "optimized_size_text": optimized_size_text,
                            "file_sizes_display": file_sizes_text,
                            "console_summary": console_logger.get_summary()
                        }
                        
                        # Enhanced logging with file size details
                        print(f"   SIZE: Original size: {original_size:,} bytes")
                        
                        if file_sizes_text:
                            print(f"   UI: UI shows: {file_sizes_text}")
                        elif original_size_text and optimized_size_text:
                            print(f"   UI: UI shows: {original_size_text} -> {optimized_size_text}")
                        
                        if compression_ratio:
                            print(f"   COMPRESSION: Compression: {compression_ratio:.1f}% (expected: >={expected_compression}%) {'SUCCESS' if compression_ratio >= expected_compression else 'FAIL'}")
                        else:
                            print(f"   WARN: Could not extract compression ratio from UI")
                            
                        print(f"   DOWNLOAD: Downloads: {'SUCCESS' if download_available else 'FAIL'}")
                        print(f"   PANEL: Result panel: {'SUCCESS' if result_panel_visible else 'FAIL'}")
                        print(f"   RESULT: Overall: {'SUCCESS' if success else 'FAILED'}")
                        
                        # Print mini console summary for this test
                        if console_logger.errors or console_logger.warnings:
                            print(f"   LOG: Console: {len(console_logger.errors)} errors, {len(console_logger.warnings)} warnings")
                        
                        # Clear for next test
                        clear_btn = page.locator('button:has-text("Clear")')
                        if await clear_btn.count() > 0:
                            await clear_btn.click()
                            await page.wait_for_timeout(1000)
                        else:
                            # Reload page if no clear button
                            await page.reload()
                            await page.wait_for_timeout(3000)
                    
                    else:
                        print(f"   ERROR: {format_name}: No optimize button found")
                        test_results["formats_tested"][format_name] = {
                            "success": False,
                            "error": "No optimize button",
                            "original_size_bytes": original_size
                        }
                
                except Exception as e:
                    print(f"   ERROR: {format_name}: Error - {e}")
                    try:
                        original_size = fixture_file.stat().st_size
                    except:
                        original_size = 0
                    test_results["formats_tested"][format_name] = {
                        "success": False,
                        "error": str(e),
                        "original_size_bytes": original_size
                    }
                    
                    # Reset page state
                    try:
                        await page.reload()
                        await page.wait_for_timeout(3000)
                    except:
                        pass
            
            # Test quality settings with medium JPEG
            medium_jpeg = Path("fixtures/images/medium_jpg.jpg")
            if medium_jpeg.exists():
                print(f"\nTEST: Testing Quality Settings...")
                
                for quality in [25, 50, 75, 90]:
                    try:
                        print(f"\n   QUALITY: Testing Quality {quality}%...")
                        original_size = medium_jpeg.stat().st_size
                        
                        # Reset console logger for this quality test
                        console_logger.messages.clear()
                        console_logger.errors.clear()
                        console_logger.warnings.clear()
                        
                        # Upload file
                        file_input = page.locator('input[type="file"]')
                        await file_input.set_input_files(str(medium_jpeg))
                        await page.wait_for_timeout(1000)  # Reduced from 2000ms
                        
                        # Set quality
                        quality_slider = page.locator('input[type="range"]')
                        if await quality_slider.count() > 0:
                            await quality_slider.fill(str(quality))
                            await page.wait_for_timeout(300)  # Reduced from 500ms
                        
                        # Optimize
                        optimize_btn = page.locator('button:has-text("Optimize")')
                        if await optimize_btn.count() > 0:
                            print(f"      START: Starting optimization at quality {quality}%...")
                            
                            # Track console activity during optimization
                            pre_optimization_console_count = len(console_logger.logs)
                            
                            await optimize_btn.click()
                            
                            # Wait for result panel to appear
                            result_panel_visible = await wait_for_result_panel(page, 20000)  # 20 seconds max
                            
                            # Check console activity during optimization
                            post_optimization_console_count = len(console_logger.logs)
                            if post_optimization_console_count > pre_optimization_console_count:
                                print(f"      CONSOLE: Console activity during optimization: {post_optimization_console_count - pre_optimization_console_count} new messages")
                            
                            # Wait for result panel to appear (reliable completion indicator)
                            result_panel_visible = await wait_for_result_panel(page, 15000)  # 15 seconds max
                            
                            # Extract metrics from result panel
                            metrics = await extract_metrics_from_result_panel(page)
                            
                            compression_ratio = metrics.get("compression_percentage")
                            download_available = metrics.get("download_available", False)
                            original_size_text = metrics.get("original_size")
                            optimized_size_text = metrics.get("optimized_size")
                            file_sizes_text = metrics.get("file_sizes_text")
                            
                            # Store quality results with enhanced metrics
                            test_results["compression_results"][f"Quality_{quality}%"] = {
                                "quality_setting": quality,
                                "original_size_bytes": original_size,
                                "compression_ratio_percent": compression_ratio,
                                "result_panel_visible": result_panel_visible,
                                "download_available": download_available,
                                "original_size_text": original_size_text,
                                "optimized_size_text": optimized_size_text,
                                "file_sizes_display": file_sizes_text,
                                "console_summary": console_logger.get_summary()
                            }
                            
                            # Enhanced logging with file size details
                            print(f"      SIZE: Original size: {original_size:,} bytes")
                            
                            if file_sizes_text:
                                print(f"      UI: UI shows: {file_sizes_text}")
                            elif original_size_text and optimized_size_text:
                                print(f"      UI: UI shows: {original_size_text} -> {optimized_size_text}")
                            
                            if compression_ratio:
                                print(f"      COMPRESSION: Compression: {compression_ratio:.1f}%")
                            else:
                                print(f"      WARN: Could not extract compression ratio from UI")
                            
                            print(f"      DOWNLOAD: Downloads: {'SUCCESS' if download_available else 'FAIL'}")
                            print(f"      PANEL: Result panel: {'SUCCESS' if result_panel_visible else 'FAIL'}")
                            print(f"      RESULT: Quality {quality}%: {'SUCCESS' if download_available else 'FAILED'}")
                            
                            # Print mini console summary for quality test
                            if console_logger.errors or console_logger.warnings:
                                print(f"      LOG: Console: {len(console_logger.errors)} errors, {len(console_logger.warnings)} warnings")
                            
                            # Reset
                            await page.reload()
                            await page.wait_for_timeout(2000)
                    
                    except Exception as e:
                        print(f"      ERROR: Quality {quality}% error: {e}")
                        await page.reload()
                        await page.wait_for_timeout(2000)
            
        except Exception as e:
            print(f"ERROR: Test execution failed: {e}")
            test_results["execution_error"] = str(e)
            
        finally:
            await browser.close()
    
    # Save results
    results_file = "test_complete_results.json"
    with open(results_file, "w") as f:
        json.dump(test_results, f, indent=2)
    
    # Print summary
    print("\n" + "=" * 60)
    print("COMPREHENSIVE TEST RESULTS")
    print("=" * 60)
    print(f"Test Date: {test_results['test_date']}")
    
    if test_results["formats_tested"]:
        print("\nTEST: Format Testing Results:")
        successful_formats = 0
        total_formats = 0
        
        for fmt, result in test_results["formats_tested"].items():
            total_formats += 1
            if result.get("success"):
                successful_formats += 1
                size = result.get("original_size_bytes", 0)
                compression = result.get("compression_ratio_percent")
                file_sizes_display = result.get("file_sizes_display")
                
                # Enhanced format display string with UI-extracted sizes
                display_parts = [f"{size:,}B"]
                
                if file_sizes_display:
                    display_parts.append(f"UI: {file_sizes_display}")
                elif compression:
                    display_parts.append(f"({compression:.1f}% compression)")
                
                print(f"  SUCCESS: {fmt}: {' '.join(display_parts)}")
            else:
                error = result.get("error", "Failed")
                print(f"  ERROR: {fmt}: {error}")
        
        success_rate = (successful_formats / total_formats) * 100 if total_formats > 0 else 0
        print(f"\nSTATS: Success Rate: {successful_formats}/{total_formats} ({success_rate:.1f}%)")
    
    if test_results["compression_results"]:
        print("\nTEST: Quality Testing Results:")
        for quality, result in test_results["compression_results"].items():
            download_available = result.get("download_available", False)
            compression = result.get("compression_ratio_percent")
            original_size = result.get("original_size_bytes", 0)
            file_sizes_display = result.get("file_sizes_display")
            
            status = "SUCCESS" if download_available else "FAIL"
            display_parts = [f"{original_size:,}B"]
            
            if file_sizes_display:
                display_parts.append(f"UI: {file_sizes_display}")
            elif compression:
                display_parts.append(f"({compression:.1f}% compression)")
            
            print(f"  {status} {quality}: {' '.join(display_parts)}")
    
    print(f"\nRESULTS: Results saved to: {results_file}")
    
    # Final console summary
    if 'console_logger' in locals():
        console_summary = console_logger.get_summary()
        test_results["final_console_summary"] = console_summary
        
        print(f"\nLOG: Complete Browser Console Summary:")
        print(f"   Total messages: {console_summary['total_messages']}")
        print(f"   Errors: {console_summary['error_count']} ERROR")
        print(f"   Warnings: {console_summary['warning_count']} WARN")
        print(f"   Info/Debug: {console_summary['info_count']} INFO")
        
        if console_logger.errors:
            print(f"   Recent errors:")
            for error in console_logger.errors[-3:]:  # Show last 3 errors
                print(f"     ERROR {error['timestamp']}: {error['message'][:100]}")
        
        if console_logger.warnings:
            print(f"   Recent warnings:")
            for warning in console_logger.warnings[-2:]:  # Show last 2 warnings
                print(f"     WARN {warning['timestamp']}: {warning['message'][:100]}")
    
    return test_results


if __name__ == "__main__":
    asyncio.run(run_comprehensive_test())


# Pytest
@pytest.mark.asyncio
@pytest.mark.browser_automation
async def test_image_optimization_comprehensive():
    """Pytest-compatible comprehensive image test."""
    return await run_comprehensive_test()
