"""
Optional 1Password SDK helper for resolving op:// secret references in-process.

Use when you want to load secrets from 1Password without using the CLI (op) or shell wrappers.
Requires: pip install onepassword-sdk  (or uv add onepassword-sdk)

Auth: OP_SERVICE_ACCOUNT_TOKEN (automation) or 1Password desktop app (OP_1PASSWORD_ACCOUNT_NAME).
Docs: https://developer.1password.com/docs/sdks/
"""

from __future__ import annotations

import asyncio
import os
from typing import Optional

_client = None


async def _get_client_async():
    """Create or return cached 1Password client. Returns None if SDK unavailable or auth missing."""
    global _client
    if _client is not None:
        return _client
    try:
        from onepassword.client import Client
        from onepassword.client import DesktopAuth
    except ImportError:
        return None

    token = os.getenv("OP_SERVICE_ACCOUNT_TOKEN")
    account_name = os.getenv("OP_1PASSWORD_ACCOUNT_NAME")

    if token:
        _client = await Client.authenticate(
            auth=token,
            integration_name="ib_box_spread",
            integration_version="1.0.0",
        )
    elif account_name:
        _client = await Client.authenticate(
            auth=DesktopAuth(account_name=account_name),
            integration_name="ib_box_spread",
            integration_version="1.0.0",
        )
    else:
        return None
    return _client


def resolve_secret(secret_ref: str) -> Optional[str]:
    """
    Resolve a single op:// secret reference. Returns None if ref is not op://, SDK unavailable, or resolve fails.
    """
    if not secret_ref or not str(secret_ref).strip().startswith("op://"):
        return None

    async def _resolve():
        client = await _get_client_async()
        if not client:
            return None
        try:
            return await client.secrets.resolve(secret_ref)
        except Exception:
            return None

    try:
        return asyncio.run(_resolve())
    except Exception:
        return None


def resolve_secrets(refs: dict[str, str]) -> dict[str, str]:
    """
    Resolve multiple op:// refs. refs maps output_key -> op:// ref or plain value.
    Returns dict of output_key -> resolved value (plain values passed through, op:// resolved).
    """
    result = {}
    for key, ref in refs.items():
        if ref and str(ref).strip().startswith("op://"):
            value = resolve_secret(ref)
            if value is not None:
                result[key] = value
        elif ref:
            result[key] = ref
    return result


def getenv_or_resolve(env_var: str, op_ref_env_var: str, default: str = "") -> str:
    """
    Return os.getenv(env_var) if set; otherwise if op_ref_env_var is set and is an op:// ref,
    resolve it via 1Password SDK and return the value. Else return default.
    Use in clients so credentials can come from env or from 1Password refs when SDK is available.
    """
    value = os.getenv(env_var, "").strip()
    if value:
        return value
    ref = os.getenv(op_ref_env_var, "").strip()
    if not ref:
        return default
    resolved = resolve_secret(ref)
    return (resolved or default).strip()
