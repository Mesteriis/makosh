//! Issues and routes one exact Kernel-signed Blob custody release.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_kernel_control_store::ModuleBlobOperationV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        BlobCustodyReleaseGrantV1, BlobCustodyReleaseReasonV1, BlobCustodyReleaseRequestV1,
        BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1,
        ManagedRuntimeBlobCustodyReleaseDeliveryV1, ManagedRuntimeBlobCustodyReleaseRequestV1,
        blob_runtime_control_request_v1::Operation as BlobOperation,
        blob_runtime_control_response_v1::Result as BlobResult,
    },
    validation::blob::validate_blob_runtime_control_response,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::platform::blob::{
    binding::BLOB_PROCESS_ID,
    catalog,
    session::{CustodySourceProofUseV1, proof_authorizes_target, verify_custody_source_proof},
    status,
};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobCustodyReleaseHandler, ManagedRuntimeExpectation,
};
use crate::runtime::lifecycle::fence::current_managed_runtime_matches;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

const RELEASE_GRANT_TTL_MS: u64 = 30_000;
const MAX_SOURCE_PROOF_BYTES: usize = 2_048;
const MAX_BLOB_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) struct BlobCustodyReleaseHandlerV1 {
    store: Arc<SqliteControlStore>,
    relay: ManagedRuntimeRelayPort,
    data_dir: PathBuf,
}

impl BlobCustodyReleaseHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        relay: ManagedRuntimeRelayPort,
        data_dir: PathBuf,
    ) -> Self {
        Self {
            store,
            relay,
            data_dir,
        }
    }
}

impl ManagedRuntimeBlobCustodyReleaseHandler for BlobCustodyReleaseHandlerV1 {
    fn release_blob_custody(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobCustodyReleaseRequestV1,
    ) -> Result<ManagedRuntimeBlobCustodyReleaseDeliveryV1, String> {
        if !current_managed_runtime_matches(
            &*self.store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| denied())?
        {
            return Err(denied());
        }
        let target = catalog::resolve(&*self.store)?
            .into_iter()
            .find(|entry| {
                entry.registration_id() == expectation.registration_id()
                    && entry.capability_id() == request.capability_id
                    && entry.grant_epoch() == expectation.grant_epoch()
                    && entry
                        .request()
                        .allows(ModuleBlobOperationV1::ReleaseCustody)
            })
            .ok_or_else(denied)?;
        if request.declared_size > target.request().max_bytes() {
            return Err(denied());
        }

        let now = now_unix_ms()?;
        let signer = FileDeviceSigner::open_for_instance(&self.data_dir)?;
        let proof = verify_custody_source_proof(
            &request.custody_source_proof,
            &signer.public_key_sec1(),
            self.store.snapshot().instance_id(),
            now,
            CustodySourceProofUseV1::Release,
        )
        .map_err(|_| denied())?;
        if proof.declared_size != request.declared_size
            || proof.receipt_sha256 != request.receipt_sha256
            || !proof_authorizes_target(
                &proof,
                target.request().owner_id(),
                expectation.module_id(),
                &request.capability_id,
            )
        {
            return Err(denied());
        }

        let blob = status::read_current(&self.store, &self.relay)?;
        let authorized_blob_generation = blob.runtime_generation();
        let expires_at_unix_ms = now
            .checked_add(RELEASE_GRANT_TTL_MS)
            .ok_or_else(unavailable)?;
        let mut grant = BlobCustodyReleaseGrantV1 {
            major: 1,
            kernel_instance_id: self.store.snapshot().instance_id().to_owned(),
            operation_id: request.operation_id.clone(),
            owner_id: target.request().owner_id().to_owned(),
            registration_id: expectation.registration_id().to_owned(),
            capability_id: request.capability_id,
            runtime_instance_id: expectation.runtime_instance_id().to_owned(),
            runtime_generation: expectation.runtime_generation(),
            grant_epoch: expectation.grant_epoch(),
            reference_id: request.reference_id,
            declared_size: request.declared_size,
            receipt_sha256: request.receipt_sha256,
            custody_source_proof_sha256: Sha256::digest(&request.custody_source_proof).to_vec(),
            custody_scope_id: target.request().custody_scope_id().to_owned(),
            reason: request.reason,
            issued_at_unix_ms: now,
            expires_at_unix_ms,
            blob_runtime_generation: authorized_blob_generation,
            kernel_authorization_signature_raw: Vec::new(),
            backup_class: proof.backup_class,
            reference_expires_at_unix_ms: proof.reference_expires_at_unix_ms,
        };
        let mut message = b"makosh.blob-custody-release.v1\0".to_vec();
        message.extend_from_slice(&grant.encode_to_vec());
        grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();

        let response = self.relay.relay(
            BLOB_PROCESS_ID,
            BlobRuntimeControlRequestV1 {
                operation: Some(BlobOperation::ReleaseCustody(BlobCustodyReleaseRequestV1 {
                    grant: Some(grant),
                })),
            }
            .encode_to_vec(),
        )?;
        let response =
            BlobRuntimeControlResponseV1::decode(response.as_slice()).map_err(|_| unavailable())?;
        validate_blob_runtime_control_response(&response).map_err(|_| unavailable())?;
        match response.result {
            Some(BlobResult::CustodyRelease(release))
                if response.error_code.is_empty()
                    && release.operation_id == request.operation_id =>
            {
                Ok(ManagedRuntimeBlobCustodyReleaseDeliveryV1 {
                    operation_id: release.operation_id,
                    outcome: release.outcome,
                    delete_not_before_unix_ms: release.delete_not_before_unix_ms,
                })
            }
            None if response.error_code.contains("unavailable") => Err(unavailable()),
            _ => match status::read_current(&self.store, &self.relay) {
                Ok(current) if current.runtime_generation() == authorized_blob_generation => {
                    Err(denied())
                }
                Ok(_) | Err(_) => Err(unavailable()),
            },
        }
    }
}

pub(crate) fn valid_request(request: &ManagedRuntimeBlobCustodyReleaseRequestV1) -> bool {
    request.operation_id.len() == 16
        && request.operation_id.iter().any(|byte| *byte != 0)
        && valid_token(&request.capability_id)
        && request.reference_id.len() == 16
        && request.reference_id.iter().any(|byte| *byte != 0)
        && (1..=MAX_BLOB_BYTES).contains(&request.declared_size)
        && request.receipt_sha256.len() == 32
        && request.receipt_sha256.iter().any(|byte| *byte != 0)
        && !request.custody_source_proof.is_empty()
        && request.custody_source_proof.len() <= MAX_SOURCE_PROOF_BYTES
        && matches!(
            BlobCustodyReleaseReasonV1::try_from(request.reason),
            Ok(
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
                    | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
                    | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalCancelledV1
            )
        )
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn now_unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| unavailable())?
        .as_millis()
        .try_into()
        .map_err(|_| unavailable())
}

fn denied() -> String {
    "managed runtime Blob custody release is denied".to_owned()
}

fn unavailable() -> String {
    "managed runtime Blob custody release is unavailable".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_exact_and_bounded() {
        let request = ManagedRuntimeBlobCustodyReleaseRequestV1 {
            operation_id: vec![1; 16],
            capability_id: "attachment_security.blob.v1".to_owned(),
            reference_id: vec![2; 16],
            declared_size: 3,
            receipt_sha256: vec![4; 32],
            custody_source_proof: vec![5; 64],
            reason: BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1 as i32,
        };
        assert!(valid_request(&request));
        assert!(!valid_request(&ManagedRuntimeBlobCustodyReleaseRequestV1 {
            operation_id: vec![0; 16],
            ..request
        }));
    }
}
