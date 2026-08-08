use chrono::Utc;
use makosh_communications_api::commands::CommunicationProviderCommand;
use makosh_events_api::{NewEventEnvelope, StoredEventEnvelope};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::zulip_event_types;
use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_postgres::store::EventStore;

pub const ZULIP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER: &str =
    "zulip_provider_observation_reconciliation";

pub async fn reconcile_zulip_provider_observation_event(
    pool: PgPool,
    event_bus: InMemoryEventBus,
    event: StoredEventEnvelope,
) -> Result<(), String> {
    if !makosh_provider_zulip::reconciliation::supports_observation_event(&event.event.event_type) {
        return Ok(());
    }

    let raw_record_id = required_json_str(&event.event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Zulip raw record `{raw_record_id}` not found"))?;

    let Some(account) = CommunicationProviderAccountStore::new(pool.clone())
        .get(&raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    if account.provider_kind != CommunicationProviderKind::ZulipBot {
        return Ok(());
    }

    let provider_message_id = required_json_str(&raw_record.payload, "provider_message_id")?;
    let command_kinds = makosh_provider_zulip::reconciliation::command_kinds_for_observation(
        &event.event.event_type,
        &raw_record.payload,
    );
    if command_kinds.is_empty() {
        return Ok(());
    }
    let observed_at = raw_record.occurred_at.unwrap_or(event.event.occurred_at);
    let provider_state = makosh_provider_zulip::reconciliation::provider_state(
        &event.event.event_id,
        &event.event.event_type,
        &raw_record.raw_record_id,
        &raw_record.payload,
        observed_at,
    );
    let command_store = CommunicationProviderCommandStore::new(pool.clone());
    let commands = makosh_provider_orchestration::reconcile_provider_command_observation(
        &command_store,
        &raw_record.account_id,
        "zulip",
        provider_message_id,
        &command_kinds,
        observed_at,
        provider_state,
    )
    .await
    .map_err(|error| error.to_string())?;

    let event_store = EventStore::new(pool);
    for command in commands {
        publish_zulip_command_events(
            &event_store,
            &event_bus,
            &event,
            &command,
            "provider_observation_consumer",
        )
        .await?;
    }

    Ok(())
}

async fn publish_zulip_command_events(
    event_store: &EventStore,
    event_bus: &InMemoryEventBus,
    parent: &StoredEventEnvelope,
    command: &CommunicationProviderCommand,
    source: &str,
) -> Result<(), String> {
    let payload = json!({
        "account_id": command.account_id,
        "command_id": command.command_id,
        "idempotency_key": command.idempotency_key,
        "command_kind": command.command_kind,
        "provider_message_id": command.provider_message_id,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "completed_at": command.completed_at,
        "source": source,
    });
    publish_zulip_command_event(
        event_store,
        event_bus,
        parent,
        zulip_event_types::COMMAND_STATUS_CHANGED,
        command,
        payload.clone(),
    )
    .await?;
    publish_zulip_command_event(
        event_store,
        event_bus,
        parent,
        zulip_event_types::COMMAND_RECONCILED,
        command,
        payload,
    )
    .await
}

async fn publish_zulip_command_event(
    event_store: &EventStore,
    event_bus: &InMemoryEventBus,
    parent: &StoredEventEnvelope,
    event_type: &str,
    command: &CommunicationProviderCommand,
    payload: Value,
) -> Result<(), String> {
    let now = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_zulip_command_{}_{}_{}",
            event_type.replace('.', "_"),
            command.command_id,
            Uuid::now_v7()
        ),
        event_type.to_owned(),
        now,
        json!({
            "channel": "zulip",
            "account_id": command.account_id,
            "actor_id": "makosh-frontend",
            "kind": "communication_provider_commands",
            "source_id": format!("{}:{}:{}", command.command_id, event_type, now.timestamp_micros()),
        }),
        json!({
            "id": command.command_id,
            "entity_id": command.command_id,
            "kind": "communication_provider_command",
        }),
    )
    .payload(payload)
    .provenance(json!({
        "source": "zulip_provider_observation_reconciliation",
        "parent_event_id": parent.event.event_id,
    }))
    .causation_id(parent.event.event_id.clone())
    .correlation_id(parent.event.correlation_id.clone().unwrap_or_else(|| parent.event.event_id.clone()))
    .build()
    .map_err(|error| error.to_string())?;
    event_store
        .append(&event)
        .await
        .map_err(|error| error.to_string())?;
    let _ = event_bus.broadcast(event);
    Ok(())
}

fn required_json_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("field `{field}` is required"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_kinds_for_zulip_reaction_uses_reaction_op() {
        assert_eq!(
            makosh_provider_zulip::reconciliation::command_kinds_for_observation(
                "signal.accepted.zulip.reaction",
                &json!({"reaction_op": "add"}),
            ),
            vec!["add_reaction"]
        );
        assert_eq!(
            makosh_provider_zulip::reconciliation::command_kinds_for_observation(
                "signal.accepted.zulip.reaction",
                &json!({"reaction_op": "remove"}),
            ),
            vec!["remove_reaction"]
        );
        assert_eq!(
            makosh_provider_zulip::reconciliation::command_kinds_for_observation(
                "signal.accepted.zulip.reaction",
                &json!({}),
            ),
            vec!["add_reaction", "remove_reaction"]
        );
    }
}
