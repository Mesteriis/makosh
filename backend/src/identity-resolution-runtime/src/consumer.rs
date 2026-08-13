use crate::{
    IdentityResolutionExecutionContextV1, IdentityResolutionExecutionErrorV1,
    process_persons_identity_evidence_v1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_identity_resolution_persistence::{
    IdentityResolutionPersistenceErrorV1, IdentityResolutionPersistenceV1,
};

pub async fn consume_persons_identity_evidence_once_v1(
    persistence: &IdentityResolutionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &IdentityResolutionExecutionContextV1,
) -> Result<bool, IdentityResolutionExecutionErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    match process_persons_identity_evidence_v1(persistence, &record, context).await {
        Ok(_) => {}
        Err(e) if bounded(e) => {}
        Err(e) => return Err(e),
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}
const fn bounded(e: IdentityResolutionExecutionErrorV1) -> bool {
    matches!(
        e,
        IdentityResolutionExecutionErrorV1::InvalidEnvelope
            | IdentityResolutionExecutionErrorV1::InvalidPayload
            | IdentityResolutionExecutionErrorV1::Persistence(
                IdentityResolutionPersistenceErrorV1::InvalidInput
                    | IdentityResolutionPersistenceErrorV1::Conflict
                    | IdentityResolutionPersistenceErrorV1::RevisionConflict
            )
    )
}
const fn event_error(_: RuntimePullDeliveryErrorV1) -> IdentityResolutionExecutionErrorV1 {
    IdentityResolutionExecutionErrorV1::EventUnavailable
}
