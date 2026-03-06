"""
Optional 1Password SDK helper for resolving op:// secret references in-process.

Use when you want to load secrets from 1Password without using the CLI (op) or shell wrappers.

Install (optional): from the python/ directory:
  uv sync --extra onepassword
  # or: pip install .[onepassword]
  # or: pip install onepassword-sdk

Auth: OP_SERVICE_ACCOUNT_TOKEN (automation) or 1Password desktop app (OP_1PASSWORD_ACCOUNT_NAME).
When the SDK fails to authenticate, the helper falls back to the op CLI (op vault list, op read)
so token-valid + CLI works still gives Auth: ready and op:// resolution.
Docs: https://developer.1password.com/docs/sdks/
"""

from __future__ import annotations

import asyncio
import os
import subprocess
from typing import Optional, Tuple

_client = None


def _cli_available() -> bool:
    """Return True if OP_SERVICE_ACCOUNT_TOKEN is set and op vault list succeeds."""
    token = os.getenv("OP_SERVICE_ACCOUNT_TOKEN", "").strip()
    if not token:
        return False
    env = os.environ.copy()
    env["OP_SERVICE_ACCOUNT_TOKEN"] = token
    env.pop("OP_CONNECT_HOST", None)
    env.pop("OP_CONNECT_TOKEN", None)
    try:
        r = subprocess.run(
            ["op", "vault", "list"],
            env=env,
            capture_output=True,
            timeout=10,
        )
        return r.returncode == 0
    except Exception:
        return False


def _is_op_ref(value: str) -> bool:
    """Return True if value looks like an op:// secret reference."""
    return bool(value and str(value).strip().startswith("op://"))


def validate_secret_reference(secret_ref: str) -> bool:
    """
    Validate op:// secret reference syntax. Returns True if valid, False otherwise.
    Uses SDK when available for precise errors; otherwise checks op:// prefix only.
    """
    if not _is_op_ref(secret_ref):
        return False
    if not sdk_available():
        return True
    try:
        from onepassword.secrets import Secrets
        Secrets.validate_secret_reference(secret_ref.strip())
        return True
    except Exception:
        return False


def _resolve_secret_via_cli(secret_ref: str) -> Optional[str]:
    """Resolve op:// ref using op read. Returns None on failure.
    Uses OP_SERVICE_ACCOUNT_TOKEN if set; otherwise runs with current env (e.g. OP_SESSION_* from op signin).
    """
    if not secret_ref or not _is_op_ref(secret_ref):
        return None
    env = os.environ.copy()
    token = os.getenv("OP_SERVICE_ACCOUNT_TOKEN", "").strip()
    if token:
        env["OP_SERVICE_ACCOUNT_TOKEN"] = token
    env.pop("OP_CONNECT_HOST", None)
    env.pop("OP_CONNECT_TOKEN", None)
    try:
        r = subprocess.run(
            ["op", "read", secret_ref.strip()],
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
        )
        if r.returncode == 0 and r.stdout is not None:
            return r.stdout.strip()
    except Exception:
        pass
    return None


def sdk_available() -> bool:
    """Return True if the 1Password Python SDK is installed."""
    try:
        from onepassword.client import Client  # noqa: F401
        return True
    except ImportError:
        return False


def client_available() -> bool:
    """
    Return True if 1Password can be used: SDK auth succeeds, or OP_SERVICE_ACCOUNT_TOKEN
    is set and op vault list succeeds (CLI fallback when SDK auth fails).
    """
    if os.getenv("OP_1PASSWORD_ACCOUNT_NAME"):
        if not sdk_available():
            return False
        try:
            client = asyncio.run(_get_client_async())
            return client is not None
        except Exception:
            return False
    token = os.getenv("OP_SERVICE_ACCOUNT_TOKEN", "").strip()
    if not token:
        return False
    # Prefer SDK
    if sdk_available():
        try:
            client = asyncio.run(_get_client_async())
            if client is not None:
                return True
        except Exception:
            pass
    # Fallback: token valid for CLI
    return _cli_available()


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


def resolve_secret(secret_ref: str, validate: bool = False) -> Optional[str]:
    """
    Resolve a single op:// secret reference. Tries SDK first; if that fails, uses op read (CLI).
    Returns None if ref is not op://, or both SDK and CLI fail.
    If validate=True and ref is invalid syntax, returns None (no network call).
    """
    if not secret_ref or not _is_op_ref(secret_ref):
        return None
    ref = secret_ref.strip()
    if validate and not validate_secret_reference(ref):
        return None

    async def _resolve_sdk():
        client = await _get_client_async()
        if not client:
            return None
        try:
            return await client.secrets.resolve(ref)
        except Exception:
            return None

    try:
        value = asyncio.run(_resolve_sdk())
        if value is not None:
            return value
    except Exception:
        pass
    return _resolve_secret_via_cli(ref)


def resolve_secrets(refs: dict[str, str]) -> dict[str, str]:
    """
    Resolve multiple op:// refs. refs maps output_key -> op:// ref or plain value.
    Returns dict of output_key -> resolved value (plain values passed through, op:// resolved).
    When SDK is available and multiple refs are op://, uses resolve_all for one round-trip.
    """
    result = {}
    op_refs: list[tuple[str, str]] = []  # (key, ref)
    for key, ref in refs.items():
        if ref and _is_op_ref(ref):
            op_refs.append((key, ref.strip()))
        elif ref:
            result[key] = ref

    if not op_refs:
        return result

    # Prefer SDK resolve_all when we have at least one op:// ref (one round-trip)
    if len(op_refs) >= 1 and sdk_available():
        async def _resolve_all_sdk():
            client = await _get_client_async()
            if not client or not op_refs:
                return {}
            ref_list = [r for _, r in op_refs]
            try:
                batch = await client.secrets.resolve_all(ref_list)
                responses = getattr(batch, "individual_responses", None)
                if responses is None:
                    return {}
                out = {}
                vals = list(responses.values()) if hasattr(responses, "values") else list(responses)
                for i, (key, _ref) in enumerate(op_refs):
                    if i >= len(vals):
                        break
                    resp = vals[i]
                    if getattr(resp, "content", None) is not None and getattr(resp.content, "secret", None) is not None:
                        out[key] = resp.content.secret
                return out
            except Exception:
                return {}

        try:
            batch_result = asyncio.run(_resolve_all_sdk())
            for key, val in batch_result.items():
                result[key] = val
            for key, ref in op_refs:
                if key not in result:
                    value = resolve_secret(ref)
                    if value is not None:
                        result[key] = value
            return result
        except Exception:
            pass

    # Fallback: resolve one by one
    for key, ref in op_refs:
        value = resolve_secret(ref)
        if value is not None:
            result[key] = value
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


def _op_run(args: list[str], timeout: int = 10) -> Optional[str]:
    """Run op CLI. Uses OP_SERVICE_ACCOUNT_TOKEN if set; otherwise current env (e.g. op signin session).
    Returns stdout or None on failure."""
    env = os.environ.copy()
    token = os.getenv("OP_SERVICE_ACCOUNT_TOKEN", "").strip()
    if token:
        env["OP_SERVICE_ACCOUNT_TOKEN"] = token
    env.pop("OP_CONNECT_HOST", None)
    env.pop("OP_CONNECT_TOKEN", None)
    try:
        r = subprocess.run(args, capture_output=True, text=True, timeout=timeout, env=env)
        if r.returncode == 0 and r.stdout:
            return r.stdout.strip()
    except Exception:
        pass
    return None


def get_credentials_from_1password_item(
    title_contains: str,
    key_field_labels: Tuple[str, ...] = ("username", "API Key ID", "Key ID"),
    secret_field_labels: Tuple[str, ...] = ("credential", "API Secret Key", "Secret Key", "password"),
) -> Optional[Tuple[str, str]]:
    """
    Discover a 1Password item by title (e.g. "alpaca"), read its key/secret fields, and resolve via SDK/CLI.
    Returns (key_id, secret) or None. Uses op CLI for item list/get; resolve_secret() for values (SDK when available).
    """
    import json
    out = _op_run(["op", "item", "list", "--format=json"])
    if not out:
        return None
    try:
        items = json.loads(out)
    except Exception:
        return None
    for it in items:
        title = (it.get("title") or "").strip()
        if title_contains.lower() not in title.lower():
            continue
        vault = it.get("vault") or {}
        vault_name = vault.get("name") or ""
        item_id = it.get("id")
        if not item_id or not vault_name:
            continue
        detail_out = _op_run(["op", "item", "get", item_id, "--format=json"])
        if not detail_out:
            continue
        try:
            detail = json.loads(detail_out)
        except Exception:
            continue
        fields = detail.get("fields") or []
        key_ref = None
        secret_ref = None
        for f in fields:
            lab = (f.get("label") or "").strip()
            purpose = (f.get("purpose") or "").strip()
            ref = (f.get("reference") or "").strip()
            if not ref or not ref.startswith("op://"):
                continue
            if purpose == "USERNAME" or lab in key_field_labels:
                key_ref = ref
            elif purpose == "PASSWORD" or (f.get("type") == "CONCEALED" and lab in secret_field_labels):
                secret_ref = ref
            if key_ref and secret_ref:
                break
        if not key_ref or not secret_ref:
            continue
        key_val = resolve_secret(key_ref)
        secret_val = resolve_secret(secret_ref)
        if key_val and secret_val:
            return (key_val.strip(), secret_val.strip())
    return None


def get_alpaca_credentials_from_1password() -> Optional[Tuple[str, str]]:
    """
    If 1Password SDK or CLI is available, find an Alpaca item (title contains "alpaca")
    and return (api_key_id, api_secret_key). Returns None if not found or auth fails.
    Allows TUI and Alpaca service to use the vault entry without exporting OP_* env vars.
    """
    if not client_available():
        return None
    return get_credentials_from_1password_item(
        "alpaca",
        key_field_labels=("username", "API Key ID", "Key ID"),
        secret_field_labels=("credential", "API Secret Key", "Secret Key", "password"),
    )


def get_secret_from_1password_item(
    title_contains: str,
    field_labels: Tuple[str, ...] = ("credential", "API Key", "password", "secret"),
) -> Optional[str]:
    """
    Discover a 1Password item by title and return a single secret field (e.g. API key).
    Uses op CLI for list/get; resolve_secret() for value (SDK when available).
    """
    import json
    out = _op_run(["op", "item", "list", "--format=json"])
    if not out:
        return None
    try:
        items = json.loads(out)
    except Exception:
        return None
    for it in items:
        title = (it.get("title") or "").strip()
        if title_contains.lower() not in title.lower():
            continue
        vault = it.get("vault") or {}
        vault_name = vault.get("name") or ""
        item_id = it.get("id")
        if not item_id or not vault_name:
            continue
        detail_out = _op_run(["op", "item", "get", item_id, "--format=json"])
        if not detail_out:
            continue
        try:
            detail = json.loads(detail_out)
        except Exception:
            continue
        for f in detail.get("fields") or []:
            lab = (f.get("label") or "").strip()
            purpose = (f.get("purpose") or "").strip()
            ref = (f.get("reference") or "").strip()
            if not ref or not ref.startswith("op://"):
                continue
            is_secret = (
                purpose == "PASSWORD"
                or f.get("type") == "CONCEALED"
                or lab in field_labels
            )
            if is_secret:
                val = resolve_secret(ref)
                if val:
                    return val.strip()
    return None


def get_fred_api_key_from_1password() -> Optional[str]:
    """
    Find a FRED API item in 1Password (title contains "fred") and return its API key.
    Uses op CLI for discovery (op item list/get); resolve_secret for value (SDK or op read).
    Works with OP_SERVICE_ACCOUNT_TOKEN or an op signin session in the current env.
    """
    return get_secret_from_1password_item(
        "fred",
        field_labels=("credential", "API Key", "password", "secret"),
    )


def get_alpha_vantage_api_key_from_1password() -> Optional[str]:
    """
    Find an Alpha Vantage API item in 1Password (title contains "alpha vantage") and return its API key.
    Uses op CLI for discovery; resolve_secret for value (SDK or op read).
    """
    return get_secret_from_1password_item(
        "alpha vantage",
        field_labels=("credential", "API Key", "password", "secret"),
    )


def get_finnhub_api_key_from_1password() -> Optional[str]:
    """
    Find a Finnhub API item in 1Password (title contains "finnhub") and return its API key.
    Uses op CLI for discovery; resolve_secret for value (SDK or op read).
    """
    return get_secret_from_1password_item(
        "finnhub",
        field_labels=("credential", "API Key", "password", "secret"),
    )
