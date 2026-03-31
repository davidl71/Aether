//! Loans NATS request/reply handlers.
//! Subjects: api.loans.*

use std::sync::Arc;

use crate::handlers::{api_queue_group, handle_sub};
use api::loans::loans_response_proto;
use api::{LoanRecord, LoanRepository, LoansBulkImportRequest, LoansBulkImportResponse};
use nats_adapter::async_nats::Client;
use nats_adapter::topics;
use prost::Message;
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct LoanIdRequest {
    loan_id: String,
}

/// Spawn Loans NATS API handlers.
pub async fn spawn(nc: Client, repo: Arc<LoanRepository>) {
    let sub_list = match nc
        .queue_subscribe(topics::api::loans::LIST.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_list_proto = match nc
        .queue_subscribe(
            topics::api::loans::LIST_PROTO.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list.proto failed");
            return;
        }
    };
    let sub_get = match nc
        .queue_subscribe(topics::api::loans::GET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.get failed");
            return;
        }
    };
    let sub_create = match nc
        .queue_subscribe(topics::api::loans::CREATE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.create failed");
            return;
        }
    };
    let sub_update = match nc
        .queue_subscribe(topics::api::loans::UPDATE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.update failed");
            return;
        }
    };
    let sub_delete = match nc
        .queue_subscribe(topics::api::loans::DELETE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.delete failed");
            return;
        }
    };
    let sub_import_bulk = match nc
        .queue_subscribe(
            topics::api::loans::IMPORT_BULK.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.import_bulk failed");
            return;
        }
    };

    let repo_list = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list,
        move |_body: Option<Vec<u8>>| {
            let repo = repo_list.clone();
            async move {
                let list = repo.list().await;
                let r: Result<_, String> = Ok(list);
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_list_proto = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list_proto,
        move |_body: Option<Vec<u8>>| {
            let repo = repo_list_proto.clone();
            async move {
                let list = repo.list().await;
                let resp = loans_response_proto(&list);
                resp.encode_to_vec()
            }
        },
    ));
    let repo_get = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_get,
        move |body: Option<Vec<u8>>| {
            let repo = repo_get.clone();
            async move {
                let loan_id = body
                    .as_deref()
                    .and_then(|b| serde_json::from_slice::<LoanIdRequest>(b).ok())
                    .map(|req| req.loan_id);
                let r = match loan_id {
                    Some(id) => Ok(repo.get(&id).await),
                    None => Err("loan_id required".to_string()),
                };
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_create = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_create,
        move |body: Option<Vec<u8>>| {
            let repo = repo_create.clone();
            async move {
                let loan: LoanRecord =
                    match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                        Some(l) => l,
                        None => {
                            let r: Result<(), String> =
                                Err("request body must be a LoanRecord".to_string());
                            return serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
                        }
                    };
                let r = repo.create(loan).await;
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_update = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_update,
        move |body: Option<Vec<u8>>| {
            let repo = repo_update.clone();
            async move {
                let loan: LoanRecord =
                    match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                        Some(l) => l,
                        None => {
                            let r: Result<(), String> =
                                Err("request body must be a LoanRecord".to_string());
                            return serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
                        }
                    };
                let loan_id = loan.loan_id.clone();
                let r = repo.update(&loan_id, loan).await;
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_import = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_import_bulk,
        move |body: Option<Vec<u8>>| {
            let repo = repo_import.clone();
            async move {
                let req: LoansBulkImportRequest =
                    match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                        Some(r) => r,
                        None => {
                            let r: Result<LoansBulkImportResponse, String> = Err(
                                "request body must be LoansBulkImportRequest { loans: [...] }"
                                    .to_string(),
                            );
                            return serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
                        }
                    };
                let summary = repo.import_bulk(req.loans).await;
                let r: Result<LoansBulkImportResponse, String> = Ok(summary);
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    tokio::spawn(handle_sub(nc, sub_delete, move |body: Option<Vec<u8>>| {
        let repo = repo.clone();
        async move {
            let loan_id = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<LoanIdRequest>(b).ok())
                .map(|req| req.loan_id);
            let r = match loan_id {
                Some(id) => repo.delete(&id).await,
                None => Err("loan_id required".to_string()),
            };
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        }
    }));
}

/// Spawn Loans NATS API handlers (unconfigured - returns error responses).
pub async fn spawn_unconfigured(nc: Client) {
    let sub_list = match nc
        .queue_subscribe(topics::api::loans::LIST.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_list_proto = match nc
        .queue_subscribe(
            topics::api::loans::LIST_PROTO.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list.proto failed");
            return;
        }
    };
    let sub_get = match nc
        .queue_subscribe(topics::api::loans::GET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.get failed");
            return;
        }
    };

    info!("LoanRepository not configured, loans API returning unconfigured replies");

    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list,
        move |_body: Option<Vec<u8>>| async move {
            let r: Result<Vec<LoanRecord>, String> =
                Err("Loan database not configured".to_string());
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list_proto,
        move |_body: Option<Vec<u8>>| async move { loans_response_proto(&[]).encode_to_vec() },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_get,
        move |_body: Option<Vec<u8>>| async move {
            let r: Result<Option<LoanRecord>, String> =
                Err("Loan database not configured".to_string());
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
}
