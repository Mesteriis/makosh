use makosh_attachment_preview_evidence_replay_core::ReplayProducerV1;
use makosh_attachment_preview_evidence_replay_persistence::{
    AttachmentPreviewEvidenceReplayPersistenceV1, ReplayPersistenceErrorV1,
    ReplayResultInboxRecordV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayResultConsumerErrorV1 {
    InvalidTimestamp,
    InvalidEnvelope,
    Persistence(ReplayPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_next_communications_replay_result_v1(
    persistence: &AttachmentPreviewEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    accepted_at_unix_seconds: i64,
) -> Result<bool, ReplayResultConsumerErrorV1> {
    consume_next_result(
        persistence,
        connection,
        permit,
        ReplayProducerV1::Communications,
        accepted_at_unix_seconds,
    )
    .await
}

pub async fn consume_next_mail_replay_result_v1(
    persistence: &AttachmentPreviewEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    accepted_at_unix_seconds: i64,
) -> Result<bool, ReplayResultConsumerErrorV1> {
    consume_next_result(
        persistence,
        connection,
        permit,
        ReplayProducerV1::Mail,
        accepted_at_unix_seconds,
    )
    .await
}

async fn consume_next_result(
    persistence: &AttachmentPreviewEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    producer: ReplayProducerV1,
    accepted_at_unix_seconds: i64,
) -> Result<bool, ReplayResultConsumerErrorV1> {
    if accepted_at_unix_seconds <= 0 {
        return Err(ReplayResultConsumerErrorV1::InvalidTimestamp);
    }
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| ReplayResultConsumerErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = ReplayResultInboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReplayResultConsumerErrorV1::InvalidEnvelope)?;
    persistence
        .accept_producer_result(producer, &record, accepted_at_unix_seconds)
        .await
        .map_err(ReplayResultConsumerErrorV1::Persistence)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| ReplayResultConsumerErrorV1::EventUnavailable)?;
    Ok(true)
}
