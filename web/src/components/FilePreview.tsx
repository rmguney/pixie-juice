import { lazy, Suspense, useState, useEffect, useRef } from 'react';
import { getPixie } from '../core/client';
import type { FilePreviewProps } from '../types';
import { extensionOf, isMeshExtension } from '../utils/files';

// three.js is only needed once a mesh is previewed.
const MeshPreview = lazy(() => import('./MeshPreview'));

/** Formats a browser <img> displays; TGA and TIFF go through the engine's decoder. */
const BROWSER_IMAGES = new Set(['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'ico', 'svg']);

type Preview = { kind: 'url'; url: string } | { kind: 'pixels'; image: ImageData };

function DecodedImage({ image, label }: { image: ImageData; label: string }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    canvas.current?.getContext('2d')?.putImageData(image, 0, 0);
  }, [image]);
  return <canvas ref={canvas} width={image.width} height={image.height} className="decoded" role="img" aria-label={label} />;
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
  const [preview, setPreview] = useState<Preview | null>(null);
  const [error, setError] = useState<string | null>(null);

  const extension = extensionOf(file.name);

  useEffect(() => {
    if (isMeshExtension(extension)) return; // MeshPreview reads the file itself
    setPreview(null);
    setError(null);

    let cancelled = false;
    let url: string | null = null;
    (async () => {
      try {
        if (BROWSER_IMAGES.has(extension)) {
          url = URL.createObjectURL(file);
          if (!cancelled) setPreview({ kind: 'url', url });
        } else {
          const decoded = await getPixie().decodeImage(new Uint8Array(await file.arrayBuffer()));
          if (!cancelled) {
            setPreview({ kind: 'pixels', image: new ImageData(decoded.rgba, decoded.width, decoded.height) });
          }
        }
      } catch (err) {
        if (!cancelled) setError(`Cannot preview ${file.name}: ${err instanceof Error ? err.message : String(err)}`);
      }
    })();

    return () => {
      cancelled = true;
      if (url) URL.revokeObjectURL(url);
    };
  }, [file, extension]);

  // Rendered for every mesh, so switching between meshes keeps one 3D view (and one WebGL context).
  if (isMeshExtension(extension)) {
    return (
      <div className="stage">
        <Suspense fallback={<Loading>Loading 3D preview…</Loading>}>
          <MeshPreview file={file} type={extension} />
        </Suspense>
      </div>
    );
  }

  if (error) {
    return (
      <div className="stage">
        <p className="stage-message error-text">{error}</p>
      </div>
    );
  }

  if (!preview) {
    return (
      <div className="stage">
        <Loading>Loading preview…</Loading>
      </div>
    );
  }

  const label = `Preview of ${file.name}`;
  return (
    <div className="stage checker">
      {preview.kind === 'url' ? (
        <img
          src={preview.url}
          alt={label}
          decoding="async"
          onError={() => setError(`Cannot preview ${file.name}: the browser could not decode it`)}
        />
      ) : (
        <DecodedImage image={preview.image} label={label} />
      )}
    </div>
  );
}
