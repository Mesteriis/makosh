//! Full managed Text Extraction custody, Gateway, content and SSE conformance.

use std::time::{Duration, Instant};

use super::*;
use super::{
    attachment_security_blob_fixture::{
        AttachmentSecurityBlobSourceFixture, AttachmentSecurityFixtureBlobV1,
    },
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    attachment_security_event_flow::{
        assert_clean_attachment_security_verdict_flow, prepare_communications_attachment_for_scan,
    },
    attachment_text_extraction_gateway_fixture::{
        attachment_text_extraction_gateway_v1, get_attachment_text_extraction_v1,
        post_attachment_text_proto_status_v1, post_attachment_text_proto_v1,
        read_terminal_attachment_text_sse_event_v1, wait_for_ready_attachment_text_v1,
        wait_for_terminal_attachment_text_v1,
    },
    attachment_text_extraction_persistence_fixture::{
        AttachmentTextExtractionDiagnosticsV1, attachment_text_extraction_diagnostics_v1,
        replace_attachment_text_parser_identity_v1,
    },
    attachment_text_extraction_source_fixtures::{
        attachment_text_docx_source_v1, attachment_text_ocr_png_source_v1,
        attachment_text_pdf_source_v1,
    },
    mail_attachment_flow::wait_for_attachment_state,
};

use crate::identity::device::signer::DeviceSigner;
use hyper::StatusCode;
use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
    ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
    ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1, ATTACHMENT_TEXT_EXTRACTION_CONTRACT_REVISION_V1,
    ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1, ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
    ATTACHMENT_TEXT_EXTRACTION_QUERY_CONNECT_PATH_V1, ATTACHMENT_TEXT_EXTRACTION_SCHEMA_SHA256,
    wire::{
        AttachmentTextExtractionErrorCodeV1, AttachmentTextExtractionStateV1,
        AttachmentTextExtractionStatusChangedV1, AttachmentTextFormatV1,
        GetAttachmentTextExtractionRequestV1, ReadAttachmentTextRequestV1,
        ReadAttachmentTextResponseV1, StartAttachmentTextExtractionRequestV1,
        StartAttachmentTextExtractionResponseV1,
    },
};
use makosh_attachment_text_extraction_runtime::AttachmentTextExtractionParserRuntimeV1;
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};

const PRIVATE_SOURCE_TEXT: &[u8] =
    b"Private clean-room attachment text that must stay out of events and SSE.\r\nLine two.";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, Attachment Security and Text Extraction binaries"]
fn managed_attachment_text_extraction_completes_through_gateway_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-attachment-text-extraction");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_text_extraction_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Attachment Text Extraction logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1,
    );

    let admitted_text = admit_attachment_text_extraction_runtime_v1(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_attachment_text_extraction_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted_text =
        prepare_attachment_text_extraction_runtime_v1(&supervisor, &store, admitted_text);
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mut security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    let text = start_attachment_text_extraction_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_text,
    );
    assert_eq!(security.runtime_generation, 1);
    assert_eq!(text.runtime_generation, 1);
    assert!(
        text.capability_ids
            .iter()
            .any(|capability| capability == "attachment_text_extraction.ocr_runtime.v1")
    );

    let blob = blob_source.write(&store, &supervisor, &data, [0xa1; 16], PRIVATE_SOURCE_TEXT);
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "text-extraction",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(
        &store,
        &attachment,
        &blob,
        &clamav,
        PRIVATE_SOURCE_TEXT,
    );
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    wait_for_attachment_text_evidence_v1(1);

    assert_attachment_text_runtime_fences_v1(
        &store,
        &supervisor,
        &text,
        attachment.attachment_anchor_id,
    );
    let wrong_owner = route_attachment_text_start_as_v1(
        &store,
        &supervisor,
        &text,
        "owner-2",
        801,
        [0xa2; 16],
        attachment.attachment_anchor_id,
    );
    assert_eq!(wrong_owner.request_id, 801);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router =
        attachment_text_extraction_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let request = StartAttachmentTextExtractionRequestV1 {
        protocol_major: 1,
        operation_id: vec![0xa3; 16],
        attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
    };
    assert_eq!(
        post_attachment_text_proto_status_v1(
            &router,
            &gateway_runtime,
            None,
            ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
            request.clone(),
        ),
        StatusCode::UNAUTHORIZED,
        "Text Extraction Start must require an authenticated Gateway session"
    );

    set_authenticated_nats_container_running(false);
    let accepted = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.error, unspecified_error_v1());
    assert_eq!(accepted.run_id.len(), 16);
    wait_for_pending_attachment_text_custody_v1();
    set_authenticated_nats_container_running(true);

    let ready =
        wait_for_ready_attachment_text_v1(&router, &gateway_runtime, &cookie, &accepted.run_id);
    assert_eq!(ready.attachment_anchor_id, attachment.attachment_anchor_id);
    assert_eq!(
        text_state_v1(ready.state),
        AttachmentTextExtractionStateV1::Ready
    );
    assert_eq!(
        text_format_v1(ready.format),
        AttachmentTextFormatV1::PlainUtf8
    );
    assert_eq!(ready.extracted_size_bytes, 82);
    assert!(!ready.extraction_truncated);
    assert_eq!(ready.error, unspecified_error_v1());
    assert_private_attachment_text_absent_v1(&ready.encode_to_vec(), PRIVATE_SOURCE_TEXT, &blob);

    let read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(read.error, unspecified_error_v1());
    assert_eq!(read.run_id, accepted.run_id);
    assert_eq!(
        read.text_utf8,
        b"Private clean-room attachment text that must stay out of events and SSE.\nLine two."
    );
    assert_eq!(read.extracted_size_bytes, 82);
    assert!(!read.visible_truncated);

    let first_event = read_terminal_attachment_text_sse_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
    );
    let first_payload =
        AttachmentTextExtractionStatusChangedV1::decode(first_event.payload.as_slice())
            .expect("Attachment Text Extraction realtime payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    assert_eq!(
        text_state_v1(first_payload.state),
        AttachmentTextExtractionStateV1::Ready
    );
    assert_private_attachment_text_absent_v1(
        &first_event.encode_to_vec(),
        PRIVATE_SOURCE_TEXT,
        &blob,
    );
    let first_cursor = first_event.cursor.clone();

    let completed = attachment_text_extraction_diagnostics_v1();
    assert_eq!(
        completed,
        AttachmentTextExtractionDiagnosticsV1 {
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
    let duplicate = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(
        text_state_v1(duplicate.state),
        AttachmentTextExtractionStateV1::Ready
    );
    assert_eq!(attachment_text_extraction_diagnostics_v1(), completed);

    let conflicting = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
        StartAttachmentTextExtractionRequestV1 {
            attachment_anchor_id: vec![0xa4; 16],
            ..request
        },
    );
    assert_eq!(
        text_error_v1(conflicting.error),
        AttachmentTextExtractionErrorCodeV1::InvalidRequest
    );
    assert_eq!(attachment_text_extraction_diagnostics_v1(), completed);

    assert!(
        realtime
            .revoke_owner(ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1)
            .expect("clear Attachment Text Extraction Gateway replay cache")
    );
    let previous_generation = text.runtime_generation;
    let mut text = restart_attachment_text_extraction_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        text,
    );
    assert_eq!(text.runtime_generation, previous_generation + 1);
    let restarted_router =
        attachment_text_extraction_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = get_attachment_text_extraction_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed, ready);
    let replayed_read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(replayed_read, read);
    let replayed_event = read_terminal_attachment_text_sse_event_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_cursor);
    assert_eq!(replayed_event.payload, first_event.payload);
    assert_private_attachment_text_absent_v1(
        &replayed_event.encode_to_vec(),
        PRIVATE_SOURCE_TEXT,
        &blob,
    );
    assert_eq!(
        attachment_text_extraction_diagnostics_v1(),
        completed,
        "restart and SSE replay must not transfer custody or parse the attachment twice"
    );

    for (
        expected_count,
        scenario_id,
        blob_id,
        operation_id,
        source,
        expected_format,
        expected_text,
    ) in [
        (
            2_i64,
            "text-extraction-pdf",
            [0xb1; 16],
            [0xb2; 16],
            attachment_text_pdf_source_v1("Макошь managed PDF"),
            AttachmentTextFormatV1::Pdf,
            "Макошь managed PDF",
        ),
        (
            3_i64,
            "text-extraction-docx",
            [0xc1; 16],
            [0xc2; 16],
            attachment_text_docx_source_v1("Макошь managed DOCX"),
            AttachmentTextFormatV1::Docx,
            "Макошь managed DOCX",
        ),
        (
            4_i64,
            "text-extraction-ocr",
            [0xd1; 16],
            [0xd2; 16],
            attachment_text_ocr_png_source_v1(),
            AttachmentTextFormatV1::Ocr,
            "MAKOSH",
        ),
    ] {
        eprintln!("managed_attachment_text_extraction_scenario={scenario_id}");
        let scenario_blob =
            blob_source.write(&store, &supervisor, &data, blob_id, source.as_slice());
        let scenario_attachment = prepare_communications_attachment_for_scan(
            &store,
            scenario_id,
            scenario_blob.declared_size,
            scenario_blob.receipt_sha256,
        );
        assert_clean_attachment_security_verdict_flow(
            &store,
            &scenario_attachment,
            &scenario_blob,
            &clamav,
            source.as_slice(),
        );
        assert_eq!(
            wait_for_attachment_state(
                &store,
                &supervisor,
                scenario_attachment.attachment_anchor_id,
            ),
            makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
                as u32
        );
        wait_for_attachment_text_evidence_v1(expected_count);
        let accepted = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
            StartAttachmentTextExtractionRequestV1 {
                protocol_major: 1,
                operation_id: operation_id.to_vec(),
                attachment_anchor_id: scenario_attachment.attachment_anchor_id.to_vec(),
            },
        );
        assert_eq!(accepted.error, unspecified_error_v1());
        let ready = wait_for_ready_attachment_text_v1(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            &accepted.run_id,
        );
        assert_eq!(text_format_v1(ready.format), expected_format);
        assert_private_attachment_text_absent_v1(
            &ready.encode_to_vec(),
            source.as_slice(),
            &scenario_blob,
        );
        let read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
            ReadAttachmentTextRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id.clone(),
            },
        );
        assert_eq!(read.error, unspecified_error_v1());
        assert!(
            String::from_utf8_lossy(&read.text_utf8).contains(expected_text),
            "{scenario_id} extracted text did not contain {expected_text:?}: {:?}",
            String::from_utf8_lossy(&read.text_utf8),
        );
        let event = read_terminal_attachment_text_sse_event_v1(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            &accepted.run_id,
        );
        assert_private_attachment_text_absent_v1(
            &event.encode_to_vec(),
            source.as_slice(),
            &scenario_blob,
        );
    }

    supervisor
        .stop(blob_binding::BLOB_PROCESS_ID)
        .expect("stop Blob for Text Extraction private-content outage");
    let unavailable_read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(
        text_error_v1(unavailable_read.error),
        AttachmentTextExtractionErrorCodeV1::Unavailable
    );
    assert!(unavailable_read.text_utf8.is_empty());
    assert_eq!(attachment_text_extraction_diagnostics_v1().artifacts, 4);
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after Text Extraction outage"),
        2
    );
    let recovered_read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(recovered_read, read);

    let stale_proof_source =
        b"Private source whose delegated custody proof must fail after source restart.";
    let stale_proof_blob =
        blob_source.write(&store, &supervisor, &data, [0xe7; 16], stale_proof_source);
    let stale_proof_attachment = prepare_communications_attachment_for_scan(
        &store,
        "text-extraction-stale-custody-proof",
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
    wait_for_attachment_text_evidence_v1(5);
    supervisor
        .stop(&security.registration_id)
        .expect("stop Attachment Security before stale Text custody request");
    let stale_proof_run = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
        StartAttachmentTextExtractionRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xe8; 16],
            attachment_anchor_id: stale_proof_attachment.attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(stale_proof_run.error, unspecified_error_v1());
    wait_for_attachment_text_custody_request_v1(5);
    supervisor
        .stop(&text.registration_id)
        .expect("stop Text Extraction before stale custody result delivery");
    security = restart_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        &security,
        clamav.port(),
    );
    wait_for_attachment_text_security_delegation_result_v1(5);
    let proof_source_grant_epoch = security.grant_epoch;
    let security_capabilities = store
        .module_grant_snapshot(&security.registration_id)
        .expect("read Attachment Security grants before stale-proof fence")
        .expect("Attachment Security grant snapshot")
        .effective_grants()
        .expect("approved Attachment Security grants")
        .capability_ids()
        .to_vec();
    supervisor
        .stop(&security.registration_id)
        .expect("stop Attachment Security before stale-proof grant replacement");
    store
        .transition_module_registration(
            &security.registration_id,
            ModuleRegistrationState::Suspended,
        )
        .expect("suspend Attachment Security after custody proof issue");
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
    text = restart_attachment_text_extraction_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        text,
    );
    wait_for_attachment_text_stale_proof_failure_v1(5, 4);
    let stale_proof_status = get_attachment_text_extraction_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &stale_proof_run.run_id,
    );
    assert_ne!(
        text_state_v1(stale_proof_status.state),
        AttachmentTextExtractionStateV1::Ready
    );
    assert_private_attachment_text_absent_v1(
        &stale_proof_status.encode_to_vec(),
        stale_proof_source,
        &stale_proof_blob,
    );

    let negative = ManagedAttachmentTextNegativeContourV1 {
        store: &store,
        supervisor: &supervisor,
        data: &data,
        clamav: &clamav,
        blob_source: &blob_source,
        router: &restarted_router,
        gateway_runtime: &gateway_runtime,
        cookie: &restarted_cookie,
    };
    negative.assert_terminal_failure_v1(ManagedAttachmentTextFailureScenarioV1 {
        expected_count: 6,
        scenario_id: "text-extraction-malformed-pdf",
        blob_id: [0xe1; 16],
        operation_id: [0xe2; 16],
        source: b"%PDF-1.7\ninvalid".to_vec(),
        expected_state: AttachmentTextExtractionStateV1::Rejected,
        expected_error: AttachmentTextExtractionErrorCodeV1::ParserFailed,
    });
    negative.assert_terminal_failure_v1(ManagedAttachmentTextFailureScenarioV1 {
        expected_count: 7,
        scenario_id: "text-extraction-unsupported",
        blob_id: [0xe3; 16],
        operation_id: [0xe4; 16],
        source: vec![0xff, 0xfe, 0xfd],
        expected_state: AttachmentTextExtractionStateV1::Unsupported,
        expected_error: AttachmentTextExtractionErrorCodeV1::Unsupported,
    });
    remove_staged_attachment_text_extraction_ocr_runner_v1(
        &root.join("runtime"),
        text.runtime_generation,
    );
    negative.assert_terminal_failure_v1(ManagedAttachmentTextFailureScenarioV1 {
        expected_count: 8,
        scenario_id: "text-extraction-parser-unavailable",
        blob_id: [0xe5; 16],
        operation_id: [0xe6; 16],
        source: attachment_text_ocr_png_source_v1(),
        expected_state: AttachmentTextExtractionStateV1::Rejected,
        expected_error: AttachmentTextExtractionErrorCodeV1::ParserUnavailable,
    });

    let current_parser_identity = AttachmentTextExtractionParserRuntimeV1::new(None)
        .extract(PRIVATE_SOURCE_TEXT)
        .expect("current plain parser identity")
        .parser_identity_sha256;
    let stale_parser_identity = [0x77; 32];
    assert_ne!(current_parser_identity, stale_parser_identity);
    replace_attachment_text_parser_identity_v1(
        ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        current_parser_identity,
        stale_parser_identity,
    );
    let stale_revision_read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(
        text_error_v1(stale_revision_read.error),
        AttachmentTextExtractionErrorCodeV1::Unavailable
    );
    assert!(stale_revision_read.text_utf8.is_empty());
    replace_attachment_text_parser_identity_v1(
        ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1,
        &accepted.run_id,
        stale_parser_identity,
        current_parser_identity,
    );
    let current_revision_read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(current_revision_read, read);
    assert_eq!(
        attachment_text_extraction_diagnostics_v1(),
        AttachmentTextExtractionDiagnosticsV1 {
            candidates: 8,
            safety_facts: 8,
            custody_requests: 8,
            pending_custody_outbox: 0,
            custody_results: 8,
            jobs: 8,
            attempts: 8,
            artifacts: 4,
            security_delegation_commands: 8,
            security_delegation_attempts: 8,
            security_delegation_results: 8,
        },
        "each supported parser must execute once through the same event-only custody path"
    );

    supervisor
        .stop("vault")
        .expect("stop Vault for Text Extraction private-content outage");
    let vault_unavailable_read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
        ReadAttachmentTextRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(
        text_error_v1(vault_unavailable_read.error),
        AttachmentTextExtractionErrorCodeV1::Unavailable
    );
    assert!(vault_unavailable_read.text_utf8.is_empty());
    assert_eq!(attachment_text_extraction_diagnostics_v1().artifacts, 4);

    assert_eq!(
        post_attachment_text_proto_status_v1(
            &restarted_router,
            &gateway_runtime,
            None,
            ATTACHMENT_TEXT_EXTRACTION_QUERY_CONNECT_PATH_V1,
            GetAttachmentTextExtractionRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id,
            },
        ),
        StatusCode::UNAUTHORIZED,
        "Text Extraction metadata query must require an authenticated Gateway session"
    );

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Attachment Text Extraction fixture");
    std::fs::remove_dir_all(data).expect("remove short Attachment Text Extraction Kernel fixture");
}

struct ManagedAttachmentTextNegativeContourV1<'a> {
    store: &'a Arc<SqliteControlStore>,
    supervisor: &'a ManagedRuntimeSupervisor,
    data: &'a Path,
    clamav: &'a AttachmentSecurityClamAvFixture,
    blob_source: &'a AttachmentSecurityBlobSourceFixture,
    router: &'a attachment_text_extraction_gateway_fixture::AttachmentTextExtractionGateway,
    gateway_runtime: &'a tokio::runtime::Runtime,
    cookie: &'a str,
}

struct ManagedAttachmentTextFailureScenarioV1 {
    expected_count: i64,
    scenario_id: &'static str,
    blob_id: [u8; 16],
    operation_id: [u8; 16],
    source: Vec<u8>,
    expected_state: AttachmentTextExtractionStateV1,
    expected_error: AttachmentTextExtractionErrorCodeV1,
}

impl ManagedAttachmentTextNegativeContourV1<'_> {
    fn assert_terminal_failure_v1(&self, scenario: ManagedAttachmentTextFailureScenarioV1) {
        let ManagedAttachmentTextFailureScenarioV1 {
            expected_count,
            scenario_id,
            blob_id,
            operation_id,
            source,
            expected_state,
            expected_error,
        } = scenario;
        let blob = self.blob_source.write(
            self.store,
            self.supervisor,
            self.data,
            blob_id,
            source.as_slice(),
        );
        let attachment = prepare_communications_attachment_for_scan(
            self.store,
            scenario_id,
            blob.declared_size,
            blob.receipt_sha256,
        );
        assert_clean_attachment_security_verdict_flow(
            self.store,
            &attachment,
            &blob,
            self.clamav,
            source.as_slice(),
        );
        assert_eq!(
            wait_for_attachment_state(
                self.store,
                self.supervisor,
                attachment.attachment_anchor_id,
            ),
            makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
                as u32
        );
        wait_for_attachment_text_evidence_v1(expected_count);
        let accepted = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
            self.router,
            self.gateway_runtime,
            self.cookie,
            ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
            StartAttachmentTextExtractionRequestV1 {
                protocol_major: 1,
                operation_id: operation_id.to_vec(),
                attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
            },
        );
        assert_eq!(accepted.error, unspecified_error_v1());
        let terminal = wait_for_terminal_attachment_text_v1(
            self.router,
            self.gateway_runtime,
            self.cookie,
            &accepted.run_id,
        );
        assert_eq!(text_state_v1(terminal.state), expected_state);
        assert_eq!(text_error_v1(terminal.error), expected_error);
        assert_eq!(terminal.format, AttachmentTextFormatV1::Unspecified as i32);
        assert_eq!(terminal.extracted_size_bytes, 0);
        assert!(!terminal.extraction_truncated);
        assert_private_attachment_text_absent_v1(&terminal.encode_to_vec(), &source, &blob);

        let read = post_attachment_text_proto_v1::<_, ReadAttachmentTextResponseV1>(
            self.router,
            self.gateway_runtime,
            self.cookie,
            ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
            ReadAttachmentTextRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id.clone(),
            },
        );
        assert_eq!(
            text_error_v1(read.error),
            AttachmentTextExtractionErrorCodeV1::NotFound
        );
        assert!(read.text_utf8.is_empty());
        let event = read_terminal_attachment_text_sse_event_v1(
            self.router,
            self.gateway_runtime,
            self.cookie,
            &accepted.run_id,
        );
        assert_private_attachment_text_absent_v1(&event.encode_to_vec(), &source, &blob);
        assert_eq!(
            attachment_text_extraction_diagnostics_v1().artifacts,
            4,
            "{scenario_id} must not commit a derived artifact"
        );
    }
}

fn wait_for_attachment_text_evidence_v1(expected: i64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = attachment_text_extraction_diagnostics_v1();
        if diagnostics.candidates == expected && diagnostics.safety_facts == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Text Extraction did not persist source evidence: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_pending_attachment_text_custody_v1() {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = attachment_text_extraction_diagnostics_v1();
        if diagnostics.custody_requests == 1 && diagnostics.pending_custody_outbox == 1 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Text Extraction did not retain custody command during NATS outage: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_attachment_text_custody_request_v1(expected_count: i64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = attachment_text_extraction_diagnostics_v1();
        if diagnostics.custody_requests == expected_count && diagnostics.pending_custody_outbox == 0
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Text Extraction did not publish custody request {expected_count}: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_attachment_text_security_delegation_result_v1(expected_count: i64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = attachment_text_extraction_diagnostics_v1();
        if diagnostics.security_delegation_results == expected_count {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Security did not publish Text custody result {expected_count}: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_attachment_text_stale_proof_failure_v1(expected_count: i64, expected_artifacts: i64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = attachment_text_extraction_diagnostics_v1();
        if diagnostics.custody_results == expected_count
            && diagnostics.jobs == expected_count
            && diagnostics.attempts == expected_count
            && diagnostics.artifacts == expected_artifacts
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "stale Text custody proof did not fail closed: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn route_attachment_text_start_as_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    text: &StartedAttachmentTextExtractionRuntimeV1,
    logical_owner_id: &str,
    request_id: u64,
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
) -> ModuleClientResponseV1 {
    let request = attachment_text_module_request_v1(
        logical_owner_id,
        request_id,
        operation_id,
        attachment_anchor_id,
    );
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &text.registration_id,
        &text.runtime_instance_id,
        text.runtime_generation,
        text.grant_epoch,
        ATTACHMENT_TEXT_EXTRACTION_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Attachment Text Extraction owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice())
        .expect("decode Attachment Text Extraction owner-fence response")
}

fn assert_attachment_text_runtime_fences_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    text: &StartedAttachmentTextExtractionRuntimeV1,
    attachment_anchor_id: [u8; 16],
) {
    let request = attachment_text_module_request_v1(
        ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1,
        800,
        [0xaf; 16],
        attachment_anchor_id,
    );
    for (runtime_generation, grant_epoch, label) in [
        (
            text.runtime_generation + 1,
            text.grant_epoch,
            "stale Attachment Text Extraction runtime generation",
        ),
        (
            text.runtime_generation,
            text.grant_epoch + 1,
            "stale Attachment Text Extraction grant epoch",
        ),
    ] {
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            &text.registration_id,
            &text.runtime_instance_id,
            runtime_generation,
            grant_epoch,
            ATTACHMENT_TEXT_EXTRACTION_CAPABILITY_ID_V1,
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

fn attachment_text_module_request_v1(
    logical_owner_id: &str,
    request_id: u64,
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
) -> Vec<u8> {
    ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
            name: ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_TEXT_EXTRACTION_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: StartAttachmentTextExtractionRequestV1 {
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

fn assert_private_attachment_text_absent_v1(
    bytes: &[u8],
    source: &[u8],
    blob: &AttachmentSecurityFixtureBlobV1,
) {
    for private in [
        source,
        blob.reference_id.as_slice(),
        blob.receipt_sha256.as_slice(),
        blob.custody_transfer_source_proof.as_slice(),
    ] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "private attachment text or Blob authority crossed a metadata client boundary"
        );
    }
}

fn text_state_v1(value: i32) -> AttachmentTextExtractionStateV1 {
    AttachmentTextExtractionStateV1::try_from(value)
        .expect("known Attachment Text Extraction state")
}

fn text_format_v1(value: i32) -> AttachmentTextFormatV1 {
    AttachmentTextFormatV1::try_from(value).expect("known Attachment Text Extraction format")
}

fn text_error_v1(value: i32) -> AttachmentTextExtractionErrorCodeV1 {
    AttachmentTextExtractionErrorCodeV1::try_from(value)
        .expect("known Attachment Text Extraction error")
}

fn unspecified_error_v1() -> i32 {
    AttachmentTextExtractionErrorCodeV1::Unspecified as i32
}
