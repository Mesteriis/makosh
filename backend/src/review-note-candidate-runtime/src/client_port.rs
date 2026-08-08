use std::os::unix::net::UnixStream;

use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1, REVIEW_NOTE_CANDIDATE_MODULE_ID_V1,
    ReviewNoteCandidateEnvelopeContextV1, build_review_note_candidate_approved_outbox_record_v1,
    wire::{
        DecideReviewNoteCandidateRequestV1, DecideReviewNoteCandidateResponseV1,
        GetReviewNoteCandidateRequestV1, GetReviewNoteCandidateResponseV1,
        NoteCandidateApprovedForPromotionV1, ReviewNoteCandidateDecisionV1 as WireDecision,
        ReviewNoteCandidateErrorCodeV1 as WireError,
        ReviewNoteCandidatePromotionStatusV1 as WirePromotionStatus,
        ReviewNoteCandidateStateV1 as WireState, ReviewNoteCandidateStatusChangedV1,
        ReviewNoteCandidateSummaryV1,
    },
};
use makosh_review_note_candidate_core::{
    ReviewNoteCandidateDecisionV1, ReviewNoteCandidatePromotionStatusV1,
    ReviewNoteCandidateStateV1, ReviewNoteCandidateTimestampV1, ReviewNoteCandidateV1,
    ReviewNoteSourceBasisV1, ReviewNoteTopicHintV1,
};
use makosh_review_note_candidate_persistence::{
    CheckReviewNoteCandidateDecisionReplayV1, DecideReviewNoteCandidateOperationV1,
    ReviewNoteCandidateDecisionOutcomeV1, ReviewNoteCandidateOutboxRecordV1,
    ReviewNoteCandidatePersistenceErrorV1, ReviewNoteCandidatePersistenceV1,
    ReviewNoteCandidateRealtimeTransitionV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::blob_materialization::{ReviewNoteCandidateBlobErrorV1, write_promotion_candidate_v1};

pub(crate) struct ReviewNoteCandidateClientRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub authenticated_device_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn decide_payload_v1(
    persistence: &ReviewNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ReviewNoteCandidateClientRuntimeContextV1<'_>,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = DecideReviewNoteCandidateRequestV1::decode(payload) else {
        return decide_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    };
    let Some(operation_id) = id16(&request.operation_id) else {
        return decide_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    };
    let Some(review_id) = id16(&request.review_id) else {
        return decide_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    };
    let Some(decision) = decision(request.decision) else {
        return decide_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1
        || request.expected_review_revision == 0
        || runtime.logical_owner_id.is_empty()
        || runtime.authenticated_device_id.is_empty()
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.now_unix_millis <= 0
    {
        return decide_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    }
    let owner_device_id = owner_device_actor_id(runtime.authenticated_device_id);
    let request_sha256: [u8; 32] = Sha256::digest(payload).into();
    let replay = persistence
        .load_decision_replay(&CheckReviewNoteCandidateDecisionReplayV1 {
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
        return decide_error(WireError::ReviewNoteCandidateErrorCodeRevisionConflict);
    }
    if current.state != ReviewNoteCandidateStateV1::Pending {
        return decide_error(WireError::ReviewNoteCandidateErrorCodeTerminalDecision);
    }
    let decided_at = timestamp(runtime.now_unix_millis);
    let approved_event = if decision == ReviewNoteCandidateDecisionV1::Approve {
        match approved_event(channel, dispatcher, owner_device_id, runtime, &current) {
            Ok(value) => Some(value),
            Err(ReviewNoteCandidateBlobErrorV1::InvalidReceipt) => {
                return decide_error(WireError::ReviewNoteCandidateErrorCodePolicy);
            }
            Err(ReviewNoteCandidateBlobErrorV1::Unavailable) => {
                return decide_error(WireError::ReviewNoteCandidateErrorCodeUnavailable);
            }
        }
    } else {
        None
    };
    let result = persistence
        .decide(DecideReviewNoteCandidateOperationV1 {
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
        Ok(ReviewNoteCandidateDecisionOutcomeV1::Applied(review)) => decide_response(review, false),
        Ok(ReviewNoteCandidateDecisionOutcomeV1::Replayed(review)) => decide_response(review, true),
        Err(error) => decide_error(persistence_error(error)),
    }
}

pub(crate) async fn get_payload_v1(
    persistence: &ReviewNoteCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetReviewNoteCandidateRequestV1::decode(payload) else {
        return get_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    };
    let Some(review_id) = id16(&request.review_id) else {
        return get_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1 {
        return get_error(WireError::ReviewNoteCandidateErrorCodeInvalidRequest);
    }
    match persistence.load_review(logical_owner_id, &review_id).await {
        Ok(review) => GetReviewNoteCandidateResponseV1 {
            review: Some(summary(&review)),
            error: WireError::ReviewNoteCandidateErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(error) => get_error(persistence_error(error)),
    }
}

pub(crate) fn realtime_payload_v1(transition: &ReviewNoteCandidateRealtimeTransitionV1) -> Vec<u8> {
    ReviewNoteCandidateStatusChangedV1 {
        review_id: transition.review_id.to_vec(),
        candidate_id: transition.candidate_id.to_vec(),
        state: wire_state(transition.state) as i32,
        promotion_status: wire_promotion(transition.promotion_status) as i32,
        review_revision: transition.review_revision,
        occurred_at_unix_millis: u64::try_from(transition.occurred_at_unix_millis)
            .unwrap_or_default(),
        error: WireError::ReviewNoteCandidateErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn approved_event(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    owner_device_id: [u8; 16],
    runtime: &ReviewNoteCandidateClientRuntimeContextV1<'_>,
    review: &ReviewNoteCandidateV1,
) -> Result<ReviewNoteCandidateOutboxRecordV1, ReviewNoteCandidateBlobErrorV1> {
    let receipt = write_promotion_candidate_v1(channel, dispatcher, review)?;
    let record = build_review_note_candidate_approved_outbox_record_v1(
        NoteCandidateApprovedForPromotionV1 {
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
    .map_err(|_| ReviewNoteCandidateBlobErrorV1::InvalidReceipt)?;
    Ok(ReviewNoteCandidateOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    })
}

fn envelope_context(
    runtime: &ReviewNoteCandidateClientRuntimeContextV1<'_>,
) -> ReviewNoteCandidateEnvelopeContextV1 {
    ReviewNoteCandidateEnvelopeContextV1 {
        module_id: REVIEW_NOTE_CANDIDATE_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn summary(review: &ReviewNoteCandidateV1) -> ReviewNoteCandidateSummaryV1 {
    ReviewNoteCandidateSummaryV1 {
        review_id: review.review_id.to_vec(),
        candidate_id: review.candidate_id.to_vec(),
        candidate_digest: review.candidate_digest.to_vec(),
        source_evidence_id: review.source_evidence_id.to_vec(),
        source_evidence_revision: review.source_evidence_revision,
        title: review.title.clone(),
        excerpt: review.excerpt.clone(),
        topic_hints: review
            .topic_hints
            .iter()
            .copied()
            .map(topic_hint_code)
            .collect(),
        source_basis: source_basis_code(review.source_basis),
        confidence_basis_points: review.confidence_basis_points,
        state: wire_state(review.state) as i32,
        promotion_status: wire_promotion(review.promotion_status) as i32,
        review_revision: review.review_revision,
        decided_by_owner_device_id: review
            .decided_by_owner_device_id
            .map(|value| value.to_vec()),
        decided_at_unix_millis: review.decided_at.and_then(timestamp_millis),
    }
}

const fn source_basis_code(value: ReviewNoteSourceBasisV1) -> i32 {
    match value {
        ReviewNoteSourceBasisV1::Subject => 1,
        ReviewNoteSourceBasisV1::Body => 2,
        ReviewNoteSourceBasisV1::Combined => 3,
    }
}

const fn topic_hint_code(value: ReviewNoteTopicHintV1) -> i32 {
    match value {
        ReviewNoteTopicHintV1::Financial => 1,
        ReviewNoteTopicHintV1::Legal => 2,
        ReviewNoteTopicHintV1::DecisionStatement => 3,
        ReviewNoteTopicHintV1::DeadlineStatement => 4,
    }
}

fn decide_response(review: ReviewNoteCandidateV1, replayed: bool) -> Vec<u8> {
    DecideReviewNoteCandidateResponseV1 {
        review: Some(summary(&review)),
        replayed,
        error: WireError::ReviewNoteCandidateErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn decide_error(error: WireError) -> Vec<u8> {
    DecideReviewNoteCandidateResponseV1 {
        review: None,
        replayed: false,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(error: WireError) -> Vec<u8> {
    GetReviewNoteCandidateResponseV1 {
        review: None,
        error: error as i32,
    }
    .encode_to_vec()
}

fn persistence_error(error: ReviewNoteCandidatePersistenceErrorV1) -> WireError {
    match error {
        ReviewNoteCandidatePersistenceErrorV1::NotFound => {
            WireError::ReviewNoteCandidateErrorCodeNotFound
        }
        ReviewNoteCandidatePersistenceErrorV1::RevisionConflict => {
            WireError::ReviewNoteCandidateErrorCodeRevisionConflict
        }
        ReviewNoteCandidatePersistenceErrorV1::OperationConflict => {
            WireError::ReviewNoteCandidateErrorCodeOperationConflict
        }
        ReviewNoteCandidatePersistenceErrorV1::InvalidTransition => {
            WireError::ReviewNoteCandidateErrorCodeTerminalDecision
        }
        ReviewNoteCandidatePersistenceErrorV1::InvalidInput
        | ReviewNoteCandidatePersistenceErrorV1::InvalidRow
        | ReviewNoteCandidatePersistenceErrorV1::SubmissionConflict
        | ReviewNoteCandidatePersistenceErrorV1::InboxConflict => {
            WireError::ReviewNoteCandidateErrorCodeInvalidRequest
        }
        ReviewNoteCandidatePersistenceErrorV1::StorageUnavailable => {
            WireError::ReviewNoteCandidateErrorCodeUnavailable
        }
    }
}

fn decision(value: i32) -> Option<ReviewNoteCandidateDecisionV1> {
    match WireDecision::try_from(value).ok()? {
        WireDecision::ReviewNoteCandidateDecisionApprove => {
            Some(ReviewNoteCandidateDecisionV1::Approve)
        }
        WireDecision::ReviewNoteCandidateDecisionReject => {
            Some(ReviewNoteCandidateDecisionV1::Reject)
        }
        WireDecision::ReviewNoteCandidateDecisionUnspecified => None,
    }
}

pub(crate) const fn wire_state(value: ReviewNoteCandidateStateV1) -> WireState {
    match value {
        ReviewNoteCandidateStateV1::Pending => WireState::ReviewNoteCandidateStatePending,
        ReviewNoteCandidateStateV1::Approved => WireState::ReviewNoteCandidateStateApproved,
        ReviewNoteCandidateStateV1::Rejected => WireState::ReviewNoteCandidateStateRejected,
    }
}

pub(crate) const fn wire_promotion(
    value: ReviewNoteCandidatePromotionStatusV1,
) -> WirePromotionStatus {
    match value {
        ReviewNoteCandidatePromotionStatusV1::NotRequested => {
            WirePromotionStatus::ReviewNoteCandidatePromotionStatusNotRequested
        }
        ReviewNoteCandidatePromotionStatusV1::Pending => {
            WirePromotionStatus::ReviewNoteCandidatePromotionStatusPending
        }
        ReviewNoteCandidatePromotionStatusV1::Succeeded => {
            WirePromotionStatus::ReviewNoteCandidatePromotionStatusSucceeded
        }
        ReviewNoteCandidatePromotionStatusV1::Failed => {
            WirePromotionStatus::ReviewNoteCandidatePromotionStatusFailed
        }
    }
}

fn timestamp(now_unix_millis: i64) -> ReviewNoteCandidateTimestampV1 {
    ReviewNoteCandidateTimestampV1 {
        unix_seconds: now_unix_millis / 1_000,
        nanos: i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default(),
    }
}

fn timestamp_millis(value: ReviewNoteCandidateTimestampV1) -> Option<u64> {
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
        let payload = realtime_payload_v1(&ReviewNoteCandidateRealtimeTransitionV1 {
            sequence: 1,
            review_id: [1; 16],
            candidate_id: [2; 16],
            state: ReviewNoteCandidateStateV1::Pending,
            promotion_status: ReviewNoteCandidatePromotionStatusV1::NotRequested,
            review_revision: 1,
            occurred_at_unix_millis: 1_800_000_000_000,
        });
        let decoded = ReviewNoteCandidateStatusChangedV1::decode(payload.as_slice()).expect("wire");
        assert_eq!(decoded.review_id, vec![1; 16]);
        assert_eq!(decoded.candidate_id, vec![2; 16]);
    }
}
