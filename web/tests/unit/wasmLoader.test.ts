import { describe, it, expect, vi } from 'vitest';

describe('WasmLoader', () => {
  it('should export initWasm function', async () => {
    const wasmLoader = await import('../../src/utils/wasmLoader');
    expect(typeof wasmLoader.initWasm).toBe('function');
  });

  it('should export isWasmReady function', async () => {
    const wasmLoader = await import('../../src/utils/wasmLoader');
    expect(typeof wasmLoader.isWasmReady).toBe('function');
  });

  it('should export getWasm function', async () => {
    const wasmLoader = await import('../../src/utils/wasmLoader');
    expect(typeof wasmLoader.getWasm).toBe('function');
  });

  it('isWasmReady should return false initially', async () => {
    vi.resetModules();
    const wasmLoader = await import('../../src/utils/wasmLoader');
    expect(wasmLoader.isWasmReady()).toBe(false);
  });
});

describe('Type Definitions', () => {
  it('should have proper WasmModule type', async () => {
    const types = await import('../../src/types');
    expect(types).toBeDefined();
  });
});
