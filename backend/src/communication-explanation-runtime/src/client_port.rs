use makosh_communication_explanation_api::{
    COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1,
    wire::{
        CommunicationExplanationCandidateV1,
        CommunicationExplanationCompletenessV1 as WireCompleteness,
        CommunicationExplanationErrorCodeV1 as WireError,
        CommunicationExplanationReasonKindV1 as WireReasonKind,
        CommunicationExplanationReasonV1 as WireReason,
        CommunicationExplanationSourceBasisV1 as WireSourceBasis,
        CommunicationExplanationStateV1 as WireState, GetCommunicationExplanationRequestV1,
        GetCommunicationExplanationResponseV1, StartCommunicationExplanationRequestV1,
        StartCommunicationExplanationResponseV1,
    },
};
use makosh_communication_explanation_core::{
    CommunicationExplanationCompletenessV1, CommunicationExplanationDraftV1,
    CommunicationExplanationReasonKindV1, CommunicationExplanationReasonV1,
    CommunicationExplanationRejectionCodeV1, CommunicationExplanationSourceBasisV1,
    CommunicationExplanationStateV1,
};
use makosh_communication_explanation_persistence::{
    CommunicationExplanationPersistenceErrorV1, CommunicationExplanationPersistenceV1,
    CreateCommunicationExplanationOutcomeV1, CreateCommunicationExplanationRunV1,
    PersistedCommunicationExplanationRunV1,
};
use makosh_communications_ai_source_api::{
    CommunicationExplanationSourceEnvelopeContextV1,
    build_communication_explanation_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct CommunicationExplanationClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_communication_explanation_payload_v1(
    persistence: &CommunicationExplanationPersistenceV1,
    logical_owner_id: &str,
    runtime: &CommunicationExplanationClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCommunicationExplanationRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::CommunicationExplanationErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::CommunicationExplanationErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::CommunicationExplanationErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateCommunicationExplanationRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateCommunicationExplanationOutcomeV1::Created(value))
        | Ok(CreateCommunicationExplanationOutcomeV1::Existing(value)) => value,
        Err(CommunicationExplanationPersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationExplanationErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationExplanationErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == CommunicationExplanationStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::CommunicationExplanationErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartCommunicationExplanationResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_communication_explanation_payload_v1(
    persistence: &CommunicationExplanationPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCommunicationExplanationRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::CommunicationExplanationErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::CommunicationExplanationErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::CommunicationExplanationErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(CommunicationExplanationPersistenceErrorV1::NotFound) => get_error(
            response_run_id,
            WireError::CommunicationExplanationErrorCodeNotFound,
        ),
        Err(_) => get_error(
            response_run_id,
            WireError::CommunicationExplanationErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartCommunicationExplanationRequestV1,
) -> Option<CommunicationExplanationDraftV1> {
    if request.protocol_major != COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(CommunicationExplanationDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &CommunicationExplanationClientRuntimeContextV1<'_>,
    draft: &CommunicationExplanationDraftV1,
    now_unix_millis: i64,
) -> Option<makosh_events_protocol::delivery::OutboxRecordV1> {
    if now_unix_millis <= 0
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
    {
        return None;
    }
    let seconds = now_unix_millis / 1_000;
    let deadline = seconds.checked_add(300)?;
    let nanos = i32::try_from((now_unix_millis % 1_000) * 1_000_000).ok()?;
    build_communication_explanation_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationExplanationSourceEnvelopeContextV1 {
            module_id: makosh_communication_explanation_api::COMMUNICATION_EXPLANATION_MODULE_ID_V1
                .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedCommunicationExplanationRunV1) -> Vec<u8> {
    let candidate = run
        .status
        .candidate
        .map(|value| CommunicationExplanationCandidateV1 {
            reasons: value.reasons.into_iter().map(wire_reason).collect(),
            completeness: wire_completeness(value.completeness) as i32,
            confidence_basis_points: value.confidence_basis_points,
        });
    GetCommunicationExplanationResponseV1 {
        run_id: run.draft.run_id.to_vec(),
        source_message_id: run.draft.source_message_id.to_vec(),
        expected_source_revision: run.draft.expected_source_revision,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        candidate,
        error: rejection_error(run.status.rejection) as i32,
    }
    .encode_to_vec()
}

fn run_id(logical_owner_id: &str, operation_id: &[u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_explanation.run.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(operation_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let value: [u8; 16] = value.try_into().map_err(|_| ())?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(())
}

pub(crate) const fn wire_state(value: CommunicationExplanationStateV1) -> WireState {
    match value {
        CommunicationExplanationStateV1::Accepted => {
            WireState::CommunicationExplanationStateAccepted
        }
        CommunicationExplanationStateV1::PreparingSource => {
            WireState::CommunicationExplanationStatePreparingSource
        }
        CommunicationExplanationStateV1::AwaitingInference => {
            WireState::CommunicationExplanationStateAwaitingInference
        }
        CommunicationExplanationStateV1::Ready => WireState::CommunicationExplanationStateReady,
        CommunicationExplanationStateV1::Rejected => {
            WireState::CommunicationExplanationStateRejected
        }
    }
}

fn wire_reason(value: CommunicationExplanationReasonV1) -> WireReason {
    WireReason {
        kind: wire_reason_kind(value.kind) as i32,
        explanation_utf8: value.explanation_utf8,
        source_basis: wire_source_basis(value.source_basis) as i32,
        confidence_basis_points: value.confidence_basis_points,
    }
}

const fn wire_reason_kind(value: CommunicationExplanationReasonKindV1) -> WireReasonKind {
    match value {
        CommunicationExplanationReasonKindV1::Urgency => {
            WireReasonKind::CommunicationExplanationReasonKindUrgency
        }
        CommunicationExplanationReasonKindV1::FinancialAttention => {
            WireReasonKind::CommunicationExplanationReasonKindFinancialAttention
        }
        CommunicationExplanationReasonKindV1::LegalOrContractual => {
            WireReasonKind::CommunicationExplanationReasonKindLegalOrContractual
        }
        CommunicationExplanationReasonKindV1::ReplyRequested => {
            WireReasonKind::CommunicationExplanationReasonKindReplyRequested
        }
        CommunicationExplanationReasonKindV1::Deadline => {
            WireReasonKind::CommunicationExplanationReasonKindDeadline
        }
        CommunicationExplanationReasonKindV1::AttachmentReference => {
            WireReasonKind::CommunicationExplanationReasonKindAttachmentReference
        }
        CommunicationExplanationReasonKindV1::MarketingOrBulk => {
            WireReasonKind::CommunicationExplanationReasonKindMarketingOrBulk
        }
        CommunicationExplanationReasonKindV1::OtherAttention => {
            WireReasonKind::CommunicationExplanationReasonKindOtherAttention
        }
    }
}

const fn wire_source_basis(value: CommunicationExplanationSourceBasisV1) -> WireSourceBasis {
    match value {
        CommunicationExplanationSourceBasisV1::Subject => {
            WireSourceBasis::CommunicationExplanationSourceBasisSubject
        }
        CommunicationExplanationSourceBasisV1::Body => {
            WireSourceBasis::CommunicationExplanationSourceBasisBody
        }
        CommunicationExplanationSourceBasisV1::CanonicalMetadata => {
            WireSourceBasis::CommunicationExplanationSourceBasisCanonicalMetadata
        }
        CommunicationExplanationSourceBasisV1::Combined => {
            WireSourceBasis::CommunicationExplanationSourceBasisCombined
        }
    }
}

const fn wire_completeness(value: CommunicationExplanationCompletenessV1) -> WireCompleteness {
    match value {
        CommunicationExplanationCompletenessV1::Complete => {
            WireCompleteness::CommunicationExplanationCompletenessComplete
        }
        CommunicationExplanationCompletenessV1::Partial => {
            WireCompleteness::CommunicationExplanationCompletenessPartial
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<CommunicationExplanationRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::CommunicationExplanationErrorCodeUnspecified,
        Some(CommunicationExplanationRejectionCodeV1::InvalidRequest) => {
            WireError::CommunicationExplanationErrorCodeInvalidRequest
        }
        Some(CommunicationExplanationRejectionCodeV1::SourceRejected) => {
            WireError::CommunicationExplanationErrorCodeSourceRejected
        }
        Some(CommunicationExplanationRejectionCodeV1::InferenceRejected) => {
            WireError::CommunicationExplanationErrorCodeInferenceRejected
        }
        Some(CommunicationExplanationRejectionCodeV1::Policy) => {
            WireError::CommunicationExplanationErrorCodePolicy
        }
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartCommunicationExplanationResponseV1 {
        run_id,
        state: WireState::CommunicationExplanationStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetCommunicationExplanationResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::CommunicationExplanationStateUnspecified as i32,
        state_revision: 0,
        candidate: None,
        error: error as i32,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_identity_is_owner_and_operation_bound() {
        assert_eq!(run_id("owner-1", &[1; 16]), run_id("owner-1", &[1; 16]));
        assert_ne!(run_id("owner-1", &[1; 16]), run_id("owner-2", &[1; 16]));
        assert_ne!(run_id("owner-1", &[1; 16]), run_id("owner-1", &[2; 16]));
    }

    #[test]
    fn start_draft_is_concrete_and_provider_neutral() {
        let draft = start_draft(
            "owner-1",
            StartCommunicationExplanationRequestV1 {
                protocol_major: 1,
                operation_id: vec![1; 16],
                source_message_id: vec![2; 16],
                expected_source_revision: 3,
            },
        )
        .expect("draft");
        assert_eq!(draft.source_message_id, [2; 16]);
        assert_eq!(draft.expected_source_revision, 3);
        assert_ne!(draft.run_id, [0; 16]);
    }
}
