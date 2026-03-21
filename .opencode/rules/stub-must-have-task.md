---
description: Any function returning Vec::new(), None, todo!(), or with STUB comment must have an exarp-go task
alwaysApply: false
---

# Stub Must Have Task

**Rule:** Any function that is a stub (`Vec::new()`, `None`, `todo!()`, `unimplemented!()`, or contains `// STUB` / `// TODO` in the body) **must** have an exarp-go task tracking it. Untracked stubs are subject to immediate implementation or removal.

## Rationale

Stubs accumulate silently. A function returning `Vec::new()` looks valid in code review but is dead code that misleads future readers. The `positions_display_info` stub in `ui/positions.rs` had `// STUB: not implemented` but no task — it sat for months before being discovered.

## How to Identify Stubs

Look for these patterns:

```rust
// ❌ Returns empty — likely a stub
fn positions_display_info(...) -> Vec<PositionDisplayInfo> {
    Vec::new()
}

// ❌ Always None — likely unimplemented
fn get_strategy(&self) -> Option<Strategy> {
    None
}

// ❌ Contains STUB or TODO marker
// STUB: not implemented
async fn fetch_chain(...) { ... }

// ❌ todo!() or unimplemented!()
fn resolve_conid(...) -> Result<i64, BrokerError> {
    todo!()
}
```

## What to Do

| If you find a stub | Action |
|---|---|
| Has a task | Check the task is accurate; implement or update |
| No task | Create an exarp-go task before leaving the code |
| Stub is wrong approach | Remove it and implement the correct behavior |

## Pattern for Stubs in New Code

When adding a new function that can't be fully implemented yet:

```rust
// TODO(T-XXXXXXXX): implement full resolution
// Tracked at: https://exarp.io/tasks/T-XXXXXXXX
pub async fn resolve_conid(...) -> Result<i64, BrokerError> {
    todo!("tracked in T-XXXXXXXX")
}
```

## Enforcement

During code review or session cleanup, grep for stubs:

```bash
# Find likely stubs (returns empty Vec or None)
rg "fn.+\).+(Vec<|.+) \{[\s\n]*Vec::new\(\)" agents/backend/crates --type rust
rg "fn.+\).+Option<.+> \{[\s\n]*None" agents/backend/crates --type rust

# Find STUB/TODO comments without task references
rg "// STUB|// TODO.*not implemented" agents/backend --type rust
```
