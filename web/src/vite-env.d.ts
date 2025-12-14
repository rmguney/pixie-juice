/// <reference types="vite/client" />

declare module '*.wasm' {
  const content: WebAssembly.Module;
  export default content;
  export const memory: WebAssembly.Memory;
  export function __wbindgen_start(): void;
}
