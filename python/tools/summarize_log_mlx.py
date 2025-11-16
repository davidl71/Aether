#!/usr/bin/env python3
"""
Summarize build/test logs on-device.

This script attempts to use MLX (Apple Silicon) if available.
Fallback: heuristic summary of errors/warnings and failing tests.

Usage:
  python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log
  python3 python/tools/summarize_log_mlx.py --path logs/tests_latest.log

Optional:
  --model  MLX model id or local path (e.g., mlx-community/Mistral-7B-Instruct-v0.2)
  --max_tokens  Max tokens to generate (default: 256)
"""
from __future__ import annotations

import argparse
import os
import re
import sys
from typing import Optional


def read_text(path: str) -> str:
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        return f.read()


def summarize_heuristic(text: str, is_test_log: bool) -> str:
    lines = text.splitlines()
    errors = [ln for ln in lines if re.search(r"\b(error|fatal)\b", ln, re.I)]
    warns = [ln for ln in lines if re.search(r"\b(warn(ing)?)\b", ln, re.I)]

    failing_tests = []
    if is_test_log:
        for ln in lines:
            if re.search(r"(FAILED|FAILED TEST|failures:)", ln, re.I):
                failing_tests.append(ln.strip())

    out = []
    out.append("Heuristic Summary")
    out.append("==================")
    out.append(f"Total lines: {len(lines)}")
    out.append(f"Errors: {len(errors)}")
    out.append(f"Warnings: {len(warns)}")
    if is_test_log:
        out.append(f"Failing-related lines: {len(failing_tests)}")
    out.append("")

    def head(lst, k=10):
        return "\n".join(f"- {x}" for x in lst[:k]) if lst else "(none)"

    out.append("Top error lines:")
    out.append(head(errors))
    out.append("")
    out.append("Top warning lines:")
    out.append(head(warns))
    if is_test_log:
        out.append("")
        out.append("Failing test cues:")
        out.append(head(failing_tests))
    return "\n".join(out)


def summarize_with_mlx(text: str, model_id: str, max_tokens: int) -> Optional[str]:
    try:
        # Lazy import to avoid hard dependency
        from mlx_lm import load, generate
    except Exception:
        return None

    try:
        model, tokenizer = load(model_id)
        prompt = (
            "You are a build/test log assistant. Summarize the following log. "
            "Highlight the most important errors, failing tests, and actionable next steps. "
            "Respond concisely with bullet points.\n\n"
            f"LOG:\n{text[:100000]}"
        )
        result = generate(model, tokenizer, prompt=prompt, max_tokens=max_tokens, verbose=False)
        return result
    except Exception:
        return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--path", required=True, help="Path to log file")
    ap.add_argument("--model", default=os.environ.get("MLX_MODEL", "mlx-community/Mistral-7B-Instruct-v0.2"))
    ap.add_argument("--max_tokens", type=int, default=256)
    args = ap.parse_args()

    if not os.path.exists(args.path):
        print(f"Log not found: {args.path}", file=sys.stderr)
        return 1

    text = read_text(args.path)
    is_test_log = "ctest" in text.lower() or "testing started" in text.lower()

    # Try MLX first
    summary = summarize_with_mlx(text, args.model, args.max_tokens)
    if summary is None:
        summary = summarize_heuristic(text, is_test_log)

    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
