// Web Worker hosting the engine, so processing never blocks the page.

import { Engine } from './engine';
import type { Request, Response } from './protocol';

const engine = Engine.create();

/** The buffers behind a result's typed arrays, moved to the page instead of copied. */
function transferables(result: unknown): Transferable[] {
  if (!result || typeof result !== 'object') return [];
  return Object.values(result)
    .filter(ArrayBuffer.isView)
    .map((view) => view.buffer as ArrayBuffer);
}

self.onmessage = async (event: MessageEvent<Request>) => {
  const { id, op, args } = event.data;
  let response: Response;
  try {
    const instance = await engine;
    const result: unknown = await (instance[op] as (...a: unknown[]) => unknown).apply(instance, args);
    response = { id, result };
  } catch (error) {
    response = {
      id,
      error: error instanceof Error ? error.message : String(error),
      // A WebAssembly trap (e.g. out of memory) leaves the module unusable.
      fatal: error instanceof WebAssembly.RuntimeError,
    };
  }
  self.postMessage(response, { transfer: 'result' in response ? transferables(response.result) : [] });
};
