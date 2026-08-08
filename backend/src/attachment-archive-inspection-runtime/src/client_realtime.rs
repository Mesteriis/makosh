use std::os::unix::net::UnixStream;

use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_EVENT_KIND_V1, wire::ArchiveInspectionStatusChangedV1,
};
use makosh_attachment_archive_inspection_persistence::{
    ARCHIVE_INSPECTION_REALTIME_LIMIT_V1, ArchiveInspectionPersistenceErrorV1,
    AttachmentArchiveInspectionPersistenceV1,
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
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    client_port::{wire_error, wire_state},
    contracts::archive_inspection_realtime_contract_v1,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ArchiveInspectionClientRealtimePublisherV1 {
    last_sequence: u64,
}

impl ArchiveInspectionClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &AttachmentArchiveInspectionPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, ArchiveInspectionClientRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(
                logical_owner_id,
                self.last_sequence,
                ARCHIVE_INSPECTION_REALTIME_LIMIT_V1,
            )
            .await
            .map_err(ArchiveInspectionClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_millis)
                .map_err(|_| ArchiveInspectionClientRealtimeErrorV1::InvalidTransition)?;
            let payload = ArchiveInspectionStatusChangedV1 {
                run_id: transition.run_id.to_vec(),
                state: wire_state(transition.state) as i32,
                state_revision: transition.state_revision,
                occurred_at_unix_millis,
                error: wire_error(transition.error) as i32,
            }
            .encode_to_vec();
            let request = ManagedRuntimeClientRealtimePublishRequestV1 {
                contract: Some(archive_inspection_realtime_contract_v1()),
                logical_owner_id: logical_owner_id.to_owned(),
                event_id: event_id(transition.run_id, transition.state_revision).to_vec(),
                cursor: format!("attachment-archive-inspection/{}", transition.sequence),
                event_kind: ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_EVENT_KIND_V1.to_owned(),
                occurred_at_unix_millis,
                causation_id: String::new(),
                correlation_id: String::new(),
                trace_id: String::new(),
                payload,
            };
            validate_managed_client_realtime_publish_request_v1(&request)
                .map_err(|_| ArchiveInspectionClientRealtimeErrorV1::InvalidTransition)?;
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| ArchiveInspectionClientRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(ArchiveInspectionClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(ArchiveInspectionClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(ArchiveInspectionClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = transition.sequence;
        }
        Ok(published)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArchiveInspectionClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(ArchiveInspectionPersistenceErrorV1),
    Unavailable,
}

fn event_id(run_id: [u8; 16], state_revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-archive-inspection.client-realtime.v1\0");
    digest.update(run_id);
    digest.update(state_revision.to_be_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::event_id;

    #[test]
    fn realtime_event_identity_is_stable_and_revision_specific() {
        assert_eq!(event_id([1; 16], 2), event_id([1; 16], 2));
        assert_ne!(event_id([1; 16], 2), event_id([1; 16], 3));
        assert_ne!(event_id([1; 16], 2), event_id([2; 16], 2));
    }
}
