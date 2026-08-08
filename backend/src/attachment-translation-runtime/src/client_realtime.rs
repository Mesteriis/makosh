use std::os::unix::net::UnixStream;

use makosh_attachment_translation_api::wire::AttachmentTranslationArtifactV1 as WireArtifact;
use makosh_attachment_translation_api::{
    ATTACHMENT_TRANSLATION_REALTIME_EVENT_KIND_V1, wire::AttachmentTranslationStatusChangedV1,
};
use makosh_attachment_translation_core::AttachmentTranslationArtifactV1;
use makosh_attachment_translation_core::AttachmentTranslationStateV1;
use makosh_attachment_translation_persistence::{
    AttachmentTranslationPersistenceErrorV1, AttachmentTranslationPersistenceV1,
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
    client_port::{rejection_error, wire_artifact, wire_state},
    contracts::attachment_translation_realtime_contract_v1,
};

const REPLAY_WINDOW_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttachmentTranslationClientRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl AttachmentTranslationClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &AttachmentTranslationPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, AttachmentTranslationClientRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(logical_owner_id, self.last_sequence, REPLAY_WINDOW_V1)
            .await
            .map_err(AttachmentTranslationClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let artifact = if transition.state == AttachmentTranslationStateV1::Ready {
                persistence
                    .load_run(logical_owner_id, &transition.run_id)
                    .await
                    .map_err(AttachmentTranslationClientRealtimeErrorV1::Persistence)?
                    .status
                    .artifact
            } else {
                None
            };
            let artifact = realtime_artifact(transition.state, artifact)?;
            let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_millis)
                .map_err(|_| AttachmentTranslationClientRealtimeErrorV1::InvalidTransition)?;
            let payload = AttachmentTranslationStatusChangedV1 {
                run_id: transition.run_id.to_vec(),
                state: wire_state(transition.state) as i32,
                state_revision: transition.state_revision,
                artifact,
                occurred_at_unix_millis,
                error: rejection_error(transition.rejection) as i32,
            }
            .encode_to_vec();
            let request = ManagedRuntimeClientRealtimePublishRequestV1 {
                contract: Some(attachment_translation_realtime_contract_v1()),
                logical_owner_id: logical_owner_id.to_owned(),
                event_id: event_id(transition.run_id, transition.state_revision).to_vec(),
                cursor: format!("attachment-translation/{}", transition.sequence),
                event_kind: ATTACHMENT_TRANSLATION_REALTIME_EVENT_KIND_V1.to_owned(),
                occurred_at_unix_millis,
                causation_id: String::new(),
                correlation_id: String::new(),
                trace_id: String::new(),
                payload,
            };
            validate_managed_client_realtime_publish_request_v1(&request)
                .map_err(|_| AttachmentTranslationClientRealtimeErrorV1::InvalidTransition)?;
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| AttachmentTranslationClientRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(AttachmentTranslationClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(AttachmentTranslationClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(AttachmentTranslationClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = Some(transition.sequence);
        }
        Ok(published)
    }
}

fn realtime_artifact(
    state: AttachmentTranslationStateV1,
    artifact: Option<AttachmentTranslationArtifactV1>,
) -> Result<Option<WireArtifact>, AttachmentTranslationClientRealtimeErrorV1> {
    match (state, artifact) {
        (AttachmentTranslationStateV1::Ready, Some(artifact)) => Ok(Some(wire_artifact(artifact))),
        (AttachmentTranslationStateV1::Ready, None) | (_, Some(_)) => {
            Err(AttachmentTranslationClientRealtimeErrorV1::InvalidTransition)
        }
        (_, None) => Ok(None),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachmentTranslationClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(AttachmentTranslationPersistenceErrorV1),
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
    use makosh_attachment_translation_core::{
        AttachmentTranslationArtifactV1, AttachmentTranslationCompletenessV1,
        AttachmentTranslationDetectedLanguageV1, AttachmentTranslationLanguageV1,
        AttachmentTranslationStateV1,
    };

    use super::{event_id, realtime_artifact};

    #[test]
    fn realtime_event_identity_is_stable_and_revision_specific() {
        assert_eq!(event_id([1; 16], 2), event_id([1; 16], 2));
        assert_ne!(event_id([1; 16], 2), event_id([1; 16], 3));
    }

    #[test]
    fn ready_realtime_requires_and_projects_bounded_artifact_metadata() {
        let artifact = AttachmentTranslationArtifactV1 {
            artifact_id: [1; 16],
            translated_sha256: [2; 32],
            translated_size_bytes: 7,
            detected_source_language: AttachmentTranslationDetectedLanguageV1::English,
            target_language: AttachmentTranslationLanguageV1::Russian,
            completeness: AttachmentTranslationCompletenessV1::Complete,
            confidence_basis_points: 9_000,
        };
        let projected = realtime_artifact(AttachmentTranslationStateV1::Ready, Some(artifact))
            .expect("ready realtime artifact")
            .expect("projected ready artifact");
        assert_eq!(projected.translated_sha256, vec![2; 32]);
        assert_eq!(projected.translated_size_bytes, 7);
        assert!(realtime_artifact(AttachmentTranslationStateV1::Ready, None).is_err());
        assert!(
            realtime_artifact(AttachmentTranslationStateV1::AwaitingInference, None)
                .expect("nonterminal realtime")
                .is_none()
        );
    }
}
