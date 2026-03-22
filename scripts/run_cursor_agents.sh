#!/bin/bash
# Run Cursor agent batches in parallel for Aether
# Usage: ./run_cursor_agents.sh [batch]
#   batch: a, b, c, or all (default)

PROJECT="/Users/davidl/Projects/Trading/Aether"
LOGDIR="/tmp/cursor_batch_logs"
mkdir -p "$LOGDIR"

CURSOR_PROMPT_A="You are fixing TUI bugs in $PROJECT.

Fix these tasks (use exarp-go task update to mark Done when complete, add comment with findings):

1. T-1773939588206136000 [low] Fix per-row to_uppercase allocation in dashboard.rs — the to_uppercase() call allocates on every render row; cache or avoid it
2. T-1773939583046251000 [low] Remove or wire up dead event variants in events.rs — only AppEvent::Connection is used, other variants have #![allow(dead_code)]
3. T-1773939585598180000 [low] Fix hint bar tab count inconsistency — ui.rs says 1-5 but keys 1-9 are bound
4. T-1773939580016880000 [low] Implement or stub positions_display_info params — combo_view and expanded params are unused

After each fix verify: cd $PROJECT/agents/backend && cargo check -p tui_service

Mark each task Done in exarp when complete."

CURSOR_PROMPT_B="You are evaluating Rust trading frameworks for $PROJECT.

Evaluate these tasks (mark Done in exarp with comment when complete):

1. T-1773941258705443000 [medium] Evaluate OptionStratLib for options pricing and Greeks — check lib.rs/options_pricing and docs/RESEARCH_RUST_TRADING_FRAMEWORKS.md
2. T-1773940296400695000 [medium] Evaluate RustQuant for box spread pricing validation — compare with quant crate at agents/backend/crates/quant/src/lib.rs
3. T-1773940292514030000 [medium] Evaluate matchcore for paper trading simulation — check docs/RESEARCH_RUST_TRADING_FRAMEWORKS.md
4. T-1773940287635672000 [medium] Evaluate yatws migration vs fixing ib_adapter — read ib_adapter/src/types.rs construct_box_spread_order and compare with yatws OptionsStrategyBuilder
5. T-1773941020422614000 [medium] Implement IBKR conId resolution workflow for combo orders — read IBKR workflow in docs/RESEARCH_RUST_TRADING_FRAMEWORKS.md and ib_adapter types
6. T-1773941016184003000 [medium] Handle TickAttrib quote quality flags in market data — read tws_yield_curve and market_data for TickAttrib usage
7. T-1773941370747334000 [medium] Deduplicate and consolidate framework research tasks — review RESEARCH_RUST_TRADING_FRAMEWORKS.md for duplicate entries

Read the codebase and check relevant docs before making any changes. Mark each task Done in exarp when complete."

CURSOR_PROMPT_C="You are doing architecture analysis for $PROJECT.

Do these tasks (mark Done in exarp with comment when complete):

1. T-1773939573972364000 [high] Analyze ScenarioDto / BoxSpreadScenario dataflow — read agents/backend/crates/api/src/runtime_state.rs, agents/backend/crates/api/src/combo_strategy.rs, proto/messages.proto for BoxSpreadScenario
2. T-1773939577056591000 [medium] Resolve ui.rs vs ui/mod.rs dual-rendering dead code — read services/tui_service/src/ui.rs and services/tui_service/src/ui/mod.rs, determine which is live
3. T-1773940123381922000 [medium] Evaluate yatws as ib_adapter replacement — check ibkrcampus.com/docs for yatws comparison
4. T-1773941611822304000 [medium] Evaluate Longbridge Terminal as ratatui TUI reference — read longbridge-terminal/src/ for ratatui patterns

Read the codebase and check relevant docs before making any changes. Mark each task Done in exarp when complete."

run_batch() {
  local id="$1"
  local prompt="$2"
  local name="$3"
  echo "Starting Batch $id ($name)..."
  cursor agent --workspace "$PROJECT" -p --force "$prompt" >"$LOGDIR/batch_$id.log" 2>&1 &
  echo "  PID: $!  Log: $LOGDIR/batch_$id.log"
}

case "${1:-all}" in
a | A)
  run_batch "a" "$CURSOR_PROMPT_A" "TUI fixes"
  ;;
b | B)
  run_batch "b" "$CURSOR_PROMPT_B" "Framework evaluation"
  ;;
c | C)
  run_batch "c" "$CURSOR_PROMPT_C" "Architecture analysis"
  ;;
all | ALL | "")
  run_batch "a" "$CURSOR_PROMPT_A" "TUI fixes"
  run_batch "b" "$CURSOR_PROMPT_B" "Framework evaluation"
  run_batch "c" "$CURSOR_PROMPT_C" "Architecture analysis"
  ;;
*)
  echo "Usage: $0 [a|b|c|all]"
  exit 1
  ;;
esac

echo ""
echo "Running in background. Logs:"
echo "  $LOGDIR/batch_a.log  (TUI fixes)"
echo "  $LOGDIR/batch_b.log  (Framework evaluation)"
echo "  $LOGDIR/batch_c.log  (Architecture analysis)"
echo ""
echo "Watch: tail -f $LOGDIR/batch_{a,b,c}.log"
echo "Kill:  pkill -f 'cursor agent'"
