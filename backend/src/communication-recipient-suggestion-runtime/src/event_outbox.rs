use makosh_communication_recipient_suggestion_persistence::{
    CommunicationRecipientSuggestionPersistenceErrorV1,
    CommunicationRecipientSuggestionPersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(CommunicationRecipientSuggestionPersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_source_prepare_outbox_once_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, CommunicationRecipientSuggestionEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(CommunicationRecipientSuggestionEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_source_prepare_events(logical_owner_id, 1)
        .await
        .map_err(CommunicationRecipientSuggestionEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| CommunicationRecipientSuggestionEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_source_prepare_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(CommunicationRecipientSuggestionEventRelayErrorV1::Persistence)?;
    Ok(true)
}
