//! NATS request/reply handlers for api.discount_bank.*, api.loans.*, and api.fmp.*.
//! Scope per docs/platform/NATS_API.md §3.
//! FMP fundamentals wired when FMP_API_KEY is set (task T-1773509396765766000).

use std::sync::Arc;

use api::discount_bank::{get_balance, get_bank_accounts, get_transactions, ImportPositionsQuery};
use api::LoanRepository;
use bytes::Bytes;
use futures::StreamExt;
use market_data::FmpClient;
use nats_adapter::NatsClient;
use nats_adapter::async_nats::Client;
use reqwest::Client as ReqwestClient;
use serde_json::Value;
use tracing::{info, warn};

const SUBJECT_DISCOUNT_BANK_BALANCE: &str = "api.discount_bank.balance";
const SUBJECT_DISCOUNT_BANK_TRANSACTIONS: &str = "api.discount_bank.transactions";
const SUBJECT_DISCOUNT_BANK_BANK_ACCOUNTS: &str = "api.discount_bank.bank_accounts";
const SUBJECT_DISCOUNT_BANK_IMPORT_POSITIONS: &str = "api.discount_bank.import_positions";
const SUBJECT_LOANS_LIST: &str = "api.loans.list";
const SUBJECT_LOANS_GET: &str = "api.loans.get";

const SUBJECT_FMP_INCOME_STATEMENT: &str = "api.fmp.income_statement";
const SUBJECT_FMP_BALANCE_SHEET: &str = "api.fmp.balance_sheet";
const SUBJECT_FMP_CASH_FLOW: &str = "api.fmp.cash_flow";
const SUBJECT_FMP_QUOTE: &str = "api.fmp.quote";

/// Spawn NATS API handlers for Discount Bank, Loans, and optionally FMP fundamentals.
pub fn spawn(
    nats_client: Arc<NatsClient>,
    loan_repo: Option<Arc<LoanRepository>>,
    fmp_client: Option<Arc<FmpClient>>,
) {
    let nc = nats_client.client().clone();
    let nc_loans = nc.clone();
    tokio::spawn(async move {
        run_discount_bank(nc).await;
    });
    tokio::spawn(async move {
        run_loans(nc_loans, loan_repo).await;
    });
    if let Some(fmp) = fmp_client {
        let nc_fmp = nats_client.client().clone();
        tokio::spawn(async move {
            run_fmp(nc_fmp, fmp).await;
        });
        info!("NATS API handlers spawned (discount_bank, loans, fmp)");
    } else {
        info!("NATS API handlers spawned (discount_bank, loans)");
    }
}

async fn run_discount_bank(nc: Client) {
    let sub_balance = match nc.subscribe(SUBJECT_DISCOUNT_BANK_BALANCE.to_string()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.balance failed");
            return;
        }
    };
    let sub_tx = match nc.subscribe(SUBJECT_DISCOUNT_BANK_TRANSACTIONS.to_string()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.transactions failed");
            return;
        }
    };
    let sub_accounts = match nc.subscribe(SUBJECT_DISCOUNT_BANK_BANK_ACCOUNTS.to_string()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.bank_accounts failed");
            return;
        }
    };
    let sub_import = match nc.subscribe(SUBJECT_DISCOUNT_BANK_IMPORT_POSITIONS.to_string()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.import_positions failed");
            return;
        }
    };

    let client = ReqwestClient::new();

    tokio::spawn(handle_sub(nc.clone(), sub_balance, |_body: Option<Vec<u8>>| async {
        let r = get_balance().await;
        serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
    }));
    tokio::spawn(handle_sub(nc.clone(), sub_tx, |body: Option<Vec<u8>>| async move {
        let limit = body
            .as_deref()
            .and_then(|b| serde_json::from_slice::<Value>(b).ok())
            .and_then(|v| v.get("limit").and_then(Value::as_u64))
            .unwrap_or(100) as usize;
        let r = get_transactions(limit).await;
        serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
    }));
    tokio::spawn(handle_sub(nc.clone(), sub_accounts, |_body: Option<Vec<u8>>| async {
        let r = get_bank_accounts().await;
        serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
    }));
    tokio::spawn(handle_sub_with_client(nc, sub_import, client));
}

async fn handle_sub<F, Fut>(
    nc: Client,
    mut sub: nats_adapter::async_nats::Subscriber,
    handler: F,
) where
    F: Fn(Option<Vec<u8>>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Vec<u8>> + Send,
{
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let body = if msg.payload.is_empty() {
            None
        } else {
            Some(msg.payload.to_vec())
        };
        let response = handler(body).await;
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply publish failed");
        }
    }
}

async fn handle_sub_with_client(
    nc: Client,
    mut sub: nats_adapter::async_nats::Subscriber,
    client: ReqwestClient,
) {
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

async fn run_loans(nc: Client, loan_repo: Option<Arc<LoanRepository>>) {
    let repo: Arc<LoanRepository> = match loan_repo {
        Some(r) => r,
        None => {
            warn!("LoanRepository not configured, loans API handlers disabled");
            return;
        }
    };

    let sub_list = match nc.subscribe(SUBJECT_LOANS_LIST.to_string()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_get = match nc.subscribe(SUBJECT_LOANS_GET.to_string()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.get failed");
            return;
        }
    };

    let repo_list = repo.clone();
    tokio::spawn(handle_sub(nc.clone(), sub_list, move |_body: Option<Vec<u8>>| {
        let repo = repo_list.clone();
        async move {
            let list = repo.list().await;
            let r: Result<_, String> = Ok(list);
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        }
    }));
    tokio::spawn(handle_sub(nc, sub_get, move |body: Option<Vec<u8>>| {
        let repo = repo.clone();
        async move {
            let loan_id = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| v.get("loan_id").and_then(Value::as_str).map(String::from));
            let r = match loan_id {
                Some(id) => Ok(repo.get(&id).await),
                None => Err("loan_id required".to_string()),
            };
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        }
    }));
}
