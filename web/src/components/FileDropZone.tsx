import { useState, useRef } from 'react';
import type { FileDropZoneProps } from '../types';
import { IMAGE_EXTENSIONS, MESH_EXTENSIONS, extensionOf, fileKind } from '../utils/files';
import { formatBytes } from '../utils/format';
import Icon from './Icon';

const ACCEPT = [...IMAGE_EXTENSIONS, ...MESH_EXTENSIONS].map((ext) => `.${ext}`).join(',');
const FORMATS = ['PNG', 'JPEG', 'WebP', 'GIF', 'BMP', 'TGA', 'TIFF', 'ICO', 'SVG', 'OBJ', 'STL', 'PLY', 'glTF', 'GLB', 'FBX'];

export default function FileDropZone({ selectedFiles, setSelectedFiles, activeFile, onFileSelect }: FileDropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);
  const empty = selectedFiles.length === 0;

  const handleFiles = (files: File[]) => {
    const skipped = files.filter((file) => fileKind(file.name) === null);
    setNotice(skipped.length > 0 ? `Not supported: ${skipped.map((file) => file.name).join(', ')}` : null);
    // Names identify files in the list and in downloads, so the first file with a name wins.
    const names = new Set(selectedFiles.map((file) => file.name));
    const added: File[] = [];
    for (const file of files) {
      if (fileKind(file.name) === null || names.has(file.name)) continue;
      names.add(file.name);
      added.push(file);
    }
    if (added.length > 0) {
      setSelectedFiles([...selectedFiles, ...added]);
      onFileSelect(added[0]);
    }
  };

  const zone = (
    <div
      role="button"
      tabIndex={0}
      aria-label="Drop files or activate to choose files"
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
          <span className="dropzone-sub">or click to browse</span>
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

  const message = notice && (
    <p className="notice" role="alert">
      {notice}
    </p>
  );

  if (empty) {
    return (
      <div>
        {zone}
        {picker}
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
              aria-current={file === activeFile}
              aria-label={`Preview ${file.name}`}
              onClick={() => onFileSelect(file)}
            >
              <span className="thumb">
                <Icon name={fileKind(file.name) === 'mesh' ? 'cube' : 'image'} />
              </span>
              <span className="row-text">
                <span className="row-name">{file.name}</span>
                <span className="row-meta">
                  {formatBytes(file.size)} · {extensionOf(file.name).toUpperCase()}
                </span>
              </span>
            </button>
            <button
              type="button"
              className="icon-button"
              aria-label={`Remove ${file.name}`}
              onClick={() => setSelectedFiles(selectedFiles.filter((f) => f !== file))}
            >
              <Icon name="close" size={14} />
            </button>
          </li>
        ))}
      </ul>
      {zone}
      {picker}
      {message}
    </section>
  );
}
