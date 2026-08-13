use std::os::unix::net::UnixStream;

use makosh_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_CONTRACT_MAJOR_V1, REVIEW_TASK_CANDIDATE_MODULE_ID_V1,
    ReviewTaskCandidateEnvelopeContextV1, build_review_task_candidate_approved_outbox_record_v1,
    wire::{
        DecideReviewTaskCandidateRequestV1, DecideReviewTaskCandidateResponseV1,
        GetReviewTaskCandidateRequestV1, GetReviewTaskCandidateResponseV1,
        ListReviewTaskCandidatesRequestV1, ListReviewTaskCandidatesResponseV1,
        ReviewTaskCandidateDecisionV1 as WireDecision, ReviewTaskCandidateErrorCodeV1 as WireError,
        ReviewTaskCandidatePromotionStatusV1 as WirePromotionStatus,
        ReviewTaskCandidateStateV1 as WireState, ReviewTaskCandidateStatusChangedV1,
        ReviewTaskCandidateSummaryV1, TaskCandidateApprovedForPromotionV1,
    },
};
use makosh_review_task_candidate_core::{
    ReviewTaskCandidateDecisionV1, ReviewTaskCandidatePromotionStatusV1,
    ReviewTaskCandidateStateV1, ReviewTaskCandidateTimestampV1, ReviewTaskCandidateV1,
};
use makosh_review_task_candidate_persistence::{
    CheckReviewTaskCandidateDecisionReplayV1, DecideReviewTaskCandidateOperationV1,
    ListReviewTaskCandidatesV1, ReviewTaskCandidateDecisionOutcomeV1,
    ReviewTaskCandidateOutboxRecordV1, ReviewTaskCandidatePersistenceErrorV1,
    ReviewTaskCandidatePersistenceV1, ReviewTaskCandidateRealtimeTransitionV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::blob_materialization::{ReviewTaskCandidateBlobErrorV1, write_tasks_candidate_v1};

pub(crate) struct ReviewTaskCandidateClientRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub authenticated_device_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn decide_payload_v1(
    persistence: &ReviewTaskCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ReviewTaskCandidateClientRuntimeContextV1<'_>,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = DecideReviewTaskCandidateRequestV1::decode(payload) else {
        return decide_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    let Some(operation_id) = id16(&request.operation_id) else {
        return decide_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    let Some(review_id) = id16(&request.review_id) else {
        return decide_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    let Some(decision) = decision(request.decision) else {
        return decide_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_TASK_CANDIDATE_CONTRACT_MAJOR_V1
        || request.expected_review_revision == 0
        || runtime.logical_owner_id.is_empty()
        || runtime.authenticated_device_id.is_empty()
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.now_unix_millis <= 0
    {
        return decide_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    }
    let owner_device_id = owner_device_actor_id(runtime.authenticated_device_id);
    let request_sha256: [u8; 32] = Sha256::digest(payload).into();
    let replay = persistence
        .load_decision_replay(&CheckReviewTaskCandidateDecisionReplayV1 {
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
        return decide_error(WireError::ReviewTaskCandidateErrorCodeRevisionConflict);
    }
    if current.state != ReviewTaskCandidateStateV1::Pending {
        return decide_error(WireError::ReviewTaskCandidateErrorCodeTerminalDecision);
    }
    let decided_at = timestamp(runtime.now_unix_millis);
    let approved_event = if decision == ReviewTaskCandidateDecisionV1::Approve {
        match approved_event(channel, dispatcher, owner_device_id, runtime, &current) {
            Ok(value) => Some(value),
            Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt) => {
                return decide_error(WireError::ReviewTaskCandidateErrorCodePolicy);
            }
            Err(ReviewTaskCandidateBlobErrorV1::Unavailable) => {
                return decide_error(WireError::ReviewTaskCandidateErrorCodeUnavailable);
            }
        }
    } else {
        None
    };
    let result = persistence
        .decide(DecideReviewTaskCandidateOperationV1 {
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
        Ok(ReviewTaskCandidateDecisionOutcomeV1::Applied(review)) => decide_response(review, false),
        Ok(ReviewTaskCandidateDecisionOutcomeV1::Replayed(review)) => decide_response(review, true),
        Err(error) => decide_error(persistence_error(error)),
    }
}

pub(crate) async fn get_payload_v1(
    persistence: &ReviewTaskCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetReviewTaskCandidateRequestV1::decode(payload) else {
        return get_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    let Some(review_id) = id16(&request.review_id) else {
        return get_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_TASK_CANDIDATE_CONTRACT_MAJOR_V1 {
        return get_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    }
    match persistence.load_review(logical_owner_id, &review_id).await {
        Ok(review) => GetReviewTaskCandidateResponseV1 {
            review: Some(summary(&review)),
            error: WireError::ReviewTaskCandidateErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(error) => get_error(persistence_error(error)),
    }
}

pub(crate) async fn list_payload_v1(
    persistence: &ReviewTaskCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = ListReviewTaskCandidatesRequestV1::decode(payload) else {
        return list_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    let after_review_id = if request.after_review_id.is_empty() {
        None
    } else {
        let Some(review_id) = id16(&request.after_review_id) else {
            return list_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
        };
        Some(review_id)
    };
    let state = match request.state {
        None => None,
        Some(value) => {
            let Some(value) = review_state(value) else {
                return list_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
            };
            Some(value)
        }
    };
    let Ok(limit) = u16::try_from(request.limit) else {
        return list_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    };
    if request.protocol_major != REVIEW_TASK_CANDIDATE_CONTRACT_MAJOR_V1
        || logical_owner_id.is_empty()
    {
        return list_error(WireError::ReviewTaskCandidateErrorCodeInvalidRequest);
    }
    match persistence
        .list_reviews(
            logical_owner_id,
            ListReviewTaskCandidatesV1 {
                after_review_id,
                state,
                limit,
            },
        )
        .await
    {
        Ok(page) => ListReviewTaskCandidatesResponseV1 {
            reviews: page.reviews.iter().map(summary).collect(),
            next_after_review_id: page
                .next_after_review_id
                .map_or_else(Vec::new, |review_id| review_id.to_vec()),
            error: WireError::ReviewTaskCandidateErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(error) => list_error(persistence_error(error)),
    }
}

pub(crate) fn realtime_payload_v1(transition: &ReviewTaskCandidateRealtimeTransitionV1) -> Vec<u8> {
    ReviewTaskCandidateStatusChangedV1 {
        review_id: transition.review_id.to_vec(),
        candidate_id: transition.candidate_id.to_vec(),
        state: wire_state(transition.state) as i32,
        promotion_status: wire_promotion(transition.promotion_status) as i32,
        review_revision: transition.review_revision,
        occurred_at_unix_millis: u64::try_from(transition.occurred_at_unix_millis)
            .unwrap_or_default(),
        error: WireError::ReviewTaskCandidateErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn approved_event(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    owner_device_id: [u8; 16],
    runtime: &ReviewTaskCandidateClientRuntimeContextV1<'_>,
    review: &ReviewTaskCandidateV1,
) -> Result<ReviewTaskCandidateOutboxRecordV1, ReviewTaskCandidateBlobErrorV1> {
    let receipt = write_tasks_candidate_v1(channel, dispatcher, review)?;
    let record = build_review_task_candidate_approved_outbox_record_v1(
        TaskCandidateApprovedForPromotionV1 {
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
    .map_err(|_| ReviewTaskCandidateBlobErrorV1::InvalidReceipt)?;
    Ok(ReviewTaskCandidateOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    })
}

fn envelope_context(
    runtime: &ReviewTaskCandidateClientRuntimeContextV1<'_>,
) -> ReviewTaskCandidateEnvelopeContextV1 {
    ReviewTaskCandidateEnvelopeContextV1 {
        module_id: REVIEW_TASK_CANDIDATE_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn summary(review: &ReviewTaskCandidateV1) -> ReviewTaskCandidateSummaryV1 {
    ReviewTaskCandidateSummaryV1 {
        review_id: review.review_id.to_vec(),
        candidate_id: review.candidate_id.to_vec(),
        candidate_digest: review.candidate_digest.to_vec(),
        source_evidence_id: review.source_evidence_id.to_vec(),
        source_evidence_revision: review.source_evidence_revision,
        title: review.title.clone(),
        due_text_hint: review.due_text_hint.clone(),
        assignee_label_hint: review.assignee_label_hint.clone(),
        state: wire_state(review.state) as i32,
        promotion_status: wire_promotion(review.promotion_status) as i32,
        review_revision: review.review_revision,
        decided_by_owner_device_id: review
            .decided_by_owner_device_id
            .map(|value| value.to_vec()),
        decided_at_unix_millis: review.decided_at.and_then(timestamp_millis),
    }
}

fn decide_response(review: ReviewTaskCandidateV1, replayed: bool) -> Vec<u8> {
    DecideReviewTaskCandidateResponseV1 {
        review: Some(summary(&review)),
        replayed,
        error: WireError::ReviewTaskCandidateErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn decide_error(error: WireError) -> Vec<u8> {
    DecideReviewTaskCandidateResponseV1 {
        review: None,
        replayed: false,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(error: WireError) -> Vec<u8> {
    GetReviewTaskCandidateResponseV1 {
        review: None,
        error: error as i32,
    }
    .encode_to_vec()
}

fn list_error(error: WireError) -> Vec<u8> {
    ListReviewTaskCandidatesResponseV1 {
        reviews: Vec::new(),
        next_after_review_id: Vec::new(),
        error: error as i32,
    }
    .encode_to_vec()
}

fn persistence_error(error: ReviewTaskCandidatePersistenceErrorV1) -> WireError {
    match error {
        ReviewTaskCandidatePersistenceErrorV1::NotFound => {
            WireError::ReviewTaskCandidateErrorCodeNotFound
        }
        ReviewTaskCandidatePersistenceErrorV1::RevisionConflict => {
            WireError::ReviewTaskCandidateErrorCodeRevisionConflict
        }
        ReviewTaskCandidatePersistenceErrorV1::OperationConflict => {
            WireError::ReviewTaskCandidateErrorCodeOperationConflict
        }
        ReviewTaskCandidatePersistenceErrorV1::InvalidTransition => {
            WireError::ReviewTaskCandidateErrorCodeTerminalDecision
        }
        ReviewTaskCandidatePersistenceErrorV1::InvalidInput
        | ReviewTaskCandidatePersistenceErrorV1::InvalidRow
        | ReviewTaskCandidatePersistenceErrorV1::SubmissionConflict
        | ReviewTaskCandidatePersistenceErrorV1::InboxConflict => {
            WireError::ReviewTaskCandidateErrorCodeInvalidRequest
        }
        ReviewTaskCandidatePersistenceErrorV1::StorageUnavailable => {
            WireError::ReviewTaskCandidateErrorCodeUnavailable
        }
    }
}

fn decision(value: i32) -> Option<ReviewTaskCandidateDecisionV1> {
    match WireDecision::try_from(value).ok()? {
        WireDecision::ReviewTaskCandidateDecisionApprove => {
            Some(ReviewTaskCandidateDecisionV1::Approve)
        }
        WireDecision::ReviewTaskCandidateDecisionReject => {
            Some(ReviewTaskCandidateDecisionV1::Reject)
        }
        WireDecision::ReviewTaskCandidateDecisionUnspecified => None,
    }
}

fn review_state(value: i32) -> Option<ReviewTaskCandidateStateV1> {
    match WireState::try_from(value).ok()? {
        WireState::ReviewTaskCandidateStatePending => Some(ReviewTaskCandidateStateV1::Pending),
        WireState::ReviewTaskCandidateStateApproved => Some(ReviewTaskCandidateStateV1::Approved),
        WireState::ReviewTaskCandidateStateRejected => Some(ReviewTaskCandidateStateV1::Rejected),
        WireState::ReviewTaskCandidateStateUnspecified => None,
    }
}

pub(crate) const fn wire_state(value: ReviewTaskCandidateStateV1) -> WireState {
    match value {
        ReviewTaskCandidateStateV1::Pending => WireState::ReviewTaskCandidateStatePending,
        ReviewTaskCandidateStateV1::Approved => WireState::ReviewTaskCandidateStateApproved,
        ReviewTaskCandidateStateV1::Rejected => WireState::ReviewTaskCandidateStateRejected,
    }
}

pub(crate) const fn wire_promotion(
    value: ReviewTaskCandidatePromotionStatusV1,
) -> WirePromotionStatus {
    match value {
        ReviewTaskCandidatePromotionStatusV1::NotRequested => {
            WirePromotionStatus::ReviewTaskCandidatePromotionStatusNotRequested
        }
        ReviewTaskCandidatePromotionStatusV1::Pending => {
            WirePromotionStatus::ReviewTaskCandidatePromotionStatusPending
        }
        ReviewTaskCandidatePromotionStatusV1::Succeeded => {
            WirePromotionStatus::ReviewTaskCandidatePromotionStatusSucceeded
        }
        ReviewTaskCandidatePromotionStatusV1::Failed => {
            WirePromotionStatus::ReviewTaskCandidatePromotionStatusFailed
        }
    }
}

fn timestamp(now_unix_millis: i64) -> ReviewTaskCandidateTimestampV1 {
    ReviewTaskCandidateTimestampV1 {
        unix_seconds: now_unix_millis / 1_000,
        nanos: i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default(),
    }
}

fn timestamp_millis(value: ReviewTaskCandidateTimestampV1) -> Option<u64> {
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
        let payload = realtime_payload_v1(&ReviewTaskCandidateRealtimeTransitionV1 {
            sequence: 1,
            review_id: [1; 16],
            candidate_id: [2; 16],
            state: ReviewTaskCandidateStateV1::Pending,
            promotion_status: ReviewTaskCandidatePromotionStatusV1::NotRequested,
            review_revision: 1,
            occurred_at_unix_millis: 1_800_000_000_000,
        });
        let decoded = ReviewTaskCandidateStatusChangedV1::decode(payload.as_slice()).expect("wire");
        assert_eq!(decoded.review_id, vec![1; 16]);
        assert_eq!(decoded.candidate_id, vec![2; 16]);
    }
}
