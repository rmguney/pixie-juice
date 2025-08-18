'use client';

import { useState, useCallback } from 'react';

export default function ProcessingPanel({ files, wasm, onProcess, setIsProcessing }) {
  const [quality, setQuality] = useState(75); // default quality
  const [outputFormat, setOutputFormat] = useState('auto');
  const [preserveMetadata, setPreserveMetadata] = useState(false);
  const [losslessMode, setLosslessMode] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [isProcessing, setLocalProcessing] = useState(false);

  const processFiles = useCallback(async () => {
    if (!wasm.available || files.length === 0) return;

    // Verify WASM version by checking memory target and build timestamp
    try {
      if (wasm.build_timestamp) {
        const buildTime = wasm.build_timestamp();
        console.log(`🚀 WASM Build Timestamp: ${buildTime}`);
      }
      
      if (wasm.pixie_get_memory_target_mb) {
        const memoryTarget = wasm.pixie_get_memory_target_mb();
        console.log(`🧠 WASM Memory Target: ${memoryTarget}MB`);
        if (memoryTarget < 1000) {
          console.warn('⚠️ Using old WASM version - memory target should be 1024MB');
        }
      }
    } catch (e) {
      console.warn('Could not check WASM version:', e);
    }

    setLocalProcessing(true);
    setIsProcessing(true);
    setProgress(0);

    // Apply global configuration settings
    try {
      if (wasm.set_preserve_metadata) {
        wasm.set_preserve_metadata(preserveMetadata);
        console.log(`Metadata preservation set to: ${preserveMetadata}`);
      }
      
      // Set lossless mode based on user preference or high quality threshold
      const shouldUseLossless = losslessMode || quality >= 95;
      if (wasm.set_lossless_mode) {
        wasm.set_lossless_mode(shouldUseLossless);
        console.log(`Lossless mode set to: ${shouldUseLossless} (user: ${losslessMode}, quality: ${quality})`);
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
          
          if (!fileData || fileData.length === 0) {
            throw new Error(`File data is empty for ${file.name}`);
          }
          
          console.log(`Processing ${file.name}: ${fileData.length} bytes`);
          console.log(`File details:`, {
            name: file.name,
            type: file.type,
            size: file.size,
            lastModified: new Date(file.lastModified).toISOString(),
            dataLength: fileData.length,
            quality: quality,
            outputFormat: outputFormat
          });
          
          const firstBytes = Array.from(fileData.slice(0, 16)).map(b => b.toString(16).padStart(2, '0')).join(' ');
          console.log(`First 16 bytes: ${firstBytes}`);
          
          const consoleLogs = [];
          const originalConsoleError = console.error;
          console.error = (...args) => {
            const message = args.join(' ');
            consoleLogs.push({ type: 'error', message, timestamp: Date.now() });
            if (message.includes('compress_lz4')) {
              console.log(`   compress_lz4 ERROR detected for ${file.name}!`);
              console.log(`   File size: ${file.size} bytes`);
              console.log(`   Data length: ${fileData.length} bytes`);
              console.log(`   Quality: ${quality}%`);
              console.log(`   Format: ${outputFormat}`);
              console.log(`   Type: ${file.type}`);
            }
            originalConsoleError.apply(console, args);
          };

          try {
            let optimizedData;
            let targetFormat;
            let fileType;

            // Enhanced file type detection for images and meshes
            const fileExtension = file.name.toLowerCase().split('.').pop();
            const isImageFile = file.type.startsWith('image/') || 
                              fileExtension === 'svg' || 
                              ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'tga', 'targa'].includes(fileExtension);
            const isMeshFile = ['obj', 'stl', 'fbx', 'gltf', 'glb', 'ply'].includes(fileExtension);
                              
            if (isImageFile) {
              fileType = 'image';
              
              // Auto-detect format using WASM detection
              let detectedFormat = 'unknown';
              if (wasm.detect_format) {
                try {
                  detectedFormat = wasm.detect_format(fileData);
                  console.log(`Detected format: ${detectedFormat}`);
                } catch (e) {
                  console.warn('Format detection failed:', e);
                  detectedFormat = 'unknown';
                }
              }

              // Determine input format with priority: WASM detection > file extension > MIME type
              let inputFormat;
              if (detectedFormat && detectedFormat !== 'unknown' && !detectedFormat.startsWith('image:')) {
                inputFormat = detectedFormat;
              } else if (detectedFormat.startsWith('image:')) {
                inputFormat = detectedFormat.split(':')[1]; // Extract format from "image:jpeg" format
              } else if (fileExtension === 'svg') {
                inputFormat = 'svg';
              } else if (file.type.includes('/')) {
                inputFormat = file.type.split('/')[1]; // Extract from MIME type like "image/jpeg"
              } else {
                inputFormat = fileExtension;
              }
              
              targetFormat = outputFormat === 'auto' ? inputFormat : outputFormat;

              console.log(`Optimizing image ${file.name} (${inputFormat}) with quality ${quality}`);
              console.log(`Target format: ${targetFormat}, Input format: ${inputFormat}`);
              
              // CRITICAL: Format conversion logic fix
              if (outputFormat === 'auto') {
                // Auto mode - use best optimization for the current format
                if (wasm.optimize_auto) {
                  console.log('Using auto-optimization (preserving original format)');
                  optimizedData = wasm.optimize_auto(fileData, quality);
                } else {
                  console.log('Using generic image optimization');
                  optimizedData = wasm.optimize_image(fileData, quality);
                }
              } else if (outputFormat !== inputFormat) {
                // Format conversion requested - ALWAYS use convert_to_* functions
                console.log(`Converting from ${inputFormat} to ${outputFormat} using converter`);
                if (wasm[`convert_to_${outputFormat}`]) {
                  if (outputFormat === 'png') {
                    optimizedData = wasm.convert_to_png(fileData);
                  } else {
                    optimizedData = wasm[`convert_to_${outputFormat}`](fileData, quality);
                  }
                } else {
                  throw new Error(`Conversion to ${outputFormat} not supported`);
                }
              } else {
                // Same format optimization - use format-specific optimizer if available
                console.log(`Optimizing ${inputFormat} format (no conversion needed)`);
                if (wasm[`optimize_${outputFormat}`]) {
                  console.log(`Using format-specific optimizer: optimize_${outputFormat}`);
                  optimizedData = wasm[`optimize_${outputFormat}`](fileData, quality);
                } else {
                  // Fallback to generic optimization
                  console.log('Using generic image optimization');
                  optimizedData = wasm.optimize_image(fileData, quality);
                }
              }
              
              console.log(`Image optimization result: ${optimizedData.length} bytes`);
            } else if (isMeshFile) {
              // Mesh processing using format-specific optimizers
              fileType = 'mesh';
              const extension = file.name.toLowerCase().split('.').pop();
              targetFormat = extension;

              // Convert quality to ratio for mesh optimization (0-100 quality becomes 0.1-1.0 ratio)
              const targetRatio = Math.max((100 - quality) / 100, 0.1); // Higher quality = less reduction
              console.log(`Optimizing ${extension.toUpperCase()} mesh ${file.name} with ratio ${targetRatio}`);
              
              // Use format-specific optimizer
              if (extension === 'obj' && wasm.optimize_obj) {
                optimizedData = wasm.optimize_obj(fileData, targetRatio);
              } else if (extension === 'stl' && wasm.optimize_stl) {
                optimizedData = wasm.optimize_stl(fileData, targetRatio);
              } else if (extension === 'fbx' && wasm.optimize_fbx) {
                optimizedData = wasm.optimize_fbx(fileData, targetRatio);
              } else if ((extension === 'gltf' || extension === 'glb') && wasm.optimize_gltf) {
                optimizedData = wasm.optimize_gltf(fileData, targetRatio);
              } else if (extension === 'ply' && wasm.optimize_ply) {
                optimizedData = wasm.optimize_ply(fileData, targetRatio);
              } else if (wasm.optimize_mesh) {
                // Fallback to generic mesh optimizer
                console.log('Using generic mesh optimizer as fallback');
                optimizedData = wasm.optimize_mesh(fileData, targetRatio);
              } else {
                throw new Error(`No optimizer available for ${extension.toUpperCase()} format`);
              }
              console.log(`${extension.toUpperCase()} optimization result: ${optimizedData.length} bytes`);
            } else {
              // Unsupported file format
              throw new Error(`Unsupported file format: ${fileExtension}. Supported formats: PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, TGA, ICO (images) and OBJ, STL, FBX, GLTF, GLB, PLY (meshes)`);
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
            
          } catch (processingError) {
            // Enhanced error logging for compress_lz4 issues
            if (processingError.toString().includes('compress_lz4')) {
              console.log(`🎯 compress_lz4 ERROR CONFIRMED for ${file.name}!`);
              console.log(`   Error message: ${processingError.toString()}`);
              console.log(`   File characteristics that may be relevant:`);
              console.log(`     - Size: ${file.size} bytes`);
              console.log(`     - Type: ${file.type}`);
              console.log(`     - Quality setting: ${quality}%`);
              console.log(`     - Output format: ${outputFormat}`);
              console.log(`   Please note these characteristics for debugging.`);
            }
            
            results.push({
              originalFile: file,
              error: processingError.toString(),
              success: false,
              fileType: (() => {
                const ext = file.name.toLowerCase().split('.').pop();
                if (file.type.startsWith('image/') || ['svg', 'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'tga', 'targa'].includes(ext)) {
                  return 'image';
                } else if (['obj', 'stl', 'fbx', 'gltf', 'glb', 'ply'].includes(ext)) {
                  return 'mesh';
                } else {
                  return 'unknown';
                }
              })(),
              // Add diagnostic info for failed files
              diagnostics: {
                fileName: file.name,
                fileSize: file.size,
                fileType: file.type,
                quality: quality,
                outputFormat: outputFormat,
                errorMessage: processingError.toString(),
                isCompressLz4Error: processingError.toString().includes('compress_lz4')
              }
            });
            
            // Re-throw to be caught by outer catch
            throw processingError;
          } finally {
            // Always restore console.error
            console.error = originalConsoleError;
          }

        } catch (error) {
          console.error(`Error processing ${file.name}:`, error);
          results.push({
            originalFile: file,
            error: error.toString(),
            success: false,
            fileType: (() => {
              const ext = file.name.toLowerCase().split('.').pop();
              if (file.type.startsWith('image/') || ['svg', 'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'tga', 'targa'].includes(ext)) {
                return 'image';
              } else if (['obj', 'stl', 'fbx', 'gltf', 'glb', 'ply'].includes(ext)) {
                return 'mesh';
              } else {
                return 'unknown';
              }
            })()
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
  }, [files, wasm, quality, outputFormat, preserveMetadata, losslessMode, onProcess, setIsProcessing]);

  // Helper function to read file as bytes
  const readFileAsBytes = (file) => {
    return new Promise((resolve, reject) => {
      if (!file) {
        reject(new Error('File is null or undefined'));
        return;
      }
      
      const reader = new FileReader();
      reader.onload = () => {
        const arrayBuffer = reader.result;
        if (!arrayBuffer) {
          reject(new Error('FileReader returned null result'));
          return;
        }
        
        const uint8Array = new Uint8Array(arrayBuffer);
        if (!uint8Array || uint8Array.length === 0) {
          reject(new Error('File data is empty'));
          return;
        }
        
        resolve(uint8Array);
      };
      reader.onerror = (error) => {
        reject(new Error(`FileReader error: ${error}`));
      };
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
          <div className="flex items-center justify-between mb-3">
            <label htmlFor="quality" className="text-xs text-neutral-400">
              Quality
            </label>
            <div className="flex items-center space-x-2">
              <span className="text-xs font-mono text-white bg-neutral-800 px-2 py-1 rounded">{quality}%</span>
            </div>
          </div>
          
          <div className="relative">
            <div className="absolute top-0 left-0 right-0 h-1 bg-neutral-700 rounded-full pointer-events-none"></div>
            <div 
              className="absolute top-0 left-0 h-1 bg-gradient-to-r from-red-500 via-yellow-500 to-green-500 rounded-full pointer-events-none transition-all duration-200"
              style={{ width: `${((quality - 10) / (100 - 10)) * 100}%` }}
            ></div>
            <input
              id="quality"
              type="range"
              min="10"
              max="100"
              value={quality}
              onChange={(e) => setQuality(parseInt(e.target.value))}
              className="quality-slider w-full h-1 bg-transparent appearance-none cursor-pointer relative z-10"
            />
          </div>
          
          <div className="flex justify-between text-xs text-neutral-600 mt-2 px-1">
            <span>10%</span>
            <span>50%</span>
            <span>100%</span>
          </div>
          
          <style jsx>{`
            .quality-slider::-webkit-slider-thumb {
              appearance: none;
              height: 14px;
              width: 14px;
              border-radius: 50%;
              background: #ffffff;
              border: 2px solid #525252;
              cursor: pointer;
              box-shadow: 0 1px 3px rgba(0,0,0,0.4);
              transition: all 0.15s ease;
              margin-top: -20px;
            }
            .quality-slider::-webkit-slider-thumb:hover {
              transform: scale(1.2);
              border-color: #737373;
              box-shadow: 0 2px 6px rgba(0,0,0,0.5);
            }
            .quality-slider::-webkit-slider-thumb:active {
              transform: scale(1.3);
              border-color: #a3a3a3;
              box-shadow: 0 3px 8px rgba(0,0,0,0.6);
            }
            .quality-slider::-moz-range-thumb {
              width: 14px;
              height: 14px;
              border-radius: 50%;
              background: #ffffff;
              border: 2px solid #525252;
              cursor: pointer;
              box-shadow: 0 1px 3px rgba(0,0,0,0.4);
              transition: all 0.15s ease;
            }
            .quality-slider::-moz-range-thumb:hover {
              transform: scale(1.2);
              border-color: #737373;
              box-shadow: 0 2px 6px rgba(0,0,0,0.5);
            }
            .quality-slider::-moz-range-track {
              background: transparent;
              height: 1px;
              border: none;
            }
          `}</style>
          
          <div className="text-xs text-neutral-500 mt-2">
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
            <optgroup label="Image Formats">
              <option value="png">PNG (Lossless)</option>
              <option value="jpeg">JPEG (Lossy)</option>
              <option value="webp">WebP (Modern)</option>
              <option value="bmp">BMP (Basic)</option>
              <option value="gif">GIF (Animation)</option>
              <option value="ico">ICO (Icons)</option>
              <option value="tiff">TIFF (Professional)</option>
              <option value="svg">SVG (Vector)</option>
            </optgroup>
          </select>
          <div className="text-xs text-neutral-500 mt-1">
            {outputFormat === 'auto' ? "Intelligent format selection based on content" :
             outputFormat === 'webp' ? "Modern format with excellent compression" :
             outputFormat === 'svg' ? "Vector format, scalable graphics" :
             outputFormat === 'ico' ? "Icon format for favicons and applications" :
             outputFormat === 'png' ? "Lossless format, preserves quality" :
             outputFormat === 'jpeg' ? "Widely supported, good for photos" :
             outputFormat === 'gif' ? "Animation support, good for simple graphics" :
             outputFormat === 'bmp' ? "Basic format, widely compatible" :
             outputFormat === 'tiff' ? "Professional format, high quality" :
             "Select a format for conversion and optimization"}
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

        {/* Lossless Mode */}
        <div>
          <label className="flex items-center space-x-3 cursor-pointer">
            <input
              type="checkbox"
              checked={losslessMode}
              onChange={(e) => setLosslessMode(e.target.checked)}
              className="w-4 h-4 bg-black border border-neutral-800 rounded focus:ring-1 focus:ring-neutral-600 text-white"
            />
            <div className="text-xs">
              <div className="text-neutral-400">Lossless Mode</div>
              <div className="text-neutral-500 text-xs">
                Force lossless compression (overrides quality setting)
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
