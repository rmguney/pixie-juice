import { useState, useEffect, useLayoutEffect, useRef } from 'react';
import type { CoreOptions, PixieApi } from '../core/types';
import type { OutputChoice, ProcessingPanelProps, ProcessedResult } from '../types';
import { extensionOf, fileKind } from '../utils/files';

const OUTPUT_FORMATS: { value: OutputChoice; label: string; hint: string }[] = [
  { value: 'auto', label: 'Auto (keep format)', hint: "Optimizes within each file's own format; keeps the original if nothing is smaller." },
  { value: 'png', label: 'PNG (lossless)', hint: 'Lossless, with the smallest colour type and palette.' },
  { value: 'jpeg', label: 'JPEG (lossy)', hint: 'For photos; transparency is flattened onto white.' },
  { value: 'webp', label: 'WebP', hint: 'Lossy at the chosen quality, lossless at 100%.' },
  { value: 'gif', label: 'GIF', hint: 'Up to 256 colours.' },
  { value: 'bmp', label: 'BMP', hint: 'Uncompressed.' },
  { value: 'tga', label: 'TGA', hint: 'Run-length encoded.' },
  { value: 'ico', label: 'ICO', hint: 'Icon, scaled to fit 256 px.' },
  { value: 'tiff', label: 'TIFF', hint: 'Deflate compressed, lossless.' },
];

/** The track's filled share, for the slider styles. */
const fill = (value: number, min: number, max: number) =>
  ({ '--fill': `${((value - min) / (max - min)) * 100}%` }) as React.CSSProperties;

/** Raster images can change format; meshes and SVG are optimized in their own. */
const convertible = (name: string) => fileKind(name) === 'image' && extensionOf(name) !== 'svg';

/**
 * Input bytes in flight at once. Decoding takes many times a file's size, so
 * large files go one or two at a time; a file over the budget runs alone.
 */
const BYTES_IN_FLIGHT = 32 * 1024 * 1024;

async function processFile(api: PixieApi, file: File, output: OutputChoice, options: CoreOptions): Promise<ProcessedResult> {
  try {
    // A fresh copy of the file, so it can move to the worker instead of being copied again.
    const input = new Uint8Array(await file.arrayBuffer());
    const result =
      output !== 'auto' && convertible(file.name)
        ? await api.convert(input, output, options, true)
        : await api.optimize(input, options, true);
    return { file, success: true, data: result.data, format: result.format, changed: result.changed };
  } catch (error) {
    return { file, success: false, error: error instanceof Error ? error.message : String(error) };
  }
}

interface Progress {
  done: number;
  total: number;
  /** The files being processed now, in list order. */
  active: string[];
}

export default function ProcessingPanel({ files, api, settings, onSettingsChange, onProcess }: ProcessingPanelProps) {
  const { quality, meshDetail, output, preserveMetadata, lossless } = settings;
  const change = (update: Partial<typeof settings>) => onSettingsChange({ ...settings, ...update });
  // Progress of the batch being processed; the file list itself may change meanwhile.
  const [progress, setProgress] = useState<Progress | null>(null);
  // A run stops when the panel goes away (reset or all files removed) or the list changes. A
  // layout effect, so no file starts between the list changing on screen and the run seeing it.
  const current = useRef<File[] | null>(files);
  useLayoutEffect(() => {
    current.current = files;
    return () => {
      current.current = null;
    };
  }, [files]);

  // The Optimize button goes away when a run starts: focus moves to the progress heading.
  const progressHeading = useRef<HTMLHeadingElement>(null);
  const running = progress !== null;
  useEffect(() => {
    if (running && (!document.activeElement || document.activeElement === document.body)) progressHeading.current?.focus();
  }, [running]);

  const hasMeshes = files.some((file) => fileKind(file.name) === 'mesh');
  const hasOwnFormatFiles = files.some((file) => !convertible(file.name));
  const label = `Optimize ${files.length} file${files.length === 1 ? '' : 's'}`;

  /** Processes the files several at a time; results stay in list order. */
  async function processFiles() {
    const options: CoreOptions = { quality, lossless, preserveMetadata, meshRatio: meshDetail / 100 };
    const results: ProcessedResult[] = new Array(files.length);
    const active = new Set<number>();
    let next = 0;
    let done = 0;
    let bytes = 0;
    const stopped = () => current.current !== files;
    const report = () =>
      setProgress({ done, total: files.length, active: [...active].sort((a, b) => a - b).map((i) => files[i].name) });

    await new Promise<void>((finish) => {
      const launch = () => {
        // A changed list discards the run's results: start nothing more, let the rest finish.
        while (
          !stopped() &&
          next < files.length &&
          active.size < api.concurrency &&
          (active.size === 0 || bytes + files[next].size <= BYTES_IN_FLIGHT)
        ) {
          const index = next++;
          active.add(index);
          bytes += files[index].size;
          void processFile(api, files[index], output, options).then((result) => {
            results[index] = result;
            active.delete(index);
            bytes -= files[index].size;
            done++;
            if (!stopped()) report();
            launch();
            if (active.size === 0) finish();
          });
        }
      };
      launch();
      report();
      if (active.size === 0) finish();
    });

    if (stopped()) {
      if (current.current) setProgress(null);
      return;
    }
    setProgress(null);
    onProcess(files, results);
  }

  if (progress) {
    const percent = Math.round((progress.done / progress.total) * 100);
    return (
      <section className="card panel" aria-labelledby="panel-title">
        <div className="card-head">
          <h2 className="card-title" id="panel-title" ref={progressHeading} tabIndex={-1}>
            Optimizing
            <span className="count" aria-live="polite">
              {progress.done} of {progress.total} done
            </span>
          </h2>
        </div>
        <div className="panel-body">
          <div
            className="progress"
            role="progressbar"
            aria-label="Processing progress"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={percent}
          >
            <span style={{ width: `${percent}%` }} />
          </div>
          <div className="progress-meta">
            <span>{progress.active.join(', ')}</span>
            <span className="mono">{percent}%</span>
          </div>
        </div>
      </section>
    );
  }

  const outputHint = OUTPUT_FORMATS.find((f) => f.value === output)?.hint;

  return (
    <section className="card panel" aria-labelledby="panel-title">
      <div className="card-head">
        <h2 className="card-title" id="panel-title">
          Settings
        </h2>
      </div>

      <div className="panel-body">
        <div className="field">
          <div className="field-head">
            <label htmlFor="quality" className="label">Quality</label>
            <span className="value">{quality}</span>
          </div>
          <input
            id="quality"
            type="range"
            min="10"
            max="100"
            value={quality}
            onChange={(e) => change({ quality: Number(e.target.value) })}
            className="range"
            style={fill(quality, 10, 100)}
          />
          <p className="hint">Lossy formats re-encode at this quality; lower allows more change. PNG, GIF and TIFF stay lossless.</p>
        </div>

        {hasMeshes && (
          <div className="field">
            <div className="field-head">
              <label htmlFor="mesh-detail" className="label">Mesh triangles kept</label>
              <span className="value">{meshDetail}%</span>
            </div>
            <input
              id="mesh-detail"
              type="range"
              min="10"
              max="100"
              value={meshDetail}
              onChange={(e) => change({ meshDetail: Number(e.target.value) })}
              className="range"
              style={fill(meshDetail, 10, 100)}
            />
          </div>
        )}

        <div className="field">
          <label htmlFor="format" className="label">Output format</label>
          <select id="format" value={output} onChange={(e) => change({ output: e.target.value as OutputChoice })} className="select">
            {OUTPUT_FORMATS.map((f) => (
              <option key={f.value} value={f.value}>{f.label}</option>
            ))}
          </select>
          <p className="hint">
            {outputHint}
            {output !== 'auto' && hasOwnFormatFiles && ' Meshes and SVG are always optimized in their own format.'}
          </p>
        </div>

        <div className="divider" />

        <label className="toggle">
          <span className="toggle-text">
            <span className="label">Preserve metadata</span>
            <span className="hint">Keep EXIF, XMP and comments. Orientation and colour profiles are always kept.</span>
          </span>
          <input
            type="checkbox"
            role="switch"
            className="switch"
            checked={preserveMetadata}
            onChange={(e) => change({ preserveMetadata: e.target.checked })}
          />
        </label>

        <label className="toggle">
          <span className="toggle-text">
            <span className="label">Lossless</span>
            <span className="hint">Never change pixels or geometry; conversions to formats that can't hold the image exactly fail.</span>
          </span>
          <input
            type="checkbox"
            role="switch"
            className="switch"
            checked={lossless}
            onChange={(e) => change({ lossless: e.target.checked })}
          />
        </label>

        <button type="button" className="button primary wide" onClick={processFiles}>
          {label}
        </button>
      </div>
    </section>
  );
}
