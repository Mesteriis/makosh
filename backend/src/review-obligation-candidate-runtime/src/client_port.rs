use std::os::unix::net::UnixStream;

use makosh_review_obligation_candidate_api::{
    REVIEW_OBLIGATION_CANDIDATE_CONTRACT_MAJOR_V1, REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1,
    ReviewObligationCandidateEnvelopeContextV1,
    build_review_obligation_candidate_approved_outbox_record_v1,
    wire::{
        DecideReviewObligationCandidateRequestV1, DecideReviewObligationCandidateResponseV1,
        GetReviewObligationCandidateRequestV1, GetReviewObligationCandidateResponseV1,
        ListReviewObligationCandidatesRequestV1, ListReviewObligationCandidatesResponseV1,
        ObligationCandidateApprovedForPromotionV1,
        ReviewObligationCandidateDecisionV1 as WireDecision,
        ReviewObligationCandidateErrorCodeV1 as WireError,
        ReviewObligationCandidatePromotionStatusV1 as WirePromotionStatus,
        ReviewObligationCandidateStateV1 as WireState, ReviewObligationCandidateStatusChangedV1,
        ReviewObligationCandidateSummaryV1, ReviewObligationEvidenceLinkV1 as WireEvidenceLink,
        TimestampV1 as WireTimestamp,
    },
};
use makosh_review_obligation_candidate_core::{
    ReviewObligationCandidateDecisionV1, ReviewObligationCandidatePromotionStatusV1,
    ReviewObligationCandidateStateV1, ReviewObligationCandidateTimestampV1,
    ReviewObligationCandidateV1,
};
use makosh_review_obligation_candidate_persistence::{
    CheckReviewObligationCandidateDecisionReplayV1, DecideReviewObligationCandidateOperationV1,
    ListReviewObligationCandidatesV1, ReviewObligationCandidateDecisionOutcomeV1,
    ReviewObligationCandidateOutboxRecordV1, ReviewObligationCandidatePersistenceErrorV1,
    ReviewObligationCandidatePersistenceV1, ReviewObligationCandidateRealtimeTransitionV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::blob_materialization::{
    ReviewObligationCandidateBlobErrorV1, write_obligations_candidate_v1,
};

pub(crate) struct ReviewObligationCandidateClientRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub authenticated_device_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn decide_payload_v1(
    persistence: &ReviewObligationCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ReviewObligationCandidateClientRuntimeContextV1<'_>,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = DecideReviewObligationCandidateRequestV1::decode(payload) else {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    let Some(operation_id) = id16(&request.operation_id) else {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    let Some(review_id) = id16(&request.review_id) else {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    let Some(decision) = decision(request.decision) else {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_OBLIGATION_CANDIDATE_CONTRACT_MAJOR_V1
        || request.expected_review_revision == 0
        || runtime.logical_owner_id.is_empty()
        || runtime.authenticated_device_id.is_empty()
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.now_unix_millis <= 0
    {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    }
    let owner_device_id = owner_device_actor_id(runtime.authenticated_device_id);
    let request_sha256: [u8; 32] = Sha256::digest(payload).into();
    let replay = persistence
        .load_decision_replay(&CheckReviewObligationCandidateDecisionReplayV1 {
            logical_owner_id: runtime.logical_owner_id.to_owned(),
            operation_id,
            request_sha256,
            review_id,
            expected_review_revision: request.expected_review_revision,
            decision,
            owner_device_id,
        })
        .await;
    match replay {
        Ok(Some(review)) => return decide_response(review, true),
        Ok(None) => {}
        Err(error) => return decide_error(persistence_error(error)),
    }
    let current = match persistence
        .load_review(runtime.logical_owner_id, &review_id)
        .await
    {
        Ok(value) => value,
        Err(error) => return decide_error(persistence_error(error)),
    };
    if current.review_revision != request.expected_review_revision {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeRevisionConflict);
    }
    if current.state != ReviewObligationCandidateStateV1::Pending {
        return decide_error(WireError::ReviewObligationCandidateErrorCodeTerminalDecision);
    }
    let decided_at = timestamp(runtime.now_unix_millis);
    let approved_event = if decision == ReviewObligationCandidateDecisionV1::Approve {
        match approved_event(channel, dispatcher, owner_device_id, runtime, &current) {
            Ok(value) => Some(value),
            Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt) => {
                return decide_error(WireError::ReviewObligationCandidateErrorCodePolicy);
            }
            Err(ReviewObligationCandidateBlobErrorV1::Unavailable) => {
                return decide_error(WireError::ReviewObligationCandidateErrorCodeUnavailable);
            }
        }
    } else {
        None
    };
    let result = persistence
        .decide(DecideReviewObligationCandidateOperationV1 {
            logical_owner_id: runtime.logical_owner_id.to_owned(),
            operation_id,
            request_sha256,
            review_id,
            expected_review_revision: request.expected_review_revision,
            decision,
            owner_device_id,
            decided_at,
            approved_event,
        })
        .await;
    match result {
        Ok(ReviewObligationCandidateDecisionOutcomeV1::Applied(review)) => {
            decide_response(review, false)
        }
        Ok(ReviewObligationCandidateDecisionOutcomeV1::Replayed(review)) => {
            decide_response(review, true)
        }
        Err(error) => decide_error(persistence_error(error)),
    }
}

pub(crate) async fn get_payload_v1(
    persistence: &ReviewObligationCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetReviewObligationCandidateRequestV1::decode(payload) else {
        return get_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    let Some(review_id) = id16(&request.review_id) else {
        return get_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_OBLIGATION_CANDIDATE_CONTRACT_MAJOR_V1 {
        return get_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    }
    match persistence.load_review(logical_owner_id, &review_id).await {
        Ok(review) => GetReviewObligationCandidateResponseV1 {
            review: Some(summary(&review)),
            error: WireError::ReviewObligationCandidateErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(error) => get_error(persistence_error(error)),
    }
}

pub(crate) async fn list_payload_v1(
    persistence: &ReviewObligationCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = ListReviewObligationCandidatesRequestV1::decode(payload) else {
        return list_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    let after_review_id = if request.after_review_id.is_empty() {
        None
    } else {
        let Some(review_id) = id16(&request.after_review_id) else {
            return list_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
        };
        Some(review_id)
    };
    let state = match request.state {
        None => None,
        Some(value) => {
            let Some(value) = review_state(value) else {
                return list_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
            };
            Some(value)
        }
    };
    let Ok(limit) = u16::try_from(request.limit) else {
        return list_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_OBLIGATION_CANDIDATE_CONTRACT_MAJOR_V1
        || logical_owner_id.is_empty()
    {
        return list_error(WireError::ReviewObligationCandidateErrorCodeInvalidRequest);
    }
    match persistence
        .list_reviews(
            logical_owner_id,
            ListReviewObligationCandidatesV1 {
                after_review_id,
                state,
                limit,
            },
        )
        .await
    {
        Ok(page) => ListReviewObligationCandidatesResponseV1 {
            reviews: page.reviews.iter().map(summary).collect(),
            next_after_review_id: page
                .next_after_review_id
                .map_or_else(Vec::new, |review_id| review_id.to_vec()),
            error: WireError::ReviewObligationCandidateErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(error) => list_error(persistence_error(error)),
    }
}

pub(crate) fn realtime_payload_v1(
    transition: &ReviewObligationCandidateRealtimeTransitionV1,
) -> Vec<u8> {
    ReviewObligationCandidateStatusChangedV1 {
        review_id: transition.review_id.to_vec(),
        candidate_id: transition.candidate_id.to_vec(),
        state: wire_state(transition.state) as i32,
        promotion_status: wire_promotion(transition.promotion_status) as i32,
        review_revision: transition.review_revision,
        occurred_at_unix_millis: u64::try_from(transition.occurred_at_unix_millis)
            .unwrap_or_default(),
        error: WireError::ReviewObligationCandidateErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn approved_event(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    owner_device_id: [u8; 16],
    runtime: &ReviewObligationCandidateClientRuntimeContextV1<'_>,
    review: &ReviewObligationCandidateV1,
) -> Result<ReviewObligationCandidateOutboxRecordV1, ReviewObligationCandidateBlobErrorV1> {
    let receipt = write_obligations_candidate_v1(channel, dispatcher, review)?;
    let record = build_review_obligation_candidate_approved_outbox_record_v1(
        ObligationCandidateApprovedForPromotionV1 {
            review_id: review.review_id.to_vec(),
            candidate_id: review.candidate_id.to_vec(),
            candidate_digest: review.candidate_digest.to_vec(),
            source_evidence_id: review.source_evidence_id.to_vec(),
            source_evidence_revision: review.source_evidence_revision,
            decision_revision: review.review_revision.saturating_add(1),
            decided_by_owner_device_id: owner_device_id.to_vec(),
            candidate_content: Some(receipt),
            logical_owner_id: runtime.logical_owner_id.to_owned(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ReviewObligationCandidateBlobErrorV1::InvalidReceipt)?;
    Ok(ReviewObligationCandidateOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    })
}

fn envelope_context(
    runtime: &ReviewObligationCandidateClientRuntimeContextV1<'_>,
) -> ReviewObligationCandidateEnvelopeContextV1 {
    ReviewObligationCandidateEnvelopeContextV1 {
        module_id: REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn summary(review: &ReviewObligationCandidateV1) -> ReviewObligationCandidateSummaryV1 {
    ReviewObligationCandidateSummaryV1 {
        review_id: review.review_id.to_vec(),
        candidate_id: review.candidate_id.to_vec(),
        candidate_digest: review.candidate_digest.to_vec(),
        source_evidence_id: review.source_evidence_id.to_vec(),
        source_evidence_revision: review.source_evidence_revision,
        statement: review.statement.clone(),
        due_at: review.due_at.map(|value| WireTimestamp {
            unix_seconds: value.unix_seconds,
            nanos: value.nanos,
        }),
        condition: review.condition.clone(),
        state: wire_state(review.state) as i32,
        promotion_status: wire_promotion(review.promotion_status) as i32,
        review_revision: review.review_revision,
        decided_by_owner_device_id: review
            .decided_by_owner_device_id
            .map(|value| value.to_vec()),
        decided_at_unix_millis: review.decided_at.and_then(timestamp_millis),
        obligated_party_id: review.obligated_party_id.to_vec(),
        beneficiary_party_id: review.beneficiary_party_id.map(|value| value.to_vec()),
        evidence_links: review
            .evidence_links
            .iter()
            .map(|value| WireEvidenceLink {
                evidence_link_id: value.evidence_link_id.to_vec(),
                evidence_owner_id: value.evidence_owner_id.clone(),
                evidence_record_id: value.evidence_record_id.to_vec(),
                evidence_revision: value.evidence_revision,
                evidence_digest: value.evidence_digest.to_vec(),
            })
            .collect(),
    }
}

fn decide_response(review: ReviewObligationCandidateV1, replayed: bool) -> Vec<u8> {
    DecideReviewObligationCandidateResponseV1 {
        review: Some(summary(&review)),
        replayed,
        error: WireError::ReviewObligationCandidateErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn decide_error(error: WireError) -> Vec<u8> {
    DecideReviewObligationCandidateResponseV1 {
        review: None,
        replayed: false,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(error: WireError) -> Vec<u8> {
    GetReviewObligationCandidateResponseV1 {
        review: None,
        error: error as i32,
    }
    .encode_to_vec()
}

fn list_error(error: WireError) -> Vec<u8> {
    ListReviewObligationCandidatesResponseV1 {
        reviews: Vec::new(),
        next_after_review_id: Vec::new(),
        error: error as i32,
    }
    .encode_to_vec()
}

fn persistence_error(error: ReviewObligationCandidatePersistenceErrorV1) -> WireError {
    match error {
        ReviewObligationCandidatePersistenceErrorV1::NotFound => {
            WireError::ReviewObligationCandidateErrorCodeNotFound
        }
        ReviewObligationCandidatePersistenceErrorV1::RevisionConflict => {
            WireError::ReviewObligationCandidateErrorCodeRevisionConflict
        }
        ReviewObligationCandidatePersistenceErrorV1::OperationConflict => {
            WireError::ReviewObligationCandidateErrorCodeOperationConflict
        }
        ReviewObligationCandidatePersistenceErrorV1::InvalidTransition => {
            WireError::ReviewObligationCandidateErrorCodeTerminalDecision
        }
        ReviewObligationCandidatePersistenceErrorV1::InvalidInput
        | ReviewObligationCandidatePersistenceErrorV1::InvalidRow
        | ReviewObligationCandidatePersistenceErrorV1::SubmissionConflict
        | ReviewObligationCandidatePersistenceErrorV1::InboxConflict => {
            WireError::ReviewObligationCandidateErrorCodeInvalidRequest
        }
        ReviewObligationCandidatePersistenceErrorV1::StorageUnavailable => {
            WireError::ReviewObligationCandidateErrorCodeUnavailable
        }
    }
}

fn decision(value: i32) -> Option<ReviewObligationCandidateDecisionV1> {
    match WireDecision::try_from(value).ok()? {
        WireDecision::ReviewObligationCandidateDecisionApprove => {
            Some(ReviewObligationCandidateDecisionV1::Approve)
        }
        WireDecision::ReviewObligationCandidateDecisionReject => {
            Some(ReviewObligationCandidateDecisionV1::Reject)
        }
        WireDecision::ReviewObligationCandidateDecisionUnspecified => None,
    }
}

fn review_state(value: i32) -> Option<ReviewObligationCandidateStateV1> {
    match WireState::try_from(value).ok()? {
        WireState::ReviewObligationCandidateStatePending => {
            Some(ReviewObligationCandidateStateV1::Pending)
        }
        WireState::ReviewObligationCandidateStateApproved => {
            Some(ReviewObligationCandidateStateV1::Approved)
        }
        WireState::ReviewObligationCandidateStateRejected => {
            Some(ReviewObligationCandidateStateV1::Rejected)
        }
        WireState::ReviewObligationCandidateStateUnspecified => None,
    }
}

pub(crate) const fn wire_state(value: ReviewObligationCandidateStateV1) -> WireState {
    match value {
        ReviewObligationCandidateStateV1::Pending => {
            WireState::ReviewObligationCandidateStatePending
        }
        ReviewObligationCandidateStateV1::Approved => {
            WireState::ReviewObligationCandidateStateApproved
        }
        ReviewObligationCandidateStateV1::Rejected => {
            WireState::ReviewObligationCandidateStateRejected
        }
    }
}

pub(crate) const fn wire_promotion(
    value: ReviewObligationCandidatePromotionStatusV1,
) -> WirePromotionStatus {
    match value {
        ReviewObligationCandidatePromotionStatusV1::NotRequested => {
            WirePromotionStatus::ReviewObligationCandidatePromotionStatusNotRequested
        }
        ReviewObligationCandidatePromotionStatusV1::Pending => {
            WirePromotionStatus::ReviewObligationCandidatePromotionStatusPending
        }
        ReviewObligationCandidatePromotionStatusV1::Succeeded => {
            WirePromotionStatus::ReviewObligationCandidatePromotionStatusSucceeded
        }
        ReviewObligationCandidatePromotionStatusV1::Failed => {
            WirePromotionStatus::ReviewObligationCandidatePromotionStatusFailed
        }
    }
}

fn timestamp(now_unix_millis: i64) -> ReviewObligationCandidateTimestampV1 {
    ReviewObligationCandidateTimestampV1 {
        unix_seconds: now_unix_millis / 1_000,
        nanos: i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default(),
    }
}

fn timestamp_millis(value: ReviewObligationCandidateTimestampV1) -> Option<u64> {
    u64::try_from(value.unix_seconds)
        .ok()?
        .checked_mul(1_000)?
        .checked_add(u64::try_from(value.nanos / 1_000_000).ok()?)
}

fn owner_device_actor_id(authenticated_device_id: &str) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.owner-device.actor.v1\0");
    digest.update(authenticated_device_id.as_bytes());
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Option<[u8; 16]> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_device_actor_is_stable_and_not_client_supplied() {
        assert_eq!(
            owner_device_actor_id("device-1"),
            owner_device_actor_id("device-1")
        );
        assert_ne!(
            owner_device_actor_id("device-1"),
            owner_device_actor_id("device-2")
        );
        assert_ne!(owner_device_actor_id("device-1"), [0; 16]);
    }

    #[test]
    fn realtime_payload_contains_no_candidate_text() {
        let payload = realtime_payload_v1(&ReviewObligationCandidateRealtimeTransitionV1 {
            sequence: 1,
            review_id: [1; 16],
            candidate_id: [2; 16],
            state: ReviewObligationCandidateStateV1::Pending,
            promotion_status: ReviewObligationCandidatePromotionStatusV1::NotRequested,
            review_revision: 1,
            occurred_at_unix_millis: 1_800_000_000_000,
        });
        let decoded =
            ReviewObligationCandidateStatusChangedV1::decode(payload.as_slice()).expect("wire");
        assert_eq!(decoded.review_id, vec![1; 16]);
        assert_eq!(decoded.candidate_id, vec![2; 16]);
    }
}
