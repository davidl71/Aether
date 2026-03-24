# Third-Party Libraries And Licenses

Updated: 2026-03-24

## Scope

This document summarizes the **direct third-party software dependencies**
declared by the active Rust workspace in `agents/backend/`.

Included:

- direct crates declared in workspace or member `Cargo.toml` files
- direct build-only and dev/test-only crates when they are explicitly declared
- external vendor/runtime components referenced by the current platform docs

Excluded:

- transitive Cargo dependencies
- archived C++-only dependency plans under `docs/archive/`
- services or data vendors that are only discussed as future integrations

## Method

Inventory was compiled from:

- `agents/backend/Cargo.toml`
- member crate/service `Cargo.toml` files under `agents/backend/`
- `cargo metadata --format-version 1 --manifest-path agents/backend/Cargo.toml`
- current repo docs such as `AGENTS.md` and `docs/TRACKING_AND_GITIGNORE.md`

License values below come from Cargo package metadata for the currently resolved
crate versions in the local lock/registry state.

## Summary Table

| Library / component | Declared version | Where used | Purpose | License | Credit / upstream |
|---|---:|---|---|---|---|
| `tokio` | `1` | most backend crates and services | async runtime, tasks, channels, timers | MIT | Tokio project |
| `serde`, `serde_json` | `1` | most crates | serialization for config, snapshots, API payloads | MIT OR Apache-2.0 | Serde project |
| `tracing`, `tracing-subscriber` | `0.1`, `0.3` | most crates and services | structured logging and log filtering | MIT | Tokio tracing project |
| `axum` | `0.7` | `backend_service` | REST server layer | MIT | Tokio / axum |
| `reqwest` | `0.13` | `api`, `market_data`, `backend_service`, `tui_service`, `cli` | HTTP client for upstream services and fallbacks | MIT OR Apache-2.0 | reqwest / Sean McArthur |
| `async-nats` | `0.46` | `nats_adapter` | NATS client for messaging and JetStream/KV | Apache-2.0 | NATS.rs maintainers |
| `prost`, `prost-types`, `prost-build` | `0.13` | `api`, `nats_adapter`, services | protobuf message generation and runtime types | Apache-2.0 | Tokio / prost |
| `ibapi` | `2.10` | `ib_adapter`, `tws_yield_curve`, `backend_service` | Rust IBKR/TWS API client wrapper | MIT | `rust-ibapi` / Wil Boayue |
| `ratatui`, `crossterm`, `tui-logger` | `0.30`, `0.28`, `0.18` | `tui_service` | terminal UI, input, in-TUI log viewer | MIT | Ratatui, Crossterm, tui-logger projects |
| `sqlx`, `rusqlite` | `0.9.0-alpha.1`, `0.37` | `api`, `ledger`, `market_data` | SQLite storage and queries | MIT OR Apache-2.0 / MIT | Launchbadge SQLx / rusqlite maintainers |
| `rust_decimal` | `1.36` | `ledger`, `broker_engine`, `discount_bank_parser`, `cli` | decimal-safe finance amounts | MIT | `rust-decimal` / Paul Mason |
| `yfinance-rs` | `0.7.2` | `market_data` | Yahoo Finance integration path | MIT | `yfinance-rs` upstream |

## Direct Rust Dependency Inventory

| Crate | Declared version | Scope | Used by | Purpose | License | Credit / upstream |
|---|---:|---|---|---|---|---|
| `anyhow` | `1` | runtime | most crates/services | ergonomic error propagation | MIT OR Apache-2.0 | David Tolnay |
| `async-trait` | `0.1` | runtime | broker, market-data, risk, api, backend | async trait support | MIT OR Apache-2.0 | David Tolnay |
| `async-nats` | `0.46` | runtime | `nats_adapter` | NATS connectivity | Apache-2.0 | NATS.rs |
| `axum` | `0.7` | runtime | `backend_service` | HTTP routing and handlers | MIT | Tokio / axum |
| `backoff` | `0.4` | runtime | `nats_adapter`, `tui_service`, `tws_yield_curve_daemon` | retry/backoff policy | MIT OR Apache-2.0 | Tibor Benke / `backoff` |
| `base64` | `0.22` | runtime | `api` | encoding helpers | MIT OR Apache-2.0 | `rust-base64` |
| `bytes` | `1.5` | runtime | `nats_adapter`, `backend_service`, `tws_yield_curve_daemon` | protocol buffer and messaging buffers | MIT | Tokio / bytes |
| `chrono` | `0.4` | runtime | most crates/services | timestamps, dates, finance time fields | MIT OR Apache-2.0 | Chronotope / chrono |
| `clap` | `4` | runtime | `cli` | command-line parsing | MIT OR Apache-2.0 | clap-rs |
| `clap_complete` | `4` | runtime | `cli` | shell completion generation | MIT OR Apache-2.0 | clap-rs |
| `color-eyre` | `0.6` | runtime | `tui_service` | panic and error report formatting | MIT OR Apache-2.0 | eyre-rs |
| `crossterm` | `0.28` | runtime | `tui_service` | cross-platform terminal input/output | MIT | Crossterm |
| `derive_builder` | `0.20` | runtime | `common`, `broker_engine`, `market_data`, `quant` | builder pattern for shared structs | MIT OR Apache-2.0 | rust-derive-builder |
| `dirs` | `5` | runtime | `api` | host config/keyring path discovery | MIT OR Apache-2.0 | dirs-rs |
| `encoding_rs` | `0.8` | runtime | `discount_bank_parser` | bank file text decoding | `(Apache-2.0 OR MIT) AND BSD-3-Clause` | Henri Sivonen / `encoding_rs` |
| `financial_symbols` | `1.0` | runtime | `broker_engine` | symbol normalization / parsing | MIT | `financial_symbols` |
| `futures` | `0.3` | runtime | market data, backend, tui, adapters | stream/future utilities | MIT OR Apache-2.0 | futures-rs |
| `http-body-util` | `0.1` | dev/test | `api` dev-deps | HTTP body test helpers | MIT | hyperium |
| `ibapi` | `2.10` | runtime | `ib_adapter`, `tws_yield_curve`, `backend_service` | Interactive Brokers API wrapper | MIT | `rust-ibapi` |
| `jsonc-parser` | `0.29` | runtime | `api`, `tui_service` | JSONC config parsing | MIT | dprint / `jsonc-parser` |
| `keyring` | `3` | runtime feature | `api` with `keyring` feature | OS keychain integration | MIT OR Apache-2.0 | keyring-rs |
| `log` | `0.4` | runtime | `tui_service` | compatibility logging facade for `tui-logger` | MIT OR Apache-2.0 | Rust log project |
| `optionstratlib` | `0.15` | dev/test | `quant` dev-deps | options-strategy validation/testing | MIT | OptionStratLib |
| `parking_lot` | `0.12` | runtime | `quant` | synchronization primitives | MIT OR Apache-2.0 | `parking_lot` |
| `pdf-extract` | `0.10` | runtime | `cli` | PDF text extraction for bank/import flows | MIT | `pdf-extract` |
| `prost` | `0.13` | runtime | `api`, `nats_adapter`, `backend_service` | protobuf runtime | Apache-2.0 | Tokio / prost |
| `prost-build` | `0.13` | build | `nats_adapter` build-deps | protobuf code generation | Apache-2.0 | Tokio / prost |
| `prost-types` | `0.13` | runtime | `api`, `nats_adapter`, services, tui | protobuf well-known types | Apache-2.0 | Tokio / prost |
| `rand` | `0.8` | runtime | workspace, market data, backend, quant | randomization / sampling | MIT OR Apache-2.0 | rand project |
| `rand_chacha` | `0.3` | dev/test | `quant` dev-deps | deterministic PRNG for tests | MIT OR Apache-2.0 | rand project |
| `ratatui` | `0.30` | runtime | `tui_service` | terminal dashboard rendering | MIT | Ratatui developers |
| `regex` | `1` | runtime | `market_data`, `cli` | parsing and input matching | MIT OR Apache-2.0 | Rust regex project |
| `reqwest` | `0.13` | runtime | `api`, `market_data`, `backend_service`, `tui_service`, `cli` | HTTP client | MIT OR Apache-2.0 | reqwest |
| `rusqlite` | `0.37` | runtime | `market_data` | SQLite access with bundled SQLite | MIT | rusqlite |
| `rust_decimal` | `1.36` | runtime | ledger, parser, broker, cli | fixed-point decimal arithmetic | MIT | `rust-decimal` |
| `serde` | `1` | runtime | most crates/services | serialization core | MIT OR Apache-2.0 | Serde |
| `serde_json` | `1` | runtime | most crates/services | JSON encoding/decoding | MIT OR Apache-2.0 | Serde JSON |
| `serde_urlencoded` | `0.7` | runtime | `api` | URL form/query encoding | MIT OR Apache-2.0 | `serde_urlencoded` |
| `sqlx` | `0.9.0-alpha.1` | runtime | `api`, `ledger` | async SQLite access | MIT OR Apache-2.0 | Launchbadge / SQLx |
| `tempfile` | `3.11` | dev/test | `ledger` | temp DB/files in tests | MIT OR Apache-2.0 | `tempfile` |
| `thiserror` | `1` | runtime | multiple crates | typed error definitions | MIT OR Apache-2.0 | David Tolnay |
| `time` | `0.3` | runtime | `quant`, `risk` | additional time utilities | MIT OR Apache-2.0 | time-rs |
| `tokio` | `1` | runtime | most crates/services | async runtime | MIT | Tokio |
| `tokio-stream` | `0.1` | runtime | `market_data` | async stream adapters | MIT | Tokio |
| `tokio-test` | `0.4` | dev/test | `ledger`, `nats_adapter` | async test helpers | MIT | Tokio |
| `tokio-tungstenite` | `0.26` | runtime | `market_data` | websocket client support | MIT | snapview |
| `toml` | `0.8` | runtime | `backend_service`, `cli` | TOML config parsing | MIT OR Apache-2.0 | toml-rs |
| `tracing` | `0.1` | runtime | most crates/services | instrumentation/log events | MIT | tracing |
| `tracing-subscriber` | `0.3` | runtime | parser, yield daemon, backend, tui, cli | log formatting/filter config | MIT | tracing |
| `trading-calendar` | `0.2` | runtime | `tui_service` | NYSE market-hours status | MIT OR Apache-2.0 | trading-calendar |
| `tui-logger` | `0.18` | runtime | `tui_service` | in-terminal log viewer | MIT | tui-logger |
| `url` | `2` | runtime | `market_data` | URL handling | MIT OR Apache-2.0 | rust-url |
| `uuid` | `1.10` | runtime | `api`, `ledger`, `backend_service`, `tui_service` | IDs and correlation keys | Apache-2.0 OR MIT | uuid-rs |
| `wiremock` | `0.5` | dev/test | `market_data` dev-deps | HTTP mocking for tests | MIT OR Apache-2.0 | wiremock-rs |
| `yfinance-rs` | `0.7.2` | runtime | `market_data` | Yahoo Finance adapter | MIT | yfinance-rs |

## Workspace-Declared But Currently Unused

These crates appear in `agents/backend/Cargo.toml` workspace dependencies but
are not currently referenced by member crate manifests:

| Crate | Declared version | Current status | License | Credit / upstream |
|---|---:|---|---|---|
| `futures-util` | `0.3` | declared in workspace, not referenced by members | MIT OR Apache-2.0 | futures-rs |
| `nautilus-model` | `0.55` | declared in workspace, not referenced by members | UNKNOWN in this local summary | NautilusTrader / nautilus-model |
| `openssl` | `0.10` | declared in workspace, not referenced by members | Apache-2.0 | rust-openssl |
| `tower` | `0.4` | declared in workspace, not referenced by members | MIT | tower-rs |

## External Vendor Components Outside Cargo

| Component | Status | Purpose | License / terms | Credit / upstream |
|---|---|---|---|---|
| Interactive Brokers TWS / IB Gateway | active runtime dependency for IBKR flows | broker connectivity and market/order access | proprietary vendor software; follow IBKR terms | Interactive Brokers |
| Interactive Brokers API contract / protocol surface | active operational dependency via `ibapi` crate | underlying broker API semantics used by `ib_adapter` and `tws_yield_curve` | proprietary vendor API terms | Interactive Brokers |
| `../tws-api/` sibling repo or extracted TWS API tree | documented as optional/legacy source artifact, not part of the active Rust-first build | historical/interop reference and vendor source tree handling | vendor terms not audited here | Interactive Brokers |

## Credit And Attribution Notes

- This repo does **not** currently maintain a full transitive dependency
  attribution manifest.
- The most important direct licenses in the active Rust stack are permissive:
  MIT, Apache-2.0, or dual MIT/Apache-2.0.
- Some components have special handling:
  - `encoding_rs` includes a BSD-3-Clause component in its published license expression.
  - IBKR software/API components are vendor-proprietary and should be treated
    under vendor terms rather than Cargo open-source license conventions.
- For release/legal review, supplement this direct-dependency summary with:
  - a transitive inventory from `cargo metadata` or a dedicated license tool
  - review of any vendored non-Cargo artifacts actually shipped with builds

