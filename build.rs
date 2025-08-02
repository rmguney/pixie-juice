use std::env;
use std::path::PathBuf;

fn main() {
    // Declare the custom cfg option
    println!("cargo::rustc-check-cfg=cfg(c_hotspots_available)");
    
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.contains("wasm32");
    
    // Check if c_hotspots feature is enabled
    let c_hotspots_enabled = std::env::var("CARGO_FEATURE_C_HOTSPOTS").is_ok();
    
    // Force disable C hotspots if explicitly requested
    let force_disable = std::env::var("PIXIE_DISABLE_C_HOTSPOTS").is_ok();
    
    if force_disable {
        println!("cargo:warning=C hotspots disabled by PIXIE_DISABLE_C_HOTSPOTS flag");
        create_fallback_bindings();
    } else if !c_hotspots_enabled {
        println!("cargo:warning=C hotspots feature not enabled, using Rust-only implementation");
        println!("cargo:warning=Enable with --features c_hotspots after Rust implementations are working");
        create_fallback_bindings();
    } else {
        // Try to compile C hotspots for both native and WASM targets
        println!("cargo:warning=Target detected: {} - compiling C hotspots", if is_wasm { "WASM" } else { "native" });
        match compile_c_hotspots() {
            Ok(_) => {
                println!("cargo:warning=C hotspots compiled successfully for {} target", if is_wasm { "WASM" } else { "native" });
                println!("cargo:rustc-cfg=c_hotspots_available");
            }
            Err(e) => {
                println!("cargo:warning=C hotspots compilation failed: {}", e);
                println!("cargo:warning=Falling back to Rust-only implementation");
                create_fallback_bindings();
            }
        }
    }
}

fn create_fallback_bindings() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(
        out_path.join("bindings.rs"),
        "// C optimizer disabled or compilation failed\n"
    ).expect("Unable to write bindings file");
}

fn compile_c_hotspots() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:warning=Compiling C hotspots for performance optimization");
    
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.contains("wasm32");
    
    // For WASM, check if clang is available
    if is_wasm {
        let clang_path = find_clang()?;
        println!("cargo:warning=Found clang for WASM compilation at: {}", clang_path);
    }
    
    let hotspots_dir = "hotspots";
    let hotspots_src_dir = "hotspots/src";
    let hotspots_include_dir = "hotspots/include";
    
    // Check if hotspots directories exist
    if !std::path::Path::new(hotspots_dir).exists() {
        return Err("Hotspots directory not found".into());
    }
    if !std::path::Path::new(hotspots_src_dir).exists() {
        return Err("Hotspots src directory not found".into());
    }
    if !std::path::Path::new(hotspots_include_dir).exists() {
        return Err("Hotspots include directory not found".into());
    }
    
    // Only include C hotspots that exist and provide >15% performance improvement
    let c_files = [
        "util.c",          // Utility functions
        "memory.c",        // SIMD memory operations
        "math_kernel.c",   // SIMD math operations
        "image_kernel.c",  // Image processing kernels (quantization, dithering)
        "compress.c",      // Compression algorithms
        "mesh_decimate.c", // QEM mesh decimation
    ];
    
    // Check if files exist
    for file in &c_files {
        let path = format!("{}/{}", hotspots_src_dir, file);
        if !std::path::Path::new(&path).exists() {
            return Err(format!("C file not found: {}", path).into());
        }
    }
    
    // Compile C files
    let mut build = cc::Build::new();
    
    // Add files
    for file in &c_files {
        build.file(format!("{}/{}", hotspots_src_dir, file));
    }
    
    build
        .include(hotspots_include_dir)
        .include(hotspots_src_dir)
        .opt_level(2)
        .debug(false)
        .warnings(false)
        .std("c11"); // Use C11 standard for threading and other features
    
    // Handle WASM target specific compilation
    if is_wasm {
        let clang_path = find_clang()?;
        println!("cargo:warning=WASM target detected - configuring clang for WASM compilation with maximum optimization");
        // For WASM, use clang with proper WASM flags for maximum performance
        build.compiler(&clang_path);
        build.flag("--target=wasm32-unknown-unknown");
        build.flag("-O3"); // Maximum optimization for performance
        build.flag("-flto"); // Link time optimization
        build.flag("-fno-builtin");
        build.flag("-nostdlib");
        build.flag("-Wno-unused-parameter");
        build.flag("-Wno-unused-variable");
        build.flag("-msimd128"); // Enable WASM SIMD
        build.flag("-mbulk-memory"); // Enable bulk memory operations  
        build.flag("-mmutable-globals"); // Enable mutable globals
        build.flag("-msign-ext"); // Enable sign extension
        build.flag("-mnontrapping-fptoint"); // Enable non-trapping float to int
        build.define("__wasm32__", None);
        build.define("__wasm__", None);
        build.define("WASM_TARGET", None);
        build.define("NDEBUG", None); // Release mode
        
        // Add WASM-specific memory management symbols
        println!("cargo:rustc-link-arg=--export=wasm_malloc");
        println!("cargo:rustc-link-arg=--export=wasm_free");
        println!("cargo:rustc-link-arg=--export=wasm_memcpy");
        println!("cargo:rustc-link-arg=--export=wasm_memset");
        println!("cargo:rustc-link-arg=--export=wasm_memcmp");
        
        // WASM optimization flags
        println!("cargo:rustc-link-arg=--lto-O3"); // Link time optimization
        println!("cargo:rustc-link-arg=--no-demangle"); // Smaller binary
        println!("cargo:rustc-link-arg=--strip-debug"); // Remove debug info
    } else {
        // Native compilation with optimizations
        // Use more conservative flags that work across compilers
        build.flag("-O3"); // Maximum optimization
        build.flag("-flto"); // Link time optimization when available
        
        // Only use GCC/Clang specific flags on non-Windows or if we detect GCC/Clang
        if !cfg!(target_os = "windows") {
            build.flag("-march=native");
            build.flag("-mtune=native");
        }
    }
    
    println!("cargo:warning=Compiling C files: {:?}", c_files);
    build.compile("pixie_hotspots");
    println!("cargo:warning=C hotspots compiled successfully!");
    
    // Create manual bindings instead of using bindgen for now
    // This avoids the header include issues on Windows
    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    std::fs::write(
        out_path.join("bindings.rs"),
        r#"
// Manual C hotspot bindings for existing performance-critical functions only
#[allow(non_camel_case_types)]

// Color structure for image processing
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// Quantized image result structure
#[repr(C)]
#[derive(Debug)]
pub struct QuantizedImage {
    pub palette: *mut Color32,
    pub palette_size: usize,
    pub indices: *mut u8,
    pub width: usize,
    pub height: usize,
}

// Mesh decimation result structure to match C header
#[repr(C)]
#[derive(Debug)]
pub struct MeshDecimateResult {
    pub vertices: *mut f32,
    pub indices: *mut u32,
    pub vertex_count: usize,
    pub index_count: usize,
    pub success: i32,
    pub error_message: [u8; 256],
}

extern "C" {
    // From util.h - Basic utilities
    pub fn buffer_create(initial_capacity: usize) -> *mut core::ffi::c_void;
    pub fn buffer_destroy(buffer: *mut core::ffi::c_void);
    pub fn buffer_append(buffer: *mut core::ffi::c_void, data: *const u8, size: usize) -> i32;
    
    // From memory.h - SIMD memory operations
    pub fn memcpy_simd(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, size: usize);
    pub fn memset_simd(dest: *mut core::ffi::c_void, value: i32, size: usize);
    pub fn memory_pool_create(initial_size: usize) -> *mut core::ffi::c_void;
    pub fn memory_pool_destroy(pool: *mut core::ffi::c_void);
    pub fn memory_pool_alloc(pool: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void;
    
    // From math_kernel.h - SIMD math operations
    pub fn vector_dot_product_simd(a: *const f32, b: *const f32, size: usize) -> f32;
    pub fn matrix_multiply_simd(a: *const f32, b: *const f32, result: *mut f32, 
                                m: i32, n: i32, k: i32);
    pub fn gaussian_blur_simd(image: *mut u8, width: i32, height: i32, 
                              channels: i32, sigma: f32);
    
    // From image_kernel.h - Image processing kernels
    pub fn quantize_colors_octree(rgba_data: *const u8, width: usize, height: usize, 
                                  max_colors: usize) -> *mut QuantizedImage;
    pub fn quantize_colors_median_cut(rgba_data: *const u8, width: usize, height: usize, 
                                      max_colors: usize) -> *mut QuantizedImage;
    pub fn apply_floyd_steinberg_dither(rgba_data: *mut u8, width: usize, height: usize,
                                        palette: *const Color32, palette_size: usize);
    pub fn apply_ordered_dither(rgba_data: *mut u8, width: usize, height: usize,
                                palette: *const Color32, palette_size: usize, matrix_size: i32);
    pub fn apply_gaussian_blur(rgba_data: *mut u8, width: usize, height: usize, sigma: f32);
    pub fn apply_sharpen_filter(rgba_data: *mut u8, width: usize, height: usize, strength: f32);
    pub fn apply_edge_detection(rgba_data: *mut u8, width: usize, height: usize, output: *mut u8);
    pub fn rgb_to_yuv(rgb: *const u8, yuv: *mut u8, pixel_count: usize);
    pub fn yuv_to_rgb(yuv: *const u8, rgb: *mut u8, pixel_count: usize);
    pub fn rgb_to_lab(rgb: *const u8, lab: *mut f32, pixel_count: usize);
    pub fn lab_to_rgb(lab: *const f32, rgb: *mut u8, pixel_count: usize);
    pub fn free_quantized_image(img: *mut QuantizedImage);
    
    // From mesh_decimate.h - QEM mesh decimation
    pub fn decimate_mesh_qem(vertices: *const f32, vertex_count: usize,
                             indices: *const u32, index_count: usize,
                             target_ratio: f32) -> MeshDecimateResult;
    pub fn weld_vertices_spatial(vertices: *const f32, vertex_count: usize,
                                 indices: *const u32, index_count: usize,
                                 tolerance: f32) -> MeshDecimateResult;
    pub fn free_mesh_decimate_result(result: *mut MeshDecimateResult);
    
    // From compress.h - Compression kernels  
    pub fn compress_lz4(input: *const u8, input_size: usize, output: *mut u8, 
                        max_output_size: usize) -> i32;
    pub fn decompress_lz4(input: *const u8, input_size: usize, output: *mut u8, 
                           output_size: usize) -> i32;
}
"#,
    )?;
    
    println!("cargo:warning=Manual FFI bindings created successfully!");
    
    // Tell cargo to invalidate the built crate whenever C files change
    println!("cargo:rerun-if-changed=../hotspots/");
    
    Ok(())
}

fn find_clang() -> Result<String, Box<dyn std::error::Error>> {
    // First try the PATH
    if let Ok(output) = std::process::Command::new("clang").arg("--version").output() {
        if output.status.success() {
            return Ok("clang".to_string());
        }
    }
    
    // On Windows, check common installation paths
    if cfg!(target_os = "windows") {
        let common_paths = [
            r"C:\Program Files\LLVM\bin\clang.exe",
            r"C:\Program Files (x86)\LLVM\bin\clang.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\Llvm\x64\bin\clang.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\Llvm\x64\bin\clang.exe",
        ];
        
        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                // Test that it actually works
                if let Ok(output) = std::process::Command::new(path).arg("--version").output() {
                    if output.status.success() {
                        return Ok(path.to_string());
                    }
                }
            }
        }
    }
    
    Err("Clang not found for WASM compilation".into())
}

#[cfg(not(target_os = "windows"))]
fn create_empty_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::write(
        out_path.join("bindings.rs"),
        "// C optimizer disabled on this platform\n"
    ).expect("Unable to write bindings file");
}
