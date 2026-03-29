//! `SystemCommandEvent` protobuf ↔ `CommandEvent` / `CommandReply` for NATS `system.commands.*`.

use nats_adapter::proto::v1::{SystemCommandEvent, SystemCommandStatus};

use crate::commands::{CommandEvent, CommandReply, CommandStatus};

/// Encode a domain command event for `NatsEnvelope` wrapping on NATS.
pub fn system_command_event_to_proto(e: &CommandEvent) -> SystemCommandEvent {
    SystemCommandEvent {
        command_id: e.command_id.clone(),
        issued_at: e.issued_at.clone(),
        action: e.action.clone(),
        status: match e.status {
            CommandStatus::Accepted => SystemCommandStatus::Accepted as i32,
            CommandStatus::Completed => SystemCommandStatus::Completed as i32,
            CommandStatus::Failed => SystemCommandStatus::Failed as i32,
        },
        message: e.message.clone(),
        error: e.error.clone(),
    }
}

/// Decode protobuf command event into `CommandReply` for TUI / status UI (`ok` mirrors HTTP-style success).
pub fn command_reply_from_system_command_event(p: SystemCommandEvent) -> Option<CommandReply> {
    let st = SystemCommandStatus::try_from(p.status).ok()?;
    let status = match st {
        SystemCommandStatus::Accepted => CommandStatus::Accepted,
        SystemCommandStatus::Completed => CommandStatus::Completed,
        SystemCommandStatus::Failed => CommandStatus::Failed,
        SystemCommandStatus::Unspecified => return None,
    };
    let ok = !matches!(status, CommandStatus::Failed);
    Some(CommandReply {
        command_id: p.command_id,
        issued_at: p.issued_at,
        ok,
        action: p.action,
        status,
        message: p.message,
        error: p.error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_failed_maps_ok_false() {
        let ev = CommandEvent {
            command_id: "c1".into(),
            issued_at: "t".into(),
            action: "stop".into(),
            status: CommandStatus::Failed,
            message: None,
            error: Some("boom".into()),
        };
        let pb = system_command_event_to_proto(&ev);
        let reply = command_reply_from_system_command_event(pb).expect("reply");
        assert!(!reply.ok);
        assert_eq!(reply.status, CommandStatus::Failed);
        assert_eq!(reply.error.as_deref(), Some("boom"));
    }

    #[test]
    fn round_trip_completed_maps_ok_true() {
        let ev = CommandEvent {
            command_id: "c2".into(),
            issued_at: "t2".into(),
            action: "start".into(),
            status: CommandStatus::Completed,
            message: Some("done".into()),
            error: None,
        };
        let pb = system_command_event_to_proto(&ev);
        let reply = command_reply_from_system_command_event(pb).expect("reply");
        assert!(reply.ok);
        assert_eq!(reply.message.as_deref(), Some("done"));
    }
}
