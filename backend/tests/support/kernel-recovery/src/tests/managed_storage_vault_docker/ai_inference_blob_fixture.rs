//! Kernel-issued workflow Blob input targeted only to the managed AI engine.

use super::*;

use makosh_ai_contracts::{
    AI_INFERENCE_BLOB_CAPABILITY_ID_V1, AI_INFERENCE_MODULE_ID_V1, AI_OWNER_V1,
};
use makosh_blob_client::BlobDataClient;
use makosh_kernel_control_store::{ModuleBlobOperationV1, ModuleBlobQuotaRequestV1};
use makosh_runtime_protocol::v1::{BlobDataOperationV1, ManagedRuntimeBlobSessionRequestV1};

use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};

const SOURCE_REGISTRATION_ID_V1: &str = "ai-inference-fixture-source";
const SOURCE_MODULE_ID_V1: &str = "workflow.ai-inference-fixture-source";
const SOURCE_OWNER_ID_V1: &str = "communication_reply_suggestion";
const SOURCE_BLOB_CAPABILITY_ID_V1: &str = "ai-inference-fixture-source.blob.v1";
const SOURCE_RUNTIME_INSTANCE_ID_V1: &str = "81818181818181818181818181818181";
const SOURCE_CUSTODY_SCOPE_ID_V1: &str = "communication_reply_suggestion.ai.source.v1";

pub(super) struct AiInferenceBlobSourceFixtureV1 {
    registration_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

pub(super) struct AiInferenceFixtureBlobV1 {
    pub(super) reference_id: [u8; 16],
    pub(super) receipt_sha256: [u8; 32],
    pub(super) custody_transfer_source_proof: Vec<u8>,
    pub(super) declared_size: u64,
}

impl AiInferenceBlobSourceFixtureV1 {
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
            4 * 1024 * 1024,
            SOURCE_CUSTODY_SCOPE_ID_V1,
            vec![ModuleBlobOperationV1::Write],
        );
        store
            .create_pending_registration_with_requests(
                &registration,
                &capabilities,
                &[],
                &[],
                std::slice::from_ref(&blob),
            )
            .expect("record AI inference fixture source");
        let grant_epoch = store
            .approve_module_registration(SOURCE_REGISTRATION_ID_V1, &capabilities)
            .expect("approve AI inference fixture Blob capability")
            .grant_epoch();
        store
            .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
                SOURCE_REGISTRATION_ID_V1,
                1,
                "ai-inference-fixture-source-distribution",
                SOURCE_MODULE_ID_V1,
                Sha256::digest(b"ai-inference-fixture-source-binary").into(),
                Sha256::digest(SOURCE_REGISTRATION_ID_V1.as_bytes()).into(),
                None,
            ))
            .expect("record AI inference fixture source binding");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                SOURCE_REGISTRATION_ID_V1,
                SOURCE_RUNTIME_INSTANCE_ID_V1,
                1,
                1,
                1,
                grant_epoch,
            ))
            .expect("record AI inference fixture source launch");
        Self {
            registration_id: SOURCE_REGISTRATION_ID_V1.to_owned(),
            runtime_generation: 1,
            grant_epoch,
        }
    }

    pub(super) fn write(
        &self,
        store: &Arc<SqliteControlStore>,
        supervisor: &ManagedRuntimeSupervisor,
        kernel_data: &Path,
        reference_id: [u8; 16],
        plaintext: &[u8],
    ) -> AiInferenceFixtureBlobV1 {
        assert!(!plaintext.is_empty());
        let receipt_sha256: [u8; 32] = Sha256::digest(plaintext).into();
        let channel_binding = Sha256::digest(reference_id).to_vec();
        let request_digest = Sha256::digest([reference_id.as_slice(), b"write"].concat());
        let delivery = BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            kernel_data.to_path_buf(),
        )
        .issue_blob_session(
            &ManagedRuntimeExpectation::new(
                &self.registration_id,
                SOURCE_RUNTIME_INSTANCE_ID_V1,
                SOURCE_MODULE_ID_V1,
                self.runtime_generation,
                self.grant_epoch,
                [3; 32],
                None,
            ),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: request_digest[..16].to_vec(),
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
                custody_target_owner_id: AI_OWNER_V1.to_owned(),
                custody_target_module_id: AI_INFERENCE_MODULE_ID_V1.to_owned(),
                custody_target_capability_id: AI_INFERENCE_BLOB_CAPABILITY_ID_V1.to_owned(),
            },
        )
        .expect("issue AI inference fixture Blob write");
        let custody_transfer_source_proof = delivery.custody_transfer_source_proof;
        BlobDataClient::new(delivery.data_socket_path)
            .expect("open AI inference fixture Blob client")
            .write(
                delivery.grant.expect("AI inference Blob write grant"),
                channel_binding,
                plaintext.to_vec(),
            )
            .expect("write AI inference fixture Blob");
        AiInferenceFixtureBlobV1 {
            reference_id,
            receipt_sha256,
            custody_transfer_source_proof,
            declared_size: u64::try_from(plaintext.len()).expect("fixture Blob size"),
        }
    }
}
