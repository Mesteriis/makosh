//! Signed managed lifecycle smoke gate for Attachment Translation.

use super::*;

use std::{
    io::ErrorKind, net::TcpListener, sync::atomic::AtomicUsize, thread::JoinHandle, time::Duration,
};

use super::attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture;
use super::attachment_text_extraction_gateway_fixture::{
    attachment_text_extraction_gateway_v1, post_attachment_text_proto_v1,
    wait_for_ready_attachment_text_v1,
};
use crate::identity::device::signer::DeviceSigner;
use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
    wire::{StartAttachmentTextExtractionRequestV1, StartAttachmentTextExtractionResponseV1},
};
use makosh_attachment_translation_api::{
    ATTACHMENT_TRANSLATION_CAPABILITY_ID_V1, ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1,
    ATTACHMENT_TRANSLATION_COMMAND_CONTRACT_NAME_V1, ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1,
    ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1, ATTACHMENT_TRANSLATION_CONTROL_SCHEMA_SHA256,
    ATTACHMENT_TRANSLATION_MODULE_ID_V1, ATTACHMENT_TRANSLATION_OWNER_V1,
    ATTACHMENT_TRANSLATION_QUERY_CONNECT_PATH_V1, ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1,
    wire::{
        AttachmentTranslationErrorCodeV1, AttachmentTranslationLanguageV1,
        AttachmentTranslationStateV1, AttachmentTranslationStatusChangedV1,
        GetAttachmentTranslationRequestV1, IssueAttachmentTranslationReadRequestV1,
        IssueAttachmentTranslationReadResponseV1, StartAttachmentTranslationRequestV1,
        StartAttachmentTranslationResponseV1,
    },
};
use makosh_attachment_translation_runtime::ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1;
use makosh_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use sha2::{Digest, Sha256};

const PRIVATE_ATTACHMENT_SOURCE_V1: &[u8] =
    b"Private attachment source for translation conformance.";

#[test]
#[ignore = "requires disposable Docker plus managed Vault, Storage, Blob, NATS, Text Extraction, AI inference, Ollama AI and Attachment Translation binaries"]
fn managed_attachment_translation_reaches_source_ai_and_gateway_sse() {
    run_attachment_translation_managed_contour_v1(AttachmentTranslationProviderV1::Unavailable(
        AttachmentTranslationUnavailableOllamaV1::start(),
    ));
}

#[test]
#[ignore = "requires disposable Docker plus a real loopback Ollama service with makosh-conformance:latest"]
fn managed_attachment_translation_completes_and_reads_real_provider_result() {
    let port = required("MAKOSH_OLLAMA_LIVE_PORT")
        .parse::<u16>()
        .expect("valid live Ollama port");
    run_attachment_translation_managed_contour_v1(AttachmentTranslationProviderV1::Live(port));
}

fn run_attachment_translation_managed_contour_v1(provider: AttachmentTranslationProviderV1) {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let ollama_port = provider.port();
    let root = unique_target_root("makosh-managed-attachment-translation");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_translation_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Attachment Translation logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1,
    );
    super::super::browser_gateway_session::admit_secondary_browser_test_device(
        &store,
        ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1,
    );

    let admitted_extraction = admit_attachment_text_extraction_runtime_v1(&store);
    let admitted_translation = admit_attachment_translation_runtime_v1(&store);
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
    configure_attachment_translation_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted_extraction =
        prepare_attachment_text_extraction_runtime_v1(&supervisor, &store, admitted_extraction);
    let admitted_translation =
        prepare_attachment_translation_runtime_v1(&supervisor, &store, admitted_translation);
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));

    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);
    let security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    let extraction = start_attachment_text_extraction_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_extraction,
    );
    let translation = start_attachment_translation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_translation,
    );
    for (registration_id, owner) in [
        (ollama.registration_id.as_str(), "Ollama integration"),
        (ai.registration_id.as_str(), "AI engine"),
        (
            extraction.registration_id.as_str(),
            "Text Extraction workflow",
        ),
        (
            translation.registration_id.as_str(),
            "Attachment Translation workflow",
        ),
        (
            security.registration_id.as_str(),
            "Attachment Security engine",
        ),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .unwrap_or_else(|error| panic!("observe {owner}: {error}")),
            "{owner} must remain an independently active managed process"
        );
    }
    assert_eq!(translation.runtime_generation, 1);
    assert!(translation.grant_epoch > 0);
    assert!(!translation.runtime_instance_id.is_empty());

    let blob = blob_source.write(
        &store,
        &supervisor,
        &data,
        [0xe1; 16],
        PRIVATE_ATTACHMENT_SOURCE_V1,
    );
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "attachment-translation",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(
        &store,
        &attachment,
        &blob,
        &clamav,
        PRIVATE_ATTACHMENT_SOURCE_V1,
    );
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let text_router =
        attachment_text_extraction_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &text_router,
        &gateway_runtime,
    );
    let extracted = post_attachment_text_proto_v1::<_, StartAttachmentTextExtractionResponseV1>(
        &text_router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
        StartAttachmentTextExtractionRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xe2; 16],
            attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
        },
    );
    let extracted = wait_for_ready_attachment_text_v1(
        &text_router,
        &gateway_runtime,
        &cookie,
        &extracted.run_id,
    );
    assert_eq!(
        extracted.extracted_size_bytes,
        PRIVATE_ATTACHMENT_SOURCE_V1.len() as u64
    );

    let translation_router =
        attachment_translation_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let translation_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &translation_router,
            &gateway_runtime,
            2,
        );
    let request = StartAttachmentTranslationRequestV1 {
        protocol_major: 1,
        operation_id: vec![0xe3; 16],
        source_extraction_run_id: extracted.run_id.clone(),
        expected_source_revision: extracted.state_revision,
        target_language: AttachmentTranslationLanguageV1::AttachmentTranslationLanguageRussian
            as i32,
    };
    assert_attachment_translation_runtime_fences_v1(
        &store,
        &supervisor,
        &translation,
        request.clone(),
    );
    let unsupported_language = route_attachment_translation_as_v1(
        &store,
        &supervisor,
        &translation,
        ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1,
        900,
        StartAttachmentTranslationRequestV1 {
            operation_id: vec![0xe0; 16],
            target_language:
                AttachmentTranslationLanguageV1::AttachmentTranslationLanguageUnspecified as i32,
            ..request.clone()
        },
    );
    assert_eq!(unsupported_language.request_id, 900);
    assert_eq!(unsupported_language.error_code, "REJECTED");
    assert!(unsupported_language.response_payload.is_empty());
    let wrong_owner = route_attachment_translation_as_v1(
        &store,
        &supervisor,
        &translation,
        "owner-2",
        901,
        request.clone(),
    );
    assert_eq!(wrong_owner.request_id, 901);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());
    provider.assert_attempts(0);
    assert_eq!(
        post_attachment_translation_proto_status_v1(
            &translation_router,
            &gateway_runtime,
            None,
            ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1,
            request.clone(),
        ),
        hyper::StatusCode::UNAUTHORIZED
    );
    let first_sse = open_attachment_translation_sse_v1(
        &translation_router,
        &gateway_runtime,
        &translation_cookie,
    );
    let accepted = post_attachment_translation_proto_v1::<_, StartAttachmentTranslationResponseV1>(
        &translation_router,
        &gateway_runtime,
        &translation_cookie,
        ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.run_id.len(), 16);
    assert_eq!(
        AttachmentTranslationErrorCodeV1::try_from(accepted.error)
            .expect("known Attachment Translation start error"),
        AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeUnspecified
    );
    let first_event = read_terminal_attachment_translation_sse_response_v1(
        &gateway_runtime,
        first_sse,
        &accepted.run_id,
    );
    let first_payload =
        AttachmentTranslationStatusChangedV1::decode(first_event.payload.as_slice())
            .expect("Attachment Translation SSE payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    let terminal = get_attachment_translation_v1(
        &translation_router,
        &gateway_runtime,
        &translation_cookie,
        &accepted.run_id,
    );
    let translated_body = if provider.is_live() {
        assert_eq!(
            AttachmentTranslationStateV1::try_from(terminal.state)
                .expect("known ready Attachment Translation state"),
            AttachmentTranslationStateV1::AttachmentTranslationStateReady
        );
        assert_eq!(
            AttachmentTranslationErrorCodeV1::try_from(terminal.error)
                .expect("known ready Attachment Translation error"),
            AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeUnspecified
        );
        Some(read_ready_attachment_translation_v1(
            &translation_router,
            &gateway_runtime,
            &translation_cookie,
            &accepted.run_id,
            &terminal,
        ))
    } else {
        assert_eq!(
            AttachmentTranslationStateV1::try_from(terminal.state)
                .expect("known terminal Attachment Translation state"),
            AttachmentTranslationStateV1::AttachmentTranslationStateRejected
        );
        assert_eq!(
            AttachmentTranslationErrorCodeV1::try_from(terminal.error)
                .expect("known terminal Attachment Translation error"),
            AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeInferenceRejected
        );
        assert!(terminal.artifact.is_none());
        provider.assert_attempted();
        None
    };
    assert!(
        !first_event
            .encode_to_vec()
            .windows(PRIVATE_ATTACHMENT_SOURCE_V1.len())
            .any(|window| window == PRIVATE_ATTACHMENT_SOURCE_V1),
        "private attachment source must stay out of SSE"
    );
    let duplicate = post_attachment_translation_proto_v1::<_, StartAttachmentTranslationResponseV1>(
        &translation_router,
        &gateway_runtime,
        &translation_cookie,
        ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1,
        request,
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    let mut conflicting_request = StartAttachmentTranslationRequestV1 {
        protocol_major: 1,
        operation_id: vec![0xe3; 16],
        source_extraction_run_id: extracted.run_id.clone(),
        expected_source_revision: extracted.state_revision,
        target_language: AttachmentTranslationLanguageV1::AttachmentTranslationLanguageEnglish
            as i32,
    };
    let conflicting = post_attachment_translation_proto_v1::<_, StartAttachmentTranslationResponseV1>(
        &translation_router,
        &gateway_runtime,
        &translation_cookie,
        ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1,
        conflicting_request.clone(),
    );
    assert_eq!(
        AttachmentTranslationErrorCodeV1::try_from(conflicting.error)
            .expect("known conflicting Attachment Translation error"),
        AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeInvalidRequest
    );
    let provider_attempts = provider.attempts();

    assert!(
        realtime
            .revoke_owner(ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1)
            .expect("clear Attachment Translation Gateway replay cache")
    );
    let previous_instance = translation.runtime_instance_id.clone();
    let replay_router =
        attachment_translation_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let replay_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &replay_router,
            &gateway_runtime,
            3,
        );
    let replay_sse =
        open_attachment_translation_sse_v1(&replay_router, &gateway_runtime, &replay_cookie);
    let translation = restart_attachment_translation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        translation,
    );
    assert_eq!(translation.runtime_generation, 2);
    assert_ne!(translation.runtime_instance_id, previous_instance);
    assert_eq!(
        get_attachment_translation_v1(
            &replay_router,
            &gateway_runtime,
            &replay_cookie,
            &accepted.run_id,
        ),
        terminal
    );
    if translated_body.is_some() {
        assert_attachment_translation_read_fenced_after_restart_v1(
            &replay_router,
            &gateway_runtime,
            &replay_cookie,
            &accepted.run_id,
        );
    }
    let replay_event = read_terminal_attachment_translation_sse_response_v1(
        &gateway_runtime,
        replay_sse,
        &accepted.run_id,
    );
    assert_eq!(replay_event.cursor, first_event.cursor);
    assert_eq!(replay_event.payload, first_event.payload);
    conflicting_request.operation_id = vec![0xe4; 16];
    conflicting_request.expected_source_revision = extracted.state_revision - 1;
    let stale_sse =
        open_attachment_translation_sse_v1(&replay_router, &gateway_runtime, &replay_cookie);
    let stale = post_attachment_translation_proto_v1::<_, StartAttachmentTranslationResponseV1>(
        &replay_router,
        &gateway_runtime,
        &replay_cookie,
        ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1,
        conflicting_request,
    );
    let stale_event = read_terminal_attachment_translation_sse_response_v1(
        &gateway_runtime,
        stale_sse,
        &stale.run_id,
    );
    let stale_payload =
        AttachmentTranslationStatusChangedV1::decode(stale_event.payload.as_slice())
            .expect("stale Attachment Translation SSE payload");
    assert_eq!(stale_payload.run_id, stale.run_id);
    let stale = get_attachment_translation_v1(
        &replay_router,
        &gateway_runtime,
        &replay_cookie,
        &stale.run_id,
    );
    assert_eq!(
        AttachmentTranslationErrorCodeV1::try_from(stale.error)
            .expect("known stale Attachment Translation error"),
        AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeSourceRejected
    );
    provider.assert_attempts_option(provider_attempts);
    for (registration_id, owner) in [
        (ollama.registration_id.as_str(), "Ollama integration"),
        (ai.registration_id.as_str(), "AI engine"),
        (
            extraction.registration_id.as_str(),
            "Text Extraction workflow",
        ),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .unwrap_or_else(|error| panic!("observe {owner} after restart: {error}")),
            "Attachment Translation restart must not restart {owner}"
        );
    }

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let revoked = transition_registration(
        &owner_runtime_dir,
        &owner_signer,
        &translation.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > translation.grant_epoch);
    assert_eq!(
        store
            .module_registration(&translation.registration_id)
            .expect("read revoked Attachment Translation registration")
            .expect("revoked Attachment Translation registration")
            .state(),
        ModuleRegistrationState::Revoked
    );
    assert_eq!(
        store
            .platform_storage_binding(
                &translation.registration_id,
                ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
            )
            .expect("read revoked Attachment Translation Storage binding")
            .expect("revoked Attachment Translation Storage binding")
            .state(),
        PlatformStorageBindingStateV1::Revoking
    );
    assert!(
        !supervisor
            .stop_if_active(&translation.registration_id)
            .expect("observe stopped Attachment Translation workflow")
    );
    assert_eq!(
        post_attachment_translation_proto_status_v1(
            &replay_router,
            &gateway_runtime,
            Some(&replay_cookie),
            ATTACHMENT_TRANSLATION_QUERY_CONNECT_PATH_V1,
            GetAttachmentTranslationRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id.clone(),
            },
        ),
        hyper::StatusCode::NOT_FOUND
    );

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    owner_control
        .join()
        .expect("join Attachment Translation owner control server")
        .expect("Attachment Translation owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Attachment Translation fixture");
    std::fs::remove_dir_all(data).expect("remove short Attachment Translation Kernel fixture");
}

fn route_attachment_translation_as_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    translation: &StartedAttachmentTranslationRuntimeV1,
    logical_owner_id: &str,
    request_id: u64,
    request: StartAttachmentTranslationRequestV1,
) -> ModuleClientResponseV1 {
    let payload = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: ATTACHMENT_TRANSLATION_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_TRANSLATION_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: ATTACHMENT_TRANSLATION_OWNER_V1.to_owned(),
            name: ATTACHMENT_TRANSLATION_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_TRANSLATION_CONTROL_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: request.encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &translation.registration_id,
        &translation.runtime_instance_id,
        translation.runtime_generation,
        translation.grant_epoch,
        ATTACHMENT_TRANSLATION_CAPABILITY_ID_V1,
        &payload,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Attachment Translation owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice())
        .expect("decode Attachment Translation owner-fence response")
}

fn assert_attachment_translation_runtime_fences_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    translation: &StartedAttachmentTranslationRuntimeV1,
    request: StartAttachmentTranslationRequestV1,
) {
    let payload = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: ATTACHMENT_TRANSLATION_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_TRANSLATION_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: ATTACHMENT_TRANSLATION_OWNER_V1.to_owned(),
            name: ATTACHMENT_TRANSLATION_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_TRANSLATION_CONTROL_SCHEMA_SHA256.to_vec(),
        }),
        request_id: 899,
        request_payload: request.encode_to_vec(),
        logical_owner_id: ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    for (runtime_generation, grant_epoch, label) in [
        (
            translation.runtime_generation + 1,
            translation.grant_epoch,
            "stale Attachment Translation runtime generation",
        ),
        (
            translation.runtime_generation,
            translation.grant_epoch + 1,
            "stale Attachment Translation grant epoch",
        ),
    ] {
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            &translation.registration_id,
            &translation.runtime_instance_id,
            runtime_generation,
            grant_epoch,
            ATTACHMENT_TRANSLATION_CAPABILITY_ID_V1,
            &payload,
        );
        assert_eq!(
            crate::modules::capability::router::route_managed_client_request(
                store,
                &supervisor.relay_port(),
                &route,
            )
            .expect_err(label),
            "managed runtime fence is stale",
        );
    }
}

fn read_ready_attachment_translation_v1(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
    terminal: &makosh_attachment_translation_api::wire::GetAttachmentTranslationResponseV1,
) -> Vec<u8> {
    let artifact = terminal
        .artifact
        .as_ref()
        .expect("ready Attachment Translation artifact");
    let ticket = post_attachment_translation_proto_v1::<_, IssueAttachmentTranslationReadResponseV1>(
        router,
        runtime,
        cookie,
        ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1,
        IssueAttachmentTranslationReadRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    );
    assert_eq!(ticket.run_id, run_id);
    assert_eq!(
        AttachmentTranslationErrorCodeV1::try_from(ticket.error)
            .expect("known Attachment Translation ticket error"),
        AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeUnspecified
    );
    assert_eq!(ticket.opaque_read_ticket.len(), 32);
    assert_eq!(ticket.translated_size_bytes, artifact.translated_size_bytes);
    let (unauthorized, unauthorized_body) = read_attachment_translation_blob_v1(
        router,
        runtime,
        None,
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(unauthorized, hyper::StatusCode::UNAUTHORIZED);
    assert!(unauthorized_body.is_empty());
    let secondary_cookie =
        super::super::browser_gateway_session::authenticate_secondary_gateway_router(
            router, runtime,
        );
    let (wrong_actor, wrong_actor_body) = read_attachment_translation_blob_v1(
        router,
        runtime,
        Some(&secondary_cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(wrong_actor, hyper::StatusCode::NOT_FOUND);
    assert!(wrong_actor_body.is_empty());
    let (status, body) = read_attachment_translation_blob_v1(
        router,
        runtime,
        Some(cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(status, hyper::StatusCode::OK);
    assert!(!body.is_empty());
    assert_eq!(body.len() as u64, artifact.translated_size_bytes);
    assert_eq!(Sha256::digest(&body).as_slice(), artifact.translated_sha256);
    assert!(
        !body
            .windows(PRIVATE_ATTACHMENT_SOURCE_V1.len())
            .any(|window| window == PRIVATE_ATTACHMENT_SOURCE_V1),
        "live translated result must not be an identity copy"
    );
    let (replayed, replayed_body) = read_attachment_translation_blob_v1(
        router,
        runtime,
        Some(cookie),
        ticket.opaque_read_ticket,
    );
    assert_eq!(replayed, hyper::StatusCode::NOT_FOUND);
    assert!(replayed_body.is_empty());
    body
}

fn assert_attachment_translation_read_fenced_after_restart_v1(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) {
    let ticket = post_attachment_translation_proto_v1::<_, IssueAttachmentTranslationReadResponseV1>(
        router,
        runtime,
        cookie,
        ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1,
        IssueAttachmentTranslationReadRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    );
    assert_eq!(ticket.run_id, run_id);
    assert!(ticket.opaque_read_ticket.is_empty());
    assert_eq!(
        AttachmentTranslationErrorCodeV1::try_from(ticket.error)
            .expect("known post-restart Attachment Translation ticket error"),
        AttachmentTranslationErrorCodeV1::AttachmentTranslationErrorCodeUnavailable,
    );
}

enum AttachmentTranslationProviderV1 {
    Unavailable(AttachmentTranslationUnavailableOllamaV1),
    Live(u16),
}

impl AttachmentTranslationProviderV1 {
    fn port(&self) -> u16 {
        match self {
            Self::Unavailable(probe) => probe.port(),
            Self::Live(port) => *port,
        }
    }

    fn is_live(&self) -> bool {
        matches!(self, Self::Live(_))
    }

    fn attempts(&self) -> Option<usize> {
        match self {
            Self::Unavailable(probe) => Some(probe.attempts()),
            Self::Live(_) => None,
        }
    }

    fn assert_attempts(&self, expected: usize) {
        if let Some(actual) = self.attempts() {
            assert_eq!(actual, expected);
        }
    }

    fn assert_attempted(&self) {
        if let Some(actual) = self.attempts() {
            assert!(actual > 0);
        }
    }

    fn assert_attempts_option(&self, expected: Option<usize>) {
        if let Some(expected) = expected {
            assert_eq!(self.attempts(), Some(expected));
        }
    }
}

struct AttachmentTranslationUnavailableOllamaV1 {
    port: u16,
    attempts: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl AttachmentTranslationUnavailableOllamaV1 {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind unavailable Ollama");
        listener
            .set_nonblocking(true)
            .expect("nonblocking Ollama probe");
        let port = listener.local_addr().expect("Ollama probe address").port();
        let attempts = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_attempts = Arc::clone(&attempts);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = std::thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((_connection, _)) => {
                        worker_attempts.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => return,
                }
            }
        });
        Self {
            port,
            attempts,
            shutdown,
            worker: Some(worker),
        }
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }
}

impl Drop for AttachmentTranslationUnavailableOllamaV1 {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().expect("join unavailable Ollama probe");
        }
    }
}
