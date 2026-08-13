use makosh_decisions_api::{
    DECISIONS_ADD_ALTERNATIVE_CONNECT_PATH_V1, DECISIONS_ADD_EVIDENCE_CONNECT_PATH_V1,
    DECISIONS_CANCEL_CONNECT_PATH_V1, DECISIONS_CREATE_CONNECT_PATH_V1,
    DECISIONS_DECIDE_CONNECT_PATH_V1, DECISIONS_GET_CONNECT_PATH_V1,
    DECISIONS_LIST_ALTERNATIVES_CONNECT_PATH_V1, DECISIONS_LIST_CONNECT_PATH_V1,
    DECISIONS_LIST_EVIDENCE_CONNECT_PATH_V1, DECISIONS_MODULE_ID_V1, DECISIONS_OWNER_ID_V1,
    DECISIONS_REMOVE_ALTERNATIVE_CONNECT_PATH_V1, DECISIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1,
    DECISIONS_SUPERSEDE_CONNECT_PATH_V1, DECISIONS_UPDATE_ALTERNATIVE_CONNECT_PATH_V1,
    DECISIONS_UPDATE_CONNECT_PATH_V1, DecisionsEnvelopeContextV1,
    build_decision_changed_outbox_record_v1,
    client_wire::{
        AddDecisionAlternativeRequestV1, AddDecisionEvidenceRequestV1, CancelDecisionRequestV1,
        CreateDecisionRequestV1, DecideRequestV1,
        DecisionAlternativeStateV1 as WireAlternativeState,
        DecisionAlternativeV1 as WireAlternative, DecisionChangedV1,
        DecisionEvidenceLinkV1 as WireEvidence, DecisionMutationResultV1,
        DecisionStateV1 as WireDecisionState, DecisionV1 as WireDecision, GetDecisionRequestV1,
        ListDecisionAlternativesRequestV1, ListDecisionAlternativesResultV1,
        ListDecisionEvidenceRequestV1, ListDecisionEvidenceResultV1, ListDecisionsRequestV1,
        ListDecisionsResultV1, RemoveDecisionAlternativeRequestV1, RemoveDecisionEvidenceRequestV1,
        SupersedeDecisionRequestV1, TimestampV1, UpdateDecisionAlternativeRequestV1,
        UpdateDecisionRequestV1,
    },
    decisions_client_routes_v1,
};
use makosh_decisions_core::{
    DecisionAlternativeStateV1, DecisionAlternativeV1, DecisionEvidenceLinkV1, DecisionRecordV1,
    DecisionStateV1, DecisionTimestampV1,
};
use makosh_decisions_persistence::{
    DecisionLifecycleCommitV1, DecisionLifecycleMutationV1, DecisionLifecycleOperationOutcomeV1,
    DecisionLifecycleOperationV1, DecisionOutboxRecordV1, DecisionsPersistenceErrorV1,
    DecisionsPersistenceV1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

const PAGE_LIMIT_MAX_V1: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecisionsClientRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub async fn dispatch_decisions_client_request_v1(
    persistence: &DecisionsPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    context: DecisionsClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == DECISIONS_MODULE_ID_V1
        && request.owner_id == DECISIONS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && context.runtime_instance_id.iter().any(|byte| *byte != 0)
        && context.runtime_generation > 0
        && context.now_unix_millis > 0;
    let response = if accepted {
        dispatch(persistence, logical_owner_id, &request, context).await
    } else {
        Err("REJECTED")
    };
    match response {
        Ok(response_payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload,
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
    persistence: &DecisionsPersistenceV1,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    context: DecisionsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if exact_contract(contract, DECISIONS_GET_CONNECT_PATH_V1) {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if exact_contract(contract, DECISIONS_LIST_CONNECT_PATH_V1) {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }
    if exact_contract(contract, DECISIONS_LIST_ALTERNATIVES_CONNECT_PATH_V1) {
        return list_alternatives(persistence, logical_owner_id, &request.request_payload).await;
    }
    if exact_contract(contract, DECISIONS_LIST_EVIDENCE_CONNECT_PATH_V1) {
        return list_evidence(persistence, logical_owner_id, &request.request_payload).await;
    }
    let operation_id = decode_operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_operation_replay(
            logical_owner_id,
            operation_id,
            request_sha256,
            &request.request_payload,
        )
        .await
        .map_err(persistence_error)?
    {
        return Ok(response);
    }
    let mutation = decode_mutation(
        contract,
        logical_owner_id,
        &request.request_payload,
        context.now_unix_millis,
    )?;
    let operation = DecisionLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: context.now_unix_millis,
        mutation,
    };
    let envelope_context = DecisionsEnvelopeContextV1 {
        module_id: DECISIONS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: hex16(context.runtime_instance_id),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_lifecycle_operation(operation, |decision| {
            build_commit(operation_id, decision, &envelope_context)
        })
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        DecisionLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | DecisionLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn build_commit(
    operation_id: [u8; 16],
    decision: &DecisionRecordV1,
    context: &DecisionsEnvelopeContextV1,
) -> Result<DecisionLifecycleCommitV1, DecisionsPersistenceErrorV1> {
    let response = DecisionMutationResultV1 {
        operation_id: operation_id.to_vec(),
        decision: Some(wire_decision(decision)),
    }
    .encode_to_vec();
    let changed = build_decision_changed_outbox_record_v1(
        operation_id,
        DecisionChangedV1 {
            event_id: lifecycle_event_id(operation_id, decision).to_vec(),
            decision_id: decision.decision_id.to_vec(),
            logical_owner_id: decision.logical_owner_id.clone(),
            decision_revision: decision.decision_revision,
            state: encode_state(decision.state),
            occurred_at: Some(wire_timestamp(decision.updated_at)),
        },
        context,
    )
    .map_err(|_| DecisionsPersistenceErrorV1::InvalidInput)?;
    Ok(DecisionLifecycleCommitV1 {
        response_sha256: Sha256::digest(&response).into(),
        response_bytes: response,
        lifecycle_event: DecisionOutboxRecordV1 {
            message_id: *changed.message_id(),
            envelope_sha256: *changed.envelope_sha256(),
            envelope_bytes: changed.exact_bytes().to_vec(),
        },
    })
}

fn decode_operation_id(
    contract: &ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! operation_id {
        ($path:expr, $type:ty) => {
            if exact_contract(contract, $path) {
                return id16(&exact_decode::<$type>(bytes)?.operation_id);
            }
        };
    }
    operation_id!(DECISIONS_CREATE_CONNECT_PATH_V1, CreateDecisionRequestV1);
    operation_id!(DECISIONS_UPDATE_CONNECT_PATH_V1, UpdateDecisionRequestV1);
    operation_id!(
        DECISIONS_ADD_ALTERNATIVE_CONNECT_PATH_V1,
        AddDecisionAlternativeRequestV1
    );
    operation_id!(
        DECISIONS_UPDATE_ALTERNATIVE_CONNECT_PATH_V1,
        UpdateDecisionAlternativeRequestV1
    );
    operation_id!(
        DECISIONS_REMOVE_ALTERNATIVE_CONNECT_PATH_V1,
        RemoveDecisionAlternativeRequestV1
    );
    operation_id!(
        DECISIONS_ADD_EVIDENCE_CONNECT_PATH_V1,
        AddDecisionEvidenceRequestV1
    );
    operation_id!(
        DECISIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1,
        RemoveDecisionEvidenceRequestV1
    );
    operation_id!(DECISIONS_DECIDE_CONNECT_PATH_V1, DecideRequestV1);
    operation_id!(
        DECISIONS_SUPERSEDE_CONNECT_PATH_V1,
        SupersedeDecisionRequestV1
    );
    operation_id!(DECISIONS_CANCEL_CONNECT_PATH_V1, CancelDecisionRequestV1);
    Err("REJECTED")
}

fn decode_mutation(
    contract: &ContractReferenceV1,
    owner: &str,
    bytes: &[u8],
    now_unix_millis: i64,
) -> Result<DecisionLifecycleMutationV1, &'static str> {
    if exact_contract(contract, DECISIONS_CREATE_CONNECT_PATH_V1) {
        let value = exact_decode::<CreateDecisionRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::Create {
            owner: owner.to_owned(),
            operation_id: id16(&value.operation_id)?,
            title: value.title,
            question: value.question,
            created_at: timestamp(value.created_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_UPDATE_CONNECT_PATH_V1) {
        let value = exact_decode::<UpdateDecisionRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::Update {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            title: value.title,
            question: value.question,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_ADD_ALTERNATIVE_CONNECT_PATH_V1) {
        let value = exact_decode::<AddDecisionAlternativeRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::AddAlternative {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            operation_id: id16(&value.operation_id)?,
            title: value.title,
            description: value.description,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_UPDATE_ALTERNATIVE_CONNECT_PATH_V1) {
        let value = exact_decode::<UpdateDecisionAlternativeRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::UpdateAlternative {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            alternative_id: id16(&value.alternative_id)?,
            expected_alternative_revision: nonzero_revision(value.expected_alternative_revision)?,
            title: value.title,
            description: value.description,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_REMOVE_ALTERNATIVE_CONNECT_PATH_V1) {
        let value = exact_decode::<RemoveDecisionAlternativeRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::RemoveAlternative {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            alternative_id: id16(&value.alternative_id)?,
            expected_alternative_revision: nonzero_revision(value.expected_alternative_revision)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_ADD_EVIDENCE_CONNECT_PATH_V1) {
        let value = exact_decode::<AddDecisionEvidenceRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::AddEvidence {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            evidence: core_evidence(value.evidence.as_ref().ok_or("REJECTED")?)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1) {
        let value = exact_decode::<RemoveDecisionEvidenceRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::RemoveEvidence {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            evidence_link_id: id16(&value.evidence_link_id)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_DECIDE_CONNECT_PATH_V1) {
        let value = exact_decode::<DecideRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::Decide {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            selected_alternative_id: id16(&value.selected_alternative_id)?,
            rationale: value.rationale,
            changed_at: timestamp(value.decided_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_SUPERSEDE_CONNECT_PATH_V1) {
        let value = exact_decode::<SupersedeDecisionRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::Supersede {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            replacement_decision_id: id16(&value.replacement_decision_id)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    if exact_contract(contract, DECISIONS_CANCEL_CONNECT_PATH_V1) {
        let value = exact_decode::<CancelDecisionRequestV1>(bytes)?;
        accepted_payload_owner(owner, &value.logical_owner_id)?;
        return Ok(DecisionLifecycleMutationV1::Cancel {
            decision_id: id16(&value.decision_id)?,
            expected_revision: nonzero_revision(value.expected_decision_revision)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_unix_millis)?,
        });
    }
    Err("REJECTED")
}

async fn get(
    persistence: &DecisionsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = exact_decode::<GetDecisionRequestV1>(bytes)?;
    accepted_payload_owner(owner, &value.logical_owner_id)?;
    persistence
        .get_decision(owner, id16(&value.decision_id)?)
        .await
        .map_err(persistence_error)?
        .map(|value| wire_decision(&value).encode_to_vec())
        .ok_or("NOT_FOUND")
}

async fn list(
    persistence: &DecisionsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = exact_decode::<ListDecisionsRequestV1>(bytes)?;
    accepted_payload_owner(owner, &value.logical_owner_id)?;
    let limit = page_limit(value.limit)?;
    let mut rows = persistence
        .list_decisions(owner, optional_id16(&value.after_decision_id)?, limit + 1)
        .await
        .map_err(persistence_error)?;
    let has_more = rows.len() > usize::from(limit);
    if has_more {
        rows.pop();
    }
    let cursor = if has_more {
        rows.last().expect("nonempty page").decision_id.to_vec()
    } else {
        Vec::new()
    };
    Ok(ListDecisionsResultV1 {
        decisions: rows.iter().map(wire_decision).collect(),
        next_after_decision_id: cursor,
    }
    .encode_to_vec())
}

async fn list_alternatives(
    persistence: &DecisionsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = exact_decode::<ListDecisionAlternativesRequestV1>(bytes)?;
    accepted_payload_owner(owner, &value.logical_owner_id)?;
    let limit = page_limit(value.limit)?;
    let after = optional_id16(&value.after_alternative_id)?;
    let decision = persistence
        .get_decision(owner, id16(&value.decision_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    let mut rows: Vec<_> = decision
        .alternatives
        .into_iter()
        .filter(|item| after.is_none_or(|cursor| item.alternative_id > cursor))
        .take(usize::from(limit) + 1)
        .collect();
    let has_more = rows.len() > usize::from(limit);
    if has_more {
        rows.pop();
    }
    let cursor = if has_more {
        rows.last().expect("nonempty page").alternative_id.to_vec()
    } else {
        Vec::new()
    };
    Ok(ListDecisionAlternativesResultV1 {
        alternatives: rows.iter().map(wire_alternative).collect(),
        next_after_alternative_id: cursor,
    }
    .encode_to_vec())
}

async fn list_evidence(
    persistence: &DecisionsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = exact_decode::<ListDecisionEvidenceRequestV1>(bytes)?;
    accepted_payload_owner(owner, &value.logical_owner_id)?;
    let limit = page_limit(value.limit)?;
    let after = optional_id16(&value.after_evidence_link_id)?;
    let decision = persistence
        .get_decision(owner, id16(&value.decision_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    let mut rows: Vec<_> = decision
        .evidence
        .into_iter()
        .filter(|item| after.is_none_or(|cursor| item.evidence_link_id > cursor))
        .take(usize::from(limit) + 1)
        .collect();
    let has_more = rows.len() > usize::from(limit);
    if has_more {
        rows.pop();
    }
    let cursor = if has_more {
        rows.last()
            .expect("nonempty page")
            .evidence_link_id
            .to_vec()
    } else {
        Vec::new()
    };
    Ok(ListDecisionEvidenceResultV1 {
        evidence_links: rows.iter().map(wire_evidence).collect(),
        next_after_evidence_link_id: cursor,
    }
    .encode_to_vec())
}

fn exact_contract(contract: &ContractReferenceV1, path: &str) -> bool {
    decisions_client_routes_v1()
        .into_iter()
        .find(|(_, candidate)| *candidate == path)
        .is_some_and(|(expected, _)| &expected == contract)
}

fn wire_decision(value: &DecisionRecordV1) -> WireDecision {
    WireDecision {
        decision_id: value.decision_id.to_vec(),
        logical_owner_id: value.logical_owner_id.clone(),
        title: value.title.clone(),
        question: value.question.clone(),
        rationale: value.rationale.clone(),
        state: encode_state(value.state),
        selected_alternative_id: value.selected_alternative_id.map(|id| id.to_vec()),
        superseded_by_decision_id: value.superseded_by_decision_id.map(|id| id.to_vec()),
        decision_revision: value.decision_revision,
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}

fn wire_alternative(value: &DecisionAlternativeV1) -> WireAlternative {
    WireAlternative {
        alternative_id: value.alternative_id.to_vec(),
        decision_id: value.decision_id.to_vec(),
        title: value.title.clone(),
        description: value.description.clone(),
        state: match value.state {
            DecisionAlternativeStateV1::Candidate => {
                WireAlternativeState::DecisionAlternativeStateCandidate as i32
            }
            DecisionAlternativeStateV1::Selected => {
                WireAlternativeState::DecisionAlternativeStateSelected as i32
            }
            DecisionAlternativeStateV1::Rejected => {
                WireAlternativeState::DecisionAlternativeStateRejected as i32
            }
        },
        alternative_revision: value.alternative_revision,
        updated_at_decision_revision: value.updated_at_decision_revision,
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}

fn wire_evidence(value: &DecisionEvidenceLinkV1) -> WireEvidence {
    WireEvidence {
        evidence_link_id: value.evidence_link_id.to_vec(),
        evidence_owner_id: value.evidence_owner_id.clone(),
        evidence_record_id: value.evidence_record_id.to_vec(),
        evidence_revision: value.evidence_revision,
        evidence_digest: value.evidence_digest.to_vec(),
    }
}

fn core_evidence(value: &WireEvidence) -> Result<DecisionEvidenceLinkV1, &'static str> {
    Ok(DecisionEvidenceLinkV1 {
        evidence_link_id: id16(&value.evidence_link_id)?,
        evidence_owner_id: value.evidence_owner_id.clone(),
        evidence_record_id: id16(&value.evidence_record_id)?,
        evidence_revision: nonzero_revision(value.evidence_revision)?,
        evidence_digest: value
            .evidence_digest
            .as_slice()
            .try_into()
            .map_err(|_| "REJECTED")?,
    })
}

fn encode_state(value: DecisionStateV1) -> i32 {
    match value {
        DecisionStateV1::Draft => WireDecisionState::DecisionStateDraft as i32,
        DecisionStateV1::Decided => WireDecisionState::DecisionStateDecided as i32,
        DecisionStateV1::Superseded => WireDecisionState::DecisionStateSuperseded as i32,
        DecisionStateV1::Cancelled => WireDecisionState::DecisionStateCancelled as i32,
    }
}
fn wire_timestamp(value: DecisionTimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}
fn timestamp(
    value: Option<&TimestampV1>,
    now_millis: i64,
) -> Result<DecisionTimestampV1, &'static str> {
    let value = value.ok_or("REJECTED")?;
    let millis = value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|base| base.checked_add(i64::from(value.nanos) / 1_000_000))
        .ok_or("REJECTED")?;
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) || millis > now_millis
    {
        return Err("REJECTED");
    }
    Ok(DecisionTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}
fn exact_decode<T: Message + Default>(bytes: &[u8]) -> Result<T, &'static str> {
    let value = T::decode(bytes).map_err(|_| "REJECTED")?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or("REJECTED")
}
fn accepted_payload_owner(outer: &str, payload: &str) -> Result<(), &'static str> {
    (payload.is_empty() || payload == outer)
        .then_some(())
        .ok_or("REJECTED")
}
fn id16(bytes: &[u8]) -> Result<[u8; 16], &'static str> {
    let value: [u8; 16] = bytes.try_into().map_err(|_| "REJECTED")?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or("REJECTED")
}
fn optional_id16(bytes: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        id16(bytes).map(Some)
    }
}
fn nonzero_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("REJECTED")
}
fn page_limit(value: u32) -> Result<u16, &'static str> {
    let value = if value == 0 { 50 } else { value };
    (value <= PAGE_LIMIT_MAX_V1)
        .then_some(value as u16)
        .ok_or("REJECTED")
}
fn lifecycle_event_id(operation_id: [u8; 16], decision: &DecisionRecordV1) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.decisions.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(decision.decision_id);
    hash.update(decision.decision_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed")
}
fn hex16(value: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(32);
    for byte in value {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
fn persistence_error(value: DecisionsPersistenceErrorV1) -> &'static str {
    match value {
        DecisionsPersistenceErrorV1::NotFound => "NOT_FOUND",
        DecisionsPersistenceErrorV1::RevisionConflict => "REVISION_CONFLICT",
        DecisionsPersistenceErrorV1::StateConflict => "STATE_CONFLICT",
        DecisionsPersistenceErrorV1::OperationConflict
        | DecisionsPersistenceErrorV1::OutboxConflict => "CONFLICT",
        DecisionsPersistenceErrorV1::InvalidInput => "REJECTED",
        DecisionsPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_and_cursor_boundaries_are_exact() {
        assert!(accepted_payload_owner("owner-1", "").is_ok());
        assert!(accepted_payload_owner("owner-1", "owner-1").is_ok());
        assert!(accepted_payload_owner("owner-1", "owner-2").is_err());
        assert_eq!(page_limit(0), Ok(50));
        assert!(page_limit(101).is_err());
        assert_eq!(
            persistence_error(DecisionsPersistenceErrorV1::OperationConflict),
            "CONFLICT"
        );
        assert_eq!(
            persistence_error(DecisionsPersistenceErrorV1::OutboxConflict),
            "CONFLICT"
        );
    }
}
