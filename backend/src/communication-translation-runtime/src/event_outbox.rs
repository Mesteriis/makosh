use makosh_communication_translation_persistence::{
    CommunicationTranslationPersistenceErrorV1, CommunicationTranslationPersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(CommunicationTranslationPersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_source_prepare_outbox_once_v1(
    persistence: &CommunicationTranslationPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, CommunicationTranslationEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(CommunicationTranslationEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_source_prepare_events(logical_owner_id, 1)
        .await
        .map_err(CommunicationTranslationEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| CommunicationTranslationEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_source_prepare_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(CommunicationTranslationEventRelayErrorV1::Persistence)?;
    Ok(true)
}
