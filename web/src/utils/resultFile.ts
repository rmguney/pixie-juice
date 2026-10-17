import type { ProcessedSuccess } from '../types';
import { outputName } from './files';
import { outputFormat } from './outputFormat';

const files = new WeakMap<ProcessedSuccess, File>();

/** A result as a file, made once (it copies the bytes) and kept with the result. */
export function resultFile(result: ProcessedSuccess): File {
  let file = files.get(result);
  if (!file) {
    const { extension, mimeType } = outputFormat(result.format);
    file = new File([result.data as BlobPart], outputName(result.file.name, extension), { type: mimeType });
    files.set(result, file);
  }
  return file;
}
