import { useState, useRef, useEffect } from 'react';
import type { FileDropZoneProps } from '../types';
import { IMAGE_EXTENSIONS, MESH_EXTENSIONS, RESOURCE_EXTENSIONS, extensionOf, fileKind } from '../utils/files';
import { formatBytes } from '../utils/format';
import Icon from './Icon';

const ACCEPT = [...IMAGE_EXTENSIONS, ...MESH_EXTENSIONS, ...RESOURCE_EXTENSIONS].map((ext) => `.${ext}`).join(',');
const ICONS = { image: 'image', mesh: 'cube', resource: 'file' } as const;
const FORMATS = ['PNG', 'JPEG', 'WebP', 'AVIF', 'HEIC', 'GIF', 'BMP', 'TGA', 'TIFF', 'ICO', 'SVG', 'OBJ', 'STL', 'PLY', 'glTF', 'GLB', 'FBX'];

/** `name`, or "name-2.ext", "name-3.ext", ... when it is taken. */
function freeName(name: string, taken: Set<string>): string {
  if (!taken.has(name)) return name;
  const dot = name.lastIndexOf('.');
  const [base, extension] = dot > 0 ? [name.slice(0, dot), name.slice(dot)] : [name, ''];
  for (let n = 2; ; n++) {
    const candidate = `${base}-${n}${extension}`;
    if (!taken.has(candidate)) return candidate;
  }
}

/** Where pasting means text: left to the browser. */
function isTextField(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable || target instanceof HTMLTextAreaElement) return true;
  return target instanceof HTMLInputElement && !['file', 'range', 'checkbox', 'radio', 'button', 'submit'].includes(target.type);
}

export default function FileDropZone({ selectedFiles, setSelectedFiles, activeFile, onFileSelect }: FileDropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  // Shown while the list is the one the skipped files were dropped onto.
  const [notice, setNotice] = useState<{ text: string; list: File[] } | null>(null);
  const input = useRef<HTMLInputElement>(null);
  const zoneRef = useRef<HTMLDivElement>(null);
  const rows = useRef(new Map<string, HTMLButtonElement>());
  const empty = selectedFiles.length === 0;

  // Emptying the list (Clear all, the last Remove, Start over) swaps the layout:
  // focus goes back to the drop zone instead of being lost.
  const wasEmpty = useRef(empty);
  useEffect(() => {
    if (empty && !wasEmpty.current && (!document.activeElement || document.activeElement === document.body)) {
      zoneRef.current?.focus();
    }
    wasEmpty.current = empty;
  }, [empty]);

  const handleFiles = (files: File[]) => {
    const skipped = files.filter((file) => fileKind(file.name) === null);
    // Names identify files in the list and in downloads: a file with a name already
    // listed replaces that entry (a newer version), keeping its place.
    const byName = new Map(selectedFiles.map((file) => [file.name, file]));
    const added = files.filter((file) => fileKind(file.name) !== null);
    for (const file of added) byName.set(file.name, file);
    const list = added.length > 0 ? [...byName.values()] : selectedFiles;
    setNotice(skipped.length > 0 ? { text: `Not supported: ${skipped.map((file) => file.name).join(', ')}`, list } : null);
    if (added.length > 0) {
      setSelectedFiles(list);
      onFileSelect(byName.get(added[0].name)!);
    }
  };

  // Pasting adds files too: an image copied from another app, or files copied in a file manager.
  const handlePaste = useRef<(files: File[]) => void>(() => {});
  useEffect(() => {
    handlePaste.current = (files) => {
      // The browser names copied image data itself ("image.png" every time), so a pasted
      // file whose name is listed gets a free one instead of replacing that file.
      const taken = new Set(selectedFiles.map((file) => file.name));
      handleFiles(
        files.map((file) => {
          const name = freeName(file.name, taken);
          taken.add(name);
          return name === file.name ? file : new File([file], name, { type: file.type, lastModified: file.lastModified });
        }),
      );
    };
  });
  useEffect(() => {
    const paste = (event: ClipboardEvent) => {
      const files = Array.from(event.clipboardData?.files ?? []);
      if (files.length === 0 || isTextField(event.target)) return;
      event.preventDefault();
      handlePaste.current(files);
    };
    window.addEventListener('paste', paste);
    return () => window.removeEventListener('paste', paste);
  }, []);

  const remove = (file: File) => {
    // Focus moves to the next file (or the previous one) rather than being lost.
    const index = selectedFiles.indexOf(file);
    const neighbour = selectedFiles[index + 1] ?? selectedFiles[index - 1];
    if (neighbour) rows.current.get(neighbour.name)?.focus();
    setSelectedFiles(selectedFiles.filter((f) => f !== file));
  };

  // The hidden input comes first, so the drop zone and the notice are adjacent siblings (see .compact + .notice).
  const picker = (
    <input
      ref={input}
      id="file-input"
      type="file"
      multiple
      accept={ACCEPT}
      hidden
      onChange={(e) => {
        handleFiles(Array.from(e.target.files ?? []));
        e.target.value = ''; // so the same file can be chosen again after removing it
      }}
    />
  );

  const zone = (
    <div
      ref={zoneRef}
      role="button"
      tabIndex={0}
      aria-label={empty ? 'Drop files here, click to browse, or paste' : 'Add files'}
      className={empty ? 'dropzone' : 'dropzone compact'}
      data-dragging={isDragging || undefined}
      onClick={() => input.current?.click()}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          input.current?.click();
        }
      }}
      onDragOver={(e) => {
        e.preventDefault();
        setIsDragging(true);
      }}
      onDragLeave={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setIsDragging(false);
      }}
      onDrop={(e) => {
        e.preventDefault();
        setIsDragging(false);
        handleFiles(Array.from(e.dataTransfer.files));
      }}
    >
      {empty ? (
        <>
          <span className="dropzone-icon">
            <Icon name="upload" size={20} />
          </span>
          <span className="dropzone-title">Drop files here</span>
          <span className="dropzone-sub">click to browse, or paste</span>
          <span className="dropzone-formats">
            {FORMATS.map((format) => (
              <span key={format}>{format}</span>
            ))}
          </span>
        </>
      ) : (
        <>
          <Icon name="plus" />
          Add files
        </>
      )}
    </div>
  );

  const message = notice && notice.list === selectedFiles && (
    <p className="notice" role="alert">
      {notice.text}
    </p>
  );

  if (empty) {
    return (
      <div>
        {picker}
        {zone}
        {message}
      </div>
    );
  }

  return (
    <section className="card files" aria-labelledby="files-title">
      <div className="card-head">
        <h2 className="card-title" id="files-title">
          Files<span className="count">{selectedFiles.length}</span>
        </h2>
        <button type="button" className="text-button" onClick={() => setSelectedFiles([])}>
          Clear all
        </button>
      </div>
      <ul className="list" aria-label="Selected files">
        {selectedFiles.map((file) => (
          <li key={file.name} className="row">
            <button
              type="button"
              className="row-main"
              ref={(button) => {
                if (button) rows.current.set(file.name, button);
                else rows.current.delete(file.name);
              }}
              aria-current={file === activeFile}
              aria-label={`Preview ${file.name}`}
              onClick={() => onFileSelect(file)}
            >
              <span className="thumb">
                <Icon name={ICONS[fileKind(file.name) ?? 'image']} />
              </span>
              <span className="row-text">
                <span className="row-name">{file.name}</span>
                <span className="row-meta">
                  {formatBytes(file.size)} · {extensionOf(file.name).toUpperCase()}
                  {fileKind(file.name) === 'resource' && ' · for a .gltf'}
                </span>
              </span>
            </button>
            <button type="button" className="icon-button" aria-label={`Remove ${file.name}`} onClick={() => remove(file)}>
              <Icon name="close" size={14} />
            </button>
          </li>
        ))}
      </ul>
      {picker}
      {zone}
      {message}
    </section>
  );
}
