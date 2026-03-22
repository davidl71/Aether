# Swiftness Data Import System Architecture

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document defines the architecture for importing and updating Swiftness (Israeli Pension Clearing House) position data from Excel files, with validity date tracking to ensure data accuracy.

## System Components

### 1. Data Models (`python/integration/swiftness_models.py`)

Python dataclasses representing Swiftness data structures:

```python
from dataclasses import dataclass
from datetime import datetime
from typing import Optional

@dataclass
class InsuranceCoverage:
    """Insurance coverage information (Sheet 1)"""
    coverage_type: str
    plan_name: str
    company: str
    payment_recipient: str
    one_time_amount: float
    monthly_benefit: float
    policy_number: str  # Unique identifier
    file_modified_date: datetime  # Use file mod date as validity

@dataclass
class DepositRecord:
    """Monthly deposit tracking record (Sheet 2)"""
    product_type: str
    company: str
    policy_number: str  # Unique identifier
    value_date: datetime  # VALIDITY DATE
    salary_month: datetime
    employer_name: str
    employee_deposit: float  # ILS
    employer_deposit: float  # ILS
    employer_severance: float  # ILS

@dataclass
class ProductDetails:
    """Product/policy detailed information (Sheet 3)"""
    product_name: str
    company: str
    policy_number: str  # Unique identifier
    status: str  # Active/Inactive
    total_savings: float  # ILS
    next_withdrawal_date: Optional[datetime]
    projected_savings_no_premiums: float
    monthly_benefit_no_premiums: float
    projected_savings: float
    monthly_benefit: float
    expected_pension_rate: float
    management_fee_on_deposits: float
    management_fee_annual: float
    ytd_return: float
    employee_deposits: float
    employer_deposits: float
    first_join_date: Optional[datetime]
    plan_opening_date: Optional[datetime]
    data_accuracy_date: datetime  # VALIDITY DATE
```

### 2. Excel Parser (`python/integration/swiftness_parser.py`)

Parser module to extract data from Swiftness Excel files:

```python
class SwiftnessParser:
    """Parses Swiftness Excel files and extracts position data."""

    def __init__(self, file_path: Path):
        self.file_path = Path(file_path)
        self.workbook = None

    def parse(self) -> SwiftnessData:
        """Parse all sheets and return structured data."""
        # Returns SwiftnessData containing all three sheet types

    def _parse_insurance_coverage(self, sheet) -> List[InsuranceCoverage]:
        """Parse Sheet 1: Insurance Coverage."""

    def _parse_deposit_tracking(self, sheet) -> List[DepositRecord]:
        """Parse Sheet 2: Deposit Tracking."""

    def _parse_product_details(self, sheet) -> List[ProductDetails]:
        """Parse Sheet 3: Product Details."""

    def _parse_date(self, date_str: str) -> datetime:
        """Parse M/D/YYYY date format."""
```

### 3. Storage Layer (`python/integration/swiftness_storage.py`)

Manages persistence of Swiftness positions:

```python
class SwiftnessStorage:
    """Manages storage and retrieval of Swiftness positions."""

    def __init__(self, storage_path: Path):
        self.storage_path = Path(storage_path)
        self.positions_file = self.storage_path / "swiftness_positions.json"

    def load_positions(self) -> SwiftnessPositions:
        """Load existing positions from JSON file."""

    def save_positions(self, positions: SwiftnessPositions) -> None:
        """Save positions to JSON file."""

    def get_position_by_policy(self, policy_number: str) -> Optional[ProductDetails]:
        """Get product details by policy number."""
```

### 4. Update Logic (`python/integration/swiftness_updater.py`)

Handles position updates based on validity dates:

```python
class SwiftnessUpdater:
    """Updates Swiftness positions respecting validity dates."""

    def __init__(self, storage: SwiftnessStorage):
        self.storage = storage

    def import_file(self, file_path: Path) -> ImportResult:
        """Import new Excel file and update positions."""
        # 1. Parse file
        # 2. Load existing positions
        # 3. Compare validity dates
        # 4. Update positions
        # 5. Save updated positions

    def _should_update_product(
        self,
        existing: ProductDetails,
        new: ProductDetails
    ) -> bool:
        """Check if product should be updated based on validity date."""
        return new.data_accuracy_date > existing.data_accuracy_date

    def _should_update_deposit(
        self,
        existing_deposits: List[DepositRecord],
        new_deposit: DepositRecord
    ) -> bool:
        """Check if deposit record should be added/updated."""
        # Deposits are additive - add new records, don't overwrite
        # But check for duplicates by policy + value_date
```

## Data Flow

```
Excel File (Swiftness Export)
    ↓
SwiftnessParser.parse()
    ↓
SwiftnessData (structured data models)
    ↓
SwiftnessUpdater.import_file()
    ↓
    ├─→ Load existing positions from JSON
    ├─→ Compare validity dates
    ├─→ Update/add positions
    └─→ Save to JSON
    ↓
SwiftnessStorage.save_positions()
    ↓
swiftness_positions.json
```

## Update Rules

### Product Details (Sheet 3)

- **Match by:** Policy number
- **Update condition:** New data has later `data_accuracy_date`
- **Action:** Replace entire product record if validity date is newer
- **Conflict resolution:** Always prefer data with later validity date

### Deposit Records (Sheet 2)

- **Match by:** Policy number + value date
- **Update condition:** New deposit record doesn't exist
- **Action:** Add new deposit records (deposits are additive/historical)
- **Conflict resolution:** If same policy + value_date exists, skip (already imported)

### Insurance Coverage (Sheet 1)

- **Match by:** Policy number
- **Update condition:** File modification date is newer
- **Action:** Update coverage information
- **Note:** No validity date in sheet, use file mod date

## Storage Format

### JSON Structure

```json
{
  "version": "1.0.0",
  "last_updated": "2025-11-18T12:35:27Z",
  "products": [
    {
      "policy_number": "40230914",
      "product_name": "פנסיה חדשה כללית",
      "company": "מגדל מקפת קרנות פנסיה וקופות גמל בע\"מ",
      "status": "פעיל",
      "total_savings": 137389.0,
      "data_accuracy_date": "2025-09-30T00:00:00Z",
      "...": "..."
    }
  ],
  "deposits": [
    {
      "policy_number": "10118590",
      "value_date": "2025-07-11T00:00:00Z",
      "employee_deposit": 392.8,
      "employer_deposit": 1178.4,
      "...": "..."
    }
  ],
  "insurance_coverage": [
    {
      "policy_number": "55501335",
      "coverage_type": "כיסוי למקרה מוות",
      "file_modified_date": "2025-11-18T12:35:27Z",
      "...": "..."
    }
  ]
}
```

## Integration Points

### 1. Investment Strategy Framework

Swiftness positions integrate with the Investment Strategy Framework:

- **Currency Conversion:** Convert ILS to USD for unified portfolio view
- **Position Aggregation:** Combine with IBKR positions for total portfolio value
- **Net Portfolio Calculation:** Include Swiftness positions in net portfolio value

### 2. Portfolio Tracking

- Swiftness positions stored separately from IBKR positions
- Policy numbers serve as unique identifiers
- Validity dates determine when refresh is needed

### 3. Configuration

Add to `config.json`:

```json
{
  "swiftness": {
    "enabled": true,
    "storage_path": "~/.config/ib_box_spread/swiftness",
    "currency_conversion": {
      "ils_to_usd_rate": "auto",  // or fixed rate
      "rate_source": "yahoo_finance"  // or "manual"
    },
    "auto_refresh": false,
    "refresh_interval_days": 30
  }
}
```

## Error Handling

### File Parsing Errors

- **Corrupted file:** Log error, skip import
- **Missing sheets:** Log warning, parse available sheets
- **Invalid dates:** Log warning, use file mod date as fallback

### Update Conflicts

- **Stale data:** Log warning if imported data has older validity date
- **Missing validity date:** Use file modification date
- **Duplicate deposits:** Skip if policy + value_date already exists

## Testing Strategy

### Unit Tests

- Parser tests for each sheet type
- Date parsing tests (M/D/YYYY format)
- Validity date comparison tests
- Update logic tests

### Integration Tests

- End-to-end import workflow
- Position update scenarios
- Conflict resolution tests

## Future Enhancements

1. **Database Migration:** Move from JSON to SQLite/PostgreSQL
2. **Automated Refresh:** Schedule periodic imports
3. **Change Detection:** Track what changed between imports
4. **Validation:** Validate data against known constraints
5. **Reporting:** Generate position summaries and reports

## File Structure

```
python/integration/
├── swiftness_models.py      # Data models (dataclasses)
├── swiftness_parser.py      # Excel parser
├── swiftness_storage.py     # JSON storage layer
└── swiftness_updater.py     # Update logic

docs/
└── SWIFTNESS_IMPORT_DESIGN.md  # This file

~/.config/ib_box_spread/swiftness/
└── swiftness_positions.json    # Stored positions
```

## Dependencies

- `xlrd`: Excel file reading (.xls format)
- `dataclasses`: Data models (Python 3.7+)
- `pathlib`: File path handling
- `json`: Position storage
- `datetime`: Date handling
