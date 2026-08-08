use makosh_attachment_preview_api::wire::{
    AttachmentPreviewContentTypeV1, AttachmentPreviewErrorCodeV1, AttachmentPreviewKindV1,
    AttachmentPreviewStateV1,
};
use makosh_attachment_preview_core::{
    AttachmentPreviewCustodyDelegationIntentV1, AttachmentPreviewSafetyStateV1,
    AttachmentPreviewStatusV1, validate_attachment_preview_status_v1,
};
use sha2::{Digest, Sha256};

use crate::AttachmentPreviewPersistenceErrorV1;

pub const ATTACHMENT_PREVIEW_REALTIME_LIMIT_V1: u32 = 512;
pub const ATTACHMENT_PREVIEW_MAX_ATTEMPTS_V1: u32 = 8;
pub const ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1: u64 = 100 * 1024 * 1024;
pub const ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAttachmentPreviewRunV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentPreviewRunV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub request_fingerprint: [u8; 32],
    pub attachment_anchor_id: [u8; 16],
    pub status: AttachmentPreviewStatusV1,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateAttachmentPreviewRunOutcomeV1 {
    Created(PersistedAttachmentPreviewRunV1),
    Replayed(PersistedAttachmentPreviewRunV1),
    OperationCollision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistAttachmentPreviewFactOutcomeV1 {
    Recorded { transitioned_runs: u32 },
    Replayed,
    Conflict { rejected_runs: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingAttachmentPreviewCustodyDelegationV1 {
    pub intent: AttachmentPreviewCustodyDelegationIntentV1,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistAttachmentPreviewCustodyDelegationV1 {
    pub request_id: [u8; 16],
    pub run_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedAttachmentPreviewCustodyDelegationV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistAttachmentPreviewCustodyResultOutcomeV1 {
    Recorded,
    Replayed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewJobLeaseV1 {
    pub worker_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub lease_fence: u64,
    pub lease_expires_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewTargetBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedAttachmentPreviewJobV1 {
    pub logical_owner_id: String,
    pub job_id: [u8; 16],
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub delegation_request_id: [u8; 16],
    pub delegation_result_message_id: [u8; 16],
    pub delegation_result_envelope_sha256: [u8; 32],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub source_receipt_sha256: [u8; 32],
    pub source_declared_size: u64,
    pub custody_transfer_source_proof: Vec<u8>,
    pub target_blob_receipt: Option<PreviewTargetBlobReceiptV1>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub lease: PreviewJobLeaseV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderedAttachmentPreviewArtifactV1 {
    pub target_blob_receipt: PreviewTargetBlobReceiptV1,
    pub renderer_identity_sha256: [u8; 32],
    pub preview_kind: AttachmentPreviewKindV1,
    pub content_type: AttachmentPreviewContentTypeV1,
    pub preview_size_bytes: u64,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentPreviewArtifactV1 {
    pub run_id: [u8; 16],
    pub derived_reference_id: [u8; 16],
    pub derived_receipt_sha256: [u8; 32],
    pub source_receipt_sha256: [u8; 32],
    pub renderer_identity_sha256: [u8; 32],
    pub preview_kind: AttachmentPreviewKindV1,
    pub content_type: AttachmentPreviewContentTypeV1,
    pub preview_size_bytes: u64,
    pub truncated: bool,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub status: AttachmentPreviewStatusV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssueAttachmentPreviewTicketV1 {
    pub ticket_sha256: [u8; 32],
    pub device_actor_sha256: [u8; 32],
    pub run_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub now_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssuedAttachmentPreviewTicketV1 {
    pub run_id: [u8; 16],
    pub expires_at_unix_seconds: i64,
    pub content_type: AttachmentPreviewContentTypeV1,
    pub preview_size_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedeemedAttachmentPreviewTicketV1 {
    pub run_id: [u8; 16],
    pub derived_reference_id: [u8; 16],
    pub derived_receipt_sha256: [u8; 32],
    pub renderer_identity_sha256: [u8; 32],
    pub content_type: AttachmentPreviewContentTypeV1,
    pub preview_size_bytes: u64,
}

#[must_use]
pub fn attachment_preview_run_id_v1(logical_owner_id: &str, operation_id: [u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-preview.run.v1\0");
    hasher.update(logical_owner_id.as_bytes());
    hasher.update(operation_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

#[must_use]
pub fn attachment_preview_request_fingerprint_v1(attachment_anchor_id: [u8; 16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-preview.request.v1\0");
    hasher.update(attachment_anchor_id);
    hasher.finalize().into()
}

#[must_use]
pub fn attachment_preview_job_id_v1(
    run_id: [u8; 16],
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
    delegation_request_id: [u8; 16],
    delegation_result_message_id: [u8; 16],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-preview.job.v1\0");
    hasher.update(run_id);
    hasher.update(operation_id);
    hasher.update(attachment_anchor_id);
    hasher.update(delegation_request_id);
    hasher.update(delegation_result_message_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

pub(crate) fn validate_create(
    create: &CreateAttachmentPreviewRunV1,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    if !valid_owner(&create.logical_owner_id)
        || !valid_id16(&create.operation_id)
        || !valid_id16(&create.attachment_anchor_id)
        || !valid_timestamp_millis(create.created_at_unix_millis)
    {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

pub(crate) const fn state_code(value: AttachmentPreviewStateV1) -> i16 {
    match value {
        AttachmentPreviewStateV1::Accepted => 1,
        AttachmentPreviewStateV1::AwaitingEvidence => 2,
        AttachmentPreviewStateV1::Rendering => 3,
        AttachmentPreviewStateV1::Ready => 4,
        AttachmentPreviewStateV1::Unsupported => 5,
        AttachmentPreviewStateV1::Rejected => 6,
        AttachmentPreviewStateV1::Unspecified => 0,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<AttachmentPreviewStateV1, AttachmentPreviewPersistenceErrorV1> {
    AttachmentPreviewStateV1::try_from(i32::from(value))
        .ok()
        .filter(|state| *state != AttachmentPreviewStateV1::Unspecified)
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)
}

pub(crate) const fn kind_code(value: AttachmentPreviewKindV1) -> i16 {
    value as i16
}

pub(crate) fn kind_from_code(
    value: i16,
) -> Result<AttachmentPreviewKindV1, AttachmentPreviewPersistenceErrorV1> {
    AttachmentPreviewKindV1::try_from(i32::from(value))
        .ok()
        .filter(|kind| *kind != AttachmentPreviewKindV1::Unspecified)
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)
}

pub(crate) const fn content_type_code(value: AttachmentPreviewContentTypeV1) -> i16 {
    value as i16
}

pub(crate) fn content_type_from_code(
    value: i16,
) -> Result<AttachmentPreviewContentTypeV1, AttachmentPreviewPersistenceErrorV1> {
    AttachmentPreviewContentTypeV1::try_from(i32::from(value))
        .ok()
        .filter(|kind| *kind != AttachmentPreviewContentTypeV1::Unspecified)
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)
}

pub(crate) const fn error_code(value: AttachmentPreviewErrorCodeV1) -> i16 {
    value as i16
}

pub(crate) fn error_from_code(
    value: i16,
) -> Result<AttachmentPreviewErrorCodeV1, AttachmentPreviewPersistenceErrorV1> {
    AttachmentPreviewErrorCodeV1::try_from(i32::from(value))
        .ok()
        .filter(|error| *error != AttachmentPreviewErrorCodeV1::Unspecified)
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)
}

pub(crate) const fn safety_state_code(value: AttachmentPreviewSafetyStateV1) -> i16 {
    match value {
        AttachmentPreviewSafetyStateV1::DescriptorOnly => 1,
        AttachmentPreviewSafetyStateV1::BlobPending => 2,
        AttachmentPreviewSafetyStateV1::BlobAdmitted => 3,
        AttachmentPreviewSafetyStateV1::SafeForDelivery => 4,
        AttachmentPreviewSafetyStateV1::Quarantined => 5,
        AttachmentPreviewSafetyStateV1::Rejected => 6,
    }
}

pub(crate) fn safety_state_from_code(
    value: i16,
) -> Result<AttachmentPreviewSafetyStateV1, AttachmentPreviewPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentPreviewSafetyStateV1::DescriptorOnly),
        2 => Ok(AttachmentPreviewSafetyStateV1::BlobPending),
        3 => Ok(AttachmentPreviewSafetyStateV1::BlobAdmitted),
        4 => Ok(AttachmentPreviewSafetyStateV1::SafeForDelivery),
        5 => Ok(AttachmentPreviewSafetyStateV1::Quarantined),
        6 => Ok(AttachmentPreviewSafetyStateV1::Rejected),
        _ => Err(AttachmentPreviewPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

pub(crate) fn valid_worker(value: &str) -> bool {
    valid_owner(value)
}

pub(crate) const fn valid_timestamp_millis(value: i64) -> bool {
    value > 0
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn validate_status(status: &AttachmentPreviewStatusV1) -> bool {
    validate_attachment_preview_status_v1(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_and_job_ids_bind_owner_request_and_custody_result() {
        let operation = [7; 16];
        assert_ne!(
            attachment_preview_run_id_v1("alice", operation),
            attachment_preview_run_id_v1("bob", operation)
        );
        assert_ne!(
            attachment_preview_request_fingerprint_v1([1; 16]),
            attachment_preview_request_fingerprint_v1([2; 16])
        );
        let first = attachment_preview_job_id_v1([1; 16], [2; 16], [3; 16], [4; 16], [5; 16]);
        assert_ne!(
            first,
            attachment_preview_job_id_v1([1; 16], [2; 16], [3; 16], [4; 16], [6; 16])
        );
    }
}
