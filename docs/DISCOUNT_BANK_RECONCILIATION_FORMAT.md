# Discount Bank Reconciliation File Format (Osh Matching)

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Format Specification

## Overview

Discount Bank provides a bank reconciliation service (התאמות בנקים עו"ש - Osh Matching) that allows businesses to download transaction reports in multiple formats for account reconciliation.

**Service Details:**
- **Report Range:** Up to 1 year back
- **Single Report Range:** Maximum 31 calendar days
- **File Size Limit:** 3.5MB (larger reports require reduced date range)
- **Available Formats:**
  - Hashavshevet (old/new accounting software format)
  - Excel (.xlsx)
  - Text (fixed-width format)

**Source:** [Discount Bank Osh Matching Service](https://www.discountbank.co.il/business/direct-banking/about/oshmatching/)

## Text Format Specification

The text format uses fixed-width records with three record types:

### Record Types

1. **Header Record (56 characters):** Daily account summary
2. **Transaction Record (47 characters):** Individual transaction
3. **Summary Record (107 characters):** End-of-file account summary

### Line Endings

All records end with CR + LF (2 characters: `\r\n`)

---

## Header Record (Record Code: 00)

**Length:** 56 characters
**Purpose:** Daily account summary for each business day

| Field Name | Position | Length | Format | Description |
|------------|----------|--------|--------|-------------|
| Record Code | 1-2 | 2 | Fixed "00" | Record type identifier |
| Bank Number | 3 | 1 | "0" or "1" | "0" = Discount Bank, "1" = Other banks |
| Branch Number | 4-6 | 3 | Numeric | Rightmost 3 digits of branch number |
| Section Number | 7-10 | 4 | Numeric | Account section |
| Currency Code | 11-12 | 2 | Alphanumeric | Currency identifier (e.g., "01" = ILS) |
| Account Number | 13-18 | 6 | Numeric | Old format account number (rightmost 6 digits) |
| Opening Balance | 19-32 | 14 | Numeric | Rightmost 14 digits, last 2 = cents, no decimal point |
| Opening Sign | 33 | 1 | "-" or space | "-" for negative, space for positive |
| Closing Balance | 34-47 | 14 | Numeric | Rightmost 14 digits, last 2 = cents, no decimal point |
| Closing Sign | 48 | 1 | "-" or space | "-" for negative, space for positive |
| Transaction Date | 49-54 | 6 | YYMMDD | Date in 2-digit year format |
| Filler | 55-56 | 2 | Spaces | Padding |

**Example:**
```
0001234567890123456789012345678901234567890123456789012345
```

---

## Transaction Record (Record Code: 01)

**Length:** 47 characters
**Purpose:** Individual transaction entry

| Field Name | Position | Length | Format | Description |
|------------|----------|--------|--------|-------------|
| Record Code | 1-2 | 2 | Fixed "01" | Record type identifier |
| Value Date | 3-8 | 6 | YYMMDD | Transaction value date |
| Amount | 9-20 | 12 | Numeric | Rightmost 12 digits, last 2 = cents, padded with leading zeros |
| Debit/Credit Sign | 21 | 1 | "-" or space | "-" for debit (negative), space for credit (positive) |
| Reference | 22-28 | 7 | Alphanumeric | Rightmost 7 digits of: |
| | | | | - Check number (for checks drawn) |
| | | | | - Number of checks (for deposits) |
| | | | | - Institution code (for direct debits) |
| | | | | - Reference number (from account statement) |
| Filler | 29-47 | 19 | Spaces | Padding |

**Example:**
```
01250118123456789012 1234567
```

---

## Summary Record (Record Code: 04)

**Length:** 107 characters
**Purpose:** End-of-file account summary with transaction count

| Field Name | Position | Length | Format | Description |
|------------|----------|--------|--------|-------------|
| Record Code | 1-2 | 2 | Fixed "04" | Record type identifier |
| Bank Number | 3-6 | 4 | Numeric | XX00XX format (last 2 digits of bank number) |
| Branch Number | 7-10 | 4 | Numeric | Branch number |
| Account Number | 11-20 | 10 | Numeric | New format account number |
| Transaction Counter | 21-31 | 11 | Numeric | Total number of transactions in file |
| Filler | 32-107 | 76 | Spaces | Padding |

**Example:**
```
04112345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567
```

---

## Data Format Details

### Amount Format

- **No decimal point:** Amounts stored as integers
- **Last 2 digits = cents:** Divide by 100 to get actual amount
- **Right-aligned:** Rightmost N digits used, padded with leading zeros
- **Example:** `00000000123456` = 1234.56 (12 digits, last 2 = cents)

### Date Format

- **Format:** YYMMDD (2-digit year, month, day)
- **Year Interpretation:** Typically 2000s (e.g., "25" = 2025)
- **Example:** `251118` = November 18, 2025

### Currency Codes

Common currency codes (may vary):
- `01` = ILS (Israeli Shekel)
- `02` = USD (US Dollar)
- `03` = EUR (Euro)

### Encoding

- **Hebrew Text:** Likely Windows-1255 or UTF-8 encoding
- **Numeric Fields:** ASCII digits
- **Line Endings:** CR + LF (`\r\n`)

---

## File Structure Example

```
00[Header for Day 1]
01[Transaction 1]
01[Transaction 2]
01[Transaction 3]
00[Header for Day 2]
01[Transaction 4]
01[Transaction 5]
...
04[Summary Record]
```

---

## Integration with Portfolio System

### Use Cases

1. **Cash Reconciliation:** Import bank transactions to reconcile IBKR cash positions
2. **Loan Payment Tracking:** Automatically track SHIR-based and CPI-linked loan payments
3. **Ledger Integration:** Convert bank transactions to ledger postings (see `agents/backend/crates/ledger/`)
4. **Cash Flow Forecasting:** Use historical bank data to improve cash flow predictions (see `docs/CASH_FLOW_FORECASTING_SYSTEM.md`)

### Integration Points

**Related Systems:**
- **Ledger Core Library:** `agents/backend/crates/ledger/` - Transaction and Posting models
- **Investment Strategy Framework:** `docs/INVESTMENT_STRATEGY_FRAMEWORK.md` - Cash management and loan tracking
- **Israeli Broker Import:** `docs/ISRAELI_BROKER_POSITION_IMPORT.md` - Similar import patterns

**Data Flow:**
```
Discount Bank File → Parser → Transaction Objects → Ledger Postings → Cash Flow Calculator
```

---

## Alternative: Israeli Bank Scrapers Library

**Library:** [israeli-bank-scrapers](https://github.com/eshaham/israeli-bank-scrapers)
**Language:** Node.js/TypeScript
**License:** MIT

### Overview

Automated web scraping library for Israeli banks that can fetch transaction data programmatically without manual file downloads.

**Supported Banks:**
- Discount Bank
- Bank Hapoalim
- Bank Leumi
- Mercantile
- Mizrahi
- Beinleumi
- Massad
- Bank Otsar Hahayal
- And credit card companies (Visa Cal, Max, Isracard, Amex)

### Discount Bank Scraper

**Credentials Required:**
```javascript
{
  id: <user identification number>,
  password: <user password>,
  num: <user identification code>
}
```

**Usage Example:**
```javascript
import { CompanyTypes, createScraper } from 'israeli-bank-scrapers';

const options = {
  companyId: CompanyTypes.discount,
  startDate: new Date('2025-01-01'),
  combineInstallments: false,
  showBrowser: false
};

const credentials = {
  id: 'your_id',
  password: 'your_password',
  num: 'your_num'
};

const scraper = createScraper(options);
const result = await scraper.scrape(credentials);

if (result.success) {
  result.accounts.forEach((account) => {
    console.log(`Account: ${account.accountNumber}`);
    account.txns.forEach((txn) => {
      console.log(`  ${txn.date}: ${txn.amount} ${txn.description}`);
    });
  });
}
```

### Comparison: File Format vs. Scrapers

| Aspect | Osh Matching File | Scrapers Library |
|--------|-------------------|------------------|
| **Automation** | Manual download | Fully automated |
| **Format** | Fixed-width text | JSON objects |
| **Reliability** | Stable format | Depends on website changes |
| **Security** | File-based | Requires credentials |
| **Maintenance** | No maintenance | Requires updates for site changes |
| **Integration** | Parser needed | Direct API-like access |

### Recommendation

**Use Osh Matching Files When:**
- Manual reconciliation is acceptable
- Stable, documented format is preferred
- No automation infrastructure available
- Compliance requires file-based audit trail

**Use Scrapers Library When:**
- Full automation is required
- Real-time transaction fetching needed
- Integration with Node.js/TypeScript stack
- Willing to maintain scraper updates

**Hybrid Approach:**
- Use scrapers for automated daily imports
- Use file format for manual reconciliation and audit
- Both feed into same ledger system

---

## Implementation Notes

### Parser Requirements

1. **Fixed-Width Parsing:** Extract fields by position
2. **Date Parsing:** Handle YYMMDD format with century interpretation
3. **Amount Parsing:** Convert integer format to decimal (divide by 100)
4. **Encoding Handling:** Support Windows-1255 and UTF-8
5. **Validation:** Verify record codes and field formats

### Error Handling

- **Invalid Record Codes:** Log and skip unknown records
- **Date Parsing Errors:** Use file modification date as fallback
- **Amount Parsing Errors:** Log and flag for manual review
- **Encoding Issues:** Try multiple encodings (UTF-8, Windows-1255, ISO-8859-8)

### Testing

- Test with sample files from Discount Bank
- Validate against known transaction data
- Test edge cases (negative amounts, large amounts, special characters)
- Verify currency conversion (ILS → USD) for portfolio integration

---

## References

1. [Discount Bank Osh Matching Service](https://www.discountbank.co.il/business/direct-banking/about/oshmatching/)
2. [Israeli Bank Scrapers Library](https://github.com/eshaham/israeli-bank-scrapers)
3. Investment Strategy Framework: `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`
4. Ledger Core Library: `agents/backend/crates/ledger/`
5. Cash Flow Forecasting: `docs/CASH_FLOW_FORECASTING_SYSTEM.md`
6. Israeli Broker Position Import: `docs/ISRAELI_BROKER_POSITION_IMPORT.md`

---

**Next Steps:**

1. Implement parser for fixed-width format
2. Integrate with ledger system
3. Add to cash flow forecasting
4. Consider scraper library integration for automation
