#!/usr/bin/env python3
"""
Summarize an arbitrary text/markdown file on-device.
Uses MLX (Apple Silicon) when available; otherwise a heuristic summary.

Usage:
  python3 python/tools/summarize_file_mlx.py --path <file> [--model <mlx-model>] [--max_tokens 256]
  cat README.md | python3 python/tools/summarize_file_mlx.py
"""
from __future__ import annotations

import argparse
import os
import sys
from typing import Optional

from summarize_log_mlx import summarize_heuristic, summarize_with_mlx  # type: ignore


def read_input(path: Optional[str]) -> str:
    if path:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    # Read from stdin
    return sys.stdin.read()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--path", help="Path to input file (text/markdown). If omitted, reads stdin.")
    ap.add_argument("--model", default=os.environ.get("MLX_MODEL", "mlx-community/Mistral-7B-Instruct-v0.2"))
    ap.add_argument("--max_tokens", type=int, default=256)
    args = ap.parse_args()

    text = read_input(args.path)
    if not text.strip():
        print("No input text found (empty file/stdin).", file=sys.stderr)
        return 2

    # Try MLX, else heuristic
    summary = summarize_with_mlx(text, args.model, args.max_tokens)
    if summary is None:
        summary = summarize_heuristic(text, is_test_log=False)
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
