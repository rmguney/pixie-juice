'use client';

import { useCallback } from 'react';

export default function ResultsPanel({ results, onReset }) {
  const downloadFile = useCallback((result) => {
    if (!result.success) return;

    const mimeType = result.fileType === 'image' 
      ? `image/${result.targetFormat}` 
      : 'application/octet-stream';

    const blob = new Blob([result.optimizedData], { type: mimeType });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    
    const extension = result.fileType === 'image' ? result.targetFormat : result.targetFormat;
    const baseName = result.originalFile.name.replace(/\.[^/.]+$/, '');
    a.download = `${baseName}_juice.${extension}`;
    
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    
    URL.revokeObjectURL(url);
  }, []);

  const downloadAll = useCallback(() => {
    const successfulResults = results.filter(r => r.success);
    
    successfulResults.forEach((result, index) => {
      setTimeout(() => {
        downloadFile(result);
      }, index * 100); // Small delay between downloads
    });
  }, [results, downloadFile]);

  const formatFileSize = useCallback((bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }, []);

  const totalOriginalSize = results.reduce((sum, r) => r.success ? sum + r.originalSize : sum, 0);
  const totalOptimizedSize = results.reduce((sum, r) => r.success ? sum + r.optimizedSize : sum, 0);
  const totalSavings = totalOriginalSize > 0 ? ((totalOriginalSize - totalOptimizedSize) / totalOriginalSize) * 100 : 0;
  const successCount = results.filter(r => r.success).length;

  return (
    <div className="rounded-lg overflow-hidden">
      <div className="p-[11px] border-b border-neutral-800 flex items-center justify-between">
        <h3 className="text-sm font-normal text-white flex-1 text-center">Results</h3>
        <div className="flex space-x-2">
          <button
            onClick={downloadAll}
            disabled={successCount === 0}
            className="px-3 py-1.5 bg-black hover:bg-neutral-900 border border-neutral-800 text-white text-xs rounded transition-colors disabled:text-neutral-500"
          >
            Download All
          </button>
          <button
            onClick={onReset}
            className="px-3 py-1.5 border border-neutral-700 text-neutral-400 text-xs rounded hover:border-neutral-500 hover:text-white transition-colors"
          >
            Reset
          </button>
        </div>
      </div>

      {/* Summary */}
      {successCount > 0 && (
        <div className="p-4 border-b border-neutral-800 bg-neutral-900/40">
          <div className="flex items-center space-x-2 mb-1">
            <span className="text-green-500">✓</span>
            <span className="text-sm text-white">
              {successCount} file{successCount !== 1 ? 's' : ''} optimized
            </span>
          </div>
          <div className="flex items-center space-x-2 text-sm text-neutral-400">
            <span>{formatFileSize(totalOriginalSize)} → {formatFileSize(totalOptimizedSize)}</span>
            <span className={`font-medium ${totalSavings > 0 ? 'text-green-500' : 'text-red-500'}`}>
              ({totalSavings > 0 ? '-' : '+'}{Math.abs(totalSavings).toFixed(1)}%)
            </span>
          </div>
        </div>
      )}

      {/* Results List */}
      <div className="max-h-80 overflow-y-auto p-2">
        <div className="space-y-2">
          {results.map((result, index) => (
            <div
              key={index}
              className={`border rounded p-3 ${
                result.success ? 'border-neutral-800 hover:border-neutral-700' : 'border-red-900/30'
              } transition-colors`}
            >
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-1">
                    <span className="text-sm">
                      {result.fileType === 'image' ? '🖼️' : '🧊'}
                    </span>
                    <span className="text-xs text-white truncate max-w-[240px]">
                      {result.originalFile.name}
                    </span>
                  </div>
                  
                  {result.success ? (
                    <div className="space-y-1">
                      <div className="flex items-center space-x-2 text-xs text-neutral-500">
                        <span>{formatFileSize(result.originalSize)} → {formatFileSize(result.optimizedSize)}</span>
                        <span className={`${
                          result.savings > 0 ? 'text-green-500' : 'text-red-500'
                        }`}>
                          ({result.savings > 0 ? '-' : '+'}{Math.abs(result.savings).toFixed(1)}%)
                        </span>
                        {result.targetFormat && (
                          <span className="text-neutral-400">
                            → {result.targetFormat.toUpperCase()}
                          </span>
                        )}
                      </div>
                      
                      {/* Performance metrics display */}
                      {result.performanceMetrics && (
                        <div className="text-xs text-neutral-600">
                          Performance: {result.performanceMetrics.last_operation_time_ms?.toFixed(1) || 'N/A'}ms
                          {result.performanceMetrics.memory_peak_mb && 
                            ` | Memory: ${result.performanceMetrics.memory_peak_mb.toFixed(1)}MB`
                          }
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="text-xs text-red-500 truncate max-w-[240px]">
                      Failed: {result.error}
                    </div>
                  )}
                </div>

                {result.success && (
                  <button
                    onClick={() => downloadFile(result)}
                    className="ml-3 px-2 py-1 border border-white bg-black text-white hover:bg-white hover:text-black text-xs rounded transition-colors"
                  >
                    ▼
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
