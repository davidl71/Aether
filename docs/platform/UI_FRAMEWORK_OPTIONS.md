# Future UI Framework Options

**Last updated**: 2026-03-11
**Purpose**: Document framework options for any future unified UI, without changing current runtime.

## Current Active Runtime

- **Terminal**: Rust Ratatui TUI (`agents/backend/services/tui_service`)
- **CLI**: Native C++ CLI (`native/src/ib_box_spread.cpp`)

## Historical Frameworks Considered

| Framework | Language | Status | Notes |
|-----------|----------|--------|-------|
| FTXUI | C++ | Retired | Original native TUI; replaced by Rust |
| Textual | Python | Retired | Python TUI era; replaced by Rust |
| React/Vite | TypeScript | Retired | Web frontend; archived |

See `docs/archive/TUI_ALTERNATIVES_ANALYSIS.md` for detailed historical comparison.

## Decision Factors for Future Framework Choice

When evaluating a new UI framework (whether terminal or web), weigh these factors:

### Must-Have

- **Protobuf support**: Native or easy integration with `proto/messages.proto`
- **Minimal dependencies**: Avoid pulling in heavy runtime stacks
- **Active maintenance**: Recent commits, responsive issues, healthy community

### Should-Have

- **Async-friendly**: Fits the Rust backend's async model (for terminal) or React Query/hooks pattern (for web)
- **Testability**: Built-in test utilities or clear patterns for UI testing
- **Accessibility**: Screen reader support, keyboard navigation

### Nice-to-Have

- **Hot reload**: Faster development iteration
- **Rich formatting**: Tables, progress bars, color schemes for terminal; rich components for web
- **Mobile-friendly**: Responsive or adaptive layouts for web options

## Framework Candidates

### Terminal Options

| Framework | Language | Protobuf | Async | Notes |
|-----------|----------|----------|-------|-------|
| **Ratatui** | Rust | ✅ Native | ✅ Native | Current choice; integrates with Rust backend |
| Bubbletea | Go | Via grpcui or manual | ✅ Native | Good for Go-centric teams |
| Textual | Python | ✅ Generated | ✅ Async | Best Python DX; requires Python runtime |
| FTXUI | C++ | Manual | Manual | Best C++ perf; highest maintenance cost |

### Web Options (if revived)

| Framework | Language | Protobuf | State Management | Notes |
|-----------|----------|----------|------------------|-------|
| **React + Vite** | TypeScript | Via ts-proto | React Query | Current archived choice |
| Next.js | TypeScript | Via ts-proto | Server + Client | If SSR/sEO matters later |
| Solid | TypeScript | Via protobuf-ts | Signals | Lighter React alternative |
| Svelte | TypeScript | Via protobuf-ts | Stores | If simplicity outweighs ecosystem |

## When to Re-Evaluate

Consider re-evaluating the framework decision when:

1. **Major backend shift**: If Rust backend is replaced by another language
2. **Team composition change**: If Go or Python expertise outweighs Rust for UI work
3. **Feature gap**: Current framework lacks a must-have capability (e.g., mobile, rich charts)
4. **Performance ceiling**: Active framework cannot meet latency or bundle-size requirements

## Related Documentation

- `docs/archive/TUI_ALTERNATIVES_ANALYSIS.md` — Historical framework comparisons
- `docs/archive/TUI_PYTHON_MIGRATION.md` — Python Textual migration notes
- `docs/platform/DATAFLOW_ARCHITECTURE.md` — Current dataflow and read paths
- `agents/backend/services/tui_service/` — Current Rust TUI implementation
