//! Integration tests for ledger integration with position tracking
//!
//! These tests verify that ledger transactions are recorded correctly
//! when positions are updated through `apply_strategy_execution`.

#[cfg(test)]
mod tests {
    use super::super::state::*;
    use crate::ledger;
    use crate::ledger::engine::{LedgerEngine, PersistenceLayer, TransactionFilter};
    use crate::ledger::transaction::Transaction;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use uuid::Uuid;

    // Mock persistence layer for testing
    struct MockPersistence {
        transactions: Arc<RwLock<Vec<Transaction>>>,
    }

    #[async_trait]
    impl PersistenceLayer for MockPersistence {
        async fn save_transaction(&self, transaction: &Transaction) -> ledger::Result<()> {
            self.transactions.write().await.push(transaction.clone());
            Ok(())
        }

        async fn load_transaction(&self, id: &Uuid) -> ledger::Result<Option<Transaction>> {
            let transactions = self.transactions.read().await;
            Ok(transactions.iter().find(|t| t.id == *id).cloned())
        }

        async fn load_transactions(
            &self,
            filter: &TransactionFilter,
        ) -> ledger::Result<Vec<Transaction>> {
            let transactions = self.transactions.read().await;
            let mut result = transactions.clone();

            if let Some(ref account) = filter.account {
                result.retain(|t| t.postings.iter().any(|p| p.account == *account));
            }

            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_apply_strategy_execution_records_ledger_transaction() {
        let persistence = Arc::new(MockPersistence {
            transactions: Arc::new(RwLock::new(Vec::new())),
        });
        let ledger_engine = Arc::new(LedgerEngine::new(persistence.clone()));

        let mut snapshot = SystemSnapshot::default();
        snapshot.set_ledger(ledger_engine.clone());

        let decision =
            StrategyDecisionSnapshot::new("SPY".to_string(), 100, "BUY", 450.0, chrono::Utc::now());

        snapshot.apply_strategy_execution(decision);

        // Wait a bit for async ledger recording
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let transactions = persistence.transactions.read().await;
        assert_eq!(transactions.len(), 1);
        assert!(transactions[0].description.contains("SPY"));
    }

    #[tokio::test]
    async fn test_record_box_spread_async() {
        let persistence = Arc::new(MockPersistence {
            transactions: Arc::new(RwLock::new(Vec::new())),
        });
        let ledger_engine = Arc::new(LedgerEngine::new(persistence.clone()));

        let snapshot = SystemSnapshot {
            ledger: Some(ledger_engine.clone()),
            ..Default::default()
        };

        snapshot.record_box_spread_async("SPY", 450, 460, "20251219", 1000.0, Some("BOX-12345"));

        // Wait a bit for async ledger recording
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let transactions = persistence.transactions.read().await;
        assert_eq!(transactions.len(), 1);
        assert!(transactions[0].description.contains("Box Spread"));
        assert_eq!(
            transactions[0].metadata.get("strategy"),
            Some(&"box_spread".to_string())
        );
    }
}
