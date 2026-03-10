# Continuous Integration Strategy

## Pipeline Overview

- **Global Tooling Gate**
  - GitHub Actions job executes `setup_global_tools.sh` (Linux + macOS) to ensure Ansible playbook stays green.

1. **Python Backend (Poetry)**
   - `poetry install`
   - `poetry run pytest`
   - Optional: run mock TWS integration tests and schema validation.

2. **Python/Textual TUI**
   - Run: `./scripts/run_python_tui.sh`
   - Tests: Python TUI tests run as part of the main Python test suite

3. **Web SPA**
   - `npm install`
   - `npm test -- --watch=false`

4. **iPad/Desktop**
   - If Swift projects exist, run `xcodebuild test`.
   - For Electron-based desktop, run `npm install && npm test`.

5. **Shared artifacts**
   - Validate `agents/shared/API_CONTRACT.md` (lint or schema check) if automated.
   - Ensure `agents/shared/TODO_OVERVIEW.md` stays in sync with tracked issues.

## GitHub Actions Sketch

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Install Poetry
        run: pip install poetry
      - name: Backend setup & tests
        run: bash agents/backend/scripts/run-tests.sh

  tui:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: TUI tests
        run: bash scripts/run_python_tests.sh

  web:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Web tests
        run: bash agents/web/scripts/run-tests.sh

  ios:
    runs-on: macos-latest
    if: ${{ hashFiles('ios/**') != '' }}
    steps:
      - uses: actions/checkout@v4
      - name: iPad tests
        run: bash agents/ipad/scripts/run-tests.sh

  desktop:
    runs-on: macos-latest
    if: ${{ hashFiles('desktop/**') != '' }}
    steps:
      - uses: actions/checkout@v4
      - name: Desktop tests
        run: bash agents/desktop/scripts/run-tests.sh
```

## Coordination

- Each agent should update `TODO_OVERVIEW.md` as tasks complete.
- Backend publishes schema updates in `API_CONTRACT.md`; frontends sync before merging.
- Tag releases (e.g., `git tag v1.2.0`) after CI passes on main.
