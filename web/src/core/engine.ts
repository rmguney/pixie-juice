// The engine proper: the Emscripten module (C++ core) plus SVGO for SVG, the
// HEIC module, and the browser's own AVIF decoder. Runs inside the worker; see worker.ts.

import createPixieCore, { type MainModule } from './generated/pixie_core.js';
import type { MainModule as HeifModule } from './generated/pixie_heif.js';
import { externalUris, gltfJson, uriFileName } from '../utils/companions';
import type { BuildInfo, ConvertTarget, CoreOptions, CoreResult, DecodedImage, DecodedMesh, FormatName } from './types';

/** Shape of the objects the C++ bindings return (typed `any` by Embind). */
interface NativeResult {
  data?: Uint8Array;
  format?: FormatName;
  changed?: boolean;
  error?: string;
}

interface NativeImage {
  width?: number;
  height?: number;
  rgba?: Uint8Array;
  error?: string;
}

/** Pixels decoded outside the core, in the colour space `icc` describes (sRGB when empty). */
interface ForeignImage {
  width: number;
  height: number;
  rgba: Uint8Array;
  icc: Uint8Array;
}

interface NativeMesh {
  positions?: Float32Array;
  normals?: Float32Array;
  colors?: Float32Array;
  indices?: Uint32Array;
  lines?: Uint32Array;
  error?: string;
}

/**
 * A failure the engine reports about the input. Anything else thrown while
 * running it (a WebAssembly trap, a stack overflow through WASM frames) may
 * leave the module broken, and the worker restarts it.
 */
export class EngineError extends Error {}

function unwrap(result: NativeResult): CoreResult {
  if (result.error !== undefined || !result.data || !result.format) {
    throw new EngineError(result.error ?? 'Processing failed');
  }
  return { data: result.data, format: result.format, changed: Boolean(result.changed) };
}

/** Text of an SVG: by its byte order mark, else the encoding its XML declaration names, else UTF-8. */
function decodeSvg(data: Uint8Array): string {
  if (data[0] === 0xff && data[1] === 0xfe) return new TextDecoder('utf-16le').decode(data);
  if (data[0] === 0xfe && data[1] === 0xff) return new TextDecoder('utf-16be').decode(data);
  const declaration = new TextDecoder('latin1').decode(data.subarray(0, 200));
  const declared = /^\s*<\?xml[^>]*encoding=["']([\w.:-]+)["']/i.exec(declaration)?.[1];
  // A declaration readable as ASCII can't be UTF-16 or UTF-32 text (that would be full of zero bytes).
  const encoding = declared && !/^utf-?(16|32)/i.test(declared) ? declared : 'utf-8';
  try {
    return new TextDecoder(encoding).decode(data);
  } catch {
    return new TextDecoder('utf-8').decode(data); // an encoding the browser doesn't know
  }
}

/** SVGO plugins that change neither geometry nor rendering. */
const LOSSLESS_SVG_PLUGINS = ['removeDoctype', 'removeXMLProcInst', 'removeComments', 'removeMetadata',
  'removeEditorsNSData', 'cleanupAttrs', 'removeEmptyAttrs'] as const;

/**
 * SVGO in the worker, loaded on first use: its default preset, or only the
 * lossless plugins in lossless mode. The output is UTF-8 (the XML declaration
 * is dropped) and is used only when smaller.
 */
async function optimizeSvgFile(data: Uint8Array, options: CoreOptions): Promise<CoreResult> {
  const { optimize } = await import('svgo/browser');
  const keep: string[] = options.preserveMetadata ? ['removeComments', 'removeMetadata'] : [];
  const plugins = options.lossless
    ? LOSSLESS_SVG_PLUGINS.filter((name) => !keep.includes(name))
    : [{ name: 'preset-default' as const, params: { overrides: Object.fromEntries(keep.map((name) => [name, false])) } }];
  let svg: string;
  try {
    svg = optimize(decodeSvg(data), { multipass: true, plugins }).data;
  } catch (error) {
    throw new EngineError(`SVG could not be parsed: ${error instanceof Error ? error.message : String(error)}`);
  }
  const encoded = new TextEncoder().encode(svg);
  return encoded.length < data.length
    ? { data: encoded, format: 'svg', changed: true }
    : { data, format: 'svg', changed: false };
}

/** The HEIC module (libheif and libde265), loaded the first time a HEIC file needs it. */
let heifModule: Promise<HeifModule> | null = null;

async function decodeHeic(data: Uint8Array): Promise<ForeignImage> {
  heifModule ??= import('./generated/pixie_heif.js').then(({ default: create }) =>
    create({ print: () => {}, printErr: () => {} }),
  );
  let module: HeifModule;
  try {
    module = await heifModule;
  } catch (error) {
    heifModule = null; // a failed download is tried again next time
    throw error;
  }
  const { width, height, rgba, icc, error } = module.decodeHeif(data) as Partial<ForeignImage> & { error?: string };
  if (error !== undefined || !width || !height || !rgba || !icc) throw new EngineError(error ?? 'HEIC decode failed');
  return { width, height, rgba, icc };
}

/** AVIF through the browser's own decoder, which every supported browser has; the pixels come out in sRGB. */
async function decodeAvif(data: Uint8Array): Promise<ForeignImage> {
  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(new Blob([data as BlobPart], { type: 'image/avif' }), { premultiplyAlpha: 'none' });
  } catch {
    throw new EngineError('This browser could not decode the AVIF');
  }
  try {
    const { width, height } = bitmap;
    const context = typeof OffscreenCanvas === 'undefined' ? null : new OffscreenCanvas(width, height).getContext('2d');
    if (!context) throw new EngineError('This browser cannot read decoded images in a worker');
    context.drawImage(bitmap, 0, 0);
    const pixels = context.getImageData(0, 0, width, height).data;
    return { width, height, rgba: new Uint8Array(pixels.buffer, pixels.byteOffset, pixels.byteLength), icc: new Uint8Array() };
  } finally {
    bitmap.close();
  }
}

/** The resources a glTF names, as bytes: Blobs are read here, only when named. */
async function namedResources(data: Uint8Array, resources: NonNullable<CoreOptions['resources']>) {
  let names: Set<string>;
  try {
    names = new Set(externalUris(gltfJson(data).json).map(uriFileName));
  } catch {
    return []; // not a glTF: nothing to pack
  }
  return Promise.all(
    resources
      .filter((resource) => names.has(resource.name))
      .map(async ({ name, data: content }) => ({
        name,
        data: content instanceof Blob ? new Uint8Array(await content.arrayBuffer()) : content,
      })),
  );
}

/** Area-weighted vertex normals, as three.js's computeVertexNormals. */
function smoothNormals(positions: Float32Array, indices: Uint32Array): Float32Array {
  const normals = new Float32Array(positions.length);
  for (let t = 0; t < indices.length; t += 3) {
    const a = indices[t] * 3, b = indices[t + 1] * 3, c = indices[t + 2] * 3;
    const ux = positions[b] - positions[a], uy = positions[b + 1] - positions[a + 1], uz = positions[b + 2] - positions[a + 2];
    const vx = positions[c] - positions[a], vy = positions[c + 1] - positions[a + 1], vz = positions[c + 2] - positions[a + 2];
    const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
    normals[a] += nx;
    normals[a + 1] += ny;
    normals[a + 2] += nz;
    normals[b] += nx;
    normals[b + 1] += ny;
    normals[b + 2] += nz;
    normals[c] += nx;
    normals[c + 1] += ny;
    normals[c + 2] += nz;
  }
  for (let i = 0; i < normals.length; i += 3) {
    const length = Math.hypot(normals[i], normals[i + 1], normals[i + 2]) || 1;
    normals[i] /= length;
    normals[i + 1] /= length;
    normals[i + 2] /= length;
  }
  return normals;
}

/** sRGB colours to linear light, which three.js shades in. */
function linearize(colors: Float32Array) {
  for (let i = 0; i < colors.length; i++) {
    const c = colors[i];
    colors[i] = c < 0.04045 ? c * 0.0773993808 : Math.pow(c * 0.9478672986 + 0.0521327014, 2.4);
  }
}

export class Engine {
  private constructor(private readonly native: MainModule) {}

  static async create(): Promise<Engine> {
    // Diagnostics from the C/C++ libraries would otherwise go to the console.
    return new Engine(await createPixieCore({ print: () => {}, printErr: () => {} }));
  }

  info(): BuildInfo {
    return {
      version: this.native.version(),
      timestamp: this.native.buildTimestamp(),
      source: this.native.buildSource(),
    };
  }

  detect(data: Uint8Array): string {
    return this.native.detect(data);
  }

  /** HEIC or AVIF, which the core recognises but leaves to other decoders; null for other files. */
  private foreignFormat(data: Uint8Array): 'heic' | 'avif' | null {
    // Their ftyp box is at the start: no need to copy the whole file into the module.
    const kind = this.native.detect(data.subarray(0, 512));
    return kind === 'image:heic' ? 'heic' : kind === 'image:avif' ? 'avif' : null;
  }

  private decodeForeign(data: Uint8Array): Promise<ForeignImage> | null {
    const format = this.foreignFormat(data);
    return format === 'heic' ? decodeHeic(data) : format === 'avif' ? decodeAvif(data) : null;
  }

  async optimize(data: Uint8Array, options: CoreOptions = {}): Promise<CoreResult> {
    if (data.length === 0) throw new EngineError('Input is empty');
    // There's no HEIC or AVIF encoder: kept as they are, without copying them into the module.
    const foreign = this.foreignFormat(data);
    if (foreign) return { data, format: foreign, changed: false };
    const settings = options.resources ? { ...options, resources: await namedResources(data, options.resources) } : options;
    const result = unwrap(this.native.optimize(data, settings) as NativeResult);
    // The core recognises SVG and leaves it to SVGO.
    return result.format === 'svg' ? optimizeSvgFile(data, options) : result;
  }

  async convert(data: Uint8Array, target: ConvertTarget, options: CoreOptions = {}): Promise<CoreResult> {
    if (data.length === 0) throw new EngineError('Input is empty');
    const foreign = await this.decodeForeign(data);
    if (foreign) {
      const { width, height, rgba, icc } = foreign;
      return unwrap(this.native.convertPixels(rgba, width, height, icc, target, options) as NativeResult);
    }
    return unwrap(this.native.convert(data, target, options) as NativeResult);
  }

  async decodeImage(data: Uint8Array): Promise<DecodedImage> {
    const foreign = await this.decodeForeign(data);
    if (foreign) {
      const { width, height, icc } = foreign;
      const rgba = icc.length > 0 ? (this.native.toSrgb(foreign.rgba, width, height, icc) as Uint8Array) : foreign.rgba;
      return { width, height, rgba: new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.byteLength) };
    }
    const result = this.native.decodeImage(data) as NativeImage;
    if (result.error !== undefined || !result.rgba || !result.width || !result.height) {
      throw new EngineError(result.error ?? 'Decode failed');
    }
    return {
      width: result.width,
      height: result.height,
      rgba: new Uint8ClampedArray(result.rgba.buffer, result.rgba.byteOffset, result.rgba.byteLength),
    };
  }

  /** Done here rather than on the page: normals and colours of a large mesh take a while. */
  decodeMesh(data: Uint8Array): DecodedMesh {
    const { positions, normals, colors, indices, lines, error } = this.native.decodeMesh(data) as NativeMesh;
    if (error !== undefined || !positions || !indices) throw new EngineError(error ?? 'Decode failed');
    if (colors) linearize(colors);
    // Points and lines are drawn unlit: only triangles need normals.
    const shaded = normals ?? (indices.length > 0 ? smoothNormals(positions, indices) : undefined);
    return { positions, normals: shaded, colors, indices, lines };
  }
}
