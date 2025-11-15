"""
Data loading utilities for notebooks.

Provides convenient functions to load data from QuestDB, ORATS, and other sources.
"""

import sys
from pathlib import Path
from datetime import datetime, timedelta
from typing import Optional, Dict, List, Iterator
import logging

import pandas as pd
import numpy as np

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

try:
    from python.integration.questdb_client import QuestDBClient
    from python.integration.orats_client import ORATSClient
except ImportError as e:
    logging.warning(f"Could not import integration modules: {e}")
    QuestDBClient = None
    ORATSClient = None

logger = logging.getLogger(__name__)


def setup_notebook_imports():
    """Setup imports for notebook environment."""
    if project_root not in sys.path:
        sys.path.insert(0, str(project_root))


def check_questdb_connection(host: str = "127.0.0.1", port: int = 9009) -> bool:
    """
    Check if QuestDB is accessible.

    Args:
        host: QuestDB host
        port: QuestDB port

    Returns:
        True if connection successful
    """
    if QuestDBClient is None:
        logger.warning("QuestDBClient not available")
        return False

    try:
        client = QuestDBClient(host=host, port=port)
        client.connect()
        client.close()
        return True
    except Exception as e:
        logger.error(f"QuestDB connection failed: {e}")
        return False


def load_questdb_data(
    symbol: str,
    start_date: str,
    end_date: str,
    table: str = "quotes",
    host: str = "127.0.0.1",
    port: int = 8812,  # HTTP port for queries
) -> pd.DataFrame:
    """
    Load time-series data from QuestDB.

    Args:
        symbol: Symbol to query
        start_date: Start date (YYYY-MM-DD)
        end_date: End date (YYYY-MM-DD)
        table: Table name (quotes or trades)
        host: QuestDB host
        port: QuestDB HTTP port

    Returns:
        DataFrame with time-series data
    """
    import requests

    # QuestDB REST API query
    query = f"""
    SELECT * FROM {table}
    WHERE symbol = '{symbol}'
    AND timestamp >= '{start_date}T00:00:00.000000Z'
    AND timestamp <= '{end_date}T23:59:59.999999Z'
    ORDER BY timestamp
    """

    url = f"http://{host}:{port}/exec"
    params = {"query": query}

    try:
        response = requests.get(url, params=params, timeout=10)
        response.raise_for_status()

        # QuestDB returns CSV
        from io import StringIO
        df = pd.read_csv(StringIO(response.text))

        # Convert timestamp if present
        if "timestamp" in df.columns:
            df["timestamp"] = pd.to_datetime(df["timestamp"])

        logger.info(f"Loaded {len(df)} rows from QuestDB for {symbol}")
        return df

    except Exception as e:
        logger.error(f"Failed to load QuestDB data: {e}")
        return pd.DataFrame()


def load_questdb_data_chunked(
    symbol: str,
    start_date: str,
    end_date: str,
    chunk_size: int = 10000,
    table: str = "quotes",
    host: str = "127.0.0.1",
    port: int = 8812,
) -> Iterator[pd.DataFrame]:
    """
    Load QuestDB data in chunks for large datasets.

    Args:
        symbol: Symbol to query
        start_date: Start date
        end_date: End date
        chunk_size: Number of rows per chunk
        table: Table name
        host: QuestDB host
        port: QuestDB HTTP port

    Yields:
        DataFrame chunks
    """
    import requests

    # Calculate date range
    start = datetime.strptime(start_date, "%Y-%m-%d")
    end = datetime.strptime(end_date, "%Y-%m-%d")

    current = start
    while current < end:
        chunk_end = min(current + timedelta(days=7), end)  # 7-day chunks

        query = f"""
        SELECT * FROM {table}
        WHERE symbol = '{symbol}'
        AND timestamp >= '{current.strftime('%Y-%m-%d')}T00:00:00.000000Z'
        AND timestamp < '{chunk_end.strftime('%Y-%m-%d')}T23:59:59.999999Z'
        ORDER BY timestamp
        LIMIT {chunk_size}
        """

        url = f"http://{host}:{port}/exec"
        params = {"query": query}

        try:
            response = requests.get(url, params=params, timeout=10)
            response.raise_for_status()

            from io import StringIO
            df = pd.read_csv(StringIO(response.text))

            if len(df) == 0:
                break

            if "timestamp" in df.columns:
                df["timestamp"] = pd.to_datetime(df["timestamp"])

            yield df
            current = chunk_end

        except Exception as e:
            logger.error(f"Failed to load chunk: {e}")
            break


def load_orats_data(
    ticker: str,
    trade_date: Optional[str] = None,
    api_token: Optional[str] = None,
) -> pd.DataFrame:
    """
    Load ORATS options data.

    Args:
        ticker: Stock ticker
        trade_date: Trade date (YYYY-MM-DD), defaults to today
        api_token: ORATS API token (or set ORATS_API_TOKEN env var)

    Returns:
        DataFrame with ORATS options data
    """
    if ORATSClient is None:
        logger.warning("ORATSClient not available")
        return pd.DataFrame()

    import os

    if api_token is None:
        api_token = os.getenv("ORATS_API_TOKEN")

    if not api_token:
        logger.error("ORATS API token not provided")
        return pd.DataFrame()

    try:
        client = ORATSClient(api_token=api_token)
        strikes = client.get_strikes(ticker, trade_date=trade_date)

        if not strikes:
            return pd.DataFrame()

        df = pd.DataFrame(strikes)
        logger.info(f"Loaded {len(df)} option strikes for {ticker}")
        return df

    except Exception as e:
        logger.error(f"Failed to load ORATS data: {e}")
        return pd.DataFrame()


def load_trading_logs(
    log_dir: Optional[str] = None,
    pattern: str = "*.log",
    start_date: Optional[str] = None,
    end_date: Optional[str] = None,
) -> pd.DataFrame:
    """
    Load trading logs from log directory.

    Args:
        log_dir: Log directory path (defaults to project log/ directory)
        pattern: Log file pattern
        start_date: Start date filter
        end_date: End date filter

    Returns:
        DataFrame with log entries
    """
    if log_dir is None:
        log_dir = project_root / "log"
    else:
        log_dir = Path(log_dir)

    if not log_dir.exists():
        logger.warning(f"Log directory not found: {log_dir}")
        return pd.DataFrame()

    import glob

    log_files = list(log_dir.glob(pattern))

    if not log_files:
        logger.warning(f"No log files found matching {pattern}")
        return pd.DataFrame()

    # Simple log parsing (can be enhanced)
    log_entries = []

    for log_file in log_files:
        try:
            with open(log_file, "r") as f:
                for line in f:
                    # Basic parsing - adjust based on log format
                    if "box_spread" in line.lower() or "opportunity" in line.lower():
                        log_entries.append({
                            "file": log_file.name,
                            "line": line.strip(),
                            "timestamp": None,  # Extract from line if possible
                        })
        except Exception as e:
            logger.warning(f"Failed to read {log_file}: {e}")

    if not log_entries:
        return pd.DataFrame()

    df = pd.DataFrame(log_entries)

    # Apply date filters if provided
    if start_date or end_date:
        # This is simplified - adjust based on actual log format
        pass

    logger.info(f"Loaded {len(df)} log entries")
    return df


def load_config(config_path: Optional[str] = None) -> Dict:
    """
    Load configuration from JSON file.

    Args:
        config_path: Path to config file (defaults to config/config.json)

    Returns:
        Configuration dictionary
    """
    import json

    if config_path is None:
        config_path = project_root / "config" / "config.json"
    else:
        config_path = Path(config_path)

    if not config_path.exists():
        logger.warning(f"Config file not found: {config_path}")
        return {}

    try:
        with open(config_path, "r") as f:
            config = json.load(f)
        logger.info(f"Loaded config from {config_path}")
        return config
    except Exception as e:
        logger.error(f"Failed to load config: {e}")
        return {}


def load_example_positions() -> pd.DataFrame:
    """
    Load example positions from CSV.

    Returns:
        DataFrame with example positions
    """
    positions_file = project_root / "example_positions.csv"

    if not positions_file.exists():
        logger.warning(f"Example positions file not found: {positions_file}")
        return pd.DataFrame()

    try:
        df = pd.read_csv(positions_file)
        logger.info(f"Loaded {len(df)} example positions")
        return df
    except Exception as e:
        logger.error(f"Failed to load positions: {e}")
        return pd.DataFrame()
