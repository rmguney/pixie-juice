// The engine proper: the Emscripten module (C++ core) plus SVGO for SVG.
// Runs inside the worker; see worker.ts.

import createPixieCore, { type MainModule } from './generated/pixie_core.js';
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

interface NativeMesh {
  positions?: Float32Array;
  normals?: Float32Array;
  colors?: Float32Array;
  indices?: Uint32Array;
  error?: string;
}

function unwrap(result: NativeResult): CoreResult {
  if (result.error !== undefined || !result.data || !result.format) {
    throw new Error(result.error ?? 'Processing failed');
  }
  return { data: result.data, format: result.format, changed: Boolean(result.changed) };
}

/** Text of an SVG in the encoding its XML declaration names (UTF-8 otherwise). */
function decodeSvg(data: Uint8Array): string {
  const declaration = new TextDecoder('latin1').decode(data.subarray(0, 200));
  const encoding = /^\s*<\?xml[^>]*encoding=["']([\w.:-]+)["']/i.exec(declaration)?.[1];
  try {
    return new TextDecoder(encoding ?? 'utf-8').decode(data);
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
  const { data: svg } = optimize(decodeSvg(data), { multipass: true, plugins });
  const encoded = new TextEncoder().encode(svg);
  return encoded.length < data.length
    ? { data: encoded, format: 'svg', changed: true }
    : { data, format: 'svg', changed: false };
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
      commit: this.native.buildCommit(),
      timestamp: this.native.buildTimestamp(),
    };
  }

  detect(data: Uint8Array): string {
    return this.native.detect(data);
  }

  async optimize(data: Uint8Array, options: CoreOptions = {}): Promise<CoreResult> {
    if (data.length === 0) throw new Error('Input is empty');
    const result = unwrap(this.native.optimize(data, options) as NativeResult);
    // The core recognises SVG and leaves it to SVGO.
    return result.format === 'svg' ? optimizeSvgFile(data, options) : result;
  }

  convert(data: Uint8Array, target: ConvertTarget, options: CoreOptions = {}): CoreResult {
    if (data.length === 0) throw new Error('Input is empty');
    return unwrap(this.native.convert(data, target, options) as NativeResult);
  }

  decodeImage(data: Uint8Array): DecodedImage {
    const result = this.native.decodeImage(data) as NativeImage;
    if (result.error !== undefined || !result.rgba || !result.width || !result.height) {
      throw new Error(result.error ?? 'Decode failed');
    }
    return {
      width: result.width,
      height: result.height,
      rgba: new Uint8ClampedArray(result.rgba.buffer, result.rgba.byteOffset, result.rgba.byteLength),
    };
  }

  decodeMesh(data: Uint8Array): DecodedMesh {
    const { positions, normals, colors, indices, error } = this.native.decodeMesh(data) as NativeMesh;
    if (error !== undefined || !positions || !indices) throw new Error(error ?? 'Decode failed');
    return { positions, normals, colors, indices };
  }
}
