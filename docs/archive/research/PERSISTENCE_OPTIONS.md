# Research: Persistence Options

## Overview

Document the persistence options available in the Rust backend.

## Active Persistence Options

### SQLite

**Used for:** Loans, Discount Bank transactions, Ledger

**Implementation:**

- `loans.rs` - LoanRepository with SQLite
- Discount bank parser outputs to SQLite
- Ledger transactions

**Pros:**

- Relational data (loans, transactions)
- Mature, reliable
- Simple setup

**Cons:**

- Not optimized for time-series

### NATS KV

**Used for:** Live state snapshots, key-value storage

**Implementation:**

- `snapshot_publisher.rs` - writes to KV bucket
- Live state storage

**Pros:**

- Already in the stack
- Fast for current state
- Integrated with messaging

**Cons:**

- Not for historical queries

## Optional: QuestDB

**Used for:** Historical market data (optional)

**Implementation:**

- ILP (InfluxDB Line Protocol) writes
- Only if `QUESTDB_ILP_ADDR` env var set

**Pros:**

- High-performance time-series
- SQL queries
- Great for backtesting

**Cons:**

- Additional dependency
- Not needed for current use cases

## Summary

| Data Type | Recommended Storage |
|-----------|-------------------|
| Loans/liabilities | SQLite |
| Discount Bank transactions | SQLite |
| Ledger | SQLite |
| Live state (snapshots) | NATS KV |
| Historical market data | QuestDB (optional) |

## Decision

**Keep QuestDB as optional** - minimal code, can enable later for historical analysis/backtesting.
