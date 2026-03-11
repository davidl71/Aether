# Backend Services - Active Daemons

This file tracks the active local daemon set for the current TUI/CLI-era runtime.

## Active daemonized services

| Service | Port | Managed by | Role |
|---------|------|------------|------|
| `nats` | 4222 | `scripts/service_manager.sh`, supervisor configs | NATS broker and JetStream |
| `rust_backend` | 8080 | `scripts/service_manager.sh`, supervisor configs | Shared Rust API/backend |

## Notes

- Rust now aggregates `system.health` directly; there is no separate health dashboard daemon.
- `collection-daemon` remains an active Go daemon by architecture, but it is not managed by these legacy Python-oriented daemon wrappers.
- Interactive Brokers public routes are now Rust-owned; there is no standalone Python IB daemon in the active runtime.
- Alpaca, Tastytrade, Discount Bank, risk-free-rate, and web-specific daemons are retired from the active runtime.

## Historical runtime surfaces

Retired from active daemon management:

- `alpaca`
- `tastytrade`
- `discount_bank`
- `risk_free_rate`
- `health_dashboard`
- `web`

Keep any remaining mentions of those services in archive/research material only.
