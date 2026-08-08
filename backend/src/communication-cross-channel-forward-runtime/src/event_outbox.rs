use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(CrossChannelForwardPersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_event_outbox_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, CrossChannelForwardEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(CrossChannelForwardEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .pending_event_outbox(1)
        .await
        .map_err(CrossChannelForwardEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, record.exact_bytes())
        .await
        .map_err(|_| CrossChannelForwardEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_event_outbox_published(*record.message_id(), published_at_unix_millis)
        .await
        .map_err(CrossChannelForwardEventRelayErrorV1::Persistence)?;
    Ok(true)
}
