#!/usr/bin/env python3
"""
Comprehensive mesh test that runs the full functionality suite with logging.
This test validates mesh optimization functionality for all supported mesh formats
including OBJ, STL, PLY, GLTF, GLB, and FBX files.

Run standalone: python test_mesh.py
Run via pytest: pytest test_mesh.py -v
"""

import asyncio
import json
import time
import re
from pathlib import Path
from datetime import datetime
from playwright.async_api import async_playwright
import pytest


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
        
        print(f"   WAIT: Waiting for result panel to appear...")
        
        for selector in result_selectors:
            try:
                await page.locator(selector).wait_for(state="visible", timeout=timeout_ms)
                print(f"   SUCCESS: Result panel found: {selector}")
                return True
            except:
                continue
        
        print(f"   WARN: No result panel found within {timeout_ms/1000}s")
        return False
        
    except Exception as e:
        print(f"   ERROR: Error waiting for result panel: {e}")
        return False


async def debug_ui_content(page) -> None:
    """Debug helper to understand what's actually in the UI."""
    try:
        print("\nDEBUG: UI Content Analysis")
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
        print(f"   ERROR: Debug failed: {e}")


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
            print(f"   PANEL: Results panel text: {panel_text[:200]}...")
            
            # Extract file size information using multiple patterns
            size_patterns = [
                # Pattern: "123 KB → 89 KB (-27.6%)" or "123 KB → 89 KB (+27.6%)"
                r'(\d+(?:\.\d+)?)\s*([KMGT]?B)\s*→\s*(\d+(?:\.\d+)?)\s*([KMGT]?B)\s*\(([+-]?\d+(?:\.\d+)?)%\)',
                # Pattern: "(+15.5%)" or "(-25.3%)"
                r'\(([+-]?\d+(?:\.\d+)?)%\)',
                # Pattern: "15.5% smaller" or "25.3% reduction"
                r'(\d+(?:\.\d+)?)%\s*(?:smaller|reduction|saved|compressed|optimized)',
            ]
            
            for pattern in size_patterns:
                matches = re.findall(pattern, panel_text, re.IGNORECASE)
                if matches:
                    print(f"   MATCH: Found matches with pattern: {pattern[:50]}...")
                    print(f"   Matches: {matches}")
                    
                    if len(matches[0]) == 5:  # Full size pattern match
                        orig_size, orig_unit, opt_size, opt_unit, percentage = matches[0]
                        results["original_size"] = f"{orig_size} {orig_unit}"
                        results["optimized_size"] = f"{opt_size} {opt_unit}"
                        results["compression_percentage"] = abs(float(percentage))
                        results["file_sizes_text"] = f"{orig_size} {orig_unit} → {opt_size} {opt_unit}"
                        print(f"   SUCCESS: Extracted: {results['file_sizes_text']} ({percentage}%)")
                        break
                    elif isinstance(matches[0], str):  # Simple percentage match
                        percentage = matches[0].replace('+', '').replace('-', '')
                        results["compression_percentage"] = abs(float(percentage))
                        print(f"   SUCCESS: Extracted compression: {percentage}%")
                        break
                    else:
                        percentage = matches[0]
                        results["compression_percentage"] = abs(float(percentage))
                        print(f"   SUCCESS: Extracted compression: {percentage}%")
                        break
        
        # Alternative: Look for summary section with total savings
        summary_section = page.locator('div:has-text("optimized"), div:has-text("file"), .bg-neutral-900')
        if await summary_section.count() > 0 and not results["compression_percentage"]:
            summary_text = await summary_section.first.text_content()
            print(f"   SUMMARY: Summary text: {summary_text}")
            
            # Look for percentage in summary
            summary_matches = re.findall(r'[(-](\d+(?:\.\d+)?)%[)]', summary_text)
            if summary_matches:
                results["compression_percentage"] = float(summary_matches[0])
                print(f"   SUCCESS: Found summary compression: {results['compression_percentage']}%")
        
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
        print(f"   ERROR: Error extracting metrics: {e}")
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
    
    for format_name, fixture_path, expected_optimization in test_cases:
        fixture_file = Path(fixture_path)
        if fixture_file.exists():
            available_cases.append((format_name, fixture_path, expected_optimization))
        else:
            missing_files.append((format_name, fixture_path))
    
    print(f"FILES: Test File Validation:")
    print(f"   SUCCESS: Available: {len(available_cases)} files")
    print(f"   MISSING: Missing: {len(missing_files)} files")
    
    if missing_files and len(missing_files) <= 5:  # Show first 5 missing files
        print(f"   NOTE: Missing files:")
        for format_name, path in missing_files[:5]:
            print(f"      - {format_name}: {path}")
    
    return available_cases


async def read_file_as_bytes(file):
    """Read a file object as bytes (Playwright compatibility)."""
    return await file.async_read() if hasattr(file, 'async_read') else file.read()


class ConsoleLogger:
    """Capture and log browser console messages during testing."""
    
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
        
        self.messages.append(log_entry)
        self.logs.append(log_entry)
        
        # Categorize messages
        if message_type == 'error':
            self.errors.append(log_entry)
        elif message_type == 'warning':
            self.warnings.append(log_entry)
            
        prefix = {
            'log': '[LOG]',
            'info': '[INF]',
            'warn': '[WRN]',
            'error': '[ERR]',
            'debug': '[DBG]'
        }.get(message_type, '')
        
        print(f"   {prefix} [{timestamp}] {message_type.upper()}: {message_text}")
        
        # Special handling for WASM-related messages
        if 'wasm' in message_text.lower() or 'pixie' in message_text.lower():
            print(f"      WASM/Pixie: {message_text}")
            
        # Special handling for optimization messages
        if any(keyword in message_text.lower() for keyword in ['optimization', 'compress', 'mesh', 'vertices']):
            print(f"      OPT: Optimization: {message_text}")
    
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
        print(f"   [{timestamp}] PAGE ERROR: {str(error)}")
    
    def print_summary(self):
        """Print a summary of console activity."""
        summary = self.get_summary()
        print(f"\nCONSOLE: Browser Console Summary:")
        print(f"   Total messages: {summary['total_messages']}")
        print(f"   Errors: {summary['error_count']}")
        print(f"   Warnings: {summary['warning_count']}")
        
        if summary['error_messages']:
            print(f"   Recent errors:")
            for error in summary['error_messages']:
                print(f"     ERROR: {error}")
                
        if summary['warning_messages']:
            print(f"   Recent warnings:")
            for warning in summary['warning_messages']:
                print(f"     WARN: {warning}")


async def setup_page_logging(page):
    """Set up comprehensive page logging including console, network, and errors."""
    console_logger = ConsoleLogger()
    
    # Console message logging
    page.on("console", console_logger.handle_console)
    
    # Page error logging
    page.on("pageerror", console_logger.handle_page_error)
    
    # Request/Response logging for WASM and API calls
    def handle_request(request):
        if any(keyword in request.url.lower() for keyword in ['wasm', 'pixie', 'optimize']):
            print(f"   NET: Request: {request.method} {request.url}")
    
    def handle_response(response):
        if any(keyword in response.url.lower() for keyword in ['wasm', 'pixie', 'optimize']):
            status_icon = 'OK' if response.ok else 'FAIL'
            print(f"   NET: Response: {status_icon} {response.status} {response.url}")
    
    page.on("request", handle_request)
    page.on("response", handle_response)
    
    return console_logger


async def run_comprehensive_mesh_test():
    """Run comprehensive mesh testing with all metrics and logging."""
    
    test_results = {
        "test_date": time.strftime("%Y-%m-%d %H:%M:%S"),
        "test_type": "mesh_optimization",
        "formats_tested": {},
        "performance_metrics": {},
        "optimization_results": {},
        "issues_found": []
    }
    
    print("TEST: Pixie Juice Comprehensive Mesh Testing Suite")
    print("=" * 60)
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=False, slow_mo=200)  # need to switch to headless for async tests sometime
        page = await browser.new_page()
        
        # Set up comprehensive logging
        print("SETUP: Setting up browser console logging...")
        console_logger = await setup_page_logging(page)
        
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
            
            # Test comprehensive mesh format coverage
            test_cases = [
                # OBJ formats - using generated fixtures
                ("OBJ_Cube", "tests/fixtures/meshes/obj/simple_cube.obj", 15),
                ("OBJ_Sphere_Medium", "tests/fixtures/meshes/obj/medium_sphere.obj", 20),
                ("OBJ_Sphere_Complex", "tests/fixtures/meshes/obj/complex_sphere.obj", 30),
                ("OBJ_Grid_Large", "tests/fixtures/meshes/obj/large_grid.obj", 40),
                
                # STL formats - using generated fixtures
                ("STL_ASCII_Simple", "tests/fixtures/meshes/stl/simple_ascii.stl", 10),
                ("STL_Binary_Simple", "tests/fixtures/meshes/stl/simple_binary.stl", 12),
                ("STL_Binary_Medium", "tests/fixtures/meshes/stl/medium_binary.stl", 18),
                ("STL_Binary_Large", "tests/fixtures/meshes/stl/large_binary.stl", 25),
                
                # PLY formats - using generated fixtures
                ("PLY_ASCII_Simple", "tests/fixtures/meshes/ply/simple_quad.ply", 15),
                ("PLY_Binary_Medium", "tests/fixtures/meshes/ply/medium_binary.ply", 20),
                ("PLY_Binary_Large", "tests/fixtures/meshes/ply/large_binary.ply", 30),
                
                # glTF formats - using generated fixtures
                ("GLTF_Simple", "tests/fixtures/meshes/gltf/simple_triangle.gltf", 20),
                
                # GLB formats - using generated fixtures
                ("GLB_Simple", "tests/fixtures/meshes/glb/simple_triangle.glb", 22),
                
                # FBX formats - using generated fixtures
                ("FBX_Simple", "tests/fixtures/meshes/fbx/simple_mesh.fbx", 25),
                ("FBX_Medium", "tests/fixtures/meshes/fbx/medium_mesh.fbx", 30),
                ("FBX_Complex", "tests/fixtures/meshes/fbx/complex_mesh.fbx", 40),
            ]
            
            # Validate test files and get available cases
            available_test_cases = await validate_test_files(test_cases)
            
            if not available_test_cases:
                print("ERROR: No mesh test files available!")
                return test_results
            
            print(f"\nTEST: Running mesh tests on {len(available_test_cases)} available files...")
            
            for format_name, fixture_path, expected_optimization in available_test_cases:
                fixture_file = Path(fixture_path)
                
                print(f"\nTEST: Testing {format_name} format...")
                print(f"FILE: {fixture_path}")
                
                # Reset console logger for this test
                console_logger.messages.clear()
                console_logger.errors.clear()
                console_logger.warnings.clear()
                
                try:
                    # Get original file size
                    original_size = fixture_file.stat().st_size
                    print(f"   SIZE: Original size: {original_size:,} bytes")
                    
                    # Upload file
                    print(f"   Uploading file...")
                    file_input = page.locator('input[type="file"]')
                    await file_input.set_input_files(str(fixture_file))
                    await page.wait_for_timeout(1000)
                    
                    # Log any immediate console activity after file upload
                    if console_logger.messages:
                        print(f"   LOG: Console activity after upload: {len(console_logger.messages)} messages")
                    
                    # Set quality (for meshes this becomes target ratio)
                    quality_slider = page.locator('input[type="range"]')
                    if await quality_slider.count() > 0:
                        print(f"   SET: Setting quality to 75%...")
                        await quality_slider.fill('75')  # 75% quality = 0.25 reduction ratio
                        await page.wait_for_timeout(300)
                    
                    # Click optimize
                    optimize_btn = page.locator('button:has-text("Optimize")')
                    if await optimize_btn.count() > 0:
                        print("   START: Starting mesh optimization...")
                        
                        # Clear console before optimization
                        optimization_start_messages = len(console_logger.messages)
                        
                        await optimize_btn.click()
                        
                        # Monitor console during optimization
                        print("   Monitoring console during optimization...")
                        
                        # Wait for result panel to appear (reliable completion indicator)
                        result_panel_visible = await wait_for_result_panel(page, 30000)  # 30 seconds for mesh processing
                        
                        # Log optimization console activity
                        optimization_messages = len(console_logger.messages) - optimization_start_messages
                        print(f"   LOG: Console activity during optimization: {optimization_messages} new messages")
                        
                        if not result_panel_visible:
                            # Debug UI content when result panel is not found
                            print("   DEBUG: Result panel not found, debugging UI content...")
                            await debug_ui_content(page)
                        
                        # Extract metrics from result panel
                        metrics = await extract_metrics_from_result_panel(page)
                        
                        optimization_ratio = metrics.get("compression_percentage")
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
                            "optimization_ratio_percent": optimization_ratio,
                            "result_panel_visible": result_panel_visible,
                            "download_available": download_available,
                            "quality_setting": 75,
                            "original_size_text": original_size_text,
                            "optimized_size_text": optimized_size_text,
                            "file_sizes_display": file_sizes_text,
                            "console_summary": console_logger.get_summary()
                        }
                        
                        # Enhanced logging with file size details
                        print(f"   Original size: {original_size:,} bytes")
                        
                        if file_sizes_text:
                            print(f"   UI shows: {file_sizes_text}")
                        elif original_size_text and optimized_size_text:
                            print(f"   UI shows: {original_size_text} -> {optimized_size_text}")
                        
                        if optimization_ratio:
                            print(f"   OPT: Optimization: {optimization_ratio:.1f}% (expected: >={expected_optimization}%) {'SUCCESS' if optimization_ratio >= expected_optimization else 'FAIL'}")
                        else:
                            print(f"   WARN: Could not extract optimization ratio from UI")
                            
                        print(f"   DOWNLOAD: Downloads: {'SUCCESS' if download_available else 'FAIL'}")
                        print(f"   PANEL: Result panel: {'SUCCESS' if result_panel_visible else 'FAIL'}")
                        print(f"   RESULT: Overall: {'SUCCESS' if success else 'FAILED'}")
                        
                        # Print console summary for this test
                        console_logger.print_summary()
                        
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
            
            # Test optimization levels with simple OBJ cube
            cube_obj = Path("fixtures/meshes/cube.obj")
            if cube_obj.exists():
                print(f"\nTEST: Testing Optimization Levels...")
                
                for quality in [25, 50, 75, 90]:
                    try:
                        print(f"\n   QUALITY: Testing Quality {quality}% (mesh reduction ratio)...")
                        original_size = cube_obj.stat().st_size
                        
                        # Reset console logger for this quality test
                        console_logger.messages.clear()
                        console_logger.errors.clear()
                        console_logger.warnings.clear()
                        
                        # Upload file
                        print(f"      Uploading {cube_obj.name}...")
                        file_input = page.locator('input[type="file"]')
                        await file_input.set_input_files(str(cube_obj))
                        await page.wait_for_timeout(1000)
                        
                        # Set quality (for meshes: higher quality = less reduction)
                        quality_slider = page.locator('input[type="range"]')
                        if await quality_slider.count() > 0:
                            print(f"      SET: Setting quality to {quality}%...")
                            await quality_slider.fill(str(quality))
                            await page.wait_for_timeout(300)
                        
                        # Optimize
                        optimize_btn = page.locator('button:has-text("Optimize")')
                        if await optimize_btn.count() > 0:
                            print(f"      START: Starting mesh optimization at quality {quality}%...")
                            
                            # Track console activity during optimization
                            optimization_start_messages = len(console_logger.messages)
                            
                            await optimize_btn.click()
                            
                            print("      Monitoring console during optimization...")
                            
                            # Wait for result panel to appear (reliable completion indicator)
                            result_panel_visible = await wait_for_result_panel(page, 25000)  # 25 seconds for mesh
                            
                            # Log optimization console activity
                            optimization_messages = len(console_logger.messages) - optimization_start_messages
                            print(f"      LOG: Console activity: {optimization_messages} messages")
                            
                            # Extract metrics from result panel
                            metrics = await extract_metrics_from_result_panel(page)
                            
                            optimization_ratio = metrics.get("compression_percentage")
                            download_available = metrics.get("download_available", False)
                            original_size_text = metrics.get("original_size")
                            optimized_size_text = metrics.get("optimized_size")
                            file_sizes_text = metrics.get("file_sizes_text")
                            
                            # Store quality results with enhanced metrics
                            test_results["optimization_results"][f"Quality_{quality}%"] = {
                                "quality_setting": quality,
                                "original_size_bytes": original_size,
                                "optimization_ratio_percent": optimization_ratio,
                                "result_panel_visible": result_panel_visible,
                                "download_available": download_available,
                                "original_size_text": original_size_text,
                                "optimized_size_text": optimized_size_text,
                                "file_sizes_display": file_sizes_text,
                                "console_summary": console_logger.get_summary()
                            }
                            
                            # Enhanced logging with file size details
                            print(f"      Original size: {original_size:,} bytes")
                            
                            if file_sizes_text:
                                print(f"      UI shows: {file_sizes_text}")
                            elif original_size_text and optimized_size_text:
                                print(f"      UI shows: {original_size_text} -> {optimized_size_text}")
                            
                            if optimization_ratio:
                                print(f"      Optimization: {optimization_ratio:.1f}%")
                            else:
                                print(f"      WARN: Could not extract optimization ratio from UI")
                            
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
    results_file = "/tests/test_mesh_results.json"
    with open(results_file, "w") as f:
        json.dump(test_results, f, indent=2)
    
    # Print summary
    print("\n" + "=" * 60)
    print("COMPREHENSIVE MESH TEST RESULTS")
    print("=" * 60)
    print(f"Test Date: {test_results['test_date']}")
    print(f"Test Type: {test_results['test_type']}")
    
    if test_results["formats_tested"]:
        print("\nRESULTS: Mesh Format Testing Results:")
        successful_formats = 0
        total_formats = 0
        
        for fmt, result in test_results["formats_tested"].items():
            total_formats += 1
            if result.get("success"):
                successful_formats += 1
                size = result.get("original_size_bytes", 0)
                optimization = result.get("optimization_ratio_percent")
                file_sizes_display = result.get("file_sizes_display")
                
                # Enhanced format display string with UI-extracted sizes
                display_parts = [f"{size:,}B"]
                
                if file_sizes_display:
                    display_parts.append(f"UI: {file_sizes_display}")
                elif optimization:
                    display_parts.append(f"({optimization:.1f}% optimization)")
                
                print(f"  SUCCESS: {fmt}: {' '.join(display_parts)}")
            else:
                error = result.get("error", "Failed")
                print(f"  ERROR: {fmt}: {error}")
        
        success_rate = (successful_formats / total_formats) * 100 if total_formats > 0 else 0
        print(f"\nSTATS: Success Rate: {successful_formats}/{total_formats} ({success_rate:.1f}%)")
    
    if test_results["optimization_results"]:
        print("\nQUALITY: Quality Testing Results:")
        for quality, result in test_results["optimization_results"].items():
            download_available = result.get("download_available", False)
            optimization = result.get("optimization_ratio_percent")
            original_size = result.get("original_size_bytes", 0)
            file_sizes_display = result.get("file_sizes_display")
            
            status = "SUCCESS" if download_available else "FAIL"
            display_parts = [f"{original_size:,}B"]
            
            if file_sizes_display:
                display_parts.append(f"UI: {file_sizes_display}")
            elif optimization:
                display_parts.append(f"({optimization:.1f}% optimization)")
            
            print(f"  {status} {quality}: {' '.join(display_parts)}")
    
    print(f"\nResults saved to: {results_file}")
    
    # Final console summary
    if console_logger:
        console_summary = console_logger.get_summary()
        test_results["final_console_summary"] = console_summary
        
        print(f"\nCONSOLE: Complete Browser Console Summary:")
        print(f"   Total messages: {console_summary['total_messages']}")
        print(f"   Errors: {console_summary['error_count']} ERROR")
        print(f"   Warnings: {console_summary['warning_count']} WARN")
        print(f"   Info/Debug: {console_summary['info_count']} INFO")
        
        if console_logger.errors:
            print(f"   Recent errors:")
            for error in console_logger.errors[-3:]:  # Show last 3 errors
                print(f"     {error['timestamp']}: {error['message'][:100]}")
        
        if console_logger.warnings:
            print(f"   Recent warnings:")
            for warning in console_logger.warnings[-2:]:  # Show last 2 warnings
                print(f"     {warning['timestamp']}: {warning['message'][:100]}")
    
    return test_results


if __name__ == "__main__":
    asyncio.run(run_comprehensive_mesh_test())


# Pytest
@pytest.mark.asyncio
@pytest.mark.browser_automation
async def test_mesh_optimization_comprehensive():
    """Pytest-compatible comprehensive mesh test."""
    return await run_comprehensive_mesh_test()
