#!/usr/bin/env python3
"""
Run an MLX model (e.g., DiffuCode-7B-cpGRPO) to produce code review recommendations.

Usage:
  export MLX_MODEL=DiffuCode-7B-cpGRPO
  python3 python/tools/diffucode_review.py \
    --files native/src/box_spread_strategy.cpp native/src/order_manager.cpp native/src/tws_client.cpp CMakePresets.json

Notes:
  - Requires MLX + MLX-LM installed and the model available locally or downloadable.
  - Keep the file list concise; script truncates long contents to fit prompt.
"""
from __future__ import annotations

import argparse
import os
import textwrap
from pathlib import Path
from typing import List


def safe_read(path: Path, limit: int = 16000) -> str:
    try:
        data = path.read_text(encoding="utf-8", errors="replace")
        if len(data) > limit:
            half = limit // 2
            return data[:half] + "\n...\n[truncated]\n...\n" + data[-half:]
        return data
    except Exception as exc:
        return f"[ERROR READING FILE: {path} - {exc}]"


def build_prompt(files: List[Path]) -> str:
    parts = []
    for p in files:
        parts.append(f"FILE: {p}\n" + "-" * 80 + "\n" + safe_read(p) + "\n")
    corpus = "\n".join(parts)
    instructions = textwrap.dedent(
        """
        You are an expert C++/CMake reviewer for a trading application.
        Provide high-signal, actionable recommendations. Focus on:
        - Correctness and race conditions (TWS callbacks, threading)
        - Robust error handling and early-exit patterns
        - Performance (allocations, logging hot paths, unnecessary copies)
        - Cross-platform build hygiene (CMake presets, compiler flags)
        - Code style alignment (2-space indent, Allman braces)
        - Security and configuration validation (no secrets, input validation)
        Output:
        - Top 5–10 Recommendations (bullets, concise)
        - Quick Wins (bullets)
        - Potential Risks (bullets)
        """
    ).strip()
    return f"{instructions}\n\n=== CODE CONTEXT START ===\n{corpus}\n=== CODE CONTEXT END ===\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default=os.environ.get("MLX_MODEL", "DiffuCode-7B-cpGRPO"))
    ap.add_argument("--max_tokens", type=int, default=512)
    ap.add_argument("--files", nargs="+", required=True, help="Files to include in the review prompt.")
    args = ap.parse_args()

    try:
        from mlx_lm import load, generate
    except Exception as exc:
        print(f"MLX not available: {exc}")
        return 2

    files = [Path(f) for f in args.files if Path(f).exists()]
    if not files:
        print("No valid files provided.")
        return 1

    prompt = build_prompt(files)
    model, tokenizer = load(args.model)
    out = generate(model, tokenizer, prompt=prompt, max_tokens=args.max_tokens, verbose=False)
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
