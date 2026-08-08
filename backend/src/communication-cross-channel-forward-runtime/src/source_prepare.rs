use makosh_communication_cross_channel_forward_api::COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1;
use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
};
use makosh_communications_cross_channel_forward_source_api::{
    CrossChannelForwardSourceEnvelopeContextV1,
    build_cross_channel_forward_source_prepare_outbox_record_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardSourcePrepareErrorV1 {
    InvalidContext,
    Persistence(CrossChannelForwardPersistenceErrorV1),
}

pub async fn enqueue_source_prepare_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    logical_owner_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    now_unix_millis: i64,
) -> Result<bool, CrossChannelForwardSourcePrepareErrorV1> {
    let (seconds, nanos) = timestamp(now_unix_millis)?;
    let Some(candidate) = persistence
        .next_source_prepare_candidate(logical_owner_id)
        .await
        .map_err(CrossChannelForwardSourcePrepareErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let deadline = seconds
        .checked_add(300)
        .ok_or(CrossChannelForwardSourcePrepareErrorV1::InvalidContext)?;
    let record = build_cross_channel_forward_source_prepare_outbox_record_v1(
        candidate.forward_id,
        candidate.source_message_id,
        candidate.target_conversation_id,
        logical_owner_id,
        deadline,
        &CrossChannelForwardSourceEnvelopeContextV1 {
            module_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_instance_id.to_owned(),
            runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .map_err(|_| CrossChannelForwardSourcePrepareErrorV1::InvalidContext)?;
    persistence
        .persist_source_prepare_outbox(
            logical_owner_id,
            candidate.forward_id,
            &record,
            now_unix_millis,
        )
        .await
        .map_err(CrossChannelForwardSourcePrepareErrorV1::Persistence)?;
    Ok(true)
}

fn timestamp(unix_millis: i64) -> Result<(i64, i32), CrossChannelForwardSourcePrepareErrorV1> {
    if unix_millis <= 0 {
        return Err(CrossChannelForwardSourcePrepareErrorV1::InvalidContext);
    }
    let seconds = unix_millis / 1_000;
    let nanos = i32::try_from((unix_millis % 1_000) * 1_000_000)
        .map_err(|_| CrossChannelForwardSourcePrepareErrorV1::InvalidContext)?;
    Ok((seconds, nanos))
}

#[cfg(test)]
mod tests {
    use super::timestamp;

    #[test]
    fn timestamp_preserves_millisecond_precision() {
        assert_eq!(
            timestamp(1_750_000_123).expect("timestamp"),
            (1_750_000, 123_000_000)
        );
    }
}
