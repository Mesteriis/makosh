use chrono::{DateTime, Utc};
use makosh_events_api::{NewEventEnvelope, StoredEventEnvelope};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::integrations::whatsapp::client::models::WhatsappWebDeliveryState;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage, NewWhatsappWebMessageDelete,
    NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant, NewWhatsappWebReaction,
    NewWhatsappWebReceipt, NewWhatsappWebStatus,
};
use crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::whatsapp_event_types;
use makosh_communications_api::evidence::StoredRawCommunicationRecord;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_events_postgres::store::EventStore;

pub(crate) const WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER: &str =
    "whatsapp_provider_observation_reconciliation";

pub(crate) async fn reconcile_whatsapp_provider_observation_event(
    pool: PgPool,
    event_bus: InMemoryEventBus,
    event: StoredEventEnvelope,
) -> Result<(), String> {
    if !supports_whatsapp_provider_reconciliation_event(&event.event.event_type) {
        return Ok(());
    }

    let account_store = CommunicationProviderAccountStore::new(pool.clone());
    let raw_record_id = required_subject_str(&event.event.subject, "raw_record_id")?;
    let raw_record =
        makosh_communications_postgres::store::CommunicationIngestionStore::new(pool.clone())
            .raw_record(raw_record_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("WhatsApp raw record `{raw_record_id}` not found"))?;

    let Some(account) = account_store
        .get(&raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    if !account.provider_kind.is_whatsapp() {
        return Ok(());
    }

    let runtime =
        crate::application::provider_runtime_factories::whatsapp_provider_runtime(pool.clone());
    let commands = match event.event.event_type.as_str() {
        "signal.accepted.whatsapp.message" => {
            runtime
                .reconcile_fixture_message_commands(&raw_record_to_whatsapp_message(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.reaction" => {
            runtime
                .reconcile_fixture_reaction_commands(&raw_record_to_whatsapp_reaction(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.receipt" => {
            runtime
                .reconcile_fixture_receipt_commands(&raw_record_to_whatsapp_receipt(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.media" => {
            runtime
                .reconcile_fixture_media_commands(&raw_record_to_whatsapp_media(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.status" => {
            runtime
                .reconcile_fixture_status_commands(&raw_record_to_whatsapp_status(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.dialog" => {
            runtime
                .reconcile_fixture_dialog_commands(&raw_record_to_whatsapp_dialog(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.participant" => {
            runtime
                .reconcile_fixture_participant_commands(&raw_record_to_whatsapp_participant(
                    &raw_record,
                )?)
                .await
        }
        "signal.accepted.whatsapp.message_update" => {
            runtime
                .reconcile_fixture_message_update_commands(&raw_record_to_whatsapp_message_update(
                    &raw_record,
                )?)
                .await
        }
        "signal.accepted.whatsapp.message_delete" => {
            runtime
                .reconcile_fixture_message_delete_commands(&raw_record_to_whatsapp_message_delete(
                    &raw_record,
                )?)
                .await
        }
        _ => return Ok(()),
    }
    .map_err(|error| error.to_string())?;

    let event_store = EventStore::new(pool);
    for command in commands {
        publish_whatsapp_command_events(
            &event_store,
            &event_bus,
            &command,
            "provider_observation_consumer",
        )
        .await?;
    }
    Ok(())
}

fn supports_whatsapp_provider_reconciliation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.whatsapp.message"
            | "signal.accepted.whatsapp.reaction"
            | "signal.accepted.whatsapp.receipt"
            | "signal.accepted.whatsapp.media"
            | "signal.accepted.whatsapp.status"
            | "signal.accepted.whatsapp.dialog"
            | "signal.accepted.whatsapp.participant"
            | "signal.accepted.whatsapp.message_update"
            | "signal.accepted.whatsapp.message_delete"
    )
}

async fn publish_whatsapp_command_events(
    event_store: &EventStore,
    event_bus: &InMemoryEventBus,
    command: &WhatsAppProviderCommand,
    source: &str,
) -> Result<(), String> {
    let payload = json!({
        "account_id": command.account_id,
        "command_id": command.command_id,
        "idempotency_key": command.idempotency_key,
        "command_kind": command.command_kind,
        "action": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "provider_message_id": command.provider_message_id,
        "capability_state": command.capability_state,
        "action_class": command.action_class,
        "confirmation_decision": command.confirmation_decision,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "audit_metadata": command.audit_metadata,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "source": source,
    });
    publish_whatsapp_command_event(
        event_store,
        event_bus,
        whatsapp_event_types::COMMAND_STATUS_CHANGED,
        command,
        payload.clone(),
    )
    .await?;
    publish_whatsapp_command_event(
        event_store,
        event_bus,
        whatsapp_event_types::COMMAND_RECONCILED,
        command,
        payload,
    )
    .await?;
    Ok(())
}

async fn publish_whatsapp_command_event(
    event_store: &EventStore,
    event_bus: &InMemoryEventBus,
    event_type: &str,
    command: &WhatsAppProviderCommand,
    payload: Value,
) -> Result<(), String> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}:{}",
        command.command_id,
        command.command_kind,
        command.status,
        event_type,
        now.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        format!(
            "evt_whatsapp_command_{}_{}_{}",
            event_type.replace('.', "_"),
            command.command_id,
            Uuid::now_v7()
        ),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": command.account_id,
            "actor_id": "makosh-frontend",
            "kind": "whatsapp_provider_commands",
            "source_id": source_id,
        }),
        json!({
            "id": command.command_id,
            "entity_id": command.command_id,
            "kind": "whatsapp_provider_command",
        }),
    )
    .payload(payload)
    .build()
    .map_err(|error| error.to_string())?;
    event_store
        .append(&event)
        .await
        .map_err(|error| error.to_string())?;
    let _ = event_bus.broadcast(event);
    Ok(())
}

fn raw_record_to_whatsapp_message(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebMessage, String> {
    Ok(NewWhatsappWebMessage {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: raw_record.provider_record_id.clone(),
        chat_title: required_payload_str(&raw_record.payload, "chat_title")?.to_owned(),
        sender_id: required_payload_str(&raw_record.payload, "sender_id")?.to_owned(),
        sender_display_name: required_payload_str(&raw_record.payload, "sender_display_name")?
            .to_owned(),
        text: required_payload_str(&raw_record.payload, "text")?.to_owned(),
        reply_to_provider_message_id: optional_payload_str(
            &raw_record.payload,
            "reply_to_provider_message_id",
        )
        .map(str::to_owned),
        forward_origin_chat_id: optional_payload_str(&raw_record.payload, "forward_origin_chat_id")
            .map(str::to_owned),
        forward_origin_message_id: optional_payload_str(
            &raw_record.payload,
            "forward_origin_message_id",
        )
        .map(str::to_owned),
        forward_origin_sender_id: optional_payload_str(
            &raw_record.payload,
            "forward_origin_sender_id",
        )
        .map(str::to_owned),
        forward_origin_sender_name: optional_payload_str(
            &raw_record.payload,
            "forward_origin_sender_name",
        )
        .map(str::to_owned),
        forwarded_at: optional_payload_datetime(&raw_record.payload, "forwarded_at")?,
        message_metadata: raw_record
            .payload
            .get("message_metadata")
            .cloned()
            .unwrap_or_else(|| json!({})),
        import_batch_id: raw_record.import_batch_id.clone(),
        occurred_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
        delivery_state: parse_delivery_state(required_payload_str(
            &raw_record.payload,
            "delivery_state",
        )?)?,
    })
}

fn raw_record_to_whatsapp_reaction(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebReaction, String> {
    Ok(NewWhatsappWebReaction {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: required_payload_str(&raw_record.payload, "provider_message_id")?
            .to_owned(),
        provider_actor_id: required_payload_str(&raw_record.payload, "provider_actor_id")?
            .to_owned(),
        sender_display_name: required_payload_str(&raw_record.payload, "sender_display_name")?
            .to_owned(),
        reaction: required_payload_str(&raw_record.payload, "reaction")?.to_owned(),
        is_active: required_payload_bool(&raw_record.payload, "is_active")?,
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_receipt(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebReceipt, String> {
    Ok(NewWhatsappWebReceipt {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: raw_record.provider_record_id.clone(),
        delivery_state: required_delivery_state(&raw_record.payload)?,
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: required_datetime(
            optional_payload_datetime(&raw_record.payload, "observed_at")?,
            "observed_at",
        )?,
    })
}

fn raw_record_to_whatsapp_media(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebMedia, String> {
    Ok(NewWhatsappWebMedia {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: required_payload_str(&raw_record.payload, "provider_message_id")?
            .to_owned(),
        provider_attachment_id: required_payload_str(
            &raw_record.payload,
            "provider_attachment_id",
        )?
        .to_owned(),
        filename: optional_payload_str(&raw_record.payload, "filename").map(str::to_owned),
        content_type: required_payload_str(&raw_record.payload, "content_type")?.to_owned(),
        size_bytes: required_payload_i64(&raw_record.payload, "size_bytes")?,
        sha256: required_payload_str(&raw_record.payload, "sha256")?.to_owned(),
        storage_kind: required_payload_str(&raw_record.payload, "storage_kind")?.to_owned(),
        storage_path: required_payload_str(&raw_record.payload, "storage_path")?.to_owned(),
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_status(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebStatus, String> {
    Ok(NewWhatsappWebStatus {
        account_id: raw_record.account_id.clone(),
        provider_status_id: raw_record.provider_record_id.clone(),
        sender_id: required_payload_str(&raw_record.payload, "sender_id")?.to_owned(),
        sender_display_name: required_payload_str(&raw_record.payload, "sender_display_name")?
            .to_owned(),
        sender_identity_kind: optional_payload_str(&raw_record.payload, "sender_identity_kind")
            .map(str::to_owned),
        sender_address: optional_payload_str(&raw_record.payload, "sender_address")
            .map(str::to_owned),
        sender_push_name: optional_payload_str(&raw_record.payload, "sender_push_name")
            .map(str::to_owned),
        sender_business_profile: raw_record
            .payload
            .get("sender_business_profile")
            .cloned()
            .unwrap_or_else(|| json!({})),
        sender_profile_photo_ref: raw_record
            .payload
            .get("sender_profile_photo_ref")
            .cloned()
            .unwrap_or_else(|| json!({})),
        text: required_payload_str(&raw_record.payload, "text")?.to_owned(),
        import_batch_id: raw_record.import_batch_id.clone(),
        occurred_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_dialog(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebDialog, String> {
    Ok(NewWhatsappWebDialog {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: raw_record.provider_record_id.clone(),
        chat_title: required_payload_str(&raw_record.payload, "chat_title")?.to_owned(),
        chat_kind: required_payload_str(&raw_record.payload, "chat_kind")?.to_owned(),
        is_archived: optional_payload_bool(&raw_record.payload, "is_archived"),
        is_pinned: optional_payload_bool(&raw_record.payload, "is_pinned"),
        is_muted: optional_payload_bool(&raw_record.payload, "is_muted"),
        is_unread: optional_payload_bool(&raw_record.payload, "is_unread"),
        unread_count: optional_payload_i64(&raw_record.payload, "unread_count"),
        participant_count: optional_payload_i64(&raw_record.payload, "participant_count"),
        community_parent_chat_id: optional_payload_str(
            &raw_record.payload,
            "community_parent_chat_id",
        )
        .map(str::to_owned),
        community_parent_title: optional_payload_str(&raw_record.payload, "community_parent_title")
            .map(str::to_owned),
        invite_link: optional_payload_str(&raw_record.payload, "invite_link").map(str::to_owned),
        is_community_root: optional_payload_bool(&raw_record.payload, "is_community_root"),
        is_broadcast: optional_payload_bool(&raw_record.payload, "is_broadcast"),
        is_newsletter: optional_payload_bool(&raw_record.payload, "is_newsletter"),
        avatar_metadata: raw_record
            .payload
            .get("avatar_metadata")
            .cloned()
            .unwrap_or_else(|| json!({})),
        provider_labels: raw_record
            .payload
            .get("provider_labels")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_participant(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebParticipant, String> {
    Ok(NewWhatsappWebParticipant {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        chat_title: required_payload_str(&raw_record.payload, "chat_title")?.to_owned(),
        chat_kind: required_payload_str(&raw_record.payload, "chat_kind")?.to_owned(),
        provider_member_id: required_payload_str(&raw_record.payload, "provider_member_id")?
            .to_owned(),
        provider_identity_id: required_payload_str(&raw_record.payload, "provider_identity_id")?
            .to_owned(),
        identity_kind: required_payload_str(&raw_record.payload, "identity_kind")?.to_owned(),
        display_name: required_payload_str(&raw_record.payload, "display_name")?.to_owned(),
        push_name: optional_payload_str(&raw_record.payload, "push_name").map(str::to_owned),
        address: optional_payload_str(&raw_record.payload, "address").map(str::to_owned),
        business_profile: raw_record
            .payload
            .get("business_profile")
            .cloned()
            .unwrap_or_else(|| json!({})),
        profile_photo_ref: raw_record
            .payload
            .get("profile_photo_ref")
            .cloned()
            .unwrap_or_else(|| json!({})),
        role: required_payload_str(&raw_record.payload, "role")?.to_owned(),
        status: required_payload_str(&raw_record.payload, "status")?.to_owned(),
        is_self: required_payload_bool(&raw_record.payload, "is_self")?,
        is_admin: required_payload_bool(&raw_record.payload, "is_admin")?,
        is_owner: required_payload_bool(&raw_record.payload, "is_owner")?,
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_message_update(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebMessageUpdate, String> {
    Ok(NewWhatsappWebMessageUpdate {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: raw_record.provider_record_id.clone(),
        text: required_payload_str(&raw_record.payload, "text")?.to_owned(),
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_message_delete(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebMessageDelete, String> {
    Ok(NewWhatsappWebMessageDelete {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: raw_record.provider_record_id.clone(),
        reason_class: required_payload_str(&raw_record.payload, "reason_class")?.to_owned(),
        actor_class: required_payload_str(&raw_record.payload, "actor_class")?.to_owned(),
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn parse_delivery_state(value: &str) -> Result<WhatsappWebDeliveryState, String> {
    WhatsappWebDeliveryState::try_from(value.to_owned()).map_err(|error| error.to_string())
}

fn required_subject_str<'a>(subject: &'a Value, field: &str) -> Result<&'a str, String> {
    subject
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("event subject field `{field}` is required"))
}

fn required_payload_str<'a>(payload: &'a Value, field: &str) -> Result<&'a str, String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("raw payload field `{field}` is required"))
}

fn optional_payload_str<'a>(payload: &'a Value, field: &str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn required_payload_bool(payload: &Value, field: &str) -> Result<bool, String> {
    payload
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("raw payload boolean field `{field}` is required"))
}

fn optional_payload_bool(payload: &Value, field: &str) -> Option<bool> {
    payload.get(field).and_then(Value::as_bool)
}

fn required_payload_i64(payload: &Value, field: &str) -> Result<i64, String> {
    payload
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("raw payload integer field `{field}` is required"))
}

fn optional_payload_i64(payload: &Value, field: &str) -> Option<i64> {
    payload.get(field).and_then(Value::as_i64)
}

fn required_delivery_state(payload: &Value) -> Result<WhatsappWebDeliveryState, String> {
    parse_delivery_state(required_payload_str(payload, "delivery_state")?)
}

fn required_datetime(value: Option<DateTime<Utc>>, field: &str) -> Result<DateTime<Utc>, String> {
    value.ok_or_else(|| format!("raw payload datetime field `{field}` is required"))
}

fn optional_payload_datetime(
    payload: &Value,
    field: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = optional_payload_str(payload, field) else {
        return Ok(None);
    };
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map(Some)
        .map_err(|error| format!("raw payload datetime field `{field}` is invalid: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_whatsapp_provider_reconciliation_event_filters_expected_types() {
        assert!(supports_whatsapp_provider_reconciliation_event(
            "signal.accepted.whatsapp.message"
        ));
        assert!(supports_whatsapp_provider_reconciliation_event(
            "signal.accepted.whatsapp.participant"
        ));
        assert!(supports_whatsapp_provider_reconciliation_event(
            "signal.accepted.whatsapp.receipt"
        ));
        assert!(!supports_whatsapp_provider_reconciliation_event(
            "signal.accepted.telegram.message"
        ));
    }

    #[test]
    fn raw_record_to_whatsapp_message_restores_command_reconcile_shape() {
        let raw_record = StoredRawCommunicationRecord {
            raw_record_id: "raw".to_owned(),
            observation_id: "obs".to_owned(),
            account_id: "acct".to_owned(),
            record_kind: "whatsapp_web_message".to_owned(),
            provider_record_id: "msg-1".to_owned(),
            source_fingerprint: "fp".to_owned(),
            import_batch_id: "batch".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "provider_chat_id": "chat-1",
                "chat_title": "Chat",
                "sender_id": "sender-1",
                "sender_display_name": "Sender",
                "text": "hello",
                "delivery_state": "received",
                "message_metadata": {"kind": "text"}
            }),
            provenance: json!({}),
        };

        let message = raw_record_to_whatsapp_message(&raw_record).expect("message dto");
        assert_eq!(message.account_id, "acct");
        assert_eq!(message.provider_chat_id, "chat-1");
        assert_eq!(message.provider_message_id, "msg-1");
        assert_eq!(message.delivery_state.as_str(), "received");
        assert_eq!(message.message_metadata["kind"], json!("text"));
    }

    #[test]
    fn raw_record_to_whatsapp_receipt_restores_command_reconcile_shape() {
        let observed_at = Utc::now();
        let raw_record = StoredRawCommunicationRecord {
            raw_record_id: "raw".to_owned(),
            observation_id: "obs".to_owned(),
            account_id: "acct".to_owned(),
            record_kind: "whatsapp_web_receipt".to_owned(),
            provider_record_id: "msg-1".to_owned(),
            source_fingerprint: "fp".to_owned(),
            import_batch_id: "batch".to_owned(),
            occurred_at: Some(observed_at),
            captured_at: Utc::now(),
            payload: json!({
                "provider_chat_id": "chat-1",
                "delivery_state": "sent",
                "observed_at": observed_at,
            }),
            provenance: json!({}),
        };

        let receipt = raw_record_to_whatsapp_receipt(&raw_record).expect("receipt dto");
        assert_eq!(receipt.account_id, "acct");
        assert_eq!(receipt.provider_chat_id, "chat-1");
        assert_eq!(receipt.provider_message_id, "msg-1");
        assert_eq!(receipt.delivery_state.as_str(), "sent");
    }
}
