use makosh_communication_recipient_suggestion_api::{
    COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1,
    wire::{
        CommunicationRecipientRationaleV1 as WireRationale,
        CommunicationRecipientRoleCandidateV1 as WireCandidate,
        CommunicationRecipientRoleV1 as WireRole,
        CommunicationRecipientSourceBasisV1 as WireSourceBasis,
        CommunicationRecipientSuggestionErrorCodeV1 as WireError,
        CommunicationRecipientSuggestionStateV1 as WireState,
        GetCommunicationRecipientSuggestionRequestV1,
        GetCommunicationRecipientSuggestionResponseV1,
        StartCommunicationRecipientSuggestionRequestV1,
        StartCommunicationRecipientSuggestionResponseV1,
    },
};
use makosh_communication_recipient_suggestion_core::{
    CommunicationRecipientCandidateV1, CommunicationRecipientRationaleV1,
    CommunicationRecipientRoleV1, CommunicationRecipientSourceBasisV1,
    CommunicationRecipientSuggestionDraftV1, CommunicationRecipientSuggestionRejectionCodeV1,
    CommunicationRecipientSuggestionStateV1,
};
use makosh_communication_recipient_suggestion_persistence::{
    CommunicationRecipientSuggestionPersistenceErrorV1,
    CommunicationRecipientSuggestionPersistenceV1, CreateCommunicationRecipientSuggestionOutcomeV1,
    CreateCommunicationRecipientSuggestionRunV1, PersistedCommunicationRecipientSuggestionRunV1,
};
use makosh_communications_recipient_source_api::{
    CommunicationRecipientSourceEnvelopeContextV1,
    build_communication_recipient_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct CommunicationRecipientSuggestionClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_communication_recipient_suggestion_payload_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    logical_owner_id: &str,
    runtime: &CommunicationRecipientSuggestionClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCommunicationRecipientSuggestionRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::CommunicationRecipientSuggestionErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateCommunicationRecipientSuggestionRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateCommunicationRecipientSuggestionOutcomeV1::Created(value))
        | Ok(CreateCommunicationRecipientSuggestionOutcomeV1::Existing(value)) => value,
        Err(CommunicationRecipientSuggestionPersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationRecipientSuggestionErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == CommunicationRecipientSuggestionStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::CommunicationRecipientSuggestionErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartCommunicationRecipientSuggestionResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_communication_recipient_suggestion_payload_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCommunicationRecipientSuggestionRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(CommunicationRecipientSuggestionPersistenceErrorV1::NotFound) => get_error(
            response_run_id,
            WireError::CommunicationRecipientSuggestionErrorCodeNotFound,
        ),
        Err(_) => get_error(
            response_run_id,
            WireError::CommunicationRecipientSuggestionErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartCommunicationRecipientSuggestionRequestV1,
) -> Option<CommunicationRecipientSuggestionDraftV1> {
    if request.protocol_major != COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(CommunicationRecipientSuggestionDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &CommunicationRecipientSuggestionClientRuntimeContextV1<'_>,
    draft: &CommunicationRecipientSuggestionDraftV1,
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
    build_communication_recipient_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationRecipientSourceEnvelopeContextV1 {
            module_id: makosh_communication_recipient_suggestion_api::COMMUNICATION_RECIPIENT_SUGGESTION_MODULE_ID_V1
                .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedCommunicationRecipientSuggestionRunV1) -> Vec<u8> {
    let candidates = run
        .status
        .candidates
        .unwrap_or_default()
        .into_iter()
        .map(wire_candidate)
        .collect();
    GetCommunicationRecipientSuggestionResponseV1 {
        run_id: run.draft.run_id.to_vec(),
        source_message_id: run.draft.source_message_id.to_vec(),
        expected_source_revision: run.draft.expected_source_revision,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        candidates,
        error: rejection_error(run.status.rejection) as i32,
    }
    .encode_to_vec()
}

fn run_id(logical_owner_id: &str, operation_id: &[u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_recipient_suggestion.run.v1\0");
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

pub(crate) const fn wire_state(value: CommunicationRecipientSuggestionStateV1) -> WireState {
    match value {
        CommunicationRecipientSuggestionStateV1::Accepted => {
            WireState::CommunicationRecipientSuggestionStateAccepted
        }
        CommunicationRecipientSuggestionStateV1::PreparingSource => {
            WireState::CommunicationRecipientSuggestionStatePreparingSource
        }
        CommunicationRecipientSuggestionStateV1::Evaluating => {
            WireState::CommunicationRecipientSuggestionStateEvaluating
        }
        CommunicationRecipientSuggestionStateV1::Ready => {
            WireState::CommunicationRecipientSuggestionStateReady
        }
        CommunicationRecipientSuggestionStateV1::Rejected => {
            WireState::CommunicationRecipientSuggestionStateRejected
        }
    }
}

fn wire_candidate(value: CommunicationRecipientCandidateV1) -> WireCandidate {
    WireCandidate {
        role: wire_role(value.role) as i32,
        rationale: wire_rationale(value.rationale) as i32,
        source_basis: wire_source_basis(value.source_basis) as i32,
        confidence_basis_points: value.confidence_basis_points,
    }
}

const fn wire_role(value: CommunicationRecipientRoleV1) -> WireRole {
    match value {
        CommunicationRecipientRoleV1::AccountingOrBookkeeping => {
            WireRole::CommunicationRecipientRoleAccountingOrBookkeeping
        }
        CommunicationRecipientRoleV1::LegalCounsel => {
            WireRole::CommunicationRecipientRoleLegalCounsel
        }
        CommunicationRecipientRoleV1::ProjectStakeholder => {
            WireRole::CommunicationRecipientRoleProjectStakeholder
        }
    }
}

const fn wire_rationale(value: CommunicationRecipientRationaleV1) -> WireRationale {
    match value {
        CommunicationRecipientRationaleV1::FinancialDocumentOrPayment => {
            WireRationale::CommunicationRecipientRationaleFinancialDocumentOrPayment
        }
        CommunicationRecipientRationaleV1::LegalOrContractualReview => {
            WireRationale::CommunicationRecipientRationaleLegalOrContractualReview
        }
        CommunicationRecipientRationaleV1::ProjectStatusOrUpdate => {
            WireRationale::CommunicationRecipientRationaleProjectStatusOrUpdate
        }
    }
}

const fn wire_source_basis(value: CommunicationRecipientSourceBasisV1) -> WireSourceBasis {
    match value {
        CommunicationRecipientSourceBasisV1::Body => {
            WireSourceBasis::CommunicationRecipientSourceBasisBody
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<CommunicationRecipientSuggestionRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::CommunicationRecipientSuggestionErrorCodeUnspecified,
        Some(CommunicationRecipientSuggestionRejectionCodeV1::InvalidRequest) => {
            WireError::CommunicationRecipientSuggestionErrorCodeInvalidRequest
        }
        Some(CommunicationRecipientSuggestionRejectionCodeV1::SourceRejected) => {
            WireError::CommunicationRecipientSuggestionErrorCodeSourceRejected
        }
        Some(CommunicationRecipientSuggestionRejectionCodeV1::EvaluationRejected) => {
            WireError::CommunicationRecipientSuggestionErrorCodeEvaluationRejected
        }
        Some(CommunicationRecipientSuggestionRejectionCodeV1::Policy) => {
            WireError::CommunicationRecipientSuggestionErrorCodePolicy
        }
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartCommunicationRecipientSuggestionResponseV1 {
        run_id,
        state: WireState::CommunicationRecipientSuggestionStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetCommunicationRecipientSuggestionResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::CommunicationRecipientSuggestionStateUnspecified as i32,
        state_revision: 0,
        candidates: Vec::new(),
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
            StartCommunicationRecipientSuggestionRequestV1 {
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
