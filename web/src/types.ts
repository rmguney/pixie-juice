import type { ConvertTarget, FormatName, PixieApi } from './core/types';

export type OutputChoice = 'auto' | ConvertTarget;

/** What the next run uses; kept between runs. */
export interface Settings {
  quality: number;
  /** Share of mesh triangles kept, 10-100. */
  meshDetail: number;
  output: OutputChoice;
  preserveMetadata: boolean;
  lossless: boolean;
  /** Longest side images are scaled down to, in pixels; 0 keeps their size. */
  maxSize: number;
}

export const DEFAULT_SETTINGS: Settings = {
  quality: 75,
  meshDetail: 100,
  output: 'auto',
  preserveMetadata: false,
  lossless: false,
  maxSize: 0,
};

export type ProcessedResult =
  | {
      file: File;
      success: true;
      data: Uint8Array;
      format: FormatName;
      /** False when the original came back unchanged. */
      changed: boolean;
    }
  | { file: File; success: false; error: string };

export type ProcessedSuccess = Extract<ProcessedResult, { success: true }>;

export interface FileDropZoneProps {
  selectedFiles: File[];
  setSelectedFiles: React.Dispatch<React.SetStateAction<File[]>>;
  /** The file shown in the preview. */
  activeFile: File | null;
  onFileSelect: (file: File) => void;
}

export interface ProcessingPanelProps {
  files: File[];
  api: PixieApi;
  settings: Settings;
  onSettingsChange: (settings: Settings) => void;
  /** Called with the files snapshot the results were made for, and how many it processes (not .bin files). */
  onProcess: (files: File[], results: ProcessedResult[], total: number) => void;
}

export interface ResultsPanelProps {
  results: ProcessedResult[];
  /** Files the run processes: more than the results when it was cancelled. */
  total: number;
  onReset: () => void;
}

export interface FilePreviewProps {
  file: File;
  /** The file list, where a .gltf finds the files it refers to. */
  files: File[];
}

export interface ImageCompareProps {
  original: File;
  optimized: File;
}
