#!/usr/bin/env python3
"""
Code review using Ollama (CodeLlama) for privacy-sensitive code analysis.

Usage:
  python3 python/tools/ollama_code_review.py \
    --files native/src/risk_calculator.cpp native/src/order_manager.cpp \
    --model codellama:7b

Notes:
  - Uses Ollama MCP server for local code analysis
  - All code stays on your local machine (privacy-preserving)
  - Best for proprietary trading algorithms and sensitive code
  - Requires Ollama service running and model downloaded
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import List

try:
    import requests
except ImportError:
    print("Error: requests not installed. Install with: pip install requests")
    sys.exit(1)


def safe_read(path: Path, limit: int = 16000) -> str:
    """Read file with truncation for large files."""
    try:
        data = path.read_text(encoding="utf-8", errors="replace")
        if len(data) > limit:
            half = limit // 2
            return data[:half] + "\n...\n[truncated]\n...\n" + data[-half:]
        return data
    except Exception as exc:
        return f"[ERROR READING FILE: {path} - {exc}]"


def build_review_prompt(files: List[Path]) -> str:
    """Build code review prompt for Ollama."""
    parts = []
    for p in files:
        parts.append(f"FILE: {p}\n" + "=" * 80 + "\n" + safe_read(p) + "\n")
    corpus = "\n".join(parts)

    instructions = """
You are an expert C++ code reviewer for a trading application.
Review the code for:

1. **Security Issues**: Memory safety, input validation, path boundaries
2. **Correctness**: Logic errors, edge cases, race conditions
3. **Error Handling**: Missing checks, exception safety
4. **Performance**: Unnecessary allocations, hot path optimizations
5. **Best Practices**: Code style, maintainability, documentation

Provide:
- Top 5-10 actionable recommendations (bullets)
- Quick wins (easy fixes)
- Potential risks (critical issues)

Be concise and specific. Focus on high-signal issues.
"""

    return f"{instructions}\n\n=== CODE TO REVIEW ===\n{corpus}\n=== END CODE ===\n"


def query_ollama(
    model: str, prompt: str, base_url: str = "http://localhost:11434"
) -> str:
    """Query Ollama API for code review."""
    try:
        response = requests.post(
            f"{base_url}/api/generate",
            json={
                "model": model,
                "prompt": prompt,
                "stream": False,
                "options": {
                    "temperature": 0.3,  # Lower temperature for more focused reviews
                    "num_predict": 2000,  # Max tokens
                },
            },
            timeout=120,
        )
        response.raise_for_status()
        return response.json().get("response", "No response from Ollama")
    except requests.exceptions.RequestException as e:
        return f"Error querying Ollama: {e}"


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Code review using Ollama (privacy-preserving)"
    )
    ap.add_argument("--files", nargs="+", required=True, help="Files to review")
    ap.add_argument(
        "--model",
        default="codellama:7b",
        help="Ollama model to use (default: codellama:7b)",
    )
    ap.add_argument(
        "--base-url",
        default="http://localhost:11434",
        help="Ollama API base URL (default: http://localhost:11434)",
    )
    args = ap.parse_args()

    # Check Ollama is running
    try:
        response = requests.get(f"{args.base_url}/api/tags", timeout=5)
        response.raise_for_status()
    except requests.exceptions.RequestException:
        print(f"Error: Ollama not running at {args.base_url}")
        print("Start Ollama with: ollama serve")
        return 1

    # Read files
    files = [Path(f) for f in args.files if Path(f).exists()]
    if not files:
        print("Error: No valid files found")
        return 1

    print(f"📋 Reviewing {len(files)} file(s) with {args.model}...")
    print("=" * 80)

    # Build prompt and query
    prompt = build_review_prompt(files)
    review = query_ollama(args.model, prompt, args.base_url)

    print(review)
    print("=" * 80)
    print(f"✅ Review complete using {args.model}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
