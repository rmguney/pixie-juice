import { defineConfig, devices } from '@playwright/test';

// E2E_PORT runs the suite beside a dev server that already holds the default port.
const port = Number(process.env.E2E_PORT) || 3000;

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  reporter: [['html', { outputFolder: 'test-results/html', open: 'never' }], ['list']],
  outputDir: 'test-results/artifacts',

  use: {
    baseURL: `http://localhost:${port}`,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        // Headless Firefox has no WebGL on Linux, so CI runs it headed on a virtual display
        // (xvfb, see ci.yml) and allows WebGL on that display's software renderer.
        headless: !process.env.E2E_FIREFOX_HEADED,
        launchOptions: { firefoxUserPrefs: { 'webgl.force-enabled': true } },
      },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  // CI tests the production bundle Vercel ships (the workflow builds it first); locally, the dev server.
  webServer: {
    command: `npm run ${process.env.CI ? 'preview' : 'dev'} -- --port ${port} --strictPort`,
    url: `http://localhost:${port}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },

  timeout: 60000,
  expect: {
    timeout: 10000,
  },
});
