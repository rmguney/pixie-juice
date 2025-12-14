use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(c_hotspots_available)");
    
    let target = env::var("TARGET").unwrap_or_default();
    
    if !target.contains("wasm32") {
        println!("cargo:warning=WASM-only targeting: native compilation not supported");
        println!("cargo:warning=Use --target=wasm32-unknown-unknown for WASM builds");
        create_fallback_bindings();
        return;
    }
    
    match compile_c_hotspots() {
        Ok(_) => {
            println!("cargo:rustc-cfg=c_hotspots_available");
        },
        Err(e) => {
            println!("cargo:warning=Failed to compile C hotspots: {}", e);
            println!("cargo:warning=Using pure Rust implementations as fallback");
            create_fallback_bindings();
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
    
    if !target.contains("wasm32") {
        return Err("WASM-only targeting: native compilation removed".into());
    }
    
    let clang_path = find_clang()?;
    println!("cargo:warning=Found clang for WASM compilation at: {}", clang_path);
    
    let hotspots_dir = "hotspots";
    let hotspots_src_dir = "hotspots/src";
    let hotspots_include_dir = "hotspots/include";
    
    if !std::path::Path::new(hotspots_dir).exists() {
        return Err("Hotspots directory not found".into());
    }
    if !std::path::Path::new(hotspots_src_dir).exists() {
        return Err("Hotspots src directory not found".into());
    }
    if !std::path::Path::new(hotspots_include_dir).exists() {
        return Err("Hotspots include directory not found".into());
    }
    
    let c_files = [
        "util.c",
        "memory.c",
        "math_kernel.c",
        "image_kernel.c",
        "compress.c",
        "mesh_decimate.c",
        "mesh_attributes.c",
        "vertex_cache.c",
    ];
    
    for file in &c_files {
        let path = format!("{}/{}", hotspots_src_dir, file);
        if !std::path::Path::new(&path).exists() {
            return Err(format!("C file not found: {}", path).into());
        }
    }
    
    let mut build = cc::Build::new();
    
    for file in &c_files {
        build.file(format!("{}/{}", hotspots_src_dir, file));
    }
    
    build
        .include(hotspots_include_dir)
        .include(hotspots_src_dir)
        .opt_level(2)
        .debug(false)
        .warnings(false);
    
    if !build.get_compiler().is_like_msvc() {
        build.std("c11");
    }
    
    println!("cargo:warning=WASM-only target - configuring clang");
    
    build.compiler(&clang_path);
    build.flag("--target=wasm32-unknown-unknown");
    build.flag("-O3");
    build.flag("-flto");
    build.flag("-msimd128");
    build.flag("-mbulk-memory");
    build.flag("-mmutable-globals");
    build.flag("-fno-builtin");
    build.flag("-nostdlib");
    build.flag("-Wno-unused-parameter");
    build.flag("-Wno-unused-variable");
    build.define("__wasm32__", None);
    build.define("WASM_TARGET", None);
    build.define("NDEBUG", None);

    build.flag("-fvisibility=hidden");
    build.flag("-fno-common");
    
    // CRITICAL: Export only essential WASM runtime functions, not C hotspot functions
    // This ensures all C hotspot calls go through Rust wrapper functions for safety
    println!("cargo:rustc-link-arg=--export=wasm_malloc");
    println!("cargo:rustc-link-arg=--export=wasm_free");
    println!("cargo:rustc-link-arg=--export=wasm_get_memory_usage");
    println!("cargo:rustc-link-arg=--lto-O3");
    println!("cargo:rustc-link-arg=--no-demangle");
    println!("cargo:rustc-link-arg=--strip-debug");
    println!("cargo:warning=Compiling C files: {:?}", c_files);
    match build.try_compile("pixie_hotspots") {
        Ok(_) => {
            println!("cargo:warning=C hotspots compiled successfully for WASM!");
        },
        Err(e) => {
            return Err(format!("C compilation failed: {}", e).into());
        }
    }
    
    let cpp_files = ["color_distance.cpp", "color_convert.cpp", "obj_parser.cpp"];
    
    for file in &cpp_files {
        let path = format!("{}/{}", hotspots_src_dir, file);
        if std::path::Path::new(&path).exists() {
            println!("cargo:warning=Compiling C++ file: {}", file);
            
            let mut cpp_build = cc::Build::new();
            cpp_build
                .cpp(true)
                .cpp_link_stdlib(None)
                .file(&path)
                .include(hotspots_include_dir)
                .include(hotspots_src_dir)
                .opt_level(3)
                .debug(false)
                .warnings(false)
                .compiler(&clang_path)
                .flag("--target=wasm32-unknown-unknown")
                .flag("-std=c++17")
                .flag("-O3")
                .flag("-flto")
                .flag("-msimd128")
                .flag("-mbulk-memory")
                .flag("-mmutable-globals")
                .flag("-ffreestanding")
                .flag("-nostdlib++")
                .flag("-fno-exceptions")
                .flag("-fno-rtti")
                .flag("-fno-threadsafe-statics")
                .flag("-nostdlib")
                .flag("-fvisibility=hidden")
                .flag("-fno-common")
                .define("__wasm32__", None)
                .define("WASM_TARGET", None)
                .define("NDEBUG", None);
            
            match cpp_build.try_compile(&format!("pixie_{}", file.trim_end_matches(".cpp"))) {
                Ok(_) => {
                    println!("cargo:warning=C++ hotspot '{}' compiled successfully!", file);
                },
                Err(e) => {
                    println!("cargo:warning=C++ compilation failed for '{}': {}", file, e);
                    println!("cargo:warning=Continuing with C-only hotspots");
                }
            }
        }
    }
    
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
    
    println!("cargo:rerun-if-changed=hotspots/");
    
    Ok(())
}

fn find_clang() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(output) = std::process::Command::new("clang").arg("--version").output() {
        if output.status.success() {
            return Ok("clang".to_string());
        }
    }
    
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
