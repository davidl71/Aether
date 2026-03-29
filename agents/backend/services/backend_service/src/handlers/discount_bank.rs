//! Discount Bank NATS request/reply handlers.
//! Subjects: api.discount_bank.*

use crate::handlers::{api_queue_group, handle_sub};
use api::discount_bank::{get_balance, get_bank_accounts, get_transactions, ImportPositionsQuery};
use bytes::Bytes;
use nats_adapter::async_nats::Client;
use nats_adapter::topics;
use reqwest::Client as ReqwestClient;
use serde_json::Value;
use tracing::warn;

/// Spawn Discount Bank NATS API handlers.
pub async fn spawn(nc: Client) {
    let sub_balance = match nc
        .queue_subscribe(
            topics::api::discount_bank::BALANCE.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.balance failed");
            return;
        }
    };
    let sub_tx = match nc
        .queue_subscribe(
            topics::api::discount_bank::TRANSACTIONS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.transactions failed");
            return;
        }
    };
    let sub_accounts = match nc
        .queue_subscribe(
            topics::api::discount_bank::BANK_ACCOUNTS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.bank_accounts failed");
            return;
        }
    };
    let sub_import = match nc
        .queue_subscribe(
            topics::api::discount_bank::IMPORT_POSITIONS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.import_positions failed");
            return;
        }
    };

    let client = ReqwestClient::new();

    tokio::spawn(handle_sub(
        nc.clone(),
        sub_balance,
        |_body: Option<Vec<u8>>| async {
            let r: Result<api::discount_bank::DiscountBankBalanceDto, String> = get_balance().await;
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_tx,
        |body: Option<Vec<u8>>| async move {
            let limit = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| v.get("limit").and_then(Value::as_u64))
                .unwrap_or(100) as usize;
            let r: Result<api::discount_bank::DiscountBankTransactionsListDto, String> =
                get_transactions(limit).await;
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_accounts,
        |_body: Option<Vec<u8>>| async {
            let r: Result<api::discount_bank::BankAccountsListDto, String> =
                get_bank_accounts().await;
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub_with_import(nc, sub_import, client));
}

async fn handle_sub_with_import(
    nc: Client,
    mut sub: nats_adapter::async_nats::Subscriber,
    client: ReqwestClient,
) {
    use futures::StreamExt;

    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let query: ImportPositionsQuery = msg
            .payload
            .as_ref()
            .len()
            .gt(&0)
            .then(|| serde_json::from_slice(msg.payload.as_ref()))
            .and_then(Result::ok)
            .unwrap_or_else(|| ImportPositionsQuery {
                broker: "ibkr".to_string(),
                account_id: None,
                dry_run: Some(true),
            });
        let r = api::discount_bank::import_positions(query, &client).await;
        let response = serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply publish failed");
        }
    }
}
