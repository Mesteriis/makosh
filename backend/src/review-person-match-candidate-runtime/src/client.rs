use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_DECIDE_CONTRACT_NAME_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_GET_CONTRACT_NAME_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_LIST_CONTRACT_NAME_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1, REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1,
    review_person_match_candidate_decision_contract_reference_v1,
    wire::{
        DecidePersonMatchCandidateRequestV1, GetPersonMatchCandidateRequestV1,
        ListPersonMatchCandidatesRequestV1, ListPersonMatchCandidatesResultV1,
        PersonMatchCandidateEvidenceV1 as WireEvidence,
        PersonMatchCandidatePromotionStatusV1 as WirePromotion,
        PersonMatchCandidateStateV1 as WireState, PersonMatchCandidateSummaryV1,
        PersonMatchKindV1 as WireMatchKind, PublicPersonSourceIdentityV1 as WireSource,
    },
};
use makosh_review_person_match_candidate_core::{
    PersonMatchCandidatePromotionStatusV1, PersonMatchCandidateReviewV1,
    PersonMatchCandidateStateV1, PersonMatchKindV1, PublicPersonSourceIdentityV1,
};
use makosh_review_person_match_candidate_persistence::{
    ReviewPersonMatchCandidatePersistenceErrorV1, ReviewPersonMatchCandidatePersistenceV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::execution::REVIEW_DECISION_GATEWAY_MODULE_ID_V1;
use crate::{
    ReviewPersonMatchCandidateExecutionContextV1, process_person_match_candidate_decision_v1,
};

pub async fn dispatch_review_person_match_candidate_client_request_v1(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1
        && request.owner_id == REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1
        && request.logical_owner_id == logical_owner_id
        && now_unix_millis > 0;
    let result = if accepted {
        dispatch(
            persistence,
            runtime_instance_id,
            runtime_generation,
            logical_owner_id,
            &request,
            now_unix_millis,
        )
        .await
    } else {
        Err("REJECTED")
    };
    match result {
        Ok(payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: payload,
            error_code: String::new(),
        },
        Err(error_code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: error_code.to_owned(),
        },
    }
}

async fn dispatch(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    match contract.name.as_str() {
        REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_GET_CONTRACT_NAME_V1 => {
            let query =
                GetPersonMatchCandidateRequestV1::decode(request.request_payload.as_slice())
                    .map_err(|_| "INVALID_ARGUMENT")?;
            if !accepted_payload_owner_v1(&query.logical_owner_id, logical_owner_id) {
                return Err("REJECTED");
            }
            let review = persistence
                .load_review(logical_owner_id, required_id(&query.review_id)?)
                .await
                .map_err(persistence_error)?;
            Ok(summary(&review).encode_to_vec())
        }
        REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_LIST_CONTRACT_NAME_V1 => {
            let query =
                ListPersonMatchCandidatesRequestV1::decode(request.request_payload.as_slice())
                    .map_err(|_| "INVALID_ARGUMENT")?;
            if !accepted_payload_owner_v1(&query.logical_owner_id, logical_owner_id)
                || !(1..=200).contains(&query.limit)
            {
                return Err("INVALID_ARGUMENT");
            }
            let after = optional_id(&query.after_review_id)?;
            let mut reviews = persistence
                .list_reviews(logical_owner_id, after, query.limit + 1)
                .await
                .map_err(persistence_error)?;
            let has_more = reviews.len() > query.limit as usize;
            reviews.truncate(query.limit as usize);
            let next = page_cursor_v1(
                &reviews
                    .iter()
                    .map(|review| review.review_id)
                    .collect::<Vec<_>>(),
                query.limit as usize,
                has_more,
            )
            .map_or_else(Vec::new, |id| id.to_vec());
            Ok(ListPersonMatchCandidatesResultV1 {
                candidates: reviews.iter().map(summary).collect(),
                next_after_review_id: next,
            }
            .encode_to_vec())
        }
        REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_DECIDE_CONTRACT_NAME_V1 => {
            let mut payload =
                DecidePersonMatchCandidateRequestV1::decode(request.request_payload.as_slice())
                    .map_err(|_| "INVALID_ARGUMENT")?;
            if request.authenticated_device_id.is_empty() {
                return Err("REJECTED");
            }
            payload.decided_by_owner_device_id =
                device_public_id(logical_owner_id, &request.authenticated_device_id).to_vec();
            payload.decided_at_unix_millis = now_unix_millis;
            let record = decision_record(
                payload,
                runtime_instance_id,
                runtime_generation,
                now_unix_millis,
            )?;
            let outcome = process_person_match_candidate_decision_v1(
                persistence,
                &record,
                &ReviewPersonMatchCandidateExecutionContextV1 {
                    logical_owner_id: logical_owner_id.to_owned(),
                    runtime_instance_id: runtime_instance_id.to_owned(),
                    runtime_generation,
                    now_unix_millis,
                },
            )
            .await
            .map_err(|_| "FAILED_PRECONDITION")?;
            let review = match outcome {
                makosh_review_person_match_candidate_persistence::ReviewPersonMatchCandidateReplayOutcomeV1::Applied(value)
                | makosh_review_person_match_candidate_persistence::ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(value) => value,
            };
            Ok(summary(&review).encode_to_vec())
        }
        _ => Err("REJECTED"),
    }
}

fn decision_record(
    payload: DecidePersonMatchCandidateRequestV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    now_unix_millis: i64,
) -> Result<OutboxRecordV1, &'static str> {
    let operation_id = required_id(&payload.operation_id)?;
    let review_id = required_id(&payload.review_id)?;
    let source_runtime_id = stable_id(
        b"makosh.review.client-runtime.v1",
        runtime_instance_id.as_bytes(),
    );
    let expected = review_person_match_candidate_decision_contract_reference_v1();
    let payload_bytes = payload.encode_to_vec();
    let recorded_at = timestamp(now_unix_millis)?;
    let deadline = timestamp(
        now_unix_millis
            .checked_add(30_000)
            .ok_or("INVALID_ARGUMENT")?,
    )?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: operation_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: expected.owner,
            name: expected.name,
            major: expected.major,
            revision: expected.revision,
            schema_sha256: expected.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: REVIEW_DECISION_GATEWAY_MODULE_ID_V1.to_owned(),
            runtime_instance_id: source_runtime_id.to_vec(),
            runtime_generation,
        }),
        recorded_at: Some(recorded_at),
        partition_key: review_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: review_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::OwnerDevice as i32,
            actor_id: payload.decided_by_owner_device_id.clone(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: REVIEW_DECISION_GATEWAY_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(&payload_bytes).to_vec(),
            deadline: Some(deadline),
            logical_attempt: 1,
        })),
        payload: payload_bytes,
    };
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(|_| "INVALID_ARGUMENT")
}

fn accepted_payload_owner_v1(payload_owner: &str, authenticated_owner: &str) -> bool {
    payload_owner.is_empty() || payload_owner == authenticated_owner
}

fn page_cursor_v1(returned_ids: &[[u8; 16]], limit: usize, has_more: bool) -> Option<[u8; 16]> {
    has_more
        .then(|| returned_ids.get(limit.saturating_sub(1)).copied())
        .flatten()
}

fn summary(review: &PersonMatchCandidateReviewV1) -> PersonMatchCandidateSummaryV1 {
    PersonMatchCandidateSummaryV1 {
        review_id: review.review_id.to_vec(),
        evidence: Some(WireEvidence {
            evidence_event_id: review.evidence.evidence_event_id.to_vec(),
            candidate_id: review.evidence.candidate_id.to_vec(),
            logical_owner_id: review.evidence.logical_owner_id.clone(),
            first_person_id: review.evidence.first_person_id.to_vec(),
            second_person_id: review.evidence.second_person_id.to_vec(),
            first_source: Some(wire_source(review.evidence.first_source)),
            second_source: Some(wire_source(review.evidence.second_source)),
            match_kind: match review.evidence.match_kind {
                PersonMatchKindV1::NormalizedEmail => WireMatchKind::PersonMatchKindNormalizedEmail,
                PersonMatchKindV1::NormalizedPhone => WireMatchKind::PersonMatchKindNormalizedPhone,
            } as i32,
            observed_at_unix_millis: review.evidence.observed_at_unix_millis,
            resulting_owner_revision: review.evidence.resulting_owner_revision,
            candidate_digest: review.evidence.candidate_digest.to_vec(),
        }),
        state: match review.state {
            PersonMatchCandidateStateV1::Pending => WireState::PersonMatchCandidateStatePending,
            PersonMatchCandidateStateV1::Approved => WireState::PersonMatchCandidateStateApproved,
            PersonMatchCandidateStateV1::Rejected => WireState::PersonMatchCandidateStateRejected,
        } as i32,
        promotion_status: match review.promotion_status {
            PersonMatchCandidatePromotionStatusV1::NotRequested => {
                WirePromotion::PersonMatchCandidatePromotionStatusNotRequested
            }
            PersonMatchCandidatePromotionStatusV1::Pending => {
                WirePromotion::PersonMatchCandidatePromotionStatusPending
            }
            PersonMatchCandidatePromotionStatusV1::Succeeded => {
                WirePromotion::PersonMatchCandidatePromotionStatusSucceeded
            }
            PersonMatchCandidatePromotionStatusV1::Failed => {
                WirePromotion::PersonMatchCandidatePromotionStatusFailed
            }
        } as i32,
        review_revision: review.review_revision,
        decision_id: review.decision_id.map(|value| value.to_vec()),
        decided_by_owner_device_id: review
            .decided_by_owner_device_id
            .map(|value| value.to_vec()),
        decided_at_unix_millis: review.decided_at_unix_millis,
        approved_action_digest: review.approved_action_digest.map(|value| value.to_vec()),
    }
}

fn wire_source(value: PublicPersonSourceIdentityV1) -> WireSource {
    WireSource {
        integration_public_id: value.integration_public_id.to_vec(),
        account_public_id: value.account_public_id.to_vec(),
        provider_source_contact_public_id: value.provider_source_contact_public_id.to_vec(),
    }
}

fn persistence_error(error: ReviewPersonMatchCandidatePersistenceErrorV1) -> &'static str {
    match error {
        ReviewPersonMatchCandidatePersistenceErrorV1::NotFound => "NOT_FOUND",
        ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput => "INVALID_ARGUMENT",
        ReviewPersonMatchCandidatePersistenceErrorV1::Conflict
        | ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict => "FAILED_PRECONDITION",
        _ => "UNAVAILABLE",
    }
}

fn required_id(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let id: [u8; 16] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or("INVALID_ARGUMENT")
}

fn optional_id(value: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        required_id(value).map(Some)
    }
}

fn device_public_id(owner: &str, device: &str) -> [u8; 16] {
    let mut input = Vec::with_capacity(owner.len() + device.len() + 2);
    input.extend_from_slice(owner.as_bytes());
    input.push(0);
    input.extend_from_slice(device.as_bytes());
    stable_id(b"makosh.review.owner-device.v1", &input)
}

fn stable_id(domain: &[u8], value: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value);
    let digest = hash.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest[..16]);
    id
}

fn timestamp(unix_millis: i64) -> Result<Timestamp, &'static str> {
    if unix_millis <= 0 {
        return Err("INVALID_ARGUMENT");
    }
    Ok(Timestamp {
        seconds: unix_millis / 1_000,
        nanos: ((unix_millis % 1_000) * 1_000_000) as i32,
    })
}

#[cfg(test)]
mod client_boundary_tests {
    use super::{accepted_payload_owner_v1, page_cursor_v1};

    #[test]
    fn authenticated_owner_accepts_empty_or_exact_payload_owner_only() {
        assert!(accepted_payload_owner_v1("", "owner-1"));
        assert!(accepted_payload_owner_v1("owner-1", "owner-1"));
        assert!(!accepted_payload_owner_v1("owner-2", "owner-1"));
    }

    #[test]
    fn page_cursor_is_last_returned_and_never_the_overflow_item() {
        let overflow = vec![[1_u8; 16], [2_u8; 16], [3_u8; 16]];
        assert_eq!(page_cursor_v1(&overflow, 2, true), Some([2_u8; 16]));
        assert_eq!(page_cursor_v1(&overflow[..2], 2, false), None);
    }
}
