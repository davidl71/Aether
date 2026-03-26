//! Strategy control NATS request/reply handlers (read-only mode).
//! Subjects: api.strategy.*
//!
//! Note: Strategy start/stop/execute are deprecated in data-exploration mode.
//! Execution is disabled — platform is in read-only data-exploration mode.
//! See docs/DATA_EXPLORATION_MODE.md.

use std::sync::Arc;

use crate::handlers::{api_queue_group, handle_sub};
use api::{CommandContext, CommandEvent, ScenarioDto};
use broker_engine::BrokerEngine;
use bytes::Bytes;
use nats_adapter::async_nats::Client;
use nats_adapter::topics;
use tracing::warn;

const SUBJECT_STRATEGY_START: &str = "api.strategy.start";
const SUBJECT_STRATEGY_STOP: &str = "api.strategy.stop";
const SUBJECT_STRATEGY_CANCEL_ALL: &str = "api.strategy.cancel_all";
const SUBJECT_STRATEGY_EXECUTE: &str = "api.strategy.execute";

use crate::shared_state::SharedSnapshot;

/// Spawn Strategy control NATS API handlers.
pub async fn spawn(
    nc: Client,
    _strategy_controller: api::StrategyController,
    state: SharedSnapshot,
    _broker_engine: Option<Arc<dyn BrokerEngine>>,
) {
    let sub_start = match nc
        .queue_subscribe(SUBJECT_STRATEGY_START.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.start failed");
            return;
        }
    };
    let sub_stop = match nc
        .queue_subscribe(SUBJECT_STRATEGY_STOP.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.stop failed");
            return;
        }
    };
    let sub_cancel_all = match nc
        .queue_subscribe(SUBJECT_STRATEGY_CANCEL_ALL.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.cancel_all failed");
            return;
        }
    };
    let sub_execute = match nc
        .queue_subscribe(SUBJECT_STRATEGY_EXECUTE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.execute failed");
            return;
        }
    };

    let nc_start = nc.clone();
    tokio::spawn(handle_sub(
        nc_start.clone(),
        sub_start,
        move |_body: Option<Vec<u8>>| {
            let nc = nc_start.clone();
            async move {
                let command = CommandContext::new("start");
                let message = "strategy start is deprecated; backend is in data-exploration mode";
                let reply = command.failed_reply(message);
                publish_command_event(&nc, "start", &command.failed_event(message)).await;
                serde_json::to_vec(&reply).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));

    let nc_stop = nc.clone();
    tokio::spawn(handle_sub(
        nc_stop.clone(),
        sub_stop,
        move |_body: Option<Vec<u8>>| {
            let nc = nc_stop.clone();
            async move {
                let command = CommandContext::new("stop");
                let message = "strategy stop is deprecated; backend is in data-exploration mode";
                let reply = command.failed_reply(message);
                publish_command_event(&nc, "stop", &command.failed_event(message)).await;
                serde_json::to_vec(&reply).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));

    let state_cancel = state.clone();
    let nc_cancel = nc.clone();
    tokio::spawn(handle_sub(
        nc_cancel.clone(),
        sub_cancel_all,
        move |_body: Option<Vec<u8>>| {
            let state = state_cancel.clone();
            let nc = nc_cancel.clone();
            async move {
                let command = CommandContext::new("cancel_all");
                let open_count = state.read().await.orders.len();
                let error = format!(
                    "cancel_all is deprecated in data-exploration mode; {} order snapshot(s) remain visible but execution is disabled",
                    open_count
                );
                let reply = command.failed_reply(error.clone());
                publish_command_event(&nc, "cancel_all", &command.failed_event(error)).await;
                let mut value = serde_json::to_value(&reply)
                    .unwrap_or_else(|_| serde_json::json!({ "ok": false, "action": "cancel_all" }));
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("open_order_count".into(), serde_json::json!(open_count));
                }
                serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));

    let nc_execute = nc.clone();
    tokio::spawn(handle_sub(
        nc_execute.clone(),
        sub_execute,
        move |body: Option<Vec<u8>>| {
            let nc = nc_execute.clone();
            async move { execute_scenario_reply(&nc, body).await }
        },
    ));
}

async fn execute_scenario_reply(nc: &Client, body: Option<Vec<u8>>) -> Vec<u8> {
    let command = CommandContext::new("execute_scenario");
    let Some(bytes) = body else {
        let reply = command.failed_reply("missing request body");
        publish_command_event(
            nc,
            "execute_scenario",
            &command.failed_event("missing request body"),
        )
        .await;
        return serde_json::to_vec(&reply).unwrap_or_else(|_| b"{}".to_vec());
    };

    let scenario: ScenarioDto = match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(e) => {
            let err = format!("failed to parse scenario: {}", e);
            let reply = command.failed_reply(err.clone());
            publish_command_event(nc, "execute_scenario", &command.failed_event(err)).await;
            return serde_json::to_vec(&reply).unwrap_or_else(|_| b"{}".to_vec());
        }
    };
    let message = format!(
        "execute_scenario is deprecated in data-exploration mode; scenario for {} {} was not submitted",
        scenario.symbol, scenario.expiration
    );
    let reply = command.failed_reply(message.clone());
    publish_command_event(nc, "execute_scenario", &command.failed_event(message)).await;
    serde_json::to_vec(&reply).unwrap_or_else(|_| b"{}".to_vec())
}

async fn publish_command_event(nc: &Client, action: &str, event: &CommandEvent) {
    let subject = topics::system::commands(action);
    let body = match serde_json::to_vec(event) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(action = %action, error = %e, "serialize command event failed");
            return;
        }
    };

    if let Err(e) = nc.publish(subject.clone(), Bytes::from(body)).await {
        warn!(action = %action, subject = %subject, error = %e, "publish command event failed");
    }
}
