import { useState, useCallback } from 'react';
import type { ProcessingPanelProps, ProcessedResult, WasmHook } from '../types';

const readFileAsBytes = (file: File): Promise<Uint8Array> => {
  return new Promise((resolve, reject) => {
    if (!file) {
      reject(new Error('File is null or undefined'));
      return;
    }
    
    const reader = new FileReader();
    reader.onload = () => {
      const arrayBuffer = reader.result;
      if (!arrayBuffer || !(arrayBuffer instanceof ArrayBuffer)) {
        reject(new Error('FileReader returned invalid result'));
        return;
      }
      
      const uint8Array = new Uint8Array(arrayBuffer);
      if (uint8Array.length === 0) {
        reject(new Error('File data is empty'));
        return;
      }
      
      resolve(uint8Array);
    };
    reader.onerror = () => reject(new Error('FileReader error'));
    reader.readAsArrayBuffer(file);
  });
};

const getFileType = (file: File): 'image' | 'mesh' | 'unknown' => {
  const ext = file.name.toLowerCase().split('.').pop() || '';
  if (file.type.startsWith('image/') || ['svg', 'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'tga', 'targa'].includes(ext)) {
    return 'image';
  }
  if (['obj', 'stl', 'fbx', 'gltf', 'glb', 'ply'].includes(ext)) {
    return 'mesh';
  }
  return 'unknown';
};

export default function ProcessingPanel({ files, wasm, onProcess, setIsProcessing }: ProcessingPanelProps) {
  const [quality, setQuality] = useState(75);
  const [outputFormat, setOutputFormat] = useState('auto');
  const [preserveMetadata, setPreserveMetadata] = useState(false);
  const [losslessMode, setLosslessMode] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [isProcessing, setLocalProcessing] = useState(false);

  const processFiles = useCallback(async () => {
    if (!wasm.available || files.length === 0) return;

    setLocalProcessing(true);
    setIsProcessing(true);
    setProgress(0);

    try {
      if (wasm.set_preserve_metadata) wasm.set_preserve_metadata(preserveMetadata);
      const shouldUseLossless = losslessMode || quality >= 95;
      if (wasm.set_lossless_mode) wasm.set_lossless_mode(shouldUseLossless);
    } catch (e) {
      console.warn('Configuration setting failed:', e);
    }

    const results: ProcessedResult[] = [];

    try {
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        setCurrentFile(file.name);
        setProgress(((i + 1) / files.length) * 100);

        try {
          const fileData = await readFileAsBytes(file);
          if (!fileData || fileData.length === 0) {
            throw new Error(`File data is empty for ${file.name}`);
          }

          let optimizedData: Uint8Array;
          let targetFormat: string;
          const fileType = getFileType(file);
          const fileExtension = file.name.toLowerCase().split('.').pop() || '';

          if (fileType === 'image') {
            let detectedFormat = 'unknown';
            if (wasm.detect_format) {
              try {
                detectedFormat = wasm.detect_format(fileData);
              } catch {
                detectedFormat = 'unknown';
              }
            }

            let inputFormat: string;
            if (detectedFormat && detectedFormat !== 'unknown' && !detectedFormat.startsWith('image:')) {
              inputFormat = detectedFormat;
            } else if (detectedFormat.startsWith('image:')) {
              inputFormat = detectedFormat.split(':')[1];
            } else if (fileExtension === 'svg') {
              inputFormat = 'svg';
            } else if (file.type.includes('/')) {
              inputFormat = file.type.split('/')[1];
            } else {
              inputFormat = fileExtension;
            }
            
            targetFormat = outputFormat === 'auto' ? inputFormat : outputFormat;

            if (outputFormat === 'auto') {
              if (wasm.optimize_auto) {
                optimizedData = wasm.optimize_auto(fileData, quality);
              } else if (wasm.optimize_image) {
                optimizedData = wasm.optimize_image(fileData, quality);
              } else {
                throw new Error('No optimization function available');
              }
            } else if (outputFormat !== inputFormat) {
              const converterKey = `convert_to_${outputFormat}` as keyof WasmHook;
              const converter = wasm[converterKey];
              if (typeof converter === 'function') {
                if (outputFormat === 'png') {
                  optimizedData = (converter as (data: Uint8Array) => Uint8Array)(fileData);
                } else {
                  optimizedData = (converter as (data: Uint8Array, quality: number) => Uint8Array)(fileData, quality);
                }
              } else {
                throw new Error(`Conversion to ${outputFormat} not supported`);
              }
            } else {
              const optimizerKey = `optimize_${outputFormat}` as keyof WasmHook;
              const optimizer = wasm[optimizerKey];
              if (typeof optimizer === 'function') {
                optimizedData = (optimizer as (data: Uint8Array, quality: number) => Uint8Array)(fileData, quality);
              } else if (wasm.optimize_image) {
                optimizedData = wasm.optimize_image(fileData, quality);
              } else {
                throw new Error('No optimization function available');
              }
            }

            results.push({
              originalFile: file,
              originalData: fileData,
              optimizedData,
              targetFormat,
              originalSize: fileData.length,
              optimizedSize: optimizedData.length,
              savings: ((fileData.length - optimizedData.length) / fileData.length) * 100,
              success: true,
              fileType: 'image',
              performanceMetrics: wasm.get_performance_metrics?.() || null
            });

          } else if (fileType === 'mesh') {
            targetFormat = fileExtension;
            const targetRatio = Math.max((100 - quality) / 100, 0.1);

            type MeshOptimizer = (data: Uint8Array, ratio: number) => Uint8Array;
            const optimizers: Record<string, MeshOptimizer | undefined> = {
              obj: wasm.optimize_obj,
              stl: wasm.optimize_stl,
              fbx: wasm.optimize_fbx,
              gltf: wasm.optimize_gltf,
              glb: wasm.optimize_gltf,
              ply: wasm.optimize_ply
            };

            const optimizer = optimizers[fileExtension] || wasm.optimize_mesh;
            if (!optimizer) {
              throw new Error(`No optimizer available for ${fileExtension.toUpperCase()} format`);
            }

            optimizedData = optimizer(fileData, targetRatio);

            results.push({
              originalFile: file,
              originalData: fileData,
              optimizedData,
              targetFormat,
              originalSize: fileData.length,
              optimizedSize: optimizedData.length,
              savings: ((fileData.length - optimizedData.length) / fileData.length) * 100,
              success: true,
              fileType: 'mesh',
              performanceMetrics: wasm.get_performance_metrics?.() || null
            });
          } else {
            throw new Error(`Unsupported file format: ${fileExtension}`);
          }
        } catch (error) {
          results.push({
            originalFile: file,
            error: String(error),
            success: false,
            fileType: getFileType(file)
          });
        }
      }

      if (wasm.check_performance_compliance) {
        try {
          const isCompliant = wasm.check_performance_compliance();
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
              <p className="truncate max-w-xs">{currentFile}</p>
              <p className="text-right">{Math.round(progress)}%</p>
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
        <div>
          <div className="flex items-center justify-between mb-3">
            <label htmlFor="quality" className="text-xs text-neutral-400">Quality</label>
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
          
          <style>{`
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
            .quality-slider::-moz-range-thumb {
              width: 14px;
              height: 14px;
              border-radius: 50%;
              background: #ffffff;
              border: 2px solid #525252;
              cursor: pointer;
              box-shadow: 0 1px 3px rgba(0,0,0,0.4);
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

        <div>
          <label htmlFor="format" className="block text-xs text-neutral-400 mb-1.5">Output Format</label>
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
              <div className="text-neutral-500 text-xs">Keep EXIF, XMP, and other metadata (larger file size)</div>
            </div>
          </label>
        </div>

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
              <div className="text-neutral-500 text-xs">Force lossless compression (overrides quality setting)</div>
            </div>
          </label>
        </div>

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
