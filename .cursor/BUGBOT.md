# Bugbot Review Rules for Aether

This file provides project-specific context for Cursor Bugbot when reviewing pull requests.

## Project Overview

Multi-asset synthetic financing platform; **Rust-first**. Rust backend (`agents/backend/`) is the primary runtime (API, TUI, ib_adapter, quant, risk). C++ native build has been removed; no `native/` tree. Box spreads are one strategy component, not the whole product.

## Critical Security Requirements

### Trading Software Safety

- **NEVER** commit credentials, API keys, or secrets
- **NEVER** log sensitive information (account numbers, positions, PII)
- **ALWAYS** use paper trading port (7497) for testing
- **GATE** live trading behind explicit configuration flags
- **VALIDATE** all configuration before use
- **REQUIRE** explicit confirmation for any live trading operations

### Code Security Checks

- Scan for hardcoded credentials or API keys
- Verify no sensitive data in logs or error messages
- Check that authentication tokens are never committed
- Ensure environment variables are used for secrets
- Validate input sanitization for all user inputs

## Code Style Requirements

### Rust (primary) and style

- **Rust**: Primary codebase in `agents/backend/` (crates + services). Follow AGENTS.md and CLAUDE.md; `cargo fmt`, `cargo clippy`, `just build-rust`, `just test`.
- **File organization**: Core logic in `agents/backend/crates/`; scripts in `scripts/`; generated output in `build/`, `agents/backend/target/` (disposable).

## Build System Requirements

### Rust build and test

- **Build**: `just build-rust` or `cargo build` in `agents/backend/`
- **Test**: `just test` or `cargo test` in `agents/backend/`
- **Lint**: `just lint` or `./scripts/run_linters.sh` (includes Rust fmt + clippy)

## Testing Requirements

- **All tests must pass** before merging
- Tests mirror source file names (use Catch2 framework)
- Run: `ctest --output-on-failure`
- New features require corresponding tests
- Critical trading logic requires comprehensive test coverage

## Static Analysis

- Run linters before committing: `just lint` or `./scripts/run_linters.sh` (Rust: fmt + clippy; shell, Ansible, etc.)
- Rust: `cargo clippy` with `-D warnings`; use `#[must_use]`, `Result`/`Option` appropriately
- See `docs/STATIC_ANALYSIS_ANNOTATIONS.md` for legacy C++ annotations (reference only)

## Common Issues to Flag

### Security Issues

- Hardcoded credentials or API keys
- Sensitive data in logs
- Missing input validation
- Unsafe memory operations (buffer overflows, use-after-free)
- Missing error handling for critical operations

### Code Quality Issues

- Rust: `cargo fmt` and `cargo clippy` must pass; follow project naming and style (see AGENTS.md)
- Missing or incorrect naming conventions
- Missing comments for non-obvious trading math

### Trading Logic Issues

- Missing validation for financial calculations
- Incorrect decimal precision handling
- Missing error handling for API failures
- Race conditions in concurrent operations
- Missing bounds checking for market data

### Build System Issues

- Missing CMake configuration
- Incorrect dependency paths
- Missing test targets
- Build artifacts committed to repository

## Pre-Commit Checklist

Before approving a PR, verify:

1. ✅ All tests pass (`just test` or `cargo test` in `agents/backend/`)
2. ✅ Linters pass (`just lint` or `./scripts/run_linters.sh`)
3. ✅ Build succeeds (`just build-rust` or `cargo build` in `agents/backend/`)
4. ✅ No credentials or secrets committed
5. ✅ Code follows style guidelines (Rust: fmt + clippy; see AGENTS.md)
6. ✅ Documentation updated if needed
7. ✅ Critical paths have tests

## Documentation Requirements

- Update `docs/API_DOCUMENTATION_INDEX.md` for API changes
- Add comments for non-obvious trading math
- Update README if project structure changes
- Document new configuration options

## Multi-Language Considerations

This project is **Rust-first**:

- **Rust**: Primary codebase (`agents/backend/` — API, TUI, ib_adapter, quant, risk). Core build, test, and lint target.
- **Python**: Scripts and optional agents; no active C++ bindings. Web and legacy C++ native build removed.

When reviewing: verify Rust conventions (fmt, clippy), cross-crate and REST contracts, and that tests cover changed paths.

## Git Workflow

- Use `git worktree add <path> [branch]` for new worktrees
- Commit messages: Imperative mood, 72-character subject lines
- Never commit build artifacts, logs, or credentials
- PRs should be focused and atomic

## References

- `.cursorrules` - Main project rules
- `docs/STATIC_ANALYSIS_ANNOTATIONS.md` - Static analysis guidelines
- `docs/API_DOCUMENTATION_INDEX.md` - API documentation
- `docs/CURSOR_SETUP.md` - Cursor configuration
