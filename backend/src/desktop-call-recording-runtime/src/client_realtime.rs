use std::os::unix::net::UnixStream;

use makosh_desktop_call_recording_api::{REALTIME_CONTRACT_NAME_V1, contract_reference_v1};
use makosh_desktop_call_recording_persistence::{
    DesktopCallRecordingRepositoryV1, PersistenceErrorV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeClientRealtimePublishRequestV1, ManagedRuntimeControlRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::client_realtime::{
        validate_managed_client_realtime_publish_request_v1,
        validate_managed_client_realtime_publish_response_v1,
    },
};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopRecordingRealtimeErrorV1 {
    Persistence(PersistenceErrorV1),
    InvalidTransition,
    Unavailable,
}

pub async fn publish_pending_realtime_v1(
    persistence: &DesktopCallRecordingRepositoryV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    published_at_unix_ms: i64,
) -> Result<bool, DesktopRecordingRealtimeErrorV1> {
    if published_at_unix_ms <= 0 {
        return Err(DesktopRecordingRealtimeErrorV1::InvalidTransition);
    }
    let records = persistence
        .pending_realtime(64)
        .await
        .map_err(DesktopRecordingRealtimeErrorV1::Persistence)?;
    let published = !records.is_empty();
    for record in records {
        let event_id = event_id(record.recording_evidence_id, record.recording_revision);
        let request = ManagedRuntimeClientRealtimePublishRequestV1 {
            contract: Some(contract_reference_v1(REALTIME_CONTRACT_NAME_V1)),
            logical_owner_id: record.logical_owner_id,
            event_id: event_id.to_vec(),
            cursor: format!("desktop-call-recording/{}", record.sequence_id),
            event_kind: REALTIME_CONTRACT_NAME_V1.to_owned(),
            occurred_at_unix_millis: u64::try_from(record.occurred_at_unix_ms)
                .map_err(|_| DesktopRecordingRealtimeErrorV1::InvalidTransition)?,
            causation_id: String::new(),
            correlation_id: String::new(),
            trace_id: String::new(),
            payload: record.payload_bytes,
        };
        validate_managed_client_realtime_publish_request_v1(&request)
            .map_err(|_| DesktopRecordingRealtimeErrorV1::InvalidTransition)?;
        let cursor = request.cursor.clone();
        let response = channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::PublishClientRealtime(request)),
                },
                dispatcher,
            )
            .map_err(|_| DesktopRecordingRealtimeErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(DesktopRecordingRealtimeErrorV1::Unavailable);
        }
        let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
            return Err(DesktopRecordingRealtimeErrorV1::Unavailable);
        };
        if validate_managed_client_realtime_publish_response_v1(&response).is_err()
            || response.accepted_cursor != cursor
        {
            return Err(DesktopRecordingRealtimeErrorV1::Unavailable);
        }
        persistence
            .mark_realtime_published(
                record.sequence_id,
                record.payload_sha256,
                published_at_unix_ms,
            )
            .await
            .map_err(DesktopRecordingRealtimeErrorV1::Persistence)?;
    }
    Ok(published)
}

fn event_id(recording_id: [u8; 16], revision: u64) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.desktop-call-recording.client-realtime.v1\0");
    hash.update(recording_id);
    hash.update(revision.to_be_bytes());
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realtime_event_identity_is_revision_specific() {
        assert_ne!(event_id([4; 16], 1), event_id([4; 16], 2));
    }
}
