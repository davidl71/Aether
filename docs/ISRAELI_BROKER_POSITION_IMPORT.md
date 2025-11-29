# Israeli Broker Position Import System

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document designs a comprehensive position import system for Israeli brokers that don't provide APIs. The system supports multiple import methods:

1. **Static Excel/CSV Files:** Import from exported position files
2. **Excel RTD (Real-Time Data):** Real-time position updates via Excel RTD server
3. **Excel DDE (Dynamic Data Exchange):** Real-time position updates via DDE
4. **Web Scraping:** Automated position extraction from broker web pages

**TASE (Tel Aviv Stock Exchange) Considerations:**
Israeli brokers typically hold positions in TASE-listed securities and derivatives. The system must handle:

- **TASE Indices:** TA-35, TA-125, TA-90, TA-Banks5
- **TASE Options:** Index options, currency options (ILS/USD, ILS/EUR), stock options
- **TASE Futures:** Index futures (TA-35, TA-90, TA-Banks5) - relaunched September 2024
- **TASE Securities:** Stocks, bonds, ETFs listed on TASE
- **MAOF Clearing House:** All TASE derivatives cleared through MAOF
- See [TASE Knowledge Center](https://www.tase.co.il/en/content/knowledge_center/securities_derivatives) for TASE instrument details

## Integration with Investment Strategy Framework

**Purpose:** Import positions from Israeli brokers to include in portfolio allocation calculations. Positions are factored into the investment strategy framework alongside IBKR positions.

**Net Portfolio Value Impact:**

```
Net Portfolio Value = IBKR Assets + Israeli Broker Assets - Loan Liabilities
```

**Allocation Calculation:**

- Imported positions are included in total portfolio value
- Allocation percentages apply to combined portfolio value
- Currency conversion (ILS → USD) for position valuation
- Position data feeds into portfolio allocation manager

## Import Methods

### 1. Static Excel/CSV File Import

**Description:** Import position data from manually exported or automatically generated Excel/CSV files.

**Use Cases:**

- Manual exports from broker platform
- Scheduled automated exports to file system
- Google Drive sync with automated exports

**Implementation:**

```python
# python/integration/israeli_broker_importer.py
class ExcelFileImporter:
    """Import positions from static Excel/CSV files."""

    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.field_mapping = config.get('field_mapping', {})

    def import_positions(self, file_path: str) -> List[Position]:
        """
        Import positions from Excel/CSV file.

        Args:
            file_path: Path to Excel (.xlsx, .xls) or CSV file

        Returns:
            List of Position objects
        """
        # Use pandas or openpyxl to read file
        # Map broker-specific fields to standard format
        # Validate data
        # Return normalized positions
        pass

    def detect_file_format(self, file_path: str) -> str:
        """Detect file format (xlsx, xls, csv)."""
        pass

    def parse_excel(self, file_path: str) -> pd.DataFrame:
        """Parse Excel file using openpyxl or pandas."""
        pass

    def parse_csv(self, file_path: str) -> pd.DataFrame:
        """Parse CSV file using pandas."""
        pass

    def normalize_positions(self, df: pd.DataFrame) -> List[Position]:
        """Normalize broker-specific format to standard Position model."""
        pass
```

**Field Mapping Configuration:**

```json
{
  "israeli_broker_import": {
    "excel_file_import": {
      "brokers": {
        "poalim": {
          "field_mapping": {
            "symbol": "Ticker/Symbol",
            "quantity": "Quantity/Shares",
            "cost_basis": "Purchase Price",
            "current_price": "Current Price",
            "currency": "Currency",
            "account_type": "Account Type",
            "exchange": "Exchange",
            "instrument_type": "Instrument Type",
            "underlying": "Underlying"
          },
          "file_format": "xlsx",
          "sheet_name": "Positions",
          "skip_rows": 1,
          "tase_identifiers": {
            "index_options": ["TA35", "TA125", "TA90", "TA-Banks5"],
            "currency_options": ["USD/ILS", "EUR/ILS"],
            "futures": ["TA35F", "TA90F", "TA-Banks5F"]
          }
        },
        "leumi": {
          "field_mapping": {
            "symbol": "Symbol",
            "quantity": "Amount",
            "cost_basis": "Buy Price",
            "current_price": "Market Price",
            "currency": "Currency",
            "exchange": "Exchange",
            "instrument_type": "Type"
          },
          "file_format": "csv",
          "delimiter": ","
        }
      }
    }
  }
}
```

**TASE-Specific Field Mapping:**

- **Exchange:** Identify TASE-listed securities (value = "TASE" or "Tel Aviv")
- **Instrument Type:** Differentiate between stocks, options, futures, bonds
- **Underlying:** For TASE derivatives, identify underlying asset (TA-35, USD/ILS, individual stock)
- **TASE Identifiers:** Predefined lists of TASE index options, currency options, futures for automatic classification

**Supported File Formats:**

- `.xlsx` (Excel 2007+)
- `.xls` (Excel 97-2003)
- `.csv` (Comma-separated values)
- `.tsv` (Tab-separated values)

**Dependencies:**

- `pandas` - DataFrame manipulation
- `openpyxl` - Excel file reading
- `xlrd` - Legacy Excel support (if needed)

### 2. Excel RTD (Real-Time Data) Import

**Description:** Connect to Excel RTD server for real-time position updates. Requires Excel running with RTD add-in or broker-provided RTD server.

**Use Cases:**

- Broker provides RTD server add-in
- Real-time position monitoring
- Automated updates without manual export

**Implementation:**

```python
# python/integration/excel_rtd_client.py
import xlwings as xw  # Windows COM automation
import win32com.client  # Alternative COM interface

class ExcelRTDClient:
    """Connect to Excel RTD server for real-time position data."""

    def __init__(self, excel_file_path: str, rtd_topic: str):
        """
        Initialize RTD client.

        Args:
            excel_file_path: Path to Excel file with RTD connections
            rtd_topic: RTD topic identifier (broker-specific)
        """
        self.excel_file = excel_file_path
        self.rtd_topic = rtd_topic
        self.app = None
        self.wb = None

    def connect(self) -> bool:
        """Connect to Excel RTD server."""
        try:
            # Method 1: Using xlwings (recommended)
            self.app = xw.App(visible=False)
            self.wb = self.app.books.open(self.excel_file)

            # Method 2: Using win32com (alternative)
            # self.app = win32com.client.Dispatch("Excel.Application")
            # self.wb = self.app.Workbooks.Open(self.excel_file)

            return True
        except Exception as e:
            logger.error(f"Failed to connect to Excel RTD: {e}")
            return False

    def get_position_data(self, range_name: str) -> Dict[str, Any]:
        """
        Read position data from Excel RTD range.

        Args:
            range_name: Named range in Excel containing position data

        Returns:
            Position data dictionary
        """
        try:
            # xlwings method
            data_range = self.wb.names[range_name].refers_to_range
            values = data_range.value

            # Parse RTD data into positions
            return self._parse_rtd_data(values)
        except Exception as e:
            logger.error(f"Failed to read RTD data: {e}")
            return {}

    def monitor_positions(self, callback: Callable, interval: int = 5):
        """
        Monitor RTD data and call callback when positions change.

        Args:
            callback: Function to call with updated positions
            interval: Polling interval in seconds
        """
        import time
        last_data = None

        while True:
            try:
                current_data = self.get_position_data("Positions")
                if current_data != last_data:
                    callback(current_data)
                    last_data = current_data
                time.sleep(interval)
            except KeyboardInterrupt:
                break
            except Exception as e:
                logger.error(f"Error monitoring RTD: {e}")
                time.sleep(interval)

    def disconnect(self):
        """Disconnect from Excel."""
        if self.wb:
            self.wb.close()
        if self.app:
            self.app.quit()
```

**Requirements:**

- Windows OS (Excel RTD requires Windows COM automation)
- Excel installed with RTD add-in
- Broker-provided RTD server or Excel workbook with RTD connections
- `xlwings` or `pywin32` library

**Setup Steps:**

1. Install broker RTD add-in in Excel
2. Configure Excel workbook with RTD topic connections
3. Set up named ranges for position data
4. Configure Python RTD client with workbook path and range names

### 3. Excel DDE (Dynamic Data Exchange) Import

**Description:** Connect via DDE to broker-provided data feed. DDE is legacy Windows technology but still used by some brokers.

**Use Cases:**

- Broker provides DDE data feed
- Legacy broker systems
- Real-time position updates

**Implementation:**

```python
# python/integration/excel_dde_client.py
import win32ui
import dde  # Python DDE module

class ExcelDDEClient:
    """Connect via DDE for real-time position data."""

    def __init__(self, server_name: str, topic: str):
        """
        Initialize DDE client.

        Args:
            server_name: DDE server name (broker-specific)
            topic: DDE topic identifier
        """
        self.server_name = server_name
        self.topic = topic
        self.conversation = None

    def connect(self) -> bool:
        """Establish DDE conversation."""
        try:
            # Create DDE conversation
            self.conversation = dde.CreateConversation(
                dde.CreateStringHandle(server_name),
                dde.CreateStringHandle(topic),
                dde.CBF_FAIL_ALLSVRXACTIONS | dde.CBF_SKIP_ALLNOTIFICATIONS
            )
            self.conversation.Connect()
            return True
        except Exception as e:
            logger.error(f"Failed to connect via DDE: {e}")
            return False

    def request_position_data(self, item: str) -> str:
        """
        Request position data via DDE.

        Args:
            item: DDE item identifier (e.g., "POSITIONS")

        Returns:
            Position data string
        """
        try:
            handle = self.conversation.RequestData(item)
            data = dde.GetString(handle)
            dde.FreeStringHandle(handle)
            return data
        except Exception as e:
            logger.error(f"Failed to request DDE data: {e}")
            return None

    def disconnect(self):
        """Close DDE conversation."""
        if self.conversation:
            self.conversation.Disconnect()
```

**Requirements:**

- Windows OS (DDE is Windows-specific)
- Broker-provided DDE server
- Python DDE library (`pywin32` includes DDE support)

**Limitations:**

- Windows-only (DDE is legacy Windows technology)
- Less reliable than modern APIs
- Broker must provide DDE server

### 4. Web Scraping Import

**Description:** Automatically extract position data from broker web pages using browser automation.

**Use Cases:**

- Broker provides web portal but no export/API
- Automated position monitoring
- Real-time position tracking

**Implementation:**

```python
# python/integration/web_scraper.py
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from bs4 import BeautifulSoup
import time

class IsraeliBrokerWebScraper:
    """Scrape position data from Israeli broker web pages."""

    def __init__(self, config: Dict[str, Any]):
        """
        Initialize web scraper.

        Args:
            config: Configuration dict with broker-specific settings
        """
        self.config = config
        self.broker = config.get('broker')
        self.login_url = config.get('login_url')
        self.positions_url = config.get('positions_url')
        self.credentials = config.get('credentials')
        self.driver = None
        self.session = None  # For requests/BeautifulSoup approach

    def initialize_browser(self, headless: bool = True):
        """Initialize browser (Selenium or Playwright)."""
        # Option 1: Selenium (more common)
        from selenium.webdriver.chrome.options import Options
        chrome_options = Options()
        if headless:
            chrome_options.add_argument('--headless')
        chrome_options.add_argument('--no-sandbox')
        chrome_options.add_argument('--disable-dev-shm-usage')

        self.driver = webdriver.Chrome(options=chrome_options)

        # Option 2: Playwright (modern alternative)
        # from playwright.sync_api import sync_playwright
        # playwright = sync_playwright().start()
        # self.browser = playwright.chromium.launch(headless=headless)
        # self.page = self.browser.new_page()

    def login(self) -> bool:
        """Login to broker web portal."""
        try:
            self.driver.get(self.login_url)

            # Wait for login form
            username_field = WebDriverWait(self.driver, 10).until(
                EC.presence_of_element_located((By.ID, "username"))
            )
            password_field = self.driver.find_element(By.ID, "password")

            # Enter credentials
            username_field.send_keys(self.credentials['username'])
            password_field.send_keys(self.credentials['password'])

            # Submit form
            login_button = self.driver.find_element(By.ID, "login_button")
            login_button.click()

            # Wait for successful login
            WebDriverWait(self.driver, 10).until(
                EC.presence_of_element_located((By.ID, "portfolio"))
            )

            return True
        except Exception as e:
            logger.error(f"Login failed: {e}")
            return False

    def scrape_positions(self) -> List[Position]:
        """Scrape position data from broker web page."""
        try:
            # Navigate to positions page
            self.driver.get(self.positions_url)

            # Wait for positions table to load
            positions_table = WebDriverWait(self.driver, 10).until(
                EC.presence_of_element_located((By.ID, "positions_table"))
            )

            # Extract position data
            positions = []
            rows = positions_table.find_elements(By.TAG_NAME, "tr")

            for row in rows[1:]:  # Skip header row
                cells = row.find_elements(By.TAG_NAME, "td")
                if len(cells) >= 4:
                    position = Position(
                        symbol=cells[0].text,
                        quantity=float(cells[1].text),
                        cost_basis=float(cells[2].text),
                        current_price=float(cells[3].text),
                        currency=cells[4].text if len(cells) > 4 else "ILS"
                    )
                    positions.append(position)

            return positions
        except Exception as e:
            logger.error(f"Failed to scrape positions: {e}")
            return []

    def scrape_positions_playwright(self) -> List[Position]:
        """Alternative: Scrape using Playwright (more modern)."""
        from playwright.sync_api import sync_playwright

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page()

            # Login
            page.goto(self.login_url)
            page.fill('#username', self.credentials['username'])
            page.fill('#password', self.credentials['password'])
            page.click('#login_button')
            page.wait_for_selector('#portfolio')

            # Scrape positions
            page.goto(self.positions_url)
            page.wait_for_selector('#positions_table')

            positions = page.evaluate("""
                () => {
                    const rows = Array.from(document.querySelectorAll('#positions_table tr'));
                    return rows.slice(1).map(row => {
                        const cells = row.querySelectorAll('td');
                        return {
                            symbol: cells[0].textContent,
                            quantity: parseFloat(cells[1].textContent),
                            cost_basis: parseFloat(cells[2].textContent),
                            current_price: parseFloat(cells[3].textContent),
                            currency: cells[4]?.textContent || 'ILS'
                        };
                    });
                }
            """)

            browser.close()
            return [Position(**p) for p in positions]

    def close(self):
        """Close browser."""
        if self.driver:
            self.driver.quit()
```

**Browser Automation Options:**

1. **Selenium WebDriver:**
   - Mature, widely used
   - Supports Chrome, Firefox, Edge
   - Requires browser driver installation
   - Dependencies: `selenium`, browser driver (ChromeDriver, GeckoDriver)

2. **Playwright:**
   - Modern, faster than Selenium
   - Built-in browser binaries
   - Better handling of modern web apps
   - Dependencies: `playwright`

**Web Scraping Configuration:**

```json
{
  "israeli_broker_import": {
    "web_scraping": {
      "brokers": {
        "poalim": {
          "login_url": "https://www.bankhapoalim.co.il/wps/portal/Home",
          "positions_url": "https://www.bankhapoalim.co.il/wps/portal/Home/PrivateBanking/Portfolio",
          "selectors": {
            "username": "#username",
            "password": "#password",
            "login_button": "#loginBtn",
            "positions_table": "#positionsTable",
            "position_rows": "tr.position-row",
            "symbol": "td.symbol",
            "quantity": "td.quantity",
            "cost_basis": "td.cost-basis",
            "current_price": "td.current-price"
          },
          "browser": "chrome",
          "headless": true,
          "wait_timeout": 10
        }
      }
    }
  }
}
```

**Security Considerations:**

- Store credentials securely (environment variables, encrypted config)
- Respect broker Terms of Service
- Implement rate limiting to avoid being blocked
- Handle CAPTCHA challenges (may require manual intervention)
- Use session cookies for authentication persistence

## Data Models

### Position Data Model

```python
# python/integration/models.py
from dataclasses import dataclass
from datetime import datetime
from typing import Optional
from enum import Enum

class PositionSource(Enum):
    IBKR = "ibkr"
    ISRAELI_BROKER_EXCEL = "israeli_excel"
    ISRAELI_BROKER_RTD = "israeli_rtd"
    ISRAELI_BROKER_DDE = "israeli_dde"
    ISRAELI_BROKER_WEB = "israeli_web"

@dataclass
class Position:
    """Standardized position data model."""
    symbol: str
    quantity: float
    cost_basis: float
    current_price: float
    currency: str  # USD, ILS, etc.
    broker: str
    source: PositionSource
    account_id: Optional[str] = None
    last_updated: Optional[datetime] = None
    unrealized_pnl: Optional[float] = None

    # TASE-specific fields
    exchange: Optional[str] = None  # "TASE", "NYSE", etc.
    instrument_type: Optional[str] = None  # "stock", "option", "future", "bond", "etf"
    underlying: Optional[str] = None  # For derivatives: underlying asset (TA-35, USD/ILS, stock symbol)
    strike: Optional[float] = None  # For options/futures
    expiration_date: Optional[datetime] = None  # For options/futures
    option_type: Optional[str] = None  # "call", "put" for options

    def calculate_pnl(self) -> float:
        """Calculate unrealized P&L."""
        return (self.current_price - self.cost_basis) * self.quantity

    def get_market_value_usd(self, fx_rate: float) -> float:
        """Convert market value to USD."""
        market_value_local = self.current_price * self.quantity
        if self.currency == "USD":
            return market_value_local
        return market_value_local * fx_rate

    def is_tase_instrument(self) -> bool:
        """Check if position is TASE-listed."""
        return self.exchange and "TASE" in self.exchange.upper()

    def is_tase_derivative(self) -> bool:
        """Check if position is a TASE derivative (option/future)."""
        return (self.is_tase_instrument() and
                self.instrument_type in ["option", "future"])

    def get_tase_index_type(self) -> Optional[str]:
        """Get TASE index type if this is an index derivative."""
        if not self.is_tase_derivative():
            return None

        underlying_upper = (self.underlying or "").upper()
        if "TA-35" in underlying_upper or "TA35" in underlying_upper:
            return "TA-35"
        elif "TA-125" in underlying_upper or "TA125" in underlying_upper:
            return "TA-125"
        elif "TA-90" in underlying_upper or "TA90" in underlying_upper:
            return "TA-90"
        elif "BANKS" in underlying_upper or "BANKS5" in underlying_upper:
            return "TA-Banks5"

        return None
```

## Integration with Portfolio Allocation Manager

### Portfolio Aggregation

```python
# python/integration/portfolio_aggregator.py
class PortfolioAggregator:
    """Aggregate positions from multiple sources (IBKR + Israeli brokers)."""

    def __init__(self, ibkr_client, israeli_importers: List):
        self.ibkr_client = ibkr_client
        self.israeli_importers = israeli_importers
        self.fx_client = None  # For ILS/USD conversion

    def get_all_positions(self) -> List[Position]:
        """Get positions from all sources."""
        all_positions = []

        # Get IBKR positions
        ibkr_positions = self.ibkr_client.get_portfolio_positions()
        all_positions.extend(ibkr_positions)

        # Get Israeli broker positions
        for importer in self.israeli_importers:
            try:
                positions = importer.get_positions()
                # Convert ILS to USD if needed
                for pos in positions:
                    if pos.currency == "ILS":
                        pos.market_value_usd = pos.get_market_value_usd(
                            self.get_ils_usd_rate()
                        )
                all_positions.extend(positions)
            except Exception as e:
                logger.error(f"Failed to import from {importer}: {e}")

        return all_positions

    def get_total_portfolio_value(self) -> float:
        """Calculate total portfolio value (USD)."""
        positions = self.get_all_positions()
        total_value = 0.0

        for pos in positions:
            if pos.currency == "USD":
                total_value += pos.current_price * pos.quantity
            else:
                # Convert to USD
                fx_rate = self.get_fx_rate(pos.currency, "USD")
                total_value += pos.current_price * pos.quantity * fx_rate

        return total_value

    def get_fx_rate(self, from_currency: str, to_currency: str) -> float:
        """Get foreign exchange rate."""
        # Use IBKR API or external FX service
        # For ILS/USD, may use Bank of Israel or IBKR rates
        pass
```

### Configuration Structure

```json
{
  "israeli_broker_import": {
    "enabled": true,
    "import_methods": {
      "excel_file": {
        "enabled": true,
        "file_paths": [
          "/path/to/poalim_positions.xlsx",
          "/path/to/leumi_positions.csv"
        ],
        "poll_interval_seconds": 300
      },
      "excel_rtd": {
        "enabled": false,
        "excel_file_path": "C:/path/to/rtd_workbook.xlsx",
        "range_names": {
          "positions": "PositionsData"
        }
      },
      "excel_dde": {
        "enabled": false,
        "server_name": "BrokerDDE",
        "topic": "Positions"
      },
      "web_scraping": {
        "enabled": false,
        "broker_configs": {
          "poalim": {
            "credentials": {
              "username_env": "POALIM_USERNAME",
              "password_env": "POALIM_PASSWORD"
            }
          }
        },
        "scrape_interval_seconds": 600
      }
    },
    "fx_rates": {
      "ils_usd_source": "ibkr",  // or "boi" (Bank of Israel)
      "update_interval_seconds": 3600
    },
    "data_storage": {
      "cache_file": "~/.config/ib_box_spread/israeli_positions_cache.json",
      "cache_ttl_seconds": 300
    }
  }
}
```

## TASE Instrument Handling

### TASE Index Options

**Supported Indices:**

- **TA-35:** Main Israeli stock index (35 largest companies)
- **TA-125:** Broader Israeli stock index (125 companies)
- **TA-90:** Extended index (90 companies) - includes TA-35 + next 55
- **TA-Banks5:** Banking sector index (5 major banks)

**Position Classification:**

```python
def classify_tase_position(position: Position) -> str:
    """Classify TASE position by type."""
    if position.is_tase_derivative():
        index_type = position.get_tase_index_type()
        if index_type:
            return f"TASE {index_type} {position.instrument_type}"

        # Currency options
        if position.underlying and ("USD" in position.underlying or "EUR" in position.underlying):
            return f"TASE Currency {position.instrument_type}"

        # Stock options
        if position.instrument_type == "option":
            return "TASE Stock Option"

        return f"TASE {position.instrument_type}"

    return "TASE Stock/Bond/ETF"
```

### TASE Derivatives Greeks Calculation

**TASE Options Greeks:**

- Use Black-Scholes formulas (similar to US options)
- Adjust for TASE-specific features:
  - ILS-denominated underlying prices
  - TASE volatility conventions
  - MAOF clearing house margin requirements

**TASE Futures Greeks:**

- Futures delta = 1.0 (futures track underlying 1:1)
- Futures gamma = 0 (linear relationship)
- Futures vega ≈ 0 (unless considering volatility)
- Futures theta ≈ 0 (no time decay for futures)
- Futures rho: Interest rate sensitivity based on time to expiration

**Integration with Portfolio Greeks:**

- TASE derivatives included in portfolio Greeks calculation
- Currency conversion: ILS → USD for Greeks aggregation
- See `docs/PORTFOLIO_GREEKS_SYSTEM.md` for Greeks calculation methodology

## Implementation Roadmap

### Phase 1: Static File Import (Week 1-2)

- [ ] Design position data models
- [ ] Implement Excel/CSV file parser
- [ ] Create field mapping configuration system
- [ ] **Add TASE instrument classification**
- [ ] **Handle TASE-specific fields (exchange, instrument_type, underlying)**
- [ ] Implement data validation
- [ ] Add currency conversion (ILS → USD)
- [ ] Integrate with PortfolioAllocationManager
- [ ] Create tests for file parsing

### Phase 2: Excel RTD Integration (Week 3-4)

- [ ] Research broker RTD server requirements
- [ ] Implement RTD client using xlwings
- [ ] Create Excel workbook template for RTD connections
- [ ] Implement real-time position update mechanism
- [ ] Add error handling for RTD connection failures
- [ ] Create documentation for broker setup

### Phase 3: Web Scraping (Week 5-6)

- [ ] Choose browser automation library (Selenium vs Playwright)
- [ ] Implement login automation for Israeli brokers
- [ ] Create position extraction logic
- [ ] Implement session management and cookie handling
- [ ] Add CAPTCHA handling (manual intervention)
- [ ] Create broker-specific scraper configurations
- [ ] Add rate limiting and retry logic

### Phase 4: Portfolio Aggregation (Week 7)

- [ ] Implement PortfolioAggregator class
- [ ] Integrate IBKR + Israeli broker positions
- [ ] Add FX rate conversion (ILS/USD)
- [ ] Update PortfolioAllocationManager to use aggregated positions
- [ ] Create unified position reporting
- [ ] Add position reconciliation logic

### Phase 5: Testing & Validation (Week 8)

- [ ] Test file import with sample broker exports
- [ ] Test RTD connection (if broker provides RTD)
- [ ] Test web scraping (with test broker accounts)
- [ ] Validate position data accuracy
- [ ] Test currency conversion accuracy
- [ ] Integration testing with portfolio allocation
- [ ] Performance testing for large portfolios

## Dependencies

### Python Libraries

**Core:**

- `pandas` - Data manipulation for Excel/CSV parsing
- `openpyxl` - Excel file reading/writing
- `xlrd` - Legacy Excel support (if needed)

**Excel Automation (RTD/DDE):**

- `xlwings` - Excel COM automation (Windows)
- `pywin32` - Windows COM/DDE support

**Web Scraping:**

- `selenium` - Browser automation (alternative)
- `playwright` - Modern browser automation (recommended)
- `beautifulsoup4` - HTML parsing (for non-JS pages)
- `requests` - HTTP client

**Configuration & Utilities:**

- `pydantic` - Data validation (already in codebase)
- `python-dotenv` - Environment variable management

### System Requirements

**Windows (for RTD/DDE):**

- Excel installed
- Windows COM support
- Broker RTD add-in (if using RTD)

**All Platforms (for file import/web scraping):**

- Python 3.12+
- Browser driver (ChromeDriver for Selenium, or Playwright browsers)

## Error Handling & Validation

### Data Validation

```python
def validate_position(position: Position) -> List[str]:
    """Validate position data and return list of errors."""
    errors = []

    if not position.symbol:
        errors.append("Symbol is required")
    if position.quantity <= 0:
        errors.append("Quantity must be positive")
    if position.cost_basis < 0:
        errors.append("Cost basis cannot be negative")
    if position.current_price < 0:
        errors.append("Current price cannot be negative")
    if position.currency not in ["USD", "ILS", "EUR", "GBP"]:
        errors.append(f"Unsupported currency: {position.currency}")

    return errors
```

### Error Handling Strategies

1. **File Import Errors:**
   - File not found → Log error, use cached data if available
   - Invalid format → Log error, skip file, continue with other sources
   - Missing required fields → Log warning, use defaults where possible

2. **RTD/DDE Connection Errors:**
   - Connection failed → Retry with exponential backoff
   - Excel not running → Log error, fall back to file import
   - Data timeout → Log warning, use last known good data

3. **Web Scraping Errors:**
   - Login failed → Log error, require manual intervention
   - Page structure changed → Log error, alert for configuration update
   - CAPTCHA encountered → Pause scraping, alert user
   - Rate limiting → Implement exponential backoff

### Caching Strategy

- Cache imported positions to file
- Use cached data if import fails
- TTL-based cache invalidation
- Version cache by import timestamp

## Security & Compliance

### Credentials Management

```python
# Store credentials in environment variables or encrypted config
import os
from cryptography.fernet import Fernet

class CredentialManager:
    """Securely manage broker credentials."""

    def get_credentials(self, broker: str) -> Dict[str, str]:
        """Get credentials from secure storage."""
        username_env = f"{broker.upper()}_USERNAME"
        password_env = f"{broker.upper()}_PASSWORD"

        username = os.getenv(username_env)
        password = os.getenv(password_env)

        if not username or not password:
            raise ValueError(f"Credentials not found for {broker}")

        return {"username": username, "password": password}
```

### Compliance Considerations

- **Terms of Service:** Review broker ToS before web scraping
- **Rate Limiting:** Implement delays to avoid overwhelming broker servers
- **Data Privacy:** Encrypt stored credentials and position data
- **Audit Logging:** Log all import operations for compliance

## Usage Examples

### Static File Import

```python
from python.integration.israeli_broker_importer import ExcelFileImporter

importer = ExcelFileImporter(config)
positions = importer.import_positions("/path/to/poalim_positions.xlsx")
```

### Excel RTD Import

```python
from python.integration.excel_rtd_client import ExcelRTDClient

rtd_client = ExcelRTDClient(
    excel_file_path="C:/path/to/rtd_workbook.xlsx",
    rtd_topic="BrokerPositions"
)

if rtd_client.connect():
    def on_positions_update(new_positions):
        # Update portfolio allocation
        portfolio_manager.update_positions(new_positions)

    rtd_client.monitor_positions(on_positions_update, interval=5)
```

### Web Scraping Import

```python
from python.integration.web_scraper import IsraeliBrokerWebScraper

scraper = IsraeliBrokerWebScraper({
    "broker": "poalim",
    "login_url": "https://...",
    "positions_url": "https://...",
    "credentials": {"username": "...", "password": "..."}
})

scraper.initialize_browser(headless=True)
if scraper.login():
    positions = scraper.scrape_positions()
```

## Testing

### Unit Tests

- File parsing (Excel/CSV)
- Data normalization
- Field mapping
- Currency conversion
- Position validation

### Integration Tests

- End-to-end file import
- RTD connection (requires Excel)
- Web scraping (requires test broker account)
- Portfolio aggregation
- FX rate conversion

### Mock Data

- Sample Excel files for each broker format
- Mock RTD responses
- Mock web pages for scraping tests

## Future Enhancements

1. **Automated File Monitoring:**
   - Watch file system for new exports
   - Google Drive API integration
   - OneDrive API integration

2. **Advanced Web Scraping:**
   - CAPTCHA solving integration
   - Multi-factor authentication support
   - Session persistence across restarts

3. **Position Reconciliation:**
   - Compare IBKR vs Israeli broker positions
   - Identify discrepancies
   - Alert on position mismatches

4. **Real-Time Updates:**
   - WebSocket connections (if broker supports)
   - Push notifications for position changes
   - Automatic portfolio rebalancing triggers

## References

- [Pandas Excel Documentation](https://pandas.pydata.org/docs/reference/api/pandas.read_excel.html)
- [Xlwings Documentation](https://docs.xlwings.org/)
- [Selenium WebDriver](https://www.selenium.dev/documentation/)
- [Playwright Documentation](https://playwright.dev/python/)
- [TASE Knowledge Center - Securities & Derivatives](https://www.tase.co.il/en/content/knowledge_center/securities_derivatives)
- [Portfolio Greeks System](research/architecture/PORTFOLIO_GREEKS_SYSTEM.md)

---

**TASE Resources:**

- **TASE Index Options:** TA-35, TA-125, TA-90, TA-Banks5
- **TASE Currency Options:** ILS/USD, ILS/EUR
- **TASE Futures:** TA-35, TA-90, TA-Banks5 (relaunched September 2024)
- **MAOF Clearing House:** All TASE derivatives cleared through MAOF

**Next Steps:**

1. Review and approve design
2. Select import methods to implement first
3. Add TASE instrument classification logic
4. Begin Phase 1 implementation (static file import)
