use std::os::unix::net::UnixStream;

use makosh_review_obligation_candidate_api::REVIEW_OBLIGATION_CANDIDATE_REALTIME_EVENT_KIND_V1;
use makosh_review_obligation_candidate_persistence::{
    ReviewObligationCandidatePersistenceErrorV1, ReviewObligationCandidatePersistenceV1,
    ReviewObligationCandidateRealtimeTransitionV1,
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

use crate::{client_port::realtime_payload_v1, contracts::realtime_contract_v1};

const REALTIME_REPLAY_LIMIT_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ReviewObligationCandidateClientRealtimePublisherV1 {
    last_sequence: u64,
}

impl ReviewObligationCandidateClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &ReviewObligationCandidatePersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, ReviewObligationCandidateClientRealtimeErrorV1> {
        let transitions = persistence
            .realtime_after(
                logical_owner_id,
                self.last_sequence,
                REALTIME_REPLAY_LIMIT_V1,
            )
            .await
            .map_err(ReviewObligationCandidateClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let request = publish_request(logical_owner_id, &transition)?;
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| ReviewObligationCandidateClientRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(ReviewObligationCandidateClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(ReviewObligationCandidateClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(ReviewObligationCandidateClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = transition.sequence;
        }
        Ok(published)
    }
}

fn publish_request(
    logical_owner_id: &str,
    transition: &ReviewObligationCandidateRealtimeTransitionV1,
) -> Result<
    ManagedRuntimeClientRealtimePublishRequestV1,
    ReviewObligationCandidateClientRealtimeErrorV1,
> {
    let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_millis)
        .map_err(|_| ReviewObligationCandidateClientRealtimeErrorV1::InvalidTransition)?;
    let request = ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(realtime_contract_v1()),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(transition.review_id, transition.review_revision).to_vec(),
        cursor: format!("review-obligation-candidate/{}", transition.sequence),
        event_kind: REVIEW_OBLIGATION_CANDIDATE_REALTIME_EVENT_KIND_V1.to_owned(),
        occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: realtime_payload_v1(transition),
    };
    validate_managed_client_realtime_publish_request_v1(&request)
        .map_err(|_| ReviewObligationCandidateClientRealtimeErrorV1::InvalidTransition)?;
    Ok(request)
}

fn event_id(review_id: [u8; 16], review_revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.obligation-candidate.client-realtime.v1\0");
    digest.update(review_id);
    digest.update(review_revision.to_be_bytes());
    digest.finalize()[..16].try_into().expect("digest prefix")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewObligationCandidateClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(ReviewObligationCandidatePersistenceErrorV1),
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_review_obligation_candidate_core::{
        ReviewObligationCandidatePromotionStatusV1, ReviewObligationCandidateStateV1,
    };

    #[test]
    fn transition_builds_shared_replayable_frame() {
        let request = publish_request(
            "owner-1",
            &ReviewObligationCandidateRealtimeTransitionV1 {
                sequence: 9,
                review_id: [1; 16],
                candidate_id: [2; 16],
                state: ReviewObligationCandidateStateV1::Approved,
                promotion_status: ReviewObligationCandidatePromotionStatusV1::Pending,
                review_revision: 2,
                occurred_at_unix_millis: 1_800_000_000_000,
            },
        )
        .expect("frame");
        assert_eq!(request.cursor, "review-obligation-candidate/9");
        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&request),
            Ok(())
        );
    }
}
