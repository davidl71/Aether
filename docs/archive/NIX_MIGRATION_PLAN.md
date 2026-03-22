# Nix migration plan

This document describes the current Nix setup (option 3) and the planned path to a single flake-based toolchain (options 1 and 2).

## Current state: Nix dev shell only (option 3)

- **`flake.nix`** at repo root provides a development shell with C++, Python, Rust, and Node tooling. It **coexists** with the existing Ansible/Homebrew/uv setup.
- **Use:** `nix develop` (requires [Nix with flakes](https://nix.dev/install-nix#install-nix) enabled). On many installs you need: `nix develop --extra-experimental-features 'nix-command flakes'`.
- **Scripts:** Set **`USE_NIX=1`** to run these scripts inside the Nix dev shell (they re-exec via `scripts/with_nix.sh`):
  - `scripts/build_universal.sh`
  - `scripts/run_linters.sh`
  - `scripts/build_variant.sh`
  - `scripts/verify_toolchain.sh`
  - `scripts/run_python_tests.sh`
- **Just:** Run any recipe inside Nix with **`just nix <recipe>`**, e.g. `just nix build`, `just nix test-python`.
- **Scope:** The shell supplies `cmake`, `ninja`, `boost`, `protobuf`, `abseil-cpp`, `curl`, `python3`, `uv`, `rustc`, `cargo`, `nodejs_22`, `jq`, `shellcheck`, `cppcheck`, `git`, `sqlite`. TWS API and Intel decimal are still vendored; install and global tooling (nvm, act, NATS CLI, git-lfs, etc.) remain via **`./setup_global_tools.sh`** and **Ansible** (`ansible/playbooks/setup_devtools.yml`).
- **Optional (direnv):** If you use [direnv](https://direnv.net/) with [nix-direnv](https://github.com/nix-community/nix-direnv), create a `.envrc` with `use flake` and run `direnv allow` so the Nix dev shell loads automatically in this repo. (`.envrc` is currently gitignored via `.env*`; add `!.envrc` to `.gitignore` if you want to commit it.)
- **Ansible:** The **`devtools`** role installs Nix on macOS and Debian/Ubuntu (when not already present), and configures `~/.config/nix/nix.conf` with `experimental-features = nix-command flakes`. Run: `ansible-playbook ansible/playbooks/setup_devtools.yml`. The first-time Nix install may prompt for confirmation; re-run the playbook if you skip it.

---

## Plan: Option 1 — Single flake as source of truth

**Goal:** Replace or complement `setup_global_tools.sh` and Ansible devtools with one Nix flake so the same toolchain is defined in a single place.

### Steps

1. **Expand `flake.nix`** to include every tool currently installed by Ansible/Homebrew for development:
   - From `ansible/roles/devtools/tasks/main.yml`: `act`, `fswatch`, `git-lfs`, `ccache`, `sccache`, `markdownlint-cli`, `doxygen`, `nats-server`, NATS CLI (tap), `go`, and (on Linux) equivalents for apt packages.
   - Add any macOS-specific needs (e.g. Xcode CLT not replaced; Nix provides compilers/libs).

2. **Optional: `nixos-*` / `darwin-*` modules**  
   If we want declarative system config later (NixOS or nix-darwin), introduce a module that consumes this flake and installs the same tools system-wide. Not required for “single flake” dev tooling.

3. **Decide replacement vs complement:**
   - **Replace:** Make `nix develop` the only way to get a full dev environment; deprecate `setup_global_tools.sh` and the Ansible devtools role for new contributors; document “Install Nix, then `nix develop`” in README/AGENTS.md.
   - **Complement:** Keep Ansible/scripts for those who don’t use Nix; document both paths (Nix preferred, Ansible/script fallback) and keep the flake and Ansible package list in sync manually or via a small script.

4. **Vendored / external deps:**  
   TWS API and Intel decimal stay vendored; the flake does not need to build them. Optionally add a flake input or Nix overlay for them later if we want to build from source inside Nix.

5. **Python/Node/Rust versions:**  
   Pin in the flake (e.g. `python312`, `nodejs_22`, `rustChannelOf`) so “option 1” gives reproducible versions and matches what CI uses (see option 2).

---

## Plan: Option 2 — Unify CI and local with `nix develop`

**Goal:** CI (e.g. GitHub Actions) and local development use the same Nix dev shell so there is one definition of “the” environment.

### Steps

1. **CI job using Nix:**  
   In GitHub Actions (or other CI), install Nix with flakes, then run:
   - `nix develop --command cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug`
   - `nix develop --command ninja -C build`
   - `nix develop --command ctest --test-dir build --output-on-failure`
   - Similarly for Python: `nix develop --command uv run pytest python/tests/`, and for Rust: `nix develop --command cargo build` (and test) in `agents/`.

2. **Cache Nix store (optional but recommended):**  
   Use [Cachix](https://www.cachix.org/) or GitHub cache for the Nix store so CI doesn’t rebuild the world every run. Document the cache in the repo (e.g. `cachix use <name>` in README or a CI step).

3. **Single source of truth:**  
   The same `flake.nix` used locally with `nix develop` is the one used in CI. No separate Dockerfile or `apt-get`/`brew install` lists for dev tooling; at most a minimal runner that installs Nix and runs `nix develop --command ...`.

4. **Optional: `nix build` for release artifacts:**  
   Add a `packages.default` (or named packages) to the flake that build the native CLI, Python wheel, or other artifacts, so CI can run `nix build` for reproducible release builds. This can come after “option 1” is done.

---

## Order of work

| Phase | What |
|-------|------|
| **Done** | Option 3: Nix dev shell only; keep Ansible/scripts for global tooling. |
| **Next** | Option 1: Expand flake to cover all dev tools; decide replace vs complement; document. |
| **Then** | Option 2: Add CI job that uses `nix develop` (and optionally Nix cache); remove duplicate tool installs from CI. |

---

## References

- [nix.dev](https://nix.dev/) — official Nix docs
- [Install Nix](https://nix.dev/install-nix#install-nix)
- [Ad-hoc shell environments](https://nix.dev/tutorials/first-steps/ad-hoc-shell-environments)
- [Nix flakes](https://nix.dev/concepts/flakes)
- [Cachix for CI](https://docs.cachix.org/continuous-integration-setup/)
