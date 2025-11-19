use crate::account::AccountPath;
use crate::error::Result;
use crate::money::Money;
use crate::transaction::Transaction;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Transaction filter for querying transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    pub account: Option<AccountPath>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Persistence layer trait for transaction storage
#[async_trait]
pub trait PersistenceLayer: Send + Sync {
    /// Save transaction to storage
    async fn save_transaction(&self, transaction: &Transaction) -> Result<()>;

    /// Load transaction by ID
    async fn load_transaction(&self, id: &Uuid) -> Result<Option<Transaction>>;

    /// Load transactions matching filter
    async fn load_transactions(&self, filter: &TransactionFilter) -> Result<Vec<Transaction>>;
}

/// Ledger engine for recording and querying transactions
pub struct LedgerEngine {
    persistence: Arc<dyn PersistenceLayer>,
    balance_cache: Arc<RwLock<HashMap<AccountPath, Money>>>,
}

impl LedgerEngine {
    /// Create a new ledger engine
    pub fn new(persistence: Arc<dyn PersistenceLayer>) -> Self {
        Self {
            persistence,
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a transaction (async for database operations)
    pub async fn record_transaction(&self, transaction: Transaction) -> Result<()> {
        // Validate transaction balances
        transaction.validate_balance()?;

        // Persist transaction
        self.persistence.save_transaction(&transaction).await?;

        // Update balance cache
        self.update_balance_cache(&transaction).await?;

        Ok(())
    }

    /// Get account balance (from cache if available, otherwise calculate)
    pub async fn get_balance(&self, account: &AccountPath) -> Result<Money> {
        // Check cache first
        if let Some(balance) = self.balance_cache.read().await.get(account) {
            return Ok(balance.clone());
        }

        // Calculate from transactions if not in cache
        let balance = self.calculate_balance(account).await?;

        // Update cache
        self.balance_cache.write().await.insert(account.clone(), balance.clone());

        Ok(balance)
    }

    /// Query transactions with filters
    pub async fn query_transactions(&self, filter: TransactionFilter) -> Result<Vec<Transaction>> {
        self.persistence.load_transactions(&filter).await
    }

    /// Calculate balance from all transactions (bypasses cache)
    async fn calculate_balance(&self, account: &AccountPath) -> Result<Money> {
        let filter = TransactionFilter {
            account: Some(account.clone()),
            ..Default::default()
        };

        let transactions = self.persistence.load_transactions(&filter).await?;

        let mut balance = Money::zero();
        let mut currency: Option<crate::Currency> = None;

        for transaction in transactions {
            for posting in &transaction.postings {
                if posting.account == *account {
                    if currency.is_none() {
                        currency = Some(posting.amount.currency);
                    }

                    balance = (balance + posting.amount.clone())?;
                }
            }
        }

        // Set currency if not set yet (use USD as default)
        if let Some(currency) = currency {
            balance.currency = currency;
        }

        Ok(balance)
    }

    /// Update balance cache after recording transaction
    async fn update_balance_cache(&self, transaction: &Transaction) -> Result<()> {
        let mut cache = self.balance_cache.write().await;

        for posting in &transaction.postings {
            let current_balance = cache
                .get(&posting.account)
                .cloned()
                .unwrap_or_else(|| Money::zero_with_currency(posting.amount.currency));

            let new_balance = (current_balance + posting.amount.clone())?;
            cache.insert(posting.account.clone(), new_balance);
        }

        Ok(())
    }

    /// Clear balance cache (forces recalculation on next query)
    pub async fn clear_cache(&self) {
        self.balance_cache.write().await.clear();
    }

    /// Rebuild balance cache from all transactions
    pub async fn rebuild_cache(&self) -> Result<()> {
        self.clear_cache().await;

        // Load all transactions
        let transactions = self.persistence.load_transactions(&TransactionFilter::default()).await?;

        // Rebuild cache from all transactions
        for transaction in transactions {
            self.update_balance_cache(&transaction).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountPath;
    use crate::currency::Currency;
    use crate::money::Money;
    use crate::transaction::TransactionBuilder;
    use rust_decimal::Decimal;
    use std::sync::Arc;

    // Mock persistence layer for testing
    struct MockPersistence {
        transactions: Arc<RwLock<Vec<Transaction>>>,
    }

    #[async_trait]
    impl PersistenceLayer for MockPersistence {
        async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
            self.transactions.write().await.push(transaction.clone());
            Ok(())
        }

        async fn load_transaction(&self, id: &Uuid) -> Result<Option<Transaction>> {
            let transactions = self.transactions.read().await;
            Ok(transactions.iter().find(|t| t.id == *id).cloned())
        }

        async fn load_transactions(&self, filter: &TransactionFilter) -> Result<Vec<Transaction>> {
            let transactions = self.transactions.read().await;
            let mut result = transactions.clone();

            // Apply filters
            if let Some(ref account) = filter.account {
                result.retain(|t| {
                    t.postings
                        .iter()
                        .any(|p| p.account == *account)
                });
            }

            if let Some(ref desc) = filter.description {
                result.retain(|t| t.description.contains(desc));
            }

            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_record_transaction() {
        let persistence = Arc::new(MockPersistence {
            transactions: Arc::new(RwLock::new(Vec::new())),
        });
        let engine = LedgerEngine::new(persistence.clone());

        let account1 = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let account2 = AccountPath::from_string("Equity:Capital").unwrap();
        let amount = Money::new(Decimal::from(100), Currency::USD);

        let transaction = TransactionBuilder::new("Test transaction")
            .debit(account1.clone(), amount.clone())
            .credit(account2, amount)
            .build()
            .unwrap();

        engine.record_transaction(transaction).await.unwrap();

        let transactions = persistence.transactions.read().await;
        assert_eq!(transactions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_balance() {
        let persistence = Arc::new(MockPersistence {
            transactions: Arc::new(RwLock::new(Vec::new())),
        });
        let engine = LedgerEngine::new(persistence);

        let account1 = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let account2 = AccountPath::from_string("Equity:Capital").unwrap();
        let amount = Money::new(Decimal::from(100), Currency::USD);

        let transaction = TransactionBuilder::new("Test transaction")
            .debit(account1.clone(), amount.clone())
            .credit(account2, amount)
            .build()
            .unwrap();

        engine.record_transaction(transaction).await.unwrap();

        let balance = engine.get_balance(&account1).await.unwrap();
        assert_eq!(balance.amount, Decimal::from(100));
    }
}
