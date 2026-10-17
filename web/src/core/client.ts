// Main-thread side of the engine: a promise API over a small pool of workers,
// so a batch's files are processed side by side.

import type { Operation, Request, Response } from './protocol';
import type { BuildInfo, ConvertTarget, CoreOptions, CoreResult, DecodedImage, DecodedMesh, PixieApi } from './types';

interface Pending {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
  /** Stops listening to the request's abort signal. */
  release?: () => void;
}

/** One engine worker. */
class WorkerClient {
  private readonly pending = new Map<number, Pending>();
  private nextId = 0;
  /** Set when the worker couldn't start (missing chunk, CSP, offline); later calls fail fast. */
  broken: Error | null = null;
  /** Stopped to cancel a request: the pool drops it and starts another when needed. */
  stopped = false;
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
    pending.release?.();
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
    for (const { reject, release } of this.pending.values()) {
      release?.();
      reject(error);
    }
    this.pending.clear();
  }

  /**
   * Sends a request. Aborting `signal` stops the worker mid-task (WebAssembly
   * can't be interrupted otherwise), failing whatever else it was doing too.
   */
  call<T>(op: Operation, args: unknown[], transfer: Transferable[], signal?: AbortSignal): Promise<T> {
    if (this.broken) return Promise.reject(this.broken);
    if (signal?.aborted) return Promise.reject(signal.reason as Error);
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      const abort = () => this.stop(signal!.reason as Error);
      signal?.addEventListener('abort', abort, { once: true });
      const release = signal ? () => signal.removeEventListener('abort', abort) : undefined;
      this.pending.set(id, { resolve: resolve as (value: unknown) => void, reject, release });
      const request: Request = { id, op, args };
      try {
        this.worker.postMessage(request, transfer);
      } catch (error) {
        // Not sent (such as a buffer already moved elsewhere): fail this request alone.
        this.pending.delete(id);
        release?.();
        reject(error instanceof Error ? error : new Error(String(error)));
        this.onSettled();
      }
    });
  }

  terminate() {
    this.stop(new Error('The engine worker was stopped'));
  }

  private stop(reason: Error) {
    this.stopped = true;
    this.rejectAll(reason);
    this.worker.terminate();
    this.onSettled();
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
    this.dropStopped();
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

  private dropStopped() {
    for (let i = this.workers.length - 1; i >= 0; i--) if (this.workers[i].stopped) this.workers.splice(i, 1);
  }

  private settled() {
    this.dropStopped();
    if (this.workers.length === 0 || this.workers.some((worker) => worker.load > 0)) return;
    clearTimeout(this.idleTimer);
    const extra = this.workers.length > 1;
    this.idleTimer = setTimeout(
      () => {
        // Each stop settles the pool again, which sets the timer for the one left.
        for (const worker of this.workers.splice(extra ? 1 : 0)) worker.terminate();
      },
      extra ? IDLE_MS : LAST_IDLE_MS,
    );
  }

  /** Sends `op`; `data` moves to the worker when `transfer` is set, rather than being copied. */
  private call<T>(
    op: Operation,
    data: Uint8Array | undefined,
    transfer: boolean,
    signal: AbortSignal | undefined,
    ...rest: unknown[]
  ): Promise<T> {
    const args = data === undefined ? rest : [data, ...rest];
    return this.pick().call<T>(op, args, transfer && data ? [data.buffer as ArrayBuffer] : [], signal);
  }

  info() {
    return this.call<BuildInfo>('info', undefined, false, undefined);
  }

  detect(data: Uint8Array) {
    return this.call<string>('detect', data, false, undefined);
  }

  optimize(data: Uint8Array, options: CoreOptions = {}, transfer = false, signal?: AbortSignal) {
    return this.call<CoreResult>('optimize', data, transfer, signal, options);
  }

  convert(data: Uint8Array, target: ConvertTarget, options: CoreOptions = {}, transfer = false, signal?: AbortSignal) {
    return this.call<CoreResult>('convert', data, transfer, signal, target, options);
  }

  decodeImage(data: Uint8Array, transfer = false) {
    return this.call<DecodedImage>('decodeImage', data, transfer, undefined);
  }

  decodeMesh(data: Uint8Array, transfer = false) {
    return this.call<DecodedMesh>('decodeMesh', data, transfer, undefined);
  }
}

let client: PixieApi | null = null;

/** The shared engine client; workers start when there is work for them. */
export function getPixie(): PixieApi {
  client ??= new EnginePool();
  return client;
}
