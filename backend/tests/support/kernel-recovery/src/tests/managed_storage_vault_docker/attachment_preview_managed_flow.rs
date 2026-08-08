//! Full managed Preview custody, Gateway, client Blob, SSE and restart conformance.

use super::*;
use super::{
    attachment_preview_gateway_fixture::{
        AttachmentPreviewGateway, attachment_preview_gateway_v1, get_attachment_preview_v1,
        post_attachment_preview_proto_status_v1, post_attachment_preview_proto_v1,
        read_attachment_preview_blob_v1, read_terminal_attachment_preview_sse_event_after_v1,
        read_terminal_attachment_preview_sse_event_v1, wait_for_ready_attachment_preview_v1,
    },
    attachment_preview_managed_formats::assert_managed_attachment_preview_formats_v1,
    attachment_preview_persistence_fixture::{
        expire_attachment_preview_job_lease_v1, expire_attachment_preview_ticket_v1,
        replace_attachment_preview_job_source_receipt_v1,
        replace_attachment_preview_renderer_identity_v1,
        replace_attachment_preview_state_revision_v1,
    },
    attachment_security_blob_fixture::AttachmentSecurityBlobSourceFixture,
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    attachment_security_event_flow::{
        assert_clean_attachment_security_verdict_flow, prepare_communications_attachment_for_scan,
    },
    mail_attachment_flow::wait_for_attachment_state,
};

use crate::identity::device::signer::DeviceSigner;
use hyper::StatusCode;
use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_COMMAND_CONTRACT_NAME_V1,
    ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1, ATTACHMENT_PREVIEW_CONTRACT_REVISION_V1,
    ATTACHMENT_PREVIEW_CONTROL_SCHEMA_SHA256, ATTACHMENT_PREVIEW_MODULE_ID_V1,
    ATTACHMENT_PREVIEW_OWNER_V1, ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewContentTypeV1, AttachmentPreviewErrorCodeV1, AttachmentPreviewKindV1,
        AttachmentPreviewStateV1, AttachmentPreviewStatusChangedV1,
        IssueAttachmentPreviewReadRequestV1, IssueAttachmentPreviewReadResponseV1,
        StartAttachmentPreviewRequestV1, StartAttachmentPreviewResponseV1,
    },
};
use makosh_attachment_preview_runtime::admission::ATTACHMENT_PREVIEW_CLIENT_CAPABILITY_ID_V1;
use makosh_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1};

const PRIVATE_SOURCE: &[u8] =
    b"Private clean-room preview payload.\r\nThe bytes must stay outside query and SSE.";
const EXPECTED_PREVIEW: &[u8] =
    b"Private clean-room preview payload.\nThe bytes must stay outside query and SSE.";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, Attachment Security and Preview binaries"]
fn managed_attachment_preview_reaches_gateway_blob_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-attachment-preview");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_preview_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Attachment Preview logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
    );
    super::super::browser_gateway_session::admit_secondary_browser_test_device(
        &store,
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
    );

    let admitted_preview = admit_attachment_preview_runtime_v1(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_attachment_preview_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    let admitted_preview =
        prepare_attachment_preview_runtime_v1(&supervisor, &store, admitted_preview);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mut security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    let mut preview = start_attachment_preview_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_preview,
    );
    assert_eq!(security.runtime_generation, 1);
    assert_eq!(preview.runtime_generation, 1);

    let blob = blob_source.write(&store, &supervisor, &data, [0xA1; 16], PRIVATE_SOURCE);
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "attachment-preview-text",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(
        &store,
        &attachment,
        &blob,
        &clamav,
        PRIVATE_SOURCE,
    );
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    assert_attachment_preview_runtime_fences_v1(
        &store,
        &supervisor,
        &preview,
        attachment.attachment_anchor_id,
    );
    wait_for_attachment_preview_evidence_v1();

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = attachment_preview_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let request = StartAttachmentPreviewRequestV1 {
        protocol_major: 1,
        operation_id: vec![0xA2; 16],
        attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
    };
    set_authenticated_nats_container_running(false);
    let accepted = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(
        accepted.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(accepted.run_id.len(), 16);
    wait_for_pending_attachment_preview_custody_outbox_v1();
    set_authenticated_nats_container_running(true);

    let ready = wait_for_ready_attachment_preview_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
        "text",
    );
    assert_eq!(ready.attachment_anchor_id, attachment.attachment_anchor_id);
    assert_eq!(ready.state, AttachmentPreviewStateV1::Ready as i32);
    assert_eq!(ready.preview_kind, AttachmentPreviewKindV1::Text as i32);
    assert_eq!(
        ready.content_type,
        AttachmentPreviewContentTypeV1::TextUtf8 as i32
    );
    assert_eq!(ready.preview_size_bytes, EXPECTED_PREVIEW.len() as u64);
    assert!(!ready.truncated);
    assert!(
        !ready
            .encode_to_vec()
            .windows(PRIVATE_SOURCE.len())
            .any(|window| window == PRIVATE_SOURCE)
    );

    let first_event = read_terminal_attachment_preview_sse_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
    );
    let first_payload = AttachmentPreviewStatusChangedV1::decode(first_event.payload.as_slice())
        .expect("Attachment Preview realtime payload");
    assert_eq!(first_payload.state, AttachmentPreviewStateV1::Ready as i32);
    assert!(
        !first_event
            .encode_to_vec()
            .windows(PRIVATE_SOURCE.len())
            .any(|window| window == PRIVATE_SOURCE)
    );
    let completed = attachment_preview_diagnostics_v1();
    assert_eq!(
        completed,
        AttachmentPreviewDiagnosticsV1 {
            candidates: 1,
            safety_facts: 1,
            custody_requests: 1,
            pending_custody_outbox: 0,
            custody_results: 1,
            jobs: 1,
            attempts: 1,
            artifacts: 1,
            security_delegation_commands: 1,
            security_delegation_attempts: 1,
            security_delegation_results: 1,
        }
    );
    let duplicate = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(duplicate.state, AttachmentPreviewStateV1::Ready as i32);
    assert_eq!(attachment_preview_diagnostics_v1(), completed);
    let conflicting = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            attachment_anchor_id: vec![0xA3; 16],
            ..request
        },
    );
    assert_eq!(
        conflicting.error,
        AttachmentPreviewErrorCodeV1::InvalidRequest as i32
    );
    assert_eq!(attachment_preview_diagnostics_v1(), completed);

    let ticket = post_attachment_preview_proto_v1::<_, IssueAttachmentPreviewReadResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(
        ticket.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(ticket.opaque_read_ticket.len(), 32);
    let secondary_cookie =
        super::super::browser_gateway_session::authenticate_secondary_gateway_router(
            &router,
            &gateway_runtime,
        );
    let (wrong_actor_status, wrong_actor_body) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&secondary_cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(wrong_actor_status, StatusCode::NOT_FOUND);
    assert!(wrong_actor_body.is_empty());
    let (status, body) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, EXPECTED_PREVIEW);
    let (replay_status, _) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket,
    );
    assert_eq!(replay_status, StatusCode::NOT_FOUND);

    assert_managed_attachment_preview_formats_v1(
        &store,
        &supervisor,
        &data,
        &blob_source,
        &clamav,
        &router,
        &gateway_runtime,
        &cookie,
    );

    let stale_proof_baseline = attachment_preview_diagnostics_v1();
    let stale_proof_source =
        b"Private Preview source whose delegated custody proof must become stale.";
    let stale_proof_blob =
        blob_source.write(&store, &supervisor, &data, [0xf1; 16], stale_proof_source);
    let stale_proof_attachment = prepare_communications_attachment_for_scan(
        &store,
        "preview-stale-custody-proof",
        stale_proof_blob.declared_size,
        stale_proof_blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(
        &store,
        &stale_proof_attachment,
        &stale_proof_blob,
        &clamav,
        stale_proof_source,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            stale_proof_attachment.attachment_anchor_id,
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    wait_for_attachment_preview_evidence_counts_v1(
        stale_proof_baseline.candidates + 1,
        stale_proof_baseline.safety_facts + 1,
    );
    supervisor
        .stop(&security.registration_id)
        .expect("stop Attachment Security before stale Preview custody request");
    let stale_proof_run = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xf2; 16],
            attachment_anchor_id: stale_proof_attachment.attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(
        stale_proof_run.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    wait_for_attachment_preview_custody_request_v1(stale_proof_baseline.custody_requests + 1);
    supervisor
        .stop(&preview.registration_id)
        .expect("stop Preview before stale custody result delivery");
    security = restart_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        &security,
        clamav.port(),
    );
    wait_for_attachment_preview_security_result_v1(
        stale_proof_baseline.security_delegation_results + 1,
    );
    let proof_source_grant_epoch = security.grant_epoch;
    let security_capabilities = store
        .module_grant_snapshot(&security.registration_id)
        .expect("read Attachment Security grants before stale Preview proof fence")
        .expect("Attachment Security grant snapshot")
        .effective_grants()
        .expect("approved Attachment Security grants")
        .capability_ids()
        .to_vec();
    supervisor
        .stop(&security.registration_id)
        .expect("stop Attachment Security before Preview proof grant replacement");
    store
        .transition_module_registration(
            &security.registration_id,
            ModuleRegistrationState::Suspended,
        )
        .expect("suspend Attachment Security after Preview custody proof issue");
    let reapproved_security = store
        .approve_module_registration(&security.registration_id, &security_capabilities)
        .expect("reapprove Attachment Security with exact capabilities");
    assert!(reapproved_security.grant_epoch() > proof_source_grant_epoch);
    security = restart_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        &security,
        clamav.port(),
    );
    assert_eq!(security.grant_epoch, reapproved_security.grant_epoch());
    preview =
        restart_attachment_preview_runtime_v1(&supervisor, &store, &root.join("runtime"), preview);
    wait_for_attachment_preview_failed_job_attempt_v1(
        stale_proof_baseline.jobs + 1,
        stale_proof_baseline.attempts + 1,
        stale_proof_baseline.artifacts,
        "stale Preview custody proof",
    );
    let stale_proof_status =
        get_attachment_preview_v1(&router, &gateway_runtime, &cookie, &stale_proof_run.run_id);
    assert_ne!(
        stale_proof_status.state,
        AttachmentPreviewStateV1::Ready as i32
    );
    assert_private_preview_source_absent_v1(
        &stale_proof_status.encode_to_vec(),
        stale_proof_source,
        "stale custody proof status",
    );

    let source_hash_baseline = attachment_preview_diagnostics_v1();
    let source_hash_source = b"Private Preview source protected by its exact SHA-256 receipt.";
    let source_hash_blob =
        blob_source.write(&store, &supervisor, &data, [0xf3; 16], source_hash_source);
    let source_hash_attachment = prepare_communications_attachment_for_scan(
        &store,
        "attachment-preview-source-hash",
        source_hash_blob.declared_size,
        source_hash_blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(
        &store,
        &source_hash_attachment,
        &source_hash_blob,
        &clamav,
        source_hash_source,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            source_hash_attachment.attachment_anchor_id,
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    wait_for_attachment_preview_evidence_counts_v1(
        source_hash_baseline.candidates + 1,
        source_hash_baseline.safety_facts + 1,
    );
    supervisor
        .stop(&security.registration_id)
        .expect("stop Attachment Security before source-hash Preview request");
    let source_hash_run = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xf4; 16],
            attachment_anchor_id: source_hash_attachment.attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(
        source_hash_run.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    wait_for_attachment_preview_custody_request_v1(source_hash_baseline.custody_requests + 1);
    supervisor
        .stop(&preview.registration_id)
        .expect("stop Preview before source-hash custody result delivery");
    let _source_hash_security = restart_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        &security,
        clamav.port(),
    );
    wait_for_attachment_preview_security_result_v1(
        source_hash_baseline.security_delegation_results + 1,
    );
    supervisor
        .stop(blob_binding::BLOB_PROCESS_ID)
        .expect("stop Blob before source-hash Preview attempt");
    preview =
        restart_attachment_preview_runtime_v1(&supervisor, &store, &root.join("runtime"), preview);
    wait_for_attachment_preview_failed_job_attempt_v1(
        source_hash_baseline.jobs + 1,
        source_hash_baseline.attempts + 1,
        source_hash_baseline.artifacts,
        "source-hash setup Blob outage",
    );
    let replacement_receipt_sha256 = [0x5c; 32];
    assert_ne!(source_hash_blob.receipt_sha256, replacement_receipt_sha256);
    replace_attachment_preview_job_source_receipt_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &source_hash_run.run_id,
        source_hash_blob.receipt_sha256,
        replacement_receipt_sha256,
    );
    expire_attachment_preview_job_lease_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &source_hash_run.run_id,
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime for source-hash Preview fence"),
        2
    );
    wait_for_attachment_preview_failed_job_attempt_v1(
        source_hash_baseline.jobs + 1,
        source_hash_baseline.attempts + 2,
        source_hash_baseline.artifacts,
        "mismatched Preview source receipt",
    );
    let source_hash_status =
        get_attachment_preview_v1(&router, &gateway_runtime, &cookie, &source_hash_run.run_id);
    assert_ne!(
        source_hash_status.state,
        AttachmentPreviewStateV1::Ready as i32
    );
    assert_private_preview_source_absent_v1(
        &source_hash_status.encode_to_vec(),
        source_hash_source,
        "source-hash status",
    );

    let accepted = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xf5; 16],
            attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(
        accepted.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    let ready = wait_for_ready_attachment_preview_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
        "current-generation ticket fences",
    );
    let first_event = read_terminal_attachment_preview_sse_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
    );
    let first_cursor = first_event.cursor.clone();

    let current_renderer_identity =
        makosh_attachment_preview_runtime::attachment_preview_renderer_identity_v1();
    let stale_renderer_identity = [0x79; 32];
    assert_ne!(current_renderer_identity, stale_renderer_identity);
    let renderer_ticket =
        issue_attachment_preview_ticket_v1(&router, &gateway_runtime, &cookie, &accepted.run_id);
    replace_attachment_preview_renderer_identity_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        current_renderer_identity,
        stale_renderer_identity,
    );
    assert_failed_attachment_preview_blob_read_v1(
        &router,
        &gateway_runtime,
        &cookie,
        renderer_ticket,
        StatusCode::NOT_FOUND,
        "stale renderer identity",
    );
    replace_attachment_preview_renderer_identity_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        stale_renderer_identity,
        current_renderer_identity,
    );

    let stale_revision_ticket =
        issue_attachment_preview_ticket_v1(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let stale_revision = ready
        .state_revision
        .checked_add(1)
        .expect("bounded stale Preview revision");
    replace_attachment_preview_state_revision_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        ready.state_revision,
        stale_revision,
    );
    assert_failed_attachment_preview_blob_read_v1(
        &router,
        &gateway_runtime,
        &cookie,
        stale_revision_ticket,
        StatusCode::NOT_FOUND,
        "stale state revision",
    );
    replace_attachment_preview_state_revision_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        stale_revision,
        ready.state_revision,
    );

    let expired_ticket =
        issue_attachment_preview_ticket_v1(&router, &gateway_runtime, &cookie, &accepted.run_id);
    expire_attachment_preview_ticket_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        &expired_ticket,
    );
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_failed_attachment_preview_blob_read_v1(
        &router,
        &gateway_runtime,
        &cookie,
        expired_ticket,
        StatusCode::NOT_FOUND,
        "expired ticket",
    );

    let stale_generation_ticket =
        issue_attachment_preview_ticket_v1(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let blob_outage_ticket =
        issue_attachment_preview_ticket_v1(&router, &gateway_runtime, &cookie, &accepted.run_id);
    supervisor
        .stop(blob_binding::BLOB_PROCESS_ID)
        .expect("stop Blob for Preview client-blob outage");
    assert_failed_attachment_preview_blob_read_v1(
        &router,
        &gateway_runtime,
        &cookie,
        blob_outage_ticket,
        StatusCode::SERVICE_UNAVAILABLE,
        "Blob outage",
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after Preview outage"),
        3
    );

    assert!(
        realtime
            .revoke_owner(ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1)
            .expect("clear Attachment Preview Gateway replay cache")
    );
    let previous_generation = preview.runtime_generation;
    preview =
        restart_attachment_preview_runtime_v1(&supervisor, &store, &root.join("runtime"), preview);
    assert_eq!(preview.runtime_generation, previous_generation + 1);
    let restarted_router =
        attachment_preview_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    assert_eq!(
        get_attachment_preview_v1(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            &accepted.run_id,
        ),
        ready
    );
    let replayed_event = read_terminal_attachment_preview_sse_event_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_cursor);
    assert_eq!(replayed_event.payload, first_event.payload);
    assert_failed_attachment_preview_blob_read_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        stale_generation_ticket,
        StatusCode::NOT_FOUND,
        "stale runtime generation",
    );

    let post_restart = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xA4; 16],
            attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(
        post_restart.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    wait_for_ready_attachment_preview_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &post_restart.run_id,
        "post-restart Vault outage",
    );
    let continued_event = read_terminal_attachment_preview_sse_event_after_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        Some(&first_cursor),
        &post_restart.run_id,
    );
    assert_ne!(continued_event.cursor, first_cursor);
    let continued_payload =
        AttachmentPreviewStatusChangedV1::decode(continued_event.payload.as_slice())
            .expect("continued Attachment Preview realtime payload");
    assert_eq!(continued_payload.run_id, post_restart.run_id);
    assert_eq!(
        continued_payload.state,
        AttachmentPreviewStateV1::Ready as i32
    );
    assert_private_preview_source_absent_v1(
        &continued_event.encode_to_vec(),
        PRIVATE_SOURCE,
        "Last-Event-ID continuation",
    );
    let vault_outage_ticket = issue_attachment_preview_ticket_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &post_restart.run_id,
    );
    supervisor
        .stop("vault")
        .expect("stop Vault for Preview client-blob outage");
    assert_failed_attachment_preview_blob_read_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        vault_outage_ticket,
        StatusCode::SERVICE_UNAVAILABLE,
        "Vault outage",
    );

    let unauthenticated = post_attachment_preview_proto_status_v1(
        &restarted_router,
        &gateway_runtime,
        None,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id,
        },
    );
    assert_eq!(unauthenticated, StatusCode::UNAUTHORIZED);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn issue_attachment_preview_ticket_v1(
    router: &AttachmentPreviewGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> Vec<u8> {
    let ticket = post_attachment_preview_proto_v1::<_, IssueAttachmentPreviewReadResponseV1>(
        router,
        runtime,
        cookie,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    );
    assert_eq!(
        ticket.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(ticket.opaque_read_ticket.len(), 32);
    ticket.opaque_read_ticket
}

fn assert_failed_attachment_preview_blob_read_v1(
    router: &AttachmentPreviewGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    ticket: Vec<u8>,
    expected_status: StatusCode,
    scenario: &str,
) {
    let (status, body) = read_attachment_preview_blob_v1(router, runtime, Some(cookie), ticket);
    assert_eq!(status, expected_status, "{scenario}");
    assert!(
        !body
            .windows(PRIVATE_SOURCE.len())
            .any(|window| window == PRIVATE_SOURCE),
        "Preview source bytes escaped through failed client-blob response: {scenario}"
    );
}

fn wait_for_attachment_preview_evidence_v1() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.candidates == 1 && diagnostics.safety_facts == 1 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview did not consume source evidence: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn wait_for_attachment_preview_evidence_counts_v1(
    expected_candidates: i64,
    expected_safety_facts: i64,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.candidates == expected_candidates
            && diagnostics.safety_facts == expected_safety_facts
        {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview did not consume expected source evidence: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn wait_for_attachment_preview_custody_request_v1(expected_count: i64) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.custody_requests == expected_count && diagnostics.pending_custody_outbox == 0
        {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview did not publish custody request {expected_count}: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn wait_for_attachment_preview_security_result_v1(expected_count: i64) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.security_delegation_results == expected_count {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Security did not publish Preview custody result {expected_count}: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn wait_for_attachment_preview_failed_job_attempt_v1(
    expected_jobs: i64,
    expected_attempts: i64,
    expected_artifacts: i64,
    scenario: &str,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.jobs == expected_jobs
            && diagnostics.attempts == expected_attempts
            && diagnostics.artifacts == expected_artifacts
        {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview failure fence did not settle for {scenario}: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn assert_private_preview_source_absent_v1(carrier: &[u8], source: &[u8], scenario: &str) {
    assert!(
        !carrier.windows(source.len()).any(|window| window == source),
        "Preview source bytes escaped through {scenario}"
    );
}

fn wait_for_pending_attachment_preview_custody_outbox_v1() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.custody_requests == 1 && diagnostics.pending_custody_outbox == 1 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview did not retain its custody command during NATS outage: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn assert_attachment_preview_runtime_fences_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    preview: &StartedAttachmentPreviewRuntimeV1,
    attachment_anchor_id: [u8; 16],
) {
    let request = attachment_preview_module_request_v1(
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
        700,
        [0x8a; 16],
        attachment_anchor_id,
    );
    for (runtime_generation, grant_epoch, label) in [
        (
            preview.runtime_generation + 1,
            preview.grant_epoch,
            "stale Attachment Preview runtime generation",
        ),
        (
            preview.runtime_generation,
            preview.grant_epoch + 1,
            "stale Attachment Preview grant epoch",
        ),
    ] {
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            &preview.registration_id,
            &preview.runtime_instance_id,
            runtime_generation,
            grant_epoch,
            ATTACHMENT_PREVIEW_CLIENT_CAPABILITY_ID_V1,
            &request,
        );
        assert_eq!(
            crate::modules::capability::router::route_managed_client_request(
                store,
                &supervisor.relay_port(),
                &route,
            )
            .expect_err(label),
            "managed runtime fence is stale"
        );
    }
}

fn attachment_preview_module_request_v1(
    logical_owner_id: &str,
    request_id: u64,
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
) -> Vec<u8> {
    ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: ATTACHMENT_PREVIEW_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
            name: ATTACHMENT_PREVIEW_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_PREVIEW_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_PREVIEW_CONTROL_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: operation_id.to_vec(),
            attachment_anchor_id: attachment_anchor_id.to_vec(),
        }
        .encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec()
}
