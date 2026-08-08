use makosh_contacts_persistence::{ContactsPersistenceErrorV1, ContactsPersistenceV1};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContactsEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(ContactsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_contacts_outbox_once_v1(
    persistence: &ContactsPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, ContactsEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(ContactsEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .load_pending_outbox(logical_owner_id)
        .await
        .map_err(ContactsEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| ContactsEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_outbox_published(
            logical_owner_id,
            record.message_id,
            published_at_unix_millis,
        )
        .await
        .map_err(ContactsEventRelayErrorV1::Persistence)?;
    Ok(true)
}
