use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use serde::{Deserialize, Serialize};

static COMMAND_SEQ: AtomicU64 = AtomicU64::new(1);

fn generate_command_id(action: &str) -> String {
    let seq = COMMAND_SEQ.fetch_add(1, Ordering::Relaxed);
    format!(
        "cmd-{}-{}-{seq:016x}",
        Utc::now().timestamp_micros(),
        action
    )
}

fn issued_at_now() -> String {
    Utc::now().to_rfc3339()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    pub command_id: String,
    pub issued_at: String,
    pub action: String,
}

impl CommandContext {
    pub fn new(action: impl Into<String>) -> Self {
        let action = action.into();
        Self {
            command_id: generate_command_id(&action),
            issued_at: issued_at_now(),
            action,
        }
    }

    fn event(
        &self,
        status: CommandStatus,
        message: Option<String>,
        error: Option<String>,
    ) -> CommandEvent {
        CommandEvent {
            command_id: self.command_id.clone(),
            issued_at: self.issued_at.clone(),
            action: self.action.clone(),
            status,
            message,
            error,
        }
    }

    fn reply(
        &self,
        ok: bool,
        status: CommandStatus,
        message: Option<String>,
        error: Option<String>,
    ) -> CommandReply {
        CommandReply {
            command_id: self.command_id.clone(),
            issued_at: self.issued_at.clone(),
            ok,
            action: self.action.clone(),
            status,
            message,
            error,
        }
    }

    pub fn accepted_event(&self, message: impl Into<String>) -> CommandEvent {
        self.event(CommandStatus::Accepted, Some(message.into()), None)
    }

    pub fn completed_event(&self, message: impl Into<String>) -> CommandEvent {
        self.event(CommandStatus::Completed, Some(message.into()), None)
    }

    pub fn failed_event(&self, error: impl Into<String>) -> CommandEvent {
        self.event(CommandStatus::Failed, None, Some(error.into()))
    }

    pub fn accepted_reply(&self, message: impl Into<String>) -> CommandReply {
        self.reply(true, CommandStatus::Accepted, Some(message.into()), None)
    }

    pub fn completed_reply(&self, message: impl Into<String>) -> CommandReply {
        self.reply(true, CommandStatus::Completed, Some(message.into()), None)
    }

    pub fn failed_reply(&self, error: impl Into<String>) -> CommandReply {
        self.reply(false, CommandStatus::Failed, None, Some(error.into()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    pub command_id: String,
    pub issued_at: String,
    pub action: String,
    pub status: CommandStatus,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl CommandEvent {
    /// Maps a NATS JSON `CommandEvent` (no `ok` field) into `CommandReply` for the TUI status model.
    pub fn to_reply(&self) -> CommandReply {
        let ok = !matches!(self.status, CommandStatus::Failed);
        CommandReply {
            command_id: self.command_id.clone(),
            issued_at: self.issued_at.clone(),
            ok,
            action: self.action.clone(),
            status: self.status.clone(),
            message: self.message.clone(),
            error: self.error.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Accepted,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandReply {
    pub command_id: String,
    pub issued_at: String,
    pub ok: bool,
    pub action: String,
    pub status: CommandStatus,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl CommandReply {
    pub fn accepted(action: impl Into<String>, message: impl Into<String>) -> Self {
        CommandContext::new(action).accepted_reply(message)
    }

    pub fn completed(action: impl Into<String>, message: impl Into<String>) -> Self {
        CommandContext::new(action).completed_reply(message)
    }

    pub fn failed(action: impl Into<String>, error: impl Into<String>) -> Self {
        CommandContext::new(action).failed_reply(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPublishReply {
    pub command_id: String,
    pub issued_at: String,
    pub ok: bool,
    pub action: String,
    pub status: CommandStatus,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
}

impl SnapshotPublishReply {
    pub fn completed_from_context(
        context: &CommandContext,
        generated_at: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            command_id: context.command_id.clone(),
            issued_at: context.issued_at.clone(),
            ok: true,
            action: context.action.clone(),
            status: CommandStatus::Completed,
            message: Some(message.into()),
            error: None,
            generated_at: Some(generated_at.into()),
            subject: Some(subject.into()),
        }
    }

    pub fn failed_from_context(
        context: &CommandContext,
        subject: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            command_id: context.command_id.clone(),
            issued_at: context.issued_at.clone(),
            ok: false,
            action: context.action.clone(),
            status: CommandStatus::Failed,
            message: None,
            error: Some(error.into()),
            generated_at: None,
            subject: Some(subject.into()),
        }
    }
}
