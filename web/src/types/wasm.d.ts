declare module '@pkg/pixie_juice_bg.js' {
  export function __wbg_set_wasm(wasm: unknown): void;
  export function optimize_image(data: Uint8Array, quality: number): Uint8Array;
  export function optimize_mesh(data: Uint8Array, target_ratio: number): Uint8Array;
  export function optimize_auto(data: Uint8Array, quality: number): Uint8Array;
  export function detect_format(data: Uint8Array): string;
  export function version(): string;
  export function optimize_png(data: Uint8Array, quality: number): Uint8Array;
  export function optimize_jpeg(data: Uint8Array, quality: number): Uint8Array;
  export function optimize_webp(data: Uint8Array, quality: number): Uint8Array;
  export function optimize_gif(data: Uint8Array, quality: number): Uint8Array;
  export function optimize_ico(data: Uint8Array, quality: number): Uint8Array;
  export function is_webp(data: Uint8Array): boolean;
  export function is_gif(data: Uint8Array): boolean;
  export function is_ico(data: Uint8Array): boolean;
  export function convert_to_webp(data: Uint8Array, quality: number): Uint8Array;
  export function convert_to_png(data: Uint8Array): Uint8Array;
  export function convert_to_jpeg(data: Uint8Array, quality: number): Uint8Array;
  export function convert_to_bmp(data: Uint8Array): Uint8Array;
  export function convert_to_gif(data: Uint8Array, quality: number): Uint8Array;
  export function convert_to_ico(data: Uint8Array, quality: number): Uint8Array;
  export function convert_to_tiff(data: Uint8Array, quality: number): Uint8Array;
  export function convert_to_svg(data: Uint8Array, quality: number): Uint8Array;
  export function get_performance_metrics(): Record<string, unknown>;
  export function reset_performance_stats(): void;
  export function check_performance_compliance(): boolean;
  export function set_lossless_mode(enabled: boolean): void;
  export function set_preserve_metadata(enabled: boolean): void;
  export function build_timestamp(): string;
  export function pixie_get_memory_target_mb(): number;
  export function optimize_obj(data: Uint8Array, target_ratio: number): Uint8Array;
  export function optimize_gltf(data: Uint8Array, target_ratio: number): Uint8Array;
  export function optimize_stl(data: Uint8Array, target_ratio: number): Uint8Array;
  export function optimize_fbx(data: Uint8Array, target_ratio: number): Uint8Array;
  export function optimize_ply(data: Uint8Array, target_ratio: number): Uint8Array;
  export function is_obj(data: Uint8Array): boolean;
  export function is_gltf(data: Uint8Array): boolean;
  export function is_stl(data: Uint8Array): boolean;
  export function is_fbx(data: Uint8Array): boolean;
  export function is_ply(data: Uint8Array): boolean;
  export function optimize_tga(data: Uint8Array, quality: number): Uint8Array;
  export function is_tga(data: Uint8Array): boolean;
  export function convert_to_tga(data: Uint8Array, quality: number): Uint8Array;
}

declare module '@pkg/pixie_juice_bg.wasm' {
  export const memory: WebAssembly.Memory;
  export function __wbindgen_start(): void;
}
