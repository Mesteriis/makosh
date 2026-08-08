//! Kernel-issued private Blob source and transcript target for managed Whisper STT.

use super::*;

use makosh_blob_client::BlobDataClient;
use makosh_kernel_control_store::{ModuleBlobOperationV1, ModuleBlobQuotaRequestV1};
use makosh_runtime_protocol::v1::{BlobDataOperationV1, ManagedRuntimeBlobSessionRequestV1};
use makosh_speech_to_text_api::{SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1};
use makosh_speech_to_text_runtime::SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1;

const SOURCE_REGISTRATION_ID_V1: &str = "whisper-stt-fixture-source";
pub(super) const SOURCE_MODULE_ID_V1: &str = "workflow.whisper-stt-fixture-source";
pub(super) const SOURCE_OWNER_ID_V1: &str = "call_transcription";
pub(super) const SOURCE_BLOB_CAPABILITY_ID_V1: &str = "whisper-stt-fixture-source.blob.v1";
const SOURCE_RUNTIME_INSTANCE_ID_V1: &str = "91919191919191919191919191919191";
const SOURCE_CUSTODY_SCOPE_ID_V1: &str = "call_transcription.private_audio_and_transcript.v1";

pub(super) struct WhisperSttBlobSourceFixtureV1 {
    grant_epoch: u64,
}

pub(super) struct WhisperSttFixtureBlobV1 {
    pub(super) reference_id: [u8; 16],
    pub(super) receipt_sha256: [u8; 32],
    pub(super) custody_transfer_source_proof: Vec<u8>,
    pub(super) declared_size: u64,
}

struct WhisperSttBlobTargetV1<'a> {
    owner_id: &'a str,
    module_id: &'a str,
    capability_id: &'a str,
}

impl WhisperSttBlobSourceFixtureV1 {
    pub(super) fn admit(store: &SqliteControlStore) -> Self {
        let registration = ModuleRegistration::new(
            SOURCE_REGISTRATION_ID_V1,
            SOURCE_MODULE_ID_V1,
            SOURCE_OWNER_ID_V1,
            Sha256::digest(SOURCE_REGISTRATION_ID_V1.as_bytes()).into(),
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = [SOURCE_BLOB_CAPABILITY_ID_V1.to_owned()];
        let blob = ModuleBlobQuotaRequestV1::new(
            SOURCE_REGISTRATION_ID_V1,
            SOURCE_BLOB_CAPABILITY_ID_V1,
            SOURCE_OWNER_ID_V1,
            32 * 1024 * 1024,
            SOURCE_CUSTODY_SCOPE_ID_V1,
            vec![
                ModuleBlobOperationV1::CustodyTransfer,
                ModuleBlobOperationV1::ReadRange,
                ModuleBlobOperationV1::Write,
            ],
        );
        store
            .create_pending_registration_with_requests(
                &registration,
                &capabilities,
                &[],
                &[],
                std::slice::from_ref(&blob),
            )
            .expect("record Whisper STT fixture source");
        let grant_epoch = store
            .approve_module_registration(SOURCE_REGISTRATION_ID_V1, &capabilities)
            .expect("approve Whisper STT fixture Blob capability")
            .grant_epoch();
        store
            .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
                SOURCE_REGISTRATION_ID_V1,
                1,
                "whisper-stt-fixture-source-distribution",
                SOURCE_MODULE_ID_V1,
                Sha256::digest(b"whisper-stt-fixture-source-binary").into(),
                Sha256::digest(SOURCE_REGISTRATION_ID_V1.as_bytes()).into(),
                None,
            ))
            .expect("record Whisper STT fixture source binding");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                SOURCE_REGISTRATION_ID_V1,
                SOURCE_RUNTIME_INSTANCE_ID_V1,
                1,
                1,
                1,
                grant_epoch,
            ))
            .expect("record Whisper STT fixture source launch");
        Self { grant_epoch }
    }

    pub(super) fn write_audio(
        &self,
        store: &Arc<SqliteControlStore>,
        supervisor: &ManagedRuntimeSupervisor,
        kernel_data: &Path,
        reference_id: [u8; 16],
        plaintext: &[u8],
    ) -> WhisperSttFixtureBlobV1 {
        self.write(
            store,
            supervisor,
            kernel_data,
            reference_id,
            plaintext,
            WhisperSttBlobTargetV1 {
                owner_id: SPEECH_TO_TEXT_OWNER_V1,
                module_id: SPEECH_TO_TEXT_MODULE_ID_V1,
                capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
            },
        )
    }

    pub(super) fn read_transcript(
        &self,
        store: &Arc<SqliteControlStore>,
        supervisor: &ManagedRuntimeSupervisor,
        kernel_data: &Path,
        blob: &WhisperSttFixtureBlobV1,
    ) -> Vec<u8> {
        let transfer_channel_binding =
            Sha256::digest([blob.reference_id.as_slice(), b"transfer"].concat()).to_vec();
        let transfer = BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            kernel_data.to_path_buf(),
        )
        .issue_blob_session(
            &self.expectation(),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: Sha256::digest(
                    [blob.reference_id.as_slice(), b"transfer-request"].concat(),
                )[..16]
                    .to_vec(),
                capability_id: SOURCE_BLOB_CAPABILITY_ID_V1.to_owned(),
                operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as u32,
                channel_binding_sha256: Sha256::digest(&transfer_channel_binding).to_vec(),
                reference_id: blob.reference_id.to_vec(),
                declared_size: blob.declared_size,
                backup_class: 1,
                ttl_seconds: 30,
                receipt_sha256: blob.receipt_sha256.to_vec(),
                custody_source_proof: blob.custody_transfer_source_proof.clone(),
                evidence_id: Sha256::digest(
                    [blob.reference_id.as_slice(), b"transcript-evidence"].concat(),
                )[..16]
                    .to_vec(),
                evidence_envelope_sha256: Sha256::digest(
                    [blob.receipt_sha256.as_slice(), b"transcript-envelope"].concat(),
                )
                .to_vec(),
                custody_target_owner_id: String::new(),
                custody_target_module_id: String::new(),
                custody_target_capability_id: String::new(),
            },
        )
        .expect("issue Whisper STT transcript custody transfer");
        let transfer_grant = transfer
            .custody_transfer_grant
            .expect("Whisper STT transcript custody transfer grant");
        let target_reference_id = transfer_grant.target_reference_id.clone();
        BlobDataClient::new(transfer.data_socket_path)
            .expect("open Whisper STT transcript custody transfer client")
            .custody_transfer(transfer_grant, transfer_channel_binding)
            .expect("transfer Whisper STT transcript custody");

        let read_channel_binding =
            Sha256::digest([target_reference_id.as_slice(), b"read"].concat()).to_vec();
        let read = BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            kernel_data.to_path_buf(),
        )
        .issue_blob_session(
            &self.expectation(),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: Sha256::digest(
                    [target_reference_id.as_slice(), b"read-request"].concat(),
                )[..16]
                    .to_vec(),
                capability_id: SOURCE_BLOB_CAPABILITY_ID_V1.to_owned(),
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1 as u32,
                channel_binding_sha256: Sha256::digest(&read_channel_binding).to_vec(),
                reference_id: target_reference_id,
                declared_size: blob.declared_size,
                backup_class: 1,
                ttl_seconds: 30,
                receipt_sha256: blob.receipt_sha256.to_vec(),
                custody_source_proof: Vec::new(),
                evidence_id: Vec::new(),
                evidence_envelope_sha256: Vec::new(),
                custody_target_owner_id: String::new(),
                custody_target_module_id: String::new(),
                custody_target_capability_id: String::new(),
            },
        )
        .expect("issue Whisper STT transcript read");
        BlobDataClient::new(read.data_socket_path)
            .expect("open Whisper STT transcript Blob client")
            .read_range(
                read.grant.expect("Whisper STT transcript read grant"),
                read_channel_binding,
                0,
                blob.declared_size,
            )
            .expect("read Whisper STT transcript Blob")
    }

    fn write(
        &self,
        store: &Arc<SqliteControlStore>,
        supervisor: &ManagedRuntimeSupervisor,
        kernel_data: &Path,
        reference_id: [u8; 16],
        plaintext: &[u8],
        target: WhisperSttBlobTargetV1<'_>,
    ) -> WhisperSttFixtureBlobV1 {
        assert!(!plaintext.is_empty());
        let receipt_sha256: [u8; 32] = Sha256::digest(plaintext).into();
        let channel_binding = Sha256::digest(reference_id).to_vec();
        let delivery = BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            kernel_data.to_path_buf(),
        )
        .issue_blob_session(
            &self.expectation(),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: Sha256::digest([reference_id.as_slice(), b"write"].concat())[..16]
                    .to_vec(),
                capability_id: SOURCE_BLOB_CAPABILITY_ID_V1.to_owned(),
                operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
                channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
                reference_id: reference_id.to_vec(),
                declared_size: u64::try_from(plaintext.len()).expect("fixture Blob size"),
                backup_class: 1,
                ttl_seconds: 30,
                receipt_sha256: receipt_sha256.to_vec(),
                custody_source_proof: Vec::new(),
                evidence_id: Vec::new(),
                evidence_envelope_sha256: Vec::new(),
                custody_target_owner_id: target.owner_id.to_owned(),
                custody_target_module_id: target.module_id.to_owned(),
                custody_target_capability_id: target.capability_id.to_owned(),
            },
        )
        .expect("issue Whisper STT fixture Blob write");
        let custody_transfer_source_proof = delivery.custody_transfer_source_proof;
        BlobDataClient::new(delivery.data_socket_path)
            .expect("open Whisper STT fixture Blob client")
            .write(
                delivery.grant.expect("Whisper STT Blob write grant"),
                channel_binding,
                plaintext.to_vec(),
            )
            .expect("write Whisper STT fixture Blob");
        WhisperSttFixtureBlobV1 {
            reference_id,
            receipt_sha256,
            custody_transfer_source_proof,
            declared_size: u64::try_from(plaintext.len()).expect("fixture Blob size"),
        }
    }

    fn expectation(&self) -> ManagedRuntimeExpectation {
        ManagedRuntimeExpectation::new(
            SOURCE_REGISTRATION_ID_V1,
            SOURCE_RUNTIME_INSTANCE_ID_V1,
            SOURCE_MODULE_ID_V1,
            1,
            self.grant_epoch,
            [3; 32],
            None,
        )
    }
}
