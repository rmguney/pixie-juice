#!/bin/bash

# Build script for the webapp
# This script builds the WASM module and sets up the webapp for development

set -e

# Build WASM module
echo "Building WASM module..."
cd webapp
wasm-pack build --target web --out-dir pkg --no-typescript

# Copy files to a dist directory for serving
echo "Setting up dist directory..."
cd ..
mkdir -p dist
cp webapp/index.html dist/
cp -r webapp/pkg dist/

echo "Build complete! Serve the dist directory with a local HTTP server."
echo "For example: python -m http.server 8000 --directory dist"
