#!/usr/bin/env python3
"""Run tests using uvx and capture output to file."""
import subprocess
import sys
from pathlib import Path
import os

project_root = Path(__file__).parent.parent
os.chdir(project_root)

output_file = project_root / "test_results.txt"

print(f"Running tests from: {project_root}")
print(f"Output will be saved to: {output_file}")

# Try uvx first, fallback to pytest
commands = [
    ["uvx", "pytest", "python/tests/", "python/integration/", "-v", "--tb=short"],
    [sys.executable, "-m", "pytest", "python/tests/", "python/integration/", "-v", "--tb=short"],
]

for cmd in commands:
    print(f"\nTrying: {' '.join(cmd)}")
    try:
        result = subprocess.run(
            cmd,
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=300
        )

        with open(output_file, "w") as f:
            f.write(f"Command: {' '.join(cmd)}\n")
            f.write(f"Return code: {result.returncode}\n\n")
            f.write("=== STDOUT ===\n")
            f.write(result.stdout)
            f.write("\n=== STDERR ===\n")
            f.write(result.stderr)

        print(f"Return code: {result.returncode}")
        if result.stdout:
            print("\nSTDOUT (first 500 chars):")
            print(result.stdout[:500])
        if result.stderr:
            print("\nSTDERR (first 500 chars):")
            print(result.stderr[:500])

        print(f"\nFull output saved to: {output_file}")
        sys.exit(result.returncode)

    except FileNotFoundError:
        print(f"Command not found: {cmd[0]}")
        continue
    except subprocess.TimeoutExpired:
        print(f"Command timed out: {' '.join(cmd)}")
        continue
    except Exception as e:
        print(f"Error: {e}")
        continue

print("\nAll commands failed")
sys.exit(1)
