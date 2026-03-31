//! Ledger NATS request/reply handlers.
//! Subjects: api.ledger.*

use crate::handlers::{api_queue_group, handle_sub};
use api::ledger_journal::list_ledger_journal;
use nats_adapter::async_nats::Client;
use nats_adapter::topics;
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Deserialize)]
struct LedgerJournalRequest {
    #[serde(default)]
    limit: Option<u64>,
}

/// Spawn Ledger NATS API handlers.
pub async fn spawn(nc: Client) {
    let sub_journal = match nc
        .queue_subscribe(topics::api::ledger::JOURNAL.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.ledger.journal failed");
            return;
        }
    };

    tokio::spawn(handle_sub(nc, sub_journal, |body: Option<Vec<u8>>| async move {
        let limit = body
            .as_deref()
            .and_then(|b| serde_json::from_slice::<LedgerJournalRequest>(b).ok())
            .and_then(|req| req.limit)
            .unwrap_or(200) as usize;

        let r: Result<api::LedgerJournalListDto, String> = list_ledger_journal(limit).await;
        serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
    }));
}

