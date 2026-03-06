"""Display helpers for TUI: shorten endpoint/URL display when localhost."""

from __future__ import annotations

from urllib.parse import urlparse


def format_endpoint_display(url_or_endpoint: str) -> str:
    """
    If the endpoint is localhost or 127.0.0.1, return only port and protocol (e.g. "8002 (HTTP)", "7497 (TCP)").
    Otherwise return the original string.
    """
    if not url_or_endpoint or not isinstance(url_or_endpoint, str):
        return url_or_endpoint or ""
    s = url_or_endpoint.strip()
    if not s:
        return s

    # Already "port (TCP)" style
    if " (TCP)" in s:
        rest = s.replace(" (TCP)", "").strip()
        if rest in ("localhost", "127.0.0.1"):
            return s
        if ":" in rest:
            host, port = rest.rsplit(":", 1)
            host = (host or "").strip()
            if host.lower() in ("localhost", "127.0.0.1") and port.isdigit():
                return f"{port} (TCP)"
        return s

    # NATS URL: nats://localhost:4222
    if s.startswith("nats://"):
        try:
            parsed = urlparse(s)
            host = (parsed.hostname or "").strip() or ""
            port = parsed.port
            if host.lower() in ("localhost", "127.0.0.1") and port is not None:
                return f"{port} (NATS)"
        except Exception:
            pass
        return s

    # HTTP/HTTPS URL
    if "://" in s:
        try:
            parsed = urlparse(s)
            host = (parsed.hostname or "").strip() or ""
            port = parsed.port
            scheme = (parsed.scheme or "http").upper()
            if host.lower() in ("localhost", "127.0.0.1") and port is not None:
                return f"{port} ({scheme})"
        except Exception:
            pass
        return s

    # Bare host:port
    if ":" in s:
        host, port = s.rsplit(":", 1)
        host = (host or "").strip()
        port = (port or "").strip()
        if host.lower() in ("localhost", "127.0.0.1") and port.isdigit():
            return f"{port} (TCP)"
    return s
