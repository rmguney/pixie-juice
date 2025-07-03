# Pixie Juice (pxjc)

High-performance media processing utility with Rust-first architecture and C performance hotspots

## Features

### Image Processing (Completed)
- **Universal optimization** for PNG, JPEG, WebP, GIF, BMP, and TIFF formats
- **Format auto-detection** and validation
- **Quality control** and compression settings
- **Pure Rust implementation** using proven crates (`oxipng`, `jpeg-encoder`, `webp`)

### Mesh Processing (In Development)
- 3D model optimization and format conversion
- Support for OBJ, PLY, DAE, and glTF formats
- Mesh decimation algorithms

### Video Processing  (PLanned)
- **Format support** for MP4 and WebM
- **Compression optimization** with quality/size balance
- **Frame extraction** and thumbnail generation
- **Batch processing** for video libraries
- **Metadata preservation** and optimization
- **Resolution scaling** and aspect ratio management
- **Audio track processing** and optimization

### WASM Web Application (PLanned)
- **Browser-native processing** - No server uploads required
- **Cross-platform compatibility** - Works in any modern browser
- **Progressive Web App (PWA)** - Install locally, work offline
- **Drag-and-drop interface** - Intuitive file handling
- **Real-time preview** - See changes before processing
- **Batch queue management** - Process multiple files sequentially
- **Settings persistence** - Remember your preferences
- **Download management** - Organized output with original filenames

## Installation

### Prerequisites
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- C compiler (for performance hotspots)

### Build from Source
```bash
git clone <your-repo-url>
cd pixie-juice
cargo build --release
```

## Usage

### Image Optimization
```bash
# Optimize a single image
pxjc optimize input.jpg

# Optimize multiple images
pxjc optimize *.png

# Batch optimize with custom settings
pxjc optimize --quality 85 --format webp images/
```

### Available Commands
- `pxjc optimize` - Optimize images with auto-format detection
- `pxjc help` - Show available commands and options

## Project Structure

```
pixie-juice/
├── rust_core/          # Core Rust processing logic
├── frontend/           # CLI application
├── hotspots/           # C performance hotspots
├── webapp/             # Web application (WASM)
├── tests/              # Test suite
└── cli/                # Command-line interface
```


### Testing
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package rust_core
```
