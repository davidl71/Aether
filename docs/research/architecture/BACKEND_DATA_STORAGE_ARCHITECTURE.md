# Backend Data Storage Architecture

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document defines the comprehensive data storage architecture for the investment strategy backend, covering all data types including market data, positions, orders, portfolio allocation, cash flows, Greeks, loans, and configuration.

## Current State Analysis

### Existing Storage Infrastructure

1. **In-Memory State (Rust Backend):**
   - `SystemSnapshot` stored in `Arc<RwLock<SystemSnapshot>>` (in-memory)
   - Current positions, orders, market data snapshots
   - **Limitation:** Data lost on restart, no historical persistence
   - **Location:** `agents/backend/crates/api/src/state.rs`

2. **QuestDB (Time-Series Database):**
   - **Status:** Partially integrated
   - **Purpose:** Historical market data (quotes, trades)
   - **Protocol:** InfluxDB Line Protocol (ILP) on port 9009
   - **Client:** `python/integration/questdb_client.py`
   - **Tables:** `quotes`, `trades`
   - **Limitation:** Not yet used for positions, orders, or portfolio data

3. **Configuration Storage:**
   - **Format:** TOML files
   - **Location:** `agents/backend/config/default.toml`
   - **Current:** Basic market data settings only

## Proposed Data Storage Architecture

### Multi-Layer Storage Strategy

```
┌─────────────────────────────────────────────────────────┐
│  Layer 1: In-Memory Cache (Fast Access)                │
│  - SystemSnapshot (current state)                       │
│  - Active positions, orders                            │
│  - Real-time market data                               │
│  Storage: Arc<RwLock<SystemSnapshot>>                  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 2: Operational Database (Structured Data)       │
│  - Positions (current + historical)                     │
│  - Orders (current + historical)                        │
│  - Portfolio allocation snapshots                       │
│  - Cash flow forecasts                                  │
│  - Loans/liabilities                                    │
│  - Greeks calculations                                  │
│  Storage: PostgreSQL or SQLite                         │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Time-Series Database (Market Data)           │
│  - Real-time market data (quotes, trades)               │
│  - Historical price data                                │
│  - Market data snapshots                                │
│  Storage: QuestDB                                       │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Configuration Storage                         │
│  - Investment strategy configuration                    │
│  - Portfolio allocation settings                        │
│  - Risk limits                                          │
│  - Loan configurations                                  │
│  Storage: TOML files + Environment variables           │
└─────────────────────────────────────────────────────────┘
```

## Storage Layer Details

### Layer 1: In-Memory Cache (Fast Access)

**Purpose:** Real-time state for API responses and strategy execution

**Storage Technology:** Rust `Arc<RwLock<SystemSnapshot>>`

**Data Stored:**

- Current positions (`PositionSnapshot[]`)
- Active orders (`OrderSnapshot[]`)
- Recent market data snapshots (`SymbolSnapshot[]`)
- Recent strategy decisions (`StrategyDecisionSnapshot[]`)
- Current alerts (`Alert[]`)
- Risk status (`RiskStatus`)
- Account metrics (`Metrics`)
- **NEW:** Current cash flow timeline (`CashFlowTimeline`)
- **NEW:** Portfolio Greeks (`PortfolioGreeks`)

**Retention Policy:**

- Positions: All current positions
- Orders: Last 100 orders
- Decisions: Last 50 decisions
- Alerts: Last 32 alerts
- Market data: Real-time only (historical in QuestDB)

**Refresh Strategy:**

- Updated in real-time as events occur
- Persisted to Layer 2 (PostgreSQL) periodically (every 5 seconds) and on state changes
- Persisted to Layer 3 (QuestDB) for market data in real-time

**Implementation:**

```rust
// agents/backend/crates/api/src/state.rs (existing)
pub type SharedSnapshot = Arc<RwLock<SystemSnapshot>>;

// NEW: Add persistence hooks
impl SystemSnapshot {
    pub async fn persist(&self, db: &Database) -> Result<()> {
        // Persist to PostgreSQL
        db.save_snapshot(self).await?;
        Ok(())
    }
}
```

### Layer 2: Operational Database (PostgreSQL/SQLite)

**Purpose:** Persistent storage for structured data (positions, orders, portfolio, cash flows, loans)

**Storage Technology:** PostgreSQL (recommended) or SQLite (development)

**Why PostgreSQL:**

- Robust ACID guarantees for financial data
- Excellent JSON support for flexible schemas (positions, cash flows)
- Time-series extensions (TimescaleDB) if needed
- Concurrent access for multiple backend instances
- Mature ecosystem and tooling

**Why SQLite (Development):**

- Zero configuration
- File-based (easy backups)
- Good for single-instance deployments
- Can migrate to PostgreSQL later

**Database Schema:**

#### Positions Table

```sql
CREATE TABLE positions (
    id VARCHAR(255) PRIMARY KEY,
    symbol VARCHAR(50) NOT NULL,
    quantity DECIMAL(15, 4) NOT NULL,
    cost_basis DECIMAL(15, 4) NOT NULL,
    mark DECIMAL(15, 4) NOT NULL,
    unrealized_pnl DECIMAL(15, 4) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    broker VARCHAR(50) NOT NULL,  -- "IBKR" or Israeli broker name
    account_id VARCHAR(100),
    instrument_type VARCHAR(50),  -- "stock", "option", "bond", "etf", etc.

    -- Option-specific fields
    strike DECIMAL(15, 4),
    expiration_date TIMESTAMP,
    option_type VARCHAR(10),  -- "call" or "put"

    -- TASE-specific fields
    exchange VARCHAR(50),
    underlying VARCHAR(50),

    -- Greeks (snapshot)
    delta DECIMAL(15, 8),
    gamma DECIMAL(15, 8),
    vega DECIMAL(15, 8),
    theta DECIMAL(15, 8),
    rho DECIMAL(15, 8),

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated TIMESTAMP,

    -- Indexes
    INDEX idx_symbol (symbol),
    INDEX idx_broker (broker),
    INDEX idx_account_id (account_id),
    INDEX idx_expiration_date (expiration_date),
    INDEX idx_updated_at (updated_at)
);
```

#### Positions History Table

```sql
CREATE TABLE positions_history (
    id SERIAL PRIMARY KEY,
    position_id VARCHAR(255) NOT NULL,
    symbol VARCHAR(50) NOT NULL,
    quantity DECIMAL(15, 4) NOT NULL,
    realized_pnl DECIMAL(15, 4) NOT NULL,
    closed_at TIMESTAMP NOT NULL,
    currency VARCHAR(10) NOT NULL,
    broker VARCHAR(50) NOT NULL,
    account_id VARCHAR(100),

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    INDEX idx_position_id (position_id),
    INDEX idx_symbol (symbol),
    INDEX idx_closed_at (closed_at),
    FOREIGN KEY (position_id) REFERENCES positions(id) ON DELETE SET NULL
);
```

#### Orders Table

```sql
CREATE TABLE orders (
    id VARCHAR(255) PRIMARY KEY,
    symbol VARCHAR(50) NOT NULL,
    side VARCHAR(10) NOT NULL,  -- "BUY" or "SELL"
    quantity INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL,  -- "PENDING", "SUBMITTED", "FILLED", "CANCELLED", "REJECTED"
    submitted_at TIMESTAMP NOT NULL,
    filled_at TIMESTAMP,
    fill_price DECIMAL(15, 4),
    fill_quantity INTEGER,
    commission DECIMAL(10, 4),

    -- Order details
    order_type VARCHAR(20),  -- "MARKET", "LIMIT", etc.
    limit_price DECIMAL(15, 4),
    time_in_force VARCHAR(10),

    -- Relationship
    position_id VARCHAR(255),  -- Link to position if created

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    INDEX idx_symbol (symbol),
    INDEX idx_status (status),
    INDEX idx_submitted_at (submitted_at),
    INDEX idx_position_id (position_id),
    FOREIGN KEY (position_id) REFERENCES positions(id) ON DELETE SET NULL
);
```

#### Portfolio Snapshots Table

```sql
CREATE TABLE portfolio_snapshots (
    id SERIAL PRIMARY KEY,
    snapshot_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    account_id VARCHAR(100) NOT NULL,

    -- Portfolio metrics
    net_liq DECIMAL(15, 4) NOT NULL,
    buying_power DECIMAL(15, 4) NOT NULL,
    excess_liquidity DECIMAL(15, 4) NOT NULL,
    margin_requirement DECIMAL(15, 4) NOT NULL,
    total_cash DECIMAL(15, 4) NOT NULL,

    -- Allocation percentages
    equity_percent DECIMAL(5, 2),
    bond_percent DECIMAL(5, 2),
    cash_percent DECIMAL(5, 2),
    box_spread_percent DECIMAL(5, 2),
    tbill_percent DECIMAL(5, 2),
    spare_cash_percent DECIMAL(5, 2),

    -- Portfolio Greeks
    portfolio_delta DECIMAL(15, 8),
    portfolio_gamma DECIMAL(15, 8),
    portfolio_vega DECIMAL(15, 8),
    portfolio_theta DECIMAL(15, 8),
    portfolio_rho DECIMAL(15, 8),

    -- Cash flow summary
    upcoming_cash_flows JSONB,  -- Next 30 days cash flows
    net_cash_flow_30d DECIMAL(15, 4),

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    INDEX idx_snapshot_at (snapshot_at),
    INDEX idx_account_id (account_id)
);
```

#### Cash Flow Events Table

```sql
CREATE TABLE cash_flow_events (
    id SERIAL PRIMARY KEY,
    event_date TIMESTAMP NOT NULL,
    amount DECIMAL(15, 4) NOT NULL,  -- Positive for inflows, negative for outflows
    currency VARCHAR(10) NOT NULL,
    cash_flow_type VARCHAR(50) NOT NULL,  -- "loan_payment", "option_expiration", "bond_coupon", "bond_maturity", "dividend"
    description TEXT,

    -- Relationships
    position_id VARCHAR(255),  -- Link to position if applicable
    loan_id VARCHAR(255),      -- Link to loan if applicable
    order_id VARCHAR(255),     -- Link to order if applicable

    -- Type-specific fields
    underlying_price DECIMAL(15, 4),  -- For options
    strike DECIMAL(15, 4),           -- For options
    coupon_rate DECIMAL(8, 4),       -- For bonds
    dividend_per_share DECIMAL(10, 4), -- For dividends

    -- Status
    status VARCHAR(20) DEFAULT 'projected',  -- "projected", "confirmed", "paid", "cancelled"
    actual_amount DECIMAL(15, 4),  -- Actual amount when paid
    paid_at TIMESTAMP,

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    INDEX idx_event_date (event_date),
    INDEX idx_cash_flow_type (cash_flow_type),
    INDEX idx_position_id (position_id),
    INDEX idx_loan_id (loan_id),
    INDEX idx_status (status),
    FOREIGN KEY (position_id) REFERENCES positions(id) ON DELETE SET NULL
);
```

#### Loans Table

```sql
CREATE TABLE loans (
    id VARCHAR(255) PRIMARY KEY,
    loan_type VARCHAR(50) NOT NULL,  -- "SHIR" or "CPI_LINKED"
    currency VARCHAR(10) NOT NULL,  -- "ILS"
    principal_remaining DECIMAL(15, 4) NOT NULL,
    monthly_payment DECIMAL(15, 4) NOT NULL,

    -- SHIR-based loan fields
    spread DECIMAL(8, 4),  -- Spread over SHIR
    shir_rate DECIMAL(8, 4),  -- Current SHIR rate

    -- CPI-linked loan fields
    fixed_rate DECIMAL(8, 4),  -- Fixed interest rate
    cpi_index DECIMAL(10, 4),  -- Current CPI index

    remaining_months INTEGER NOT NULL,
    next_payment_date TIMESTAMP NOT NULL,

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    INDEX idx_loan_type (loan_type),
    INDEX idx_next_payment_date (next_payment_date)
);
```

#### Greek Snapshots Table

```sql
CREATE TABLE greek_snapshots (
    id SERIAL PRIMARY KEY,
    snapshot_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    position_id VARCHAR(255),  -- NULL for portfolio-level
    symbol VARCHAR(50),

    -- Greeks
    delta DECIMAL(15, 8) NOT NULL,
    gamma DECIMAL(15, 8) NOT NULL,
    vega DECIMAL(15, 8) NOT NULL,
    theta DECIMAL(15, 8) NOT NULL,
    rho DECIMAL(15, 8) NOT NULL,

    -- Market conditions
    underlying_price DECIMAL(15, 4),
    implied_volatility DECIMAL(8, 4),
    time_to_expiry DECIMAL(10, 4),  -- Days

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    INDEX idx_snapshot_at (snapshot_at),
    INDEX idx_position_id (position_id),
    INDEX idx_symbol (symbol),
    FOREIGN KEY (position_id) REFERENCES positions(id) ON DELETE SET NULL
);
```

### Layer 3: Time-Series Database (QuestDB)

**Purpose:** High-frequency time-series data (market data, quotes, trades)

**Storage Technology:** QuestDB

**Data Stored:**

- Real-time quotes (bid/ask, spread, volume)
- Trade executions
- Market data snapshots
- Historical price data

**Tables:**

#### Quotes Table (Existing)

```sql
CREATE TABLE quotes (
    symbol SYMBOL,
    bid DOUBLE,
    ask DOUBLE,
    spread DOUBLE,
    volume LONG,
    timestamp TIMESTAMP
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

#### Trades Table (Existing)

```sql
CREATE TABLE trades (
    symbol SYMBOL,
    price DOUBLE,
    size LONG,
    timestamp TIMESTAMP
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

**NEW Tables for Portfolio Data:**

#### Portfolio Metrics Table

```sql
CREATE TABLE portfolio_metrics (
    account_id SYMBOL,
    net_liq DOUBLE,
    buying_power DOUBLE,
    excess_liquidity DOUBLE,
    margin_requirement DOUBLE,
    total_cash DOUBLE,
    timestamp TIMESTAMP
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

#### Position Values Table

```sql
CREATE TABLE position_values (
    position_id SYMBOL,
    symbol SYMBOL,
    market_value DOUBLE,
    unrealized_pnl DOUBLE,
    delta DOUBLE,
    gamma DOUBLE,
    vega DOUBLE,
    theta DOUBLE,
    rho DOUBLE,
    timestamp TIMESTAMP
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

### Layer 4: Configuration Storage

**Purpose:** Investment strategy configuration, portfolio allocation settings, risk limits

**Storage Technology:** TOML files + Environment variables

**Location:** `agents/backend/config/`

**Configuration Files:**

1. **`default.toml`** (Existing) - Basic backend configuration
2. **`investment_strategy.toml`** (NEW) - Investment strategy framework config
3. **`portfolio_allocation.toml`** (NEW) - Portfolio allocation percentages
4. **`risk_limits.toml`** (NEW) - Risk limits and constraints
5. **`loans.toml`** (NEW) - Loan configurations (SHIR-based, CPI-linked)
6. **`israeli_brokers.toml`** (NEW) - Israeli broker import configurations

**Example Investment Strategy Config:**

```toml
[investment_strategy]
mode = "DRY-RUN"  # or "LIVE"

[allocation]
equity_target = 40.0
bond_target = 30.0
cash_target = 15.0
box_spread_target = 10.0
spare_cash_target = 5.0

[bond_allocation]
short_term_percent = 50.0
long_term_percent = 50.0

[cash_management]
immediate_cash_min = 3.0
immediate_cash_max = 5.0
spare_cash_min = 7.0
spare_cash_max = 10.0
loan_payment_reserve_months = 2

[rebalancing]
threshold_percent = 5.0
frequency_days = 30

[loans]
shir_rate_source = "BANK_OF_ISRAEL"  # or "IBKR"
cpi_rate_source = "BANK_OF_ISRAEL"

[[loans.list]]
id = "loan-001"
loan_type = "SHIR"
principal_remaining = 1000000.0
spread = 0.0025
remaining_months = 120
next_payment_date = "2025-12-01"

[[loans.list]]
id = "loan-002"
loan_type = "CPI_LINKED"
principal_remaining = 500000.0
fixed_rate = 0.025
remaining_months = 60
next_payment_date = "2025-12-01"
```

## Data Flow

### Write Flow

1. **Real-time Events** (Market data, orders, positions):

   ```
   Event → In-Memory Cache (SystemSnapshot) → PostgreSQL (positions/orders) → QuestDB (time-series)
   ```

2. **Periodic Snapshots**:

   ```
   SystemSnapshot → PostgreSQL (portfolio_snapshots) [Every 5 seconds or on state change]
   ```

3. **Cash Flow Forecasts**:

   ```
   CashFlowCalculator → PostgreSQL (cash_flow_events) [Recalculated daily or on position change]
   ```

4. **Greeks Calculations**:

   ```
   RiskCalculator → In-Memory Cache → PostgreSQL (greek_snapshots) → QuestDB (position_values) [Real-time]
   ```

### Read Flow

1. **Real-time API Requests**:

   ```
   GET /api/v1/snapshot → In-Memory Cache (SystemSnapshot) [Fast, <1ms]
   ```

2. **Historical Queries**:

   ```
   GET /api/v1/positions/history → PostgreSQL [Structured queries]
   GET /api/v1/market-data/history → QuestDB [Time-series queries]
   ```

3. **Cash Flow Timeline**:

   ```
   GET /api/v1/cash-flow/timeline → PostgreSQL (cash_flow_events) [Sorted by date]
   ```

## Database Selection: PostgreSQL vs SQLite

### Recommended: PostgreSQL

**Advantages:**

- **Production-ready:** ACID guarantees for financial data
- **Concurrent access:** Multiple backend instances can share database
- **JSON support:** Flexible schema for positions, cash flows
- **Time-series extensions:** TimescaleDB for advanced time-series queries
- **Mature tooling:** pgAdmin, backup tools, monitoring
- **Scalability:** Can handle large datasets and high write rates

**Disadvantages:**

- Requires separate database server
- More complex setup
- More resource-intensive

### Alternative: SQLite (Development)

**Advantages:**

- **Zero configuration:** File-based database
- **Simple setup:** No separate server needed
- **Easy backups:** Copy the database file
- **Good for:** Single-instance deployments, development, testing

**Disadvantages:**

- **Single writer:** Limited concurrent writes
- **Not ideal for:** High-frequency writes, multiple backend instances
- **Limited JSON support:** Requires more manual schema management

**Recommendation:** Use SQLite for development/testing, PostgreSQL for production.

## Implementation Plan

**Strategy:** Start with SQLite for development, design portable code that scales to PostgreSQL/cloud.

### Phase 1: Database Abstraction Layer (Week 1)

- [ ] Add SQLx dependency to `Cargo.toml` (with SQLite + PostgreSQL features)
- [ ] Create `crates/database` crate with `DBPool` abstraction (SQLite + PostgreSQL)
- [ ] Implement database connection pool that auto-detects backend from connection string
- [ ] Create database configuration in `config/default.toml` (default to SQLite)
- [ ] Set up SQLx migration system (works with both SQLite and PostgreSQL)
- [ ] Create database schema migrations (SQLite + PostgreSQL compatible)
- [ ] **See:** `docs/DATABASE_ABSTRACTION_LAYER.md` for detailed implementation

### Phase 2: Repository Pattern (Week 1-2)

- [ ] Implement repository pattern using database abstraction layer (works with both SQLite and PostgreSQL):
  - `PositionRepository` (database-agnostic queries)
  - `OrderRepository` (database-agnostic queries)
  - `PortfolioSnapshotRepository` (database-agnostic queries)
  - `CashFlowRepository` (database-agnostic queries)
  - `LoanRepository` (database-agnostic queries)
  - `GreekSnapshotRepository` (database-agnostic queries)
- [ ] Test repositories with SQLite (local development)
- [ ] Test repositories with PostgreSQL (verify portability)
- [ ] Implement periodic snapshot persistence (every 5 seconds)
- [ ] Implement on-state-change persistence hooks

### Phase 3: Cash Flow Storage (Week 2-3)

- [ ] Implement `CashFlowRepository` for PostgreSQL
- [ ] Store cash flow forecasts in `cash_flow_events` table
- [ ] Implement daily recalculation and update of cash flow events
- [ ] Add API endpoints to query cash flow timeline from database

### Phase 4: Historical Data Access (Week 3)

- [ ] Implement historical position queries from PostgreSQL
- [ ] Implement historical order queries from PostgreSQL
- [ ] Implement portfolio snapshot queries (time-series from PostgreSQL)
- [ ] Implement market data historical queries from QuestDB
- [ ] Add API endpoints for historical data access

### Phase 5: Configuration Storage (Week 4)

- [ ] Create investment strategy configuration files (TOML)
- [ ] Implement configuration loader and validator
- [ ] Add configuration API endpoints (GET/PUT `/api/v1/config/investment-strategy`)
- [ ] Implement configuration hot-reloading

## Cloud-Based Storage Options

Since you have accounts with **AWS**, **Vultr**, and **Google Cloud**, here are cloud-based alternatives for each layer of the storage architecture:

### Layer 2: Operational Database (Structured Data) - Cloud Options

#### Option 1: AWS (Recommended for Financial Data)

**Amazon RDS for PostgreSQL** (Recommended):

- **Purpose:** Structured data (positions, orders, portfolio snapshots, cash flows, loans, Greeks)
- **Advantages:**
  - Fully managed PostgreSQL with automatic backups, patching, and scaling
  - Multi-AZ deployments for high availability (99.95% SLA)
  - Encryption at rest and in transit (AWS KMS)
  - Point-in-time recovery (up to 35 days)
  - Automated backups with 7-day retention (extendable)
  - Read replicas for scaling read operations
  - VPC integration for security
  - Performance Insights for monitoring
- **Cost:** ~$15-50/month (db.t3.micro to db.t3.small, varies by region)
- **Use Case:** Production deployments requiring high availability and ACID guarantees

**Amazon Aurora PostgreSQL** (High-Performance Alternative):

- **Purpose:** Same as RDS, but with better performance and scaling
- **Advantages:**
  - Up to 3x better performance than standard PostgreSQL
  - Automatic scaling (up to 128TB per instance)
  - Fast failover (<30 seconds)
  - Global database replication
  - Serverless option for variable workloads
- **Cost:** ~$30-100/month (varies by region and instance size)
- **Use Case:** High-performance production deployments with global access

**Amazon DynamoDB** (Alternative for NoSQL Needs):

- **Purpose:** Document-based storage for flexible schemas
- **Advantages:**
  - Single-digit millisecond latency
  - Automatic scaling
  - Built-in backup and restore
  - Encryption at rest
- **Disadvantages:**
  - NoSQL (less structured than PostgreSQL)
  - Different query patterns required
  - Less suitable for complex relational data
- **Cost:** Pay-per-request pricing (~$1.25/million writes, $0.25/million reads)
- **Use Case:** High-throughput, low-latency needs with flexible schemas

**Setup:**

```toml
# agents/backend/config/database.toml
[database.aws_rds]
provider = "postgres"
host = "your-db-instance.region.rds.amazonaws.com"
port = 5432
database = "ib_box_spread"
username_env = "AWS_RDS_USERNAME"
password_env = "AWS_RDS_PASSWORD"
pool_size = 10
ssl_mode = "require"
```

#### Option 2: Google Cloud

**Cloud SQL for PostgreSQL** (Recommended):

- **Purpose:** Structured data (same as AWS RDS)
- **Advantages:**
  - Fully managed PostgreSQL
  - Automatic backups (7-day default, extendable to 365 days)
  - High availability with automatic failover
  - Encryption at rest and in transit
  - Point-in-time recovery
  - Read replicas
  - Integration with BigQuery for analytics
  - Google Cloud IAM integration
- **Cost:** ~$15-50/month (db-f1-micro to db-n1-standard-1, varies by region)
- **Use Case:** Production deployments with Google Cloud integration needs

**Cloud Spanner** (Global Scale Alternative):

- **Purpose:** Globally distributed, strongly consistent database
- **Advantages:**
  - TrueSQL with ACID transactions
  - Global distribution with low latency
  - Automatic sharding and replication
  - 99.999% availability SLA
- **Disadvantages:**
  - More expensive than Cloud SQL
  - Overkill for single-region deployments
- **Cost:** ~$100+/month (minimum 3 nodes required)
- **Use Case:** Global deployments requiring strong consistency

**BigQuery** (Analytics Alternative):

- **Purpose:** Analytics and historical queries on large datasets
- **Advantages:**
  - Serverless, auto-scaling data warehouse
  - Excellent for time-series analytics
  - SQL interface
  - Built-in ML capabilities
- **Disadvantages:**
  - Not ideal for transactional workloads
  - Higher latency for real-time queries
- **Cost:** Pay-per-query pricing (~$5/TB scanned, first 1 TB/month free)
- **Use Case:** Analytics, reporting, historical data analysis (complement to Cloud SQL)

**Setup:**

```toml
# agents/backend/config/database.toml
[database.gcp_cloud_sql]
provider = "postgres"
host = "your-instance-ip:5432"  # Private IP or public IP
database = "ib_box_spread"
username_env = "GCP_CLOUD_SQL_USERNAME"
password_env = "GCP_CLOUD_SQL_PASSWORD"
pool_size = 10
ssl_mode = "require"
# Option: Use Cloud SQL Proxy for secure connections
use_cloud_sql_proxy = true
```

#### Option 3: Vultr

**Vultr Managed PostgreSQL**:

- **Purpose:** Structured data (same as AWS/GCP options)
- **Advantages:**
  - Lower cost than AWS/GCP
  - Fully managed with automatic backups
  - Simple setup and management
  - Multiple regions available
- **Disadvantages:**
  - Less feature-rich than AWS RDS/Cloud SQL
  - Smaller ecosystem and tooling
  - Limited high-availability options
- **Cost:** ~$15-40/month (varies by instance size)
- **Use Case:** Cost-effective production deployments, single-region deployments

**Vultr VPS with Self-Hosted PostgreSQL** (DIY Option):

- **Purpose:** Full control over database configuration
- **Advantages:**
  - Lowest cost
  - Full control over configuration
  - Can host multiple services (PostgreSQL + QuestDB)
- **Disadvantages:**
  - Manual management (backups, updates, monitoring)
  - Requires database administration expertise
  - No automatic failover
- **Cost:** ~$6-24/month (VPS instances)
- **Use Case:** Development, testing, or cost-constrained production deployments

**Setup:**

```toml
# agents/backend/config/database.toml
[database.vultr_managed]
provider = "postgres"
host = "your-db.vultr.com"
port = 5432
database = "ib_box_spread"
username_env = "VULTR_DB_USERNAME"
password_env = "VULTR_DB_PASSWORD"
pool_size = 10
ssl_mode = "require"
```

### Layer 3: Time-Series Database (Market Data) - Cloud Options

#### Option 1: AWS

**Amazon Timestream** (Managed Time-Series Database):

- **Purpose:** Market data (quotes, trades, portfolio metrics)
- **Advantages:**
  - Fully managed time-series database
  - Purpose-built for time-series data
  - Automatic scaling
  - Built-in analytics functions
  - Automatic data lifecycle management (move old data to cheaper storage)
- **Disadvantages:**
  - Different query language (SQL-like but not standard SQL)
  - Migration from QuestDB required
  - Newer service (less mature than QuestDB)
- **Cost:** ~$0.50/GB-month for hot storage, $0.03/GB-month for cold storage
- **Use Case:** Production deployments requiring managed time-series database

**Self-Hosted QuestDB on AWS EC2** (Keep Existing):

- **Purpose:** Market data (same as current QuestDB setup)
- **Advantages:**
  - Keep existing QuestDB setup
  - Full control over configuration
  - Standard SQL queries
  - Lower cost than managed services
- **Disadvantages:**
  - Manual management required
  - Need to handle backups and scaling
- **Cost:** ~$10-50/month (EC2 instance, varies by size)
- **Setup:** Deploy QuestDB on EC2 instance or ECS container

**AWS S3 + Athena** (Archival Alternative):

- **Purpose:** Long-term storage and analytics of historical market data
- **Advantages:**
  - Very cheap storage (~$0.023/GB-month)
  - Query with SQL via Athena
  - Automatic lifecycle management
- **Disadvantages:**
  - Not for real-time queries (cold storage)
  - Higher query costs for frequent access
- **Cost:** ~$5/TB scanned (first 1 TB/month free)
- **Use Case:** Archival storage and infrequent historical queries

#### Option 2: Google Cloud

**BigQuery** (Recommended for Analytics):

- **Purpose:** Market data analytics and historical queries
- **Advantages:**
  - Excellent for time-series analytics
  - SQL interface
  - Automatic scaling
  - Built-in ML capabilities
  - Partitioning by timestamp for efficient queries
- **Disadvantages:**
  - Higher latency than QuestDB (not for real-time)
  - Better for analytics than real-time ingestion
- **Cost:** ~$5/TB scanned (first 1 TB/month free), $20/TB/month storage
- **Use Case:** Historical market data analysis, complement to real-time database

**Self-Hosted QuestDB on Google Compute Engine**:

- **Purpose:** Market data (same as current QuestDB setup)
- **Advantages:**
  - Keep existing QuestDB setup
  - Full control
  - Integration with Google Cloud networking
- **Cost:** ~$10-50/month (Compute Engine instance)
- **Setup:** Deploy QuestDB on Compute Engine VM or GKE (Kubernetes)

**Cloud Bigtable** (High-Throughput Alternative):

- **Purpose:** High-frequency time-series data ingestion
- **Advantages:**
  - Single-digit millisecond latency
  - Handles millions of writes/second
  - Automatic scaling
- **Disadvantages:**
  - NoSQL (different query patterns)
  - More complex setup than QuestDB
- **Cost:** ~$65/month (minimum cluster with 3 nodes)
- **Use Case:** Very high-frequency market data ingestion

#### Option 3: Vultr

**Self-Hosted QuestDB on Vultr VPS** (Recommended):

- **Purpose:** Market data (same as current QuestDB setup)
- **Advantages:**
  - Lowest cost
  - Keep existing QuestDB setup
  - Full control
  - Can co-locate with backend if needed
- **Disadvantages:**
  - Manual management required
- **Cost:** ~$6-24/month (VPS instance)
- **Setup:** Deploy QuestDB on Vultr VPS instance

**Vultr Object Storage** (Archival Alternative):

- **Purpose:** Long-term storage of historical market data
- **Advantages:**
  - Very cheap storage (~$0.01/GB-month)
  - S3-compatible API
- **Disadvantages:**
  - Not for real-time queries
- **Cost:** ~$5/month (500GB included, then $0.01/GB)
- **Use Case:** Archival storage and backups

### Layer 4: Backup Storage - Cloud Options

#### AWS S3 (Recommended for AWS Deployments)

- **Purpose:** Database backups, configuration backups
- **Advantages:**
  - 99.999999999% (11 9's) durability
  - Automatic lifecycle policies (move to cheaper storage)
  - Versioning and cross-region replication
  - Encryption at rest
- **Cost:** ~$0.023/GB-month (Standard), ~$0.004/GB-month (Glacier for long-term)
- **Setup:** Automated backups from RDS can be stored in S3

#### Google Cloud Storage (Recommended for GCP Deployments)

- **Purpose:** Database backups, configuration backups
- **Advantages:**
  - 99.999999999% (11 9's) durability
  - Automatic lifecycle management
  - Integration with Cloud SQL automatic backups
  - Encryption at rest
- **Cost:** ~$0.020/GB-month (Standard), ~$0.004/GB-month (Archive for long-term)
- **Setup:** Cloud SQL backups automatically stored in Cloud Storage

#### Vultr Object Storage (Recommended for Vultr Deployments)

- **Purpose:** Database backups, configuration backups
- **Advantages:**
  - Low cost
  - S3-compatible API
  - Simple setup
- **Cost:** ~$0.01/GB-month (500GB included)
- **Setup:** Use `pg_dump` or similar to backup to Vultr Object Storage

## Cloud Provider Recommendations

### Recommended Architecture by Provider

#### AWS Architecture (Recommended for Financial Data)

```
┌─────────────────────────────────────────┐
│  AWS RDS PostgreSQL                     │
│  - Positions, Orders, Portfolio         │
│  - Cash Flows, Loans, Greeks            │
│  - Multi-AZ for HA                      │
│  - Automated backups → S3               │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│  QuestDB on EC2                         │
│  - Market data (quotes, trades)         │
│  - Portfolio metrics time-series        │
│  - Backup to S3                         │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│  AWS S3                                 │
│  - Database backups                     │
│  - Configuration backups                │
│  - Lifecycle policies (Glacier)         │
└─────────────────────────────────────────┘
```

**Cost Estimate:** ~$50-150/month (RDS + EC2 + S3)

#### Google Cloud Architecture

```
┌─────────────────────────────────────────┐
│  Cloud SQL PostgreSQL                   │
│  - Positions, Orders, Portfolio         │
│  - Cash Flows, Loans, Greeks            │
│  - High Availability                    │
│  - Automated backups → Cloud Storage    │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│  QuestDB on Compute Engine              │
│  - Market data (quotes, trades)         │
│  - Portfolio metrics time-series        │
│  - Backup to Cloud Storage              │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│  Cloud Storage                          │
│  - Database backups                     │
│  - Configuration backups                │
│  - Lifecycle policies (Archive)         │
└─────────────────────────────────────────┘
```

**Cost Estimate:** ~$50-150/month (Cloud SQL + Compute Engine + Cloud Storage)

#### Vultr Architecture (Most Cost-Effective)

```
┌─────────────────────────────────────────┐
│  Vultr Managed PostgreSQL               │
│  - Positions, Orders, Portfolio         │
│  - Cash Flows, Loans, Greeks            │
│  - Automated backups                    │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│  QuestDB on Vultr VPS                   │
│  - Market data (quotes, trades)         │
│  - Portfolio metrics time-series        │
│  - Backup to Object Storage             │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│  Vultr Object Storage                   │
│  - Database backups                     │
│  - Configuration backups                │
└─────────────────────────────────────────┘
```

**Cost Estimate:** ~$30-80/month (Managed DB + VPS + Object Storage)

### Recommendation Summary

**For Production (Financial Data):**

1. **AWS RDS PostgreSQL** + QuestDB on EC2 + S3 (Best for: High availability, regulatory compliance, enterprise features)
2. **Google Cloud SQL** + QuestDB on Compute Engine + Cloud Storage (Best for: Integration with GCP services, BigQuery analytics)
3. **Vultr Managed PostgreSQL** + QuestDB on VPS + Object Storage (Best for: Cost-effectiveness, simplicity)

**For Development:**

- Use local SQLite + local QuestDB (zero cloud costs)

**Hybrid Approach:**

- Development: Local SQLite + QuestDB
- Staging: Vultr (lower cost)
- Production: AWS or Google Cloud (high availability)

## Storage Location Recommendations

### Development (SQLite - Recommended Starting Point)

```
project_root/
├── data/
│   ├── sqlite/              # SQLite database file (development)
│   │   └── backend.db       # Created automatically on first run
│   └── questdb/             # QuestDB data directory
├── agents/backend/
│   ├── config/
│   │   ├── default.toml     # database.url = "sqlite://data/sqlite/backend.db"
│   │   ├── investment_strategy.toml
│   │   ├── portfolio_allocation.toml
│   │   └── loans.toml
│   └── crates/
│       └── database/
│           └── migrations/  # SQL migration files (SQLite + PostgreSQL compatible)
└── .env                     # Optional: DATABASE_URL="sqlite://data/sqlite/backend.db"
```

**To Start:**

1. Set `DATABASE_URL="sqlite://data/sqlite/backend.db"` (or use default in config)
2. Run backend - SQLite database created automatically
3. Migrations run automatically on startup
4. No database server setup needed!

**To Scale to PostgreSQL:**

1. Install PostgreSQL locally, or use cloud database
2. Set `DATABASE_URL="postgresql://user:password@localhost:5432/ib_box_spread"`
3. Run backend - Migrations run on PostgreSQL automatically
4. No code changes needed!

### Production (Cloud-Based)

**AWS:**

- **PostgreSQL:** AWS RDS PostgreSQL (Multi-AZ for HA)
- **QuestDB:** EC2 instance or ECS container
- **Backups:** AWS S3 (with lifecycle policies to Glacier)
- **Configuration:** Version controlled in Git, deployed with application

**Google Cloud:**

- **PostgreSQL:** Cloud SQL PostgreSQL (High Availability)
- **QuestDB:** Compute Engine VM or GKE (Kubernetes)
- **Backups:** Cloud Storage (with lifecycle policies to Archive)
- **Configuration:** Version controlled in Git, deployed with application

**Vultr:**

- **PostgreSQL:** Vultr Managed PostgreSQL
- **QuestDB:** Vultr VPS instance
- **Backups:** Vultr Object Storage
- **Configuration:** Version controlled in Git, deployed with application

## Backup Strategy

### PostgreSQL Backups

- **Daily full backups:** pg_dump to compressed files
- **Continuous archiving:** WAL archiving for point-in-time recovery
- **Retention:** 30 days of daily backups, 7 days of WAL archives

### QuestDB Backups

- **Daily snapshots:** Copy QuestDB data directory
- **Retention:** 30 days

### Configuration Backups

- **Version control:** All configuration files in Git
- **Deployment:** Configuration files deployed with application code

## Security Considerations

1. **Database Credentials:**
   - Store in environment variables
   - Never commit to Git
   - Use secrets management (e.g., Kubernetes secrets)

2. **Database Access:**
   - Restrict network access (bind to localhost or private network)
   - Use strong passwords
   - Enable SSL/TLS for PostgreSQL connections

3. **Data Encryption:**
   - Encrypt sensitive data at rest (database encryption)
   - Encrypt data in transit (SSL/TLS connections)

4. **Backup Encryption:**
   - Encrypt backup files
   - Store backups securely

## Monitoring and Maintenance

### Database Health Checks

- Monitor database connection pool usage
- Monitor query performance (slow queries)
- Monitor database size and growth
- Monitor QuestDB ingestion rate

### Maintenance Tasks

- **Weekly:** Vacuum PostgreSQL database
- **Monthly:** Analyze PostgreSQL statistics
- **Quarterly:** Review and archive old data
- **As needed:** Database index optimization

## References

- **Database Abstraction Layer:** `docs/DATABASE_ABSTRACTION_LAYER.md` - Complete implementation guide for portable database layer
- [SQLx Documentation](https://docs.rs/sqlx/) - Database abstraction library
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [QuestDB Documentation](https://questdb.io/docs/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [TimescaleDB](https://www.timescale.com/) - PostgreSQL time-series extension

---

**Next Steps:**

1. Review and approve storage architecture
2. Set up PostgreSQL database
3. Create database schema
4. Implement persistence layer
5. Integrate with existing backend
