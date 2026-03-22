# Quick Start — Rust Development

Get up and running with the Rust backend in minutes.

## Prerequisites

- **Rust** 1.75+ (stable)
- **macOS** 11+ / **Linux** / **Windows** 10/11 (64-bit)
- **NATS** 2.10+ (for live state)
- **IBKR TWS or IB Gateway** (for live trading)

## 1. Install Rust

```bash
# macOS
brew install rust

# Linux/Windows
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify
rustc --version  # Should be 1.75+
```

## 2. Clone & Build

```bash
git clone git@github.com:davidl71/Aether.git
cd Aether/agents/backend
cargo build
```

## 3. Run Services

**Backend (REST + WebSocket on :8080):**
```bash
cargo run -p backend_service
```

**TUI (ratatui dashboard):**
```bash
cargo run -p tui_service
```

**CLI:**
```bash
cargo run -p cli -- --help
```

## 4. Configure

```bash
# Copy example config
cp config/config.example.json config/config.json

# Edit settings
nano config/config.json
```

**Important:**
- `tws.port: 7497` = Paper Trading (safe)
- `tws.port: 7496` = Live Trading (real money!)
- `dry_run: true` = Simulate without executing trades

## 5. Verify

```bash
# Check backend is running
curl http://localhost:8080/health

# Check API
curl http://localhost:8080/api/v1/snapshot
```

## Common Tasks

### Add a new crate

```bash
# 1. Create crate
cargo new --lib crates/my_new_crate

# 2. Add to workspace in Cargo.toml
members = [..., "crates/my_new_crate"]

# 3. Add tests
mkdir -p crates/my_new_crate/tests/
```

### Run tests

```bash
# All tests
cargo test

# Single crate
cargo test -p ib_adapter

# With output
cargo test -- --nocapture
```

### Lint

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Troubleshooting

**Build fails with missing dependency:**
```bash
cargo fetch
```

**TWS connection issues:**
- Ensure TWS/Gateway is running
- Check paper trading port (7497) vs live (7496)
- Verify `client_id` doesn't conflict with other connections

**NATS connection issues:**
- Ensure NATS is running on port 4222
- Check `NATS_URL` env var if non-standard
