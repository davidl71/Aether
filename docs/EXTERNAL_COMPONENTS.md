## Components worth extracting to a separate repository

These parts are useful across projects and benefit from independent versioning and CI.

### 1) Global Kit (this repo’s `global_kit/`)

- Contents: `.editorconfig`, `.gitignore_global`, `.markdownlint.json`, `cspell.json`, `.clang-format`, `.clang-tidy`, `.pre-commit-config.yaml`, `.shellcheckrc`
- Why: Consistent editor/linter experience across all repos
- How to link:
  - Option A: dedicated repo `dev-global-kit`, consumed via `git submodule` or copied into dotfiles
  - Option B: publish as release artifact and curl in CI
  - Option C: template repo for new projects

### 2) Extension Management Scripts

- Scripts: `scripts/analyze_by_category.sh`, `scripts/check_extension_security.sh`, `scripts/check_extension_redundancy.sh`, `scripts/disable_unwanted_extensions.sh`, `scripts/check_extensions.sh`
- Why: IDE-agnostic helpers (support `--ide code|cursor`)
- How to link:
  - Separate repo `ide-extension-tooling` pulled via submodule or vendored
  - CI: nightly job to audit devcontainers/codespaces images

### 3) CI Reusable Workflows

- Reusable GitHub Actions for CMake build/test, lint, and artifact upload
- Why: Share hardened pipelines (compiler matrix, cache, notarization stubs)
- How: `org/.github` repo with `workflow_call` actions

### 4) Schema/Contracts (e.g., Protobuf)

- Why: Shared between clients/tools/services; versioned independently
- How: Separate repo; publish language-specific packages (PyPI, npm, Maven, crates) via CI

### 5) CMake Toolchain/Presets

- Files: toolchain files, common presets for macOS universal, static analysis toggles
- Why: Standardize flags and build modes
- How: Separate repo consumed via `FetchContent` or subtree

## Linking strategies

- Submodule: precise pin, explicit updates (good for infra/config repos)
- Subtree: simpler for consumers, merges into tree (less indirection)
- Package releases: best for schemas/libs that need semantic versioning
- FetchContent/curl in CI: good for config snapshots or build toolchains

## CI recommendations

- Add update bots:
  - Weekly submodule bump PR
  - Dependabot for packaged components

- Pin versions; avoid floating main for critical toolchains
- Verify with `ctest --output-on-failure` and lint steps
