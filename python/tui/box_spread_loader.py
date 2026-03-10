"""
Load box spread scenario data from REST API or local file.

Used by TUIApp to refresh the Scenarios tab without embedding HTTP + file logic in the app.
"""

from __future__ import annotations

import json
import logging
from pathlib import Path
from typing import Optional, Tuple, Any

from .models import BoxSpreadPayload
from .config import TUIConfig

logger = logging.getLogger(__name__)


def get_box_spread_payload(
    config: TUIConfig,
    file_path: Path,
    last_file_mtime: Optional[float] = None,
) -> Tuple[Optional[BoxSpreadPayload], Optional[float]]:
    """
    Load box spread payload from REST (derived from config.rest_endpoint) or from file.

    Returns:
        (payload, new_file_mtime): payload is None if no data could be loaded;
        new_file_mtime is set only when data was loaded from file (for skip-unchanged optimization).
    """
    data: Optional[dict[str, Any]] = None
    new_mtime: Optional[float] = None

    # Prefer API router base when set; otherwise derive base from snapshot endpoint
    base: Optional[str] = None
    api_base_url = getattr(config, "api_base_url", None)
    if api_base_url:
        base = api_base_url.strip().rstrip("/")
    if not base:
        base = (config.rest_endpoint or "").rsplit("/", 1)[0]

    # Try REST first: /scenarios from base
    if base:
        api_url = f"{base}/scenarios"
        try:
            import requests
            resp = requests.get(
                api_url, timeout=2.0, headers={"Accept": "application/json"}
            )
            if resp.ok:
                data = resp.json()
        except Exception:
            pass

    # Fallback: file
    if data is None and file_path.exists():
        try:
            current_mtime = file_path.stat().st_mtime
            if last_file_mtime is not None and current_mtime <= last_file_mtime:
                return (None, None)
            with open(file_path, "r") as f:
                data = json.load(f)
            new_mtime = current_mtime
        except Exception as e:
            logger.error("Error reading box spread file: %s", e)
            return (None, None)

    if data is None:
        return (None, None)

    try:
        payload = BoxSpreadPayload.from_dict(data)
        return (payload, new_mtime)
    except Exception as e:
        logger.error("Error parsing box spread data: %s", e)
        return (None, None)
