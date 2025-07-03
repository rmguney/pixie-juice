#!/bin/bash
# Build script for Pixie Juice

set -e

echo "🔨 Building Pixie Juice..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    cargo install wasm-pack
fi

echo "🏗️  Building native CLI..."
cargo build --release --package cli

echo "🌐 Building WASM webapp..."
cd webapp
wasm-pack build --target web --out-dir ../pkg
cd ..

echo "✅ Build complete!"
echo ""
echo "🚀 To run:"
echo "  CLI: ./target/release/pxjc --help"
echo "  Web: Serve webapp/index.html with a local HTTP server"
echo ""
echo "📝 For development:"
echo "  Tests: python -m pytest tests/ -v"
echo "  Watch: cargo watch -x 'build --package cli'"
