@echo off
REM Build script for Pixie Juice (Windows)

echo 🔨 Building Pixie Juice...

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust not found. Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Check if wasm-pack is installed
where wasm-pack >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo 📦 Installing wasm-pack...
    cargo install wasm-pack
)

echo 🏗️  Building native CLI...
cargo build --release --package cli

echo 🌐 Building WASM webapp...
cd webapp
wasm-pack build --target web --out-dir ../pkg
cd ..

echo ✅ Build complete!
echo.
echo 🚀 To run:
echo   CLI: .\target\release\pxjc.exe --help
echo   Web: Serve webapp\index.html with a local HTTP server
echo.
echo 📝 For development:
echo   Tests: python -m pytest tests\ -v
echo   Watch: cargo watch -x "build --package cli"
