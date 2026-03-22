# Service start deep check

When a service fails to start, use this runbook to verify environment, config, build, and startup scripts.

---

## 1. Which service?

| Service | Script / entry | Port(s) | Purpose |
|---------|----------------|---------|---------|
| **rust_backend** | `./scripts/service_manager.sh start rust_backend` or `./scripts/service.sh start rust` | 9090 (REST snapshot, only when `REST_SNAPSHOT_PORT` set) | Backend: NATS, snapshot, api handlers |
| **nats** | `./scripts/service_manager.sh start nats` or `./scripts/service.sh start nats` | 4222 (client), 8222 (HTTP), 8081 (WS) | Message broker |
| **tui_service** | Not managed by service_manager; run manually: `cargo run -p tui_service` | — | TUI client |

Start-all order (service.sh): **nats** → **rust** → memcached.

---

## 2. Prerequisites

- **Rust**: `cargo --version` and `rustc --version` (see `just verify-rust` or `./scripts/verify_toolchain.sh`).
- **NATS** (for rust_backend): `nats-server -js` or install via `brew install nats-server`. Backend runs without NATS but won’t publish snapshots.
- **Config**: Backend expects TOML at `agents/backend/config/default.toml` (or `BACKEND_CONFIG`); scripts use `config/config.json` or `config/config.example.json` for port/config.

---

## 3. Config and install paths

### Backend (backend_service)

- **Config file**: `BACKEND_CONFIG` env or default `config/default.toml` (relative to **CWD** when starting).  
  Start command is `cd ${PROJECT_ROOT}/agents/backend && cargo run ...`, so effective path is **`agents/backend/config/default.toml`**.
- **Optional override**: `agents/backend/config/config.local.toml` (gitignored) is merged on top if present.
- **Port**: Scripts read port from `rest_addr` in that TOML (e.g. `0.0.0.0:9090` → 9090). The backend **only binds** if **`REST_SNAPSHOT_PORT`** is set; `service_manager.sh` sets it when starting rust_backend so the port check passes.

### service_manager.sh

- **Port source**: `get_port rust_backend 9090` → from `config/config.json` (`.services.rust_backend.port`) or `config_get_rust_backend_port 9090` which reads **`agents/backend/config/default.toml`** or **`config/backend.toml`** (`rest_addr`).
- **REST_SNAPSHOT_PORT**: Set automatically by the script when starting rust_backend so the process listens on that port.

### service.sh (legacy / alternate)

- **Rust** is started via `./scripts/start_rust_backend.sh` → which runs `service_manager.sh start rust_backend`.  
- **Port**: `_svc_port rust` uses `config_get_rust_backend_port 9090`.  
- **Health**: `http://localhost:${PORT}/health` — the backend does **not** expose `/health`; it only serves `GET /api/v1/snapshot` when REST is enabled. So service.sh may report rust as **unhealthy** even when the process is up. Use “port in use” or log tail as the real indicator, or hit `curl -s http://localhost:9090/api/v1/snapshot` to confirm.

---

## 4. Build and run (manual)

From repo root:

```bash
# Build (debug)
cd agents/backend && cargo build -p backend_service

# Build (release, what service_manager uses)
cd agents/backend && cargo build --release -p backend_service

# Run with REST port so something listens (for script port check)
cd agents/backend && REST_SNAPSHOT_PORT=9090 NATS_URL=nats://localhost:4222 cargo run --release -p backend_service
```

If this fails, the error is the real startup failure (e.g. missing config, parse error, NATS connection). Check stderr.

---

## 5. What the start script does

1. **service_manager.sh start rust_backend**  
   - Resolves port (default 9090) from config.  
   - Optionally sources `scripts/include/onepassword.sh` and `export_op_secrets_for_rust`.  
   - Sets **`REST_SNAPSHOT_PORT=$port`** and runs:  
     `cd ${PROJECT_ROOT}/agents/backend && cargo run --release -p backend_service`  
   - Logs go to **`logs/rust_backend_service.log`**.  
   - For **rust_backend**, polls the port every 2s for up to **90s** (so a cold release build can finish). Other services use a 2s wait. If port never in use → “failed to start”.

2. **Why it might say “failed to start”**  
   - **Backend didn’t bind**: Before the fix, `REST_SNAPSHOT_PORT` was not set, so the backend never listened on 9090. Now the script sets it.  
   - **Cold release build**: First start (or after `cargo clean`) runs `cargo run --release`, which can take 30–90s to compile. The script now waits up to 90s for rust_backend before failing.  
   - **Cargo/build failure**: Backend process exits quickly; log file has the error.  
   - **Config missing/invalid**: `load_config()` fails → backend exits; see log.  
   - **Wrong CWD**: If something runs from a different CWD, `config/default.toml` may not be found.

---

## 6. Logs and errors

| Where | What to check |
|-------|----------------|
| **logs/rust_backend_service.log** | Full stdout/stderr of `cargo run -p backend_service`. Build errors, panic, “unable to read config”, NATS errors. |
| **logs/nats_service.log** | Only if you start nats via service_manager. |
| **service.sh** | Uses **logs/rust-backend.log** (different name). |

Example checks:

```bash
tail -100 logs/rust_backend_service.log
# or
tail -100 logs/rust-backend.log
```

Common log messages:

- `"failed to load backend config"` → Missing or invalid `config/default.toml` (or path in `BACKEND_CONFIG`).
- `"NATS integration unavailable, continuing without NATS"` → Expected if NATS_URL unset or nats-server not running.
- `"REST snapshot server bind failed"` → Port in use or permission; or REST_SNAPSHOT_PORT not set (script should set it now).

---

## 7. Quick checklist

- [ ] **Rust**: `cargo build -p backend_service` succeeds from `agents/backend`.
- [ ] **Config**: `agents/backend/config/default.toml` exists (or `BACKEND_CONFIG` points to a valid TOML).
- [ ] **Port 9090 free**: `lsof -i :9090` (or your configured port) is empty before start.
- [ ] **NATS** (optional): `nats-server -js` running if you want snapshot publishing; backend starts without it.
- [ ] **Start**: `./scripts/service_manager.sh start rust_backend`; then `tail -20 logs/rust_backend_service.log`.
- [ ] **REST_SNAPSHOT_PORT**: Script now sets it; if you start the backend manually and want the script’s port check to pass, run with `REST_SNAPSHOT_PORT=9090`.

---

## 8. References

- **Backend config**: `agents/backend/config/default.toml`, `BACKEND_CONFIG`, `docs/platform/BACKEND_CONFIG_ENV_OVERLAY.md`
- **NATS**: `docs/platform/NATS_API.md`, `docs/TWS_ORATS_PORTAL_QUESTDB.md`
- **Scripts**: `scripts/service_manager.sh`, `scripts/service.sh`, `scripts/start_rust_backend.sh`, `scripts/include/config.sh`
- **REST snapshot**: Backend only binds when `REST_SNAPSHOT_PORT` is set; see `agents/backend/services/backend_service/src/rest_snapshot.rs`
