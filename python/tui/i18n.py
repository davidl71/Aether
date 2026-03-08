"""
Minimal i18n scaffold for the TUI. Loads string catalog by locale; defaults to English.
Add keys to LOCALES and use t(key) or t(key, locale=...) in TUI components.
"""
from __future__ import annotations

import os
from typing import Dict, Optional

# Default locale from env (e.g. LANG=he_IL.UTF-8 -> he)
_DEFAULT_LOCALE = (os.environ.get("LANG", "en").split("_")[0].split(".")[0]) or "en"

LOCALES: Dict[str, Dict[str, str]] = {
    "en": {
        "common.loading": "Loading…",
        "common.error": "Error",
        "common.close": "Close",
        "tabs.dashboard": "Dashboard",
        "tabs.positions": "Positions",
        "tabs.orders": "Orders",
        "tabs.alerts": "Alerts",
        "status.awaiting_snapshot": "Awaiting snapshot…",
    },
    "he": {
        "common.loading": "טוען…",
        "common.error": "שגיאה",
        "common.close": "סגור",
        "tabs.dashboard": "לוח בקרה",
        "tabs.positions": "פוזיציות",
        "tabs.orders": "הזמנות",
        "tabs.alerts": "התראות",
        "status.awaiting_snapshot": "ממתין לצילום מצב…",
    },
}


def t(key: str, locale: Optional[str] = None, default: Optional[str] = None) -> str:
    """Return translated string for key. Falls back to English then key itself."""
    loc = (locale or _DEFAULT_LOCALE).lower()
    for candidate in (loc, "en"):
        if candidate in LOCALES and key in LOCALES[candidate]:
            return LOCALES[candidate][key]
    return default if default is not None else key
