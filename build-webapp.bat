@echo off
REM Build script for the webapp on Windows
REM This script builds the WASM module and sets up the webapp for development

echo Building WASM module...
cd webapp
wasm-pack build --target web --out-dir pkg --no-typescript
cd ..

echo Setting up dist directory...
if not exist dist mkdir dist
copy webapp\index.html dist\
xcopy webapp\pkg dist\pkg\ /E /I /Y

echo Build complete! Serve the dist directory with a local HTTP server.
echo For example: python -m http.server 8000 --directory dist
