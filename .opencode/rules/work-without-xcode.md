---
description: Work without Xcode command-line tools using rustfmt, grep, and targeted cargo check
alwaysApply: false
---

# Working Without Xcode Command-Line Tools

**Problem:** `cargo build`, `cargo test`, and `cargo clippy` require native crates (`ring`, `aws-lc-sys`, `libsqlite3-sys`, `time-macros`) that need a C compiler. Without Xcode command-line tools, these fail at the linking stage.

**Root cause:** Every workspace member transitively depends on native crates:
- `quant` → `aws-lc-sys`, `ring` (via `jiff`, `rust_decimal`)
- `nats_adapter` → `async-nats` → `rustls`, `openssl-sys`
- `api` → `sqlx` → `libsqlite3-sys`
- `ledger` → `sqlx`
- `backend_service` → `nats_adapter` → `async-nats`

No workspace crate escapes the native-dep chain.

## What Works Without Xcode

| Tool | Works? | Use |
|------|--------|-----|
| `rustfmt --edition 2021 --check` | ✅ | Format/style check (pure Rust parsing) |
| `rg` / `grep` | ✅ | Dead code, pattern search |
| `cargo metadata` | ✅ | Dependency analysis |
| `cargo search` | ✅ | Crate lookup |
| Text editors (VS Code, etc.) | ✅ | LSP if rust-analyzer installed |

## What Requires Xcode

| Tool | Why |
|------|-----|
| `cargo build` | Native crate linking |
| `cargo test` | Build + linking |
| `cargo clippy` | Build + linking |
| `cargo check` | Build scripts for deps |

## Workflow Without Xcode

### 1. Format check (fast, no linking)
```bash
# Check all Rust files
cd agents/backend && rustfmt --edition 2021 --check $(find . -name "*.rs" -not -path "./target/*") 2>&1 | head -50

# Fix formatting
rustfmt --edition 2021 $(find . -name "*.rs" -not -path "./target/*")
```

### 2. Dead code analysis (grep-based)
```bash
# Find structs/enums never constructed or matched
rg 'pub struct \w+\s*\{' --type rust | grep -v '^.*://' | head -50

# Find trait objects never used
rg 'dyn \w+Trait' --type rust

# Find allow(dead_code) overrides
rg 'allow\(dead_code\)' --type rust
```

### 3. Dependency analysis (cargo metadata)
```bash
# List native vs pure crates
cargo metadata --format-version 1 2>/dev/null | python3 -c "
import json,sys
data = json.load(sys.stdin)
native = {'ring','aws-lc-sys','libsqlite3-sys','time-macros','async-nats'}
for pkg in data['packages']:
    deps = {d['name'] for d in pkg.get('dependencies',[])}
    if deps & native:
        print(f'NATIVE: {pkg[\"name\"]}')"

# Find which crates depend on a specific crate
cargo metadata --format-version 1 2>/dev/null | python3 -c "
import json,sys
data = json.load(sys.stdin)
target = 'sqlx'
for pkg in data['packages']:
    deps = {d['name'] for d in pkg.get('dependencies',[])}
    if target in deps:
        print(f'{pkg[\"name\"]} -> {target}')"
```

### 4. Manual type checking
Since `cargo check` is unavailable, verify correctness by:
- Reading function signatures and ensuring field names match
- Checking `From` trait implementations for type conversions
- Verifying `#[derive(...)]` attributes include all needed traits

### 5. Code review checklist before Xcode is restored
- [ ] `rustfmt` passes on all modified files
- [ ] No new `unwrap()` or `expect()` in hot paths
- [ ] New structs use `derive_builder::Builder` where appropriate
- [ ] No duplicate type definitions (check `common/` first)
- [ ] All `todo!()` and `unimplemented!()` have exarp-go tasks

## Getting Xcode Back

```bash
# Option 1: Install command-line tools
sudo xcode-select --install

# Option 2: If already installed at non-default location
sudo xcode-select --switch /path/to/Xcode.app

# Verify
cc --version
xcode-select -p
```

Once Xcode is restored: `cd agents/backend && cargo build && cargo test`
