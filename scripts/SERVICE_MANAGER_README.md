# Service Manager

`scripts/service_manager.sh` manages the active local platform services from one entrypoint.

Supported runtime services:
- `rust_backend`
- `nats`
- `israeli_bank_scrapers`

The React/Vite web app is retired as an active runtime surface and is not managed here.

## Usage

```bash
./scripts/service_manager.sh start <service>
./scripts/service_manager.sh stop <service>
./scripts/service_manager.sh restart <service>
./scripts/service_manager.sh status [service]
./scripts/service_manager.sh start-all
./scripts/service_manager.sh stop-all
./scripts/service_manager.sh list
```

Examples:

```bash
./scripts/service_manager.sh start rust_backend
./scripts/service_manager.sh status
./scripts/service_manager.sh stop-all
```

## Active Services

| Service | Port | Purpose |
|---------|------|---------|
| `rust_backend` | 8080 | Shared Rust API/backend |
| `nats` | 4222 | NATS broker |
| `israeli_bank_scrapers` | 8010 | Israeli bank scraping service |

Ports come from `config/config.json` when present and fall back to the defaults above.

## Logs

Each service writes to:

```text
logs/<service>_service.log
```

## Notes

- Use this manager for active TUI/CLI-era services only.
- Interactive Brokers public routes are now served by the Rust backend; there is no separate `ib` runtime service here.
- Risk-free-rate/benchmark routes are now Rust-owned; the remaining Python implementation is internal and is not managed as a standalone public service here.
- Discount Bank public routes are now Rust-owned and are not managed as a standalone Python service here.
- Historical browser/PWA scripts remain under `web/` as archive/reference material.
- Old one-off `start_*` / `stop_*` wrappers are convenience shims and should not be expanded with new runtime surfaces.
