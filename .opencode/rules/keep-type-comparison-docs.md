---
description: Keep BACKEND_TYPE_COMPARISON.md current when adding types, traits, or adapters
alwaysApply: false
---

# Keep Type Comparison Docs Current

**Rule:** After adding a new type, trait method, adapter, or conversion, update `docs/BACKEND_TYPE_COMPARISON.md` in the same commit. The comparative table is the primary tool for understanding cross-backend architecture drift.

## When to Update

| Change | Required update |
|--------|---------------|
| New `BrokerEngine` trait method | Table 5 — add row, mark ✅/❌ per adapter |
| New adapter impl | Tables 5, 6, 7 — add adapter column |
| New `*Position*` type | Table 1 — add row, note conversions |
| New `*Order*` type | Table 2 — add row |
| New proto message | Table 8 — add row |
| New NATS topic | Table 8 — add to topic hierarchy |
| Missing `From` conversion | Tables 1, 4 — mark ❌ |
| New option chain resolution path | Table 6 — add row |

## Quick Check

Before committing, verify the table reflects reality:

```bash
# Verify position types are consistent
rg "pub struct.*Position" agents/backend/crates --type rust

# Verify BrokerEngine methods match trait
rg "async fn " agents/backend/crates/broker_engine/src/traits.rs

# Check for missing From conversions
rg "From<.*> for.*Position|From<.*Position> for" agents/backend/crates --type rust
```
