'use client';

import { useState, useCallback } from 'react';

export default function ProcessingPanel({ files, wasm, onProcess, setIsProcessing }) {
  const [quality, setQuality] = useState(85);
  const [outputFormat, setOutputFormat] = useState('auto');
  const [progress, setProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [isProcessing, setLocalProcessing] = useState(false);

  const processFiles = useCallback(async () => {
    if (!wasm || files.length === 0) return;

    setLocalProcessing(true);
    setIsProcessing(true);
    setProgress(0);

    const results = [];

    try {
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        setCurrentFile(file.name);
        setProgress(((i + 1) / files.length) * 100);

        try {
          // Read file as bytes
          const fileData = await readFileAsBytes(file);

          // Create config
          const config = new wasm.WasmOptConfig();
          config.quality = quality;
          config.preserve_metadata = true;

          let optimizedData;
          let targetFormat;
          let fileType;

          if (file.type.startsWith('image/')) {
            // Image processing
            fileType = 'image';
            const inputFormat = file.type.split('/')[1];
            targetFormat = outputFormat === 'auto' ? inputFormat : outputFormat;

            optimizedData = await wasm.imageOptimizer.process_image_file(
              fileData,
              inputFormat,
              targetFormat,
              config
            );
          } else {
            // Mesh processing
            fileType = 'mesh';
            const extension = file.name.toLowerCase().split('.').pop();
            targetFormat = extension;

            optimizedData = await wasm.meshOptimizer.process_mesh_file(
              fileData,
              extension,
              config
            );
          }

          // Calculate savings
          const originalSize = fileData.length;
          const optimizedSize = optimizedData.length;
          const savings = ((originalSize - optimizedSize) / originalSize) * 100;

          results.push({
            originalFile: file,
            originalData: fileData,
            optimizedData: optimizedData,
            targetFormat: targetFormat,
            originalSize: originalSize,
            optimizedSize: optimizedSize,
            savings: savings,
            success: true,
            fileType: fileType
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
    <div className="border border-neutral-800 rounded-lg overflow-hidden">
      <div className="p-4 border-b border-neutral-800">
        <h3 className="text-sm font-normal text-white">Settings</h3>
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
            <option value="auto">Auto (Keep Original)</option>
            <option value="png">PNG</option>
            <option value="jpeg">JPEG</option>
            <option value="webp">WebP</option>
          </select>
        </div>

        {/* Process Button */}
        <button
          onClick={processFiles}
          disabled={!wasm || files.length === 0}
          className="w-full py-3 px-4 bg-black hover:bg-neutral-900 border border-neutral-800 disabled:bg-neutral-800 text-white text-sm font-medium rounded-md transition-colors disabled:text-neutral-500"
        >
          Optimize {files.length} File{files.length !== 1 ? 's' : ''}
        </button>
      </div>
    </div>
  );
}
