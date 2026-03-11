"""
Shared pytest configuration for python/tests.

- Adds project root to sys.path so that "from python.integration.X" works when
  running pytest from repo root (e.g. pytest python/tests/ or uv run pytest python/tests/).
  Note: Most integration modules are retired; only discount_bank_helpers remains.
- Provides mock_http_response() helper for broker/client tests.
"""

from pathlib import Path
import sys

# Project root (parent of python/)
_root = Path(__file__).resolve().parent.parent.parent
if str(_root) not in sys.path:
    sys.path.insert(0, str(_root))


def mock_http_response(json_data, status_code=200):
    """Create a MagicMock response with .json(), .status_code, .raise_for_status."""
    from unittest.mock import MagicMock

    resp = MagicMock()
    resp.status_code = status_code
    resp.ok = status_code < 400
    resp.json.return_value = json_data
    resp.raise_for_status = MagicMock()
    if status_code >= 400:
        resp.raise_for_status.side_effect = Exception(f"HTTP {status_code}")
    return resp
