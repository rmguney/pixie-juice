import { useState, useEffect, useRef } from 'react';
import type { CoreOptions } from '../core/types';
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

export default function ProcessingPanel({ files, api, settings, onSettingsChange, onProcess }: ProcessingPanelProps) {
  const { quality, meshDetail, output, preserveMetadata, lossless } = settings;
  const change = (update: Partial<typeof settings>) => onSettingsChange({ ...settings, ...update });
  // Progress of the batch being processed; the file list itself may change meanwhile.
  const [progress, setProgress] = useState<{ done: number; total: number; current: string } | null>(null);
  // A run stops when the panel goes away (reset or all files removed) or the list changes.
  const current = useRef<File[] | null>(files);
  useEffect(() => {
    current.current = files;
    return () => {
      current.current = null;
    };
  }, [files]);

  const hasMeshes = files.some((file) => fileKind(file.name) === 'mesh');
  const hasOwnFormatFiles = files.some((file) => !convertible(file.name));
  const label = `Optimize ${files.length} file${files.length === 1 ? '' : 's'}`;

  async function processFiles() {
    const options: CoreOptions = { quality, lossless, preserveMetadata, meshRatio: meshDetail / 100 };
    const results: ProcessedResult[] = [];
    for (const [index, file] of files.entries()) {
      // Its results would be discarded anyway: stop instead of processing the rest.
      if (current.current !== files) {
        if (current.current) setProgress(null);
        return;
      }
      setProgress({ done: index, total: files.length, current: file.name });
      try {
        const input = new Uint8Array(await file.arrayBuffer());
        const result =
          output !== 'auto' && convertible(file.name)
            ? await api.convert(input, output, options)
            : await api.optimize(input, options);
        results.push({ file, success: true, data: result.data, format: result.format, changed: result.changed });
      } catch (error) {
        results.push({ file, success: false, error: error instanceof Error ? error.message : String(error) });
      }
    }

    setProgress(null);
    onProcess(files, results);
  }

  if (progress) {
    const percent = Math.round((progress.done / progress.total) * 100);
    return (
      <section className="card panel" aria-labelledby="panel-title">
        <div className="card-head">
          <h2 className="card-title" id="panel-title">
            Optimizing<span className="count">{progress.done + 1} of {progress.total}</span>
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
            <span>{progress.current}</span>
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
