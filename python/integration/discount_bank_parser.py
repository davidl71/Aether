"""File parsing for Discount Bank reconciliation (.dat) files.

The canonical parser is the Rust implementation in
agents/backend/crates/discount_bank_parser/. Prefer parse_file_via_rust();
it invokes the Rust binary with --json. The pure-Python path (read_balance_from_file)
is deprecated and used only when the Rust binary is unavailable.
See docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md Phase 6.
"""

from __future__ import annotations

import json
import logging
import subprocess
import warnings
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

from .discount_bank_models import TransactionResponse

logger = logging.getLogger(__name__)


def find_latest_file(file_path: str) -> Optional[Path]:
    """Find the latest Discount Bank file if path is a directory or pattern."""
    path = Path(file_path).expanduser()

    if path.is_file():
        return path

    if path.is_dir():
        files = list(path.glob("DISCOUNT*.dat")) + list(path.glob("DISCOUNT*.DAT"))
        if files:
            return max(files, key=lambda p: p.stat().st_mtime)

    parent = path.parent
    pattern = path.name
    if parent.exists():
        files = list(parent.glob(pattern))
        if files:
            return max(files, key=lambda p: p.stat().st_mtime)

    return None


def parse_file_via_rust(file_path: Path) -> Dict[str, Any]:
    """Parse Discount Bank file using Rust parser, falling back to Python."""
    root_dir = Path(__file__).parent.parent.parent
    rust_binary = (
        root_dir / "agents" / "backend" / "target" / "debug"
        / "examples" / "show_balances"
    )

    def _try_run(cmd: list, cwd: str) -> Optional[Dict[str, Any]]:
        try:
            result = subprocess.run(
                cmd, cwd=cwd, capture_output=True, text=True, timeout=15,
            )
            if result.returncode == 0 and result.stdout.strip():
                return json.loads(result.stdout)
        except (subprocess.TimeoutExpired, FileNotFoundError, json.JSONDecodeError) as exc:
            logger.debug("Rust parser attempt failed: %s", exc)
        return None

    if rust_binary.exists():
        parsed = _try_run(
            [str(rust_binary), "--json", str(file_path)],
            cwd=str(root_dir / "agents" / "backend"),
        )
        if parsed:
            return parsed

    cargo_toml = root_dir / "agents" / "backend" / "Cargo.toml"
    if cargo_toml.exists():
        parsed = _try_run(
            ["cargo", "run", "--example", "show_balances",
             "--manifest-path", str(cargo_toml), "--", "--json", str(file_path)],
            cwd=str(root_dir / "agents" / "backend"),
        )
        if parsed:
            return parsed

    logger.info("Rust parser unavailable, using Python fallback for %s", file_path)
    return read_balance_from_file(file_path)


def read_balance_from_file(file_path: Path) -> Dict[str, Any]:
    """Read balance from Discount Bank file (simplified header parser).

    DEPRECATED: Prefer parse_file_via_rust() which uses the Rust parser.
    This pure-Python path is kept only when the Rust binary is unavailable.
    """
    warnings.warn(
        "read_balance_from_file is deprecated; use parse_file_via_rust() and the Rust discount bank parser (see docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md Phase 6).",
        DeprecationWarning,
        stacklevel=2,
    )
    with open(file_path, "rb") as f:
        content = f.read()

    try:
        text = content.decode("utf-8")
    except UnicodeDecodeError:
        text = content.decode("windows-1255", errors="replace")

    lines = text.splitlines()
    last_header = None
    for line in lines:
        line_str = line.decode("utf-8") if isinstance(line, bytes) else str(line).strip()
        if line_str.startswith("00") and len(line_str) >= 54:
            last_header = line_str

    if not last_header:
        raise ValueError("No header record found in file")

    branch = last_header[3:6].strip()
    section = last_header[6:10].strip()
    account = last_header[12:18].strip()
    currency_code = last_header[10:12].strip()

    closing_str = last_header[33:47].strip()
    closing_sign = last_header[47] if len(last_header) > 47 else " "
    closing_int = int(closing_str) if closing_str else 0
    closing_balance = closing_int / 100.0
    if closing_sign == "-":
        closing_balance = -closing_balance

    date_str = last_header[48:54] if len(last_header) >= 54 else ""
    if date_str:
        year = 2000 + int(date_str[0:2])
        month = int(date_str[2:4])
        day = int(date_str[4:6])
        balance_date = f"{year}-{month:02d}-{day:02d}"
    else:
        balance_date = datetime.now().strftime("%Y-%m-%d")

    return {
        "account": f"{branch}-{section}-{account}",
        "balance": closing_balance,
        "currency": "ILS" if currency_code == "01" else currency_code,
        "balance_date": balance_date,
        "branch_number": branch,
        "section_number": section,
        "account_number": account,
    }


def parse_transactions_from_file(
    file_path: Path, limit: int = 20
) -> List[TransactionResponse]:
    """Parse transaction records from Discount Bank reconciliation file."""
    try:
        raw = file_path.read_bytes()
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            text = raw.decode("windows-1255", errors="replace")

        transactions: List[TransactionResponse] = []
        for line in text.splitlines():
            line_str = line.strip() if isinstance(line, str) else line.decode("utf-8", errors="replace").strip()
            if not line_str.startswith("01") or len(line_str) < 26:
                continue

            date_raw = line_str[2:10].strip()
            if len(date_raw) == 8:
                if int(date_raw[:4]) > 1900:
                    value_date = f"{date_raw[:4]}-{date_raw[4:6]}-{date_raw[6:8]}"
                else:
                    value_date = f"20{date_raw[4:6]}-{date_raw[2:4]}-{date_raw[0:2]}"
            else:
                value_date = date_raw

            amount_str = line_str[10:25].strip()
            try:
                amount_int = int(amount_str)
                amount = abs(amount_int) / 100.0
            except ValueError:
                continue

            sign_char = line_str[25] if len(line_str) > 25 else " "
            is_debit = sign_char != "+"
            reference = line_str[26:66].strip() if len(line_str) > 26 else ""

            transactions.append(
                TransactionResponse(
                    value_date=value_date,
                    amount=amount,
                    is_debit=is_debit,
                    reference=reference,
                )
            )

        transactions.reverse()
        return transactions[:limit]
    except Exception:
        return []
