"""
excel_file_importer.py - Excel/CSV static file import for Israeli broker positions
"""
import logging
from pathlib import Path
from typing import List, Dict, Any
from datetime import datetime

try:
    import pandas as pd
except ImportError:
    raise ImportError(
        "pandas is required for Excel file import. Install with: pip install pandas openpyxl"
    )

from .israeli_broker_models import Position, PositionSource

logger = logging.getLogger(__name__)


class ExcelFileImporter:
    """Import positions from static Excel/CSV files."""

    def __init__(self, config: Dict[str, Any]):
        """
        Initialize Excel file importer.

        Args:
            config: Configuration dict with broker-specific settings
                   Expected keys: 'broker', 'field_mapping', 'file_format', etc.
        """
        self.config = config
        self.broker = config.get('broker', 'unknown')
        self.field_mapping = config.get('field_mapping', {})
        self.file_format = config.get('file_format', 'xlsx')
        self.sheet_name = config.get('sheet_name', 0)  # 0 = first sheet
        self.skip_rows = config.get('skip_rows', 0)
        self.encoding = config.get('encoding', 'utf-8')  # cp1255 for Hebrew

    def import_positions(self, file_path: str) -> List[Position]:
        """
        Import positions from Excel/CSV file.

        Args:
            file_path: Path to Excel (.xlsx, .xls) or CSV file

        Returns:
            List of Position objects

        Raises:
            FileNotFoundError: If file doesn't exist
            ValueError: If file format is invalid or data is malformed
        """
        file_path_obj = Path(file_path)
        if not file_path_obj.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        # Detect file format
        file_format = self.detect_file_format(file_path)
        logger.info(f"Importing {file_format} file: {file_path}")

        # Parse file based on format
        if file_format in ['xlsx', 'xls']:
            df = self.parse_excel(file_path, file_format)
        elif file_format in ['csv', 'tsv']:
            df = self.parse_csv(file_path, file_format)
        else:
            raise ValueError(f"Unsupported file format: {file_format}")

        # Normalize to Position objects
        positions = self.normalize_positions(df)
        logger.info(f"Imported {len(positions)} positions from {file_path}")

        return positions

    def detect_file_format(self, file_path: str) -> str:
        """
        Detect file format from extension.

        Args:
            file_path: Path to file

        Returns:
            File format string ('xlsx', 'xls', 'csv', 'tsv')
        """
        path = Path(file_path)
        ext = path.suffix.lower()

        if ext == '.xlsx':
            return 'xlsx'
        elif ext == '.xls':
            return 'xls'
        elif ext == '.csv':
            return 'csv'
        elif ext == '.tsv':
            return 'tsv'
        else:
            # Try to infer from file format config
            return self.file_format

    def parse_excel(self, file_path: str, file_format: str = 'xlsx') -> pd.DataFrame:
        """
        Parse Excel file using pandas.

        Args:
            file_path: Path to Excel file
            file_format: 'xlsx' or 'xls'

        Returns:
            DataFrame with position data

        Raises:
            ValueError: If file cannot be parsed
        """
        try:
            # Choose engine based on format
            engine = 'openpyxl' if file_format == 'xlsx' else 'xlrd'

            # Read Excel file
            # Note: pd.read_excel() doesn't support encoding parameter
            # Encoding is handled by the engine (openpyxl/xlrd)
            df = pd.read_excel(
                file_path,
                sheet_name=self.sheet_name,
                skiprows=self.skip_rows,
                engine=engine
            )

            logger.debug(f"Parsed Excel file: {len(df)} rows, {len(df.columns)} columns")
            return df

        except Exception as e:
            logger.error(f"Failed to parse Excel file {file_path}: {e}")
            raise ValueError(f"Failed to parse Excel file: {e}") from e

    def parse_csv(self, file_path: str, file_format: str = 'csv') -> pd.DataFrame:
        """
        Parse CSV/TSV file using pandas.

        Args:
            file_path: Path to CSV/TSV file
            file_format: 'csv' or 'tsv'

        Returns:
            DataFrame with position data

        Raises:
            ValueError: If file cannot be parsed
        """
        try:
            # Determine delimiter
            delimiter = '\t' if file_format == 'tsv' else ','
            delimiter = self.config.get('delimiter', delimiter)

            # Read CSV file
            # If skip_rows > 0, we skip header rows but still need column names
            # So we read header separately if needed
            if self.skip_rows > 0:
                # Read with header in first row, then skip specified rows
                df = pd.read_csv(
                    file_path,
                    delimiter=delimiter,
                    skiprows=self.skip_rows,
                    encoding=self.encoding
                )
            else:
                # Read normally with header
                df = pd.read_csv(
                    file_path,
                    delimiter=delimiter,
                    encoding=self.encoding
                )

            logger.debug(f"Parsed CSV file: {len(df)} rows, {len(df.columns)} columns")
            return df

        except Exception as e:
            logger.error(f"Failed to parse CSV file {file_path}: {e}")
            raise ValueError(f"Failed to parse CSV file: {e}") from e

    def normalize_positions(self, df: pd.DataFrame) -> List[Position]:
        """
        Normalize broker-specific format to standard Position model.

        Args:
            df: DataFrame with broker-specific columns

        Returns:
            List of Position objects

        Raises:
            ValueError: If required fields are missing or data is invalid
        """
        positions = []

        # Validate required field mappings exist in config
        required_fields = ['symbol', 'quantity', 'cost_basis', 'current_price']
        missing_mappings = [f for f in required_fields if f not in self.field_mapping]
        if missing_mappings:
            raise ValueError(
                f"Missing required field mappings in config: {missing_mappings}. "
                f"Available columns in file: {list(df.columns)}"
            )

        # Validate that mapped columns exist in DataFrame
        missing_columns = []
        for field_name, broker_column in self.field_mapping.items():
            if field_name in required_fields and broker_column not in df.columns:
                missing_columns.append(f"{field_name} -> '{broker_column}'")

        if missing_columns:
            raise ValueError(
                f"Required columns not found in file: {missing_columns}. "
                f"Available columns: {list(df.columns)}"
            )

        # Map columns using field mapping
        for idx, row in df.iterrows():
            try:
                # Extract mapped fields
                symbol = self._get_mapped_value(row, 'symbol')
                quantity = self._get_mapped_value(row, 'quantity', float)
                cost_basis = self._get_mapped_value(row, 'cost_basis', float)
                current_price = self._get_mapped_value(row, 'current_price', float)
                # Currency: use mapped value if available, otherwise default to ILS for Israeli brokers
                currency = self._get_mapped_value(row, 'currency', str, default='ILS')

                # Validate required fields
                if not symbol or pd.isna(symbol):
                    logger.warning(f"Row {idx}: Missing symbol, skipping")
                    continue

                if pd.isna(quantity) or quantity == 0:
                    logger.warning(f"Row {idx}: Invalid quantity {quantity}, skipping")
                    continue

                if pd.isna(cost_basis) or cost_basis < 0:
                    logger.warning(f"Row {idx}: Invalid cost_basis {cost_basis}, skipping")
                    continue

                if pd.isna(current_price) or current_price < 0:
                    logger.warning(f"Row {idx}: Invalid current_price {current_price}, skipping")
                    continue

                # Extract optional fields
                exchange = self._get_mapped_value(row, 'exchange', str, default=None)
                instrument_type = self._get_mapped_value(row, 'instrument_type', str, default=None)
                underlying = self._get_mapped_value(row, 'underlying', str, default=None)
                strike = self._get_mapped_value(row, 'strike', float, default=None)
                expiration_date = self._get_mapped_value(row, 'expiration_date', str, default=None)
                option_type = self._get_mapped_value(row, 'option_type', str, default=None)
                account_id = self._get_mapped_value(row, 'account_id', str, default=None)

                # Parse expiration date if provided
                parsed_expiration = None
                if expiration_date and not pd.isna(expiration_date):
                    try:
                        parsed_expiration = pd.to_datetime(expiration_date)
                    except Exception:
                        logger.warning(f"Row {idx}: Could not parse expiration_date {expiration_date}")

                # Calculate unrealized P&L
                unrealized_pnl = (current_price - cost_basis) * quantity

                # Create Position object
                position = Position(
                    symbol=str(symbol).strip(),
                    quantity=float(quantity),
                    cost_basis=float(cost_basis),
                    current_price=float(current_price),
                    currency=str(currency).upper(),
                    broker=self.broker,
                    source=PositionSource.ISRAELI_BROKER_EXCEL,
                    account_id=account_id,
                    last_updated=datetime.now(),
                    unrealized_pnl=unrealized_pnl,
                    exchange=exchange,
                    instrument_type=instrument_type,
                    underlying=underlying,
                    strike=strike,
                    expiration_date=parsed_expiration,
                    option_type=option_type
                )

                positions.append(position)

            except Exception as e:
                logger.error(f"Row {idx}: Failed to normalize position: {e}")
                continue

        return positions

    def _get_mapped_value(
        self,
        row: pd.Series,
        field_name: str,
        dtype: type = str,
        default: Any = None
    ) -> Any:
        """
        Get value from row using field mapping.

        Args:
            row: DataFrame row
            field_name: Standard field name (e.g., 'symbol')
            dtype: Expected data type
            default: Default value if field not found

        Returns:
            Mapped value or default
        """
        # Get broker-specific column name from mapping
        broker_column = self.field_mapping.get(field_name)

        if not broker_column:
            return default

        # Get value from row
        if broker_column not in row.index:
            logger.warning(f"Column '{broker_column}' not found in row")
            return default

        value = row[broker_column]

        # Handle NaN values
        if pd.isna(value):
            return default

        # Convert type
        try:
            if dtype == float:
                return float(value)
            elif dtype == int:
                return int(float(value))  # Convert via float to handle "1.0" -> 1
            elif dtype == str:
                return str(value)
            else:
                return value
        except (ValueError, TypeError) as e:
            logger.warning(f"Could not convert {broker_column}={value} to {dtype}: {e}")
            return default
