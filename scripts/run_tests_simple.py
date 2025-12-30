#!/usr/bin/env python3
import subprocess
import sys
from pathlib import Path

project_root = Path(__file__).parent.parent
output_file = project_root / "test_output.txt"

try:
    result = subprocess.run(
        [sys.executable, "-m", "pytest", "python/tests/", "-v", "--tb=short"],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=300
    )

    with open(output_file, "w") as f:
        f.write("=== STDOUT ===\n")
        f.write(result.stdout)
        f.write("\n=== STDERR ===\n")
        f.write(result.stderr)
        f.write(f"\n=== RETURN CODE: {result.returncode} ===\n")

    print(f"Test execution complete. Results saved to: {output_file}")
    print(f"Return code: {result.returncode}")
    if result.stdout:
        print("\nSTDOUT preview:")
        print(result.stdout[:500])

except Exception as e:
    with open(output_file, "w") as f:
        f.write(f"ERROR: {e}\n")
    print(f"Error: {e}")
