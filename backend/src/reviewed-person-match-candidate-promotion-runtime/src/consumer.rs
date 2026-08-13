use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_reviewed_person_match_candidate_promotion_persistence::ReviewedPersonMatchCandidatePromotionPersistenceV1;

use crate::{
    ReviewedPersonMatchCandidatePromotionExecutionContextV1,
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1,
    process_person_match_candidate_approval_v1, process_persons_terminal_v1,
};

pub async fn consume_person_match_candidate_approval_once_v1(
    persistence: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    consume(connection, permit, |record| async move {
        process_person_match_candidate_approval_v1(persistence, &record, context)
            .await
            .map(|_| ())
    })
    .await
}

pub async fn consume_persons_succeeded_terminal_once_v1(
    persistence: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    consume(connection, permit, |record| async move {
        process_persons_terminal_v1(persistence, &record, context)
            .await
            .map(|_| ())
    })
    .await
}

pub async fn consume_persons_rejected_terminal_once_v1(
    persistence: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    consume_persons_succeeded_terminal_once_v1(persistence, connection, permit, context).await
}

async fn consume<F, Fut>(
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    process: F,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionExecutionErrorV1>
where
    F: FnOnce(OutboxRecordV1) -> Fut,
    Fut: std::future::Future<
            Output = Result<(), ReviewedPersonMatchCandidatePromotionExecutionErrorV1>,
        >,
{
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    process(record).await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

const fn event_error(
    _: RuntimePullDeliveryErrorV1,
) -> ReviewedPersonMatchCandidatePromotionExecutionErrorV1 {
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1::EventUnavailable
}
