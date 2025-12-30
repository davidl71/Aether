#!/usr/bin/env python3
"""
Generate documentation using Ollama (Mistral) for privacy-sensitive code.

Usage:
  python3 python/tools/ollama_documentation.py \
    --file native/src/risk_calculator.cpp \
    --model mistral:7b \
    --output docs/RISK_CALCULATOR_API.md

Notes:
  - Uses Ollama MCP server for local documentation generation
  - All code stays on your local machine (privacy-preserving)
  - Best for proprietary trading algorithms and sensitive code
  - Requires Ollama service running and model downloaded
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import requests
except ImportError:
    print("Error: requests not installed. Install with: pip install requests")
    sys.exit(1)


def safe_read(path: Path, limit: int = 20000) -> str:
    """Read file with truncation for large files."""
    try:
        data = path.read_text(encoding="utf-8", errors="replace")
        if len(data) > limit:
            half = limit // 2
            return data[:half] + "\n...\n[truncated]\n...\n" + data[-half:]
        return data
    except Exception as exc:
        return f"[ERROR READING FILE: {path} - {exc}]"


def build_doc_prompt(file_path: Path, code: str) -> str:
    """Build documentation generation prompt for Ollama."""
    instructions = f"""
Generate comprehensive API documentation for this C++ code file: {file_path.name}

Include:

1. **Class/Module Overview**: Purpose, responsibilities, and use cases
2. **Method Documentation**:
   - Function signatures with parameter descriptions
   - Return value descriptions
   - Preconditions and postconditions
3. **Usage Examples**: Code snippets showing how to use the API
4. **Important Considerations**:
   - Edge cases and error conditions
   - Thread safety notes
   - Performance considerations
   - Memory management notes
5. **Integration Notes**: How this module fits into the larger system

Format as markdown suitable for Doxygen or similar documentation generators.
Use clear sections, code blocks with syntax highlighting, and examples.

The code is part of a trading application for box spread arbitrage on Interactive Brokers.
"""

    return f"{instructions}\n\n=== CODE ===\n{code}\n=== END CODE ===\n"


def query_ollama(
    model: str, prompt: str, base_url: str = "http://localhost:11434"
) -> str:
    """Query Ollama API for documentation generation."""
    try:
        response = requests.post(
            f"{base_url}/api/generate",
            json={
                "model": model,
                "prompt": prompt,
                "stream": False,
                "options": {
                    "temperature": 0.5,  # Balanced for documentation
                    "num_predict": 3000,  # More tokens for comprehensive docs
                },
            },
            timeout=180,
        )
        response.raise_for_status()
        return response.json().get("response", "No response from Ollama")
    except requests.exceptions.RequestException as e:
        return f"Error querying Ollama: {e}"


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Generate documentation using Ollama (privacy-preserving)"
    )
    ap.add_argument("--file", required=True, help="File to document")
    ap.add_argument(
        "--model",
        default="mistral:7b",
        help="Ollama model to use (default: mistral:7b)",
    )
    ap.add_argument("--output", help="Output file path (default: print to stdout)")
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

    # Read file
    file_path = Path(args.file)
    if not file_path.exists():
        print(f"Error: File not found: {file_path}")
        return 1

    print(f"📝 Generating documentation for {file_path.name} with {args.model}...")

    # Build prompt and query
    code = safe_read(file_path)
    prompt = build_doc_prompt(file_path, code)
    documentation = query_ollama(args.model, prompt, args.base_url)

    # Add header
    header = f"""# {file_path.stem} API Documentation

*Generated using Ollama ({args.model})*
*File: {file_path}*

---

"""
    full_doc = header + documentation

    # Write output
    if args.output:
        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(full_doc, encoding="utf-8")
        print(f"✅ Documentation written to: {output_path}")
    else:
        print(full_doc)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
