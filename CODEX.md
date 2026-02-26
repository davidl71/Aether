# Codex Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines.

## Overview

Comprehensive multi-asset synthetic financing platform. Manages financing across options, futures, bonds, loans, and pension funds with unified portfolio management across 21+ accounts and multiple brokers (IBKR, Alpaca, Tradier, Tastytrade). Box spreads are one active strategy (spare cash, T-bill-equivalent yields). Multi-language: C++ core, Python integration, Rust backend.

## Structure

- `native/src/` — C++ source files
- `native/include/` — C++ headers
- `native/tests/` — Catch2 tests (mirror source names with `test_` prefix)
- `native/CMakeLists.txt` — main build definition
- `python/` — Python integration layer
- `agents/` — Rust backend services
- `config/` — configuration templates
- `docs/` — documentation

## Build & Test

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
ninja -C build
ctest --test-dir build --output-on-failure
./scripts/run_linters.sh
```

## Style

- C++20, 2-space indent, Allman braces, 100-char lines
- Types: `PascalCase` — Functions/variables: `snake_case` — Constants: `kPrefixed`
- Python: use `uv` for deps when available

## Rules

- Never commit credentials or secrets
- Use paper trading port 7497
- Never modify `native/third_party/` directly
- All trading/risk calculations must have tests
- Imperative commit messages, 72-char subject lines
