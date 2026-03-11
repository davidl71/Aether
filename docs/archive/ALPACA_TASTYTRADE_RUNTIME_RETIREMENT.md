## Alpaca and Tastytrade Runtime Retirement

As of March 11, 2026, Alpaca and Tastytrade are retired from the active runtime surface.

What this means:
- no active `api-gateway` routing for Alpaca or Tastytrade
- no TUI provider presets or default health/status wiring for those services
- no active web status/account assumptions for those services
- no active supervisor/service-manager entries for those services

Why:
- current migration work is focused on reducing Python specialist-service surface
- IB plus Rust-owned shared APIs remain the active supported trading path
- carrying half-supported Alpaca/Tastytrade runtime paths slowed frontend and gateway simplification

Future reimplementation boundary:
- if either broker returns, the preferred shape is an explicit specialist integration behind the Rust shared origin
- reintroduction should come with:
  - a clear owner
  - one public API surface
  - no parallel direct frontend/TUI routing assumptions

This note is intentionally short. Historical/research docs may still mention Alpaca or Tastytrade, but they are not active runtime guidance.
