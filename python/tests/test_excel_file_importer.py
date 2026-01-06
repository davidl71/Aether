"""
test_excel_file_importer.py - Tests for Excel/CSV file import
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

import pytest
import pandas as pd
import tempfile
from datetime import datetime

from integration.excel_file_importer import ExcelFileImporter
from integration.israeli_broker_models import Position, PositionSource


class TestExcelFileImporter:
    """Tests for ExcelFileImporter class."""

    @pytest.fixture
    def sample_config(self):
        """Sample broker configuration."""
        return {
            'broker': 'test_broker',
            'field_mapping': {
                'symbol': 'Symbol',
                'quantity': 'Quantity',
                'cost_basis': 'Cost Basis',
                'current_price': 'Current Price',
                'currency': 'Currency',
                'exchange': 'Exchange',
                'instrument_type': 'Type'
            },
            'file_format': 'xlsx',
            'sheet_name': 0,
            'skip_rows': 1,
            'encoding': 'utf-8'
        }

    @pytest.fixture
    def sample_data(self):
        """Sample position data for testing."""
        return pd.DataFrame({
            'Symbol': ['AAPL', 'MSFT', 'TA35'],
            'Quantity': [10, 5, 100],
            'Cost Basis': [150.0, 300.0, 1500.0],
            'Current Price': [155.0, 310.0, 1520.0],
            'Currency': ['USD', 'USD', 'ILS'],
            'Exchange': ['NYSE', 'NASDAQ', 'TASE'],
            'Type': ['stock', 'stock', 'index']
        })

    def test_detect_file_format_xlsx(self, sample_config):
        """Test file format detection for .xlsx files."""
        importer = ExcelFileImporter(sample_config)
        assert importer.detect_file_format('test.xlsx') == 'xlsx'

    def test_detect_file_format_xls(self, sample_config):
        """Test file format detection for .xls files."""
        importer = ExcelFileImporter(sample_config)
        assert importer.detect_file_format('test.xls') == 'xls'

    def test_detect_file_format_csv(self, sample_config):
        """Test file format detection for .csv files."""
        importer = ExcelFileImporter(sample_config)
        assert importer.detect_file_format('test.csv') == 'csv'

    def test_parse_excel(self, sample_config, sample_data):
        """Test Excel file parsing."""
        # Excel config should not skip rows when header is in first row
        excel_config = sample_config.copy()
        excel_config['skip_rows'] = 0

        with tempfile.NamedTemporaryFile(suffix='.xlsx', delete=False) as f:
            file_path = f.name
            sample_data.to_excel(file_path, index=False, engine='openpyxl')

        try:
            importer = ExcelFileImporter(excel_config)
            df = importer.parse_excel(file_path)
            assert len(df) == 3
            assert 'Symbol' in df.columns
        finally:
            Path(file_path).unlink()

    def test_parse_csv(self, sample_config, sample_data):
        """Test CSV file parsing."""
        # CSV config should not skip rows (header is in first row)
        csv_config = sample_config.copy()
        csv_config['skip_rows'] = 0

        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, encoding='utf-8') as f:
            file_path = f.name
            sample_data.to_csv(f, index=False)

        try:
            importer = ExcelFileImporter(csv_config)
            df = importer.parse_csv(file_path)
            assert len(df) == 3
            assert 'Symbol' in df.columns
        finally:
            Path(file_path).unlink()

    def test_normalize_positions(self, sample_config, sample_data):
        """Test position normalization."""
        importer = ExcelFileImporter(sample_config)
        positions = importer.normalize_positions(sample_data)

        assert len(positions) == 3
        assert all(isinstance(p, Position) for p in positions)
        assert all(p.source == PositionSource.ISRAELI_BROKER_EXCEL for p in positions)
        assert all(p.broker == 'test_broker' for p in positions)

        # Check first position
        pos = positions[0]
        assert pos.symbol == 'AAPL'
        assert pos.quantity == 10.0
        assert pos.cost_basis == 150.0
        assert pos.current_price == 155.0
        # Currency comes from data (USD in sample_data), not default
        assert pos.currency == 'USD'
        assert pos.unrealized_pnl == (155.0 - 150.0) * 10.0

    def test_normalize_positions_missing_fields(self, sample_config):
        """Test normalization with missing required fields."""
        # Missing 'current_price' field
        df = pd.DataFrame({
            'Symbol': ['AAPL'],
            'Quantity': [10],
            'Cost Basis': [150.0]
            # Missing 'Current Price'
        })

        importer = ExcelFileImporter(sample_config)
        with pytest.raises(ValueError, match="Required columns not found in file"):
            importer.normalize_positions(df)

    def test_normalize_positions_invalid_data(self, sample_config):
        """Test normalization with invalid data (skips invalid rows)."""
        df = pd.DataFrame({
            'Symbol': ['AAPL', '', 'MSFT'],  # Empty symbol
            'Quantity': [10, 5, 0],  # Zero quantity
            'Cost Basis': [150.0, 300.0, 400.0],
            'Current Price': [155.0, 310.0, 410.0],
            'Currency': ['USD', 'USD', 'USD']
        })

        importer = ExcelFileImporter(sample_config)
        positions = importer.normalize_positions(df)

        # Should only have 1 valid position (AAPL)
        assert len(positions) == 1
        assert positions[0].symbol == 'AAPL'

    def test_import_positions_xlsx(self, sample_config, sample_data):
        """Test full import workflow for Excel file."""
        # Excel config should not skip rows when header is in first row
        excel_config = sample_config.copy()
        excel_config['skip_rows'] = 0

        with tempfile.NamedTemporaryFile(suffix='.xlsx', delete=False) as f:
            file_path = f.name
            sample_data.to_excel(f.name, index=False, engine='openpyxl')

        try:
            importer = ExcelFileImporter(excel_config)
            positions = importer.import_positions(file_path)

            assert len(positions) == 3
            assert all(isinstance(p, Position) for p in positions)
        finally:
            Path(file_path).unlink()

    def test_import_positions_csv(self, sample_config, sample_data):
        """Test full import workflow for CSV file."""
        # CSV config should not skip rows (header is in first row)
        csv_config = sample_config.copy()
        csv_config['skip_rows'] = 0

        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, encoding='utf-8') as f:
            file_path = f.name
            sample_data.to_csv(f, index=False)

        try:
            importer = ExcelFileImporter(csv_config)
            positions = importer.import_positions(file_path)

            assert len(positions) == 3
            assert all(isinstance(p, Position) for p in positions)
        finally:
            Path(file_path).unlink()

    def test_import_positions_file_not_found(self, sample_config):
        """Test import with non-existent file."""
        importer = ExcelFileImporter(sample_config)
        with pytest.raises(FileNotFoundError):
            importer.import_positions('/nonexistent/file.xlsx')

    def test_tase_instrument_detection(self, sample_config):
        """Test TASE instrument detection."""
        df = pd.DataFrame({
            'Symbol': ['TA35'],
            'Quantity': [100],
            'Cost Basis': [1500.0],
            'Current Price': [1520.0],
            'Currency': ['ILS'],
            'Exchange': ['TASE'],
            'Type': ['index']
        })

        importer = ExcelFileImporter(sample_config)
        positions = importer.normalize_positions(df)

        assert len(positions) == 1
        assert positions[0].is_tase_instrument() is True
        assert positions[0].exchange == 'TASE'
