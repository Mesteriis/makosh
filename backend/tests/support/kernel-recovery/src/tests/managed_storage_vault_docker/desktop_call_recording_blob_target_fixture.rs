//! Exact call-transcription Blob target used before the workflow runtime is implemented.

use super::*;

use makosh_blob_client::BlobDataClient;
use makosh_call_transcription_ingress::{
    OWNER_ID_V1 as TARGET_OWNER_ID_V1, TARGET_BLOB_CAPABILITY_ID_V1, TARGET_MODULE_ID_V1,
};
use makosh_kernel_control_store::{ModuleBlobOperationV1, ModuleBlobQuotaRequestV1};
use makosh_runtime_protocol::v1::{BlobDataOperationV1, ManagedRuntimeBlobSessionRequestV1};

const TARGET_REGISTRATION_ID_V1: &str = "call-transcription-recording-target-fixture";
const TARGET_RUNTIME_INSTANCE_ID_V1: &str = "92929292929292929292929292929292";
const TARGET_CUSTODY_SCOPE_ID_V1: &str = "call_transcription.private_recording_source.v1";

pub(super) struct DesktopRecordingBlobTargetFixtureV1 {
    grant_epoch: u64,
}

pub(super) struct DesktopRecordingReadyBlobV1<'a> {
    pub(super) reference_id: [u8; 16],
    pub(super) receipt_sha256: [u8; 32],
    pub(super) custody_transfer_source_proof: &'a [u8],
    pub(super) declared_size: u64,
    pub(super) evidence_id: [u8; 16],
    pub(super) evidence_envelope_sha256: [u8; 32],
}

impl DesktopRecordingBlobTargetFixtureV1 {
    pub(super) fn admit(store: &SqliteControlStore) -> Self {
        let registration = ModuleRegistration::new(
            TARGET_REGISTRATION_ID_V1,
            TARGET_MODULE_ID_V1,
            TARGET_OWNER_ID_V1,
            Sha256::digest(TARGET_REGISTRATION_ID_V1.as_bytes()).into(),
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = [TARGET_BLOB_CAPABILITY_ID_V1.to_owned()];
        let blob = ModuleBlobQuotaRequestV1::new(
            TARGET_REGISTRATION_ID_V1,
            TARGET_BLOB_CAPABILITY_ID_V1,
            TARGET_OWNER_ID_V1,
            64 * 1024 * 1024,
            TARGET_CUSTODY_SCOPE_ID_V1,
            vec![
                ModuleBlobOperationV1::CustodyTransfer,
                ModuleBlobOperationV1::ReadRange,
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
            .expect("record call-transcription recording target fixture");
        let grant_epoch = store
            .approve_module_registration(TARGET_REGISTRATION_ID_V1, &capabilities)
            .expect("approve call-transcription recording target Blob capability")
            .grant_epoch();
        store
            .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
                TARGET_REGISTRATION_ID_V1,
                1,
                "call-transcription-recording-target-fixture-distribution",
                TARGET_MODULE_ID_V1,
                Sha256::digest(b"call-transcription-recording-target-fixture-binary").into(),
                Sha256::digest(TARGET_REGISTRATION_ID_V1.as_bytes()).into(),
                None,
            ))
            .expect("record call-transcription recording target fixture binding");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                TARGET_REGISTRATION_ID_V1,
                TARGET_RUNTIME_INSTANCE_ID_V1,
                1,
                1,
                1,
                grant_epoch,
            ))
            .expect("record call-transcription recording target fixture launch");
        Self { grant_epoch }
    }

    pub(super) fn accept_and_read(
        &self,
        store: &Arc<SqliteControlStore>,
        supervisor: &ManagedRuntimeSupervisor,
        kernel_data: &Path,
        blob: &DesktopRecordingReadyBlobV1<'_>,
    ) -> Vec<u8> {
        let transfer_channel_binding =
            Sha256::digest([blob.reference_id.as_slice(), b"recording-transfer"].concat()).to_vec();
        let transfer = BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            kernel_data.to_path_buf(),
        )
        .issue_blob_session(
            &self.expectation(),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: Sha256::digest(
                    [blob.reference_id.as_slice(), b"recording-transfer-request"].concat(),
                )[..16]
                    .to_vec(),
                capability_id: TARGET_BLOB_CAPABILITY_ID_V1.to_owned(),
                operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as u32,
                channel_binding_sha256: Sha256::digest(&transfer_channel_binding).to_vec(),
                reference_id: blob.reference_id.to_vec(),
                declared_size: blob.declared_size,
                backup_class: 1,
                ttl_seconds: 30,
                receipt_sha256: blob.receipt_sha256.to_vec(),
                custody_source_proof: blob.custody_transfer_source_proof.to_vec(),
                evidence_id: blob.evidence_id.to_vec(),
                evidence_envelope_sha256: blob.evidence_envelope_sha256.to_vec(),
                custody_target_owner_id: String::new(),
                custody_target_module_id: String::new(),
                custody_target_capability_id: String::new(),
            },
        )
        .expect("issue call-transcription recording custody transfer");
        let transfer_grant = transfer
            .custody_transfer_grant
            .expect("call-transcription recording custody transfer grant");
        let target_reference_id = transfer_grant.target_reference_id.clone();
        BlobDataClient::new(transfer.data_socket_path)
            .expect("open recording custody transfer Blob client")
            .custody_transfer(transfer_grant, transfer_channel_binding)
            .expect("transfer recording Blob custody");

        let read_channel_binding =
            Sha256::digest([target_reference_id.as_slice(), b"recording-read"].concat()).to_vec();
        let read = BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            kernel_data.to_path_buf(),
        )
        .issue_blob_session(
            &self.expectation(),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: Sha256::digest(
                    [target_reference_id.as_slice(), b"recording-read-request"].concat(),
                )[..16]
                    .to_vec(),
                capability_id: TARGET_BLOB_CAPABILITY_ID_V1.to_owned(),
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
        .expect("issue call-transcription recording Blob read");
        BlobDataClient::new(read.data_socket_path)
            .expect("open call-transcription recording Blob client")
            .read_range(
                read.grant.expect("recording Blob read grant"),
                read_channel_binding,
                0,
                blob.declared_size,
            )
            .expect("read transferred recording Blob")
    }

    fn expectation(&self) -> ManagedRuntimeExpectation {
        ManagedRuntimeExpectation::new(
            TARGET_REGISTRATION_ID_V1,
            TARGET_RUNTIME_INSTANCE_ID_V1,
            TARGET_MODULE_ID_V1,
            1,
            self.grant_epoch,
            [4; 32],
            None,
        )
    }
}
