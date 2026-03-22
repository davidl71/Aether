# TUI QA Recording Runbook (asciinema / ttyrec / VHS)

**Purpose:** Record and replay Rust TUI sessions for QA, demos, and optional scripted integration tests. Requires **backend_service** and **tui_service** with NATS (see [NATS_VERIFICATION_CHECKLIST.md](NATS_VERIFICATION_CHECKLIST.md)).

**Reference:** [docs/CLI_TUI_TOOLS_RECOMMENDATIONS.md](../CLI_TUI_TOOLS_RECOMMENDATIONS.md) — asciinema (demos), VHS (scripted tests), ttyrec (lightweight replay).  
**See also:** [INTERACTIVE_QA_DRIVER.md](../INTERACTIVE_QA_DRIVER.md) — PTY/expect-style automated QA (pexpect, VHS golden output).

---

## Prerequisites

1. **NATS** running (e.g. `nats-server -js -DV` or `just services-active-start`).
2. **backend_service** running and connected to NATS (publishes `snapshot.{backend_id}`).
3. **TUI binary** built: `just build` or `./scripts/run_rust_tui.sh` (builds if needed).
4. **Recording tools** (at least one):
   - **asciinema** — human-friendly `.cast`; replay in terminal or upload to asciinema.org.
   - **ttyrec** — binary `.ttyrec`; replay with `ttyplay`.
   - **VHS** — scripted `.tape` → reproducible runs and optional GIF/ASCII output for CI.

### Install recording tools (macOS)

```bash
brew install asciinema vhs ttyrec
```

---

## 1. Start backend and NATS

Ensure NATS and one backend are up before recording the TUI.

```bash
# From repo root
just services-active-start
# Or manually:
# Terminal 1: nats-server -js -DV
# Terminal 2: cd agents/backend && cargo run -p backend_service
```

Optional: set `NATS_URL`, `BACKEND_ID` if not using defaults (`nats://localhost:4222`, `ib`).

---

## 2. Record a TUI session

### Option A: Script wrapper (recommended)

```bash
./scripts/tui_qa_record.sh record [output.cast]
# Default output: docs/tui_qa/tui_session_<date>.cast
# Stop recording: exit the TUI (e.g. press 'q') or Ctrl+D.
```

For **ttyrec** instead of asciinema:

```bash
TUI_QA_RECORDER=ttyrec ./scripts/tui_qa_record.sh record docs/tui_qa/session.ttyrec
```

### Option B: Manual asciinema

```bash
# Start recording (output file optional)
asciinema rec docs/tui_qa/demo.cast

# In the same shell, run the TUI
./scripts/run_rust_tui.sh

# Use the TUI (tab, scroll, etc.); then quit TUI (e.g. 'q'), exit shell or Ctrl+D to stop recording
```

### Option C: Manual ttyrec

```bash
ttyrec docs/tui_qa/session.ttyrec
./scripts/run_rust_tui.sh
# Quit TUI, then exit shell to stop recording
```

---

## 3. Replay

- **asciinema:** `asciinema play docs/tui_qa/demo.cast`
- **ttyrec:** `ttyplay docs/tui_qa/session.ttyrec` (e.g. in xterm or compatible terminal)

Or use the script:

```bash
./scripts/tui_qa_record.sh play docs/tui_qa/demo.cast
./scripts/tui_qa_record.sh play docs/tui_qa/session.ttyrec
```

---

## 4. Optional: VHS tape for minimal TUI flow

VHS runs **scripted** terminal sessions from `.tape` files. Use for reproducible demos or golden-output style tests (diff `.ascii` in CI). The TUI must be able to run in the same PTY; NATS and backend must be running when you run the tape.

**Example tape:** [../tui_qa/tui_minimal.tape](../tui_qa/tui_minimal.tape)

```bash
# Ensure NATS + backend are running, then:
vhs docs/tui_qa/tui_minimal.tape
# Produces docs/tui_qa/tui_minimal.ascii (and optionally GIF if configured)
```

To **record** a new tape (then edit by hand for reliability):

```bash
vhs record > docs/tui_qa/my_flow.tape
# Perform TUI actions, then exit; edit my_flow.tape as needed.
vhs docs/tui_qa/my_flow.tape
```

See [CLI_TUI_TOOLS_RECOMMENDATIONS.md](../CLI_TUI_TOOLS_RECOMMENDATIONS.md) for VHS CI usage (e.g. `charmbracelet/vhs-action`, diff `*.ascii`).

---

## 5. Quick reference

| Task              | Command |
|-------------------|--------|
| Record (asciinema)| `./scripts/tui_qa_record.sh record [file.cast]` |
| Record (ttyrec)   | `TUI_QA_RECORDER=ttyrec ./scripts/tui_qa_record.sh record [file.ttyrec]` |
| Replay            | `./scripts/tui_qa_record.sh play <file>` |
| VHS minimal tape  | `vhs docs/tui_qa/tui_minimal.tape` |
| Start stack       | `just services-active-start` |
| Run TUI only      | `just run-tui` or `./scripts/run_rust_tui.sh` |

---

## 6. Troubleshooting

- **TUI shows "No snapshot" / stale:** Backend not running or not publishing; check NATS and `backend_service` logs.
- **asciinema/ttyrec not found:** Install with `brew install asciinema ttyrec` (see Prerequisites).
- **VHS tape fails (e.g. "tui_service not found"):** Build TUI and ensure `agents/backend/target/debug/tui_service` (or release) is on `PATH`, or set `VHS_TUI_BIN` to full path in the tape or env.
- **Replay too fast/slow:** asciinema plays at recorded timing; ttyplay has speed options (see `ttyplay -h`).
