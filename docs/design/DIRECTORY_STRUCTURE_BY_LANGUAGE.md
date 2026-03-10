# Directory Structure: By Component vs By Language

## Language–component table

| Language | Location | Components |
|----------|----------|------------|
| **C++** | `native/` | Core engine: CLI (`ib_box_spread`), TWS client/connection/contracts/orders/positions/market_data, box-spread strategy, risk/greeks/convexity/margin calculators, order_manager, hedge_manager, loan_manager, option_chain, config_manager, rate_limiter, NATS client, proto adapter, cache clients, broker adapters (TWS, Alpaca, IB Client Portal). Build: CMake. |
| **Rust** | `agents/backend/` | Backend service: REST/WebSocket API (Axum), ledger (accounts, transactions, posting), NATS adapter, market_data pipeline, strategy/risk crates, discount_bank_parser. Build: Cargo. |
| **Python** | `python/` | TUI (Textual), integration services (IB, Alpaca, Tastytrade, TradeStation, discount_bank, risk_free_rate), clients (alpaca/tastytrade/tradestation/sofr_treasury/onepassword_sdk_helper), shared_config_loader, Nautilus/LEAN/Swiftness integration, bindings, tests. |
| **TypeScript/React** | `web/` | PWA: React app, snapshot/API/config/hooks, service ports, charts. Build: Vite/npm. |
| **JavaScript (Node)** | `services/israeli-bank-scrapers-service/` | Israeli bank scrapers HTTP service and CLI; ledger writer; optional 1Password SDK. Build: npm. |
| **Swift** | `ios/`, `desktop/` | iOS/iPad (SwiftUI), macOS desktop (AppKit). |
| **Go** | `agents/go/` | Go-based agents/tools (see `go.mod`). |
| **Shell** | `scripts/`, `web/scripts/`, `ib-gateway/` | Build, lint, service start/stop, 1Password CLI wrapper, gateway runner. |
| **Proto/JSON/YAML** | `proto/`, `config/` | Shared message definitions; example configs. |

---

## Language rationale (no suggested broad changes)

- **C++**: TWS API is C++-only; core pricing/risk use QuantLib and decimal math. Rewriting the core in Rust would require redoing all IB integration and duplicating pricing/risk work — not recommended.
- **Rust**: Fits the long-lived backend (REST, ledger, NATS). No benefit to rewriting in C++.
- **Python**: Fits integration and TUI; broker SDKs and iteration speed are an advantage. Rewriting in Rust/Go would be a large effort and lose ecosystem.
- **TypeScript / Node**: Standard for React PWA; Node is confined to the Israeli bank scrapers service. No need to change.

---

## Where Go fits — and what to rewrite in Go

**Current use:** `agents/go/` holds Go-based agents/tools (see `go.mod`). Go is a good fit for small, single-binary CLI/ops tools and for NATS-heavy or high-concurrency glue services.

**Good candidates to rewrite in Go** (only if you want a single binary and no Python/Node runtime):

| Current component | Language | Why consider Go | Recommendation |
|-------------------|----------|-----------------|----------------|
| Config validator / schema checker | Shell or Python script | Becomes one binary; easy to ship in CI or containers | **Yes** — if the script grows (e.g. `scripts/validate_api_contract.sh` or a shared-config validator). |
| Small NATS bridge or fan-out daemon | New or currently shell | Goroutines + official NATS client; single binary | **Yes** — for *new* small services that are mostly NATS + light logic. |
| Service manager / process supervisor | Shell (`scripts/service_manager.sh`, etc.) | Could be one binary with better control and logging | **Optional** — only if shell becomes hard to maintain; otherwise keep shell. |
| QuestDB/NATS writer | Go (nats-questdb-bridge) | Single binary; run via scripts/run_questdb_nats_writer.sh. Requires Go. | **Optional** — if deployment favours “drop a binary” over Python. |
| Exarp / MCP / automation CLI | External (exarp-go) | Already Go elsewhere | **No** — leave as external tool. |

**Do *not* rewrite in Go:**

| Component | Reason |
|-----------|--------|
| Python broker services (IB, Alpaca, Tastytrade, TradeStation, discount_bank, risk_free_rate) | Broker SDKs and ecosystem are in Python; rewrite would be large and lose libraries. |
| Rust backend (`agents/backend/`) | Already correct fit; no gain from porting. |
| Israeli bank scrapers service (Node) | Depends on `israeli-bank-scrapers` (Node); reimplementing in Go would mean new scraping logic or subprocess to Node. |
| C++ core (`native/`) | TWS and pricing/risk stay in C++. |
| Web PWA / TUI | Keep React and Python/Textual. |

**Summary:** Use Go for **new or growing CLI/ops tools** and **small NATS-oriented services** where a single binary helps. Do not rewrite existing Python broker services, the Rust backend, or the Node scrapers service into Go.

---

## Current layout (by component / product)

Top-level directories are **component- or product-oriented**. Language is implicit per directory:

| Directory | Primary language | Purpose |
|-----------|------------------|---------|
| `native/` | C++ | Core engine: pricing, risk, order management, TWS client |
| `python/` | Python | Integration: TUI, bindings, NautilusTrader, tests |
| `agents/` | Mixed | Services and apps: Rust backend, Go tools, shared |
| `agents/backend/` | Rust | Backend services (API, ledger, market data) |
| `agents/go/` | Go | Go-based agents/tools |
| `agents/web/` | (varies) | Web-related agent code |
| `web/` | TypeScript (React) | Web frontend |
| `ios/` | Swift (SwiftUI) | iOS/iPad app |
| `desktop/` | Swift (AppKit) | macOS desktop app |
| `proto/` | Protocol Buffers | Shared message definitions (language-agnostic) |
| `config/` | JSON/YAML | Example configs (language-agnostic) |
| `scripts/` | Shell, Python | Build, lint, deploy, git hooks |
| `docs/` | Markdown | Documentation |

So: **one main "home" per product** (native core, python layer, agents, web, ios, desktop), with shared `proto/`, `config/`, `scripts/`, `docs/`.

---

## Alternative: refactor by language

A **language-first** layout would group by language at the top level, then by product inside:

```
langs/
├── cpp/           # current native/ (rename/move)
├── python/        # current python/
├── rust/          # agents/backend (+ any other Rust)
├── go/             # agents/go
├── ts/             # web/ + agents/web (and any other TS)
├── swift/          # ios/ + desktop/
└── proto/         # shared (or under shared/)
```

Or flatter:

```
cpp/       ← native
python/
rust/      ← agents/backend
go/        ← agents/go
ts/        ← web (+ agents/web)
swift/     ← ios, desktop
proto/
config/
scripts/
docs/
```

---

## Recommendation: keep current structure

- **Discoverability**: "Where is the backend?" → `agents/backend/`. "Where is the web app?" → `web/`. By-language top level answers "where is Rust?" but not "where is the backend?" without extra convention.
- **Shared and cross-cutting**: `proto/`, `config/`, `scripts/`, `docs/` are shared across languages; they sit naturally at top level. A `langs/` tree would either duplicate them or add a separate `shared/` and more indirection.
- **Tooling and CI**: Builds are per product (e.g. `native/` CMake, `agents/backend/` Cargo, `web/` npm). Keeping one directory per "thing to build" keeps scripts and CI simple.
- **Convention**: Many multi-language repos use component-first layout (e.g. `services/`, `apps/web/`, `apps/ios/`) with language obvious from files inside; our layout matches that.

If you want **visibility by language** without moving code:

- **Document the mapping** (e.g. this file, or a short "Languages" section in `AGENTS.md` / `ARCHITECTURE.md`).
- **Optional**: Add a `docs/indices/LANGUAGES.md` (or similar) that lists each language and its directories and entrypoints.

---

## If you do refactor by language

- Move `native/` → `cpp/` (or `langs/cpp/`) and update all references: CMake, scripts, docs, `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, CI.
- Consolidate Rust under one tree (e.g. `rust/backend/`, `rust/other/` if more appear); same for Go, TypeScript, Swift.
- Keep `proto/`, `config/`, `scripts/`, `docs/` at repo root.
- Run full build/test/lint and update `README.md`, `ARCHITECTURE.md`, and any "project layout" or "directory structure" sections in docs.

---

## Summary

| Approach | Pros | Cons |
|----------|------|------|
| **Current (by component)** | Clear product boundaries; simple paths for shared assets; CI/build per component | Language not obvious from top-level names |
| **By language** | Easy to answer "where is all Rust/Go/TS?" | Weaker product boundaries; more nesting or duplication for shared dirs; CI/scripts need updates |

Recommendation: **keep the current directory structure**, and add a short "Languages ↔ directories" overview (e.g. in `AGENTS.md` or `ARCHITECTURE.md`) so both "by component" and "by language" are easy to infer.
