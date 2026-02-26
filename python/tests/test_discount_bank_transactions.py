"""Tests for discount bank transaction parsing and Rust parser integration."""

import tempfile
from pathlib import Path

from python.integration.discount_bank_service import (
    _parse_transactions_from_file,
    _parse_file_via_rust,
    _read_balance_from_file,
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
