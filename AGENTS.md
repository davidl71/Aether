# Repository Guidelines

## Project Structure & Module Organization

Core pricing logic lives in `src/` with headers under `include/ib_box_spread/`. `src/box_spread_calc.cpp` holds the reusable calculations, while `src/ib_box_spread.cpp` renders the default XSP box-spread table used by the CLI/TUI binary. The AppKit bundle target resides in `app/` (Objective-C++). Tests belong in `tests/` and should mirror the production file names they exercise. Helper tooling stays in `scripts/`: `scripts/build_universal.sh` wraps fat-binary builds, `ibapi_cmake/` hosts the vendor CMake glue, and generated output should land in disposable directories such as `build/` or `protobuf-build/`.

## Build, Test & Development Commands

Configure once with Ninja for fast rebuilds:
`cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib`
Rebuild with `ninja -C build`. Run the CLI/TUI via `./build/ib_box_spread`; it emits tab-separated summaries for three strike widths. Open the macOS bundle using `open build/ib_box_spread_app.app`. Produce a universal binary when needed with `./scripts/build_universal.sh`. Execute regression tests after every change: `ctest --test-dir build --output-on-failure`.

## Coding Style & Naming Conventions

Target ISO C++17. Prefer two-space indentation, Allman braces for multi-line scopes, and 100-character soft wraps. Use `PascalCase` for types (`Scenario`), `snake_case` for functions (`make_scenario`) and locals, and prefix constants with `k`. Keep formatting explicit with `<iomanip>` helpers so TSV columns stay aligned. Add short `//` comments only where the trading math is non-obvious (e.g., APR scaling by the contract multiplier).

## IB API Integration Notes

Point CMake to an extracted IB API checkout (`~/IBJts/source/cppclient`) and the compiled client library; the included `ibapi_cmake` presets can rebuild `libTwsApiCpp.dylib` plus the Intel decimal math dependency. Never commit IB credentials, logs, or downloaded vendor artifacts—treat everything under `ibapi_build/`, `protobuf-build/`, and `build/` as ephemeral. The CLI currently prints synthetic market data; gate any future live requests behind configuration flags.

## Testing Guidelines

`tests/box_spread_calc_test.cpp` exercises the pricing helpers without touching the network. Expand coverage alongside new features, mirroring source names (`ib_market_data_client` ⇒ `ib_market_data_client_test`). Run `ctest --test-dir build --output-on-failure` locally before pushes and attach the command transcript to review threads when failures occur.

## Commit & Pull Request Guidelines

Follow imperative, 72-character subject lines (“Add TSV formatter for CLI”). In the body, summarize option scenarios touched, list the commands run (build, tests, sample CLI output), and note IB API version bumps. PRs should include screenshots or pasted TSV blocks only when they clarify behaviour, and must call out configuration changes (e.g., new env vars or IB gateway ports).
