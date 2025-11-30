# Database Abstraction Layer - Portable Backend Design

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document designs a portable database abstraction layer that allows starting with SQLite for local development and seamlessly scaling to PostgreSQL (or other cloud databases) for production, without code changes.

## Design Principles

1. **Database Agnostic:** Same code works with SQLite and PostgreSQL
2. **Start Simple:** SQLite for development (zero setup, file-based)
3. **Scale Seamlessly:** Switch to PostgreSQL/cloud by changing connection string
4. **Type Safe:** Compile-time query checking
5. **Migration Support:** Database schema migrations work for both SQLite and PostgreSQL

## Technology Choice: SQLx

**Why SQLx:**

- ✅ Supports SQLite, PostgreSQL, MySQL, MSSQL
- ✅ Compile-time query checking (catches SQL errors at compile time)
- ✅ Async/await support (works with Tokio)
- ✅ No ORM overhead (just a thin abstraction layer)
- ✅ Same API for all databases
- ✅ Built-in connection pooling
- ✅ Migration support

**Alternatives Considered:**

- **Diesel:** ORM, more complex, harder to be database-agnostic
- **Rusqlite:** SQLite-only, would require rewriting for PostgreSQL
- **Quaint:** Lower-level, less ergonomic

## Architecture

### Database Abstraction Pattern

```
┌──────────────────────────────────────────────┐
│  Application Code (Repositories)              │
│  - PositionRepository                        │
│  - OrderRepository                           │
│  - PortfolioSnapshotRepository               │
│  - CashFlowRepository                        │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│  Database Abstraction Layer (DBPool)          │
│  - Handles connection pooling                 │
│  - Provides unified query interface           │
│  - Manages transactions                       │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│  SQLx Driver Layer                            │
│  - SQLite (sqlx::sqlite)                      │
│  - PostgreSQL (sqlx::postgres)                │
│  - Automatic backend selection                │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│  Database Backend                             │
│  - SQLite (local file)                        │
│  - PostgreSQL (local/cloud)                   │
│  - Cloud (AWS RDS, GCP Cloud SQL, etc.)      │
└──────────────────────────────────────────────┘
```

## Implementation

### 1. Database Connection Pool

```rust
// agents/backend/crates/database/src/lib.rs

use sqlx::{Pool, Sqlite, Postgres, Executor};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use anyhow::{Result, Context};

#[derive(Clone, Debug)]

pub enum DatabaseBackend {
    Sqlite,
    Postgres,
}

#[derive(Clone)]

pub enum DBPool {
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<Postgres>),
}

impl DBPool {
    /// Create a database pool from connection string.
    /// Detects backend automatically from connection string.
    pub async fn connect(database_url: &str) -> Result<Self> {
        if database_url.starts_with("sqlite:") || database_url.starts_with("sqlite://") {
            let pool = SqlitePoolOptions::new()
                .max_connections(10)
                .connect(database_url)
                .await
                .context("Failed to connect to SQLite database")?;

            Ok(DBPool::Sqlite(pool))
        } else if database_url.starts_with("postgres:") || database_url.starts_with("postgresql://") {
            let pool = PgPoolOptions::new()
                .max_connections(10)
                .connect(database_url)
                .await
                .context("Failed to connect to PostgreSQL database")?;

            Ok(DBPool::Postgres(pool))
        } else {
            anyhow::bail!("Unsupported database URL format. Use 'sqlite://path' or 'postgresql://...'");
        }
    }

    /// Get backend type.
    pub fn backend(&self) -> DatabaseBackend {
        match self {
            DBPool::Sqlite(_) => DatabaseBackend::Sqlite,
            DBPool::Postgres(_) => DatabaseBackend::Postgres,
        }
    }

    /// Execute a query that returns no rows.
    pub async fn execute(&self, query: &str) -> Result<u64> {
        match self {
            DBPool::Sqlite(pool) => {
                let result = sqlx::query(query).execute(pool).await?;
                Ok(result.rows_affected())
            }
            DBPool::Postgres(pool) => {
                let result = sqlx::query(query).execute(pool).await?;
                Ok(result.rows_affected())
            }
        }
    }

    /// Execute a parameterized query.
    pub async fn execute_with<'q, E>(&self, query: E) -> Result<u64>
    where
        E: sqlx::Executor<'q>,
        for<'e> &'e mut E::Database: sqlx::Database,
    {
        // This is a simplified version - actual implementation would use
        // database-specific query builders
        todo!("Use sqlx::query! macro for compile-time checked queries")
    }
}
```

### 2. Repository Pattern

Each repository uses the `DBPool` abstraction, making it database-agnostic:

```rust
// agents/backend/crates/database/src/repositories/position_repository.rs

use sqlx::{query, query_as};
use chrono::{DateTime, Utc};
use crate::db_pool::DBPool;
use crate::models::{Position, PositionSnapshot};
use anyhow::{Result, Context};

pub struct PositionRepository {
    pool: DBPool,
}

impl PositionRepository {
    pub fn new(pool: DBPool) -> Self {
        Self { pool }
    }

    /// Save a position snapshot.
    /// Works with both SQLite and PostgreSQL.
    pub async fn save_position(&self, position: &PositionSnapshot) -> Result<()> {
        // Use sqlx::query! macro for compile-time query checking
        // SQLx will validate the query at compile time for the selected backend

        match &self.pool {
            DBPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO positions (
                        id, symbol, quantity, cost_basis, mark, unrealized_pnl,
                        currency, broker, account_id, instrument_type,
                        strike, expiration_date, option_type,
                        exchange, underlying,
                        delta, gamma, vega, theta, rho,
                        created_at, updated_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                    position.id,
                    position.symbol,
                    position.quantity as i64,
                    position.cost_basis,
                    position.mark,
                    position.unrealized_pnl,
                    position.currency,
                    position.broker,
                    position.account_id,
                    position.instrument_type,
                    position.strike,
                    position.expiration_date,
                    position.option_type,
                    position.exchange,
                    position.underlying,
                    position.delta,
                    position.gamma,
                    position.vega,
                    position.theta,
                    position.rho,
                    Utc::now(),
                    Utc::now()
                )
                .execute(pool)
                .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO positions (
                        id, symbol, quantity, cost_basis, mark, unrealized_pnl,
                        currency, broker, account_id, instrument_type,
                        strike, expiration_date, option_type,
                        exchange, underlying,
                        delta, gamma, vega, theta, rho,
                        created_at, updated_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
                    "#,
                    position.id,
                    position.symbol,
                    position.quantity as i32,
                    position.cost_basis,
                    position.mark,
                    position.unrealized_pnl,
                    position.currency,
                    position.broker,
                    position.account_id,
                    position.instrument_type,
                    position.strike,
                    position.expiration_date,
                    position.option_type,
                    position.exchange,
                    position.underlying,
                    position.delta,
                    position.gamma,
                    position.vega,
                    position.theta,
                    position.rho,
                    Utc::now(),
                    Utc::now()
                )
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Get all current positions.
    pub async fn get_all_positions(&self) -> Result<Vec<PositionSnapshot>> {
        match &self.pool {
            DBPool::Sqlite(pool) => {
                let positions = sqlx::query_as!(
                    PositionSnapshot,
                    r#"
                    SELECT
                        id, symbol, quantity as "quantity: i32", cost_basis, mark, unrealized_pnl,
                        currency, broker, account_id, instrument_type,
                        strike, expiration_date, option_type,
                        exchange, underlying,
                        delta, gamma, vega, theta, rho,
                        created_at, updated_at
                    FROM positions
                    ORDER BY updated_at DESC
                    "#
                )
                .fetch_all(pool)
                .await?;

                Ok(positions)
            }
            DBPool::Postgres(pool) => {
                let positions = sqlx::query_as!(
                    PositionSnapshot,
                    r#"
                    SELECT
                        id, symbol, quantity, cost_basis, mark, unrealized_pnl,
                        currency, broker, account_id, instrument_type,
                        strike, expiration_date, option_type,
                        exchange, underlying,
                        delta, gamma, vega, theta, rho,
                        created_at, updated_at
                    FROM positions
                    ORDER BY updated_at DESC
                    "#
                )
                .fetch_all(pool)
                .await?;

                Ok(positions)
            }
        }
    }

    /// Update position mark and unrealized P&L.
    pub async fn update_position_mark(&self, id: &str, mark: f64, unrealized_pnl: f64) -> Result<()> {
        match &self.pool {
            DBPool::Sqlite(pool) => {
                sqlx::query!(
                    "UPDATE positions SET mark = ?, unrealized_pnl = ?, updated_at = ? WHERE id = ?",
                    mark,
                    unrealized_pnl,
                    Utc::now(),
                    id
                )
                .execute(pool)
                .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query!(
                    "UPDATE positions SET mark = $1, unrealized_pnl = $2, updated_at = $3 WHERE id = $4",
                    mark,
                    unrealized_pnl,
                    Utc::now(),
                    id
                )
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }
}
```

### 3. Database-Agnostic Query Helper

To avoid code duplication, we can create a helper that abstracts SQL syntax differences:

```rust
// agents/backend/crates/database/src/query_builder.rs

use sqlx::{Pool, Sqlite, Postgres};
use crate::db_pool::DBPool;

/// Build a parameterized query that works with both SQLite and PostgreSQL.
/// SQLite uses ? placeholders, PostgreSQL uses $1, $2, etc.
pub struct QueryBuilder {
    backend: DatabaseBackend,
}

impl QueryBuilder {
    pub fn new(backend: DatabaseBackend) -> Self {
        Self { backend }
    }

    /// Convert PostgreSQL-style query ($1, $2) to SQLite-style (?, ?).
    pub fn adapt_query(&self, query: &str) -> String {
        match self.backend {
            DatabaseBackend::Sqlite => {
                // Replace $1, $2, ... with ?, ?, ...
                let mut adapted = query.to_string();
                let mut param_num = 1;
                while adapted.contains(&format!("${}", param_num)) {
                    adapted = adapted.replace(&format!("${}", param_num), "?");
                    param_num += 1;
                }
                adapted
            }
            DatabaseBackend::Postgres => {
                // Already PostgreSQL format
                query.to_string()
            }
        }
    }
}

// However, better approach: Use sqlx::query! macro with backend-specific queries
// Or use sqlx::query() with runtime parameter binding
```

**Better Approach:** Use `sqlx::query()` (not `query!`) for database-agnostic queries:

```rust
// This works with both SQLite and PostgreSQL
pub async fn save_position_agnostic(&self, position: &PositionSnapshot) -> Result<()> {
    // sqlx::query() uses runtime parameter binding that adapts to the database
    sqlx::query(
        r#"
        INSERT INTO positions (id, symbol, quantity, cost_basis, mark, unrealized_pnl, currency, broker)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(&position.id)
    .bind(&position.symbol)
    .bind(position.quantity)
    .bind(position.cost_basis)
    .bind(position.mark)
    .bind(position.unrealized_pnl)
    .bind(&position.currency)
    .bind(&position.broker)
    .execute(&self.pool.as_ref())  // Works with both Pool<Sqlite> and Pool<Postgres>
    .await?;

    Ok(())
}
```

### 4. Migration System

SQLx provides built-in migration support that works with both SQLite and PostgreSQL:

```rust
// agents/backend/crates/database/src/migrations.rs

use sqlx::migrate::Migrator;
use std::path::Path;
use crate::db_pool::DBPool;
use anyhow::Result;

pub async fn run_migrations(pool: &DBPool) -> Result<()> {
    // SQLx migrations work with both SQLite and PostgreSQL
    // The migration files contain SQL that's compatible with both databases
    // (or use conditional SQL like `-- sqlx:prepare` directives)

    match pool {
        DBPool::Sqlite(pool) => {
            let migrator = Migrator::new(Path::new("migrations")).await?;
            migrator.run(pool).await?;
        }
        DBPool::Postgres(pool) => {
            let migrator = Migrator::new(Path::new("migrations")).await?;
            migrator.run(pool).await?;
        }
    }

    Ok(())
}
```

**Migration Files:** Use SQL that works with both databases, or use SQLx conditional compilation:

```sql
-- migrations/001_create_positions.sql

-- This SQL works with both SQLite and PostgreSQL
CREATE TABLE IF NOT EXISTS positions (
    id VARCHAR(255) PRIMARY KEY,
    symbol VARCHAR(50) NOT NULL,
    quantity INTEGER NOT NULL,  -- SQLite uses INTEGER, PostgreSQL uses INTEGER too
    cost_basis DECIMAL(15, 4) NOT NULL,
    mark DECIMAL(15, 4) NOT NULL,
    unrealized_pnl DECIMAL(15, 4) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    broker VARCHAR(50) NOT NULL,
    account_id VARCHAR(100),
    instrument_type VARCHAR(50),

    -- Option-specific fields
    strike DECIMAL(15, 4),
    expiration_date TIMESTAMP,
    option_type VARCHAR(10),

    -- TASE-specific fields
    exchange VARCHAR(50),
    underlying VARCHAR(50),

    -- Greeks
    delta DECIMAL(15, 8),
    gamma DECIMAL(15, 8),
    vega DECIMAL(15, 8),
    theta DECIMAL(15, 8),
    rho DECIMAL(15, 8),

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- PostgreSQL-specific: Create indexes
-- SQLite will ignore this if not supported
CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions(symbol);
CREATE INDEX IF NOT EXISTS idx_positions_broker ON positions(broker);
CREATE INDEX IF NOT EXISTS idx_positions_expiration_date ON positions(expiration_date);
CREATE INDEX IF NOT EXISTS idx_positions_updated_at ON positions(updated_at);
```

**SQLx Conditional SQL:** For database-specific SQL, use SQLx conditional compilation:

```sql
-- migrations/002_add_jsonb_support.sql

-- This is PostgreSQL-specific (JSONB)
-- sqlx:prepare
-- sqlx:expect-type=void
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";  -- PostgreSQL only

-- Use IF NOT EXISTS for SQLite compatibility
-- For SQLite, we'll create a TEXT column instead of JSONB
-- sqlx:prepare
-- sqlx:expect-type=void
ALTER TABLE positions ADD COLUMN IF NOT EXISTS metadata TEXT;  -- SQLite uses TEXT
-- PostgreSQL would use: ALTER TABLE positions ADD COLUMN IF NOT EXISTS metadata JSONB;
```

**Better Approach:** Use separate migration files or runtime detection:

```rust
// agents/backend/crates/database/src/migrations.rs

pub async fn run_migrations(pool: &DBPool) -> Result<()> {
    match pool {
        DBPool::Sqlite(pool) => {
            // Run SQLite-specific migrations
            let migrator = Migrator::new(Path::new("migrations/sqlite")).await?;
            migrator.run(pool).await?;
        }
        DBPool::Postgres(pool) => {
            // Run PostgreSQL-specific migrations
            let migrator = Migrator::new(Path::new("migrations/postgres")).await?;
            migrator.run(pool).await?;
        }
    }

    Ok(())
}

// Or use shared migrations with SQL that works for both
pub async fn run_migrations_shared(pool: &DBPool) -> Result<()> {
    // Use migrations that work with both SQLite and PostgreSQL
    // (avoid database-specific features, or use IF NOT EXISTS)
    let migrator = Migrator::new(Path::new("migrations/shared")).await?;

    match pool {
        DBPool::Sqlite(pool) => migrator.run(pool).await?,
        DBPool::Postgres(pool) => migrator.run(pool).await?,
    }

    Ok(())
}
```

### 5. Configuration

Add database configuration to backend config:

```toml

# agents/backend/config/default.toml

[database]

# Use SQLite for development (local file)

url = "sqlite://data/sqlite/backend.db"

# Or use PostgreSQL for production
# url = "postgresql://user:password@localhost:5432/ib_box_spread"

# Connection pool settings

max_connections = 10
min_connections = 2
acquire_timeout_seconds = 30
idle_timeout_seconds = 600

# Enable compile-time query checking
# Set to "sqlite" or "postgres" to enable compile-time checking
# Leave unset or "runtime" for runtime checking (works with both)

query_check_mode = "runtime"  # or "sqlite", "postgres"
```

**Environment Variable Override:**

```bash

# Development (SQLite)

export DATABASE_URL="sqlite://data/sqlite/backend.db"

# Production (PostgreSQL local)

export DATABASE_URL="postgresql://user:password@localhost:5432/ib_box_spread"

# Production (AWS RDS PostgreSQL)

export DATABASE_URL="postgresql://user:password@your-db.region.rds.amazonaws.com:5432/ib_box_spread"

# Production (Google Cloud SQL)

export DATABASE_URL="postgresql://user:password@your-instance-ip:5432/ib_box_spread"

# Production (Vultr Managed PostgreSQL)

export DATABASE_URL="postgresql://user:password@your-db.vultr.com:5432/ib_box_spread"
```

### 6. Backend Integration

Update backend service to use database pool:

```rust
// agents/backend/services/backend_service/src/main.rs

use database::{DBPool, PositionRepository, OrderRepository, /* ... */};

#[derive(Debug, Deserialize, Clone)]

struct BackendConfig {
    #[serde(default = "default_rest_addr")]
    rest_addr: SocketAddr,
    #[serde(default = "default_grpc_addr")]
    grpc_addr: SocketAddr,
    #[serde(default)]
    market_data: MarketDataSettings,
    #[serde(default)]
    database: DatabaseSettings,
}

#[derive(Debug, Deserialize, Clone)]

struct DatabaseSettings {
    #[serde(default = "default_database_url")]
    url: String,
    #[serde(default = "default_max_connections")]
    max_connections: u32,
}

fn default_database_url() -> String {
    // Default to SQLite for local development
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/sqlite/backend.db".to_string())
}

fn default_max_connections() -> u32 {
    10
}

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = load_config().context("failed to load backend config")?;

    // Initialize database pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| config.database.url.clone());

    info!(%database_url, "Connecting to database");
    let db_pool = DBPool::connect(&database_url).await
        .context("Failed to connect to database")?;

    info!(backend = ?db_pool.backend(), "Database connected");

    // Initialize repositories
    let position_repo = PositionRepository::new(db_pool.clone());
    let order_repo = OrderRepository::new(db_pool.clone());
    // ... other repositories

    // Run migrations
    database::run_migrations(&db_pool).await
        .context("Failed to run database migrations")?;

    // Continue with existing backend initialization...
    let state: SharedSnapshot = Arc::new(RwLock::new(SystemSnapshot::default()));
    // ...

    Ok(())
}
```

## Schema Compatibility

### SQL Differences to Handle

1. **Placeholders:**
   - SQLite: `?`
   - PostgreSQL: `$1, $2, ...`
   - **Solution:** Use `sqlx::query()` with `.bind()` (auto-adapts)

2. **Data Types:**
   - SQLite: INTEGER (8 bytes), REAL, TEXT, BLOB
   - PostgreSQL: INTEGER (4 bytes), BIGINT, DECIMAL, VARCHAR, TEXT, JSONB
   - **Solution:** Use compatible types (INTEGER, DECIMAL, VARCHAR, TEXT)

3. **Boolean:**
   - SQLite: INTEGER (0/1)
   - PostgreSQL: BOOLEAN
   - **Solution:** Use INTEGER in SQL, Rust `bool` maps correctly

4. **JSON:**
   - SQLite: TEXT (store as JSON string)
   - PostgreSQL: JSONB (native JSON)
   - **Solution:** Use TEXT in SQL, serialize/deserialize in Rust

5. **Auto-increment:**
   - SQLite: AUTOINCREMENT
   - PostgreSQL: SERIAL or GENERATED ALWAYS AS IDENTITY
   - **Solution:** Use SERIAL in migrations, SQLite handles it automatically

### Compatible Schema Example

```sql
-- Works with both SQLite and PostgreSQL
CREATE TABLE IF NOT EXISTS positions (
    id VARCHAR(255) PRIMARY KEY,  -- Use VARCHAR for both
    symbol VARCHAR(50) NOT NULL,
    quantity INTEGER NOT NULL,    -- INTEGER works for both (SQLite: 8 bytes, PostgreSQL: 4 bytes, but compatible)
    cost_basis DECIMAL(15, 4) NOT NULL,  -- DECIMAL works for both
    mark DECIMAL(15, 4) NOT NULL,
    unrealized_pnl DECIMAL(15, 4) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    broker VARCHAR(50) NOT NULL,

    -- JSON fields: Use TEXT for both, serialize in Rust
    metadata TEXT,  -- Store JSON as text (works for both)

    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,  -- Works for both
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## Migration Path

### Phase 1: SQLite (Development)

**Configuration:**

```toml
[database]
url = "sqlite://data/sqlite/backend.db"
```

**Benefits:**

- Zero setup (file-based)
- Fast local development
- Easy backups (copy file)
- No database server needed

**Limitations:**

- Single writer (but fine for development)
- Limited concurrent connections
- File-based (can't share across instances)

### Phase 2: PostgreSQL (Staging/Production)

**Configuration:**

```toml
[database]
url = "postgresql://user:password@localhost:5432/ib_box_spread"
```

**Benefits:**

- Multiple concurrent writers
- Higher performance
- ACID guarantees
- Can scale to cloud later

**Migration Steps:**

1. Set `DATABASE_URL` environment variable to PostgreSQL connection string
2. Run migrations (SQLx handles SQL differences)
3. No code changes needed!

### Phase 3: Cloud Database (Production)

**AWS RDS:**

```toml
[database]
url = "postgresql://user:password@your-db.region.rds.amazonaws.com:5432/ib_box_spread"
```

**Google Cloud SQL:**

```toml
[database]
url = "postgresql://user:password@your-instance-ip:5432/ib_box_spread"
```

**Vultr Managed:**

```toml
[database]
url = "postgresql://user:password@your-db.vultr.com:5432/ib_box_spread"
```

**Migration Steps:**

1. Create cloud database
2. Run migrations on cloud database
3. Update `DATABASE_URL` environment variable
4. No code changes needed!

## Data Migration (SQLite → PostgreSQL)

When moving from SQLite to PostgreSQL:

1. **Export from SQLite:**

```bash

# Using sqlite3 CLI

sqlite3 data/sqlite/backend.db .dump > dump.sql
```

1. **Or use SQLx migrations:**

- SQLx migrations work with both databases
- Run migrations on PostgreSQL to create schema
- Then copy data using SQLx queries

1. **Data Transfer Script:**

```rust
// scripts/migrate_sqlite_to_postgres.rs

use sqlx::{Pool, Sqlite, Postgres};
use database::{DBPool, PositionRepository};

async fn migrate_data(
    sqlite_pool: Pool<Sqlite>,
    postgres_pool: Pool<Postgres>,
) -> Result<()> {
    // Read from SQLite
    let sqlite_repo = PositionRepository::new(DBPool::Sqlite(sqlite_pool));
    let positions = sqlite_repo.get_all_positions().await?;

    // Write to PostgreSQL
    let postgres_repo = PositionRepository::new(DBPool::Postgres(postgres_pool));
    for position in positions {
        postgres_repo.save_position(&position).await?;
    }

    Ok(())
}
```

## Testing Strategy

### Unit Tests

```rust

#[cfg(test)]

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_position_repository_sqlite() {
        // Create in-memory SQLite database for testing
        let pool = DBPool::connect("sqlite::memory:").await.unwrap();
        let repo = PositionRepository::new(pool);

        // Run tests
        // ...
    }

    #[tokio::test]
    #[ignore]  // Skip unless POSTGRES_URL is set
    async fn test_position_repository_postgres() {
        let database_url = std::env::var("POSTGRES_TEST_URL")
            .expect("POSTGRES_TEST_URL not set");
        let pool = DBPool::connect(&database_url).await.unwrap();
        let repo = PositionRepository::new(pool);

        // Run same tests
        // ...
    }
}
```

## Implementation Plan

### Phase 1: Database Abstraction Layer (Week 1)

- [ ] Create `crates/database` crate
- [ ] Implement `DBPool` enum (SQLite + PostgreSQL)
- [ ] Add SQLx dependencies to `Cargo.toml`
- [ ] Create database connection utilities
- [ ] Add configuration support (TOML + env vars)

### Phase 2: Repository Pattern (Week 1-2)

- [ ] Create `PositionRepository` with database-agnostic queries
- [ ] Create `OrderRepository`
- [ ] Create `PortfolioSnapshotRepository`
- [ ] Create `CashFlowRepository`
- [ ] Create `LoanRepository`
- [ ] Create `GreekSnapshotRepository`

### Phase 3: Migration System (Week 2)

- [ ] Set up SQLx migration system
- [ ] Create shared migration files (SQLite + PostgreSQL compatible)
- [ ] Test migrations on both databases
- [ ] Add migration runner to backend startup

### Phase 4: Backend Integration (Week 2)

- [ ] Integrate database pool into backend service
- [ ] Update `SystemSnapshot` to persist to database
- [ ] Implement periodic snapshot persistence
- [ ] Add database health check endpoint

### Phase 5: Testing (Week 3)

- [ ] Write unit tests with SQLite in-memory database
- [ ] Write integration tests with PostgreSQL
- [ ] Test data migration (SQLite → PostgreSQL)
- [ ] Test query performance on both databases

## Project Structure

```
agents/backend/
├── crates/
│   ├── database/              # NEW: Database abstraction layer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db_pool.rs     # DBPool enum
│   │   │   ├── migrations.rs  # Migration runner
│   │   │   └── repositories/
│   │   │       ├── mod.rs
│   │   │       ├── position_repository.rs
│   │   │       ├── order_repository.rs
│   │   │       ├── portfolio_snapshot_repository.rs
│   │   │       ├── cash_flow_repository.rs
│   │   │       ├── loan_repository.rs
│   │   │       └── greek_snapshot_repository.rs
│   │   └── migrations/        # SQL migration files
│   │       ├── 001_create_positions.sql
│   │       ├── 002_create_orders.sql
│   │       ├── 003_create_portfolio_snapshots.sql
│   │       ├── 004_create_cash_flow_events.sql
│   │       ├── 005_create_loans.sql
│   │       └── 006_create_greek_snapshots.sql
│   └── api/
│       └── src/
│           └── state.rs       # Updated to use repositories
└── config/
    └── default.toml           # Updated with database config
```

## Dependencies

Add to `agents/backend/Cargo.toml`:

```toml
[workspace.dependencies]

# ... existing dependencies ...

sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "postgres", "chrono", "uuid"] }

# For compile-time query checking (optional but recommended)
# sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "postgres", "chrono", "uuid", "offline"] }
```

Add to `crates/database/Cargo.toml`:

```toml
[dependencies]
sqlx = { workspace = true }
anyhow = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
```

## Benefits of This Approach

1. **Zero Setup for Development:** SQLite file, no database server
2. **Easy Testing:** In-memory SQLite for fast tests
3. **Seamless Scaling:** Change connection string, no code changes
4. **Type Safety:** Compile-time query checking (if using `query!` macro)
5. **Database-Agnostic:** Same code works with SQLite and PostgreSQL
6. **Cloud Ready:** Easy migration to AWS RDS, GCP Cloud SQL, or Vultr

## References

- [SQLx Documentation](https://docs.rs/sqlx/)
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [SQLx Migrations](https://docs.rs/sqlx/latest/sqlx/migrate/index.html)

---

**Next Steps:**

1. Review and approve design
2. Add SQLx dependencies to Cargo.toml
3. Implement database abstraction layer
4. Create repository pattern implementations
5. Set up migration system
6. Integrate with backend service
