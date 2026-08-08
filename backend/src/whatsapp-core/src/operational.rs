//! Mapping from private typed host observations to WhatsApp-owned operational
//! events. This module does not create Communications evidence or touch
//! persistence.

use makosh_whatsapp_api::{
    WhatsAppMedia, WhatsAppProviderEvent, WhatsAppRuntimeState,
    host_bridge::{
        WhatsAppHostBridgeEnvelopeV1, WhatsAppHostBridgeError, WhatsAppHostObservationV1,
        validate_host_bridge_envelope,
    },
    validate_event,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhatsAppOperationalProjectionV1 {
    Event {
        provider_event_id: String,
        event: WhatsAppProviderEvent,
    },
    ResyncState {
        provider_event_id: String,
        account_id: String,
        observed_at_unix_seconds: i64,
        complete: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppOperationalProjectionError {
    HostBridge(WhatsAppHostBridgeError),
    InvalidEvent,
}

pub fn project_operational_host_observation(
    envelope: &WhatsAppHostBridgeEnvelopeV1,
) -> Result<Option<WhatsAppOperationalProjectionV1>, WhatsAppOperationalProjectionError> {
    validate_host_bridge_envelope(envelope)
        .map_err(WhatsAppOperationalProjectionError::HostBridge)?;
    let observed_at_unix_seconds = envelope.observed_at_unix_seconds;
    let account_id = envelope.account_id.clone();
    let event = match &envelope.observation {
        WhatsAppHostObservationV1::RuntimeState { state } => {
            Some(WhatsAppProviderEvent::RuntimeStateChanged {
                account_id,
                state: parse_runtime_state(state)
                    .ok_or(WhatsAppOperationalProjectionError::InvalidEvent)?,
                observed_at_unix_seconds,
            })
        }
        WhatsAppHostObservationV1::MessageUpdated {
            provider_chat_id,
            provider_message_id,
        } => Some(WhatsAppProviderEvent::MessageEdited {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            text: None,
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id,
            provider_message_id,
        } => Some(WhatsAppProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::Receipt {
            provider_chat_id,
            provider_message_id,
            delivery_state,
        } => Some(WhatsAppProviderEvent::ReceiptChanged {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            delivery_state: delivery_state.clone(),
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::Reaction {
            provider_chat_id,
            provider_message_id,
            actor_id,
            emoji,
            is_active,
        } => Some(WhatsAppProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            actor_id: actor_id.clone(),
            emoji: emoji.clone(),
            is_active: *is_active,
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::Presence {
            provider_chat_id,
            provider_identity_id,
            state,
        } => Some(WhatsAppProviderEvent::PresenceChanged {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_identity_id: provider_identity_id.clone(),
            state: state.clone(),
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::MediaMetadata {
            provider_chat_id,
            provider_message_id,
            provider_media_id,
            media_kind,
            filename,
            content_type,
            declared_size,
        } => Some(WhatsAppProviderEvent::MediaObserved(WhatsAppMedia {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            provider_media_id: provider_media_id.clone(),
            media_kind: media_kind.clone(),
            filename: filename.clone(),
            content_type: content_type.clone(),
            declared_size: *declared_size,
            observed_at_unix_seconds,
        })),
        WhatsAppHostObservationV1::CallMetadata {
            provider_call_id,
            provider_chat_id,
            direction,
            state,
        } => Some(WhatsAppProviderEvent::CallObserved {
            account_id,
            provider_call_id: provider_call_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            direction: direction.clone(),
            state: state.clone(),
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::StatusMetadata {
            provider_status_id,
            sender_id,
        } => Some(WhatsAppProviderEvent::StatusObserved {
            account_id,
            provider_status_id: provider_status_id.clone(),
            sender_id: sender_id.clone(),
            text: None,
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::StatusViewMetadata {
            provider_status_id,
            viewer_id,
        } => Some(WhatsAppProviderEvent::StatusViewObserved {
            account_id,
            provider_status_id: provider_status_id.clone(),
            viewer_id: viewer_id.clone(),
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::StatusDeletedMetadata { provider_status_id } => {
            Some(WhatsAppProviderEvent::StatusDeleted {
                account_id,
                provider_status_id: provider_status_id.clone(),
                observed_at_unix_seconds,
            })
        }
        WhatsAppHostObservationV1::OperationalMessage(value) => {
            Some(WhatsAppProviderEvent::MessageObserved(value.clone()))
        }
        WhatsAppHostObservationV1::OperationalDialog(value) => {
            Some(WhatsAppProviderEvent::DialogObserved(value.clone()))
        }
        WhatsAppHostObservationV1::OperationalParticipant(value) => {
            Some(WhatsAppProviderEvent::ParticipantObserved(value.clone()))
        }
        WhatsAppHostObservationV1::OperationalParticipantRemoved {
            provider_chat_id,
            provider_identity_id,
        } => Some(WhatsAppProviderEvent::ParticipantRemoved {
            account_id,
            provider_chat_id: provider_chat_id.clone(),
            provider_identity_id: provider_identity_id.clone(),
            observed_at_unix_seconds,
        }),
        WhatsAppHostObservationV1::OperationalResyncState { complete } => {
            return Ok(Some(WhatsAppOperationalProjectionV1::ResyncState {
                provider_event_id: envelope.provider_event_id.clone(),
                account_id,
                observed_at_unix_seconds,
                complete: *complete,
            }));
        }
        WhatsAppHostObservationV1::MessageIdentity { .. }
        | WhatsAppHostObservationV1::Dialog { .. }
        | WhatsAppHostObservationV1::Participant { .. }
        | WhatsAppHostObservationV1::SessionLinked { .. }
        | WhatsAppHostObservationV1::SessionRevoked
        | WhatsAppHostObservationV1::CommandResult { .. } => None,
    };
    let Some(event) = event else {
        return Ok(None);
    };
    validate_event(&event).map_err(|_| WhatsAppOperationalProjectionError::InvalidEvent)?;
    Ok(Some(WhatsAppOperationalProjectionV1::Event {
        provider_event_id: envelope.provider_event_id.clone(),
        event,
    }))
}

fn parse_runtime_state(value: &str) -> Option<WhatsAppRuntimeState> {
    match value {
        "stopped" => Some(WhatsAppRuntimeState::Stopped),
        "starting" => Some(WhatsAppRuntimeState::Starting),
        "running" => Some(WhatsAppRuntimeState::Running),
        "degraded" => Some(WhatsAppRuntimeState::Degraded),
        "blocked" => Some(WhatsAppRuntimeState::Blocked),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use makosh_whatsapp_api::{
        WhatsAppMessage,
        host_bridge::{HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION},
    };

    use super::*;

    #[test]
    fn full_message_becomes_provider_owned_operational_event() {
        let envelope = WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "account-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_700_000_001,
            observation: WhatsAppHostObservationV1::OperationalMessage(WhatsAppMessage {
                account_id: "account-1".to_owned(),
                provider_chat_id: "chat-1".to_owned(),
                provider_message_id: "message-1".to_owned(),
                sender_id: "sender-1".to_owned(),
                sender_display_name: "Sender".to_owned(),
                text: Some("body".to_owned()),
                reply_to_provider_message_id: None,
                occurred_at_unix_seconds: 1_700_000_000,
                delivery_state: Some("delivered".to_owned()),
            }),
        };
        assert!(matches!(
            project_operational_host_observation(&envelope),
            Ok(Some(WhatsAppOperationalProjectionV1::Event {
                event: WhatsAppProviderEvent::MessageObserved(_),
                ..
            }))
        ));
    }

    #[test]
    fn metadata_only_message_does_not_invent_operational_content() {
        let envelope = WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "account-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_700_000_001,
            observation: WhatsAppHostObservationV1::MessageIdentity {
                provider_chat_id: "chat-1".to_owned(),
                provider_message_id: "message-1".to_owned(),
                sender_id: "sender-1".to_owned(),
            },
        };
        assert_eq!(project_operational_host_observation(&envelope), Ok(None));
    }
}
