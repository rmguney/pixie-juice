import { Component, lazy, Suspense, useState, useEffect, useRef, type CSSProperties, type ReactNode } from 'react';
import { getPixie } from '../core/client';
import type { FilePreviewProps, ImageCompareProps } from '../types';
import { extensionOf, isMeshExtension } from '../utils/files';

// three.js is only needed once a mesh is previewed.
const MeshPreview = lazy(() => import('./MeshPreview'));

/** Formats a browser <img> displays; TGA and TIFF go through the engine's decoder. */
const BROWSER_IMAGES = new Set(['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'ico', 'svg']);

type Preview = { kind: 'url'; url: string } | { kind: 'pixels'; image: ImageData };

/**
 * Recently decoded files' pixels, most recent last: switching between the
 * original, the result and the split view doesn't decode a TIFF or TGA again.
 */
const decodedPixels = new Map<File, Promise<ImageData>>();
const DECODED_KEPT = 4;

function pixelsOf(file: File): Promise<ImageData> {
  let pixels = decodedPixels.get(file);
  if (pixels) {
    decodedPixels.delete(file);
  } else {
    pixels = file
      .arrayBuffer()
      .then((buffer) => getPixie().decodeImage(new Uint8Array(buffer), true))
      .then(({ rgba, width, height }) => new ImageData(rgba, width, height));
    pixels.catch(() => decodedPixels.delete(file));
  }
  decodedPixels.set(file, pixels);
  if (decodedPixels.size > DECODED_KEPT) decodedPixels.delete(decodedPixels.keys().next().value!);
  return pixels;
}

interface PreviewState {
  file: File;
  preview: Preview | null;
  error: string | null;
}

/** An image file ready to show: an object URL for the browser, or pixels from the engine. */
function useImagePreview(file: File, enabled = true) {
  // Kept with the file it belongs to, so a new file never shows the last one's image.
  const [state, setState] = useState<PreviewState>({ file, preview: null, error: null });

  useEffect(() => {
    if (!enabled) return;
    setState({ file, preview: null, error: null });

    let cancelled = false;
    let url: string | null = null;
    (async () => {
      try {
        if (BROWSER_IMAGES.has(extensionOf(file.name))) {
          url = URL.createObjectURL(file);
          if (!cancelled) setState({ file, preview: { kind: 'url', url }, error: null });
        } else {
          const image = await pixelsOf(file);
          if (!cancelled) setState({ file, preview: { kind: 'pixels', image }, error: null });
        }
      } catch (err) {
        const message = `Cannot preview ${file.name}: ${err instanceof Error ? err.message : String(err)}`;
        if (!cancelled) setState({ file, preview: null, error: message });
      }
    })();

    return () => {
      cancelled = true;
      if (url) URL.revokeObjectURL(url);
    };
  }, [file, enabled]);

  const current = state.file === file;
  const fail = (message: string) => setState({ file, preview: null, error: `Cannot preview ${file.name}: ${message}` });
  return { preview: current ? state.preview : null, error: current ? state.error : null, fail };
}

function DecodedImage({ image, label, className }: { image: ImageData; label: string; className?: string }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    canvas.current?.getContext('2d')?.putImageData(image, 0, 0);
  }, [image]);
  return (
    <canvas
      ref={canvas}
      width={image.width}
      height={image.height}
      className={className ? `decoded ${className}` : 'decoded'}
      role="img"
      aria-label={label}
    />
  );
}

interface ImageViewProps {
  preview: Preview;
  label: string;
  className?: string;
  onError: (message: string) => void;
  /** An <img>'s natural size, once loaded. */
  onSize?: (width: number, height: number) => void;
}

function ImageView({ preview, label, className, onError, onSize }: ImageViewProps) {
  return preview.kind === 'url' ? (
    <img
      src={preview.url}
      alt={label}
      className={className}
      decoding="async"
      onLoad={(e) => onSize?.(e.currentTarget.naturalWidth, e.currentTarget.naturalHeight)}
      onError={() => onError('the browser could not decode it')}
    />
  ) : (
    <DecodedImage image={preview.image} label={label} className={className} />
  );
}

/** Shows why the 3D viewer failed (such as its code chunk failing to load after a new deploy) instead of unmounting the app. */
class PreviewBoundary extends Component<{ file: File; children: ReactNode }, { error: string | null; file: File }> {
  state = { error: null as string | null, file: this.props.file };

  static getDerivedStateFromError(error: unknown) {
    return { error: error instanceof Error ? error.message : String(error) };
  }

  static getDerivedStateFromProps(props: { file: File }, state: { file: File }) {
    return props.file === state.file ? null : { error: null, file: props.file };  // a new file gets a new try
  }

  render() {
    return this.state.error ? (
      <p className="stage-message error-text">Cannot show the 3D preview: {this.state.error}. Reloading the page may help.</p>
    ) : (
      this.props.children
    );
  }
}

function Loading({ children }: { children: string }) {
  return (
    <div className="stage-message" role="status">
      <div className="spinner" />
      {children}
    </div>
  );
}

export default function FilePreview({ file }: FilePreviewProps) {
  const extension = extensionOf(file.name);
  const mesh = isMeshExtension(extension);
  const { preview, error, fail } = useImagePreview(file, !mesh); // MeshPreview reads a mesh itself

  // Rendered for every mesh, so switching between meshes keeps one 3D view (and one WebGL context).
  if (mesh) {
    return (
      <div className="stage">
        <PreviewBoundary file={file}>
          <Suspense fallback={<Loading>Loading 3D preview…</Loading>}>
            <MeshPreview file={file} type={extension} />
          </Suspense>
        </PreviewBoundary>
      </div>
    );
  }

  return (
    <div className={preview && !error ? 'stage checker' : 'stage'}>
      {error ? (
        <p className="stage-message error-text">{error}</p>
      ) : preview ? (
        <ImageView preview={preview} label={`Preview of ${file.name}`} onError={fail} />
      ) : (
        <Loading>Loading preview…</Loading>
      )}
    </div>
  );
}

/**
 * The original with the optimized image on top, split by a slider: the
 * original shows left of the divider, the optimized image right of it.
 */
export function ImageCompare({ original, optimized }: ImageCompareProps) {
  const before = useImagePreview(original);
  const after = useImagePreview(optimized);
  const stage = useRef<HTMLDivElement>(null);
  const [area, setArea] = useState({ width: 0, height: 0 });
  const [loaded, setLoaded] = useState<{ preview: Preview; width: number; height: number } | null>(null);
  const [split, setSplit] = useState(50);

  useEffect(() => {
    const observer = new ResizeObserver(([entry]) => setArea({ width: entry.contentRect.width, height: entry.contentRect.height }));
    observer.observe(stage.current!);
    return () => observer.disconnect();
  }, []);

  const error = before.error ?? after.error;
  // The original's natural size: decoded pixels have theirs; an <img> reports it once loaded.
  const size =
    before.preview?.kind === 'pixels'
      ? before.preview.image
      : loaded && loaded.preview === before.preview
        ? loaded
        : null;
  // Both images fill one box: the original's size, scaled down to fit (an SVG may have no size: then the whole area).
  const scale = size && size.width > 0 && size.height > 0 ? Math.min(1, area.width / size.width, area.height / size.height) : 0;
  const box = scale > 0 ? { width: size!.width * scale, height: size!.height * scale } : area;

  return (
    <div ref={stage} className={before.preview && after.preview && !error ? 'stage checker' : 'stage'}>
      {error ? (
        <p className="stage-message error-text">{error}</p>
      ) : before.preview && after.preview ? (
        <>
          <div
            className="compare"
            style={{ ...box, '--split': `${split}%`, visibility: size ? undefined : 'hidden' } as CSSProperties}
          >
            <ImageView
              preview={before.preview}
              label={`Preview of ${original.name}`}
              onError={before.fail}
              onSize={(width, height) => setLoaded({ preview: before.preview!, width, height })}
            />
            <ImageView preview={after.preview} label={`Preview of ${optimized.name}`} className="compare-after" onError={after.fail} />
            <input
              type="range"
              className="compare-range"
              min={0}
              max={100}
              step={0.5}
              value={split}
              onChange={(e) => setSplit(Number(e.target.value))}
              aria-label="Divider between original and optimized"
              aria-valuetext={`${Math.round(split)}% original`}
            />
            <div className="compare-divider" aria-hidden="true">
              <span className="compare-knob">
                <svg viewBox="0 0 20 20" width="16" height="16">
                  <path d="M7 5 2 10l5 5M13 5l5 5-5 5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </span>
            </div>
          </div>
          <span className="compare-tag start">Original</span>
          <span className="compare-tag end">Optimized</span>
        </>
      ) : (
        <Loading>Loading preview…</Loading>
      )}
    </div>
  );
}
