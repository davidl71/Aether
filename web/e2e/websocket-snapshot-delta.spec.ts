import { test, expect } from '@playwright/test';

/**
 * E2E: PWA receives initial WebSocket snapshot then delta updates from Rust backend.
 *
 * Prerequisite: Backend must be running on port 8080, e.g.:
 *   cd agents/backend && cargo run -p backend_service
 *
 * Playwright's webServer builds the app with VITE_API_URL=http://127.0.0.1:8080
 * and serves it, so the PWA connects to ws://127.0.0.1:8080/ws.
 */
test.describe('WebSocket snapshot and delta', () => {
  test('PWA receives initial snapshot then at least one delta over WebSocket', async ({
    page,
  }) => {
    const frames: Array<{ type: string; payload: unknown }> = [];

    page.on('websocket', (ws) => {
      const url = ws.url();
      if (!url.includes('/ws')) return;
      ws.on('framereceived', (event) => {
        try {
          const raw =
            typeof event.payload === 'string'
              ? event.payload
              : Buffer.isBuffer(event.payload)
                ? event.payload.toString('utf8')
                : '';
          const parsed = JSON.parse(raw) as { type?: string; data?: unknown; sections?: unknown };
          if (parsed?.type) {
            frames.push({
              type: parsed.type,
              payload: parsed.type === 'snapshot' ? parsed.data : parsed.type === 'delta' ? parsed.sections : undefined,
            });
          }
        } catch {
          // ignore non-JSON frames
        }
      });
    });

    await page.goto('/');

    // Wait for first message: must be snapshot with data
    await expect
      .poll(
        () => frames.find((f) => f.type === 'snapshot'),
        { timeout: 10_000 }
      )
      .toBeDefined();

    const snapshotFrame = frames.find((f) => f.type === 'snapshot');
    expect(snapshotFrame).toBeDefined();
    expect(snapshotFrame!.payload).toBeDefined();
    expect(typeof snapshotFrame!.payload).toBe('object');

    // Trigger a state change so backend sends a delta (strategy start/stop updates snapshot)
    const apiBase = 'http://127.0.0.1:8080';
    try {
      await fetch(`${apiBase}/api/v1/strategy/start`, { method: 'POST' });
    } catch {
      await fetch(`${apiBase}/api/v1/strategy/stop`, { method: 'POST' }).catch(() => {});
    }

    // Wait for at least one delta (backend sends every 2s when sections change)
    await expect
      .poll(
        () => frames.find((f) => f.type === 'delta'),
        { timeout: 10_000 }
      )
      .toBeDefined();

    const deltaFrame = frames.find((f) => f.type === 'delta');
    expect(deltaFrame).toBeDefined();
    expect(deltaFrame!.payload).toBeDefined();
    expect(typeof deltaFrame!.payload).toBe('object');
  });
});
