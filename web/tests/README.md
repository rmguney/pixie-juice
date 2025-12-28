# Pixie Juice Test Suite

Test suite for the Pixie Juice

## Architecture

```
tests/
├── setup.ts                     # Vitest test setup
├── unit/                        # Unit tests (Vitest)
│   └── wasmLoader.test.ts       # WASM loader utility tests
├── e2e/                         # E2E tests (Playwright)
│   ├── wasm-integration.spec.ts # WASM module integration
│   ├── image-compression.spec.ts # Image format compression
│   ├── mesh-optimization.spec.ts # Mesh format optimization
│   ├── compression-quality.spec.ts # Compression quality validation
│   ├── regression.spec.ts       # Regression testing
│   └── ui-integration.spec.ts   # UI/UX integration
└── fixtures/                    # Test data
    ├── regression-baseline.json # Regression baselines
    └── README.md
```

## Running Tests

### Unit Tests (Vitest)

```bash
npm run test              # Run once
npm run test:watch        # Watch mode
npm run test:coverage     # With coverage report
```

### E2E Tests (Playwright)

```bash
npm run test:e2e          # Run all E2E tests
npm run test:e2e:ui       # Interactive UI mode
npm run test:e2e:headed   # Run with visible browser
npm run test:e2e:debug    # Debug mode
npm run test:e2e:report   # View HTML report
```

### Regression Tests

```bash
npm run test:regression           # Run regression tests
npm run test:regression:update    # Update baseline
```

### All Tests

```bash
npm run test:all          # Unit + E2E tests
```

## Test Categories

### 1. WASM Integration (`wasm-integration.spec.ts`)

Tests WASM module loading and API exposure:

- Module initialization
- Function exports verification
- Format detection (PNG, JPEG, WebP, GIF)
- Performance metrics API

### 2. Image Compression (`image-compression.spec.ts`)

Tests image optimization across formats:

- PNG compression and output validation
- JPEG handling
- WebP support
- GIF optimization
- BMP conversion
- Quality settings (0-100)
- Error handling (empty input, invalid data, corrupted headers)
- Performance benchmarks

### 3. Mesh Optimization (`mesh-optimization.spec.ts`)

Tests 3D mesh format handling:

- OBJ format detection and optimization
- PLY (ASCII) processing
- STL format handling
- GLTF/GLB support
- Error handling for invalid mesh data
- Performance with large meshes

### 4. Compression Quality (`compression-quality.spec.ts`)

Validates compression produces good results:

- Solid color images (should achieve >80% compression)
- Gradient images (moderate compression expected)
- Noisy images (limited compression expected)
- Size scaling consistency
- Quality setting effects
- Lossless mode validation
- Output format validation

### 5. Regression (`regression.spec.ts`)

Prevents performance and quality regressions:

- Baseline comparison for compression ratios
- Performance timing regression
- Output size consistency
- Quality setting effects
- Snapshot tests for API surface

### 6. UI Integration (`ui-integration.spec.ts`)

Tests the web application:

- App title and branding
- File drop zone functionality
- WASM load failure handling
- Responsive design (mobile, tablet, desktop)
- Console error monitoring
- Page load performance
- Accessibility basics

## Configuration

### Playwright (`playwright.config.ts`)

- Runs on Chromium, Firefox, WebKit
- Auto-starts dev server on port 3000
- Screenshots on failure
- Trace on retry
- 60s test timeout

### Vitest (`vitest.config.ts`)

- jsdom environment for React testing
- Coverage via v8
- Global test utilities

## Writing New Tests

### E2E Test Template

```typescript
import { test, expect, Page } from '@playwright/test';

async function waitForWasmReady(page: Page): Promise<void> {
  await page.waitForFunction(
    () => {
      const win = window as unknown as { pixieJuice?: { version?: () => string } };
      return win.pixieJuice && typeof win.pixieJuice.version === 'function';
    },
    { timeout: 30000 }
  );
}

test.describe('My Test Suite', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should do something', async ({ page }) => {
    // Test implementation
  });
});
```

### Unit Test Template

```typescript
import { describe, it, expect } from 'vitest';

describe('MyModule', () => {
  it('should behave correctly', () => {
    expect(true).toBe(true);
  });
});
```

## CI/CD Integration

For GitHub Actions:

```yaml
- name: Install dependencies
  run: cd web && npm ci

- name: Install Playwright
  run: cd web && npx playwright install --with-deps chromium

- name: Build WASM
  run: cd web && npm run build:wasm

- name: Run tests
  run: cd web && npm run test:all
```

## Regression Baseline

The regression baseline (`fixtures/regression-baseline.json`) stores expected values for:

- Compression ratios per format/quality
- Processing times
- Output sizes

Update with:

```bash
UPDATE_BASELINE=true npm run test:e2e
```

## Troubleshooting

### Tests failing with WASM errors

Ensure WASM is built:

```bash
npm run build:wasm
```

### Playwright browser issues

Reinstall browsers:

```bash
npx playwright install
```

### Port 3000 in use

Kill existing process or modify `playwright.config.ts` to use different port.
