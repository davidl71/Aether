# Minimal TUI flow for VHS (scripted demo / golden output).
# Run from repo root with NATS and backend_service running:
#   vhs docs/tui_qa/tui_minimal.tape
# Build TUI first: just build  or  ./scripts/run_rust_tui.sh

Output docs/tui_qa/tui_minimal.ascii
Set FontSize 14
Set Width 800
Set Height 400

Execute ./agents/backend/target/debug/tui_service
Sleep 4s
Type "2"
Sleep 1s
Type "q"
Sleep 1s
