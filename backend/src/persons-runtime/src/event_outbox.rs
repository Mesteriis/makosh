use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_persons_persistence::{PersonsPersistenceErrorV1, PersonsPersistenceV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(PersonsPersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_persons_outbox_once_v1(
    persistence: &PersonsPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, PersonsEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(PersonsEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(row) = persistence
        .load_pending_outbox(logical_owner_id)
        .await
        .map_err(PersonsEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &row.record.envelope_bytes)
        .await
        .map_err(|_| PersonsEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_outbox_published(
            logical_owner_id,
            row.record.message_id,
            row.record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(PersonsEventRelayErrorV1::Persistence)?;
    Ok(true)
}
