//! Actual Documents lifecycle, Blob custody, replay, restart, privacy and owner-RLS contour.

use super::*;

use std::time::{Duration, Instant};

use crate::identity::device::signer::DeviceSigner;
use makosh_documents_api::{
    DOCUMENTS_CLIENT_CAPABILITY_ID_V1, DOCUMENTS_MODULE_ID_V1, DOCUMENTS_OWNER_ID_V1,
    client_wire::{
        AddDocumentSourceRequestV1, AttachDocumentBlobRequestV1, CreateDocumentRequestV1,
        DocumentCustodyStateV1, DocumentMutationResultV1, DocumentSourceStateV1, DocumentStateV1,
        DocumentV1, GetDocumentRequestV1, ListDocumentSourcesRequestV1,
        ListDocumentSourcesResultV1, ListDocumentsRequestV1, ListDocumentsResultV1,
        ReleaseDocumentBlobRequestV1, RemoveDocumentSourceRequestV1, SearchDocumentsRequestV1,
        SetDocumentStateRequestV1, TimestampV1,
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
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};

const PRIVATE_TITLE_V1: &str = "documents-private-title-marker";
const PRIVATE_DESCRIPTION_V1: &str = "documents-private-description-marker";
const PRIVATE_FILE_V1: &str = "documents-private-file-marker.pdf";
const PRIVATE_SOURCE_V1: &str = "documents-private-source-record";
const PRIVATE_BLOB_V1: &[u8] = b"documents-private-blob-content-marker";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Documents binaries"]
fn managed_documents_lifecycle_custody_replays_and_restarts_with_owner_rls() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-documents");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_documents_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    crate::platform::blob::binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Documents Blob release");
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Documents owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            DOCUMENTS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Documents owner");
    let admitted = admit_documents_runtime_v1(&store);
    let source = DocumentsBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Documents Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    blob_launch::start_from_kernel(
        &supervisor,
        &store,
        release.kernel(),
        &data,
        &root.join("runtime"),
    )
    .expect("start Documents Blob runtime");
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_documents_runtime_v1(&supervisor, &store, admitted);
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let documents =
        start_documents_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);

    let source_blob = source.write(&store, &supervisor, &data, [0x41; 16], PRIVATE_BLOB_V1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Documents clock")
        .as_millis() as i64
        - 1_000;
    let timestamp = TimestampV1 {
        unix_seconds: now / 1_000,
        nanos: ((now % 1_000) * 1_000_000) as i32,
    };
    let create = CreateDocumentRequestV1 {
        operation_id: vec![0x11; 16],
        logical_owner_id: String::new(),
        title: PRIVATE_TITLE_V1.to_owned(),
        description: PRIVATE_DESCRIPTION_V1.to_owned(),
        media_type: "application/pdf".to_owned(),
        original_file_name: PRIVATE_FILE_V1.to_owned(),
        declared_size: source_blob.declared_size,
        content_sha256: source_blob.receipt_sha256.to_vec(),
        created_at: Some(timestamp),
    };
    let first: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        1,
        documents_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    let replay: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        2,
        documents_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    assert_eq!(first, replay);
    let created = first.document.expect("created Document");
    let attach = AttachDocumentBlobRequestV1 {
        operation_id: vec![0x12; 16],
        document_id: created.document_id.clone(),
        logical_owner_id: String::new(),
        expected_document_revision: 1,
        blob_reference_id: source_blob.reference_id.to_vec(),
        declared_size: source_blob.declared_size,
        content_sha256: source_blob.receipt_sha256.to_vec(),
        changed_at: Some(timestamp),
        custody_transfer_source_proof: source_blob.custody_transfer_source_proof.clone(),
        source_evidence_id: source_blob.evidence_id.to_vec(),
        source_evidence_envelope_sha256: source_blob.evidence_envelope_sha256.to_vec(),
    };
    let attached: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        3,
        documents_client_attach_blob_contract_reference_v1(),
        attach.encode_to_vec(),
    );
    let attach_replay: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        4,
        documents_client_attach_blob_contract_reference_v1(),
        attach.encode_to_vec(),
    );
    assert_eq!(attached, attach_replay);
    assert_eq!(
        attached.document.as_ref().expect("attached").custody_state,
        DocumentCustodyStateV1::DocumentCustodyStateBound as i32
    );
    let mut altered_attach = attach.clone();
    altered_attach.source_evidence_envelope_sha256 = vec![0x77; 32];
    assert_eq!(
        route_documents_response_v1(
            &store,
            &supervisor,
            &documents,
            5,
            documents_client_attach_blob_contract_reference_v1(),
            altered_attach.encode_to_vec()
        )
        .error_code,
        "CONFLICT",
    );

    let with_source: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        6,
        documents_client_add_source_contract_reference_v1(),
        AddDocumentSourceRequestV1 {
            operation_id: vec![0x13; 16],
            document_id: created.document_id.clone(),
            logical_owner_id: String::new(),
            expected_document_revision: 2,
            source_owner_id: "mail".to_owned(),
            source_record_id: PRIVATE_SOURCE_V1.to_owned(),
            source_revision: 1,
            evidence_digest: vec![0x31; 32],
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        with_source
            .document
            .as_ref()
            .expect("source")
            .document_revision,
        3
    );
    let sources: ListDocumentSourcesResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        7,
        documents_client_list_sources_contract_reference_v1(),
        ListDocumentSourcesRequestV1 {
            logical_owner_id: String::new(),
            document_id: created.document_id.clone(),
            after_source_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(sources.sources.len(), 1);
    let removed: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        8,
        documents_client_remove_source_contract_reference_v1(),
        RemoveDocumentSourceRequestV1 {
            operation_id: vec![0x14; 16],
            document_id: created.document_id.clone(),
            logical_owner_id: String::new(),
            expected_document_revision: 3,
            source_id: sources.sources[0].source_id.clone(),
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        removed
            .document
            .as_ref()
            .expect("removed")
            .document_revision,
        4
    );
    let released: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        9,
        documents_client_release_blob_contract_reference_v1(),
        ReleaseDocumentBlobRequestV1 {
            operation_id: vec![0x15; 16],
            document_id: created.document_id.clone(),
            logical_owner_id: String::new(),
            expected_document_revision: 4,
            blob_reference_id: bound_blob_reference_v1(),
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        released.document.as_ref().expect("released").custody_state,
        DocumentCustodyStateV1::DocumentCustodyStateReleased as i32
    );
    let archived: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        10,
        documents_client_set_state_contract_reference_v1(),
        SetDocumentStateRequestV1 {
            operation_id: vec![0x16; 16],
            document_id: created.document_id.clone(),
            logical_owner_id: String::new(),
            expected_document_revision: 5,
            state: DocumentStateV1::DocumentStateArchived as i32,
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        archived
            .document
            .as_ref()
            .expect("archived")
            .document_revision,
        6
    );

    let second: DocumentMutationResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        11,
        documents_client_create_contract_reference_v1(),
        CreateDocumentRequestV1 {
            operation_id: vec![0x17; 16],
            logical_owner_id: String::new(),
            title: "Second document".to_owned(),
            description: String::new(),
            media_type: "text/plain".to_owned(),
            original_file_name: "second.txt".to_owned(),
            declared_size: 1,
            content_sha256: vec![0x51; 32],
            created_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    let first_page: ListDocumentsResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        12,
        documents_client_list_contract_reference_v1(),
        ListDocumentsRequestV1 {
            logical_owner_id: String::new(),
            after_document_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let second_page: ListDocumentsResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        13,
        documents_client_list_contract_reference_v1(),
        ListDocumentsRequestV1 {
            logical_owner_id: String::new(),
            after_document_id: first_page.next_after_document_id.clone(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let mut ids = vec![
        first_page.documents[0].document_id.clone(),
        second_page.documents[0].document_id.clone(),
    ];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&created.document_id));
    assert!(ids.contains(&second.document.expect("second").document_id));
    let searched: ListDocumentsResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        14,
        documents_client_search_contract_reference_v1(),
        SearchDocumentsRequestV1 {
            logical_owner_id: String::new(),
            query: "private-description".to_owned(),
            after_document_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(searched.documents.len(), 1);

    wait_for_documents_relay_v1();
    let before_restart = durable_documents_snapshot_v1();
    assert_eq!(before_restart, (2, 1, 7, 2, 7, 0));
    assert_public_documents_outbox_is_private_free_v1(&source_blob.custody_transfer_source_proof);
    let documents =
        restart_documents_runtime_v1(&supervisor, &store, &root.join("runtime"), documents);
    let restarted: DocumentV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        15,
        documents_client_get_contract_reference_v1(),
        GetDocumentRequestV1 {
            logical_owner_id: String::new(),
            document_id: created.document_id,
        }
        .encode_to_vec(),
    );
    assert_eq!(restarted.document_revision, 6);
    assert_eq!(
        restarted.custody_state,
        DocumentCustodyStateV1::DocumentCustodyStateReleased as i32
    );
    let restarted_sources: ListDocumentSourcesResultV1 = route_documents_v1(
        &store,
        &supervisor,
        &documents,
        16,
        documents_client_list_sources_contract_reference_v1(),
        ListDocumentSourcesRequestV1 {
            logical_owner_id: String::new(),
            document_id: restarted.document_id,
            after_source_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(
        restarted_sources.sources[0].state,
        DocumentSourceStateV1::DocumentSourceStateRemoved as i32
    );
    assert_eq!(durable_documents_snapshot_v1(), before_restart);
    assert!(
        supervisor
            .is_active(&documents.registration_id)
            .expect("Documents active")
    );
    assert_eq!(
        supervisor.last_failure(&documents.registration_id),
        Ok(None)
    );
    tokio::runtime::Runtime::new()
        .expect("Documents RLS runtime")
        .block_on(assert_review_owner_rls_v1(
            "makosh_documents_rls_test",
            &[
                "documents_records",
                "documents_sources",
                "documents_client_operations",
                "documents_blob_operations",
                "documents_outbox",
            ],
        ));
    supervisor.shutdown().expect("stop Documents contour");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Documents fixture");
}

fn route_documents_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    documents: &StartedDocumentsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let response =
        route_documents_response_v1(store, supervisor, documents, request_id, contract, payload);
    assert!(
        response.error_code.is_empty(),
        "Documents request {request_id} failed: {}",
        response.error_code
    );
    T::decode(response.response_payload.as_slice()).expect("decode Documents response")
}

fn route_documents_response_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    documents: &StartedDocumentsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> ModuleClientResponseV1 {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: DOCUMENTS_MODULE_ID_V1.to_owned(),
        owner_id: DOCUMENTS_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: DOCUMENTS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &documents.registration_id,
        &documents.runtime_instance_id,
        documents.runtime_generation,
        documents.grant_epoch,
        DOCUMENTS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Documents request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("Documents response")
}

fn bound_blob_reference_v1() -> Vec<u8> {
    tokio::runtime::Runtime::new().expect("Documents SQL runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT blob_reference_id FROM makosh_data.documents_records WHERE logical_owner_id='owner-1' AND custody_state=2",
        ).fetch_one(&pool).await.expect("bound Blob reference")
    })
}

fn wait_for_documents_relay_v1() {
    let deadline = Instant::now() + Duration::from_secs(15);
    while durable_documents_snapshot_v1().5 != 0 {
        assert!(Instant::now() < deadline, "Documents relay did not drain");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn durable_documents_snapshot_v1() -> (i64, i64, i64, i64, i64, i64) {
    tokio::runtime::Runtime::new().expect("Documents SQL runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        sqlx::query_as(
            "SELECT (SELECT count(*) FROM makosh_data.documents_records WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.documents_sources WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.documents_client_operations WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.documents_blob_operations WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.documents_outbox WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.documents_outbox WHERE logical_owner_id='owner-1' AND published_at_unix_millis IS NULL)",
        ).fetch_one(&pool).await.expect("Documents durable snapshot")
    })
}

fn assert_public_documents_outbox_is_private_free_v1(proof: &[u8]) {
    tokio::runtime::Runtime::new().expect("Documents privacy runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        let rows: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT envelope_bytes FROM makosh_data.documents_outbox WHERE logical_owner_id='owner-1' ORDER BY outbox_sequence",
        ).fetch_all(&pool).await.expect("Documents outbox bytes");
        assert!(!rows.is_empty());
        for row in rows {
            for marker in [PRIVATE_TITLE_V1.as_bytes(), PRIVATE_DESCRIPTION_V1.as_bytes(), PRIVATE_FILE_V1.as_bytes(), PRIVATE_SOURCE_V1.as_bytes(), PRIVATE_BLOB_V1, proof] {
                assert!(!row.windows(marker.len()).any(|window| window == marker), "private Documents marker escaped public outbox");
            }
        }
    });
}
