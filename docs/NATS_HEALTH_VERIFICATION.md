# NATS transport health — verification guide

**Audience:** Operators and developers validating that NATS messaging and transport telemetry are healthy end-to-end (backend publisher, health aggregation, TUI, optional REST).

**See also:** Topic names and constants in [`docs/NATS_TOPICS_REGISTRY.md`](NATS_TOPICS_REGISTRY.md). Proto shapes: `NatsTransportHealth` and `SystemSnapshot.nats_transport` in [`proto/messages.proto`](../proto/messages.proto).

---

## Where transport state comes from

### Backend snapshot publisher (`snapshot-publisher`)

On each publish cycle, `backend_service` updates `SystemSnapshot.nats_transport` with a [`NatsTransportHealthState`](../agents/backend/crates/nats_adapter/src/health.rs) built from the snapshot client: NATS flush RTT (or disconnect on flush failure), `async-nats` client statistics (bytes/messages/connects), optional JetStream stream readiness, publish error counts, and snapshot metadata (`snapshot_backend_id`, `snapshot_generated_at`). That struct is written into the in-memory snapshot before the snapshot is encoded and published.

Relevant code: [`snapshot_publisher.rs`](../agents/backend/services/backend_service/src/snapshot_publisher.rs) (loop that sets `snap.nats_transport` and then calls `snapshot_to_proto` / publish).

The same field is serialized on the wire as optional protobuf `NatsTransportHealth` on `SystemSnapshot` (field `15` in `messages.proto`).

### Backend health aggregation (`subscriber` on `system.health`)

A separate task connects to NATS, subscribes to `system.health` (see `topics::system::health()` in `nats_adapter::topics`), and maintains `HealthAggregateState.transport` with role **`subscriber`**, subject **`system.health`**. It records connect/subscribe failures, subscription end, and refreshed byte/message stats on each health message.

Relevant code: [`health_aggregation.rs`](../agents/backend/services/backend_service/src/health_aggregation.rs).

This aggregate drives **optional** JSON health over HTTP (see below), not the main NATS data path.

### TUI: snapshot-embedded vs local subscriber

1. **From snapshots** — When a `SystemSnapshot` arrives on `snapshot.{BACKEND_ID}` and carries `nats_transport`, the TUI copies it into app state (publisher-side telemetry: role `snapshot-publisher` in the embedded object).

2. **From the subscriber loop** — The TUI NATS task emits `AppEvent::TransportHealth` when the snapshot subscription connects or disconnects (role `snapshot-subscriber`), and the nested `system.health` subscriber emits updates on each health message (role `health-subscriber`). Those events update `App::nats_transport`.

Relevant code:

- Subscriber and health task: [`nats.rs`](../agents/backend/services/tui_service/src/nats.rs)
- Merging snapshot transport into app state: [`app_updates.rs`](../agents/backend/services/tui_service/src/app_updates.rs) (`set_snapshot` copies `s.inner.nats_transport` when present)
- App field: [`app.rs`](../agents/backend/services/tui_service/src/app.rs) (`pub nats_transport: NatsTransportHealthState`)
- Settings UI: [`settings_health.rs`](../agents/backend/services/tui_service/src/ui/settings_health.rs) (transport section under Settings → Health)

**Operator confusion:** If NATS is up but the TUI still shows degraded status, see the comment block at the top of `nats.rs` (circuit breaker, wrong `NATS_URL`, subscription ended).

### REST aggregate health (optional server)

`backend_service` does **not** expose aggregate health on the default service port. When **`REST_SNAPSHOT_PORT`** is set to a positive integer, an auxiliary Axum server binds `0.0.0.0:{port}` and serves:

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/health` | JSON [`HealthAggregateResponse`](../agents/backend/crates/api/src/health.rs): `transport`, `nats_connected`, per-backend map, counts, staleness flags; may augment transport with KV reachability when JetStream KV check is enabled. |
| `GET` | `/api/v1/snapshot` | Full snapshot (JSON or protobuf). |

Relevant code: [`rest_snapshot.rs`](../agents/backend/services/backend_service/src/rest_snapshot.rs).

**Note:** The protobuf comment on `HealthAggregate` in `messages.proto` mentions dashboard `GET /api/health`; the Rust `backend_service` optional endpoint implemented today is **`GET /health`** on the REST snapshot port. Use that path for automated checks.

---

## Manual checklist (happy path)

 Prerequisites: `NATS_URL` and `BACKEND_ID` consistent between `backend_service` and `tui_service` (same broker URL and backend id for `snapshot.{id}`).

1. **Start NATS** (e.g. local `nats-server` on `nats://127.0.0.1:4222`).
2. **Start `backend_service`** with `NATS_URL` set; ensure snapshot publishing is active (normal startup when NATS connects).
3. **Start `tui_service`** with matching `NATS_URL` / `BACKEND_ID`.
4. **TUI — Settings → Health**
   - Open the Settings tab (digit **`0`** in the default tab strip).
   - Confirm the **Health** section shows transport line(s): status, role (`snapshot-subscriber` / `health-subscriber` from local connection, and snapshot-derived lines may show publisher metadata when snapshots include `nats_transport`).
   - Expect **non-empty** byte/message counters over time while traffic flows.
5. **Optional REST** — Set `REST_SNAPSHOT_PORT=8081` (example), restart `backend_service`, then:
   ```bash
   curl -s "http://127.0.0.1:8081/health" | jq '{status, nats_connected, transport}'
   ```
   Expect `nats_connected` true when the health aggregator’s NATS connection is up; `transport` should reflect the **`subscriber`** / `system.health` path with recent `updated_at`.

## Manual checklist (failure / degraded)

1. With all services running and healthy, **stop NATS** (or block the port).
2. **TUI:** Status bar / Settings → Health should move toward **retrying**, **disconnected**, or circuit-breaker messaging per `nats.rs`; transport summary should show **error** or non-ok status and a useful `error` / `hint`.
3. **REST `/health`:** After NATS drops, expect `nats_connected` false and transport fields indicating disconnect (and overall `status` may be `degraded` or `error` depending on backend rows and staleness).
4. **Restart NATS** and confirm both TUI and `/health` recover without requiring a full machine reboot (reconnect loops and backoff apply).

## Automated / CI-friendly checks

- **Unit tests:** `agents/backend/crates/api/src/health.rs` includes tests that **`HealthAggregateResponse`** exposes transport metadata (`transport_subject`, counts, staleness). Run from repo:
  ```bash
  cd agents/backend && cargo test -p api health_
  ```
- **REST:** If `REST_SNAPSHOT_PORT` is enabled in an environment, scripted polling of `GET /health` can assert `jq -e '.transport.status == "ok"'` when all backends are healthy and NATS is up (adjust for your expected backend set).
- **Proto round-trip:** Snapshot proto conversion tests in `api` cover `nats_transport` encode/decode; use `cargo test -p api` if you change proto fields.

---

## Trading / safety notes

- This document is **read-only observability**. No order or risk actions depend on these fields.
- Use **paper / dev** NATS and URLs when testing disconnect scenarios.
- Staleness: aggregate health treats transport as stale after a default horizon (see `DEFAULT_HEALTH_STALE_AFTER_SECS` in `api::health`); a quiet but connected NATS may still surface as degraded if no health traffic updates timestamps.
