// TypeScript bindings for emscripten-generated code.  Automatically generated at compile time.
interface WasmModule {
}

type EmbindString = ArrayBuffer|Uint8Array|Uint8ClampedArray|Int8Array|string;
interface EmbindModule {
  optimize(_0: any, _1: any): any;
  convert(_0: any, _1: EmbindString, _2: any): any;
  detect(_0: any): string;
  decodeImage(_0: any): any;
  decodeMesh(_0: any): any;
  convertPixels(_0: any, _1: number, _2: number, _3: any, _4: EmbindString, _5: any): any;
  toSrgb(_0: any, _1: number, _2: number, _3: any): any;
  version(): string;
  buildTimestamp(): string;
  buildSource(): string;
}

export type MainModule = WasmModule & EmbindModule;
export default function MainModuleFactory (options?: unknown): Promise<MainModule>;
