use std::env;
use std::path::PathBuf;

fn main() {
    // TODO: Temporarily disable C compilation until MSVC is properly configured
    // For now, just create empty bindings
    
    println!("cargo:warning=C optimizer temporarily disabled - using Rust stubs");
    
    // Create minimal bindings file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::write(
        out_path.join("bindings.rs"),
        "// Temporary empty bindings - C optimizer disabled\n"
    ).expect("Unable to write bindings file");
    
    // Tell cargo to invalidate the built crate whenever C files change
    println!("cargo:rerun-if-changed=../optimizer/");
}
