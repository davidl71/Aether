---
name: trading-safety-checklist
description: Short checklist for changes that touch orders, risk, margin, or live market data. Use when editing order flow, risk calculator, margin calculator, live TWS/NATS data paths, or config that affects trading.
---

# Trading Safety Checklist

Run this checklist when changing code that affects orders, risk, margin, or live data.

## Before merge / push

- [ ] **Paper port:** Testing uses port **7497** (paper trading). No hardcoded live port in changed code.
- [ ] **No credentials:** No API keys, secrets, or credentials in code or logs. No new env vars that leak secrets.
- [ ] **Live trading gated:** Any live order or market path is behind an explicit config flag or env; not on by default.
- [ ] **Config validated:** New or changed config is validated before use (e.g. config manager or startup checks).

## Tests and security

- [ ] **Relevant tests run:** `just test` (C++), and/or Rust tests for touched crates (e.g. `cargo test -p <crate>` from `agents/backend`). All pass.
- [ ] **Security scan:** If the project uses Semgrep (see `.cursor/rules/semgrep.mdc`), run the security check for the changed paths before suggesting merge.

## Reminders (do not commit)

- Build artifacts, logs, credentials.
- Hardcoded absolute paths — use relative paths or env.
- Deprecated APIs — see `docs/TWS_INTEGRATION_STATUS.md` for current APIs.

Keep the checklist short: bullet list + "run X" / "check Y". No long prose.
