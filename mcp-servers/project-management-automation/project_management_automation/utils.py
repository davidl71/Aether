"""
Utility functions for project management automation.
"""

from pathlib import Path
from typing import Optional


def find_project_root(start_path: Optional[Path] = None) -> Path:
    """
    Find project root by looking for marker files.

    Looks for .git, .todo2, or CMakeLists.txt to identify project root.

    Args:
        start_path: Starting path for search (default: current file location)

    Returns:
        Path to project root, or current working directory if not found
    """
    if start_path is None:
        # Default to current working directory
        start_path = Path.cwd()
    else:
        start_path = Path(start_path)

    current = start_path.resolve()

    # Look for project markers
    while current != current.parent:
        if (current / '.git').exists() or (current / '.todo2').exists() or (current / 'CMakeLists.txt').exists():
            return current
        current = current.parent

    # Fallback to current working directory
    return Path.cwd()
