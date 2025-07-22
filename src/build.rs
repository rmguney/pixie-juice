use std::env;
use std::path::PathBuf;

fn main() {
    // Declare the custom cfg option
    println!("cargo::rustc-check-cfg=cfg(c_hotspots_available)");
    
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.contains("wasm32");
    
    // Force disable C hotspots if explicitly requested or if WASM target
    let force_disable = std::env::var("PIXIE_DISABLE_C_HOTSPOTS").is_ok();
    
    if force_disable {
        println!("cargo:warning=C hotspots disabled by PIXIE_DISABLE_C_HOTSPOTS flag");
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
    
    let hotspots_dir = "../hotspots";
    let hotspots_src_dir = "../hotspots/src";
    let hotspots_include_dir = "../hotspots/include";
    
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
    
    // Enable all C hotspot files for full functionality
    let c_files = [
        "util.c",
        "memory.c", 
        "math_kernel.c",
        "image_kernel.c", 
        "compress.c",  
        "mesh_decimate.c", 
        "video_encode.c",
        "png_opt.c",  // PNG optimization with SIMD
        "webp_opt.c", // WebP optimization with SIMD
        "wasm_memory.c", // WASM memory management
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
// Manual C hotspot bindings to avoid bindgen header issues on Windows
use std::ffi::c_void;

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

// PNG optimization structures to match C header
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PngOptConfig {
    pub compress_level: i32,      // 0-9, compression level
    pub reduce_colors: i32,       // 0 or 1, whether to reduce color palette
    pub max_colors: i32,         // Maximum colors in palette (2-256)
    pub strip_metadata: i32,     // 0 or 1, whether to remove metadata chunks
    pub optimize_filters: i32,   // 0 or 1, whether to optimize PNG filters
}

#[repr(C)]
#[derive(Debug)]
pub struct PngOptResult {
    pub output_data: *mut u8,           // Optimized PNG data (caller must free)
    pub output_size: usize,             // Size of optimized data
    pub compression_ratio: f64,         // output_size / input_size
    pub error_code: i32,                // 0 = success, negative = error
    pub error_message: [u8; 256],       // Error description if error_code != 0
}

// WebP optimization structures to match C header
#[repr(C)]
#[derive(Debug, Clone)]
pub struct WebPOptConfig {
    pub quality: i32,           // 0-100, where 100 is lossless
    pub method: i32,           // 0-6, compression method (0=fast, 6=slower but better)
    pub use_lossless: i32,     // 0=lossy, 1=lossless
    pub alpha_quality: i32,    // 0-100, alpha channel quality
    pub preserve_alpha: i32,   // 0=no, 1=yes
    pub optimize_filters: i32, // 0=no, 1=yes - optimize filtering
    pub use_sharp_yuv: i32,    // 0=no, 1=yes - use sharp YUV conversion
}

#[repr(C)]
#[derive(Debug)]
pub struct WebPOptResult {
    pub data: *mut u8,        // Optimized WebP data (caller must free)
    pub size: usize,          // Size of optimized data
    pub error_code: i32,      // 0 = success, negative = error
    pub error_message: *mut i8, // Error description if error_code != 0
}

extern "C" {
    // From util.h
    pub fn buffer_create(initial_capacity: usize) -> *mut c_void;
    pub fn buffer_destroy(buffer: *mut c_void);
    pub fn buffer_append(buffer: *mut c_void, data: *const u8, size: usize) -> i32;
    pub fn buffer_resize(buffer: *mut c_void, new_capacity: usize) -> i32;
    
    // From memory.h
    pub fn memcpy_simd(dest: *mut c_void, src: *const c_void, size: usize);
    pub fn memset_simd(dest: *mut c_void, value: i32, size: usize);
    pub fn memory_pool_create(initial_size: usize) -> *mut c_void;
    pub fn memory_pool_destroy(pool: *mut c_void);
    pub fn memory_pool_alloc(pool: *mut c_void, size: usize) -> *mut c_void;
    pub fn memory_pool_reset(pool: *mut c_void);
    
    // Zero-copy buffer management
    pub fn create_zero_copy_buffer(capacity: usize) -> *mut c_void;
    pub fn wrap_zero_copy_buffer(data: *mut c_void, size: usize, deallocator: *mut c_void) -> *mut c_void;
    pub fn slice_zero_copy_buffer(buffer: *mut c_void, offset: usize, size: usize) -> *mut c_void;
    pub fn retain_zero_copy_buffer(buffer: *mut c_void);
    pub fn release_zero_copy_buffer(buffer: *mut c_void);
    
    // Memory utilities
    pub fn memory_prefetch(addr: *const c_void, size: usize);
    pub fn memory_flush_cache(addr: *const c_void, size: usize);
    pub fn get_cache_line_size() -> usize;
    pub fn fill_pattern_u32(dest: *mut u32, pattern: u32, count: usize);
    pub fn fill_pattern_u64(dest: *mut u64, pattern: u64, count: usize);
    pub fn find_pattern(haystack: *const u8, haystack_size: usize, needle: *const u8, needle_size: usize) -> usize;
    pub fn validate_buffer_bounds(buffer: *const c_void, buffer_size: usize, access_size: usize) -> i32;
    pub fn detect_buffer_overflow(buffer: *const c_void, expected_size: usize) -> i32;
    pub fn memcmp_fast(ptr1: *const u8, ptr2: *const u8, size: usize) -> i32;
    
    // From math_kernel.h
    pub fn vector_dot_product_simd(a: *const f32, b: *const f32, size: usize) -> f32;
    pub fn matrix_multiply_simd(a: *const f32, b: *const f32, result: *mut f32, 
                                m: i32, n: i32, k: i32);
    pub fn gaussian_blur_simd(image: *mut u8, width: i32, height: i32, 
                              channels: i32, sigma: f32);
    
    // From image_kernel.h
    pub fn quantize_colors_octree(image_data: *const u8, width: u32, height: u32, 
                                  max_colors: u32) -> *mut c_void;
    pub fn apply_gaussian_blur(image_data: *const u8, width: u32, height: u32, 
                               sigma: f32) -> *mut c_void;
    pub fn apply_floyd_steinberg_dither(image_data: *const u8, width: u32, 
                                        height: u32) -> *mut c_void;
    pub fn apply_unsharp_mask(image_data: *const u8, width: u32, height: u32, 
                               amount: f32, radius: f32, threshold: f32) -> *mut c_void;
    pub fn apply_edge_detection(image_data: *const u8, width: u32, height: u32, 
                                 algorithm: i32) -> *mut c_void;
    
    // From png_opt.h - PNG optimization with SIMD
    pub fn png_optimize_c(input_data: *const u8, input_len: usize, 
                         config: *const PngOptConfig) -> PngOptResult;
    pub fn png_opt_result_free(result: *mut PngOptResult);
    pub fn png_has_alpha_channel(data: *const u8, size: usize) -> i32;
    pub fn analyze_png_colors(rgba_data: *const u8, pixel_count: usize,
                             has_transparency: *mut i32, unique_colors: *mut i32) -> i32;
    
    // From webp_opt.h - WebP optimization with SIMD
    pub fn webp_optimize_c(input_data: *const u8, input_len: usize,
                          config: *const WebPOptConfig) -> WebPOptResult;
    pub fn webp_opt_result_free(result: *mut WebPOptResult);
    pub fn webp_has_alpha_channel(data: *const u8, size: usize) -> i32;
    pub fn webp_get_info(data: *const u8, size: usize, width: *mut i32, 
                        height: *mut i32, has_alpha: *mut i32) -> i32;
    
    // From mesh_decimate.h
    pub fn decimate_mesh_qem(vertices: *const f32, vertex_count: usize,
                             indices: *const u32, index_count: usize,
                             target_ratio: f32) -> MeshDecimateResult;
    pub fn weld_vertices_spatial(vertices: *const f32, vertex_count: usize,
                                 indices: *const u32, index_count: usize,
                                 tolerance: f32) -> MeshDecimateResult;
    pub fn free_mesh_decimate_result(result: *mut MeshDecimateResult);
    
    // From compress.h
    pub fn compress_lz4(input: *const u8, input_size: usize, output: *mut u8, 
                        max_output_size: usize) -> i32;
    pub fn decompress_lz4(input: *const u8, input_size: usize, output: *mut u8, 
                           output_size: usize) -> i32;
    
    // From video_encode.h
    pub fn encode_h264_frame(frame_data: *const u8, width: i32, height: i32, 
                             quality: f32, output: *mut u8, max_output_size: usize) -> i32;
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
