#!/usr/bin/env python3
"""
Run Python tests and capture output to file.
"""
import subprocess
import sys
import os
from pathlib import Path

# Change to project root
project_root = Path(__file__).parent.parent
os.chdir(project_root)

# Run pytest
result = subprocess.run(
    [sys.executable, "-m", "pytest", "python/tests/", "python/integration/", "-v", "--tb=short"],
    capture_output=True,
    text=True,
    cwd=project_root
)

# Print output
print("STDOUT:")
print(result.stdout)
print("\nSTDERR:")
print(result.stderr)
print(f"\nReturn code: {result.returncode}")

# Save to file
output_file = project_root / "test_results.txt"
with open(output_file, "w") as f:
    f.write("STDOUT:\n")
    f.write(result.stdout)
    f.write("\nSTDERR:\n")
    f.write(result.stderr)
    f.write(f"\nReturn code: {result.returncode}\n")

print(f"\nResults saved to: {output_file}")

sys.exit(result.returncode)
