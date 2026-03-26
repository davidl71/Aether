//! Loans NATS request/reply handlers.
//! Subjects: api.loans.*

use std::sync::Arc;

use crate::handlers::{api_queue_group, handle_sub};
use api::loans::loans_response_proto;
use api::{LoanRecord, LoanRepository};
use nats_adapter::async_nats::Client;
use prost::Message;
use serde_json::Value;
use tracing::{info, warn};

const SUBJECT_LOANS_LIST: &str = "api.loans.list";
const SUBJECT_LOANS_LIST_PROTO: &str = "api.loans.list.proto";
const SUBJECT_LOANS_GET: &str = "api.loans.get";
const SUBJECT_LOANS_CREATE: &str = "api.loans.create";
const SUBJECT_LOANS_UPDATE: &str = "api.loans.update";
const SUBJECT_LOANS_DELETE: &str = "api.loans.delete";

/// Spawn Loans NATS API handlers.
pub async fn spawn(nc: Client, repo: Arc<LoanRepository>) {
    let sub_list = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_list_proto = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST_PROTO.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list.proto failed");
            return;
        }
    };
    let sub_get = match nc
        .queue_subscribe(SUBJECT_LOANS_GET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.get failed");
            return;
        }
    };
    let sub_create = match nc
        .queue_subscribe(SUBJECT_LOANS_CREATE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.create failed");
            return;
        }
    };
    let sub_update = match nc
        .queue_subscribe(SUBJECT_LOANS_UPDATE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.update failed");
            return;
        }
    };
    let sub_delete = match nc
        .queue_subscribe(SUBJECT_LOANS_DELETE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.delete failed");
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
                    .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                    .and_then(|v| v.get("loan_id").and_then(Value::as_str).map(String::from));
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
    tokio::spawn(handle_sub(nc, sub_delete, move |body: Option<Vec<u8>>| {
        let repo = repo.clone();
        async move {
            let loan_id = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| v.get("loan_id").and_then(Value::as_str).map(String::from));
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
        .queue_subscribe(SUBJECT_LOANS_LIST.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_list_proto = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST_PROTO.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list.proto failed");
            return;
        }
    };
    let sub_get = match nc
        .queue_subscribe(SUBJECT_LOANS_GET.to_string(), api_queue_group())
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
