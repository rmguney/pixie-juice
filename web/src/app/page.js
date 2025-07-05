'use client';

import { useState, useEffect } from 'react';
import { useWasm } from './hooks/useWasm';
import FileDropZone from './components/FileDropZone';
import ProcessingPanel from './components/ProcessingPanel';
import ResultsPanel from './components/ResultsPanel';
import FilePreview from './components/FilePreview';

export default function PixieJuice() {
  const { wasm, loading, error } = useWasm();
  const [selectedFiles, setSelectedFiles] = useState([]);
  const [processedResults, setProcessedResults] = useState([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [selectedFile, setSelectedFile] = useState(null);

  useEffect(() => {
    if (selectedFiles.length > 0 && !selectedFile) {
      setSelectedFile(selectedFiles[0]);
    } else if (selectedFiles.length === 0) {
      setSelectedFile(null);
      setProcessedResults([]);
    } else if (selectedFile && !selectedFiles.includes(selectedFile)) {
      setSelectedFile(selectedFiles[0]);
    }
  }, [selectedFiles, selectedFile]);

  if (loading) {
    return (
      <div className="min-h-screen bg-black flex items-center justify-center">
        <div className="text-center">
          <div className="w-6 h-6 border border-white border-t-transparent rounded-full animate-spin mx-auto mb-3"></div>
          <p className="text-neutral-500 text-sm">Loading WASM module...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-black flex items-center justify-center">
        <div className="text-center max-w-md mx-auto p-4">
          <h2 className="text-base font-medium text-white mb-2">Failed to Load</h2>
          <p className="text-neutral-400 text-sm mb-4">{error.message}</p>
          <button 
            onClick={() => window.location.reload()} 
            className="px-4 py-2 border border-white text-white text-sm rounded hover:bg-white hover:text-black transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black text-white">
      {/* Header */}
      <header className="mb-6">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-center">
            <h1 className="text-xl font-light tracking-widest">Pixie Juice</h1>
            <span className="ml-3 bg-neutral-900 text-neutral-500 text-[10px] px-1.5 py-0.5 rounded">alpha</span>
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-4 pb-12">
        {selectedFiles.length === 0 ? (
          /* Center Files Section when no files selected */
          <div className="flex justify-center">
            <div className="max-w-md w-full">
              <FileDropZone 
                selectedFiles={selectedFiles}
                setSelectedFiles={setSelectedFiles}
                onFileSelect={setSelectedFile}
              />
              
              {/* Debug: Add test file button */}
              <div className="mt-4 text-center">
                <button 
                  onClick={() => {
                    // Create a simple test OBJ file
                    const objContent = `# Test OBJ
v -1.0 -1.0  1.0
v  1.0 -1.0  1.0
v  1.0  1.0  1.0
v -1.0  1.0  1.0
v -1.0 -1.0 -1.0
v  1.0 -1.0 -1.0
v  1.0  1.0 -1.0
v -1.0  1.0 -1.0

f 1 2 3 4
f 8 7 6 5
f 4 3 7 8
f 5 1 4 8
f 5 6 2 1
f 2 6 7 3`;
                    
                    console.log('Creating test OBJ file with content:', objContent);
                    
                    const blob = new Blob([objContent], { type: 'text/plain' });
                    const file = new File([blob], 'test-cube.obj', { type: 'text/plain' });
                    
                    console.log('Created test file:', file.name, file.size, 'bytes');
                    
                    setSelectedFiles([file]);
                    setSelectedFile(file);
                  }}
                  className="px-3 py-1 text-xs border border-neutral-600 text-neutral-400 rounded hover:border-neutral-500 hover:text-neutral-300 transition-colors"
                >
                  Test with Simple OBJ
                </button>
              </div>
            </div>
          </div>
        ) : (
          /* Responsive layout when files are selected */
          <div className="flex justify-center">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6 w-full">
              {/* First: Files */}
              <div className="order-1 bg-black rounded-lg overflow-hidden">
                <FileDropZone 
                  selectedFiles={selectedFiles}
                  setSelectedFiles={setSelectedFiles}
                  onFileSelect={setSelectedFile}
                />
              </div>

              {/* Second: Preview */}
              <div className="order-2 h-[300px] md:h-[400px] lg:h-[calc(100vh-150px)] bg-black rounded-lg overflow-hidden">
                <div className="p-4 border-b border-neutral-800">
                  <h3 className="text-sm font-normal text-white">Preview</h3>
                </div>
                <div className="h-[calc(100%-56px)] p-4">
                  <FilePreview file={selectedFile} />
                </div>
              </div>

              {/* Third: Processing/Results */}
              <div className="order-3 md:order-3 lg:order-3">
                {!isProcessing && !processedResults.length && (
                  <ProcessingPanel
                    files={selectedFiles}
                    wasm={wasm}
                    onProcess={setProcessedResults}
                    setIsProcessing={setIsProcessing}
                  />
                )}

                {processedResults.length > 0 && (
                  <ResultsPanel 
                    results={processedResults}
                    onReset={() => {
                      setSelectedFiles([]);
                      setProcessedResults([]);
                      setSelectedFile(null);
                    }}
                  />
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
