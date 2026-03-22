# Contributing to Aether

Thank you for contributing! This project is a Rust-first multi-asset synthetic financing platform.

## Quick Start

```bash
# Clone
git clone git@github.com:davidl71/Aether.git
cd Aether

# Build Rust backend
cd agents/backend
cargo build

# Run backend service (:8080)
cargo run -p backend_service

# Run TUI (separate terminal)
cargo run -p tui_service

# Run CLI
cargo run -p cli
```

## Build & Test

```bash
cd agents/backend

# Build
cargo build

# Test
cargo test

# Lint
cargo fmt && cargo clippy

# Full project lint
./scripts/run_linters.sh
```

## Code Style

Follow [AGENTS.md](AGENTS.md) for complete guidelines. Rust conventions:

| Element | Convention |
|---------|------------|
| Indentation | 4 spaces |
| Functions | `snake_case` |
| Types | `PascalCase` |
| Constants | `k` prefix |

## Project Structure

```
agents/backend/
├── crates/           # api, broker_engine, ib_adapter, ledger, market_data,
│                     # nats_adapter, quant, risk, strategy, etc.
├── services/         # backend_service, tui_service, tws_yield_curve_daemon
├── bin/cli           # CLI entry point
└── Cargo.toml       # Workspace
```

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and test**
   ```bash
   cargo test
   cargo fmt && cargo clippy
   ```

3. **Commit with descriptive messages**
   - Imperative mood: "Add feature" not "Added feature"
   - Subject line under 72 characters
   - Include context in body when needed

4. **Push and create PR**
   - Reference related issues
   - Include test output if applicable

## Testing Requirements

- All trading logic must have `#[test]` tests
- Risk calculations require test coverage
- Run `cargo test` before pushing

## Security Guidelines

- **Never** commit credentials, API keys, or secrets
- **Always** use paper trading port `7497` for testing
- Gate live trading behind configuration flags

## Getting Help

- [AGENTS.md](AGENTS.md) — canonical project guidelines
- [ARCHITECTURE.md](ARCHITECTURE.md) — system overview
- `docs/` — additional documentation
