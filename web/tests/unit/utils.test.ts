import { describe, it, expect } from 'vitest';
import { outputFormat } from '../../src/utils/outputFormat';
import { extensionOf, fileKind, outputName, outputNames } from '../../src/utils/files';

describe('outputFormat', () => {
  it('maps engine formats to extensions and MIME types', () => {
    expect(outputFormat('jpeg')).toEqual({ extension: 'jpg', mimeType: 'image/jpeg' });
    expect(outputFormat('svg').mimeType).toBe('image/svg+xml');
    expect(outputFormat('glb')).toEqual({ extension: 'glb', mimeType: 'model/gltf-binary' });
  });
});

describe('files', () => {
  it('classifies by extension, case-insensitively', () => {
    expect(fileKind('Photo.JPG')).toBe('image');
    expect(fileKind('scan.tif')).toBe('image');
    expect(fileKind('model.glb')).toBe('mesh');
    expect(fileKind('notes.pdf')).toBeNull();
    expect(fileKind('no-extension')).toBeNull();
  });

  it('builds output names from the real output format', () => {
    expect(outputName('holiday photo.png', 'webp')).toBe('holiday photo_juice.webp');
    expect(outputName('archive.tar.gz', 'png')).toBe('archive.tar_juice.png');
    expect(outputName('README', 'txt')).toBe('README_juice.txt');
    expect(extensionOf('a.b.C')).toBe('c');
  });

  it('keeps output names unique within a batch', () => {
    expect(
      outputNames([
        { name: 'photo.png', extension: 'png' },
        { name: 'photo.bmp', extension: 'png' },
        { name: 'icon.tga', extension: 'png' },
      ]),
    ).toEqual(['photo_png_juice.png', 'photo_bmp_juice.png', 'icon_juice.png']);
  });
});
