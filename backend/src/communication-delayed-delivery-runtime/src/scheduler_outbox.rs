use makosh_communication_delayed_delivery_persistence::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryOutboxRecordV1,
    DelayedDeliveryOutboxStreamV1, DelayedDeliveryPersistenceErrorV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

const RELAY_BATCH_V1: u16 = 64;

pub async fn relay_scheduler_commands_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    logical_owner_id: &str,
    published_at_unix_millis: u64,
) -> Result<usize, DelayedDeliverySchedulerOutboxErrorV1> {
    let records = persistence
        .pending_scheduler_commands(logical_owner_id, RELAY_BATCH_V1)
        .await
        .map_err(DelayedDeliverySchedulerOutboxErrorV1::Persistence)?;
    relay_records(
        persistence,
        connection,
        permit,
        logical_owner_id,
        DelayedDeliveryOutboxStreamV1::SchedulerCommand,
        records,
        published_at_unix_millis,
    )
    .await
}

pub async fn relay_scheduler_receipts_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    logical_owner_id: &str,
    published_at_unix_millis: u64,
) -> Result<usize, DelayedDeliverySchedulerOutboxErrorV1> {
    let records = persistence
        .pending_scheduler_receipts(logical_owner_id, RELAY_BATCH_V1)
        .await
        .map_err(DelayedDeliverySchedulerOutboxErrorV1::Persistence)?;
    relay_records(
        persistence,
        connection,
        permit,
        logical_owner_id,
        DelayedDeliveryOutboxStreamV1::SchedulerReceipt,
        records,
        published_at_unix_millis,
    )
    .await
}

async fn relay_records(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    logical_owner_id: &str,
    stream: DelayedDeliveryOutboxStreamV1,
    records: Vec<DelayedDeliveryOutboxRecordV1>,
    published_at_unix_millis: u64,
) -> Result<usize, DelayedDeliverySchedulerOutboxErrorV1> {
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, &record.message.envelope_bytes)
            .await
            .map_err(|_| DelayedDeliverySchedulerOutboxErrorV1::EventUnavailable)?;
        persistence
            .mark_scheduler_message_published(
                stream,
                logical_owner_id,
                &record.message.message_id,
                &record.message.envelope_sha256,
                published_at_unix_millis,
            )
            .await
            .map_err(DelayedDeliverySchedulerOutboxErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliverySchedulerOutboxErrorV1 {
    Persistence(DelayedDeliveryPersistenceErrorV1),
    EventUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_batch_is_bounded() {
        assert_eq!(RELAY_BATCH_V1, 64);
    }
}
