//! Live managed AI engine routing, owner fencing, terminal replay and request-conflict conformance.

use std::net::TcpListener;

use super::*;

use makosh_ai_contracts::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, communication_reply_inference_contract_reference_v1,
    encode_reply_source_content_v1, seal_reply_inference_request_v1,
    wire::{
        AiContextReceiptV1, AiEgressPolicyV1, AiInferenceCompletenessV1,
        AiInferenceTerminalStatusV1, AiPrivateSourceReceiptV1, AiReplyLanguageV1,
        AiReplySourceContentV1, AiReplySubjectPolicyV1, AiReplyToneV1, AiUseCaseV1,
        CommunicationReplySuggestionInferenceRequestV1,
        CommunicationReplySuggestionInferenceResultV1,
    },
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, AI inference and Ollama AI binaries"]
fn managed_ai_inference_routes_to_ollama_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let port_reservation =
        TcpListener::bind(("127.0.0.1", 0)).expect("reserve unavailable Ollama port");
    let ollama_port = port_reservation
        .local_addr()
        .expect("read unavailable Ollama port")
        .port();
    drop(port_reservation);

    let root = unique_target_root("makosh-managed-ai-inference-negative");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ai_inference_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    crate::platform::blob::binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            AI_INFERENCE_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim AI inference logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let source = AiInferenceBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
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
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);
    assert_eq!(ai.runtime_generation, 1);
    assert!(
        supervisor
            .is_active(&ai.registration_id)
            .expect("read AI inference process state")
    );

    let source_content = encode_reply_source_content_v1(&AiReplySourceContentV1 {
        sender_utf8: b"Alice Example <alice@example.test>".to_vec(),
        subject_utf8: b"Quarterly update".to_vec(),
        body_utf8: b"Private source body for a bounded local reply".to_vec(),
    })
    .expect("typed AI source content");
    let blob = source.write(&store, &supervisor, &data, [0x51; 16], &source_content);
    let request = inference_request_v1(&blob);
    let first = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert!(first.error_code.is_empty());
    let first_result =
        CommunicationReplySuggestionInferenceResultV1::decode(first.response_payload.as_slice())
            .expect("typed AI provider-unavailable result");
    assert_eq!(first_result.run_id, request.run_id);
    assert_eq!(
        first_result.terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable as i32
    );
    assert!(first_result.subject_utf8.is_empty());
    assert!(first_result.body_utf8.is_empty());

    supervisor
        .stop(&ollama.registration_id)
        .expect("stop Ollama dependency before AI replay");
    let previous_generation = ai.runtime_generation;
    let ai = restart_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), ai);
    assert_eq!(ai.runtime_generation, previous_generation + 1);

    let replayed = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert_eq!(replayed, first);

    let mut conflicting = request.clone();
    conflicting.maximum_output_tokens += 1;
    let conflicting =
        seal_reply_inference_request_v1(conflicting).expect("seal conflicting request");
    let rejected = deliver_inference_request_v1(&supervisor, &ai.registration_id, &conflicting);
    assert_eq!(rejected.request_id, request.run_id);
    assert_eq!(rejected.error_code, "REJECTED");
    assert!(rejected.response_payload.is_empty());

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove AI inference fixture");
    std::fs::remove_dir_all(data).expect("remove short AI inference kernel data fixture");
}

#[test]
#[ignore = "requires disposable Docker plus a real loopback Ollama service with makosh-conformance:latest"]
fn managed_ai_inference_completes_real_provider_generation() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_port = required("MAKOSH_OLLAMA_LIVE_PORT")
        .parse::<u16>()
        .expect("valid live Ollama port");
    let root = unique_target_root("makosh-managed-ai-inference-live");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ai_inference_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    crate::platform::blob::binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            AI_INFERENCE_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim live AI inference logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let source = AiInferenceBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
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
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);

    let source_content = encode_reply_source_content_v1(&AiReplySourceContentV1 {
        sender_utf8: b"Alice Example <alice@example.test>".to_vec(),
        subject_utf8: b"Quarterly update".to_vec(),
        body_utf8: b"Private source body for a bounded local reply".to_vec(),
    })
    .expect("typed live AI source content");
    let blob = source.write(&store, &supervisor, &data, [0x71; 16], &source_content);
    let request = inference_request_v1(&blob);

    let wrong_owner = deliver_inference_request_for_owner_v1(
        &supervisor,
        &ai.registration_id,
        "owner-2",
        &request,
    );
    assert_eq!(wrong_owner.request_id, request.run_id);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let first = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert!(first.error_code.is_empty());
    let result =
        CommunicationReplySuggestionInferenceResultV1::decode(first.response_payload.as_slice())
            .expect("typed successful AI inference result");
    assert_eq!(result.run_id, request.run_id);
    assert_eq!(
        result.terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32
    );
    assert_eq!(
        result.completeness,
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32
    );
    assert_eq!(
        result.resolved_language,
        AiReplyLanguageV1::AiReplyLanguageEnglish as i32
    );
    assert!(!result.body_utf8.is_empty());
    let receipt = result
        .inference_receipt
        .as_ref()
        .expect("successful AI inference receipt");
    assert_eq!(receipt.model_revision_sha256.len(), 32);
    assert_eq!(receipt.prompt_policy_sha256.len(), 32);
    assert_eq!(receipt.provider_settings_revision, 1);

    supervisor
        .stop(&ollama.registration_id)
        .expect("stop Ollama dependency before successful replay");
    let previous_generation = ai.runtime_generation;
    let ai = restart_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), ai);
    assert_eq!(ai.runtime_generation, previous_generation + 1);
    let replayed = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert_eq!(replayed, first);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove live AI inference fixture");
    std::fs::remove_dir_all(data).expect("remove short live AI inference kernel data fixture");
}

fn inference_request_v1(
    blob: &AiInferenceFixtureBlobV1,
) -> CommunicationReplySuggestionInferenceRequestV1 {
    seal_reply_inference_request_v1(CommunicationReplySuggestionInferenceRequestV1 {
        run_id: vec![0x61; 16],
        context: Some(AiContextReceiptV1 {
            context_id: vec![0x62; 16],
            use_case: AiUseCaseV1::AiUseCaseCommunicationReplySuggestion as i32,
            source_evidence_id: vec![0x63; 16],
            source_evidence_revision: 1,
            contract_major: AI_CONTRACT_MAJOR_V1,
            contract_revision: AI_CONTRACT_REVISION_V1,
            contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            request_digest: Vec::new(),
        }),
        source: Some(AiPrivateSourceReceiptV1 {
            reference_id: blob.reference_id.to_vec(),
            declared_bytes: blob.declared_size,
            sha256: blob.receipt_sha256.to_vec(),
            custody_transfer_source_proof: blob.custody_transfer_source_proof.clone(),
        }),
        tone: AiReplyToneV1::AiReplyToneNeutral as i32,
        language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
        subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
        maximum_output_bytes: 4_096,
        maximum_output_tokens: 512,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        logical_owner_id: AI_INFERENCE_LOGICAL_OWNER_ID_V1.to_owned(),
    })
    .expect("seal AI inference request")
}

fn deliver_inference_request_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    deliver_inference_request_for_owner_v1(
        supervisor,
        registration_id,
        AI_INFERENCE_LOGICAL_OWNER_ID_V1,
        request,
    )
}

fn deliver_inference_request_for_owner_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
        request_id: request.run_id.clone(),
        logical_owner_id: logical_owner_id.to_owned(),
        contract: Some(communication_reply_inference_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        response_blob_target_owner_id: String::new(),
        response_blob_target_module_id: String::new(),
        response_blob_target_capability_id: String::new(),
    };
    let response = supervisor
        .relay(
            registration_id,
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::DeliverModuleRequest(delivery)),
            }
            .encode_to_vec(),
        )
        .expect("deliver managed AI inference request");
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode managed AI inference response");
    assert!(response.error_code.is_empty());
    match response.result {
        Some(ControlResult::ModuleRequestDelivery(response)) => response,
        _ => panic!("managed AI inference response is missing"),
    }
}
