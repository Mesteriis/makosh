use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::{Postgres, Transaction};

use super::CommunicationProviderCommand;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

pub(super) const EVENT_REQUESTED: &str = "communication.provider_command.requested.v1";
pub(super) const EVENT_EXECUTING: &str = "communication.provider_command.executing.v1";
pub(super) const EVENT_COMPLETED: &str = "communication.provider_command.completed.v1";
pub(super) const EVENT_FAILED: &str = "communication.provider_command.failed.v1";
pub(super) const EVENT_RETRY_REQUESTED: &str = "communication.provider_command.retry_requested.v1";

pub(super) async fn append_provider_command_event(
    transaction: &mut Transaction<'_, Postgres>,
    event_type: &'static str,
    command: &CommunicationProviderCommand,
    occurred_at: DateTime<Utc>,
) -> Result<(), EventStoreError> {
    let event_id = format!(
        "communication_provider_command:{}:{}:{}:{}:{}:{}",
        command.command_id,
        event_type,
        command.status,
        command.retry_count,
        command.reconciliation_status,
        occurred_at.timestamp_micros()
    );
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM event_log WHERE event_id = $1)")
            .bind(&event_id)
            .fetch_one(&mut **transaction)
            .await?;
    if exists {
        return Ok(());
    }
    let event = NewEventEnvelope::builder(
        event_id,
        event_type,
        occurred_at,
        json!({ "kind": "communication_provider_command_store" }),
        json!({
            "kind": "communication_provider_command",
            "id": command.command_id,
            "account_id": command.account_id,
        }),
    )
    .actor(json!({ "actor_id": command.actor_id }))
    .payload(json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "channel_kind": command.channel_kind,
        "command_kind": command.command_kind,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "capability_state": command.capability_state,
        "action_class": command.action_class,
        "confirmation_decision": command.confirmation_decision,
        "reconciliation_status": command.reconciliation_status,
        "next_attempt_at": command.next_attempt_at,
        "dead_lettered_at": command.dead_lettered_at,
    }))
    .provenance(json!({
        "source_kind": "local_command_store",
        "source_id": command.command_id,
    }))
    .causation_id(command.command_id.clone())
    .correlation_id(command.command_id.clone())
    .build()?;
    EventStore::append_in_transaction(transaction, &event).await?;
    Ok(())
}

pub(super) async fn append_provider_command_events(
    transaction: &mut Transaction<'_, Postgres>,
    event_type: &'static str,
    commands: &[CommunicationProviderCommand],
    occurred_at: DateTime<Utc>,
) -> Result<(), EventStoreError> {
    for command in commands {
        append_provider_command_event(transaction, event_type, command, occurred_at).await?;
    }
    Ok(())
}
