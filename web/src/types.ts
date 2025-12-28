export interface WasmModule {
  optimize_image?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_mesh?: (data: Uint8Array, ratio: number) => Uint8Array;
  optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_png?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_jpeg?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_webp?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_gif?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_ico?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_tga?: (data: Uint8Array, quality: number) => Uint8Array;
  optimize_obj?: (data: Uint8Array, ratio: number) => Uint8Array;
  optimize_stl?: (data: Uint8Array, ratio: number) => Uint8Array;
  optimize_fbx?: (data: Uint8Array, ratio: number) => Uint8Array;
  optimize_gltf?: (data: Uint8Array, ratio: number) => Uint8Array;
  optimize_ply?: (data: Uint8Array, ratio: number) => Uint8Array;
  detect_format?: (data: Uint8Array) => string;
  is_webp?: (data: Uint8Array) => boolean;
  is_gif?: (data: Uint8Array) => boolean;
  is_ico?: (data: Uint8Array) => boolean;
  is_tga?: (data: Uint8Array) => boolean;
  is_obj?: (data: Uint8Array) => boolean;
  is_gltf?: (data: Uint8Array) => boolean;
  is_stl?: (data: Uint8Array) => boolean;
  is_fbx?: (data: Uint8Array) => boolean;
  is_ply?: (data: Uint8Array) => boolean;
  convert_to_webp?: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_png?: (data: Uint8Array) => Uint8Array;
  convert_to_jpeg?: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_bmp?: (data: Uint8Array) => Uint8Array;
  convert_to_gif?: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_ico?: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_tiff?: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_svg?: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_tga?: (data: Uint8Array, quality: number) => Uint8Array;
  strip_tiff_metadata_simd?: (data: Uint8Array, preserveIcc: boolean) => Uint8Array;
  get_performance_metrics?: () => PerformanceMetrics;
  reset_performance_stats?: () => void;
  check_performance_compliance?: () => boolean;
  set_lossless_mode?: (enabled: boolean) => void;
  set_preserve_metadata?: (enabled: boolean) => void;
  version?: () => string;
  build_timestamp?: () => string;
  pixie_get_memory_target_mb?: () => number;
}

export interface WasmHook extends WasmModule {
  loading: boolean;
  error: string | null;
  available: boolean;
}

export interface PerformanceMetrics {
  last_operation_time_ms?: number;
  memory_peak_mb?: number;
}

export interface ProcessedResult {
  originalFile: File;
  originalData?: Uint8Array;
  optimizedData?: Uint8Array;
  targetFormat?: string;
  originalSize?: number;
  optimizedSize?: number;
  savings?: number;
  success: boolean;
  error?: string;
  fileType: 'image' | 'mesh' | 'unknown';
  performanceMetrics?: PerformanceMetrics | null;
  diagnostics?: {
    fileName: string;
    fileSize: number;
    fileType: string;
    quality: number;
    outputFormat: string;
    errorMessage: string;
    isCompressLz4Error: boolean;
  };
}

export interface FileDropZoneProps {
  selectedFiles: File[];
  setSelectedFiles: React.Dispatch<React.SetStateAction<File[]>>;
  onFileSelect: (file: File) => void;
}

export interface ProcessingPanelProps {
  files: File[];
  wasm: WasmHook;
  onProcess: (results: ProcessedResult[]) => void;
  setIsProcessing: (processing: boolean) => void;
}

export interface ResultsPanelProps {
  results: ProcessedResult[];
  onReset: () => void;
}

export interface FilePreviewProps {
  file: File | null;
}
