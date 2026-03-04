"""Pydantic models for Discount Bank and bank account endpoints."""

from __future__ import annotations

from typing import List, Optional

from pydantic import BaseModel


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
