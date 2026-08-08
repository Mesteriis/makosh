use std::os::unix::net::UnixStream;

use makosh_call_transcription_api::{
    REALTIME_CONTRACT_NAME_V1, REALTIME_EVENT_KIND_V1, contract_reference_v1,
    wire::{CallTranscriptionArtifactV1 as WireArtifact, CallTranscriptionStatusChangedV1},
};
use makosh_call_transcription_core::{CallTranscriptionStateV1, TranscriptArtifactV1};
use makosh_call_transcription_persistence::{
    CallTranscriptionPersistenceErrorV1, CallTranscriptionPersistenceV1,
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

use crate::client_port::{rejection_error, wire_artifact, wire_state};

const REPLAY_WINDOW_V1: u32 = 256;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CallTranscriptionClientRealtimePublisherV1 {
    last_sequence: u64,
}

impl CallTranscriptionClientRealtimePublisherV1 {
    pub async fn publish_pending(
        &mut self,
        persistence: &CallTranscriptionPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, CallTranscriptionClientRealtimeErrorV1> {
        let transitions = persistence
            .realtime_after(logical_owner_id, self.last_sequence, REPLAY_WINDOW_V1)
            .await
            .map_err(CallTranscriptionClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let artifact = if transition.state == CallTranscriptionStateV1::Ready {
                persistence
                    .load_run(logical_owner_id, transition.run_id)
                    .await
                    .map_err(CallTranscriptionClientRealtimeErrorV1::Persistence)?
                    .status
                    .artifact
            } else {
                None
            };
            let artifact = realtime_artifact(transition.state, artifact)?;
            let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_millis)
                .map_err(|_| CallTranscriptionClientRealtimeErrorV1::InvalidTransition)?;
            let payload = CallTranscriptionStatusChangedV1 {
                run_id: transition.run_id.to_vec(),
                state: wire_state(transition.state) as i32,
                state_revision: transition.state_revision,
                artifact,
                occurred_at_unix_millis,
                error: rejection_error(transition.rejection) as i32,
            }
            .encode_to_vec();
            let request = ManagedRuntimeClientRealtimePublishRequestV1 {
                contract: Some(contract_reference_v1(REALTIME_CONTRACT_NAME_V1)),
                logical_owner_id: logical_owner_id.to_owned(),
                event_id: event_id(transition.run_id, transition.state_revision).to_vec(),
                cursor: format!("call-transcription/{}", transition.sequence),
                event_kind: REALTIME_EVENT_KIND_V1.to_owned(),
                occurred_at_unix_millis,
                causation_id: String::new(),
                correlation_id: String::new(),
                trace_id: String::new(),
                payload,
            };
            validate_managed_client_realtime_publish_request_v1(&request)
                .map_err(|_| CallTranscriptionClientRealtimeErrorV1::InvalidTransition)?;
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| CallTranscriptionClientRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(CallTranscriptionClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(CallTranscriptionClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(CallTranscriptionClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = transition.sequence;
        }
        Ok(published)
    }
}

fn realtime_artifact(
    state: CallTranscriptionStateV1,
    artifact: Option<TranscriptArtifactV1>,
) -> Result<Option<WireArtifact>, CallTranscriptionClientRealtimeErrorV1> {
    match (state, artifact) {
        (CallTranscriptionStateV1::Ready, Some(artifact)) => Ok(Some(wire_artifact(artifact))),
        (CallTranscriptionStateV1::Ready, None) | (_, Some(_)) => {
            Err(CallTranscriptionClientRealtimeErrorV1::InvalidTransition)
        }
        (_, None) => Ok(None),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(CallTranscriptionPersistenceErrorV1),
    Unavailable,
}

fn event_id(run_id: [u8; 16], state_revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(run_id);
    digest.update(state_revision.to_be_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use makosh_call_transcription_core::{
        CallTranscriptionCompletenessV1, CallTranscriptionLanguageV1, CallTranscriptionStateV1,
        TranscriptArtifactV1,
    };

    use super::{event_id, realtime_artifact};

    #[test]
    fn realtime_event_identity_is_stable_and_revision_specific() {
        assert_eq!(event_id([1; 16], 2), event_id([1; 16], 2));
        assert_ne!(event_id([1; 16], 2), event_id([1; 16], 3));
    }

    #[test]
    fn ready_realtime_projects_metadata_only() {
        let artifact = TranscriptArtifactV1 {
            artifact_id: [1; 16],
            transcript_sha256: [2; 32],
            transcript_size_bytes: 7,
            detected_language: CallTranscriptionLanguageV1::English,
            duration_millis: 10,
            segment_count: 1,
            completeness: CallTranscriptionCompletenessV1::Complete,
            confidence_basis_points: 9_000,
        };
        let projected = realtime_artifact(CallTranscriptionStateV1::Ready, Some(artifact))
            .expect("ready realtime artifact")
            .expect("projected ready artifact");
        assert_eq!(projected.transcript_sha256, vec![2; 32]);
        assert_eq!(projected.transcript_size_bytes, 7);
        assert!(realtime_artifact(CallTranscriptionStateV1::Ready, None).is_err());
        assert!(
            realtime_artifact(CallTranscriptionStateV1::AwaitingStt, None)
                .expect("nonterminal realtime")
                .is_none()
        );
    }
}
