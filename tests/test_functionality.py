#!/usr/bin/env python3
"""
Comprehensive test that runs the full functionality suite with logging
"""

import asyncio
import json
import time
import re
from pathlib import Path
from playwright.async_api import async_playwright


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
        
        print(f"   ⏳ Waiting for result panel to appear...")
        
        for selector in result_selectors:
            try:
                await page.locator(selector).wait_for(state="visible", timeout=timeout_ms)
                print(f"   ✅ Result panel found: {selector}")
                return True
            except:
                continue
        
        print(f"   ⚠️  No result panel found within {timeout_ms/1000}s")
        return False
        
    except Exception as e:
        print(f"   ❌ Error waiting for result panel: {e}")
        return False


async def debug_ui_content(page) -> None:
    """Debug helper to understand what's actually in the UI."""
    try:
        print("\n🔍 DEBUG: UI Content Analysis")
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
        print(f"   ❌ Debug failed: {e}")


async def extract_metrics_from_result_panel(page) -> dict:
    """Extract file size reduction metrics from the visible result panel."""
    try:
        # Wait for content to load and become stable
        await page.wait_for_timeout(1000)
        
        print(f"   🔍 Extracting metrics from result panel...")
        
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
            print(f"   ✅ Download available: {download_links} links, {download_buttons} buttons")
        
        # Look for file size information in the results panel
        # The UI shows: "123 KB → 89 KB (-27.6%)"
        results_panel = page.locator('.space-y-2, [class*="result"], .border:has(button:has-text("Download"))')
        
        if await results_panel.count() > 0:
            panel_text = await results_panel.first.text_content()
            print(f"   📊 Results panel text: {panel_text[:200]}...")
            
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
                    print(f"   🔍 Found matches with pattern: {pattern[:50]}...")
                    print(f"   📋 Matches: {matches}")
                    
                    if len(matches[0]) == 5:  # Full size pattern match
                        orig_size, orig_unit, opt_size, opt_unit, percentage = matches[0]
                        results["original_size"] = f"{orig_size} {orig_unit}"
                        results["optimized_size"] = f"{opt_size} {opt_unit}"
                        results["compression_percentage"] = abs(float(percentage))
                        results["file_sizes_text"] = f"{orig_size} {orig_unit} → {opt_size} {opt_unit}"
                        print(f"   ✅ Extracted: {results['file_sizes_text']} ({percentage}%)")
                        break
                    elif isinstance(matches[0], str):  # Simple percentage match
                        percentage = matches[0].replace('+', '').replace('-', '')
                        results["compression_percentage"] = abs(float(percentage))
                        print(f"   ✅ Extracted compression: {percentage}%")
                        break
                    else:
                        percentage = matches[0]
                        results["compression_percentage"] = abs(float(percentage))
                        print(f"   ✅ Extracted compression: {percentage}%")
                        break
        
        # Alternative: Look for summary section with total savings
        summary_section = page.locator('div:has-text("optimized"), div:has-text("file"), .bg-neutral-900')
        if await summary_section.count() > 0 and not results["compression_percentage"]:
            summary_text = await summary_section.first.text_content()
            print(f"   � Summary text: {summary_text}")
            
            # Look for percentage in summary
            summary_matches = re.findall(r'[(-](\d+(?:\.\d+)?)%[)]', summary_text)
            if summary_matches:
                results["compression_percentage"] = float(summary_matches[0])
                print(f"   ✅ Found summary compression: {results['compression_percentage']}%")
        
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
                        print(f"   ⚠️  Fallback compression detected: {pct_val}%")
                        break
        
        return results
        
    except Exception as e:
        print(f"   ❌ Error extracting metrics: {e}")
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
    
    print(f"📋 Test File Validation:")
    print(f"   ✅ Available: {len(available_cases)} files")
    print(f"   ❌ Missing: {len(missing_files)} files")
    
    if missing_files and len(missing_files) <= 5:  # Show first 5 missing files
        print(f"   📝 Missing files:")
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
    
    print("🧪 Pixie Juice Comprehensive Testing Suite")
    print("=" * 60)
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=False, slow_mo=200)  # Slower for stability
        page = await browser.new_page()
        
        # Set longer timeout for stability
        page.set_default_timeout(30000)  # 30 seconds
        
        try:
            # Navigate to app
            print("🌐 Loading application...")
            await page.goto("http://localhost:3000")
            await page.wait_for_timeout(5000)  # Longer initial wait
            
            print("🔍 Checking if application loaded properly...")
            # Verify the page loaded
            title = await page.title()
            print(f"   📄 Page title: {title}")
            
            # Look for file input to confirm app is ready
            file_input = page.locator('input[type="file"]')
            if await file_input.count() > 0:
                print("   ✅ File input found - app is ready")
            else:
                print("   ⚠️  No file input found - app may not be ready")
                # Wait a bit more
                await page.wait_for_timeout(3000)
            
            # Test comprehensive format coverage with different sizes
            test_cases = [
                # PNG formats
                ("PNG_Small", "fixtures/images/small_png.png", 15),
                ("PNG_Medium", "fixtures/images/medium_png.png", 20),
                ("PNG_Large", "fixtures/images/large_png.png", 25),
                ("PNG_Lossless_Small", "fixtures/images/small_png_lossless.png", 10),
                ("PNG_Lossless_Medium", "fixtures/images/medium_png_lossless.png", 15),
                ("PNG_Lossless_Large", "fixtures/images/large_png_lossless.png", 20),
                ("PNG_Ultra", "fixtures/images/ultra_png.png", 30),
                
                # JPEG formats
                ("JPEG_Small", "fixtures/images/small_jpeg.jpg", 10),
                ("JPEG_Medium", "fixtures/images/medium_jpeg.jpg", 15),
                ("JPEG_Large", "fixtures/images/large_jpeg.jpg", 20),
                ("JPG_Small", "fixtures/images/small_jpg.jpg", 10),
                ("JPG_Medium", "fixtures/images/medium_jpg.jpg", 15),
                ("JPG_Large", "fixtures/images/large_jpg.jpg", 20),
                ("JPG_Low_Small", "fixtures/images/small_jpg_low.jpg", 5),
                ("JPG_Low_Medium", "fixtures/images/medium_jpg_low.jpg", 8),
                ("JPG_Low_Large", "fixtures/images/large_jpg_low.jpg", 10),
                ("JPG_Ultra", "fixtures/images/ultra_jpg.jpg", 25),
                
                # WebP formats
                ("WebP_Small", "fixtures/images/small_webp.webp", 15),
                ("WebP_Medium", "fixtures/images/medium_webp.webp", 20),
                ("WebP_Large", "fixtures/images/large_webp.webp", 25),
                ("WebP_Lossless_Small", "fixtures/images/small_webp_lossless.webp", 10),
                ("WebP_Lossless_Medium", "fixtures/images/medium_webp_lossless.webp", 15),
                ("WebP_Lossless_Large", "fixtures/images/large_webp_lossless.webp", 20),
                ("WebP_Low_Small", "fixtures/images/small_webp_low.webp", 5),
                ("WebP_Low_Medium", "fixtures/images/medium_webp_low.webp", 8),
                ("WebP_Low_Large", "fixtures/images/large_webp_low.webp", 10),
                ("WebP_Ultra", "fixtures/images/ultra_webp.webp", 30),
                ("WebP_Test", "fixtures/images/test.webp", 15),
                
                # GIF formats
                ("GIF_Test", "fixtures/images/test.gif", 20),
                ("GIF_Animated", "fixtures/images/animated.gif", 25),
                ("GIF_Animated_Test", "fixtures/images/animated_test.gif", 20),
                
                # BMP formats
                ("BMP_Small", "fixtures/images/small_bmp.bmp", 30),
                ("BMP_Medium", "fixtures/images/medium_bmp.bmp", 35),
                ("BMP_Large", "fixtures/images/large_bmp.bmp", 40),
                
                # TIFF formats
                ("TIFF_Small", "fixtures/images/small_tiff.tiff", 25),
                ("TIFF_Medium", "fixtures/images/medium_tiff.tiff", 30),
                ("TIFF_Large", "fixtures/images/large_tiff.tiff", 35),
                ("TIF_Small", "fixtures/images/small_tif.tiff", 25),
                ("TIF_Medium", "fixtures/images/medium_tif.tiff", 30),
                ("TIF_Large", "fixtures/images/large_tif.tiff", 35),
                
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
                print("❌ No test files available!")
                return test_results
            
            print(f"\n🎯 Running tests on {len(available_test_cases)} available files...")
            
            for format_name, fixture_path, expected_compression in available_test_cases:
                fixture_file = Path(fixture_path)
                
                print(f"\n🎯 Testing {format_name} format...")
                
                try:
                    # Get original file size
                    original_size = fixture_file.stat().st_size
                    print(f"   📊 Original size: {original_size:,} bytes")
                    
                    # Upload file
                    file_input = page.locator('input[type="file"]')
                    await file_input.set_input_files(str(fixture_file))
                    await page.wait_for_timeout(1000)  # Reduced from 2000ms
                    
                    # Set quality
                    quality_slider = page.locator('input[type="range"]')
                    if await quality_slider.count() > 0:
                        await quality_slider.fill('75')
                        await page.wait_for_timeout(300)  # Reduced from 500ms
                    
                    # Click optimize
                    optimize_btn = page.locator('button:has-text("Optimize")')
                    if await optimize_btn.count() > 0:
                        print("   🚀 Starting optimization...")
                        await optimize_btn.click()
                        
                        # Wait for result panel to appear (reliable completion indicator)
                        result_panel_visible = await wait_for_result_panel(page, 20000)  # 20 seconds max
                        
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
                            "file_sizes_display": file_sizes_text
                        }
                        
                        # Enhanced logging with file size details
                        print(f"   📁 Original size: {original_size:,} bytes")
                        
                        if file_sizes_text:
                            print(f"   � UI shows: {file_sizes_text}")
                        elif original_size_text and optimized_size_text:
                            print(f"   📊 UI shows: {original_size_text} → {optimized_size_text}")
                        
                        if compression_ratio:
                            print(f"   � Compression: {compression_ratio:.1f}% (expected: ≥{expected_compression}%) {'✅' if compression_ratio >= expected_compression else '❌'}")
                        else:
                            print(f"   ⚠️  Could not extract compression ratio from UI")
                            
                        print(f"   📥 Downloads: {'✅' if download_available else '❌'}")
                        print(f"   🔗 Result panel: {'✅' if result_panel_visible else '❌'}")
                        print(f"   🎯 Overall: {'✅ SUCCESS' if success else '❌ FAILED'}")
                        
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
                        print(f"   ❌ {format_name}: No optimize button found")
                        test_results["formats_tested"][format_name] = {
                            "success": False,
                            "error": "No optimize button",
                            "original_size_bytes": original_size
                        }
                
                except Exception as e:
                    print(f"   ❌ {format_name}: Error - {e}")
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
                print(f"\n🎯 Testing Quality Settings...")
                
                for quality in [25, 50, 75, 90]:
                    try:
                        print(f"\n   📊 Testing Quality {quality}%...")
                        original_size = medium_jpeg.stat().st_size
                        
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
                            print(f"      🚀 Starting optimization at quality {quality}%...")
                            await optimize_btn.click()
                            
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
                                "file_sizes_display": file_sizes_text
                            }
                            
                            # Enhanced logging with file size details
                            print(f"      📁 Original size: {original_size:,} bytes")
                            
                            if file_sizes_text:
                                print(f"      � UI shows: {file_sizes_text}")
                            elif original_size_text and optimized_size_text:
                                print(f"      📊 UI shows: {original_size_text} → {optimized_size_text}")
                            
                            if compression_ratio:
                                print(f"      � Compression: {compression_ratio:.1f}%")
                            else:
                                print(f"      ⚠️  Could not extract compression ratio from UI")
                            
                            print(f"      📥 Downloads: {'✅' if download_available else '❌'}")
                            print(f"      🔗 Result panel: {'✅' if result_panel_visible else '❌'}")
                            print(f"      🎯 Quality {quality}%: {'✅ SUCCESS' if download_available else '❌ FAILED'}")
                            
                            # Reset
                            await page.reload()
                            await page.wait_for_timeout(2000)
                    
                    except Exception as e:
                        print(f"      ❌ Quality {quality}% error: {e}")
                        await page.reload()
                        await page.wait_for_timeout(2000)
            
        except Exception as e:
            print(f"❌ Test execution failed: {e}")
            test_results["execution_error"] = str(e)
            
        finally:
            await browser.close()
    
    # Save results
    results_file = "test_complete_results.json"
    with open(results_file, "w") as f:
        json.dump(test_results, f, indent=2)
    
    # Print summary
    print("\n" + "=" * 60)
    print("📋 COMPREHENSIVE TEST RESULTS")
    print("=" * 60)
    print(f"Test Date: {test_results['test_date']}")
    
    if test_results["formats_tested"]:
        print("\n🎯 Format Testing Results:")
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
                
                print(f"  ✅ {fmt}: {' '.join(display_parts)}")
            else:
                error = result.get("error", "Failed")
                print(f"  ❌ {fmt}: {error}")
        
        success_rate = (successful_formats / total_formats) * 100 if total_formats > 0 else 0
        print(f"\n📊 Success Rate: {successful_formats}/{total_formats} ({success_rate:.1f}%)")
    
    if test_results["compression_results"]:
        print("\n🎯 Quality Testing Results:")
        for quality, result in test_results["compression_results"].items():
            download_available = result.get("download_available", False)
            compression = result.get("compression_ratio_percent")
            original_size = result.get("original_size_bytes", 0)
            file_sizes_display = result.get("file_sizes_display")
            
            status = "✅" if download_available else "❌"
            display_parts = [f"{original_size:,}B"]
            
            if file_sizes_display:
                display_parts.append(f"UI: {file_sizes_display}")
            elif compression:
                display_parts.append(f"({compression:.1f}% compression)")
            
            print(f"  {status} {quality}: {' '.join(display_parts)}")
    
    print(f"\n💾 Results saved to: {results_file}")
    
    return test_results


if __name__ == "__main__":
    asyncio.run(run_comprehensive_test())
