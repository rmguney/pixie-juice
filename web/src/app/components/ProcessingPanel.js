'use client';

import { useState, useCallback } from 'react';

export default function ProcessingPanel({ files, wasm, onProcess, setIsProcessing }) {
  const [quality, setQuality] = useState(75); // default quality
  const [outputFormat, setOutputFormat] = useState('auto');
  const [preserveMetadata, setPreserveMetadata] = useState(false); // Metadata preservation
  const [progress, setProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [isProcessing, setLocalProcessing] = useState(false);

  const processFiles = useCallback(async () => {
    if (!wasm.available || files.length === 0) return;

    setLocalProcessing(true);
    setIsProcessing(true);
    setProgress(0);

    // Apply global configuration settings
    try {
      if (wasm.set_preserve_metadata) {
        wasm.set_preserve_metadata(preserveMetadata);
        console.log(`Metadata preservation set to: ${preserveMetadata}`);
      }
      
      // Set lossless mode for high quality settings
      if (wasm.set_lossless_mode && quality >= 90) {
        wasm.set_lossless_mode(true);
        console.log('Lossless mode enabled for high quality');
      } else if (wasm.set_lossless_mode) {
        wasm.set_lossless_mode(false);
      }
    } catch (configError) {
      console.warn('Configuration setting failed:', configError);
    }

    const results = [];

    try {
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        setCurrentFile(file.name);
        setProgress(((i + 1) / files.length) * 100);

        try {
          // Read file as bytes
          const fileData = await readFileAsBytes(file);
          console.log(`Processing ${file.name}: ${fileData.length} bytes`);

          let optimizedData;
          let targetFormat;
          let fileType;

          if (file.type.startsWith('image/')) {
            fileType = 'image';
            
            // Auto-detect format using WASM detection
            let detectedFormat = 'unknown';
            if (wasm.detect_format) {
              try {
                detectedFormat = wasm.detect_format(fileData);
                console.log(`Detected format: ${detectedFormat}`);
              } catch (e) {
                console.warn('Format detection failed:', e);
                detectedFormat = file.type.split('/')[1];
              }
            }

            const inputFormat = file.type.split('/')[1];
            targetFormat = outputFormat === 'auto' ? inputFormat : outputFormat;

            console.log(`Optimizing image ${file.name} with quality ${quality}`);
            
            // Use format-specific optimization if available
            if (outputFormat !== 'auto' && wasm[`optimize_${outputFormat}`]) {
              console.log(`Using format-specific optimizer: optimize_${outputFormat}`);
              optimizedData = wasm[`optimize_${outputFormat}`](fileData, quality);
            } else if (outputFormat !== 'auto' && wasm[`convert_to_${outputFormat}`]) {
              console.log(`Using format converter: convert_to_${outputFormat}`);
              if (outputFormat === 'png') {
                optimizedData = wasm.convert_to_png(fileData);
              } else {
                optimizedData = wasm[`convert_to_${outputFormat}`](fileData, quality);
              }
            } else {
              // Use auto-optimization for best results
              if (wasm.optimize_auto) {
                console.log('Using auto-optimization');
                optimizedData = wasm.optimize_auto(fileData, quality);
              } else {
                console.log('Using generic image optimization');
                optimizedData = wasm.optimize_image(fileData, quality);
              }
            }
            
            console.log(`Image optimization result: ${optimizedData.length} bytes`);
          } else {
            // Mesh processing using simple API
            fileType = 'mesh';
            const extension = file.name.toLowerCase().split('.').pop();
            targetFormat = extension;

            // Convert quality to ratio for mesh optimization (0-100 quality becomes 0.1-1.0 ratio)
            const targetRatio = Math.max((100 - quality) / 100, 0.1); // Higher quality = less reduction
            console.log(`Optimizing mesh ${file.name} with ratio ${targetRatio}`);
            optimizedData = wasm.optimize_mesh(fileData, targetRatio);
            console.log(`Mesh optimization result: ${optimizedData.length} bytes`);
          }

          // Calculate savings
          const originalSize = fileData.length;
          const optimizedSize = optimizedData.length;
          const savings = ((originalSize - optimizedSize) / originalSize) * 100;

          console.log(`Optimization complete for ${file.name}:`);
          console.log(`  Original: ${originalSize} bytes`);
          console.log(`  Optimized: ${optimizedSize} bytes`);
          console.log(`  Savings: ${savings.toFixed(1)}%`);

          // Get performance metrics if available
          let performanceMetrics = null;
          if (wasm.get_performance_metrics) {
            try {
              performanceMetrics = wasm.get_performance_metrics();
              console.log('Performance metrics:', performanceMetrics);
            } catch (e) {
              console.warn('Failed to get performance metrics:', e);
            }
          }

          results.push({
            originalFile: file,
            originalData: fileData,
            optimizedData: optimizedData,
            targetFormat: targetFormat,
            originalSize: originalSize,
            optimizedSize: optimizedSize,
            savings: savings,
            success: true,
            fileType: fileType,
            performanceMetrics: performanceMetrics
          });

        } catch (error) {
          console.error(`Error processing ${file.name}:`, error);
          results.push({
            originalFile: file,
            error: error.toString(),
            success: false,
            fileType: file.type.startsWith('image/') ? 'image' : 'mesh'
          });
        }
      }

      // Check performance compliance after processing
      if (wasm.check_performance_compliance) {
        try {
          const isCompliant = wasm.check_performance_compliance();
          console.log(`Performance compliance: ${isCompliant ? 'PASSED' : 'FAILED'}`);
          
          if (!isCompliant) {
            console.warn('Performance targets exceeded during processing');
          }
        } catch (e) {
          console.warn('Failed to check performance compliance:', e);
        }
      }

      onProcess(results);

    } catch (error) {
      console.error('Processing failed:', error);
      alert('Processing failed. Please try again.');
    } finally {
      setLocalProcessing(false);
      setIsProcessing(false);
      setProgress(0);
      setCurrentFile('');
    }
  }, [files, wasm, quality, outputFormat, onProcess, setIsProcessing]);

  // Helper function to read file as bytes
  const readFileAsBytes = (file) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const arrayBuffer = reader.result;
        const uint8Array = new Uint8Array(arrayBuffer);
        resolve(uint8Array);
      };
      reader.onerror = reject;
      reader.readAsArrayBuffer(file);
    });
  };

  if (isProcessing) {
    return (
      <div className="border border-neutral-800 rounded-lg overflow-hidden">
        <div className="p-4 border-b border-neutral-800">
          <h3 className="text-sm font-normal text-white">Processing</h3>
        </div>
        
        <div className="p-6">
          <div className="space-y-4">
            <div className="w-full bg-neutral-900 rounded-full h-2">
              <div 
                className="bg-white h-2 rounded-full transition-all duration-300"
                style={{ width: `${progress}%` }}
              ></div>
            </div>
            
            <div className="flex items-center justify-between text-xs text-neutral-400">
              <p className="truncate max-w-xs">
                {currentFile}
              </p>
              <p className="text-right">
                {Math.round(progress)}%
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-lg overflow-hidden">
      <div className="p-4 border-b border-neutral-800">
        <h3 className="text-sm font-normal text-white text-center">Settings</h3>
      </div>
      
      <div className="p-6 space-y-5">
        {/* Quality Slider */}
        <div>
          <div className="flex items-center justify-between mb-1.5">
            <label htmlFor="quality" className="text-xs text-neutral-400">
              Quality
            </label>
            <span className="text-xs font-mono text-white">{quality}%</span>
          </div>
          <input
            id="quality"
            type="range"
            min="10"
            max="100"
            value={quality}
            onChange={(e) => setQuality(parseInt(e.target.value))}
            className="w-full h-2"
          />
          <div className="text-xs text-neutral-500 mt-1">
            {quality <= 30 ? "Maximum compression (may affect quality)" : 
             quality <= 60 ? "High compression" : "Balanced compression"}
          </div>
        </div>

        {/* Output Format */}
        <div>
          <label htmlFor="format" className="block text-xs text-neutral-400 mb-1.5">
            Output Format
          </label>
          <select
            id="format"
            value={outputFormat}
            onChange={(e) => setOutputFormat(e.target.value)}
            className="w-full p-2.5 text-sm bg-black border border-neutral-800 rounded-md text-white focus:border-neutral-600 focus:outline-none"
          >
            <option value="auto">Auto (Smart Optimization)</option>
            <optgroup label="Phase 1: Core Formats">
              <option value="png">PNG (Lossless)</option>
              <option value="jpeg">JPEG (Lossy)</option>
              <option value="gif">GIF (Animation)</option>
              <option value="bmp">BMP (Basic)</option>
              <option value="webp">WebP (Modern) *</option>
              <option value="ico">ICO (Icons) *</option>
            </optgroup>
            <optgroup label="Phase 3: Advanced Formats (Coming Soon)">
              <option value="svg" disabled>SVG (Vector)</option>
              <option value="avif" disabled>AVIF (Next-gen)</option>
              <option value="heic" disabled>HEIC (Apple)</option>
              <option value="tiff" disabled>TIFF (Professional)</option>
              <option value="pdf" disabled>PDF (Documents)</option>
              <option value="hdr" disabled>HDR (High Dynamic Range)</option>
              <option value="exr" disabled>EXR (Cinema)</option>
            </optgroup>
          </select>
          <div className="text-xs text-neutral-500 mt-1">
            {outputFormat === 'auto' ? "Intelligent format selection based on content" :
             outputFormat === 'webp' ? "Modern format - implementation in progress" :
             outputFormat === 'svg' ? "Vector format - optimization in progress" :
             outputFormat === 'ico' ? "Icon format - decoder improvements needed" :
             outputFormat === 'png' ? "Lossless format, preserves quality" :
             outputFormat === 'jpeg' ? "Widely supported, good for photos" :
             outputFormat === 'gif' ? "Animation support, good for simple graphics" :
             outputFormat === 'bmp' ? "Basic format, widely compatible" :
             "Advanced format - coming in future phase"}
            {(outputFormat === 'webp' || outputFormat === 'svg' || outputFormat === 'ico') && 
             <span className="text-yellow-400"> (*Known issues being resolved)</span>}
          </div>
        </div>

        {/* Metadata Preservation */}
        <div>
          <label className="flex items-center space-x-3 cursor-pointer">
            <input
              type="checkbox"
              checked={preserveMetadata}
              onChange={(e) => setPreserveMetadata(e.target.checked)}
              className="w-4 h-4 bg-black border border-neutral-800 rounded focus:ring-1 focus:ring-neutral-600 text-white"
            />
            <div className="text-xs">
              <div className="text-neutral-400">Preserve Metadata</div>
              <div className="text-neutral-500 text-xs">
                Keep EXIF, XMP, and other metadata (larger file size)
              </div>
            </div>
          </label>
        </div>

        {/* Process Button */}
        <button
          onClick={processFiles}
          disabled={!wasm.available || files.length === 0}
          className="w-full py-3 px-4 bg-black hover:bg-neutral-900 border border-neutral-800 disabled:bg-neutral-800 text-white text-sm font-medium rounded-md transition-colors disabled:text-neutral-500"
        >
          Optimize {files.length} File{files.length !== 1 ? 's' : ''}
        </button>
      </div>
    </div>
  );
}
