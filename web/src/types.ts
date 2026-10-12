import type { FormatName, PixieApi } from './core/types';

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
