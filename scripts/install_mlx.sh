#!/usr/bin/env bash
# Install MLX and MLX-LM where supported (Apple Silicon macOS).
# Safe no-op on unsupported platforms.
set -euo pipefail

arch="$(uname -m 2>/dev/null || echo unknown)"
os="$(uname -s 2>/dev/null || echo unknown)"

if [[ "${os}" == "Darwin" && ( "${arch}" == "arm64" || "${arch}" == "aarch64" ) ]]; then
  echo "[install-mlx] Detected Apple Silicon macOS (${arch})."
  if ! command -v python3 >/dev/null 2>&1; then
    echo "[install-mlx] python3 not found. Install Python (e.g., 'brew install python')." >&2
    exit 1
  fi
  echo "[install-mlx] Installing/upgrading mlx and mlx-lm via pip..."
  python3 -m pip install --user --upgrade mlx mlx-lm
  echo "[install-mlx] Verifying import..."
  python3 - <<'PY'
import sys
try:
    import mlx  # noqa: F401
    import mlx_lm  # noqa: F401
    print("OK")
except Exception as e:
    print(f"ERR: {e}", file=sys.stderr)
    sys.exit(1)
PY
  echo "[install-mlx] Done."
else
  echo "[install-mlx] Platform not supported for MLX (os=${os}, arch=${arch}). Skipping."
fi
