use makosh_relationships_api::{
    RELATIONSHIPS_MODULE_ID_V1, RELATIONSHIPS_OWNER_ID_V1, RelationshipsEnvelopeContextV1,
    build_relationship_changed_outbox_record_v1,
    client_wire::{
        AddRelationshipEvidenceRequestV1, CreateRelationshipRequestV1, EndRelationshipRequestV1,
        GetRelationshipRequestV1, ListRelationshipEvidenceRequestV1,
        ListRelationshipEvidenceResultV1, ListRelationshipsForParticipantRequestV1,
        ListRelationshipsResultV1, ReactivateRelationshipRequestV1, RelationshipChangedV1,
        RelationshipEvidenceStateV1 as WireEvidenceState, RelationshipEvidenceV1 as WireEvidence,
        RelationshipMutationResultV1, RelationshipParticipantKindV1 as WireParticipantKind,
        RelationshipParticipantV1 as WireParticipant, RelationshipStateV1 as WireState,
        RelationshipTypeV1 as WireType, RelationshipV1 as WireRelationship,
        RemoveRelationshipEvidenceRequestV1, TimestampV1, UpdateRelationshipValidityRequestV1,
    },
    relationships_client_add_evidence_contract_reference_v1,
    relationships_client_create_contract_reference_v1,
    relationships_client_end_contract_reference_v1, relationships_client_get_contract_reference_v1,
    relationships_client_list_evidence_contract_reference_v1,
    relationships_client_list_for_participant_contract_reference_v1,
    relationships_client_reactivate_contract_reference_v1,
    relationships_client_remove_evidence_contract_reference_v1,
    relationships_client_update_validity_contract_reference_v1,
};
use makosh_relationships_core::{
    RelationshipEvidenceStateV1, RelationshipEvidenceV1, RelationshipParticipantKindV1,
    RelationshipParticipantV1, RelationshipRecordV1, RelationshipStateV1, RelationshipTimestampV1,
    RelationshipTypeV1,
};
use makosh_relationships_persistence::{
    RelationshipCommitV1, RelationshipMutationV1, RelationshipOperationOutcomeV1,
    RelationshipOperationV1, RelationshipOutboxRecordV1, RelationshipsPersistenceErrorV1,
    RelationshipsPersistenceV1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationshipsClientRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub async fn dispatch_relationships_client_request_v1(
    persistence: &RelationshipsPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    context: RelationshipsClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == RELATIONSHIPS_MODULE_ID_V1
        && request.owner_id == RELATIONSHIPS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && context.runtime_instance_id.iter().any(|byte| *byte != 0)
        && context.runtime_generation > 0
        && context.now_unix_millis > 0;
    let result = if accepted {
        dispatch(persistence, logical_owner_id, &request, context).await
    } else {
        Err("REJECTED")
    };
    match result {
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
    persistence: &RelationshipsPersistenceV1,
    owner: &str,
    request: &ModuleClientRequestV1,
    context: RelationshipsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &relationships_client_get_contract_reference_v1() {
        return get(persistence, owner, &request.request_payload).await;
    }
    if contract == &relationships_client_list_for_participant_contract_reference_v1() {
        return list_for_participant(persistence, owner, &request.request_payload).await;
    }
    if contract == &relationships_client_list_evidence_contract_reference_v1() {
        return list_evidence(persistence, owner, &request.request_payload).await;
    }
    let operation_id = decode_operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_operation_replay(
            owner,
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
        owner,
        &request.request_payload,
        context.now_unix_millis,
    )?;
    let envelope_context = RelationshipsEnvelopeContextV1 {
        runtime_instance_id: hex(context.runtime_instance_id),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_operation(
            RelationshipOperationV1 {
                logical_owner_id: owner.to_owned(),
                operation_id,
                request_sha256,
                request_bytes: request.request_payload.clone(),
                received_at_unix_millis: context.now_unix_millis,
                mutation,
            },
            |relationship| build_commit(operation_id, relationship, &envelope_context),
        )
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        RelationshipOperationOutcomeV1::Applied { response_bytes }
        | RelationshipOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn build_commit(
    operation_id: [u8; 16],
    relationship: &RelationshipRecordV1,
    context: &RelationshipsEnvelopeContextV1,
) -> Result<RelationshipCommitV1, RelationshipsPersistenceErrorV1> {
    let response = RelationshipMutationResultV1 {
        operation_id: operation_id.to_vec(),
        relationship: Some(wire_relationship(relationship)),
    }
    .encode_to_vec();
    let event = build_relationship_changed_outbox_record_v1(
        operation_id,
        RelationshipChangedV1 {
            event_id: lifecycle_event_id(operation_id, relationship).to_vec(),
            relationship_id: relationship.relationship_id.to_vec(),
            logical_owner_id: relationship.logical_owner_id.clone(),
            source: Some(wire_participant(relationship.source)),
            target: Some(wire_participant(relationship.target)),
            relationship_type: encode_type(relationship.relationship_type),
            state: encode_state(relationship.state),
            valid_from: Some(wire_timestamp(relationship.valid_from)),
            valid_until: relationship.valid_until.map(wire_timestamp),
            relationship_revision: relationship.relationship_revision,
            occurred_at: Some(wire_timestamp(relationship.updated_at)),
        },
        context,
    )
    .map_err(|_| RelationshipsPersistenceErrorV1::InvalidInput)?;
    Ok(RelationshipCommitV1 {
        response_sha256: Sha256::digest(&response).into(),
        response_bytes: response,
        lifecycle_event: RelationshipOutboxRecordV1 {
            message_id: *event.message_id(),
            envelope_sha256: *event.envelope_sha256(),
            envelope_bytes: event.exact_bytes().to_vec(),
        },
    })
}

fn decode_operation_id(
    contract: &ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! extract {
        ($reference:expr, $type:ty) => {
            if contract == &$reference {
                return id16(&exact_decode::<$type>(bytes)?.operation_id);
            }
        };
    }
    extract!(
        relationships_client_create_contract_reference_v1(),
        CreateRelationshipRequestV1
    );
    extract!(
        relationships_client_update_validity_contract_reference_v1(),
        UpdateRelationshipValidityRequestV1
    );
    extract!(
        relationships_client_end_contract_reference_v1(),
        EndRelationshipRequestV1
    );
    extract!(
        relationships_client_reactivate_contract_reference_v1(),
        ReactivateRelationshipRequestV1
    );
    extract!(
        relationships_client_add_evidence_contract_reference_v1(),
        AddRelationshipEvidenceRequestV1
    );
    extract!(
        relationships_client_remove_evidence_contract_reference_v1(),
        RemoveRelationshipEvidenceRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &ContractReferenceV1,
    owner: &str,
    bytes: &[u8],
    now: i64,
) -> Result<RelationshipMutationV1, &'static str> {
    if contract == &relationships_client_create_contract_reference_v1() {
        let mut value = exact_decode::<CreateRelationshipRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        Ok(RelationshipMutationV1::Create {
            operation_id: id16(&value.operation_id)?,
            source: decode_participant(value.source)?,
            target: decode_participant(value.target)?,
            relationship_type: decode_type(value.relationship_type)?,
            valid_from: checked_timestamp(value.valid_from, now)?,
            valid_until: optional_timestamp(value.valid_until, now)?,
            evidence_source_owner_id: value.evidence_source_owner_id,
            evidence_source_record_id: value.evidence_source_record_id,
            evidence_source_revision: positive(value.evidence_source_revision)?,
            evidence_digest: id32(&value.evidence_digest)?,
            evidence_observed_at: checked_timestamp(value.evidence_observed_at, now)?,
            created_at: checked_timestamp(value.created_at, now)?,
        })
    } else if contract == &relationships_client_update_validity_contract_reference_v1() {
        let mut value = exact_decode::<UpdateRelationshipValidityRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        Ok(RelationshipMutationV1::UpdateValidity {
            operation_id: id16(&value.operation_id)?,
            relationship_id: id16(&value.relationship_id)?,
            expected_revision: positive(value.expected_relationship_revision)?,
            valid_from: checked_timestamp(value.valid_from, now)?,
            valid_until: optional_timestamp(value.valid_until, now)?,
            changed_at: checked_timestamp(value.changed_at, now)?,
        })
    } else if contract == &relationships_client_end_contract_reference_v1() {
        let mut value = exact_decode::<EndRelationshipRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        Ok(RelationshipMutationV1::End {
            operation_id: id16(&value.operation_id)?,
            relationship_id: id16(&value.relationship_id)?,
            expected_revision: positive(value.expected_relationship_revision)?,
            valid_until: checked_timestamp(value.valid_until, now)?,
            changed_at: checked_timestamp(value.changed_at, now)?,
        })
    } else if contract == &relationships_client_reactivate_contract_reference_v1() {
        let mut value = exact_decode::<ReactivateRelationshipRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        Ok(RelationshipMutationV1::Reactivate {
            operation_id: id16(&value.operation_id)?,
            relationship_id: id16(&value.relationship_id)?,
            expected_revision: positive(value.expected_relationship_revision)?,
            valid_from: checked_timestamp(value.valid_from, now)?,
            valid_until: optional_timestamp(value.valid_until, now)?,
            changed_at: checked_timestamp(value.changed_at, now)?,
        })
    } else if contract == &relationships_client_add_evidence_contract_reference_v1() {
        let mut value = exact_decode::<AddRelationshipEvidenceRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        Ok(RelationshipMutationV1::AddEvidence {
            operation_id: id16(&value.operation_id)?,
            relationship_id: id16(&value.relationship_id)?,
            expected_revision: positive(value.expected_relationship_revision)?,
            source_owner_id: value.source_owner_id,
            source_record_id: value.source_record_id,
            source_revision: positive(value.source_revision)?,
            evidence_digest: id32(&value.evidence_digest)?,
            observed_at: checked_timestamp(value.observed_at, now)?,
            changed_at: checked_timestamp(value.changed_at, now)?,
        })
    } else if contract == &relationships_client_remove_evidence_contract_reference_v1() {
        let mut value = exact_decode::<RemoveRelationshipEvidenceRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        Ok(RelationshipMutationV1::RemoveEvidence {
            operation_id: id16(&value.operation_id)?,
            relationship_id: id16(&value.relationship_id)?,
            expected_revision: positive(value.expected_relationship_revision)?,
            evidence_id: id16(&value.evidence_id)?,
            changed_at: checked_timestamp(value.changed_at, now)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &RelationshipsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<GetRelationshipRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    persistence
        .get_relationship(owner, id16(&value.relationship_id)?)
        .await
        .map_err(persistence_error)?
        .map(|value| wire_relationship(&value).encode_to_vec())
        .ok_or("NOT_FOUND")
}

async fn list_for_participant(
    persistence: &RelationshipsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListRelationshipsForParticipantRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    let limit = checked_limit(value.limit)?;
    let mut values = persistence
        .list_for_participant(
            owner,
            decode_participant(value.participant)?,
            optional_id16(&value.after_relationship_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    let has_more = values.len() > usize::from(limit);
    values.truncate(usize::from(limit));
    Ok(ListRelationshipsResultV1 {
        next_after_relationship_id: if has_more {
            values
                .last()
                .map_or_else(Vec::new, |value| value.relationship_id.to_vec())
        } else {
            Vec::new()
        },
        relationships: values.iter().map(wire_relationship).collect(),
    }
    .encode_to_vec())
}

async fn list_evidence(
    persistence: &RelationshipsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListRelationshipEvidenceRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    let limit = checked_limit(value.limit)?;
    let mut values = persistence
        .list_evidence(
            owner,
            id16(&value.relationship_id)?,
            optional_id16(&value.after_evidence_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    let has_more = values.len() > usize::from(limit);
    values.truncate(usize::from(limit));
    Ok(ListRelationshipEvidenceResultV1 {
        next_after_evidence_id: if has_more {
            values
                .last()
                .map_or_else(Vec::new, |value| value.evidence_id.to_vec())
        } else {
            Vec::new()
        },
        evidence: values.iter().map(wire_evidence).collect(),
    }
    .encode_to_vec())
}

fn wire_relationship(value: &RelationshipRecordV1) -> WireRelationship {
    WireRelationship {
        relationship_id: value.relationship_id.to_vec(),
        logical_owner_id: value.logical_owner_id.clone(),
        source: Some(wire_participant(value.source)),
        target: Some(wire_participant(value.target)),
        relationship_type: encode_type(value.relationship_type),
        state: encode_state(value.state),
        valid_from: Some(wire_timestamp(value.valid_from)),
        valid_until: value.valid_until.map(wire_timestamp),
        relationship_revision: value.relationship_revision,
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}
fn wire_evidence(value: &RelationshipEvidenceV1) -> WireEvidence {
    WireEvidence {
        evidence_id: value.evidence_id.to_vec(),
        source_owner_id: value.source_owner_id.clone(),
        source_record_id: value.source_record_id.clone(),
        source_revision: value.source_revision,
        evidence_digest: value.evidence_digest.to_vec(),
        observed_at: Some(wire_timestamp(value.observed_at)),
        state: match value.state {
            RelationshipEvidenceStateV1::Active => {
                WireEvidenceState::RelationshipEvidenceStateActive as i32
            }
            RelationshipEvidenceStateV1::Removed => {
                WireEvidenceState::RelationshipEvidenceStateRemoved as i32
            }
        },
        updated_at_relationship_revision: value.updated_at_relationship_revision,
    }
}
fn wire_participant(value: RelationshipParticipantV1) -> WireParticipant {
    WireParticipant {
        kind: match value.kind {
            RelationshipParticipantKindV1::Person => {
                WireParticipantKind::RelationshipParticipantKindPerson as i32
            }
            RelationshipParticipantKindV1::Organization => {
                WireParticipantKind::RelationshipParticipantKindOrganization as i32
            }
        },
        public_id: value.public_id.to_vec(),
    }
}
fn decode_participant(
    value: Option<WireParticipant>,
) -> Result<RelationshipParticipantV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    Ok(RelationshipParticipantV1 {
        kind: match WireParticipantKind::try_from(value.kind).map_err(|_| "INVALID_ARGUMENT")? {
            WireParticipantKind::RelationshipParticipantKindPerson => {
                RelationshipParticipantKindV1::Person
            }
            WireParticipantKind::RelationshipParticipantKindOrganization => {
                RelationshipParticipantKindV1::Organization
            }
            WireParticipantKind::RelationshipParticipantKindUnspecified => {
                return Err("INVALID_ARGUMENT");
            }
        },
        public_id: id16(&value.public_id)?,
    })
}
fn decode_type(value: i32) -> Result<RelationshipTypeV1, &'static str> {
    match WireType::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireType::RelationshipTypeFamily => Ok(RelationshipTypeV1::Family),
        WireType::RelationshipTypeFriend => Ok(RelationshipTypeV1::Friend),
        WireType::RelationshipTypeColleague => Ok(RelationshipTypeV1::Colleague),
        WireType::RelationshipTypeReportsTo => Ok(RelationshipTypeV1::ReportsTo),
        WireType::RelationshipTypeMemberOf => Ok(RelationshipTypeV1::MemberOf),
        WireType::RelationshipTypePartner => Ok(RelationshipTypeV1::Partner),
        WireType::RelationshipTypeUnspecified => Err("INVALID_ARGUMENT"),
    }
}
fn encode_type(value: RelationshipTypeV1) -> i32 {
    match value {
        RelationshipTypeV1::Family => WireType::RelationshipTypeFamily as i32,
        RelationshipTypeV1::Friend => WireType::RelationshipTypeFriend as i32,
        RelationshipTypeV1::Colleague => WireType::RelationshipTypeColleague as i32,
        RelationshipTypeV1::ReportsTo => WireType::RelationshipTypeReportsTo as i32,
        RelationshipTypeV1::MemberOf => WireType::RelationshipTypeMemberOf as i32,
        RelationshipTypeV1::Partner => WireType::RelationshipTypePartner as i32,
    }
}
fn encode_state(value: RelationshipStateV1) -> i32 {
    match value {
        RelationshipStateV1::Confirmed => WireState::RelationshipStateConfirmed as i32,
        RelationshipStateV1::Ended => WireState::RelationshipStateEnded as i32,
    }
}
fn exact_decode<T: Message + Default>(bytes: &[u8]) -> Result<T, &'static str> {
    let value = T::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or("INVALID_ARGUMENT")
}
fn checked_timestamp(
    value: Option<TimestampV1>,
    now: i64,
) -> Result<RelationshipTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    let millis = value
        .unix_seconds
        .checked_mul(1000)
        .and_then(|base| base.checked_add(i64::from(value.nanos / 1_000_000)))
        .filter(|value| *value > 0 && *value <= now)
        .ok_or("INVALID_ARGUMENT")?;
    let _ = millis;
    if !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_ARGUMENT");
    }
    Ok(RelationshipTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}
fn optional_timestamp(
    value: Option<TimestampV1>,
    now: i64,
) -> Result<Option<RelationshipTimestampV1>, &'static str> {
    value
        .map(|value| checked_timestamp(Some(value), now))
        .transpose()
}
fn wire_timestamp(value: RelationshipTimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}
fn accept_owner(payload: &mut String, authenticated: &str) -> Result<(), &'static str> {
    if payload.is_empty() {
        *payload = authenticated.to_owned();
    } else if payload != authenticated {
        return Err("REJECTED");
    }
    Ok(())
}
fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    fixed_id(value)
}
fn id32(value: &[u8]) -> Result<[u8; 32], &'static str> {
    fixed_id(value)
}
fn fixed_id<const N: usize>(value: &[u8]) -> Result<[u8; N], &'static str> {
    let value: [u8; N] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or("INVALID_ARGUMENT")
}
fn optional_id16(value: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        id16(value).map(Some)
    }
}
fn positive(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}
fn checked_limit(value: u32) -> Result<u16, &'static str> {
    match value {
        1..=200 => u16::try_from(value).map_err(|_| "INVALID_ARGUMENT"),
        _ => Err("INVALID_ARGUMENT"),
    }
}
fn lifecycle_event_id(operation_id: [u8; 16], value: &RelationshipRecordV1) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.relationships.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(value.relationship_id);
    hash.update(value.relationship_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("digest")
}
fn hex(value: [u8; 16]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn persistence_error(value: RelationshipsPersistenceErrorV1) -> &'static str {
    match value {
        RelationshipsPersistenceErrorV1::NotFound => "NOT_FOUND",
        RelationshipsPersistenceErrorV1::RevisionConflict => "REVISION_CONFLICT",
        RelationshipsPersistenceErrorV1::StateConflict => "STATE_CONFLICT",
        RelationshipsPersistenceErrorV1::EvidenceConflict
        | RelationshipsPersistenceErrorV1::OperationConflict
        | RelationshipsPersistenceErrorV1::OutboxConflict => "CONFLICT",
        RelationshipsPersistenceErrorV1::InvalidInput
        | RelationshipsPersistenceErrorV1::InvalidRow => "INVALID_ARGUMENT",
        RelationshipsPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_cursor_and_closed_participants_are_exact() {
        let mut owner = String::new();
        accept_owner(&mut owner, "owner-1").expect("owner");
        assert_eq!(owner, "owner-1");
        assert!(accept_owner(&mut "owner-2".to_owned(), "owner-1").is_err());
        assert!(
            decode_participant(Some(WireParticipant {
                kind: 0,
                public_id: vec![1; 16]
            }))
            .is_err()
        );
        assert_eq!(checked_limit(200), Ok(200));
        assert!(checked_limit(201).is_err());
    }
}
