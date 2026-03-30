//! Read-only journal view over persisted ledger transactions (`transactions` table).
//! Exposed via NATS `api.ledger.journal` (see `nats_adapter::topics::api::ledger`).

use serde::{Deserialize, Serialize};
use sqlx::Row;

/// One row from `transactions` (indexed columns + no full JSON payload in the list API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerJournalEntryDto {
    pub id: String,
    pub date: String,
    pub description: String,
    pub cleared: bool,
    #[serde(rename = "accountPaths")]
    pub account_paths: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerJournalListDto {
    pub entries: Vec<LedgerJournalEntryDto>,
    #[serde(rename = "totalCount")]
    pub total_count: usize,
}

/// Latest `limit` transactions by `date` descending (newest first).
pub async fn list_ledger_journal(limit: usize) -> Result<LedgerJournalListDto, String> {
    let pool = crate::ledger_sqlite::open_ledger_pool().await?;
    let limit = limit.clamp(1, 10_000) as i64;

    let rows = sqlx::query(
        r#"
        SELECT id, date, description, cleared, account_paths
        FROM transactions
        ORDER BY date DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query ledger journal: {e}"))?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id").map_err(|e| e.to_string())?;
        let date: String = row.try_get("date").map_err(|e| e.to_string())?;
        let description: String = row.try_get("description").map_err(|e| e.to_string())?;
        let cleared_i: i64 = row.try_get("cleared").map_err(|e| e.to_string())?;
        let account_paths: String = row.try_get("account_paths").map_err(|e| e.to_string())?;
        entries.push(LedgerJournalEntryDto {
            id,
            date,
            description,
            cleared: cleared_i != 0,
            account_paths,
        });
    }

    let total_count = entries.len();
    Ok(LedgerJournalListDto {
        entries,
        total_count,
    })
}
