#!/usr/bin/env python3
"""
Draft a concise implementation plan from a spec using MLX (Apple Silicon) when available.
Falls back to a simple rule-based outline if MLX isn't present.

Usage:
  python3 python/tools/draft_plan_mlx.py --path spec.md
  pbpaste | python3 python/tools/draft_plan_mlx.py
"""
from __future__ import annotations

import argparse
import os
import sys
from typing import Optional

from summarize_log_mlx import summarize_with_mlx  # type: ignore


TEMPLATE = """You are a senior software engineer.
Given the SPEC below, produce a concise, actionable plan:
- Objectives (bullets)
- Constraints/Risks (bullets)
- Milestones (3-6 items)
- Tasks (up to 10, short imperative sentences)
- Validation/Success Criteria (bullets)
SPEC:
{spec}
"""


def read_input(path: Optional[str]) -> str:
    if path:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    return sys.stdin.read()


def fallback_outline(text: str) -> str:
    return "\n".join(
        [
            "Plan (Fallback Outline)",
            "=======================",
            "- Objectives:",
            "  - Extract core goals from the spec",
            "  - Identify success criteria",
            "- Constraints/Risks:",
            "  - Dependencies and environment assumptions",
            "  - Time and resource risks",
            "- Milestones:",
            "  1) Prototype",
            "  2) Integrate",
            "  3) Validate",
            "  4) Document & Handoff",
            "- Tasks:",
            "  - Implement core feature",
            "  - Add tests",
            "  - Update docs",
            "  - Run CI & address issues",
            "- Validation/Success:",
            "  - Tests passing, docs updated, criteria met",
        ]
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--path", help="Path to spec text/markdown. If omitted, reads stdin.")
    ap.add_argument("--model", default=os.environ.get("MLX_MODEL", "mlx-community/Mistral-7B-Instruct-v0.2"))
    ap.add_argument("--max_tokens", type=int, default=256)
    args = ap.parse_args()

    text = read_input(args.path)
    if not text.strip():
        print("No input text found (empty file/stdin).", file=sys.stderr)
        return 2

    prompt = TEMPLATE.format(spec=text[:100000])
    result = summarize_with_mlx(prompt, args.model, args.max_tokens)
    if result is None:
        result = fallback_outline(text)
    print(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
