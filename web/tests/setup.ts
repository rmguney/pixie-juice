import '@testing-library/jest-dom';
import { vi } from 'vitest';

globalThis.console = {
  ...console,
  log: vi.fn(),
  debug: vi.fn(),
  info: vi.fn(),
  warn: vi.fn(),
  error: vi.fn(),
};
