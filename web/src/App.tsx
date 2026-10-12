import { useState, useMemo, useCallback, useEffect } from 'react';
import { usePixie } from './core/usePixie';
import FileDropZone from './components/FileDropZone';
import ProcessingPanel from './components/ProcessingPanel';
import ResultsPanel from './components/ResultsPanel';
import FilePreview from './components/FilePreview';
import Icon, { Logo } from './components/Icon';
import type { BuildInfo } from './core/types';
import type { ProcessedResult } from './types';
import { outputName } from './utils/files';
import { formatBytes } from './utils/format';
import { outputFormat } from './utils/outputFormat';

type PreviewMode = 'original' | 'optimized';

/** Results together with the file list they were made for. */
interface Run {
  files: File[];
  results: ProcessedResult[];
}

function Header({ info }: { info: BuildInfo | null }) {
  return (
    <header className="topbar">
      <div className="brand">
        <Logo />
        <h1>Pixie Juice</h1>
        <span className="chip">alpha</span>
        {info && (
          <span className="chip" title={`Built ${info.timestamp}`} data-testid="version">
            v{info.version}
            {info.commit !== 'unknown' && ` · ${info.commit}`}
          </span>
        )}
      </div>
      <a
        className="icon-button"
        href="https://github.com/rmguney/pixie-juice"
        target="_blank"
        rel="noopener noreferrer"
        aria-label="Pixie Juice on GitHub (opens in new tab)"
      >
        <Icon name="github" size={18} />
      </a>
    </header>
  );
}

export default function App() {
  const { api, info, error } = usePixie();
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [run, setRun] = useState<Run | null>(null);
  const [focusedFile, setFocusedFile] = useState<File | null>(null);
  const [previewMode, setPreviewMode] = useState<PreviewMode>('original');

  // Changing the file list discards results (a run that finishes for an
  // earlier list is ignored below) and shows originals again.
  const updateFiles = useCallback((update: React.SetStateAction<File[]>) => {
    setSelectedFiles(update);
    setRun(null);
    setPreviewMode('original');
  }, []);

  // A file dropped outside the drop zone would otherwise replace the app with the file itself.
  useEffect(() => {
    const ignore = (event: DragEvent) => {
      if (event.dataTransfer?.types.includes('Files')) event.preventDefault();
    };
    window.addEventListener('dragover', ignore);
    window.addEventListener('drop', ignore);
    return () => {
      window.removeEventListener('dragover', ignore);
      window.removeEventListener('drop', ignore);
    };
  }, []);
  const results = run?.files === selectedFiles ? run.results : null;

  const selectedFile = focusedFile && selectedFiles.includes(focusedFile) ? focusedFile : (selectedFiles[0] ?? null);

  const optimizedFile = useMemo<File | null>(() => {
    const match = results?.find((r) => r.file === selectedFile);
    if (!selectedFile || !match?.success) return null;
    if (!match.changed) return selectedFile;
    const { extension, mimeType } = outputFormat(match.format);
    return new File([match.data as BlobPart], outputName(selectedFile.name, extension), { type: mimeType });
  }, [selectedFile, results]);

  const showOptimized = previewMode === 'optimized' && optimizedFile !== null;
  // The file in the preview; null exactly when there are no files (the landing page).
  const previewFile = showOptimized ? optimizedFile : selectedFile;

  if (error) {
    return (
      <div className="splash" role="alert">
        <Logo size={32} />
        <h1>Couldn't load the engine</h1>
        <p>{error}</p>
        <button type="button" className="button" onClick={() => window.location.reload()}>
          Retry
        </button>
      </div>
    );
  }

  if (!api) {
    return (
      <div className="splash" role="status" aria-live="polite">
        <span className="pulse">
          <Logo size={32} />
        </span>
        <p>Loading engine…</p>
      </div>
    );
  }

  return (
    <>
      <Header info={info} />
      <main className="main">
        {/* One FileDropZone in the same place for both layouts, so it keeps its state (such as
            the "not supported" message) when the first files switch the landing page to the workspace. */}
        <div className={previewFile ? 'workspace' : 'hero'}>
          {!previewFile && (
            <>
              <p className="eyebrow">Images and 3D models</p>
              <h2 className="hero-title">
                Lighter files,
                <br />
                <span className="juice-text">same look.</span>
              </h2>
              <p className="lede">
                Optimize and convert images and meshes in your browser. A C++ engine compiled to WebAssembly does the
                work, so nothing is uploaded.
              </p>
            </>
          )}
          <FileDropZone
            selectedFiles={selectedFiles}
            setSelectedFiles={updateFiles}
            activeFile={selectedFile}
            onFileSelect={setFocusedFile}
          />
          {previewFile ? (
            <section className="card preview" aria-label="Preview">
              <div className="card-head">
                <p className="preview-name">
                  {previewFile.name} <span className="count mono">{formatBytes(previewFile.size)}</span>
                </p>
                {optimizedFile && (
                  <div role="tablist" aria-label="Preview source" className="segmented">
                    {(['original', 'optimized'] as const).map((mode) => (
                      <button
                        key={mode}
                        type="button"
                        role="tab"
                        aria-selected={showOptimized === (mode === 'optimized')}
                        onClick={() => setPreviewMode(mode)}
                      >
                        {mode === 'original' ? 'Original' : 'Optimized'}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <FilePreview file={previewFile} />
            </section>
          ) : (
            <ul className="facts">
              <li>Runs on your device</li>
              <li>Every output verified against the original</li>
              <li>Open source</li>
            </ul>
          )}
          {previewFile &&
            (results ? (
              <ResultsPanel results={results} onReset={() => updateFiles([])} />
            ) : (
              <ProcessingPanel files={selectedFiles} api={api} onProcess={(files, results) => setRun({ files, results })} />
            ))}
        </div>
      </main>
    </>
  );
}
