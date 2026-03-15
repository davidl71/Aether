# Development Tools

## Platform and crate audits

- **[RUST_CRATE_OPPORTUNITIES_AUDIT.md](platform/RUST_CRATE_OPPORTUNITIES_AUDIT.md)** — Places where hand-rolled or duplicated logic could use existing crates (JSONC, backoff, config, mocks). Implemented items: `jsonc-parser` (api, tui_service), `backoff` (tui_service circuit breaker, nats_adapter DLQ).
- **[STUB_CODE_PLANNING.md](platform/STUB_CODE_PLANNING.md)** — Stub/placeholder audit: what can be implemented with no questions (e.g. FMP fundamentals routes) vs human review (ib_adapter scope, runtime_state placeholders, TUI event routing).

## Code Navigation

- **ctags** - Generate tags for code navigation

  ```bash
  ctags -R .
  ```

- **tagref** - Check cross-references in code comments

  ```bash
  brew install tagref
  # Run check
  tagref --upstream-refs .
  ```

  Format: `# [tag:tagname]` and `# [ref:tagname]`

- **xrefcheck** - Check cross-references in documentation (markdown)

  ```bash
  cargo install xrefcheck
  # Run check
  xrefcheck -i docs/
  ```

## Installation

```bash
# macOS
brew install universal-ctags tagref

# Rust tools
cargo install xrefcheck
```
