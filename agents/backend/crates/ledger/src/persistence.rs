//! Persistence layer for ledger transactions
//!
//! Provides database-backed persistence for ledger transactions using SQLite.
//! Can be extended to support PostgreSQL in the future.

use crate::engine::{PersistenceLayer, TransactionFilter};
use crate::error::{LedgerError, Result};
use crate::transaction::Transaction;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

/// SQLite-based persistence layer
pub struct SqlitePersistence {
    pool: SqlitePool,
}

impl SqlitePersistence {
    /// Create new SQLite persistence layer
    ///
    /// # Arguments
    /// * `database_url` - SQLite database URL (e.g., "sqlite:ledger.db" or "sqlite::memory:")
    ///
    /// WAL mode is enabled so multiple readers can coexist with one writer. This process
    /// must be the only writer to the database; Python/integration layers should read via
    /// REST API (GET /api/ledger/...) and never write directly to the same file.
    ///
    /// # Example
    /// ```no_run
    /// use ledger::SqlitePersistence;
    ///
    /// # tokio_test::block_on(async {
    /// let persistence = SqlitePersistence::new("sqlite:ledger.db").await.unwrap();
    /// # });
    /// ```
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(|e| LedgerError::Persistence(anyhow::anyhow!("Invalid database URL: {}", e)))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await
            .map_err(|e| {
                LedgerError::Persistence(anyhow::anyhow!("Failed to connect to database: {}", e))
            })?;

        // Initialize database schema
        Self::init_schema(&pool).await?;

        // Ensure WAL mode is active (P1-A: allows concurrent readers with single writer)
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await
            .map_err(|e| LedgerError::Persistence(anyhow::anyhow!("PRAGMA journal_mode=WAL: {}", e)))?;

        Ok(Self { pool })
    }

    /// Initialize database schema
    async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                description TEXT NOT NULL,
                cleared INTEGER NOT NULL DEFAULT 1,
                transaction_json TEXT NOT NULL,
                account_paths TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| LedgerError::Persistence(anyhow::anyhow!("Failed to create schema: {}", e)))?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date)")
            .execute(pool)
            .await
            .map_err(|e| {
                LedgerError::Persistence(anyhow::anyhow!("Failed to create index: {}", e))
            })?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_transactions_accounts ON transactions(account_paths)",
        )
        .execute(pool)
        .await
        .map_err(|e| LedgerError::Persistence(anyhow::anyhow!("Failed to create index: {}", e)))?;

        debug!("Ledger database schema initialized");
        Ok(())
    }

    /// Extract account paths from transaction for indexing
    fn extract_account_paths(transaction: &Transaction) -> String {
        transaction
            .postings
            .iter()
            .map(|p| p.account.to_string())
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Export all transactions to Ledger CLI format
    pub async fn export_to_ledger_cli(&self) -> Result<String> {
        use crate::export::LedgerExporter;
        let transactions = self
            .load_transactions(&TransactionFilter::default())
            .await?;
        Ok(LedgerExporter::export_transactions(&transactions))
    }
}

#[async_trait]
impl PersistenceLayer for SqlitePersistence {
    async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        let transaction_json = serde_json::to_string(transaction).map_err(|e| {
            LedgerError::Persistence(anyhow::anyhow!("Failed to serialize transaction: {}", e))
        })?;

        let account_paths = Self::extract_account_paths(transaction);

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO transactions (
                id, date, description, cleared, transaction_json, account_paths
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(transaction.id.to_string())
        .bind(transaction.date.to_rfc3339())
        .bind(&transaction.description)
        .bind(if transaction.cleared { 1 } else { 0 })
        .bind(&transaction_json)
        .bind(&account_paths)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            LedgerError::Persistence(anyhow::anyhow!("Failed to save transaction: {}", e))
        })?;

        debug!(transaction_id = %transaction.id, "Transaction saved to database");
        Ok(())
    }

    async fn load_transaction(&self, id: &Uuid) -> Result<Option<Transaction>> {
        let row = sqlx::query("SELECT transaction_json FROM transactions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                LedgerError::Persistence(anyhow::anyhow!("Failed to load transaction: {}", e))
            })?;

        if let Some(row) = row {
            let json_str: String = row.get(0);
            let transaction: Transaction = serde_json::from_str(&json_str).map_err(|e| {
                LedgerError::Persistence(anyhow::anyhow!(
                    "Failed to deserialize transaction: {}",
                    e
                ))
            })?;
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }

    async fn load_transactions(&self, filter: &TransactionFilter) -> Result<Vec<Transaction>> {
        let mut query = String::from("SELECT transaction_json FROM transactions WHERE 1=1");
        let mut bind_values: Vec<String> = Vec::new();

        // Apply account filter
        if let Some(ref account) = filter.account {
            query.push_str(" AND account_paths LIKE ?");
            bind_values.push(format!("%{}%", account.to_string()));
        }

        // Apply date range filter
        if let Some(ref start_date) = filter.start_date {
            query.push_str(" AND date >= ?");
            bind_values.push(start_date.to_rfc3339());
        }

        if let Some(ref end_date) = filter.end_date {
            query.push_str(" AND date <= ?");
            bind_values.push(end_date.to_rfc3339());
        }

        // Apply description filter
        if let Some(ref desc) = filter.description {
            query.push_str(" AND description LIKE ?");
            bind_values.push(format!("%{}%", desc));
        }

        // Order by date
        query.push_str(" ORDER BY date ASC");

        let mut sql_query = sqlx::query(&query);
        for value in bind_values {
            sql_query = sql_query.bind(value);
        }

        let rows = sql_query.fetch_all(&self.pool).await.map_err(|e| {
            LedgerError::Persistence(anyhow::anyhow!("Failed to query transactions: {}", e))
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            let json_str: String = row.get(0);
            match serde_json::from_str::<Transaction>(&json_str) {
                Ok(transaction) => {
                    // Apply metadata filter if present
                    if !filter.metadata.is_empty() {
                        let mut matches = true;
                        for (key, value) in &filter.metadata {
                            if let Some(tx_value) = transaction.metadata.get(key) {
                                if tx_value != value {
                                    matches = false;
                                    break;
                                }
                            } else {
                                matches = false;
                                break;
                            }
                        }
                        if !matches {
                            continue;
                        }
                    }
                    transactions.push(transaction);
                }
                Err(e) => {
                    warn!(error = %e, "Failed to deserialize transaction from database");
                }
            }
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::accounts;
    use crate::currency::Currency;
    use crate::money::Money;
    use crate::transaction::TransactionBuilder;
    use rust_decimal::Decimal;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sqlite_persistence_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_ledger.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let persistence = SqlitePersistence::new(&db_url).await.unwrap();

        let transaction = TransactionBuilder::new("Test transaction")
            .debit(
                accounts::ibkr_cash(),
                Money::new(Decimal::from(100), Currency::USD),
            )
            .credit(
                accounts::equity_capital(),
                Money::new(Decimal::from(100), Currency::USD),
            )
            .build()
            .unwrap();

        // Save transaction
        persistence.save_transaction(&transaction).await.unwrap();

        // Load transaction
        let loaded = persistence.load_transaction(&transaction.id).await.unwrap();
        assert!(loaded.is_some());
        let loaded_tx = loaded.unwrap();
        assert_eq!(loaded_tx.id, transaction.id);
        assert_eq!(loaded_tx.description, transaction.description);
    }

    #[tokio::test]
    async fn test_sqlite_persistence_query() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_ledger.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let persistence = SqlitePersistence::new(&db_url).await.unwrap();

        // Save multiple transactions
        for i in 0..5 {
            let transaction = TransactionBuilder::new(format!("Transaction {}", i))
                .debit(
                    accounts::ibkr_cash(),
                    Money::new(Decimal::from(10 * i), Currency::USD),
                )
                .credit(
                    accounts::equity_capital(),
                    Money::new(Decimal::from(10 * i), Currency::USD),
                )
                .build()
                .unwrap();
            persistence.save_transaction(&transaction).await.unwrap();
        }

        // Query all transactions
        let transactions = persistence
            .load_transactions(&TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(transactions.len(), 5);

        // Query by account
        let filter = TransactionFilter {
            account: Some(accounts::ibkr_cash()),
            ..Default::default()
        };
        let transactions = persistence.load_transactions(&filter).await.unwrap();
        assert!(transactions.len() > 0);
    }

    #[tokio::test]
    async fn test_export_to_ledger_cli() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_ledger.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let persistence = SqlitePersistence::new(&db_url).await.unwrap();

        // Save transaction
        let transaction = TransactionBuilder::new("Buy SPY")
            .debit(
                accounts::ibkr_position("SPY"),
                Money::new(Decimal::from(45000), Currency::USD),
            )
            .credit(
                accounts::ibkr_cash(),
                Money::new(Decimal::from(45000), Currency::USD),
            )
            .with_metadata("trade_id", "ORD-12345")
            .build()
            .unwrap();
        persistence.save_transaction(&transaction).await.unwrap();

        // Export to Ledger CLI format
        let exported = persistence.export_to_ledger_cli().await.unwrap();
        assert!(exported.contains("Buy SPY"));
        assert!(exported.contains("Assets:IBKR:SPY"));
        assert!(exported.contains("Assets:IBKR:Cash"));
    }
}
