use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_tasks_persistence::{TasksPersistenceErrorV1, TasksPersistenceV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TasksEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(TasksPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_tasks_outbox_once_v1(
    persistence: &TasksPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, TasksEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(TasksEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(claim) = persistence
        .claim_next_pending_outbox(logical_owner_id)
        .await
        .map_err(TasksEventRelayErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let record = claim.record().clone();
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| TasksEventRelayErrorV1::EventUnavailable)?;
    claim
        .mark_published(record.envelope_sha256, published_at_unix_millis)
        .await
        .map_err(TasksEventRelayErrorV1::Persistence)?;
    Ok(true)
}
