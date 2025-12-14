import { useState, useEffect } from 'react';
import { useWasm } from './hooks/useWasm';
import FileDropZone from './components/FileDropZone';
import ProcessingPanel from './components/ProcessingPanel';
import ResultsPanel from './components/ResultsPanel';
import FilePreview from './components/FilePreview';
import type { ProcessedResult, PerformanceMetrics } from './types';

export default function App() {
  const wasmHook = useWasm();
  const { loading, error } = wasmHook;
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [processedResults, setProcessedResults] = useState<ProcessedResult[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [performanceStats, setPerformanceStats] = useState<PerformanceMetrics | null>(null);

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

  useEffect(() => {
    if (wasmHook.available) {
      if (wasmHook.get_performance_metrics) {
        try {
          const stats = wasmHook.get_performance_metrics();
          setPerformanceStats(stats);
        } catch (e) {
          console.warn('Failed to get performance metrics:', e);
        }
      }
    }
  }, [wasmHook.available, wasmHook.get_performance_metrics]);

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
          <p className="text-neutral-400 text-sm mb-4">{error}</p>
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
      <header className="mb-6">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <h1 className="text-xl font-light tracking-widest">Pixie Juice</h1>
              <span className="ml-3 bg-neutral-900 text-neutral-500 text-[10px] px-1.5 py-0.5 rounded">alpha</span>
            </div>
            
            {performanceStats && (
              <div className="text-sm font-light tracking-widest text-neutral-300 hover:text-white transition-colors transition-300">
                <a href="https://github.com/rmguney/pixie-juice" target="_blank" rel="noopener noreferrer">/rmguney</a>
              </div>
            )}
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-4 pb-12">
        {selectedFiles.length === 0 ? (
          <div className="flex justify-center">
            <div className="max-w-md w-full">
              <FileDropZone 
                selectedFiles={selectedFiles}
                setSelectedFiles={setSelectedFiles}
                onFileSelect={setSelectedFile}
              />
            </div>
          </div>
        ) : (
          <div className="flex justify-center">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6 w-full">
              <div className="order-1 bg-black rounded-lg overflow-hidden">
                <FileDropZone 
                  selectedFiles={selectedFiles}
                  setSelectedFiles={setSelectedFiles}
                  onFileSelect={setSelectedFile}
                />
              </div>

              <div className="order-2 h-[300px] md:h-[400px] lg:h-[calc(100vh-150px)] bg-black rounded-lg overflow-hidden">
                <div className="p-4 border-b border-neutral-800">
                  <h3 className="text-sm font-normal text-white text-center">Preview</h3>
                </div>
                <div className="h-[calc(100%-56px)] p-4">
                  <FilePreview file={selectedFile} />
                </div>
              </div>

              <div className="order-3 md:order-3 lg:order-3">
                {!isProcessing && !processedResults.length && (
                  <ProcessingPanel
                    files={selectedFiles}
                    wasm={wasmHook}
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
