"""Tests for discount bank transaction parsing and Rust parser integration."""

import json
import sqlite3
import tempfile
from pathlib import Path

from fastapi.testclient import TestClient

from python.integration.discount_bank_service import (
    _extract_bank_accounts_from_ledger,
    _parse_transactions_from_file,
    _parse_file_via_rust,
    app,
)


class TestParseTransactions:
    def _make_file(self, lines: list[str]) -> Path:
        tf = tempfile.NamedTemporaryFile(
            mode="w", suffix=".dat", delete=False, encoding="utf-8"
        )
        tf.write("\n".join(lines))
        tf.close()
        return Path(tf.name)

    def test_parses_detail_records(self):
        lines = [
            "0020260115" + "000000000123456+" + "Salary payment from employer     ",
            "0120260210" + "000000000050000-" + "Electric bill payment            ",
            "0120260215" + "000000000025000+" + "Refund for overcharge            ",
        ]
        path = self._make_file(lines)
        txns = _parse_transactions_from_file(path, limit=10)

        assert len(txns) == 2
        assert txns[0].amount == 250.00
        assert txns[0].is_debit is False
        assert txns[1].amount == 500.00
        assert txns[1].is_debit is True

    def test_parses_ddmmyyyy_detail_record(self):
        lines = [
            "0115022026" + "000000000025000+" + "Refund for overcharge            ",
        ]
        path = self._make_file(lines)
        txns = _parse_transactions_from_file(path, limit=10)

        assert len(txns) == 1
        assert txns[0].value_date == "2026-02-15"

    def test_skips_invalid_detail_record_date(self):
        lines = [
            "0131022026" + "000000000025000+" + "Impossible date                  ",
        ]
        path = self._make_file(lines)
        txns = _parse_transactions_from_file(path, limit=10)

        assert txns == []

    def test_skips_header_records(self):
        lines = [
            "00" + "5350000" + "0001" + "276689" + "0000000010000000" + " " + "260115",
        ]
        path = self._make_file(lines)
        txns = _parse_transactions_from_file(path, limit=10)
        assert txns == []

    def test_respects_limit(self):
        detail = "0120260210" + "000000000010000-" + "Payment                          "
        lines = [detail] * 50
        path = self._make_file(lines)
        txns = _parse_transactions_from_file(path, limit=5)
        assert len(txns) == 5

    def test_empty_file(self):
        path = self._make_file([])
        txns = _parse_transactions_from_file(path, limit=10)
        assert txns == []

    def test_nonexistent_file(self):
        txns = _parse_transactions_from_file(Path("/nonexistent/file.dat"), limit=10)
        assert txns == []


class TestRustParserFallback:
    def test_falls_back_to_python_parser(self, tmp_path):
        dat_file = tmp_path / "DISCOUNT.dat"
        # Format: pos 0-1 = "00", 3-5 = branch, 6-9 = section,
        # 10-11 = currency, 12-17 = account, 18-32 = opening (14 digits),
        # 33 = sign, 34-47 = closing (14 digits), 48 = sign, 49-54 = YYMMDD
        # Total needs >= 54 chars with "00" prefix at position 0
        header = (
            "00"         # record type
            "1"          # bank
            "535"        # branch
            "0000"       # section
            "01"         # currency (ILS)
            "276689"     # account
            "00000100000000"  # opening balance (14 digits)
            " "          # sign
            "00000200000000"  # closing balance (14 digits)
            " "          # sign
            "260115"     # date YYMMDD
        )
        dat_file.write_text(header)

        result = _parse_file_via_rust(dat_file)
        assert "balance" in result or "account" in result


class TestExtractBankAccounts:
    def _make_db(self) -> Path:
        tf = tempfile.NamedTemporaryFile(suffix=".db", delete=False)
        tf.close()
        db_path = Path(tf.name)

        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()
        cursor.execute(
            """
            CREATE TABLE transactions (
                id TEXT PRIMARY KEY,
                transaction_json TEXT
            )
            """
        )
        conn.commit()
        conn.close()
        return db_path

    def _insert_transaction(self, db_path: Path, txn_id: str, postings: list[dict]):
        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()
        cursor.execute(
            "INSERT INTO transactions (id, transaction_json) VALUES (?, ?)",
            (
                txn_id,
                json.dumps(
                    {
                        "id": txn_id,
                        "date": "2026-03-10T00:00:00Z",
                        "description": "test",
                        "postings": postings,
                    }
                ),
            ),
        )
        conn.commit()
        conn.close()

    def test_returns_single_currency_bank_account(self, monkeypatch):
        db_path = self._make_db()
        self._insert_transaction(
            db_path,
            "txn-1",
            [
                {
                    "account": "Assets:Bank:Discount:123456",
                    "amount": {"amount": "100.50", "currency": "ILS"},
                }
            ],
        )

        monkeypatch.setenv("LEDGER_DATABASE_PATH", str(db_path))
        accounts = _extract_bank_accounts_from_ledger()

        assert len(accounts) == 1
        account = accounts[0]
        assert account["currency"] == "ILS"
        assert account["balance"] == 100.50
        assert account["balances_by_currency"] is None
        assert account["is_mixed_currency"] is False
        assert account["credit_rate"] == 0.03
        assert account["debit_rate"] == 0.103

    def test_returns_mixed_currency_bank_account_breakdown(self, monkeypatch):
        db_path = self._make_db()
        self._insert_transaction(
            db_path,
            "txn-1",
            [
                {
                    "account": "Assets:Bank:Discount:123456",
                    "amount": {"amount": "100.50", "currency": "ILS"},
                },
                {
                    "account": "Assets:Bank:Discount:123456",
                    "amount": {"amount": "25.25", "currency": "USD"},
                },
            ],
        )

        monkeypatch.setenv("LEDGER_DATABASE_PATH", str(db_path))
        accounts = _extract_bank_accounts_from_ledger()

        assert len(accounts) == 1
        account = accounts[0]
        assert account["currency"] == "MULTI"
        assert account["balance"] == 0.0
        assert account["is_mixed_currency"] is True
        assert account["balances_by_currency"] == {"ILS": 100.50, "USD": 25.25}
        assert account["credit_rate"] == 0.03
        assert account["debit_rate"] == 0.103


class TestImportPositionsEndpoint:
    def test_reconciliation_is_read_only(self, monkeypatch):
        client = TestClient(app)

        monkeypatch.setattr(
            "python.integration.discount_bank_service._fetch_ibkr_positions",
            lambda account_id=None: [
                {
                    "symbol": "SPY",
                    "quantity": 10.0,
                    "avg_price": 500.0,
                    "currency": "USD",
                },
                {
                    "symbol": "QQQ",
                    "quantity": 5.0,
                    "avg_price": 400.0,
                    "currency": "USD",
                },
            ],
        )
        monkeypatch.setattr(
            "python.integration.discount_bank_service._get_ledger_positions",
            lambda: {
                "SPY": {
                    "quantity": 10.0,
                    "currency": "USD",
                    "account_path": "Assets:IBKR:SPY",
                }
            },
        )

        response = client.get("/api/import-positions?broker=ibkr")

        assert response.status_code == 200
        payload = response.json()
        assert payload["status"] == "read_only_reconciliation"
        assert payload["write_disabled"] is True
        assert payload["imported_count"] == 0
        assert payload["existing_count"] == 1
        assert payload["missing_count"] == 1
        assert payload["total_count"] == 2
        assert payload["positions"][0]["symbol"] == "SPY"
        assert payload["positions"][0]["in_ledger"] is True
        assert payload["positions"][1]["symbol"] == "QQQ"
        assert payload["positions"][1]["in_ledger"] is False
