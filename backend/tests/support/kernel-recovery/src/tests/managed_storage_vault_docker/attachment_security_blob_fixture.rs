//! Kernel-issued Blob write fixture for Attachment Security scan inputs.

use super::*;

use makosh_attachment_security_contract::admission::{
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID,
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID,
};
use makosh_blob_client::{BlobClientError, BlobDataClient};
use makosh_kernel_control_store::{ModuleBlobOperationV1, ModuleBlobQuotaRequestV1};
use makosh_runtime_protocol::v1::{BlobDataOperationV1, ManagedRuntimeBlobSessionRequestV1};

use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};

const SOURCE_REGISTRATION_ID: &str = "attachment-security-fixture-source";
const SOURCE_MODULE_ID: &str = "integration.attachment-security-fixture-source";
const SOURCE_OWNER_ID: &str = "mail";
const SOURCE_BLOB_CAPABILITY_ID: &str = "attachment-security-fixture-source.blob.v1";
const SOURCE_RUNTIME_INSTANCE_ID: &str = "71717171717171717171717171717171";
const SOURCE_CUSTODY_SCOPE_ID: &str = "mail.attachment.content.v1";
const AUTHORITY_SOURCE_REGISTRATION_ID: &str = "attachment-security-authority-source";
const AUTHORITY_SOURCE_MODULE_ID: &str = "integration.attachment-security-authority-source";
const AUTHORITY_SOURCE_BLOB_CAPABILITY_ID: &str = "attachment-security-authority-source.blob.v1";
const AUTHORITY_SOURCE_RUNTIME_INSTANCE_ID: &str = "73737373737373737373737373737373";

pub(super) struct AttachmentSecurityBlobSourceFixture {
    registration_id: String,
    module_id: String,
    capability_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

pub(super) struct AttachmentSecurityFixtureBlobV1 {
    pub(super) reference_id: [u8; 16],
    pub(super) receipt_sha256: [u8; 32],
    pub(super) custody_transfer_source_proof: Vec<u8>,
    pub(super) declared_size: u64,
}

impl AttachmentSecurityBlobSourceFixture {
    pub(super) fn admit(store: &SqliteControlStore) -> Self {
        Self::admit_with_identity(
            store,
            SOURCE_REGISTRATION_ID,
            SOURCE_MODULE_ID,
            SOURCE_BLOB_CAPABILITY_ID,
            SOURCE_RUNTIME_INSTANCE_ID,
        )
    }

    pub(super) fn admit_authority_source(store: &SqliteControlStore) -> Self {
        Self::admit_with_identity(
            store,
            AUTHORITY_SOURCE_REGISTRATION_ID,
            AUTHORITY_SOURCE_MODULE_ID,
            AUTHORITY_SOURCE_BLOB_CAPABILITY_ID,
            AUTHORITY_SOURCE_RUNTIME_INSTANCE_ID,
        )
    }

    fn admit_with_identity(
        store: &SqliteControlStore,
        registration_id: &str,
        module_id: &str,
        capability_id: &str,
        runtime_instance_id: &str,
    ) -> Self {
        let registration = ModuleRegistration::new(
            registration_id,
            module_id,
            SOURCE_OWNER_ID,
            Sha256::digest(registration_id.as_bytes()).into(),
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = [capability_id.to_owned()];
        let blob = ModuleBlobQuotaRequestV1::new(
            registration_id,
            capability_id,
            SOURCE_OWNER_ID,
            64 * 1024 * 1024,
            SOURCE_CUSTODY_SCOPE_ID,
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
            .expect("record Attachment Security fixture source");
        let grant_epoch = store
            .approve_module_registration(registration_id, &capabilities)
            .expect("approve Attachment Security fixture Blob capability")
            .grant_epoch();
        store
            .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
                registration_id,
                1,
                format!("{registration_id}-distribution"),
                module_id,
                Sha256::digest(format!("{registration_id}-binary")).into(),
                Sha256::digest(registration_id.as_bytes()).into(),
                None,
            ))
            .expect("record Attachment Security fixture source binding");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                registration_id,
                runtime_instance_id,
                1,
                1,
                1,
                grant_epoch,
            ))
            .expect("record Attachment Security fixture source launch");
        Self {
            registration_id: registration_id.to_owned(),
            module_id: module_id.to_owned(),
            capability_id: capability_id.to_owned(),
            runtime_instance_id: runtime_instance_id.to_owned(),
            runtime_generation: 1,
            grant_epoch,
        }
    }

    pub(super) fn advance_runtime_generation(
        &mut self,
        store: &SqliteControlStore,
        successor_runtime_instance_id: &str,
    ) {
        let runtime_generation = self
            .runtime_generation
            .checked_add(1)
            .expect("Attachment Security fixture source generation");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                &self.registration_id,
                successor_runtime_instance_id,
                1,
                1,
                runtime_generation,
                self.grant_epoch,
            ))
            .expect("record Attachment Security fixture source successor launch");
        self.runtime_instance_id = successor_runtime_instance_id.to_owned();
        self.runtime_generation = runtime_generation;
    }

    pub(super) fn revoke(&self, store: &SqliteControlStore) {
        store
            .transition_module_registration(&self.registration_id, ModuleRegistrationState::Revoked)
            .expect("revoke Attachment Security fixture source");
    }

    pub(super) fn write(
        &self,
        store: &Arc<SqliteControlStore>,
        supervisor: &ManagedRuntimeSupervisor,
        kernel_data: &Path,
        reference_id: [u8; 16],
        plaintext: &[u8],
    ) -> AttachmentSecurityFixtureBlobV1 {
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
                &self.runtime_instance_id,
                &self.module_id,
                self.runtime_generation,
                self.grant_epoch,
                [3; 32],
                None,
            ),
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: request_digest[..16].to_vec(),
                capability_id: self.capability_id.clone(),
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
                custody_target_owner_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID
                    .to_owned(),
                custody_target_module_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID
                    .to_owned(),
                custody_target_capability_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID
                    .to_owned(),
            },
        )
        .expect("issue Attachment Security fixture Blob write");
        let custody_transfer_source_proof = delivery.custody_transfer_source_proof;
        BlobDataClient::new(delivery.data_socket_path)
            .expect("open Attachment Security fixture Blob client")
            .write(
                delivery
                    .grant
                    .expect("Attachment Security Blob write grant"),
                channel_binding,
                plaintext.to_vec(),
            )
            .expect("write Attachment Security fixture Blob");
        AttachmentSecurityFixtureBlobV1 {
            reference_id,
            receipt_sha256,
            custody_transfer_source_proof,
            declared_size: u64::try_from(plaintext.len()).expect("fixture Blob size"),
        }
    }
}

pub(super) fn assert_attachment_security_source_blob_read_is_denied(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
    target: &StartedAttachmentSecurityRuntime,
    blob: &AttachmentSecurityFixtureBlobV1,
) {
    let channel_binding = Sha256::digest(b"attachment-security-source-read-denial").to_vec();
    let delivery = BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        kernel_data.to_path_buf(),
    )
    .issue_blob_session(
        &ManagedRuntimeExpectation::new(
            &target.registration_id,
            &target.runtime_instance_id,
            ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID,
            target.runtime_generation,
            target.grant_epoch,
            [4; 32],
            None,
        ),
        ManagedRuntimeBlobSessionRequestV1 {
            request_id: vec![91; 16],
            capability_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID.to_owned(),
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1 as u32,
            channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
            reference_id: blob.reference_id.to_vec(),
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
    .expect("issue target-fenced source Blob read session");
    let error = BlobDataClient::new(delivery.data_socket_path)
        .expect("open target-fenced source Blob read client")
        .read_range(
            delivery.grant.expect("source Blob read grant"),
            channel_binding,
            0,
            blob.declared_size,
        )
        .expect_err("Attachment Security must not read the integration-owned source reference");
    assert_eq!(
        error,
        BlobClientError::Rejected("data_request_denied".to_owned())
    );
}
