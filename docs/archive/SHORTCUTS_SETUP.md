# macOS Shortcuts Setup (Build & Test)

This guide shows how to create macOS Shortcuts that trigger the repository’s build and test workflows and then summarize logs on-device.

## Prerequisites

- macOS Sequoia or Ventura+
- Xcode CLT, CMake, Ninja installed
- Optional: Python 3 for on-device summarization

## Scripts Provided

- `scripts/shortcuts/run_build.sh` – runs configure+build with a sensible preset and writes `logs/build_latest.log`
- `scripts/shortcuts/run_tests.sh` – runs `ctest` with preset and writes `logs/tests_latest.log`
- `python/tools/summarize_log_mlx.py` – summarizes logs using MLX if available, otherwise a heuristic summary

## Create “Build Project” Shortcut

1. Open Shortcuts.app
2. Create a new Shortcut named “Build Project”
3. Add action “Run Shell Script”
4. Set Shell to `/bin/zsh`
5. Script:

   ```
   cd <path-to-your-repo>
   ./scripts/shortcuts/run_build.sh build
   ```

6. Save

## Create “Run Tests” Shortcut

1. New Shortcut “Run Tests”
2. Action: “Run Shell Script”
3. Script:

   ```
   cd <path-to-your-repo>
   ./scripts/shortcuts/run_tests.sh
   ```

4. Save

## Optional: Summarize Logs Shortcut

1. New Shortcut “Summarize Latest Build”
2. Action: “Run Shell Script”
3. Script:

   ```
   cd <path-to-your-repo>
   python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log
   ```

4. For tests, use `--path logs/tests_latest.log`

## Notes

- Presets auto-detect platform (ARM64/x86_64, macOS/Linux/Windows via WSL) inside the scripts.
- You can set `CMAKE_PRESET`/`CTEST_PRESET` env vars in the Shortcut to force a specific preset.
- Logs are written to `logs/` (already gitignored by default patterns in `.cursorignore`).
