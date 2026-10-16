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
}

export const DEFAULT_SETTINGS: Settings = {
  quality: 75,
  meshDetail: 100,
  output: 'auto',
  preserveMetadata: false,
  lossless: false,
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
  /** Called with the files snapshot the results were made for. */
  onProcess: (files: File[], results: ProcessedResult[]) => void;
}

export interface ResultsPanelProps {
  results: ProcessedResult[];
  onReset: () => void;
}

export interface FilePreviewProps {
  file: File;
}

export interface ImageCompareProps {
  original: File;
  optimized: File;
}
