import { defineConfig, devices } from '@playwright/test';

/**
 * E2E tests for the PWA. For WebSocket snapshot+delta test:
 * - Backend must be running on port 8080 (e.g. from repo root:
 *   cd agents/backend && cargo run -p backend_service).
 * - This config builds the web app with VITE_API_URL pointing at that backend,
 *   then serves it via preview so the PWA connects to the real WebSocket.
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: 'list',
  timeout: 30_000,
  use: {
    baseURL: 'http://localhost:4173',
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'npm run build && npm run preview',
    url: 'http://localhost:4173',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    env: {
      VITE_API_URL: 'http://127.0.0.1:8080',
    },
  },
});
