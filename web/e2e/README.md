# E2E tests

## WebSocket snapshot + delta

**Prerequisite:** Rust backend must be running on port 8080:

```bash
cd agents/backend && cargo run -p backend_service
```

Then from `web/`:

```bash
npm run e2e
```

The test builds the PWA with `VITE_API_URL=http://127.0.0.1:8080`, serves it via `vite preview`, opens the app in Chromium, and asserts:

1. First WebSocket message is `type: "snapshot"` with full `data`.
2. After triggering a strategy state change (REST), at least one message is `type: "delta"` with `sections`.

See `e2e/websocket-snapshot-delta.spec.ts`.
