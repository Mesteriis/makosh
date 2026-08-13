use makosh_documents_api::{
    DOCUMENTS_MODULE_ID_V1, DOCUMENTS_OWNER_ID_V1, DocumentsEnvelopeContextV1,
    build_document_changed_outbox_record_v1,
    client_wire::{
        AddDocumentSourceRequestV1, AttachDocumentBlobRequestV1, CreateDocumentRequestV1,
        DocumentChangedV1, DocumentCustodyStateV1 as WireCustodyState, DocumentMutationResultV1,
        DocumentSourceStateV1 as WireSourceState, DocumentSourceV1 as WireSource,
        DocumentStateV1 as WireState, DocumentV1 as WireDocument, GetDocumentRequestV1,
        ListDocumentSourcesRequestV1, ListDocumentSourcesResultV1, ListDocumentsRequestV1,
        ListDocumentsResultV1, ReleaseDocumentBlobRequestV1, RemoveDocumentSourceRequestV1,
        SearchDocumentsRequestV1, SetDocumentStateRequestV1, TimestampV1, UpdateDocumentRequestV1,
    },
    documents_client_add_source_contract_reference_v1,
    documents_client_attach_blob_contract_reference_v1,
    documents_client_create_contract_reference_v1, documents_client_get_contract_reference_v1,
    documents_client_list_contract_reference_v1,
    documents_client_list_sources_contract_reference_v1,
    documents_client_release_blob_contract_reference_v1,
    documents_client_remove_source_contract_reference_v1,
    documents_client_search_contract_reference_v1,
    documents_client_set_state_contract_reference_v1,
    documents_client_update_contract_reference_v1,
};
use makosh_documents_core::{
    DocumentCustodyStateV1, DocumentSourceStateV1, DocumentSourceV1, DocumentStateV1, DocumentV1,
};
use makosh_documents_persistence::{
    CompleteDocumentBlobOperationV1, DocumentBlobOperationKindV1,
    DocumentBlobOperationStartOutcomeV1, DocumentBlobOperationStartV1, DocumentBoundBlobCustodyV1,
    DocumentLifecycleCommitV1, DocumentLifecycleMutationV1, DocumentLifecycleOperationOutcomeV1,
    DocumentLifecycleOperationV1, DocumentOutboxRecordV1, DocumentsPersistenceErrorV1,
    DocumentsPersistenceV1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentsClientRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub struct DocumentsBlobAttachRequestV1<'a> {
    pub operation_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub declared_size: u64,
    pub content_sha256: [u8; 32],
    pub custody_source_proof: &'a [u8],
    pub source_evidence_id: [u8; 16],
    pub source_evidence_envelope_sha256: [u8; 32],
}

pub struct DocumentsBlobReleaseRequestV1<'a> {
    pub operation_id: [u8; 16],
    pub custody: &'a DocumentBoundBlobCustodyV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentsBlobReceiptV1 {
    pub resolved_reference_id: [u8; 16],
    pub exact_receipt_bytes: Vec<u8>,
}

pub trait DocumentsBlobCustodyPortV1 {
    fn attach(
        &mut self,
        request: DocumentsBlobAttachRequestV1<'_>,
    ) -> Result<DocumentsBlobReceiptV1, &'static str>;

    fn release(
        &mut self,
        request: DocumentsBlobReleaseRequestV1<'_>,
    ) -> Result<DocumentsBlobReceiptV1, &'static str>;
}

pub async fn dispatch_documents_client_request_v1(
    persistence: &DocumentsPersistenceV1,
    blob: &mut dyn DocumentsBlobCustodyPortV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    context: DocumentsClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == DOCUMENTS_MODULE_ID_V1
        && request.owner_id == DOCUMENTS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && context.runtime_instance_id.iter().any(|byte| *byte != 0)
        && context.runtime_generation > 0
        && context.now_unix_millis > 0;
    let response = if accepted {
        dispatch(persistence, blob, logical_owner_id, &request, context).await
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
    persistence: &DocumentsPersistenceV1,
    blob: &mut dyn DocumentsBlobCustodyPortV1,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    context: DocumentsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &documents_client_get_contract_reference_v1() {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &documents_client_list_contract_reference_v1() {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &documents_client_search_contract_reference_v1() {
        return search(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &documents_client_list_sources_contract_reference_v1() {
        return list_sources(persistence, logical_owner_id, &request.request_payload).await;
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
    if contract == &documents_client_attach_blob_contract_reference_v1() {
        return attach_blob(
            persistence,
            blob,
            logical_owner_id,
            operation_id,
            request_sha256,
            &request.request_payload,
            context,
        )
        .await;
    }
    if contract == &documents_client_release_blob_contract_reference_v1() {
        return release_blob(
            persistence,
            blob,
            logical_owner_id,
            operation_id,
            request_sha256,
            &request.request_payload,
            context,
        )
        .await;
    }
    let mutation = decode_mutation(
        contract,
        logical_owner_id,
        &request.request_payload,
        context.now_unix_millis,
    )?;
    let operation = DocumentLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: context.now_unix_millis,
        mutation,
    };
    let envelope_context = envelope_context(context);
    let outcome = persistence
        .apply_lifecycle_operation(operation, |document| {
            build_commit(operation_id, document, &envelope_context)
        })
        .await
        .map_err(persistence_error)?;
    outcome_response(outcome)
}

async fn attach_blob(
    persistence: &DocumentsPersistenceV1,
    blob: &mut dyn DocumentsBlobCustodyPortV1,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    bytes: &[u8],
    context: DocumentsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<AttachDocumentBlobRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let document_id = id16(&value.document_id)?;
    let source_reference_id = id16(&value.blob_reference_id)?;
    let content_sha256 = id32(&value.content_sha256)?;
    let source_evidence_id = id16(&value.source_evidence_id)?;
    let source_evidence_envelope_sha256 = id32(&value.source_evidence_envelope_sha256)?;
    let changed_at = checked_timestamp(value.changed_at, context.now_unix_millis)?;
    if value.custody_transfer_source_proof.is_empty()
        || value.custody_transfer_source_proof.len() > 2_048
    {
        return Err("INVALID_ARGUMENT");
    }
    let provider_request = provider_intent_bytes(
        b"makosh.documents.blob.attach.v1",
        &[
            &operation_id,
            &source_reference_id,
            &value.declared_size.to_be_bytes(),
            &content_sha256,
            &value.custody_transfer_source_proof,
            &source_evidence_id,
            &source_evidence_envelope_sha256,
        ],
    );
    let start = DocumentBlobOperationStartV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        document_id,
        expected_revision: positive_revision(value.expected_document_revision)?,
        kind: DocumentBlobOperationKindV1::Attach,
        blob_reference_id: source_reference_id,
        declared_size: Some(value.declared_size),
        content_sha256: Some(content_sha256),
        changed_at_unix_millis: changed_at,
        custody_source_proof: value.custody_transfer_source_proof.clone(),
        source_evidence_id: Some(source_evidence_id),
        source_evidence_envelope_sha256: Some(source_evidence_envelope_sha256),
        client_request_sha256: request_sha256,
        client_request_bytes: bytes.to_vec(),
        provider_request_sha256: Sha256::digest(&provider_request).into(),
        provider_request_bytes: provider_request,
        received_at_unix_millis: context.now_unix_millis,
    };
    match persistence
        .start_blob_operation(start)
        .await
        .map_err(persistence_error)?
    {
        DocumentBlobOperationStartOutcomeV1::Replayed { response_bytes } => {
            return Ok(response_bytes);
        }
        DocumentBlobOperationStartOutcomeV1::Pending => {}
    }
    let receipt = blob.attach(DocumentsBlobAttachRequestV1 {
        operation_id,
        source_reference_id,
        declared_size: value.declared_size,
        content_sha256,
        custody_source_proof: &value.custody_transfer_source_proof,
        source_evidence_id,
        source_evidence_envelope_sha256,
    })?;
    complete_blob(
        persistence,
        logical_owner_id,
        operation_id,
        receipt,
        context,
    )
    .await
}

async fn release_blob(
    persistence: &DocumentsPersistenceV1,
    blob: &mut dyn DocumentsBlobCustodyPortV1,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    bytes: &[u8],
    context: DocumentsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ReleaseDocumentBlobRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let document_id = id16(&value.document_id)?;
    let blob_reference_id = id16(&value.blob_reference_id)?;
    let changed_at = checked_timestamp(value.changed_at, context.now_unix_millis)?;
    let custody = persistence
        .load_bound_blob_custody(logical_owner_id, document_id)
        .await
        .map_err(persistence_error)?
        .ok_or("STATE_CONFLICT")?;
    if custody.blob_reference_id != blob_reference_id {
        return Err("STATE_CONFLICT");
    }
    let provider_request = provider_intent_bytes(
        b"makosh.documents.blob.release.v1",
        &[
            &operation_id,
            &blob_reference_id,
            &custody.declared_size.to_be_bytes(),
            &custody.content_sha256,
            &custody.custody_source_proof,
        ],
    );
    let start = DocumentBlobOperationStartV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        document_id,
        expected_revision: positive_revision(value.expected_document_revision)?,
        kind: DocumentBlobOperationKindV1::Release,
        blob_reference_id,
        declared_size: None,
        content_sha256: None,
        changed_at_unix_millis: changed_at,
        custody_source_proof: custody.custody_source_proof.clone(),
        source_evidence_id: None,
        source_evidence_envelope_sha256: None,
        client_request_sha256: request_sha256,
        client_request_bytes: bytes.to_vec(),
        provider_request_sha256: Sha256::digest(&provider_request).into(),
        provider_request_bytes: provider_request,
        received_at_unix_millis: context.now_unix_millis,
    };
    match persistence
        .start_blob_operation(start)
        .await
        .map_err(persistence_error)?
    {
        DocumentBlobOperationStartOutcomeV1::Replayed { response_bytes } => {
            return Ok(response_bytes);
        }
        DocumentBlobOperationStartOutcomeV1::Pending => {}
    }
    let receipt = blob.release(DocumentsBlobReleaseRequestV1 {
        operation_id,
        custody: &custody,
    })?;
    complete_blob(
        persistence,
        logical_owner_id,
        operation_id,
        receipt,
        context,
    )
    .await
}

async fn complete_blob(
    persistence: &DocumentsPersistenceV1,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    receipt: DocumentsBlobReceiptV1,
    context: DocumentsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    if receipt.exact_receipt_bytes.is_empty() || receipt.exact_receipt_bytes.len() > 64 * 1024 {
        return Err("UNAVAILABLE");
    }
    let receipt_sha256: [u8; 32] = Sha256::digest(&receipt.exact_receipt_bytes).into();
    let envelope_context = envelope_context(context);
    let outcome = persistence
        .complete_blob_operation(
            CompleteDocumentBlobOperationV1 {
                logical_owner_id: logical_owner_id.to_owned(),
                operation_id,
                provider_receipt_sha256: receipt_sha256,
                provider_receipt_bytes: receipt.exact_receipt_bytes,
                resolved_blob_reference_id: receipt.resolved_reference_id,
                completed_at_unix_millis: context.now_unix_millis,
            },
            |document| build_commit(operation_id, document, &envelope_context),
        )
        .await
        .map_err(persistence_error)?;
    outcome_response(outcome)
}

fn outcome_response(outcome: DocumentLifecycleOperationOutcomeV1) -> Result<Vec<u8>, &'static str> {
    Ok(match outcome {
        DocumentLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | DocumentLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn build_commit(
    operation_id: [u8; 16],
    document: &DocumentV1,
    context: &DocumentsEnvelopeContextV1,
) -> Result<DocumentLifecycleCommitV1, DocumentsPersistenceErrorV1> {
    let response = DocumentMutationResultV1 {
        operation_id: operation_id.to_vec(),
        document: Some(wire_document(document)),
    }
    .encode_to_vec();
    let event = build_document_changed_outbox_record_v1(
        operation_id,
        DocumentChangedV1 {
            event_id: lifecycle_event_id(operation_id, document).to_vec(),
            document_id: document.document_id.to_vec(),
            logical_owner_id: document.logical_owner_id.clone(),
            document_revision: document.document_revision,
            state: encode_state(document.state),
            custody_state: encode_custody_state(
                document
                    .custody
                    .as_ref()
                    .map_or(DocumentCustodyStateV1::Unbound, |value| value.state),
            ),
            occurred_at: Some(wire_timestamp(document.updated_at_unix_millis)),
        },
        context,
    )
    .map_err(|_| DocumentsPersistenceErrorV1::InvalidInput)?;
    Ok(DocumentLifecycleCommitV1 {
        response_sha256: Sha256::digest(&response).into(),
        response_bytes: response,
        lifecycle_event: DocumentOutboxRecordV1 {
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
    macro_rules! operation_id {
        ($reference:expr, $type:ty) => {
            if contract == &$reference {
                return id16(&exact_decode::<$type>(bytes)?.operation_id);
            }
        };
    }
    operation_id!(
        documents_client_create_contract_reference_v1(),
        CreateDocumentRequestV1
    );
    operation_id!(
        documents_client_update_contract_reference_v1(),
        UpdateDocumentRequestV1
    );
    operation_id!(
        documents_client_set_state_contract_reference_v1(),
        SetDocumentStateRequestV1
    );
    operation_id!(
        documents_client_attach_blob_contract_reference_v1(),
        AttachDocumentBlobRequestV1
    );
    operation_id!(
        documents_client_release_blob_contract_reference_v1(),
        ReleaseDocumentBlobRequestV1
    );
    operation_id!(
        documents_client_add_source_contract_reference_v1(),
        AddDocumentSourceRequestV1
    );
    operation_id!(
        documents_client_remove_source_contract_reference_v1(),
        RemoveDocumentSourceRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &ContractReferenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
    now: i64,
) -> Result<DocumentLifecycleMutationV1, &'static str> {
    if contract == &documents_client_create_contract_reference_v1() {
        let mut value = exact_decode::<CreateDocumentRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(DocumentLifecycleMutationV1::Create {
            title: value.title,
            description: value.description,
            media_type: value.media_type,
            original_file_name: value.original_file_name,
            declared_size: value.declared_size,
            content_sha256: id32(&value.content_sha256)?,
            created_at_unix_millis: checked_timestamp(value.created_at, now)?,
        })
    } else if contract == &documents_client_update_contract_reference_v1() {
        let mut value = exact_decode::<UpdateDocumentRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(DocumentLifecycleMutationV1::Update {
            document_id: id16(&value.document_id)?,
            expected_revision: positive_revision(value.expected_document_revision)?,
            title: value.title,
            description: value.description,
            media_type: value.media_type,
            original_file_name: value.original_file_name,
            changed_at_unix_millis: checked_timestamp(value.updated_at, now)?,
        })
    } else if contract == &documents_client_set_state_contract_reference_v1() {
        let mut value = exact_decode::<SetDocumentStateRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(DocumentLifecycleMutationV1::SetState {
            document_id: id16(&value.document_id)?,
            expected_revision: positive_revision(value.expected_document_revision)?,
            state: decode_state(value.state)?,
            changed_at_unix_millis: checked_timestamp(value.changed_at, now)?,
        })
    } else if contract == &documents_client_add_source_contract_reference_v1() {
        let mut value = exact_decode::<AddDocumentSourceRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(DocumentLifecycleMutationV1::AddSource {
            document_id: id16(&value.document_id)?,
            expected_revision: positive_revision(value.expected_document_revision)?,
            source_owner_id: value.source_owner_id,
            source_record_id: value.source_record_id,
            source_revision: positive_revision(value.source_revision)?,
            evidence_digest: id32(&value.evidence_digest)?,
            changed_at_unix_millis: checked_timestamp(value.changed_at, now)?,
        })
    } else if contract == &documents_client_remove_source_contract_reference_v1() {
        let mut value = exact_decode::<RemoveDocumentSourceRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(DocumentLifecycleMutationV1::RemoveSource {
            document_id: id16(&value.document_id)?,
            expected_revision: positive_revision(value.expected_document_revision)?,
            source_id: id16(&value.source_id)?,
            changed_at_unix_millis: checked_timestamp(value.changed_at, now)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &DocumentsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<GetDocumentRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    persistence
        .get_document(owner, id16(&value.document_id)?)
        .await
        .map_err(persistence_error)?
        .map(|value| wire_document(&value).encode_to_vec())
        .ok_or("NOT_FOUND")
}

async fn list(
    persistence: &DocumentsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListDocumentsRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    list_query(
        persistence,
        owner,
        None,
        &value.after_document_id,
        value.limit,
    )
    .await
}

async fn search(
    persistence: &DocumentsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<SearchDocumentsRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    list_query(
        persistence,
        owner,
        Some(&value.query),
        &value.after_document_id,
        value.limit,
    )
    .await
}

async fn list_query(
    persistence: &DocumentsPersistenceV1,
    owner: &str,
    query: Option<&str>,
    after: &[u8],
    raw_limit: u32,
) -> Result<Vec<u8>, &'static str> {
    let limit = checked_limit(raw_limit)?;
    let mut documents = persistence
        .list_documents(owner, query, optional_id16(after)?, limit + 1)
        .await
        .map_err(persistence_error)?;
    let has_more = documents.len() > usize::from(limit);
    documents.truncate(usize::from(limit));
    Ok(ListDocumentsResultV1 {
        documents: documents.iter().map(wire_document).collect(),
        next_after_document_id: if has_more {
            documents
                .last()
                .map_or_else(Vec::new, |value| value.document_id.to_vec())
        } else {
            Vec::new()
        },
    }
    .encode_to_vec())
}

async fn list_sources(
    persistence: &DocumentsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListDocumentSourcesRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, owner)?;
    let limit = checked_limit(value.limit)?;
    let mut sources = persistence
        .list_sources(
            owner,
            id16(&value.document_id)?,
            optional_id16(&value.after_source_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    let has_more = sources.len() > usize::from(limit);
    sources.truncate(usize::from(limit));
    Ok(ListDocumentSourcesResultV1 {
        sources: sources.iter().map(wire_source).collect(),
        next_after_source_id: if has_more {
            sources
                .last()
                .map_or_else(Vec::new, |value| value.source_id.to_vec())
        } else {
            Vec::new()
        },
    }
    .encode_to_vec())
}

fn wire_document(value: &DocumentV1) -> WireDocument {
    WireDocument {
        document_id: value.document_id.to_vec(),
        logical_owner_id: value.logical_owner_id.clone(),
        title: value.title.clone(),
        description: value.description.clone(),
        media_type: value.media_type.clone(),
        original_file_name: value.original_file_name.clone(),
        declared_size: value.declared_size,
        content_sha256: value.content_sha256.to_vec(),
        state: encode_state(value.state),
        custody_state: encode_custody_state(
            value
                .custody
                .as_ref()
                .map_or(DocumentCustodyStateV1::Unbound, |custody| custody.state),
        ),
        document_revision: value.document_revision,
        created_at: Some(wire_timestamp(value.created_at_unix_millis)),
        updated_at: Some(wire_timestamp(value.updated_at_unix_millis)),
    }
}

fn wire_source(value: &DocumentSourceV1) -> WireSource {
    WireSource {
        source_id: value.source_id.to_vec(),
        source_owner_id: value.source_owner_id.clone(),
        source_record_id: value.source_record_id.clone(),
        source_revision: value.source_revision,
        evidence_digest: value.evidence_digest.to_vec(),
        state: match value.state {
            DocumentSourceStateV1::Active => WireSourceState::DocumentSourceStateActive as i32,
            DocumentSourceStateV1::Removed => WireSourceState::DocumentSourceStateRemoved as i32,
        },
        updated_at_document_revision: value.updated_at_document_revision,
    }
}

fn exact_decode<T: Message + Default>(bytes: &[u8]) -> Result<T, &'static str> {
    let value = T::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or("INVALID_ARGUMENT")
}

fn checked_timestamp(value: Option<TimestampV1>, now: i64) -> Result<i64, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    if !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_ARGUMENT");
    }
    value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|base| base.checked_add(i64::from(value.nanos / 1_000_000)))
        .filter(|millis| *millis > 0 && *millis <= now)
        .ok_or("INVALID_ARGUMENT")
}

fn wire_timestamp(millis: i64) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: millis / 1_000,
        nanos: ((millis % 1_000) * 1_000_000) as i32,
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
fn positive_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}
fn checked_limit(value: u32) -> Result<u16, &'static str> {
    match value {
        1..=200 => u16::try_from(value).map_err(|_| "INVALID_ARGUMENT"),
        _ => Err("INVALID_ARGUMENT"),
    }
}

fn decode_state(value: i32) -> Result<DocumentStateV1, &'static str> {
    match WireState::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireState::DocumentStateActive => Ok(DocumentStateV1::Active),
        WireState::DocumentStateArchived => Ok(DocumentStateV1::Archived),
        WireState::DocumentStateUnspecified => Err("INVALID_ARGUMENT"),
    }
}
fn encode_state(value: DocumentStateV1) -> i32 {
    match value {
        DocumentStateV1::Active => WireState::DocumentStateActive as i32,
        DocumentStateV1::Archived => WireState::DocumentStateArchived as i32,
    }
}
fn encode_custody_state(value: DocumentCustodyStateV1) -> i32 {
    match value {
        DocumentCustodyStateV1::Unbound => WireCustodyState::DocumentCustodyStateUnbound as i32,
        DocumentCustodyStateV1::Bound => WireCustodyState::DocumentCustodyStateBound as i32,
        DocumentCustodyStateV1::Released => WireCustodyState::DocumentCustodyStateReleased as i32,
    }
}

fn envelope_context(context: DocumentsClientRuntimeContextV1) -> DocumentsEnvelopeContextV1 {
    DocumentsEnvelopeContextV1 {
        module_id: DOCUMENTS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: encode_id(context.runtime_instance_id),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
    }
}

fn lifecycle_event_id(operation_id: [u8; 16], value: &DocumentV1) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.documents.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(value.document_id);
    hash.update(value.document_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn encode_id(value: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(32);
    for byte in value {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn provider_intent_bytes(domain: &[u8], chunks: &[&[u8]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(domain);
    for chunk in chunks {
        bytes.extend_from_slice(&(chunk.len() as u64).to_be_bytes());
        bytes.extend_from_slice(chunk);
    }
    bytes
}

fn persistence_error(value: DocumentsPersistenceErrorV1) -> &'static str {
    match value {
        DocumentsPersistenceErrorV1::NotFound => "NOT_FOUND",
        DocumentsPersistenceErrorV1::RevisionConflict => "REVISION_CONFLICT",
        DocumentsPersistenceErrorV1::StateConflict => "STATE_CONFLICT",
        DocumentsPersistenceErrorV1::OperationConflict
        | DocumentsPersistenceErrorV1::OutboxConflict => "CONFLICT",
        DocumentsPersistenceErrorV1::InvalidInput | DocumentsPersistenceErrorV1::InvalidRow => {
            "INVALID_ARGUMENT"
        }
        DocumentsPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoBlob;
    impl DocumentsBlobCustodyPortV1 for NoBlob {
        fn attach(
            &mut self,
            _: DocumentsBlobAttachRequestV1<'_>,
        ) -> Result<DocumentsBlobReceiptV1, &'static str> {
            Err("UNAVAILABLE")
        }
        fn release(
            &mut self,
            _: DocumentsBlobReleaseRequestV1<'_>,
        ) -> Result<DocumentsBlobReceiptV1, &'static str> {
            Err("UNAVAILABLE")
        }
    }

    fn document(id: u8) -> DocumentV1 {
        DocumentV1::create(makosh_documents_core::CreateDocumentV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [id; 16],
            title: format!("Document {id}"),
            description: String::new(),
            media_type: "application/pdf".to_owned(),
            original_file_name: "document.pdf".to_owned(),
            declared_size: 10,
            content_sha256: [4; 32],
            created_at_unix_millis: 1_000,
        })
        .expect("document")
    }

    #[test]
    fn owner_cursor_and_provider_intent_are_exact() {
        let mut owner = String::new();
        accept_owner(&mut owner, "owner-1").expect("owner");
        assert_eq!(owner, "owner-1");
        assert!(accept_owner(&mut "owner-2".to_owned(), "owner-1").is_err());
        let mut values = vec![document(1), document(2), document(3)];
        let has_more = values.len() > 2;
        values.truncate(2);
        assert!(has_more);
        assert_eq!(
            values.last().expect("last").document_id,
            document(2).document_id
        );
        let intent = provider_intent_bytes(b"domain", &[b"one", b"two"]);
        assert_eq!(intent, provider_intent_bytes(b"domain", &[b"one", b"two"]));
        assert_ne!(intent, provider_intent_bytes(b"domain", &[b"two", b"one"]));
        let _ = NoBlob;
    }
}
