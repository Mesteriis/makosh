use makosh_knowledge_command_api::{
    KNOWLEDGE_MODULE_ID_V1, KNOWLEDGE_OWNER_ID_V1, KnowledgeCommandEnvelopeContextV1,
    build_knowledge_note_changed_outbox_record_v1,
    client_wire::{
        AddKnowledgeSourceRequestV1, CreateKnowledgeNoteRequestV1, GetKnowledgeNoteRequestV1,
        KnowledgeNoteChangedV1, KnowledgeNoteMutationResultV1, KnowledgeNoteOriginV1 as WireOrigin,
        KnowledgeNoteStateV1 as WireState, KnowledgeNoteV1 as WireNote,
        KnowledgeSourceStateV1 as WireSourceState, KnowledgeSourceV1 as WireSource,
        ListKnowledgeNotesRequestV1, ListKnowledgeNotesResultV1, ListKnowledgeSourcesRequestV1,
        ListKnowledgeSourcesResultV1, RemoveKnowledgeSourceRequestV1,
        ReviewedCandidateProvenanceV1 as WireProvenance, SearchKnowledgeNotesRequestV1,
        SetKnowledgeNoteStateRequestV1, TimestampV1 as WireTimestamp, UpdateKnowledgeNoteRequestV1,
    },
    knowledge_client_add_source_contract_reference_v1,
    knowledge_client_create_contract_reference_v1, knowledge_client_get_contract_reference_v1,
    knowledge_client_list_contract_reference_v1,
    knowledge_client_list_sources_contract_reference_v1,
    knowledge_client_remove_source_contract_reference_v1,
    knowledge_client_search_contract_reference_v1,
    knowledge_client_set_state_contract_reference_v1,
    knowledge_client_update_contract_reference_v1,
};
use makosh_knowledge_core::{
    KnowledgeLifecycleStateV1, KnowledgeNoteOriginV1, KnowledgeNoteRecordV1,
    KnowledgeNoteTimestampV1, KnowledgeSourceStateV1, KnowledgeSourceV1,
    ManualKnowledgeNoteDraftV1, derive_knowledge_source_id_v1, derive_manual_knowledge_note_id_v1,
};
use makosh_knowledge_persistence::{
    KnowledgeLifecycleCommitV1, KnowledgeLifecycleMutationV1, KnowledgeLifecycleOperationOutcomeV1,
    KnowledgeLifecycleOperationV1, KnowledgeOutboxRecordV1, KnowledgePersistenceErrorV1,
    KnowledgePersistenceV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

pub async fn dispatch_knowledge_client_request_v1(
    persistence: &KnowledgePersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let accepted_identity = request.protocol_major == 1
        && request.module_id == KNOWLEDGE_MODULE_ID_V1
        && request.owner_id == KNOWLEDGE_OWNER_ID_V1
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
    persistence: &KnowledgePersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &knowledge_client_get_contract_reference_v1() {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &knowledge_client_list_contract_reference_v1() {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &knowledge_client_search_contract_reference_v1() {
        return search(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &knowledge_client_list_sources_contract_reference_v1() {
        return list_sources(persistence, logical_owner_id, &request.request_payload).await;
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
    let operation = KnowledgeLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: now_unix_millis,
        mutation,
    };
    let context = KnowledgeCommandEnvelopeContextV1 {
        module_id: KNOWLEDGE_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        recorded_at_unix_seconds: now_unix_millis / 1_000,
        recorded_at_nanos: ((now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_lifecycle_operation(operation, |note| {
            let response = KnowledgeNoteMutationResultV1 {
                operation_id: operation_id.to_vec(),
                note: Some(wire_note(note)),
            }
            .encode_to_vec();
            let event_id = lifecycle_event_id(operation_id, note.note_id, note.note_revision);
            let event = build_knowledge_note_changed_outbox_record_v1(
                operation_id,
                KnowledgeNoteChangedV1 {
                    event_id: event_id.to_vec(),
                    note_id: note.note_id.to_vec(),
                    logical_owner_id: note.logical_owner_id.clone(),
                    note_revision: note.note_revision,
                    state: encode_state(note.state),
                    occurred_at: Some(timestamp(note.updated_at)),
                },
                &context,
            )
            .map_err(|_| KnowledgePersistenceErrorV1::InvalidInput)?;
            Ok(KnowledgeLifecycleCommitV1 {
                response_sha256: Sha256::digest(&response).into(),
                response_bytes: response,
                lifecycle_event: KnowledgeOutboxRecordV1 {
                    message_id: *event.message_id(),
                    envelope_sha256: *event.envelope_sha256(),
                    envelope_bytes: event.exact_bytes().to_vec(),
                },
            })
        })
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        KnowledgeLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | KnowledgeLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
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
        knowledge_client_create_contract_reference_v1(),
        CreateKnowledgeNoteRequestV1
    );
    operation_id!(
        knowledge_client_update_contract_reference_v1(),
        UpdateKnowledgeNoteRequestV1
    );
    operation_id!(
        knowledge_client_set_state_contract_reference_v1(),
        SetKnowledgeNoteStateRequestV1
    );
    operation_id!(
        knowledge_client_add_source_contract_reference_v1(),
        AddKnowledgeSourceRequestV1
    );
    operation_id!(
        knowledge_client_remove_source_contract_reference_v1(),
        RemoveKnowledgeSourceRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
    now_unix_millis: i64,
) -> Result<KnowledgeLifecycleMutationV1, &'static str> {
    macro_rules! decode {
        ($type:ty) => {{
            let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
            if value.encode_to_vec() != bytes {
                return Err("INVALID_ARGUMENT");
            }
            value
        }};
    }
    if contract == &knowledge_client_create_contract_reference_v1() {
        let mut value = decode!(CreateKnowledgeNoteRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        let operation_id = id16(&value.operation_id)?;
        let note_id = derive_manual_knowledge_note_id_v1(logical_owner_id, &operation_id)
            .map_err(|_| "INVALID_ARGUMENT")?;
        if !value.note_id.is_empty() && value.note_id != note_id {
            return Err("INVALID_ARGUMENT");
        }
        Ok(KnowledgeLifecycleMutationV1::Create(
            ManualKnowledgeNoteDraftV1 {
                operation_id,
                logical_owner_id: logical_owner_id.to_owned(),
                title: value.title,
                body: value.body,
                created_at: checked_timestamp(value.created_at, now_unix_millis)?,
            },
        ))
    } else if contract == &knowledge_client_update_contract_reference_v1() {
        let mut value = decode!(UpdateKnowledgeNoteRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(KnowledgeLifecycleMutationV1::Update {
            operation_id: id16(&value.operation_id)?,
            note_id: id16(&value.note_id)?,
            expected_revision: positive_revision(value.expected_note_revision)?,
            title: value.title,
            body: value.body,
            changed_at: checked_timestamp(value.updated_at, now_unix_millis)?,
        })
    } else if contract == &knowledge_client_set_state_contract_reference_v1() {
        let mut value = decode!(SetKnowledgeNoteStateRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(KnowledgeLifecycleMutationV1::SetState {
            operation_id: id16(&value.operation_id)?,
            note_id: id16(&value.note_id)?,
            expected_revision: positive_revision(value.expected_note_revision)?,
            state: decode_state(value.state)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &knowledge_client_add_source_contract_reference_v1() {
        let mut value = decode!(AddKnowledgeSourceRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        let note_id = id16(&value.note_id)?;
        let source_record_id = id16(&value.source_record_id)?;
        let source_id =
            derive_knowledge_source_id_v1(&note_id, &value.source_owner_id, &source_record_id)
                .map_err(|_| "INVALID_ARGUMENT")?;
        if !value.source_id.is_empty() && value.source_id != source_id {
            return Err("INVALID_ARGUMENT");
        }
        Ok(KnowledgeLifecycleMutationV1::AddSource {
            operation_id: id16(&value.operation_id)?,
            note_id,
            expected_revision: positive_revision(value.expected_note_revision)?,
            source_owner_id: value.source_owner_id,
            source_record_id,
            source_revision: positive_revision(value.source_revision)?,
            evidence_digest: id32(&value.evidence_digest)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &knowledge_client_remove_source_contract_reference_v1() {
        let mut value = decode!(RemoveKnowledgeSourceRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(KnowledgeLifecycleMutationV1::RemoveSource {
            operation_id: id16(&value.operation_id)?,
            note_id: id16(&value.note_id)?,
            expected_revision: positive_revision(value.expected_note_revision)?,
            source_id: id16(&value.source_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &KnowledgePersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = canonical::<GetKnowledgeNoteRequestV1>(bytes)?;
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let note = persistence
        .get_lifecycle_note(logical_owner_id, id16(&request.note_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    Ok(wire_note(&note).encode_to_vec())
}

async fn list(
    persistence: &KnowledgePersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = canonical::<ListKnowledgeNotesRequestV1>(bytes)?;
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let after = optional_id16(&request.after_note_id)?;
    let limit = checked_limit(request.limit)?;
    let notes = persistence
        .list_lifecycle_notes(logical_owner_id, after, limit + 1)
        .await
        .map_err(persistence_error)?;
    Ok(note_page(notes, usize::from(limit)).encode_to_vec())
}

async fn search(
    persistence: &KnowledgePersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = canonical::<SearchKnowledgeNotesRequestV1>(bytes)?;
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let after = optional_id16(&request.after_note_id)?;
    let limit = checked_limit(request.limit)?;
    let notes = persistence
        .search_lifecycle_notes(logical_owner_id, &request.query, after, limit + 1)
        .await
        .map_err(persistence_error)?;
    Ok(note_page(notes, usize::from(limit)).encode_to_vec())
}

async fn list_sources(
    persistence: &KnowledgePersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = canonical::<ListKnowledgeSourcesRequestV1>(bytes)?;
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(request.limit)?;
    let mut sources = persistence
        .list_lifecycle_sources(
            logical_owner_id,
            id16(&request.note_id)?,
            optional_id16(&request.after_source_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    let has_more = sources.len() > usize::from(limit);
    sources.truncate(usize::from(limit));
    let next = has_more
        .then(|| sources.last().map(|source| source.source_id.to_vec()))
        .flatten()
        .unwrap_or_default();
    Ok(ListKnowledgeSourcesResultV1 {
        sources: sources.iter().map(wire_source).collect(),
        next_after_source_id: next,
    }
    .encode_to_vec())
}

fn note_page(mut notes: Vec<KnowledgeNoteRecordV1>, limit: usize) -> ListKnowledgeNotesResultV1 {
    let has_more = notes.len() > limit;
    notes.truncate(limit);
    let next = has_more
        .then(|| notes.last().map(|note| note.note_id.to_vec()))
        .flatten()
        .unwrap_or_default();
    ListKnowledgeNotesResultV1 {
        notes: notes.iter().map(wire_note).collect(),
        next_after_note_id: next,
    }
}

fn wire_note(note: &KnowledgeNoteRecordV1) -> WireNote {
    WireNote {
        note_id: note.note_id.to_vec(),
        logical_owner_id: note.logical_owner_id.clone(),
        title: note.title.clone(),
        body: note.body.clone(),
        state: encode_state(note.state),
        origin: match note.origin {
            KnowledgeNoteOriginV1::ReviewedCandidate => {
                WireOrigin::KnowledgeNoteOriginReviewedCandidate as i32
            }
            KnowledgeNoteOriginV1::OwnerAuthored => {
                WireOrigin::KnowledgeNoteOriginOwnerAuthored as i32
            }
        },
        note_revision: note.note_revision,
        reviewed_provenance: note
            .reviewed_provenance
            .as_ref()
            .map(|value| WireProvenance {
                approved_candidate_id: value.approved_candidate_id.to_vec(),
                candidate_digest: value.candidate_digest.to_vec(),
                source_evidence_id: value.source_evidence_id.to_vec(),
                source_evidence_revision: value.source_evidence_revision,
                review_id: value.review_id.to_vec(),
                decision_revision: value.decision_revision,
                decided_by_owner_device_id: value.decided_by_owner_device_id.to_vec(),
            }),
        created_at: Some(timestamp(note.created_at)),
        updated_at: Some(timestamp(note.updated_at)),
    }
}

fn wire_source(source: &KnowledgeSourceV1) -> WireSource {
    WireSource {
        source_id: source.source_id.to_vec(),
        source_owner_id: source.source_owner_id.clone(),
        source_record_id: source.source_record_id.to_vec(),
        source_revision: source.source_revision,
        evidence_digest: source.evidence_digest.to_vec(),
        state: match source.state {
            KnowledgeSourceStateV1::Active => WireSourceState::KnowledgeSourceStateActive as i32,
            KnowledgeSourceStateV1::Removed => WireSourceState::KnowledgeSourceStateRemoved as i32,
        },
        updated_at_note_revision: source.updated_at_note_revision,
        created_at: Some(timestamp(source.created_at)),
        updated_at: Some(timestamp(source.updated_at)),
    }
}

fn canonical<T: Message + Default>(bytes: &[u8]) -> Result<T, &'static str> {
    let value = T::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if value.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    Ok(value)
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
) -> Result<KnowledgeNoteTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    if value.unix_seconds <= 0
        || !(0..1_000_000_000).contains(&value.nanos)
        || value.unix_seconds > now_unix_millis / 1_000
        || (value.unix_seconds == now_unix_millis / 1_000
            && i64::from(value.nanos) > (now_unix_millis % 1_000) * 1_000_000)
    {
        return Err("INVALID_ARGUMENT");
    }
    Ok(KnowledgeNoteTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn timestamp(value: KnowledgeNoteTimestampV1) -> WireTimestamp {
    WireTimestamp {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn decode_state(value: i32) -> Result<KnowledgeLifecycleStateV1, &'static str> {
    match WireState::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireState::KnowledgeNoteStateActive => Ok(KnowledgeLifecycleStateV1::Active),
        WireState::KnowledgeNoteStateArchived => Ok(KnowledgeLifecycleStateV1::Archived),
        WireState::KnowledgeNoteStateUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn encode_state(value: KnowledgeLifecycleStateV1) -> i32 {
    match value {
        KnowledgeLifecycleStateV1::Active => WireState::KnowledgeNoteStateActive as i32,
        KnowledgeLifecycleStateV1::Archived => WireState::KnowledgeNoteStateArchived as i32,
    }
}

fn checked_limit(value: u32) -> Result<u16, &'static str> {
    u16::try_from(value)
        .ok()
        .filter(|value| (1..=200).contains(value))
        .ok_or("INVALID_ARGUMENT")
}

fn positive_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}

fn optional_id16(value: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        id16(value).map(Some)
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let id: [u8; 16] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or("INVALID_ARGUMENT")
}

fn id32(value: &[u8]) -> Result<[u8; 32], &'static str> {
    let id: [u8; 32] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or("INVALID_ARGUMENT")
}

fn lifecycle_event_id(operation_id: [u8; 16], note_id: [u8; 16], note_revision: u64) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.knowledge.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(note_id);
    hash.update(note_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn persistence_error(value: KnowledgePersistenceErrorV1) -> &'static str {
    match value {
        KnowledgePersistenceErrorV1::NotFound => "NOT_FOUND",
        KnowledgePersistenceErrorV1::InvalidInput | KnowledgePersistenceErrorV1::InvalidRow => {
            "INVALID_ARGUMENT"
        }
        KnowledgePersistenceErrorV1::OperationConflict
        | KnowledgePersistenceErrorV1::RevisionConflict
        | KnowledgePersistenceErrorV1::CommandConflict
        | KnowledgePersistenceErrorV1::InboxConflict
        | KnowledgePersistenceErrorV1::KnowledgeNoteConflict => "FAILED_PRECONDITION",
        KnowledgePersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_is_empty_or_exact_and_page_cursor_is_last_returned() {
        let mut owner = String::new();
        accept_owner(&mut owner, "owner-1").expect("owner");
        assert_eq!(owner, "owner-1");
        assert!(accept_owner(&mut "owner-2".to_owned(), "owner-1").is_err());

        let notes = (1_u8..=3)
            .map(|id| KnowledgeNoteRecordV1 {
                note_id: [id; 16],
                logical_owner_id: "owner-1".to_owned(),
                title: format!("Note {id}"),
                body: "Body".to_owned(),
                state: KnowledgeLifecycleStateV1::Active,
                origin: KnowledgeNoteOriginV1::OwnerAuthored,
                note_revision: 1,
                reviewed_provenance: None,
                sources: Vec::new(),
                created_at: KnowledgeNoteTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
                updated_at: KnowledgeNoteTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            })
            .collect();
        let page = note_page(notes, 2);
        assert_eq!(page.notes.len(), 2);
        assert_eq!(page.next_after_note_id, vec![2; 16]);
    }
}
