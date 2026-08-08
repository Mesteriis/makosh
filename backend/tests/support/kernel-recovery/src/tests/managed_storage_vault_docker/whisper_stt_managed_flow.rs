//! Live managed Whisper transcription, Blob custody, restart and owner-fence conformance.

use super::*;

use makosh_runtime_protocol::v1::{
    BlobCustodySourceProofV1, ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_speech_to_text_api::{
    SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1, seal_speech_to_text_request_v1,
    speech_to_text_contract_reference_v1, validate_speech_to_text_result_v1,
    wire::{
        SpeechAudioFormatV1, SpeechAudioSourceReceiptV1, SpeechLanguageV1, SpeechToTextRequestV1,
        SpeechToTextResultV1, SpeechToTextTerminalStatusV1,
    },
};
use makosh_speech_to_text_runtime::SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1;
use makosh_speech_transcript_artifact::{
    validate_speech_transcript_document_v1, wire::SpeechTranscriptDocumentV1,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob and pinned Whisper binaries"]
fn managed_speech_to_text_routes_whisper_private_blob_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-whisper-stt");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_whisper_stt_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    blob_binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            WHISPER_STT_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Whisper STT logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_engine = admit_speech_to_text_runtime_v1(&store);
    let admitted_provider = admit_whisper_stt_runtime_v1(&store);
    let source = WhisperSttBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_speech_to_text_module_request_router_v1(&supervisor, &store);
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
    let admitted_provider = prepare_whisper_stt_runtime_v1(&supervisor, &store, admitted_provider);
    let admitted_engine = prepare_speech_to_text_runtime_v1(&supervisor, &store, admitted_engine);
    let provider = start_whisper_stt_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_provider,
    );
    let engine = start_speech_to_text_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_engine,
    );
    assert_eq!(provider.runtime_generation, 1);
    assert!(provider.grant_epoch > 0);
    assert_eq!(engine.runtime_generation, 1);
    let audio = std::fs::read(required("MAKOSH_WHISPER_STT_TEST_WAV"))
        .expect("read bounded Whisper STT WAV fixture");
    let source_blob = source.write_audio(&store, &supervisor, &data, [0x91; 16], &audio);
    let source_proof =
        BlobCustodySourceProofV1::decode(source_blob.custody_transfer_source_proof.as_slice())
            .expect("decode source custody proof");
    assert_eq!(source_proof.reference_id, source_blob.reference_id);
    assert_eq!(source_proof.target_owner_id, SPEECH_TO_TEXT_OWNER_V1);
    assert_eq!(source_proof.target_module_id, SPEECH_TO_TEXT_MODULE_ID_V1);
    assert_eq!(
        source_proof.target_capability_id,
        SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1
    );
    let request = speech_request_v1(&source_blob);
    let first = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &request,
    );
    assert!(
        first.error_code.is_empty(),
        "managed Speech-to-Text delivery failed with {}",
        first.error_code
    );
    let first_result = SpeechToTextResultV1::decode(first.response_payload.as_slice())
        .expect("typed Whisper STT result");
    validate_speech_to_text_result_v1(&request, &first_result).expect("valid Whisper STT result");
    assert_eq!(
        first_result.terminal_status,
        SpeechToTextTerminalStatusV1::Ready as i32,
        "managed Speech-to-Text rejected the request with code {}",
        first_result.reject_code,
    );
    let transcript = first_result
        .transcript
        .as_ref()
        .expect("Whisper transcript receipt");
    let transcript_blob = WhisperSttFixtureBlobV1 {
        reference_id: transcript
            .reference_id
            .as_slice()
            .try_into()
            .expect("transcript reference id"),
        receipt_sha256: transcript
            .sha256
            .as_slice()
            .try_into()
            .expect("transcript digest"),
        custody_transfer_source_proof: transcript.custody_transfer_source_proof.clone(),
        declared_size: transcript.declared_bytes,
    };
    let document_bytes = source.read_transcript(&store, &supervisor, &data, &transcript_blob);
    assert_eq!(
        Sha256::digest(&document_bytes).as_slice(),
        transcript.sha256
    );
    let document = SpeechTranscriptDocumentV1::decode(document_bytes.as_slice())
        .expect("decode private transcript document");
    validate_speech_transcript_document_v1(
        &document,
        request.duration_millis,
        request.maximum_segments,
        request.maximum_transcript_bytes,
    )
    .expect("validate private transcript document");
    let text = document
        .segments
        .iter()
        .flat_map(|segment| segment.content_utf8.iter().copied())
        .collect::<Vec<_>>();
    let text = std::str::from_utf8(&text).expect("private transcript utf8");
    assert!(text.to_ascii_lowercase().contains("makosh"));

    let previous_provider_generation = provider.runtime_generation;
    let previous_engine_generation = engine.runtime_generation;
    let provider =
        restart_whisper_stt_runtime_v1(&supervisor, &store, &data, &root.join("runtime"), provider);
    let engine =
        restart_speech_to_text_runtime_v1(&supervisor, &store, &root.join("runtime"), engine);
    assert_eq!(
        provider.runtime_generation,
        previous_provider_generation + 1
    );
    assert_eq!(engine.runtime_generation, previous_engine_generation + 1);
    let replayed = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &request,
    );
    assert!(replayed.error_code.is_empty());
    let replayed_result = SpeechToTextResultV1::decode(replayed.response_payload.as_slice())
        .expect("typed replayed Whisper STT result");
    assert_eq!(replayed_result.request_id, first_result.request_id);
    assert_eq!(replayed_result.request_digest, first_result.request_digest);
    let replayed_transcript = replayed_result
        .transcript
        .as_ref()
        .expect("replayed Whisper transcript receipt");
    assert_eq!(replayed_transcript.reference_id, transcript.reference_id);
    assert_eq!(replayed_transcript.sha256, transcript.sha256);
    assert_eq!(
        replayed_transcript.declared_bytes,
        transcript.declared_bytes
    );
    let replayed_blob = WhisperSttFixtureBlobV1 {
        reference_id: replayed_transcript
            .reference_id
            .as_slice()
            .try_into()
            .expect("replayed transcript reference id"),
        receipt_sha256: replayed_transcript
            .sha256
            .as_slice()
            .try_into()
            .expect("replayed transcript digest"),
        custody_transfer_source_proof: replayed_transcript.custody_transfer_source_proof.clone(),
        declared_size: replayed_transcript.declared_bytes,
    };
    assert_eq!(
        source.read_transcript(&store, &supervisor, &data, &replayed_blob),
        document_bytes
    );

    let wrong_owner = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        "owner-2",
        &request,
    );
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let mut conflicting = request.clone();
    conflicting.maximum_segments += 1;
    let conflicting = seal_speech_to_text_request_v1(conflicting).expect("seal conflict");
    let rejected = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &conflicting,
    );
    assert_eq!(rejected.error_code, "REJECTED");
    assert!(rejected.response_payload.is_empty());

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Whisper STT fixture");
    std::fs::remove_dir_all(data).expect("remove short Whisper STT kernel data fixture");
}

fn speech_request_v1(blob: &WhisperSttFixtureBlobV1) -> SpeechToTextRequestV1 {
    seal_speech_to_text_request_v1(SpeechToTextRequestV1 {
        protocol_major: 0,
        request_id: vec![0x92; 16],
        logical_owner_id: WHISPER_STT_LOGICAL_OWNER_ID_V1.to_owned(),
        source: Some(SpeechAudioSourceReceiptV1 {
            reference_id: blob.reference_id.to_vec(),
            declared_bytes: blob.declared_size,
            sha256: blob.receipt_sha256.to_vec(),
            custody_transfer_source_proof: blob.custody_transfer_source_proof.clone(),
        }),
        audio_format: SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32,
        duration_millis: 10_000,
        requested_language: SpeechLanguageV1::English as i32,
        consent_receipt_id: vec![0x93; 16],
        consent_policy_revision: 1,
        maximum_transcript_bytes: 64 * 1024,
        maximum_segments: 128,
        request_digest: Vec::new(),
    })
    .expect("seal Whisper STT request")
}

fn deliver_speech_to_text_request_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request: &SpeechToTextRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
        request_id: request.request_id.clone(),
        logical_owner_id: logical_owner_id.to_owned(),
        contract: Some(speech_to_text_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        response_blob_target_owner_id: SOURCE_OWNER_ID_V1.to_owned(),
        response_blob_target_module_id: SOURCE_MODULE_ID_V1.to_owned(),
        response_blob_target_capability_id: SOURCE_BLOB_CAPABILITY_ID_V1.to_owned(),
    };
    let response = supervisor
        .relay(
            registration_id,
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::DeliverModuleRequest(delivery)),
            }
            .encode_to_vec(),
        )
        .expect("deliver managed Speech-to-Text request");
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode managed Speech-to-Text response");
    assert!(response.error_code.is_empty());
    match response.result {
        Some(ControlResult::ModuleRequestDelivery(response)) => response,
        _ => panic!("managed Speech-to-Text response is missing"),
    }
}
