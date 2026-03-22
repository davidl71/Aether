# Python Environment Setup

This repo no longer uses a Rust `PyO3` bridge. Python is only needed for helper scripts,
generated artifacts, and native binding tests under `native/tests/python/`.

## When you need Python

- Running `native/tests/python/`
- Working on helper scripts in `scripts/`
- Regenerating or inspecting Python protobuf output in `native/generated/python/`

Rust backend builds under `agents/backend/` do not require a Python interpreter.

## Quick Start

Using `uv`:

```bash
uv venv .venv
source .venv/bin/activate
uv pip install pytest
pytest native/tests/python -q
```

Using standard `venv`:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install pytest
pytest native/tests/python -q
```

## Notes

- The pybind11 extension is built from the native CMake build, not from Rust.
- Some Python tests may skip when the compiled module has not been built yet.
- No `PYO3_PYTHON` or backend-specific Python activation is required anymore.
