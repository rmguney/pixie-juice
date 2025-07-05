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
      <header className="border-b border-neutral-800 mb-6">
        <div className="max-w-4xl mx-auto px-4 py-4">
          <div className="flex items-center justify-center">
            <h1 className="text-xl font-light tracking-widest">Pixie Juice</h1>
            <span className="ml-3 bg-neutral-800 text-neutral-500 text-xs px-2 py-0.5 rounded">alpha</span>
          </div>
        </div>
      </header>

      <div className="max-w-4xl mx-auto px-4 pb-12">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Left Column: File Drop and Processing */}
          <div className="space-y-6">
            {/* File Drop Zone */}
            <FileDropZone 
              selectedFiles={selectedFiles}
              setSelectedFiles={setSelectedFiles}
              onFileSelect={setSelectedFile}
            />
            
            {/* Processing Panel */}
            {selectedFiles.length > 0 && !isProcessing && !processedResults.length && (
              <ProcessingPanel
                files={selectedFiles}
                wasm={wasm}
                onProcess={setProcessedResults}
                setIsProcessing={setIsProcessing}
              />
            )}

            {/* Results Panel */}
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

          {/* Right Column: Preview */}
          <div>
            <div className="mb-3 flex items-center">
              <h3 className="text-sm font-normal text-white">Preview</h3>
              {selectedFile && (
                <div className="flex-1 flex justify-end">
                  <select 
                    className="bg-black border border-neutral-800 text-white text-xs px-2 py-1 rounded"
                    value={selectedFiles.indexOf(selectedFile)}
                    onChange={(e) => setSelectedFile(selectedFiles[e.target.value])}
                    disabled={selectedFiles.length <= 1}
                  >
                    {selectedFiles.map((file, index) => (
                      <option key={index} value={index}>
                        {file.name}
                      </option>
                    ))}
                  </select>
                </div>
              )}
            </div>
            <FilePreview file={selectedFile} />
          </div>
        </div>
      </div>
    </div>
  );
}
