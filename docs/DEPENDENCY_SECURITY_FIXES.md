# Recommended dependency and security fixes

Quick reference for addressing dependency vulnerabilities (e.g. GitHub Dependabot, `npm audit`, Python/Rust audits).

## 1. Root Node (docs lint) – **Applied**

- **Issue**: `markdownlint-cli2@^0.15.0` pulled in vulnerable `js-yaml` (prototype pollution) and `markdown-it` (ReDoS).
- **Fix**: Bumped to `markdownlint-cli2@^0.21.0` in `package.json`.
- **Verify**: From repo root run `npm install` then `npm audit` (expect 0 vulnerabilities for this tree).

## 2. Web app (`web/`)

- **Check**: `cd web && npm audit`.
- **Fix**: Prefer `npm audit fix`; use `npm audit fix --force` only if you accept possible breaking changes and re-test the app.

## 3. Python (`requirements.txt` / `requirements.in`)

- **Install**: `pip-audit` is installed by Ansible (devtools role, playbook `ansible/playbooks/setup_devtools.yml`). After running `./setup_global_tools.sh` or the playbook, use `pip-audit` (or `uv tool run pip-audit` if not on PATH).
- **Check**: `pip-audit -r requirements.txt` from repo root.
- **Fix**: Bump affected packages in `requirements.in`, then regenerate:
  ```bash
  uv pip compile requirements.in -o requirements.txt
  uv sync
  ```

## 4. Rust (`agents/backend/`)

- **Check**: `cargo audit` (requires Rust toolchain).
- **Fix**: Update `Cargo.toml` / `Cargo.lock` per `cargo audit` suggestions; run `cargo build` and tests.

## 5. GitHub Dependabot

- **Alerts**: <https://github.com/davidl71/ib_box_spread_full_universal/security/dependabot> (requires repo access).
- **Action**: Review open alerts; merge or recreate Dependabot PRs; prefer non-breaking upgrades and re-run tests/CI.

## 6. Cursor extension (`cursor-extension/`)

- **Check**: `cd cursor-extension && npm audit` if that package is in use.
- **Fix**: Same as web – `npm audit fix` or targeted version bumps.

## One-time verification (after fixes)

```bash
# Root
npm audit

# Web
cd web && npm audit && cd ..

# Python (if pip-audit installed)
pip-audit -r requirements.txt

# Rust (if cargo installed)
cd agents/backend && cargo audit && cd ../..
```
