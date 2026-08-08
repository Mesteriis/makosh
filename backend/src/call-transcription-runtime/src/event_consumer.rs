use std::os::unix::net::UnixStream;

use makosh_call_transcription_persistence::CallTranscriptionPersistenceV1;
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};

use crate::ingress::{
    CallTranscriptionIngressErrorV1, apply_recording_ready_v1, apply_recording_rejected_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionEventConsumerErrorV1 {
    InvalidEnvelope,
    Ingress(CallTranscriptionIngressErrorV1),
    EventUnavailable,
}

#[allow(clippy::too_many_arguments)]
pub async fn consume_recording_ready_once_v1(
    persistence: &CallTranscriptionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, CallTranscriptionEventConsumerErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CallTranscriptionEventConsumerErrorV1::InvalidEnvelope)?;
    apply_recording_ready_v1(
        persistence,
        channel,
        dispatcher,
        &record,
        logical_owner_id,
        occurred_at_unix_millis,
    )
    .await
    .map_err(CallTranscriptionEventConsumerErrorV1::Ingress)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_recording_rejected_once_v1(
    persistence: &CallTranscriptionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, CallTranscriptionEventConsumerErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CallTranscriptionEventConsumerErrorV1::InvalidEnvelope)?;
    apply_recording_rejected_v1(
        persistence,
        &record,
        logical_owner_id,
        occurred_at_unix_millis,
    )
    .await
    .map_err(CallTranscriptionEventConsumerErrorV1::Ingress)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> CallTranscriptionEventConsumerErrorV1 {
    CallTranscriptionEventConsumerErrorV1::EventUnavailable
}
