'use client';

import { useState, useCallback } from 'react';

export default function FileDropZone({ selectedFiles, setSelectedFiles, onFileSelect }) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDrop = useCallback((e) => {
    e.preventDefault();
    setIsDragging(false);
    
    const files = Array.from(e.dataTransfer.files);
    handleFiles(files);
  }, []);

  const handleFileSelect = useCallback((e) => {
    const files = Array.from(e.target.files);
    handleFiles(files);
  }, []);

  const handleFiles = useCallback((files) => {
    // Filter supported files
    const supportedFiles = files.filter(file => {
      const fileName = file.name.toLowerCase();
      const fileType = file.type.toLowerCase();
      
      // Check for images - all supported formats
      if (fileType.startsWith('image/')) {
        return true;
      }
      
      // Check for PDF files (for image extraction)
      if (fileType === 'application/pdf' || fileName.endsWith('.pdf')) {
        return true;
      }
      
      const imageExtensions = [
        '.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp', '.ico', '.tiff', '.tif', '.svg', '.tga', '.ico'
      ];
      
      if (imageExtensions.some(ext => fileName.endsWith(ext))) {
        return true;
      }
      
      const meshExtensions = [
        '.obj', '.ply', '.gltf', '.glb', '.fbx', '.stl'
      ];
      
      if (meshExtensions.some(ext => fileName.endsWith(ext))) {
        return true;
      }
      
      return false;
    });

    if (supportedFiles.length === 0) {
  alert('Please select valid files:\n• Images: PNG, JPEG, WebP, GIF, BMP, ICO, TIFF, SVG, TGA\n• 3D Models: OBJ, PLY, STL, GLTF, GLB, FBX');
      return;
    }

    // Add new files to existing selection
    const newFiles = supportedFiles.filter(newFile => 
      !selectedFiles.some(existing => existing.name === newFile.name)
    );

    if (newFiles.length > 0) {
      setSelectedFiles(prev => [...prev, ...newFiles]);
      
      // Auto-select first file for preview
      if (onFileSelect && newFiles.length > 0) {
        onFileSelect(newFiles[0]);
      }
    }
  }, [selectedFiles, setSelectedFiles, onFileSelect]);

  const removeFile = useCallback((index) => {
    setSelectedFiles(prev => prev.filter((_, i) => i !== index));
  }, [setSelectedFiles]);

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  return (
    <div className="rounded-sm overflow-hidden">
      <div className="p-4 border-b border-neutral-800">
        <h3 className="text-sm font-normal text-white text-center">Files</h3>
      </div>
      
      {/* Drop Zone */}
      <div
        className={`p-8 text-center transition-colors cursor-pointer ${
          selectedFiles.length === 0 ? 'border-b border-dashed border-neutral-800' : ''
        } ${
          isDragging 
            ? 'bg-neutral-900/50' 
            : 'hover:bg-neutral-900/20'
        }`}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onClick={() => document.getElementById('file-input').click()}
      >
        <div className="flex flex-col items-center justify-center">
          <div className="text-2xl mb-2">🗁</div>
          <h3 className="text-sm font-normal text-white mb-2">Drop files or click to upload</h3>
          <p className="text-[10px] mb-4 text-neutral-500">OBJ, PLY, STL, GLTF, GLB, FBX, PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, TGA, ICO</p>
          <button className="px-4 py-2 border border-white text-white text-sm rounded hover:bg-white hover:text-black transition-colors">
            Choose Files
          </button>
        </div>
        <input
          id="file-input"
          type="file"
          multiple
          accept=".png,.jpg,.jpeg,.gif,.webp,.bmp,.ico,.tiff,.tif,.svg,.tga,.obj,.ply,.gltf,.glb,.fbx,.stl"
          onChange={handleFileSelect}
          className="hidden"
        />
      </div>

      {/* Selected Files */}
      {selectedFiles.length > 0 && (
        <div className="p-4">
          <div className="flex items-center justify-between mb-3">
            <h4 className="text-xs text-neutral-400">
              {selectedFiles.length} file{selectedFiles.length !== 1 ? 's' : ''} selected
            </h4>
            <button 
              onClick={() => setSelectedFiles([])} 
              className="text-xs text-neutral-500 hover:text-white"
            >
              Clear all
            </button>
          </div>
          <div className="space-y-2 max-h-56 overflow-y-auto pr-1">
            {selectedFiles.map((file, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-2.5 border border-neutral-800 rounded hover:border-neutral-700 transition-colors"
                onClick={() => onFileSelect && onFileSelect(file)}
              >
                <div className="flex items-center space-x-2.5">
                  <div className="text-lg">
                    {(() => {
                      const fileName = file.name.toLowerCase();
                      const isImage = file.type.startsWith('image/') || 
                                    fileName.endsWith('.svg') || 
                                    fileName.endsWith('.tga') || 
                                    fileName.endsWith('.ico') || 
                                    fileName.endsWith('.png') || 
                                    fileName.endsWith('.jpg') || 
                                    fileName.endsWith('.jpeg') || 
                                    fileName.endsWith('.gif') || 
                                    fileName.endsWith('.webp') || 
                                    fileName.endsWith('.bmp') || 
                                    fileName.endsWith('.tiff') || 
                                    fileName.endsWith('.tif');
                      return isImage ? '🖼️' : '🧊';
                    })()}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-xs text-white truncate max-w-[240px]">
                      {file.name}
                    </p>
                    <p className="text-xs text-neutral-500">
                      {(file.size / 1024 / 1024).toFixed(2)} MB
                    </p>
                  </div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    removeFile(index);
                  }}
                  className="ml-2 text-neutral-500 hover:text-red-400 p-1.5 transition-colors"
                  title="Remove file"
                >
                  ✕
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
