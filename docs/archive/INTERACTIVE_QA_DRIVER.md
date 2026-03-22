# Interactive QA Driver (Expect / pexpect / PTY)

**Purpose:** Automate interactive QA of CLI and TUI using PTY-based drivers. This doc evaluates options, describes minimal flow, and references the existing tools doc.

**See also:** [docs/CLI_TUI_TOOLS_RECOMMENDATIONS.md](CLI_TUI_TOOLS_RECOMMENDATIONS.md) — VHS, asciinema, ATAC, httpie, and scripted terminal testing.

---

## Option evaluation

| Tool | Pros | Cons | Best for |
|------|------|------|----------|
| **Expect** (Tcl) | Ubiquitous on Unix, no extra deps, battle-tested | Tcl syntax, less ergonomic for assertions | CI scripts where only `expect` is available |
| **pexpect** (Python) | Pure Python, easy to read/write, `pexpect.spawn` + `expect()` | Requires Python + pexpect; Windows needs `pexpect.popen_spawn` | Interactive CLI tests, subprocess automation |
| **Rust PTY** (e.g. `expectrl`, `pty-process`) | Same language as CLI/TUI, no Python | More setup, fewer examples for interactive QA | Deep integration in Rust test harness |
| **VHS** (Charmbracelet) | Scripted `.tape` files, golden-output in CI, demos | Different model (type/sleep/require); not line-by-line expect | Regression tests and docs; see CLI_TUI_TOOLS_RECOMMENDATIONS.md |

**Recommendation:** Use **pexpect** for a quick, maintainable example (see [tests/expect/README.md](../tests/expect/README.md) and `cli_help_pexpect.py`). Use **VHS** for golden-file CLI/TUI flows as already recommended in CLI_TUI_TOOLS_RECOMMENDATIONS.md. Consider Rust PTY only if you need interactive QA fully inside the Rust test suite.

---

## Minimal flow

1. **Spawn** the process under a PTY (e.g. `pexpect.spawn(cmd, timeout=...)` or `expectrl::spawn`).
2. **Send** input (e.g. `child.sendline("--help")`).
3. **Expect** output patterns or EOF (e.g. `child.expect(["Usage", "Aether", pexpect.EOF], timeout=5)`).
4. **Assert** on matched index or captured output; exit with non-zero on failure for CI.

---

## CLI vs TUI

| Target | Notes |
|--------|--------|
| **CLI** | Easy: run `cli --help` or `cli validate`; output is line-oriented; short timeouts (2–5 s) usually enough. |
| **TUI** | Harder: full-screen, control sequences, timing-dependent. Prefer (a) short runs with expect on status line or first frame, or (b) VHS tapes that script key presses and sleep. Use longer timeouts (e.g. 10–15 s) and expect on stable strings (e.g. "Dashboard" or "NATS"). |

---

## Timeouts

- **CLI help/subcommands:** 2–5 s.
- **CLI validate/snapshot (no backend):** 5–10 s.
- **TUI startup + first paint:** 10–15 s.
- **TUI after key presses:** add 1–2 s per interaction; avoid flaky tests by expecting stable text, not animations.

---

## Example and how to run

- **Pexpect (Python):** [tests/expect/cli_help_pexpect.py](../tests/expect/cli_help_pexpect.py) — runs the Rust CLI `--help` and asserts "Usage" or "Aether" in output.
- **Run:** From repo root, build CLI then run the test (see [tests/expect/README.md](../tests/expect/README.md)):
  - `cd agents/backend && cargo build -p cli`
  - `uv run python tests/expect/cli_help_pexpect.py` (requires `pexpect` in env: `uv pip install pexpect` or add to project deps).

---

## References

- [CLI_TUI_TOOLS_RECOMMENDATIONS.md](CLI_TUI_TOOLS_RECOMMENDATIONS.md) — VHS, asciinema, ATAC, httpie, jq, k6, and scripted terminal testing.
- [Pexpect 4.8 docs](https://pexpect.readthedocs.io/en/stable/) — spawn, expect, timeouts.
- [VHS (Charmbracelet)](https://github.com/charmbracelet/vhs) — scripted tapes and golden output.
