# AI Workflow

**Status:** Active repo workflow guidance

## Default Prompt Shape

For non-trivial work, use this structure:

1. `Goal`
2. `Context`
3. `Constraints`
4. `Done when`
5. `Verification`

Example:

```text
Goal: Make yield-curve sources explicit in the TUI.
Context: Product is in read-only exploration mode.
Constraints: Do not add execution code; prefer existing Rust crates.
Done when: The TUI labels source and stale state clearly.
Verification: cargo check -p tui_service
```

This keeps tasks concrete and makes backlog/result comments easier to trust.

## Thread And Scope Hygiene

- Prefer one thread per coherent work cluster such as `yield validity`,
  `provider audit`, or `docs cleanup`
- Use compact summaries between long implementation phases rather than carrying
  a single broad thread forever
- Separate planning from mutation when product direction changes

## Backlog Hygiene

After any non-trivial implementation pass:

- close tasks that are fully satisfied
- narrow tasks that are only partially satisfied
- cancel tasks made redundant by architecture or product-direction changes
- add follow-up tasks only for real residual work

## Aether-Specific Defaults

- Assume **read-only exploration mode** unless told otherwise
- Prioritize market-data truth and yield validity over execution plumbing
- Keep real positions visible, but do not restore execution paths casually
- Minimize new external integrations unless they improve data validity
- Prefer clean replacements over migration-heavy or compatibility-heavy plans
  unless a concrete constraint requires the old surface to survive

## exarp Usage In Aether

Use exarp for:

- tracking non-trivial implementation/refactor work
- recording verification and follow-up tasks
- keeping backlog truth aligned with current product direction

No exarp architectural change is required for this workflow shift. The needed
change is **how Aether tasks are framed and maintained**.
