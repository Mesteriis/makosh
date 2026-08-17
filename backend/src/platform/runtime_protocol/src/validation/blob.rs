//! Structural validation for Blob managed-runtime configuration and status.

use crate::v1::{
    BlobBackupClassV1, BlobCustodyReleaseGrantV1, BlobCustodyReleaseOutcomeV1,
    BlobCustodyReleaseReasonV1, BlobCustodyReleaseResponseV1, BlobRuntimeConfigurationV1,
    BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1, BlobRuntimeStateV1,
    BlobRuntimeStatusV1, blob_runtime_control_request_v1::Operation as RequestOperation,
    blob_runtime_control_response_v1::Result as ResponseResult,
};

const MAX_PATH_BYTES: usize = 4_096;
const MAX_UNIX_SOCKET_PATH_BYTES: usize = 100;
const MAX_BLOB_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_CUSTODY_RELEASE_GRACE_PERIOD_MS: u64 = 7 * 24 * 60 * 60 * 1_000;
const ID_BYTES: usize = 16;
const SHA256_BYTES: usize = 32;
const P256_RAW_SIGNATURE_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobRuntimeValidationErrorV1 {
    InvalidConfiguration,
    InvalidRequest,
    InvalidResponse,
    InvalidStatus,
}

pub fn validate_blob_runtime_configuration(
    configuration: &BlobRuntimeConfigurationV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    if !configuration.data_dir.starts_with('/')
        || configuration.data_dir.len() > MAX_PATH_BYTES
        || !configuration.data_socket_path.starts_with('/')
        || configuration.data_socket_path.len() > MAX_UNIX_SOCKET_PATH_BYTES
        || configuration.maximum_blob_bytes == 0
        || configuration.maximum_blob_bytes > MAX_BLOB_BYTES
        || !valid_id(&configuration.vault_instance_id)
        || configuration.vault_runtime_generation == 0
        || configuration.vault_hpke_public_key_x25519.len() != 32
        || !valid_id(&configuration.kernel_instance_id)
        || configuration.kernel_authorization_public_key_sec1.len() != 65
        || configuration.custody_release_grace_period_ms == 0
        || configuration.custody_release_grace_period_ms > MAX_CUSTODY_RELEASE_GRACE_PERIOD_MS
    {
        return Err(BlobRuntimeValidationErrorV1::InvalidConfiguration);
    }
    Ok(())
}

pub fn validate_blob_runtime_control_request(
    request: &BlobRuntimeControlRequestV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    match request.operation.as_ref() {
        Some(RequestOperation::GetStatus(_)) => Ok(()),
        Some(RequestOperation::ReleaseCustody(request)) => request
            .grant
            .as_ref()
            .filter(|grant| valid_release_grant(grant))
            .map(|_| ())
            .ok_or(BlobRuntimeValidationErrorV1::InvalidRequest),
        None => Err(BlobRuntimeValidationErrorV1::InvalidRequest),
    }
}

pub fn validate_blob_runtime_control_response(
    response: &BlobRuntimeControlResponseV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    match (&response.result, response.error_code.is_empty()) {
        (Some(ResponseResult::Status(status)), true) => validate_blob_runtime_status(status),
        (Some(ResponseResult::CustodyRelease(release)), true) => validate_release_response(release),
        (None, false) if valid_blocker_code(&response.error_code) => Ok(()),
        _ => Err(BlobRuntimeValidationErrorV1::InvalidResponse),
    }
}

fn valid_release_grant(grant: &BlobCustodyReleaseGrantV1) -> bool {
    grant.major == 1
        && valid_id(&grant.kernel_instance_id)
        && fixed_nonzero(&grant.operation_id, ID_BYTES)
        && valid_id(&grant.owner_id)
        && valid_id(&grant.registration_id)
        && valid_id(&grant.capability_id)
        && valid_id(&grant.runtime_instance_id)
        && grant.runtime_generation > 0
        && grant.grant_epoch > 0
        && fixed_nonzero(&grant.reference_id, ID_BYTES)
        && (1..=MAX_BLOB_BYTES).contains(&grant.declared_size)
        && fixed_nonzero(&grant.receipt_sha256, SHA256_BYTES)
        && fixed_nonzero(&grant.custody_source_proof_sha256, SHA256_BYTES)
        && valid_id(&grant.custody_scope_id)
        && matches!(
            BlobCustodyReleaseReasonV1::try_from(grant.reason),
            Ok(
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
                    | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
                    | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalCancelledV1
            )
        )
        && grant.issued_at_unix_ms > 0
        && grant.expires_at_unix_ms > grant.issued_at_unix_ms
        && grant.blob_runtime_generation > 0
        && grant.kernel_authorization_signature_raw.len() == P256_RAW_SIGNATURE_BYTES
        && matches!(
            BlobBackupClassV1::try_from(grant.backup_class),
            Ok(BlobBackupClassV1::BlobBackupClassRequiredV1
                | BlobBackupClassV1::BlobBackupClassRebuildableV1
                | BlobBackupClassV1::BlobBackupClassExcludedV1)
        )
}

fn validate_release_response(
    response: &BlobCustodyReleaseResponseV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    let outcome = BlobCustodyReleaseOutcomeV1::try_from(response.outcome)
        .map_err(|_| BlobRuntimeValidationErrorV1::InvalidResponse)?;
    if !fixed_nonzero(&response.operation_id, ID_BYTES)
        || response.delete_not_before_unix_ms == 0
        || !response.error_code.is_empty()
        || !matches!(
            outcome,
            BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeAcceptedV1
                | BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeExistingV1
                | BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeAlreadyReleasedV1
        )
    {
        return Err(BlobRuntimeValidationErrorV1::InvalidResponse);
    }
    Ok(())
}

pub fn validate_blob_runtime_status(
    status: &BlobRuntimeStatusV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    let state = BlobRuntimeStateV1::try_from(status.state)
        .map_err(|_| BlobRuntimeValidationErrorV1::InvalidStatus)?;
    if status.runtime_generation == 0
        || status.vault_runtime_generation == 0
        || status.maximum_blob_bytes == 0
        || status.maximum_blob_bytes > MAX_BLOB_BYTES
    {
        return Err(BlobRuntimeValidationErrorV1::InvalidStatus);
    }
    match state {
        BlobRuntimeStateV1::Ready if status.blocker_code.is_empty() => Ok(()),
        BlobRuntimeStateV1::Blocked if valid_blocker_code(&status.blocker_code) => Ok(()),
        _ => Err(BlobRuntimeValidationErrorV1::InvalidStatus),
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn valid_blocker_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn fixed_nonzero(value: &[u8], expected: usize) -> bool {
    value.len() == expected && value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use crate::v1::{
        BlobCustodyReleaseGrantV1, BlobCustodyReleaseReasonV1, BlobCustodyReleaseRequestV1,
        BlobRuntimeControlRequestV1, blob_runtime_control_request_v1::Operation,
    };

    use super::*;

    #[test]
    fn custody_release_is_exact_bounded_and_signed() {
        let request = BlobRuntimeControlRequestV1 {
            operation: Some(Operation::ReleaseCustody(BlobCustodyReleaseRequestV1 {
                grant: Some(release_grant()),
            })),
        };
        assert_eq!(validate_blob_runtime_control_request(&request), Ok(()));
        let mut invalid = request;
        invalid
            .operation
            .as_mut()
            .and_then(|operation| match operation {
                Operation::ReleaseCustody(request) => request.grant.as_mut(),
                Operation::GetStatus(_) => None,
            })
            .expect("release grant")
            .custody_source_proof_sha256
            .clear();
        assert_eq!(
            validate_blob_runtime_control_request(&invalid),
            Err(BlobRuntimeValidationErrorV1::InvalidRequest)
        );
    }

    fn release_grant() -> BlobCustodyReleaseGrantV1 {
        BlobCustodyReleaseGrantV1 {
            major: 1,
            kernel_instance_id: "kernel-1".to_owned(),
            operation_id: vec![1; 16],
            owner_id: "owner-1".to_owned(),
            registration_id: "registration-1".to_owned(),
            capability_id: "blob.release.v1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
            reference_id: vec![4; 16],
            declared_size: 5,
            receipt_sha256: vec![6; 32],
            custody_source_proof_sha256: vec![7; 32],
            custody_scope_id: "scope-1".to_owned(),
            reason: BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1 as i32,
            issued_at_unix_ms: 8,
            expires_at_unix_ms: 9,
            blob_runtime_generation: 10,
            kernel_authorization_signature_raw: vec![11; 64],
            backup_class: BlobBackupClassV1::BlobBackupClassRequiredV1 as i32,
            reference_expires_at_unix_ms: 0,
        }
    }
}
