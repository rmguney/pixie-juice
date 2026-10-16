import { useState, useMemo, useCallback, useEffect } from 'react';
import { usePixie } from './core/usePixie';
import FileDropZone from './components/FileDropZone';
import ProcessingPanel from './components/ProcessingPanel';
import ResultsPanel from './components/ResultsPanel';
import FilePreview, { ImageCompare } from './components/FilePreview';
import Icon, { Logo } from './components/Icon';
import type { BuildInfo } from './core/types';
import { DEFAULT_SETTINGS, type ProcessedResult, type Settings } from './types';
import { fileKind, outputName } from './utils/files';
import { formatBytes } from './utils/format';
import { outputFormat } from './utils/outputFormat';

type PreviewMode = 'original' | 'compare' | 'optimized';

const PREVIEW_MODES: { mode: PreviewMode; label: string }[] = [
  { mode: 'original', label: 'Original' },
  { mode: 'compare', label: 'Compare' },
  { mode: 'optimized', label: 'Optimized' },
];

/** Each result's file, made once (copying its bytes) rather than on every selection; it goes with its result. */
const resultFiles = new WeakMap<ProcessedResult, File>();

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
        {info && (
          <span className="chip" title={`Engine ${info.version}, built ${info.timestamp}`} data-testid="version">
            {__SITE_VERSION__}
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
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);

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
    let file = resultFiles.get(match);
    if (!file) {
      const { extension, mimeType } = outputFormat(match.format);
      file = new File([match.data as BlobPart], outputName(selectedFile.name, extension), { type: mimeType });
      resultFiles.set(match, file);
    }
    return file;
  }, [selectedFile, results]);

  // Images can be compared on a split view; a mesh shows one side at a time.
  const modes = optimizedFile
    ? PREVIEW_MODES.filter(({ mode }) => mode !== 'compare' || fileKind(selectedFile!.name) === 'image')
    : [];
  const mode = modes.some((m) => m.mode === previewMode) ? previewMode : 'original';
  const showOptimized = mode === 'optimized';
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
              <h2 className="hero-title">
                Lighter files,
                <br />
                <span className="juice-text">same look.</span>
              </h2>
              <p className="lede">
                Shrink images and 3D models in seconds, right in your browser. Your files never leave your device, and
                every result is checked against the original so it keeps its quality. Free and open source.
              </p>
            </>
          )}
          <FileDropZone
            selectedFiles={selectedFiles}
            setSelectedFiles={updateFiles}
            activeFile={selectedFile}
            onFileSelect={setFocusedFile}
          />
          {previewFile && (
            <section className="card preview" aria-label="Preview">
              <div className="card-head">
                <p className="preview-name">
                  {previewFile.name}{' '}
                  <span className="count mono">
                    {mode === 'compare'
                      ? `${formatBytes(selectedFile!.size)} → ${formatBytes(optimizedFile!.size)}`
                      : formatBytes(previewFile.size)}
                  </span>
                </p>
                {modes.length > 0 && (
                  <div role="group" aria-label="Preview source" className="segmented">
                    {modes.map(({ mode: option, label }) => (
                      <button
                        key={option}
                        type="button"
                        aria-pressed={mode === option}
                        onClick={() => setPreviewMode(option)}
                      >
                        {label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              {mode === 'compare' ? (
                <ImageCompare original={selectedFile!} optimized={optimizedFile!} />
              ) : (
                <FilePreview file={previewFile} />
              )}
            </section>
          )}
          {previewFile &&
            (results ? (
              <ResultsPanel results={results} onReset={() => updateFiles([])} />
            ) : (
              <ProcessingPanel
                files={selectedFiles}
                api={api}
                settings={settings}
                onSettingsChange={setSettings}
                onProcess={(files, results) => {
                  setRun({ files, results });
                  setPreviewMode('compare');
                }}
              />
            ))}
        </div>
      </main>
    </>
  );
}
