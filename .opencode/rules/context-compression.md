---
description: Proactive context compression at workflow milestones for OpenCode agent
alwaysApply: false
---

# Context Compression at Milestones

**Rule:** Aggressively compress context at workflow milestones to prevent context limit exhaustion. Favor short, closed, summary-safe compressions over accumulation.

## When to Compress

**Mandatory compression points:**
- ✅ **Task complete** — Immediately after marking task Done
- ✅ **Research complete** — Findings synthesized, no more raw exploration needed
- ✅ **Build/verification done** — cargo build/test/lint finished with results
- ✅ **Documentation written** — New/updated docs captured
- ✅ **Multi-file refactor** — Changes applied across >3 files
- ✅ **Context warning** — When >75% of context window used

**Emergency compression:**
- When `MAX CONTEXT LIMIT REACHED` warning appears → **STOP and compress immediately**
- When response latency degrades noticeably
- When you need to reference old exploration but current context is full

## Compression Workflow

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ Before:         │     │ Compress:        │     │ After:          │
│ 30 raw messages │ ──► │ Atomic summary   │ ──► │ 1 summary block │
│ about research  │     │ with file paths, │     │ + clean slate   │
│                 │     │ decisions, etc   │     │ for next step   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

## How to Compress

```typescript
// Example: After completing Alpaca integration
compress({
  topic: "Complete Alpaca Integration Implementation",
  content: [{
    startId: "m0100",  // Start of work
    endId: "m0180",    // End of work
    summary: `
      Implemented T-1774481655277034: Revive Alpaca position and market data sources.
      
      Files modified:
      - agents/backend/services/tui_service/src/alpaca_health.rs
      - agents/backend/crates/api/src/alpaca_positions.rs
      - agents/backend/services/tui_service/src/events.rs
      - agents/backend/services/tui_service/src/app_updates.rs
      - agents/backend/services/tui_service/src/main.rs
      - agents/backend/crates/market_data/src/alpaca.rs
      - agents/backend/bin/cli/src/main.rs
      
      Documentation updated:
      - docs/MARKET_DATA_INTEGRATION.md
      - docs/ADDING_DATA_SOURCE_TO_TUI.md (new)
      
      Key decisions:
      - source_priority: 55 for paper, 75 for live
      - Environment vars bridge credential mgmt → data sources
      - round-robin symbol polling with spawn_blocking
      
      Verified: Build succeeds, credentials match Alpaca API spec.
    `
  }]
})
```

## Summary Requirements

**Must include:**
- File paths modified (full paths)
- Function signatures changed
- Key decisions made
- Verification results (build/test status)
- Block references `(bN)` if compressing previously compressed blocks

**Must NOT include:**
- Conversational filler
- Failed attempts that led nowhere
- Tool output noise
- Verbose explanations of obvious code

## Integration with exarp-go

- **Task complete** → `exarp_update_task(task_id, "Done")` → **compress**
- **Follow-up suggested** → `exarp_followup(action="suggest")` → note suggestions → **compress**
- **Multi-task session** → compress between tasks, not at the very end

## Anti-Patterns

- ❌ Waiting until context is 100% full
- ❌ Keeping raw exploration when findings are clear
- ❌ Compressing active work (still being edited)
- ❌ Creating vague summaries ("did some work on X")
- ❌ Compressing only old context while ignoring recent completed work

## Emergency Protocol: MAX CONTEXT LIMIT

If you see `MAX CONTEXT LIMIT REACHED`:

1. **STOP all new work immediately**
2. Identify the **largest closed range** of completed work
3. **One large compression** covering that range
4. Include **all essential details** — this summary becomes authoritative
5. Resume work with clean context

**Never** continue with normal operations during context emergency.

## Remember

Context is a finite resource. Compression is **crystallization** — distilling raw exploration into actionable understanding. The summary becomes more valuable than the original messages because it captures what matters for downstream work.
