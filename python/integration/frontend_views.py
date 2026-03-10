"""
Shared frontend view-model helpers.

These helpers normalize backend/domain data into the shapes consumed by the
Textual TUI and the React web client so the UI layers don't maintain separate
business logic for the same concepts.
"""

from __future__ import annotations

from typing import Any, Dict, Iterable, List, Optional


def _default_candle() -> Dict[str, Any]:
    return {
        "open": 0.0,
        "high": 0.0,
        "low": 0.0,
        "close": 0.0,
        "volume": 0.0,
        "entry": 0.0,
        "updated": "",
    }


def normalize_bank_accounts_to_positions(
    bank_accounts: Iterable[Dict[str, Any]],
    *,
    reference_candle: Optional[Dict[str, Any]] = None,
) -> List[Dict[str, Any]]:
    """Convert bank-account rows into shared position-like dictionaries."""
    positions: List[Dict[str, Any]] = []
    base_candle = dict(reference_candle or _default_candle())

    for account in bank_accounts:
        if not isinstance(account, dict):
            continue

        rate = account.get("credit_rate")
        if rate in (None, 0, 0.0):
            rate = account.get("debit_rate")

        account_name = str(account.get("account_name") or "Bank Account")
        balances_by_currency = account.get("balances_by_currency")

        if account.get("is_mixed_currency") and isinstance(balances_by_currency, dict):
            for currency, amount in sorted(balances_by_currency.items()):
                positions.append(
                    _make_bank_position(
                        account_name=account_name,
                        amount=float(amount or 0.0),
                        currency=str(currency or "USD"),
                        rate=rate,
                        candle=base_candle,
                        name_suffix=f" ({currency})" if currency else "",
                    )
                )
            continue

        positions.append(
            _make_bank_position(
                account_name=account_name,
                amount=float(account.get("balance") or 0.0),
                currency=str(account.get("currency") or "USD"),
                rate=rate,
                candle=base_candle,
            )
        )

    return positions


def _make_bank_position(
    *,
    account_name: str,
    amount: float,
    currency: str,
    rate: Optional[float],
    candle: Dict[str, Any],
    name_suffix: str = "",
) -> Dict[str, Any]:
    return {
        "name": f"{account_name}{name_suffix}",
        "quantity": 1,
        "roi": float(rate or 0.0) * 100,
        "maker_count": 0,
        "taker_count": 0,
        "rebate_estimate": 0.0,
        "vega": 0.0,
        "theta": 0.0,
        "fair_diff": 0.0,
        "candle": dict(candle),
        "instrument_type": "bank_loan",
        "rate": rate,
        "currency": currency,
        "cash_flow": amount,
    }


def infer_relationships(
    positions: Iterable[Dict[str, Any]],
    bank_accounts: Iterable[Dict[str, Any]],
) -> List[Dict[str, Any]]:
    """Infer frontend relationship graph from shared positions + bank accounts."""
    normalized_positions = [p for p in positions if isinstance(p, dict)]
    synthetic_loans = _bank_accounts_as_loans(bank_accounts)

    loans = [
        p
        for p in [*normalized_positions, *synthetic_loans]
        if p.get("instrument_type") in ("bank_loan", "pension_loan")
    ]
    box_spreads = [
        p for p in normalized_positions if p.get("instrument_type") == "box_spread"
    ]
    bonds = [
        p
        for p in normalized_positions
        if p.get("instrument_type") in ("bond", "t_bill")
    ]

    relationships: List[Dict[str, Any]] = []

    for loan in loans:
        loan_value = _position_value(loan)
        for box_spread in box_spreads:
            relationships.append(
                {
                    "from": loan.get("name", "Unknown"),
                    "to": box_spread.get("name", "Unknown"),
                    "type": "margin",
                    "description": "Loan used as margin for box spread",
                    "value": loan_value,
                }
            )

    for loan in loans:
        loan_value = _position_value(loan)
        for bond in bonds:
            relationships.append(
                {
                    "from": loan.get("name", "Unknown"),
                    "to": bond.get("name", "Unknown"),
                    "type": "investment",
                    "description": "Loan proceeds invested in bond",
                    "value": loan_value,
                }
            )

    for bond in bonds:
        bond_value = (
            float(bond.get("collateral_value") or 0.0) or _position_value(bond)
        )
        bond_rate = float(bond.get("rate") or 0.0)
        for loan in loans:
            loan_rate = float(loan.get("rate") or 0.0)
            if bond_rate > loan_rate:
                relationships.append(
                    {
                        "from": bond.get("name", "Unknown"),
                        "to": loan.get("name", "Unknown"),
                        "type": "collateral",
                        "description": "Bond used as collateral for loan",
                        "value": bond_value,
                    }
                )

    for box_spread in box_spreads:
        relationships.append(
            {
                "from": box_spread.get("name", "Unknown"),
                "to": "Synthetic Financing",
                "type": "financing",
                "description": "Box spread provides synthetic financing",
                "value": _position_value(box_spread),
            }
        )

    return relationships


def relationship_nodes(
    relationships: Iterable[Dict[str, Any]],
    positions: Iterable[Dict[str, Any]],
    bank_accounts: Iterable[Dict[str, Any]],
) -> List[str]:
    nodes = set()
    for rel in relationships:
        if not isinstance(rel, dict):
            continue
        nodes.add(str(rel.get("from") or ""))
        nodes.add(str(rel.get("to") or ""))
    for position in positions:
        if isinstance(position, dict):
            nodes.add(str(position.get("name") or ""))
    for account in bank_accounts:
        if isinstance(account, dict):
            nodes.add(str(account.get("account_name") or ""))
    nodes.discard("")
    return sorted(nodes)


def _bank_accounts_as_loans(bank_accounts: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    loans: List[Dict[str, Any]] = []
    for account in bank_accounts:
        if not isinstance(account, dict):
            continue
        debit_rate = account.get("debit_rate")
        if debit_rate is None or float(debit_rate) <= 0:
            continue
        loans.append(
            {
                "name": account.get("account_name", "Bank Account"),
                "instrument_type": "bank_loan",
                "cash_flow": float(account.get("balance") or 0.0),
                "rate": float(debit_rate),
                "candle": {"close": float(account.get("balance") or 0.0)},
            }
        )
    return loans


def _position_value(position: Dict[str, Any]) -> float:
    cash_flow = position.get("cash_flow")
    if cash_flow not in (None, ""):
        return float(cash_flow)
    candle = position.get("candle")
    if isinstance(candle, dict):
        return float(candle.get("close") or 0.0)
    return 0.0
