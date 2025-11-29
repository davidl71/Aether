"""
discount_bank_service.py - FastAPI service for bank accounts (Discount Bank and others from ledger)

Endpoints:
- GET /api/health
- GET /api/balance (Discount Bank specific)
- GET /api/transactions (Discount Bank specific)
- GET /api/bank-accounts (All bank accounts from ledger)

Environment:
- DISCOUNT_BANK_FILE_PATH: Path to Discount Bank reconciliation file (default: ~/Downloads/DISCOUNT.dat)
- DISCOUNT_BANK_CREDIT_RATE: Credit interest rate (default: 0.03 = 3%)
- DISCOUNT_BANK_DEBIT_RATE: Debit interest rate (default: 0.103 = 10.30%)
- LEDGER_DATABASE_PATH: Path to ledger SQLite database (default: ledger.db in project root)
"""

from __future__ import annotations

import json
import os
import sqlite3
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Any, Optional
from collections import defaultdict

from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from pathlib import Path
import sys

# Add project root to path for security module
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware
)


class BalanceResponse(BaseModel):
    account: str
    balance: float
    currency: str
    balance_date: str
    credit_rate: float
    debit_rate: float
    branch_number: Optional[str] = None
    section_number: Optional[str] = None
    account_number: Optional[str] = None


class TransactionResponse(BaseModel):
    value_date: str
    amount: float
    is_debit: bool
    reference: str


class TransactionsResponse(BaseModel):
    account: str
    transactions: List[TransactionResponse]
    total_count: int


class BankAccount(BaseModel):
    account_path: str
    account_name: str
    bank_name: Optional[str] = None
    account_number: Optional[str] = None
    balance: float
    currency: str
    balance_date: Optional[str] = None
    credit_rate: Optional[float] = None
    debit_rate: Optional[float] = None


class BankAccountsResponse(BaseModel):
    accounts: List[BankAccount]
    total_count: int


class Position(BaseModel):
    symbol: str
    quantity: float
    avg_price: float
    current_price: Optional[float] = None
    market_value: Optional[float] = None
    unrealized_pl: Optional[float] = None
    currency: str
    broker: str
    in_ledger: bool
    ledger_account_path: Optional[str] = None


class ImportPositionsResponse(BaseModel):
    positions: List[Position]
    imported_count: int
    existing_count: int
    total_count: int


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _find_latest_file(file_path: str) -> Optional[Path]:
    """Find the latest Discount Bank file if path is a directory or pattern."""
    path = Path(file_path).expanduser()

    if path.is_file():
        return path

    if path.is_dir():
        # Look for DISCOUNT*.dat files in directory
        files = list(path.glob("DISCOUNT*.dat")) + list(path.glob("DISCOUNT*.DAT"))
        if files:
            # Return most recently modified
            return max(files, key=lambda p: p.stat().st_mtime)

    # Try as pattern
    parent = path.parent
    pattern = path.name
    if parent.exists():
        files = list(parent.glob(pattern))
        if files:
            return max(files, key=lambda p: p.stat().st_mtime)

    return None


def _parse_file_via_rust(file_path: Path) -> Dict[str, Any]:
    """Parse Discount Bank file using Rust parser via subprocess."""
    # Build path to Rust example binary
    root_dir = Path(__file__).parent.parent.parent
    rust_example = (
        root_dir
        / "agents"
        / "backend"
        / "target"
        / "debug"
        / "examples"
        / "show_balances"
    )

    # If binary doesn't exist, try to build it or use alternative approach
    if not rust_example.exists():
        # Alternative: try cargo run
        try:
            result = subprocess.run(
                [
                    "cargo",
                    "run",
                    "--example",
                    "show_balances",
                    "--manifest-path",
                    str(root_dir / "agents" / "backend" / "Cargo.toml"),
                    "--",
                    str(file_path),
                ],
                cwd=str(root_dir / "agents" / "backend"),
                capture_output=True,
                text=True,
                timeout=10,
            )
            if result.returncode == 0:
                # Parse output (would need to modify show_balances to output JSON)
                # For now, fall back to direct file reading
                pass
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass

    # For now, read file directly and parse basic info
    # TODO: Integrate with Rust parser properly
    raise NotImplementedError("Rust parser integration pending")


def _get_ledger_database_path() -> Optional[Path]:
    """Get ledger database path from environment or default location."""
    db_path = os.getenv("LEDGER_DATABASE_PATH")
    if db_path:
        return Path(db_path).expanduser()

    # Try default locations
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


def _get_ledger_positions() -> Dict[str, Dict[str, Any]]:
    """Get existing positions from ledger database.

    Returns dict mapping symbol to position info with balance.
    """
    db_path = _get_ledger_database_path()
    if not db_path or not db_path.exists():
        return {}

    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        cursor.execute(
            """
            SELECT id, transaction_json
            FROM transactions
            """
        )

        # Track position balances per symbol
        position_balances: Dict[str, Dict[str, float]] = defaultdict(
            lambda: defaultdict(float)
        )
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
                            account_path_str = account_path_str.get(
                                "to_string", ""
                            ) or str(account_path_str)

                    # Check if this is a position account (Assets:IBKR:{symbol})
                    if not account_path_str or not account_path_str.startswith(
                        "Assets:IBKR:"
                    ):
                        continue

                    # Extract symbol from account path
                    segments = account_path_str.split(":")
                    if len(segments) < 3:
                        continue

                    symbol = segments[2]  # Assets:IBKR:{symbol}

                    # Get amount
                    amount_data = posting.get("amount", {})
                    if isinstance(amount_data, dict):
                        amount = float(amount_data.get("amount", 0.0))
                        currency = amount_data.get("currency", "USD")
                    else:
                        amount = float(amount_data) if amount_data else 0.0
                        currency = "USD"

                    # Track currency
                    if symbol not in position_currencies:
                        position_currencies[symbol] = currency

                    # Sum balances (positive = long, negative = short)
                    position_balances[symbol][currency] += amount
            except (json.JSONDecodeError, KeyError, ValueError, TypeError):
                continue

        conn.close()

        # Format results
        positions = {}
        for symbol, balances in position_balances.items():
            currency = position_currencies.get(symbol, "USD")
            balance = balances.get(currency, 0.0)

            positions[symbol] = {
                "quantity": balance,  # This is the notional, not quantity - need to calculate
                "currency": currency,
                "account_path": f"Assets:IBKR:{symbol}",
            }

        return positions
    except sqlite3.Error:
        return {}


def _record_position_in_ledger(
    symbol: str, quantity: float, price: float, currency: str, broker: str
) -> bool:
    """Record a position in the ledger database.

    Creates a transaction similar to Rust record_position_change().
    Returns True if successful, False otherwise.
    """
    db_path = _get_ledger_database_path()
    if not db_path or not db_path.exists():
        return False

    try:
        import uuid
        from datetime import datetime, timezone

        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()

        # Calculate notional
        notional = abs(quantity) * price

        # Create transaction similar to Rust format
        transaction_id = str(uuid.uuid4())
        transaction_date = datetime.now(timezone.utc).isoformat()
        description = (
            f"Buy {int(abs(quantity))} {symbol}"
            if quantity > 0
            else f"Sell {int(abs(quantity))} {symbol}"
        )

        # Create postings
        position_account_path = f"Assets:IBKR:{symbol}"
        cash_account_path = "Assets:IBKR:Cash"

        # Format amount with currency
        amount_obj = {"amount": str(notional), "currency": currency}

        postings = []
        if quantity > 0:
            # Buying: Debit position, Credit cash
            postings.append(
                {
                    "account": position_account_path,
                    "amount": amount_obj,
                }
            )
            postings.append(
                {
                    "account": cash_account_path,
                    "amount": {"amount": f"-{notional}", "currency": currency},
                }
            )
        else:
            # Selling: Debit cash, Credit position
            postings.append(
                {
                    "account": cash_account_path,
                    "amount": amount_obj,
                }
            )
            postings.append(
                {
                    "account": position_account_path,
                    "amount": {"amount": f"-{notional}", "currency": currency},
                }
            )

        # Create transaction JSON
        transaction_json = json.dumps(
            {
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
            }
        )

        # Extract account paths for indexing
        account_paths = f"{position_account_path}|{cash_account_path}"

        # Insert transaction
        cursor.execute(
            """
            INSERT INTO transactions (
                id, date, description, cleared, transaction_json, account_paths
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            (
                transaction_id,
                transaction_date,
                description,
                1,  # cleared
                transaction_json,
                account_paths,
            ),
        )

        conn.commit()
        conn.close()
        return True
    except Exception as e:
        return False


def _fetch_ibkr_positions(account_id: Optional[str] = None) -> List[Dict[str, Any]]:
    """Fetch positions from IBKR Client Portal."""
    try:
        from .ibkr_portal_client import IBKRPortalClient, IBKRPortalError

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
                        "current_price": float(
                            pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0)
                        ),
                        "market_value": float(pos.get("markValue", 0.0)),
                        "unrealized_pl": float(pos.get("unrealizedPnl", 0.0)),
                        "currency": pos.get("currency", "USD"),
                    }
                )
        return formatted
    except Exception as e:
        return []


def _fetch_alpaca_positions() -> List[Dict[str, Any]]:
    """Fetch positions from Alpaca API."""
    try:
        from .alpaca_client import AlpacaClient

        client = AlpacaClient()
        positions = client.get_positions()

        formatted = []
        for pos in positions:
            if isinstance(pos, dict):
                formatted.append(
                    {
                        "symbol": pos.get("symbol", ""),
                        "quantity": float(pos.get("qty", 0.0)),
                        "avg_price": float(pos.get("avg_entry_price", 0.0)),
                        "current_price": float(pos.get("current_price", 0.0)),
                        "market_value": float(pos.get("market_value", 0.0)),
                        "unrealized_pl": float(pos.get("unrealized_pl", 0.0)),
                        "currency": pos.get("currency", "USD"),
                    }
                )
        return formatted
    except Exception:
        return []


def _extract_bank_accounts_from_ledger() -> List[Dict[str, Any]]:
    """Extract all bank accounts from ledger database."""
    db_path = _get_ledger_database_path()
    if not db_path or not db_path.exists():
        return []

    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        # Query all transactions and extract account paths from postings
        # The schema stores transactions with transaction_json containing postings
        cursor.execute(
            """
            SELECT id, transaction_json
            FROM transactions
        """
        )

        # Track balances per account
        account_balances: Dict[str, Dict[str, float]] = defaultdict(
            lambda: defaultdict(float)
        )
        account_currencies: Dict[str, str] = {}

        for row in cursor.fetchall():
            transaction_json = row["transaction_json"]
            if not transaction_json:
                continue

            try:
                transaction = json.loads(transaction_json)
                postings = transaction.get("postings", [])

                for posting in postings:
                    # Account path can be string or object with to_string method
                    account_path_str = posting.get("account", "")
                    if isinstance(account_path_str, dict):
                        # If account is an object, try to get segments or string representation
                        segments_list = account_path_str.get("segments", [])
                        if segments_list:
                            account_path_str = ":".join(segments_list)
                        else:
                            account_path_str = account_path_str.get(
                                "to_string", ""
                            ) or str(account_path_str)

                    if not account_path_str or not account_path_str.startswith(
                        "Assets:Bank:"
                    ):
                        continue

                    # Extract account info
                    segments = account_path_str.split(":")
                    if len(segments) >= 3:
                        bank_name = segments[2] if len(segments) > 2 else None
                        account_number = segments[3] if len(segments) > 3 else None
                    else:
                        bank_name = None
                        account_number = None

                    # Get amount
                    amount_data = posting.get("amount", {})
                    if isinstance(amount_data, dict):
                        amount = float(amount_data.get("amount", 0.0))
                        currency = amount_data.get("currency", "USD")
                    else:
                        # Fallback if amount is not a dict
                        amount = float(amount_data) if amount_data else 0.0
                        currency = "USD"

                    # Track currency
                    if account_path_str not in account_currencies:
                        account_currencies[account_path_str] = currency

                    # Sum balances (positive = credit, negative = debit)
                    account_balances[account_path_str][currency] += amount
            except (json.JSONDecodeError, KeyError, ValueError, TypeError) as e:
                # Skip invalid transactions/postings
                continue

        conn.close()

        # Format results
        bank_accounts = []
        for account_path, balances in account_balances.items():
            segments = account_path.split(":")
            bank_name = segments[2] if len(segments) > 2 else "Unknown"
            account_number = segments[3] if len(segments) > 3 else None

            # Get primary currency balance (use first currency found, or sum all if multiple)
            currency = account_currencies.get(account_path, "USD")
            balance = balances.get(currency, 0.0)

            # If multiple currencies, sum them (convert to primary currency)
            # For now, just use primary currency balance
            if len(balances) > 1:
                # Multiple currencies - use primary, but note this in metadata
                balance = balances.get(currency, 0.0)

            # Determine interest rates based on bank
            credit_rate = None
            debit_rate = None
            if bank_name and bank_name.lower() == "discount":
                credit_rate = 0.03
                debit_rate = 0.103

            bank_accounts.append(
                {
                    "account_path": account_path,
                    "account_name": (
                        account_path.split(":")[-1]
                        if ":" in account_path
                        else account_path
                    ),
                    "bank_name": bank_name,
                    "account_number": account_number,
                    "balance": balance,
                    "currency": currency,
                    "credit_rate": credit_rate,
                    "debit_rate": debit_rate,
                }
            )

        # Sort by account path for consistent ordering
        bank_accounts.sort(key=lambda x: x["account_path"])

        return bank_accounts
    except sqlite3.Error as e:
        # Database error - return empty list
        return []


def _read_balance_from_file(file_path: Path) -> Dict[str, Any]:
    """Read balance from Discount Bank file (simplified - reads last header)."""
    try:
        with open(file_path, "rb") as f:
            content = f.read()

        # Try to decode as UTF-8 or Windows-1255
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError:
            text = content.decode("windows-1255", errors="replace")

        # Find last header record (starts with "00")
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

        # Parse header (simplified - matches Rust parser logic)
        # Positions: 3=bank, 4-6=branch, 7-10=section, 11-12=currency, 13-18=account
        # 19-32=opening, 33=sign, 34-47=closing, 48=sign, 49-54=date
        branch = last_header[3:6].strip()
        section = last_header[6:10].strip()
        account = last_header[12:18].strip()
        currency_code = last_header[10:12].strip()

        # Parse closing balance (positions 34-47, sign at 48)
        closing_str = last_header[33:47].strip()
        closing_sign = last_header[47] if len(last_header) > 47 else " "
        closing_int = int(closing_str) if closing_str else 0
        closing_balance = closing_int / 100.0
        if closing_sign == "-":
            closing_balance = -closing_balance

        # Parse date (positions 49-54: YYMMDD)
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


app = FastAPI(title="Discount Bank Service")

# Add security components
security_components = add_security_to_app(app, project_root=project_root)
add_security_headers_middleware(app)


@app.get("/api/health")
def health() -> Dict[str, Any]:
    """Health check endpoint."""
    file_path = os.getenv("DISCOUNT_BANK_FILE_PATH", "~/Downloads/DISCOUNT.dat")
    latest_file = _find_latest_file(file_path)
    ledger_db = _get_ledger_database_path()

    return {
        "status": "ok",
        "service": "discount_bank",
        "file_path": str(file_path),
        "file_found": latest_file is not None,
        "file_path_resolved": str(latest_file) if latest_file else None,
        "ledger_database_path": str(ledger_db) if ledger_db else None,
        "ledger_database_found": (
            ledger_db is not None and ledger_db.exists() if ledger_db else False
        ),
    }


@app.get("/api/balance", response_model=BalanceResponse)
def get_balance() -> BalanceResponse:
    """Get current Discount Bank account balance."""
    file_path = os.getenv("DISCOUNT_BANK_FILE_PATH", "~/Downloads/DISCOUNT.dat")
    credit_rate = float(os.getenv("DISCOUNT_BANK_CREDIT_RATE", "0.03"))
    debit_rate = float(os.getenv("DISCOUNT_BANK_DEBIT_RATE", "0.103"))

    latest_file = _find_latest_file(file_path)
    if not latest_file or not latest_file.exists():
        raise HTTPException(
            status_code=404, detail=f"Discount Bank file not found: {file_path}"
        )

    try:
        balance_data = _read_balance_from_file(latest_file)
        return BalanceResponse(
            **balance_data,
            credit_rate=credit_rate,
            debit_rate=debit_rate,
        )
    except Exception as e:
        raise HTTPException(
            status_code=500, detail=f"Failed to read balance: {str(e)}"
        ) from e


@app.get("/api/transactions", response_model=TransactionsResponse)
def get_transactions(limit: int = 20) -> TransactionsResponse:
    """Get recent transactions from Discount Bank file."""
    file_path = os.getenv("DISCOUNT_BANK_FILE_PATH", "~/Downloads/DISCOUNT.dat")

    latest_file = _find_latest_file(file_path)
    if not latest_file or not latest_file.exists():
        raise HTTPException(
            status_code=404, detail=f"Discount Bank file not found: {file_path}"
        )

    # TODO: Parse transactions from file
    # For now, return empty list
    return TransactionsResponse(
        account="535-0000-276689",
        transactions=[],
        total_count=0,
    )


@app.get("/api/bank-accounts", response_model=BankAccountsResponse)
def get_bank_accounts() -> BankAccountsResponse:
    """Get all bank accounts from ledger database."""
    bank_accounts = _extract_bank_accounts_from_ledger()

    # Convert to response models
    accounts = [BankAccount(**account_data) for account_data in bank_accounts]

    return BankAccountsResponse(
        accounts=accounts,
        total_count=len(accounts),
    )


@app.get("/api/import-positions", response_model=ImportPositionsResponse)
def import_positions(
    broker: str = Query(..., description="Broker type: ibkr, alpaca, or tradestation"),
    account_id: Optional[str] = Query(None, description="Account ID (for IBKR)"),
    dry_run: bool = Query(False, description="If true, don't record in ledger"),
) -> ImportPositionsResponse:
    """Import positions from broker API into ledger.

    Fetches positions from the specified broker, checks if they exist in ledger,
    and optionally records new positions.
    """
    # Fetch positions from broker
    broker_positions = []
    if broker.lower() == "ibkr":
        broker_positions = _fetch_ibkr_positions(account_id)
    elif broker.lower() == "alpaca":
        broker_positions = _fetch_alpaca_positions()
    elif broker.lower() == "tradestation":
        # TODO: Implement TradeStation position fetching
        raise HTTPException(status_code=501, detail="TradeStation not yet implemented")
    else:
        raise HTTPException(
            status_code=400, detail=f"Unknown broker: {broker}. Use: ibkr, alpaca, tradestation"
        )

    # Get existing positions from ledger
    ledger_positions = _get_ledger_positions()

    # Process positions
    imported_count = 0
    existing_count = 0
    result_positions = []

    for pos in broker_positions:
        symbol = pos.get("symbol", "").upper()
        if not symbol:
            continue

        quantity = pos.get("quantity", 0.0)
        if quantity == 0.0:
            continue  # Skip zero positions

        avg_price = pos.get("avg_price", 0.0)
        currency = pos.get("currency", "USD")

        # Check if position exists in ledger
        in_ledger = symbol in ledger_positions
        ledger_account_path = f"Assets:IBKR:{symbol}" if in_ledger else None

        # Record in ledger if not dry run and not already in ledger
        if not dry_run and not in_ledger:
            if _record_position_in_ledger(symbol, quantity, avg_price, currency, broker):
                imported_count += 1
                in_ledger = True
                ledger_account_path = f"Assets:IBKR:{symbol}"
            else:
                # Failed to record, but still return position
                pass
        elif in_ledger:
            existing_count += 1

        result_positions.append(
            Position(
                symbol=symbol,
                quantity=quantity,
                avg_price=avg_price,
                current_price=pos.get("current_price"),
                market_value=pos.get("market_value"),
                unrealized_pl=pos.get("unrealized_pl"),
                currency=currency,
                broker=broker.lower(),
                in_ledger=in_ledger,
                ledger_account_path=ledger_account_path,
            )
        )

    return ImportPositionsResponse(
        positions=result_positions,
        imported_count=imported_count,
        existing_count=existing_count,
        total_count=len(result_positions),
    )


if __name__ == "__main__":
    import uvicorn

    port = int(os.getenv("PORT", "8003"))
    uvicorn.run(app, host="0.0.0.0", port=port)
