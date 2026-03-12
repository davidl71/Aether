#!/usr/bin/env python3
"""Run pytest and capture output to file. Single entrypoint for test capture.

Supports:
  - uv run with fallback to python -m pytest
  - --simple: run native/tests/python/ only
  - Single output file: test_results.txt
  - Timeout and exit code propagation
"""
import argparse
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
OUTPUT_FILE = "test_results.txt"


def main() -> int:
    parser = argparse.ArgumentParser(description="Run pytest and capture output to file.")
    parser.add_argument(
        "--simple",
        action="store_true",
        help="Run native/tests/python/ only.",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=300,
        help="Pytest timeout in seconds (default: 300).",
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=None,
        help=f"Output file path (default: project root / {OUTPUT_FILE}).",
    )
    args = parser.parse_args()

    paths = ["native/tests/python/"]
    output_path = args.output or (PROJECT_ROOT / OUTPUT_FILE)
    if not output_path.is_absolute():
        output_path = PROJECT_ROOT / output_path

    commands = [
        ["uv", "run", "--with", "pytest", "pytest", *paths, "-v", "--tb=short"],
        [sys.executable, "-m", "pytest", *paths, "-v", "--tb=short"],
    ]

    for cmd in commands:
        try:
            result = subprocess.run(
                cmd,
                cwd=PROJECT_ROOT,
                capture_output=True,
                text=True,
                timeout=args.timeout,
            )
            with open(output_path, "w") as f:
                f.write(f"Command: {' '.join(cmd)}\n")
                f.write(f"Return code: {result.returncode}\n\n")
                f.write("=== STDOUT ===\n")
                f.write(result.stdout)
                f.write("\n=== STDERR ===\n")
                f.write(result.stderr)

            print(f"Output saved to: {output_path}")
            return result.returncode

        except FileNotFoundError:
            continue
        except subprocess.TimeoutExpired:
            with open(output_path, "w") as f:
                f.write(f"Command timed out: {' '.join(cmd)}\n")
            print(f"Timeout: {' '.join(cmd)}", file=sys.stderr)
            return 1
        except Exception as e:
            with open(output_path, "w") as f:
                f.write(f"ERROR: {e}\n")
            print(f"Error: {e}", file=sys.stderr)
            return 1

    print("No pytest runner available (tried uv run and python -m pytest).", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
