"""
discount_bank_helpers.py - Legacy parser/ledger helpers for Discount Bank data.

The public Discount Bank HTTP surface is now owned by the Rust API. These
helpers remain only for parser-level Python tests and any temporary internal
analysis code that still imports them directly.
"""

from __future__ import annotations

import json
import os
import sqlite3
import subprocess
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional


def _find_latest_file(file_path: str) -> Optional[Path]:
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


def _read_balance_from_file(file_path: Path) -> Dict[str, Any]:
    try:
        with open(file_path, "rb") as f:
            content = f.read()

        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError:
            text = content.decode("windows-1255", errors="replace")

        lines = text.splitlines()
        last_header = None
        for line in lines:
            line_str = (
                line.decode("utf-8") if isinstance(line, bytes) else str(line).strip()
            )
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
    except Exception as e:
        raise ValueError(f"Failed to parse file: {e}") from e


def _parse_file_via_rust(file_path: Path) -> Dict[str, Any]:
    root_dir = Path(__file__).parent.parent.parent
    rust_binary = (
        root_dir / "agents" / "backend" / "target" / "debug" / "examples" / "show_balances"
    )

    def _try_run(cmd: list[str], cwd: str) -> Optional[Dict[str, Any]]:
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=15,
            )
            if result.returncode == 0 and result.stdout.strip():
                return json.loads(result.stdout)
        except (subprocess.TimeoutExpired, FileNotFoundError, json.JSONDecodeError):
            return None
        return None

    if rust_binary.exists():
        parsed = _try_run(
            [str(rust_binary), str(file_path)],
            cwd=str(root_dir / "agents" / "backend"),
        )
        if parsed:
            return parsed

    cargo_toml = root_dir / "agents" / "backend" / "Cargo.toml"
    if cargo_toml.exists():
        parsed = _try_run(
            [
                "cargo",
                "run",
                "--example",
                "show_balances",
                "--manifest-path",
                str(cargo_toml),
                "--",
                str(file_path),
            ],
            cwd=str(root_dir / "agents" / "backend"),
        )
        if parsed:
            return parsed

    return _read_balance_from_file(file_path)


def _get_ledger_database_path() -> Optional[Path]:
    db_path = os.getenv("LEDGER_DATABASE_PATH")
    if db_path:
        return Path(db_path).expanduser()

    root_dir = Path(__file__).parent.parent.parent
    default_paths = [
        root_dir / "ledger.db",
        root_dir / "agents" / "backend" / "ledger.db",
        Path.home() / ".ledger" / "ledger.db",
    ]

    for path in default_paths:
        if path.exists():
            return path
    return None


def _open_ledger_readonly():
    db_path = _get_ledger_database_path()
    if not db_path or not db_path.exists():
        return None
    try:
        uri = f"file:{db_path.as_posix()}?mode=ro"
        conn = sqlite3.connect(uri, uri=True)
        conn.row_factory = sqlite3.Row
        return conn
    except sqlite3.Error:
        return None


def _get_ledger_positions() -> Dict[str, Dict[str, Any]]:
    conn = _open_ledger_readonly()
    if not conn:
        return {}

    try:
        cursor = conn.cursor()
        cursor.execute("SELECT id, transaction_json FROM transactions")

        position_balances: Dict[str, Dict[str, float]] = defaultdict(lambda: defaultdict(float))
        position_currencies: Dict[str, str] = {}

        for row in cursor.fetchall():
            transaction_json = row["transaction_json"]
            if not transaction_json:
                continue
            try:
                transaction = json.loads(transaction_json)
                postings = transaction.get("postings", [])
                for posting in postings:
                    account_path_str = posting.get("account", "")
                    if isinstance(account_path_str, dict):
                        segments_list = account_path_str.get("segments", [])
                        if segments_list:
                            account_path_str = ":".join(segments_list)
                        else:
                            account_path_str = account_path_str.get("to_string", "") or str(account_path_str)

                    if not account_path_str or not account_path_str.startswith("Assets:IBKR:"):
                        continue

                    segments = account_path_str.split(":")
                    if len(segments) < 3:
                        continue

                    symbol = segments[2]
                    amount_data = posting.get("amount", {})
                    if isinstance(amount_data, dict):
                        amount = float(amount_data.get("amount", 0.0))
                        currency = amount_data.get("currency", "USD")
                    else:
                        amount = float(amount_data) if amount_data else 0.0
                        currency = "USD"

                    if symbol not in position_currencies:
                        position_currencies[symbol] = currency
                    position_balances[symbol][currency] += amount
            except (json.JSONDecodeError, KeyError, ValueError, TypeError):
                continue

        conn.close()

        positions = {}
        for symbol, balances in position_balances.items():
            currency = position_currencies.get(symbol, "USD")
            balance = balances.get(currency, 0.0)
            positions[symbol] = {
                "quantity": balance,
                "currency": currency,
                "account_path": f"Assets:IBKR:{symbol}",
            }
        return positions
    except sqlite3.Error:
        return {}


def _extract_bank_accounts_from_ledger() -> List[Dict[str, Any]]:
    conn = _open_ledger_readonly()
    if not conn:
        return []

    try:
        cursor = conn.cursor()
        cursor.execute("SELECT id, transaction_json FROM transactions")

        account_balances: Dict[str, Dict[str, float]] = defaultdict(lambda: defaultdict(float))
        account_currencies: Dict[str, str] = {}

        for row in cursor.fetchall():
            transaction_json = row["transaction_json"]
            if not transaction_json:
                continue
            try:
                transaction = json.loads(transaction_json)
                postings = transaction.get("postings", [])

                for posting in postings:
                    account_path_str = posting.get("account", "")
                    if isinstance(account_path_str, dict):
                        segments_list = account_path_str.get("segments", [])
                        if segments_list:
                            account_path_str = ":".join(segments_list)
                        else:
                            account_path_str = account_path_str.get("to_string", "") or str(account_path_str)

                    if not account_path_str or not account_path_str.startswith("Assets:Bank:"):
                        continue

                    segments = account_path_str.split(":")
                    bank_name = segments[2] if len(segments) > 2 else None
                    account_number = segments[3] if len(segments) > 3 else None

                    amount_data = posting.get("amount", {})
                    if isinstance(amount_data, dict):
                        amount = float(amount_data.get("amount", 0.0))
                        currency = amount_data.get("currency", "USD")
                    else:
                        amount = float(amount_data) if amount_data else 0.0
                        currency = "USD"

                    if account_path_str not in account_currencies:
                        account_currencies[account_path_str] = currency
                    account_balances[account_path_str][currency] += amount
            except (json.JSONDecodeError, KeyError, ValueError, TypeError):
                continue

        conn.close()

        bank_accounts = []
        for account_path, balances in account_balances.items():
            segments = account_path.split(":")
            bank_name = segments[2] if len(segments) > 2 else "Unknown"
            account_number = segments[3] if len(segments) > 3 else None

            currency_balances = {
                currency_code: amount
                for currency_code, amount in sorted(balances.items())
            }
            is_mixed_currency = len(currency_balances) > 1

            if is_mixed_currency:
                currency = "MULTI"
                balance = 0.0
            else:
                currency = account_currencies.get(account_path, "USD")
                balance = currency_balances.get(currency, 0.0)

            credit_rate = None
            debit_rate = None
            if bank_name and bank_name.lower() == "discount":
                credit_rate = 0.03
                debit_rate = 0.103

            bank_accounts.append(
                {
                    "account_path": account_path,
                    "account_name": account_path.split(":")[-1] if ":" in account_path else account_path,
                    "bank_name": bank_name,
                    "account_number": account_number,
                    "balance": balance,
                    "currency": currency,
                    "balances_by_currency": currency_balances if is_mixed_currency else None,
                    "is_mixed_currency": is_mixed_currency,
                    "credit_rate": credit_rate,
                    "debit_rate": debit_rate,
                }
            )

        bank_accounts.sort(key=lambda x: x["account_path"])
        return bank_accounts
    except sqlite3.Error:
        return []


def _parse_transactions_from_file(file_path: Path, limit: int = 20) -> List[Dict[str, Any]]:
    try:
        raw = file_path.read_bytes()
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            text = raw.decode("windows-1255", errors="replace")

        transactions: List[Dict[str, Any]] = []
        for line in text.splitlines():
            line_str = (
                line.strip()
                if isinstance(line, str)
                else line.decode("utf-8", errors="replace").strip()
            )
            if not line_str.startswith("01") or len(line_str) < 26:
                continue

            date_raw = line_str[2:10].strip()
            value_date = _parse_transaction_date(date_raw)
            if value_date is None:
                continue

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
                {
                    "value_date": value_date,
                    "amount": amount,
                    "is_debit": is_debit,
                    "reference": reference,
                }
            )

        transactions.reverse()
        return transactions[:limit]
    except Exception:
        return []


def _parse_transaction_date(date_raw: str) -> Optional[str]:
    if len(date_raw) != 8 or not date_raw.isdigit():
        return None

    try:
        year_prefix = int(date_raw[:4])
        if year_prefix >= 1900:
            parsed = datetime.strptime(date_raw, "%Y%m%d")
        else:
            parsed = datetime.strptime(date_raw, "%d%m%Y")
    except ValueError:
        return None

    return parsed.strftime("%Y-%m-%d")


def _fetch_ibkr_positions(account_id: Optional[str] = None) -> List[Dict[str, Any]]:
    try:
        from .ibkr_portal_client import IBKRPortalClient

        client = IBKRPortalClient()
        positions = client.get_portfolio_positions(account_id)

        formatted = []
        for pos in positions:
            if isinstance(pos, dict):
                formatted.append(
                    {
                        "symbol": pos.get("ticker", ""),
                        "quantity": float(pos.get("position", 0.0)),
                        "avg_price": float(pos.get("averageCost", 0.0)),
                        "current_price": float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0)),
                        "market_value": float(pos.get("markValue", 0.0)),
                        "unrealized_pl": float(pos.get("unrealizedPnl", 0.0)),
                        "currency": pos.get("currency", "USD"),
                    }
                )
        return formatted
    except Exception:
        return []
