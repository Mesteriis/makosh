use std::os::unix::net::UnixStream;

use makosh_review_attention_api::REVIEW_ATTENTION_REALTIME_EVENT_KIND_V1;
use makosh_review_attention_persistence::{
    REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1, ReviewAttentionPersistenceErrorV1,
    ReviewAttentionPersistenceV1, ReviewAttentionRealtimeTransitionV1,
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

use crate::{
    client_port::realtime_transition_payload_v1, contracts::review_attention_realtime_contract_v1,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ReviewAttentionRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl ReviewAttentionRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &ReviewAttentionPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, ReviewAttentionRealtimeErrorV1> {
        let transitions = persistence
            .realtime_window(
                logical_owner_id,
                self.last_sequence,
                REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1,
            )
            .await
            .map_err(ReviewAttentionRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let request = publish_request(logical_owner_id, &transition);
            if validate_managed_client_realtime_publish_request_v1(&request).is_err() {
                return Err(ReviewAttentionRealtimeErrorV1::InvalidTransition);
            }
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| ReviewAttentionRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(ReviewAttentionRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(ReviewAttentionRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(ReviewAttentionRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = Some(transition.sequence);
        }
        Ok(published)
    }
}

fn publish_request(
    logical_owner_id: &str,
    transition: &ReviewAttentionRealtimeTransitionV1,
) -> ManagedRuntimeClientRealtimePublishRequestV1 {
    ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(review_attention_realtime_contract_v1()),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(transition.attention_id, transition.revision).to_vec(),
        cursor: format!("review-attention/{}", transition.sequence),
        event_kind: REVIEW_ATTENTION_REALTIME_EVENT_KIND_V1.to_owned(),
        occurred_at_unix_millis: u64::try_from(transition.occurred_at.unix_seconds)
            .unwrap_or_default()
            .saturating_mul(1_000)
            .saturating_add(
                u64::try_from(transition.occurred_at.nanos / 1_000_000).unwrap_or_default(),
            ),
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: realtime_transition_payload_v1(transition),
    }
}

fn event_id(attention_id: [u8; 16], revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.attention.client-realtime.v1");
    digest.update(attention_id);
    digest.update(revision.to_be_bytes());
    digest.finalize()[..16].try_into().expect("exact prefix")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewAttentionRealtimeErrorV1 {
    InvalidTransition,
    Persistence(ReviewAttentionPersistenceErrorV1),
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_review_attention_core::{
        ReviewDispositionV1, ReviewImportanceV1, ReviewTimestampV1,
    };
    use makosh_runtime_protocol::validation::client_realtime::validate_managed_client_realtime_publish_request_v1;

    #[test]
    fn transition_maps_to_valid_shared_realtime_request() {
        let request = publish_request(
            "owner-1",
            &ReviewAttentionRealtimeTransitionV1 {
                sequence: 1,
                attention_id: [1; 16],
                revision: 2,
                disposition: ReviewDispositionV1::Reviewed,
                pinned: false,
                importance: ReviewImportanceV1::Normal,
                snoozed_until: None,
                occurred_at: ReviewTimestampV1 {
                    unix_seconds: 1_783_100_000,
                    nanos: 0,
                },
            },
        );
        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&request),
            Ok(())
        );
        assert_eq!(request.cursor, "review-attention/1");
    }
}
