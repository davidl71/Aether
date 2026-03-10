# Discount Bank Integration for PWA

This guide shows how to connect the PWA to Discount Bank account data via reconciliation files.

## Overview

The Discount Bank service reads reconciliation files (Osh Matching format) and provides account balance and transaction data to the PWA.

**Account:** 535-0000-276689
**Currency:** ILS (Israeli Shekel)
**Interest Rates:**
- **Credit Balance (Positive):** 3.00% per year
- **Debit Balance (Negative):** 10.30% per year ⚠️ Avoid negative balances

## Prerequisites

1. **Discount Bank Reconciliation File:** Download from Discount Bank Osh Matching service
   - File format: Fixed-width text (see `docs/DISCOUNT_BANK_RECONCILIATION_FORMAT.md`)
   - Default location: `~/Downloads/DISCOUNT.dat`
2. **Python Dependencies:** The service requires `fastapi`, `uvicorn`, and `pydantic`

## Quick Start

### 1. Set File Path

```bash
export DISCOUNT_BANK_FILE_PATH="~/Downloads/DISCOUNT.dat"
# Or use absolute path
export DISCOUNT_BANK_FILE_PATH="/Users/davidlowes/Downloads/DISCOUNT.dat"
```

### 2. Run Service

```bash
./web/scripts/run-discount-bank-service.sh
```

The service will:
- Start on port 8003 (configurable via `PORT` environment variable)
- Read the latest Discount Bank file from the configured path
- Provide REST API endpoints for balance and transaction data

### 3. Test Service

```bash
# Health check
curl http://localhost:8003/api/health

# Get balance
curl http://localhost:8003/api/balance

# Get transactions
curl http://localhost:8003/api/transactions
```

## API Endpoints

### GET /api/health

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "service": "discount_bank",
  "file_path": "~/Downloads/DISCOUNT.dat",
  "file_found": true,
  "file_path_resolved": "/Users/davidlowes/Downloads/DISCOUNT.dat"
}
```

### GET /api/balance

Get current account balance from the latest reconciliation file.

**Response:**
```json
{
  "account": "535-0000-276689",
  "balance": 15570.34,
  "currency": "ILS",
  "balance_date": "2025-11-16",
  "credit_rate": 0.03,
  "debit_rate": 0.103,
  "branch_number": "535",
  "section_number": "0000",
  "account_number": "276689"
}
```

### GET /api/transactions

Get recent transactions (currently returns empty list - implementation pending).

**Query Parameters:**
- `limit` (optional): Maximum number of transactions to return (default: 20)

**Response:**
```json
{
  "account": "535-0000-276689",
  "transactions": [],
  "total_count": 0
}
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DISCOUNT_BANK_FILE_PATH` | Path to Discount Bank reconciliation file | `~/Downloads/DISCOUNT.dat` |
| `DISCOUNT_BANK_CREDIT_RATE` | Credit interest rate (positive balance) | `0.03` (3%) |
| `DISCOUNT_BANK_DEBIT_RATE` | Debit interest rate (negative balance) | `0.103` (10.30%) |
| `PORT` | Service port | `8003` |

### File Path Resolution

The service supports multiple file path formats:

1. **Direct file path:**
   ```bash
   export DISCOUNT_BANK_FILE_PATH="/path/to/DISCOUNT.dat"
   ```

2. **Directory path:** Service will find the most recent `DISCOUNT*.dat` file
   ```bash
   export DISCOUNT_BANK_FILE_PATH="~/Downloads"
   ```

3. **Pattern matching:** Supports glob patterns
   ```bash
   export DISCOUNT_BANK_FILE_PATH="~/Downloads/DISCOUNT*.dat"
   ```

## Integration with PWA

The Discount Bank service is automatically launched when using the unified launch script:

```bash
./web/scripts/launch-all-pwa-services.sh
```

This starts all PWA services including:
- Web frontend (port 5173)
- Alpaca service (port 8000)
- IB service (port 8000, optional)
- **Discount Bank service (port 8003)** ✨

## Troubleshooting

### File Not Found

**Error:** `Discount Bank file not found`

**Solutions:**
1. Check file path: `echo $DISCOUNT_BANK_FILE_PATH`
2. Verify file exists: `ls -la ~/Downloads/DISCOUNT*.dat`
3. Update path: `export DISCOUNT_BANK_FILE_PATH="/correct/path/DISCOUNT.dat"`

### Service Won't Start

**Error:** Port 8003 already in use

**Solutions:**
1. Use different port: `PORT=8004 ./web/scripts/run-discount-bank-service.sh`
2. Stop existing service: `lsof -ti :8003 | xargs kill`

### Invalid File Format

**Error:** `Failed to parse file`

**Solutions:**
1. Verify file is from Discount Bank Osh Matching service
2. Check file encoding (should be UTF-8 or Windows-1255)
3. Ensure file has header records (lines starting with "00")

## Architecture

```
┌─────────────────┐
│  PWA Frontend   │
│   (React/Vite)  │
└────────┬────────┘
         │ HTTP
         ▼
┌─────────────────┐
│ Discount Bank   │
│   Service       │
│  (FastAPI)      │
│  Port: 8003     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ DISCOUNT.dat    │
│ Reconciliation  │
│     File        │
└─────────────────┘
```

## Future Enhancements

- [ ] Real-time file watching (auto-refresh on file update)
- [ ] Transaction history endpoint (full implementation)
- [ ] Integration with Rust parser for better parsing
- [ ] File upload UI in PWA
- [ ] Multiple account support
- [ ] Historical balance tracking

## Related Documentation

- **Format Specification:** `docs/DISCOUNT_BANK_RECONCILIATION_FORMAT.md`
- **Rust Parser:** `agents/backend/crates/discount_bank_parser/`
- **Investment Strategy:** `docs/INVESTMENT_STRATEGY_FRAMEWORK.md` (Cash Management section)
