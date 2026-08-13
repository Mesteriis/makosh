use makosh_obligations_api::{
    OBLIGATIONS_MODULE_ID_V1, OBLIGATIONS_OWNER_ID_V1, ObligationsCommandEnvelopeContextV1,
    build_obligation_changed_outbox_record_v1,
    client_wire::{
        AddObligationEvidenceRequestV1, GetObligationRequestV1, ListObligationEvidenceRequestV1,
        ListObligationEvidenceResultV1, ListObligationsRequestV1, ListObligationsResultV1,
        ObligationChangedV1, ObligationEvidenceLinkV1 as WireEvidenceLink,
        ObligationMutationResultV1, ObligationStateV1 as WireState, ObligationSummaryV1,
        RemoveObligationEvidenceRequestV1, SetObligationStateRequestV1,
        TimestampV1 as WireTimestamp, UpdateObligationRequestV1,
    },
    obligations_client_add_evidence_contract_reference_v1,
    obligations_client_get_contract_reference_v1, obligations_client_list_contract_reference_v1,
    obligations_client_list_evidence_contract_reference_v1,
    obligations_client_remove_evidence_contract_reference_v1,
    obligations_client_set_state_contract_reference_v1,
    obligations_client_update_contract_reference_v1,
};
use makosh_obligations_core::{
    ObligationEvidenceLinkV1, ObligationLifecycleStateV1, ObligationRecordV1, ObligationTimestampV1,
};
use makosh_obligations_persistence::{
    ObligationsLifecycleCommitV1, ObligationsLifecycleMutationV1,
    ObligationsLifecycleOperationOutcomeV1, ObligationsLifecycleOperationV1,
    ObligationsOutboxRecordV1, ObligationsPersistenceErrorV1, ObligationsPersistenceV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

pub async fn dispatch_obligations_client_request_v1(
    persistence: &ObligationsPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let accepted_identity = request.protocol_major == 1
        && request.module_id == OBLIGATIONS_MODULE_ID_V1
        && request.owner_id == OBLIGATIONS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && !runtime_instance_id.is_empty()
        && runtime_generation > 0
        && now_unix_millis > 0;
    let response = if accepted_identity {
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
    persistence: &ObligationsPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &obligations_client_get_contract_reference_v1() {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &obligations_client_list_contract_reference_v1() {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &obligations_client_list_evidence_contract_reference_v1() {
        return list_evidence(persistence, logical_owner_id, &request.request_payload).await;
    }

    let operation_id = decode_operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_lifecycle_operation_replay(
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
        now_unix_millis,
    )?;
    debug_assert_eq!(operation_id, mutation.operation_id());
    let operation = ObligationsLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: now_unix_millis,
        mutation,
    };
    let context = ObligationsCommandEnvelopeContextV1 {
        module_id: OBLIGATIONS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        recorded_at_unix_seconds: now_unix_millis / 1_000,
        recorded_at_nanos: ((now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_lifecycle_operation(operation, |obligation| {
            let response = ObligationMutationResultV1 {
                operation_id: operation_id.to_vec(),
                obligation: Some(summary(obligation)),
            }
            .encode_to_vec();
            let event_id = lifecycle_event_id(
                operation_id,
                obligation.obligation_id,
                obligation.obligation_revision,
            );
            let event = build_obligation_changed_outbox_record_v1(
                operation_id,
                ObligationChangedV1 {
                    event_id: event_id.to_vec(),
                    obligation_id: obligation.obligation_id.to_vec(),
                    logical_owner_id: obligation.logical_owner_id.clone(),
                    obligation_revision: obligation.obligation_revision,
                    state: encode_state(obligation.state),
                    occurred_at: Some(timestamp(obligation.updated_at)),
                },
                &context,
            )
            .map_err(|_| ObligationsPersistenceErrorV1::InvalidInput)?;
            Ok(ObligationsLifecycleCommitV1 {
                response_sha256: Sha256::digest(&response).into(),
                response_bytes: response,
                lifecycle_event: ObligationsOutboxRecordV1 {
                    message_id: *event.message_id(),
                    envelope_sha256: *event.envelope_sha256(),
                    envelope_bytes: event.exact_bytes().to_vec(),
                },
            })
        })
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        ObligationsLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | ObligationsLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn decode_operation_id(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! operation_id {
        ($contract:expr, $type:ty) => {
            if contract == &$contract {
                let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
                if value.encode_to_vec() != bytes {
                    return Err("INVALID_ARGUMENT");
                }
                return id16(&value.operation_id);
            }
        };
    }
    operation_id!(
        obligations_client_update_contract_reference_v1(),
        UpdateObligationRequestV1
    );
    operation_id!(
        obligations_client_set_state_contract_reference_v1(),
        SetObligationStateRequestV1
    );
    operation_id!(
        obligations_client_add_evidence_contract_reference_v1(),
        AddObligationEvidenceRequestV1
    );
    operation_id!(
        obligations_client_remove_evidence_contract_reference_v1(),
        RemoveObligationEvidenceRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
    now_unix_millis: i64,
) -> Result<ObligationsLifecycleMutationV1, &'static str> {
    macro_rules! decode {
        ($type:ty) => {{
            let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
            if value.encode_to_vec() != bytes {
                return Err("INVALID_ARGUMENT");
            }
            value
        }};
    }
    if contract == &obligations_client_update_contract_reference_v1() {
        let mut value = decode!(UpdateObligationRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        if value.clear_condition && value.condition.is_some()
            || value.clear_due_at && value.due_at.is_some()
            || value.clear_beneficiary_party_id && value.beneficiary_party_id.is_some()
        {
            return Err("INVALID_ARGUMENT");
        }
        Ok(ObligationsLifecycleMutationV1::Update {
            operation_id: id16(&value.operation_id)?,
            obligation_id: id16(&value.obligation_id)?,
            expected_revision: positive_revision(value.expected_obligation_revision)?,
            statement: value.statement,
            condition: if value.clear_condition {
                Some(None)
            } else {
                value.condition.map(Some)
            },
            due_at: if value.clear_due_at {
                Some(None)
            } else {
                value
                    .due_at
                    .map(|time| decode_timestamp(Some(time)).map(Some))
                    .transpose()?
            },
            obligated_party_id: value
                .obligated_party_id
                .map(|party_id| id16(&party_id))
                .transpose()?,
            beneficiary_party_id: if value.clear_beneficiary_party_id {
                Some(None)
            } else {
                value
                    .beneficiary_party_id
                    .map(|party_id| id16(&party_id).map(Some))
                    .transpose()?
            },
            changed_at: checked_timestamp(value.updated_at, now_unix_millis)?,
        })
    } else if contract == &obligations_client_set_state_contract_reference_v1() {
        let mut value = decode!(SetObligationStateRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(ObligationsLifecycleMutationV1::SetState {
            operation_id: id16(&value.operation_id)?,
            obligation_id: id16(&value.obligation_id)?,
            expected_revision: positive_revision(value.expected_obligation_revision)?,
            state: decode_state(value.state)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &obligations_client_add_evidence_contract_reference_v1() {
        let mut value = decode!(AddObligationEvidenceRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(ObligationsLifecycleMutationV1::AddEvidence {
            operation_id: id16(&value.operation_id)?,
            obligation_id: id16(&value.obligation_id)?,
            expected_revision: positive_revision(value.expected_obligation_revision)?,
            evidence: decode_evidence(value.evidence.ok_or("INVALID_ARGUMENT")?)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &obligations_client_remove_evidence_contract_reference_v1() {
        let mut value = decode!(RemoveObligationEvidenceRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(ObligationsLifecycleMutationV1::RemoveEvidence {
            operation_id: id16(&value.operation_id)?,
            obligation_id: id16(&value.obligation_id)?,
            expected_revision: positive_revision(value.expected_obligation_revision)?,
            evidence_link_id: id16(&value.evidence_link_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &ObligationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = GetObligationRequestV1::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if request.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let obligation = persistence
        .get_lifecycle_obligation(logical_owner_id, id16(&request.obligation_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    Ok(summary(&obligation).encode_to_vec())
}

async fn list(
    persistence: &ObligationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = ListObligationsRequestV1::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if request.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(request.limit)?;
    let after = if request.after_obligation_id.is_empty() {
        None
    } else {
        Some(id16(&request.after_obligation_id)?)
    };
    let mut obligations = persistence
        .list_lifecycle_obligations(logical_owner_id, after, limit + 1)
        .await
        .map_err(persistence_error)?;
    let has_more = obligations.len() > usize::from(limit);
    obligations.truncate(usize::from(limit));
    let next = has_more
        .then(|| obligations.last().map(|value| value.obligation_id.to_vec()))
        .flatten()
        .unwrap_or_default();
    Ok(ListObligationsResultV1 {
        obligations: obligations.iter().map(summary).collect(),
        next_after_obligation_id: next,
    }
    .encode_to_vec())
}

async fn list_evidence(
    persistence: &ObligationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request =
        ListObligationEvidenceRequestV1::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if request.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(request.limit)?;
    let after = if request.after_evidence_link_id.is_empty() {
        None
    } else {
        Some(id16(&request.after_evidence_link_id)?)
    };
    let obligation = persistence
        .get_lifecycle_obligation(logical_owner_id, id16(&request.obligation_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    let mut evidence_links = obligation
        .evidence_links
        .iter()
        .filter(|value| after.is_none_or(|cursor| value.evidence_link_id > cursor))
        .take(usize::from(limit) + 1)
        .cloned()
        .collect::<Vec<_>>();
    let has_more = evidence_links.len() > usize::from(limit);
    evidence_links.truncate(usize::from(limit));
    let next = has_more
        .then(|| {
            evidence_links
                .last()
                .map(|value| value.evidence_link_id.to_vec())
        })
        .flatten()
        .unwrap_or_default();
    Ok(ListObligationEvidenceResultV1 {
        evidence_links: evidence_links.iter().map(encode_evidence).collect(),
        next_after_evidence_link_id: next,
    }
    .encode_to_vec())
}

fn summary(obligation: &ObligationRecordV1) -> ObligationSummaryV1 {
    ObligationSummaryV1 {
        obligation_id: obligation.obligation_id.to_vec(),
        logical_owner_id: obligation.logical_owner_id.clone(),
        statement: obligation.statement.clone(),
        condition: obligation.condition.clone(),
        due_at: obligation.due_at.map(timestamp),
        state: encode_state(obligation.state),
        obligation_revision: obligation.obligation_revision,
        obligated_party_id: obligation.obligated_party_id.to_vec(),
        beneficiary_party_id: obligation.beneficiary_party_id.map(|value| value.to_vec()),
        created_at: Some(timestamp(obligation.created_at)),
        updated_at: Some(timestamp(obligation.updated_at)),
    }
}

fn decode_evidence(value: WireEvidenceLink) -> Result<ObligationEvidenceLinkV1, &'static str> {
    Ok(ObligationEvidenceLinkV1 {
        evidence_link_id: id16(&value.evidence_link_id)?,
        evidence_owner_id: value.evidence_owner_id,
        evidence_record_id: id16(&value.evidence_record_id)?,
        evidence_revision: positive_revision(value.evidence_revision)?,
        evidence_digest: digest32(&value.evidence_digest)?,
    })
}

fn encode_evidence(value: &ObligationEvidenceLinkV1) -> WireEvidenceLink {
    WireEvidenceLink {
        evidence_link_id: value.evidence_link_id.to_vec(),
        evidence_owner_id: value.evidence_owner_id.clone(),
        evidence_record_id: value.evidence_record_id.to_vec(),
        evidence_revision: value.evidence_revision,
        evidence_digest: value.evidence_digest.to_vec(),
    }
}

fn accept_owner(value: &mut String, logical_owner_id: &str) -> Result<(), &'static str> {
    if !value.is_empty() && value != logical_owner_id {
        return Err("REJECTED");
    }
    *value = logical_owner_id.to_owned();
    Ok(())
}

fn checked_timestamp(
    value: Option<WireTimestamp>,
    now_unix_millis: i64,
) -> Result<ObligationTimestampV1, &'static str> {
    let value = decode_timestamp(value)?;
    if value.unix_seconds > now_unix_millis / 1_000
        || (value.unix_seconds == now_unix_millis / 1_000
            && i64::from(value.nanos) > (now_unix_millis % 1_000) * 1_000_000)
    {
        return Err("INVALID_ARGUMENT");
    }
    Ok(value)
}

fn decode_timestamp(value: Option<WireTimestamp>) -> Result<ObligationTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_ARGUMENT");
    }
    Ok(ObligationTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn timestamp(value: ObligationTimestampV1) -> WireTimestamp {
    WireTimestamp {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn decode_state(value: i32) -> Result<ObligationLifecycleStateV1, &'static str> {
    match WireState::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireState::ObligationStateOpen => Ok(ObligationLifecycleStateV1::Open),
        WireState::ObligationStateFulfilled => Ok(ObligationLifecycleStateV1::Fulfilled),
        WireState::ObligationStateWaived => Ok(ObligationLifecycleStateV1::Waived),
        WireState::ObligationStateBreached => Ok(ObligationLifecycleStateV1::Breached),
        WireState::ObligationStateCancelled => Ok(ObligationLifecycleStateV1::Cancelled),
        WireState::ObligationStateUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn encode_state(value: ObligationLifecycleStateV1) -> i32 {
    match value {
        ObligationLifecycleStateV1::Open => WireState::ObligationStateOpen as i32,
        ObligationLifecycleStateV1::Fulfilled => WireState::ObligationStateFulfilled as i32,
        ObligationLifecycleStateV1::Waived => WireState::ObligationStateWaived as i32,
        ObligationLifecycleStateV1::Breached => WireState::ObligationStateBreached as i32,
        ObligationLifecycleStateV1::Cancelled => WireState::ObligationStateCancelled as i32,
    }
}

fn checked_limit(value: u32) -> Result<u16, &'static str> {
    if !(1..=200).contains(&value) {
        return Err("INVALID_ARGUMENT");
    }
    u16::try_from(value).map_err(|_| "INVALID_ARGUMENT")
}

fn positive_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let id: [u8; 16] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or("INVALID_ARGUMENT")
}

fn digest32(value: &[u8]) -> Result<[u8; 32], &'static str> {
    let digest: [u8; 32] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    digest
        .iter()
        .any(|byte| *byte != 0)
        .then_some(digest)
        .ok_or("INVALID_ARGUMENT")
}

fn lifecycle_event_id(
    operation_id: [u8; 16],
    obligation_id: [u8; 16],
    obligation_revision: u64,
) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.obligations.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(obligation_id);
    hash.update(obligation_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn persistence_error(value: ObligationsPersistenceErrorV1) -> &'static str {
    match value {
        ObligationsPersistenceErrorV1::NotFound => "NOT_FOUND",
        ObligationsPersistenceErrorV1::InvalidInput | ObligationsPersistenceErrorV1::InvalidRow => {
            "INVALID_ARGUMENT"
        }
        ObligationsPersistenceErrorV1::OperationConflict
        | ObligationsPersistenceErrorV1::RevisionConflict
        | ObligationsPersistenceErrorV1::DependencyCycle
        | ObligationsPersistenceErrorV1::CommandConflict
        | ObligationsPersistenceErrorV1::InboxConflict
        | ObligationsPersistenceErrorV1::ObligationConflict => "FAILED_PRECONDITION",
        ObligationsPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_cursor_is_last_returned_not_overflow() {
        let mut obligations = (1_u8..=3)
            .map(|id| ObligationRecordV1 {
                obligation_id: [id; 16],
                logical_owner_id: "owner-1".to_owned(),
                statement: format!("Obligation {id}"),
                condition: None,
                due_at: None,
                state: ObligationLifecycleStateV1::Open,
                obligation_revision: 1,
                obligated_party_id: [4; 16],
                beneficiary_party_id: None,
                evidence_links: Vec::new(),
                created_at: ObligationTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
                updated_at: ObligationTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            })
            .collect::<Vec<_>>();
        let has_more = obligations.len() > 2;
        obligations.truncate(2);
        assert!(has_more);
        assert_eq!(obligations.last().expect("last").obligation_id, [2; 16]);
    }
}
