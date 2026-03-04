"""Ledger database queries for bank accounts and positions."""

from __future__ import annotations

import json
import os
import sqlite3
import uuid
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


def get_ledger_database_path() -> Optional[Path]:
    """Get ledger database path from environment or default location."""
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


def _parse_account_path(posting: Dict) -> str:
    """Extract account path string from a posting dict."""
    account_path_str = posting.get("account", "")
    if isinstance(account_path_str, dict):
        segments_list = account_path_str.get("segments", [])
        if segments_list:
            return ":".join(segments_list)
        return account_path_str.get("to_string", "") or str(account_path_str)
    return account_path_str


def _sum_posting_balances(
    cursor: sqlite3.Cursor,
    account_prefix: str,
) -> tuple[Dict[str, Dict[str, float]], Dict[str, str]]:
    """Sum posting amounts grouped by account path and currency.

    Returns (balances, currencies) where:
      balances[account_path][currency] = total
      currencies[account_path] = primary_currency
    """
    cursor.execute("SELECT id, transaction_json FROM transactions")

    balances: Dict[str, Dict[str, float]] = defaultdict(lambda: defaultdict(float))
    currencies: Dict[str, str] = {}

    for row in cursor.fetchall():
        txn_json = row["transaction_json"]
        if not txn_json:
            continue
        try:
            txn = json.loads(txn_json)
            for posting in txn.get("postings", []):
                acct = _parse_account_path(posting)
                if not acct or not acct.startswith(account_prefix):
                    continue

                amount_data = posting.get("amount", {})
                if isinstance(amount_data, dict):
                    amount = float(amount_data.get("amount", 0.0))
                    currency = amount_data.get("currency", "USD")
                else:
                    amount = float(amount_data) if amount_data else 0.0
                    currency = "USD"

                if acct not in currencies:
                    currencies[acct] = currency
                balances[acct][currency] += amount
        except (json.JSONDecodeError, KeyError, ValueError, TypeError):
            continue

    return dict(balances), currencies


def get_ledger_positions() -> Dict[str, Dict[str, Any]]:
    """Get existing positions from ledger database (Assets:IBKR:*)."""
    db_path = get_ledger_database_path()
    if not db_path or not db_path.exists():
        return {}

    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        balances, currencies = _sum_posting_balances(conn.cursor(), "Assets:IBKR:")
        conn.close()

        positions = {}
        for acct, cur_balances in balances.items():
            segments = acct.split(":")
            if len(segments) < 3:
                continue
            symbol = segments[2]
            currency = currencies.get(acct, "USD")
            positions[symbol] = {
                "quantity": cur_balances.get(currency, 0.0),
                "currency": currency,
                "account_path": acct,
            }
        return positions
    except sqlite3.Error:
        return {}


def extract_bank_accounts_from_ledger() -> List[Dict[str, Any]]:
    """Extract all bank accounts from ledger database (Assets:Bank:*)."""
    db_path = get_ledger_database_path()
    if not db_path or not db_path.exists():
        return []

    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        balances, currencies = _sum_posting_balances(conn.cursor(), "Assets:Bank:")
        conn.close()

        accounts = []
        for acct, cur_balances in balances.items():
            segments = acct.split(":")
            bank_name = segments[2] if len(segments) > 2 else "Unknown"
            account_number = segments[3] if len(segments) > 3 else None
            currency = currencies.get(acct, "USD")
            balance = cur_balances.get(currency, 0.0)

            credit_rate = None
            debit_rate = None
            if bank_name and bank_name.lower() == "discount":
                credit_rate = 0.03
                debit_rate = 0.103

            accounts.append({
                "account_path": acct,
                "account_name": acct.split(":")[-1] if ":" in acct else acct,
                "bank_name": bank_name,
                "account_number": account_number,
                "balance": balance,
                "currency": currency,
                "credit_rate": credit_rate,
                "debit_rate": debit_rate,
            })

        accounts.sort(key=lambda x: x["account_path"])
        return accounts
    except sqlite3.Error:
        return []


def record_position_in_ledger(
    symbol: str, quantity: float, price: float, currency: str, broker: str
) -> bool:
    """Record a position in the ledger database."""
    db_path = get_ledger_database_path()
    if not db_path or not db_path.exists():
        return False

    try:
        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()

        notional = abs(quantity) * price
        transaction_id = str(uuid.uuid4())
        transaction_date = datetime.now(timezone.utc).isoformat()
        description = (
            f"Buy {int(abs(quantity))} {symbol}" if quantity > 0
            else f"Sell {int(abs(quantity))} {symbol}"
        )

        position_account = f"Assets:IBKR:{symbol}"
        cash_account = "Assets:IBKR:Cash"

        if quantity > 0:
            postings = [
                {"account": position_account, "amount": {"amount": str(notional), "currency": currency}},
                {"account": cash_account, "amount": {"amount": f"-{notional}", "currency": currency}},
            ]
        else:
            postings = [
                {"account": cash_account, "amount": {"amount": str(notional), "currency": currency}},
                {"account": position_account, "amount": {"amount": f"-{notional}", "currency": currency}},
            ]

        txn_json = json.dumps({
            "id": transaction_id,
            "date": transaction_date,
            "description": description,
            "cleared": True,
            "postings": postings,
            "metadata": {
                "source": "position_import",
                "broker": broker,
                "symbol": symbol,
                "quantity": str(int(quantity)),
            },
        })

        cursor.execute(
            "INSERT INTO transactions (id, date, description, cleared, transaction_json, account_paths) VALUES (?, ?, ?, ?, ?, ?)",
            (transaction_id, transaction_date, description, 1, txn_json, f"{position_account}|{cash_account}"),
        )
        conn.commit()
        conn.close()
        return True
    except Exception:
        return False
