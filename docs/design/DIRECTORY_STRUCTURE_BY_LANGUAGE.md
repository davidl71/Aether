# Directory Structure: By Component vs By Language

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
| `agents/tui/` | (varies) | TUI-related agent code |
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
