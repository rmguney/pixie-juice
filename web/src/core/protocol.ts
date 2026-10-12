import type { Engine } from './engine';

export type Operation = 'info' | 'detect' | 'optimize' | 'convert' | 'decodeImage' | 'decodeMesh';

export interface Request {
  id: number;
  op: Operation & keyof Engine;
  args: unknown[];
}

export type Response =
  | { id: number; result: unknown }
  | { id: number; error: string; fatal: boolean };
