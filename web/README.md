# Pixie Juice Web Application

A modern Next.js web application that brings Pixie Juice's powerful media optimization capabilities directly to your browser using WebAssembly.

## Features

- **🚀 WebAssembly Performance**: Rust-powered optimization running at near-native speeds
- **🎨 Modern React UI**: Built with Next.js 15, React 19, and Tailwind CSS
- **🧊 3D Model Preview**: Interactive mesh visualization with React Three Fiber
- **📁 Drag & Drop**: Intuitive file handling with visual feedback
- **⚡ Batch Processing**: Optimize multiple files simultaneously
- **🎛️ Real-time Controls**: Adjust quality settings with instant feedback
- **📱 Responsive Design**: Works seamlessly on desktop and mobile
- **🔒 Privacy First**: All processing happens in your browser - no server uploads

## Supported Formats

### Images
- PNG, JPEG, WebP, GIF, BMP, TIFF
- Quality control and lossless optimization
- Format conversion with size optimization

### 3D Models
- OBJ, PLY files with interactive preview
- STL support (optimization without preview)
- Mesh decimation and optimization

## Quick Start

### Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Open http://localhost:3000
```

### Building for Production

```bash
# Build the application
npm run build

# Start production server
npm start
```

## Architecture

```
web/
├── src/app/
│   ├── components/          # React components
│   │   ├── FileDropZone.js  # File selection and drag & drop
│   │   ├── ProcessingPanel.js # Optimization settings
│   │   ├── ResultsPanel.js   # Results and downloads
│   │   └── MeshViewer.js     # 3D model preview
│   ├── hooks/
│   │   └── useWasm.js       # WASM module integration
│   ├── loaders/
│   │   └── main.js          # Legacy WASM logic (reference)
│   ├── layout.js            # App layout and metadata
│   ├── page.js              # Main application page
│   └── globals.css          # Global styles
├── pkg/                     # Generated WASM bindings
├── public/                  # Static assets
└── package.json
```

## Component Overview

### Main Application (`page.js`)
- Manages global application state
- Handles file selection and processing workflow
- Coordinates between all components

### File Drop Zone (`FileDropZone.js`)
- Drag & drop file handling
- File validation and preview
- Multi-file selection support

### Processing Panel (`ProcessingPanel.js`)
- Quality control sliders
- Output format selection
- WASM optimization execution
- Progress tracking

### Results Panel (`ResultsPanel.js`)
- Optimization results display
- File size comparisons
- Individual and bulk downloads

### Mesh Viewer (`MeshViewer.js`)
- React Three Fiber integration
- OBJ/PLY file parsing
- Interactive 3D controls
- Lighting and material setup

### WASM Hook (`useWasm.js`)
- WebAssembly module loading
- Error handling and loading states
- WASM API exposure to React components

## Dependencies

- **Next.js 15**: React framework with app router
- **React 19**: Latest React with concurrent features
- **@react-three/fiber**: React renderer for Three.js
- **@react-three/drei**: Useful helpers for React Three Fiber
- **Three.js**: 3D graphics library
- **Tailwind CSS**: Utility-first CSS framework

## Deployment

The application can be deployed to any static hosting service:

### Vercel (Recommended)
```bash
npm run build
# Deploy to Vercel
```

### Netlify
```bash
npm run build
# Upload dist/ folder to Netlify
```

### GitHub Pages
```bash
npm run build
# Configure GitHub Actions for static deployment
```

## Development Notes

### WASM Integration
The application uses dynamic imports to load the WebAssembly module, avoiding SSR issues with Next.js. The WASM module is initialized once and shared across all components via the `useWasm` hook.

### File Handling
Files are processed entirely in the browser using the FileReader API and WASM. No data ever leaves the user's device, ensuring privacy and enabling offline usage.

### 3D Preview
The mesh viewer uses simple parsers for OBJ and PLY formats, converting them to Three.js BufferGeometry for display. More complex formats could be added by extending the parser functions.

### Performance
The React components are optimized for performance with proper memoization and state management. File processing is handled asynchronously to keep the UI responsive.

## Contributing

1. Follow the existing component structure
2. Use TypeScript for new components (optional but recommended)
3. Maintain consistent styling with Tailwind classes
4. Test with various file formats and sizes
5. Ensure responsive design principles

## Troubleshooting

### WASM Loading Issues
- Ensure the `pkg/` directory contains the generated WASM files
- Check browser console for module loading errors
- Verify the dynamic import path in `useWasm.js`

### File Processing Errors
- Check supported file formats
- Verify file size limits (browser memory constraints)
- Monitor browser console for WASM runtime errors

### 3D Preview Issues
- Ensure OBJ/PLY files have valid geometry
- Check for proper vertex/face formatting
- Verify file encoding (should be UTF-8 for text formats)
