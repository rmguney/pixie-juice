use std::env;
use std::path::PathBuf;

fn main() {
    // Declare the custom cfg option
    println!("cargo::rustc-check-cfg=cfg(c_hotspots_available)");
    
    let target = env::var("TARGET").unwrap_or_default();
    
    // WASM-only targeting - no native compilation support
    if !target.contains("wasm32") {
        println!("cargo:warning=WASM-only targeting: native compilation not supported");
        println!("cargo:warning=Use --target=wasm32-unknown-unknown for WASM builds");
        create_fallback_bindings();
        return;
    }
    
    // Check if c_hotspots feature is enabled
    let c_hotspots_enabled = std::env::var("CARGO_FEATURE_C_HOTSPOTS").is_ok();
    
    // Force disable C hotspots if explicitly requested
    let force_disable = std::env::var("PIXIE_DISABLE_C_HOTSPOTS").is_ok();
    
    if force_disable {
        println!("cargo:warning=C hotspots disabled by PIXIE_DISABLE_C_HOTSPOTS flag");
        create_fallback_bindings();
    } else if !c_hotspots_enabled {
        println!("cargo:warning=C hotspots feature not enabled, using Rust-only implementation");
        println!("cargo:warning=Enable with --features c_hotspots for performance improvements");
        create_fallback_bindings();
    } else {
        // Compile C hotspots for WASM-only target with production implementations
        println!("cargo:warning=Compiling C hotspots for WASM target");
        match compile_c_hotspots() {
            Ok(_) => {
                println!("cargo:warning=C hotspots compiled successfully with WASM SIMD-128 support");
                println!("cargo:rustc-cfg=c_hotspots_available");
            }
            Err(e) => {
                println!("cargo:warning=C hotspots compilation failed: {}", e);
                println!("cargo:warning=Falling back to pure Rust implementations");
                println!("cargo:warning=Performance will be lower but all functionality will work");
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
    println!("cargo:warning=Compiling C hotspots for WASM-only target");
    
    let target = env::var("TARGET").unwrap_or_default();
    
    // WASM-only compilation - no native support
    if !target.contains("wasm32") {
        return Err("WASM-only targeting: native compilation removed".into());
    }
    
    let clang_path = find_clang()?;
    println!("cargo:warning=Found clang for WASM compilation at: {}", clang_path);
    
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
    
    // C hotspots - all providing >15% performance improvement
    let c_files = [
        "util.c",          // High-performance utility functions
        "memory.c",        // WASM memory management with 100MB pool
        "math_kernel.c",   // Optimized WASM SIMD math operations
        "image_kernel.c",  // Advanced image processing (octree quantization, Floyd-Steinberg)
        "compress.c",      // C LZ4 and Huffman compression
        "mesh_decimate.c", // Advanced QEM mesh decimation algorithm
    ];
    
    // Check if files exist
    for file in &c_files {
        let path = format!("{}/{}", hotspots_src_dir, file);
        if !std::path::Path::new(&path).exists() {
            return Err(format!("C file not found: {}", path).into());
        }
    }
    
    // Compile C files for WASM target only
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
        .warnings(false);
    
    // Only set C11 standard for non-MSVC compilers (MSVC has different syntax)
    if !build.get_compiler().is_like_msvc() {
        build.std("c11");
    }
    
    // WASM-only target specific compilation
    println!("cargo:warning=WASM-only target - configuring clang");
    
    build.compiler(&clang_path);
    build.flag("--target=wasm32-unknown-unknown");
    build.flag("-O3");                    // Maximum optimization for performance
    build.flag("-flto");                  // Link time optimization  
    build.flag("-msimd128");              // Enable WASM SIMD
    build.flag("-mbulk-memory");          // Enable bulk memory operations
    build.flag("-mmutable-globals");      // Enable mutable globals
    build.flag("-fno-builtin");
    build.flag("-nostdlib");
    build.flag("-Wno-unused-parameter");
    build.flag("-Wno-unused-variable");
    build.define("__wasm32__", None);
    build.define("WASM_TARGET", None);
    build.define("NDEBUG", None);         // Release mode
    
    // CRITICAL: WASM visibility control for performance isolation
    build.flag("-fvisibility=hidden");   // Hide all symbols by default
    build.flag("-fno-common");           // Prevent common symbol export
    
    // CRITICAL: Export only essential WASM runtime functions, not C hotspot functions
    // This ensures all C hotspot calls go through Rust wrapper functions for safety
    println!("cargo:rustc-link-arg=--export=wasm_malloc");
    println!("cargo:rustc-link-arg=--export=wasm_free");
    println!("cargo:rustc-link-arg=--export=wasm_get_memory_usage");
    
    // CRITICAL: WASM optimization flags for performance targets compliance
    println!("cargo:rustc-link-arg=--lto-O3"); // Link time optimization
    println!("cargo:rustc-link-arg=--no-demangle"); // Smaller binary
    println!("cargo:rustc-link-arg=--strip-debug"); // Remove debug info
    
    println!("cargo:warning=Compiling C files: {:?}", c_files);
    // Try to compile - wrap in error handling
    match build.try_compile("pixie_hotspots") {
        Ok(_) => {
            println!("cargo:warning=C hotspots compiled successfully for WASM!");
        },
        Err(e) => {
            return Err(format!("C compilation failed: {}", e).into());
        }
    }
    
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
    // From image_kernel.h - Image processing kernels
    pub fn quantize_colors_octree(rgba_data: *const u8, width: usize, height: usize, 
                                  max_colors: usize) -> *mut QuantizedImage;
    pub fn quantize_colors_median_cut(rgba_data: *const u8, width: usize, height: usize, 
                                      max_colors: usize) -> *mut QuantizedImage;
    pub fn gaussian_blur_simd(image: *mut u8, width: i32, height: i32, 
                              channels: i32, sigma: f32);
    pub fn dither_floyd_steinberg(image: *mut u8, width: i32, height: i32, channels: i32,
                                  palette: *const Color32, palette_size: usize);
    pub fn free_quantized_image(img: *mut QuantizedImage);
    
    // From mesh_decimate.h - QEM mesh decimation
    pub fn decimate_mesh_qem(vertices: *const f32, vertex_count: usize,
                             indices: *const u32, index_count: usize,
                             target_ratio: f32) -> MeshDecimateResult;
    pub fn free_mesh_decimate_result(result: *mut MeshDecimateResult);
    
    // From compress.h - Compression kernels  
    pub fn compress_lz4(input: *const u8, input_size: usize, output: *mut u8, 
                        max_output_size: usize) -> i32;
    pub fn decompress_lz4(input: *const u8, input_size: usize, output: *mut u8, 
                           output_size: usize) -> i32;
    pub fn compress_huffman(input: *const u8, input_size: usize, output: *mut u8,
                            max_output_size: usize) -> i32;
    pub fn get_optimal_compression(data: *const u8, size: usize) -> u32;
    
    // From math_kernel.h - SIMD math operations
    pub fn simd_vec4_add(a: *const f32, b: *const f32, result: *mut f32);
    pub fn simd_vec4_multiply(a: *const f32, b: *const f32, result: *mut f32);
    pub fn simd_matrix4_multiply(a: *const f32, b: *const f32, result: *mut f32);
    pub fn simd_color_convert_batch(rgb_array: *const f32, hsv_array: *mut f32, count: usize);
    
    // From memory.h - WASM memory management
    pub fn wasm_malloc(size: usize) -> *mut core::ffi::c_void;
    pub fn wasm_free(ptr: *mut core::ffi::c_void);
    pub fn wasm_get_memory_usage() -> u64;
}
"#,
    )?;
    
    println!("cargo:warning=Manual FFI bindings created successfully!");
    
    // Tell cargo to invalidate the built crate whenever C files change
    println!("cargo:rerun-if-changed=hotspots/");
    
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
