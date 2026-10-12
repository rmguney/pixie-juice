// Main-thread side of the engine: a promise API over the worker.

import type { Operation, Request, Response } from './protocol';
import type { BuildInfo, ConvertTarget, CoreOptions, CoreResult, DecodedImage, DecodedMesh, PixieApi } from './types';

interface Pending {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
}

class WorkerClient implements PixieApi {
  private readonly pending = new Map<number, Pending>();
  private nextId = 0;
  /** Set when the worker couldn't start (missing chunk, CSP, offline); later calls fail fast. */
  private broken: Error | null = null;
  private worker = this.spawn();

  private spawn(): Worker {
    const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (event: MessageEvent<Response>) => this.settle(event.data);
    worker.onerror = (event) => {
      // A worker that fails to load would fail again on every restart, so it isn't restarted.
      this.broken = new Error(event.message || 'The engine worker failed to start');
      this.rejectAll(this.broken);
      this.worker.terminate();
    };
    return worker;
  }

  private settle(response: Response) {
    const pending = this.pending.get(response.id);
    if (!pending) return;
    this.pending.delete(response.id);
    if ('error' in response) {
      pending.reject(new Error(response.error));
      // A WebAssembly trap leaves the module unusable: requests queued behind it
      // fail too, and the next call gets a fresh engine.
      if (response.fatal) {
        this.rejectAll(new Error('The engine stopped after an error; please try again'));
        this.worker.terminate();
        this.worker = this.spawn();
      }
    } else {
      pending.resolve(response.result);
    }
  }

  private rejectAll(error: Error) {
    for (const { reject } of this.pending.values()) reject(error);
    this.pending.clear();
  }

  private call<T>(op: Operation, ...args: unknown[]): Promise<T> {
    if (this.broken) return Promise.reject(this.broken);
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (value: unknown) => void, reject });
      const request: Request = { id, op, args };
      this.worker.postMessage(request);
    });
  }

  info() {
    return this.call<BuildInfo>('info');
  }

  detect(data: Uint8Array) {
    return this.call<string>('detect', data);
  }

  optimize(data: Uint8Array, options: CoreOptions = {}) {
    return this.call<CoreResult>('optimize', data, options);
  }

  convert(data: Uint8Array, target: ConvertTarget, options: CoreOptions = {}) {
    return this.call<CoreResult>('convert', data, target, options);
  }

  decodeImage(data: Uint8Array) {
    return this.call<DecodedImage>('decodeImage', data);
  }

  decodeMesh(data: Uint8Array) {
    return this.call<DecodedMesh>('decodeMesh', data);
  }
}

let client: PixieApi | null = null;

/** The shared engine client; the worker starts on first use. */
export function getPixie(): PixieApi {
  client ??= new WorkerClient();
  return client;
}
