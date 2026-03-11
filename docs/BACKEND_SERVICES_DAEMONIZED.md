# Backend Services - Active Daemons

This file tracks the active local daemon set for the current TUI/CLI-era runtime.

## Active daemonized services

| Service | Port | Managed by | Role |
|---------|------|------------|------|
| `nats` | 4222 | `scripts/service_manager.sh`, supervisor configs | NATS broker and JetStream |
| `ib` | 8002 | `scripts/service_manager.sh`, supervisor configs | Interactive Brokers specialist service |
| `health_dashboard` | 8011 | `scripts/service_manager.sh`, supervisor configs | Internal `system.health` aggregation behind Rust health routes |
| `rust_backend` | 8080 | `scripts/service_manager.sh`, supervisor configs | Shared Rust API/backend |

## Notes

- `health_dashboard` is still active because Rust health routes proxy to it through `HEALTH_DASHBOARD_URL`.
- `collection-daemon` remains an active Go daemon by architecture, but it is not managed by these legacy Python-oriented daemon wrappers.
- Alpaca, Tastytrade, Discount Bank, risk-free-rate, and web-specific daemons are retired from the active runtime.

## Historical runtime surfaces

Retired from active daemon management:

- `alpaca`
- `tastytrade`
- `discount_bank`
- `risk_free_rate`
- `web`

Keep any remaining mentions of those services in archive/research material only.
