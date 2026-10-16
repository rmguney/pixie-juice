// Main-thread side of the engine: a promise API over a small pool of workers,
// so a batch's files are processed side by side.

import type { Operation, Request, Response } from './protocol';
import type { BuildInfo, ConvertTarget, CoreOptions, CoreResult, DecodedImage, DecodedMesh, PixieApi } from './types';

interface Pending {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
}

/** One engine worker. */
class WorkerClient {
  private readonly pending = new Map<number, Pending>();
  private nextId = 0;
  /** Set when the worker couldn't start (missing chunk, CSP, offline); later calls fail fast. */
  broken: Error | null = null;
  private worker: Worker;

  constructor(private readonly onSettled: () => void) {
    this.worker = this.spawn();
  }

  /** Requests sent and not yet answered. */
  get load(): number {
    return this.pending.size;
  }

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
    this.onSettled();
  }

  private rejectAll(error: Error) {
    for (const { reject } of this.pending.values()) reject(error);
    this.pending.clear();
  }

  call<T>(op: Operation, args: unknown[], transfer: Transferable[]): Promise<T> {
    if (this.broken) return Promise.reject(this.broken);
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (value: unknown) => void, reject });
      const request: Request = { id, op, args };
      this.worker.postMessage(request, transfer);
    });
  }

  terminate() {
    this.rejectAll(new Error('The engine worker was stopped'));
    this.worker.terminate();
  }
}

/** Workers for a batch: one per spare core, at most four, two on devices with little memory. */
function poolSize(): number {
  const cores = Math.min(Math.max((navigator.hardwareConcurrency || 2) - 1, 1), 4);
  const memory = (navigator as Navigator & { deviceMemory?: number }).deviceMemory;
  return memory !== undefined && memory <= 4 ? Math.min(cores, 2) : cores;
}

/**
 * Idle workers stop, since WebAssembly memory never shrinks: the extra ones
 * after a short pause, the last one (which restarts in a moment) after a long one.
 */
const IDLE_MS = 30_000;
const LAST_IDLE_MS = 5 * 60_000;

class EnginePool implements PixieApi {
  readonly concurrency = poolSize();
  private readonly workers: WorkerClient[] = [];
  private idleTimer: ReturnType<typeof setTimeout> | undefined;

  private create(): WorkerClient {
    return new WorkerClient(() => this.settled());
  }

  /** An idle worker, else a new one while the pool has room, else the least busy. */
  private pick(): WorkerClient {
    clearTimeout(this.idleTimer);
    const live = this.workers.filter((worker) => !worker.broken);
    const idle = live.find((worker) => worker.load === 0);
    if (idle) return idle;
    // After a worker failed to start, no more are started (they would fail the same way).
    if (this.workers.length < this.concurrency && live.length === this.workers.length) {
      const worker = this.create();
      this.workers.push(worker);
      return worker;
    }
    return (live.length > 0 ? live : this.workers).reduce((a, b) => (b.load < a.load ? b : a));
  }

  private settled() {
    if (this.workers.length === 0 || this.workers.some((worker) => worker.load > 0)) return;
    clearTimeout(this.idleTimer);
    const extra = this.workers.length > 1;
    this.idleTimer = setTimeout(
      () => {
        for (const worker of this.workers.splice(extra ? 1 : 0)) worker.terminate();
        if (extra) this.settled();
      },
      extra ? IDLE_MS : LAST_IDLE_MS,
    );
  }

  /** Sends `op`; `data` moves to the worker when `transfer` is set, rather than being copied. */
  private call<T>(op: Operation, data: Uint8Array | undefined, transfer: boolean, ...rest: unknown[]): Promise<T> {
    const args = data === undefined ? rest : [data, ...rest];
    return this.pick().call<T>(op, args, transfer && data ? [data.buffer as ArrayBuffer] : []);
  }

  info() {
    return this.call<BuildInfo>('info', undefined, false);
  }

  detect(data: Uint8Array) {
    return this.call<string>('detect', data, false);
  }

  optimize(data: Uint8Array, options: CoreOptions = {}, transfer = false) {
    return this.call<CoreResult>('optimize', data, transfer, options);
  }

  convert(data: Uint8Array, target: ConvertTarget, options: CoreOptions = {}, transfer = false) {
    return this.call<CoreResult>('convert', data, transfer, target, options);
  }

  decodeImage(data: Uint8Array, transfer = false) {
    return this.call<DecodedImage>('decodeImage', data, transfer);
  }

  decodeMesh(data: Uint8Array, transfer = false) {
    return this.call<DecodedMesh>('decodeMesh', data, transfer);
  }
}

let client: PixieApi | null = null;

/** The shared engine client; workers start when there is work for them. */
export function getPixie(): PixieApi {
  client ??= new EnginePool();
  return client;
}
