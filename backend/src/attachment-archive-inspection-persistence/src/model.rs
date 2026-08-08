use makosh_attachment_archive_inspection_core::{
    ArchiveEntryInspectionV1, ArchiveEntryKindV1, ArchiveInspectionErrorV1,
    ArchiveInspectionReportV1, ArchiveInspectionRequestV1, ArchiveInspectionStateV1,
    ArchiveInspectionStatusV1,
};
use makosh_attachment_archive_inspection_ingress::wire::RequestArchiveInspectionCustodyDelegationV1;
use sha2::{Digest, Sha256};

use crate::ArchiveInspectionPersistenceErrorV1;

pub const ARCHIVE_INSPECTION_MAX_ATTEMPTS_V1: u32 = 8;
pub const ARCHIVE_INSPECTION_REALTIME_LIMIT_V1: u32 = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateArchiveInspectionRunV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedArchiveInspectionRunV1 {
    pub logical_owner_id: String,
    pub request: ArchiveInspectionRequestV1,
    pub request_fingerprint: [u8; 32],
    pub status: ArchiveInspectionStatusV1,
    pub rejection_evidence_id: Option<[u8; 16]>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateArchiveInspectionRunOutcomeV1 {
    Created(PersistedArchiveInspectionRunV1),
    Replayed(PersistedArchiveInspectionRunV1),
    OperationCollision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistArchiveInspectionFactOutcomeV1 {
    Recorded { transitioned_runs: u32 },
    Duplicate,
    Conflict { rejected_runs: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingArchiveInspectionCustodyDelegationV1 {
    pub request: RequestArchiveInspectionCustodyDelegationV1,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedArchiveInspectionCustodyDelegationV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistArchiveInspectionCustodyResultOutcomeV1 {
    Recorded,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionLeaseV1 {
    pub worker_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub lease_fence: u64,
    pub lease_expires_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedArchiveInspectionJobV1 {
    pub logical_owner_id: String,
    pub job_id: [u8; 16],
    pub request: ArchiveInspectionRequestV1,
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub delegation_request_id: [u8; 16],
    pub delegation_result_message_id: [u8; 16],
    pub delegation_result_envelope_sha256: [u8; 32],
    pub source_reference_id: [u8; 16],
    pub target_blob_receipt: Option<ArchiveInspectionTargetBlobReceiptV1>,
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub safety_evidence_id: [u8; 16],
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub lease: ArchiveInspectionLeaseV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionTargetBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: ArchiveInspectionStateV1,
    pub state_revision: u64,
    pub error: Option<ArchiveInspectionErrorV1>,
    pub occurred_at_unix_millis: i64,
}

#[must_use]
pub fn archive_inspection_run_id_v1(logical_owner_id: &str, operation_id: [u8; 16]) -> [u8; 16] {
    digest16(
        b"makosh.attachment-archive-inspection.run.v1\0",
        &[logical_owner_id.as_bytes(), &operation_id],
    )
}

#[must_use]
pub fn archive_inspection_request_fingerprint_v1(attachment_anchor_id: [u8; 16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-archive-inspection.request.v1\0");
    hasher.update(attachment_anchor_id);
    hasher.finalize().into()
}

#[must_use]
pub fn archive_inspection_job_id_v1(
    request: &ArchiveInspectionRequestV1,
    candidate_message_id: [u8; 16],
    safety_message_id: [u8; 16],
    delegation_request_id: [u8; 16],
    delegation_result_message_id: [u8; 16],
) -> [u8; 16] {
    digest16(
        b"makosh.attachment-archive-inspection.job.v1\0",
        &[
            &request.run_id,
            &request.attachment_anchor_id,
            &candidate_message_id,
            &safety_message_id,
            &delegation_request_id,
            &delegation_result_message_id,
        ],
    )
}

#[must_use]
pub fn archive_inspection_terminal_evidence_id_v1(
    run_id: [u8; 16],
    error: ArchiveInspectionErrorV1,
) -> [u8; 16] {
    digest16(
        b"makosh.attachment-archive-inspection.terminal-error.v1\0",
        &[
            &run_id,
            &[u8::try_from(error_code(error)).expect("bounded error")],
        ],
    )
}

pub(crate) fn validate_create(
    create: &CreateArchiveInspectionRunV1,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    if !valid_owner(&create.logical_owner_id)
        || !valid_id16(&create.operation_id)
        || !valid_id16(&create.attachment_anchor_id)
        || !valid_timestamp_millis(create.created_at_unix_millis)
    {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn state_code(value: ArchiveInspectionStateV1) -> i16 {
    match value {
        ArchiveInspectionStateV1::Accepted => 1,
        ArchiveInspectionStateV1::AwaitingEvidence => 2,
        ArchiveInspectionStateV1::Inspecting => 3,
        ArchiveInspectionStateV1::Ready => 4,
        ArchiveInspectionStateV1::Rejected => 5,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<ArchiveInspectionStateV1, ArchiveInspectionPersistenceErrorV1> {
    match value {
        1 => Ok(ArchiveInspectionStateV1::Accepted),
        2 => Ok(ArchiveInspectionStateV1::AwaitingEvidence),
        3 => Ok(ArchiveInspectionStateV1::Inspecting),
        4 => Ok(ArchiveInspectionStateV1::Ready),
        5 => Ok(ArchiveInspectionStateV1::Rejected),
        _ => Err(ArchiveInspectionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn error_code(value: ArchiveInspectionErrorV1) -> i16 {
    match value {
        ArchiveInspectionErrorV1::NotSafe => 1,
        ArchiveInspectionErrorV1::NotZip => 2,
        ArchiveInspectionErrorV1::PolicyRejected => 3,
        ArchiveInspectionErrorV1::CorruptArchive => 4,
        ArchiveInspectionErrorV1::Unavailable => 5,
    }
}

pub(crate) fn error_from_code(
    value: i16,
) -> Result<ArchiveInspectionErrorV1, ArchiveInspectionPersistenceErrorV1> {
    match value {
        1 => Ok(ArchiveInspectionErrorV1::NotSafe),
        2 => Ok(ArchiveInspectionErrorV1::NotZip),
        3 => Ok(ArchiveInspectionErrorV1::PolicyRejected),
        4 => Ok(ArchiveInspectionErrorV1::CorruptArchive),
        5 => Ok(ArchiveInspectionErrorV1::Unavailable),
        _ => Err(ArchiveInspectionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn entry_kind_code(value: ArchiveEntryKindV1) -> i16 {
    match value {
        ArchiveEntryKindV1::File => 1,
        ArchiveEntryKindV1::Directory => 2,
    }
}

pub(crate) fn entry_kind_from_code(
    value: i16,
) -> Result<ArchiveEntryKindV1, ArchiveInspectionPersistenceErrorV1> {
    match value {
        1 => Ok(ArchiveEntryKindV1::File),
        2 => Ok(ArchiveEntryKindV1::Directory),
        _ => Err(ArchiveInspectionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn valid_report(report: &ArchiveInspectionReportV1) -> bool {
    report.entry_count == report.entries.len()
        && report.entries.len() <= 1_000
        && report.total_uncompressed_bytes
            == report.entries.iter().fold(0_u64, |total, entry| {
                total.saturating_add(entry.uncompressed_size)
            })
        && report.entries.iter().all(valid_entry)
}

fn valid_entry(entry: &ArchiveEntryInspectionV1) -> bool {
    !entry.normalized_path.is_empty()
        && entry.normalized_path.len() <= 1_024
        && entry.uncompressed_size <= 256 * 1024 * 1024
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
}

pub(crate) fn valid_worker(value: &str) -> bool {
    valid_owner(value)
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp_millis(value: i64) -> bool {
    (-62_135_596_800_000..=253_402_300_799_999).contains(&value)
}

fn digest16(domain: &[u8], parts: &[&[u8]]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for part in parts {
        hasher.update(part);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_deterministic_owner_and_evidence_scoped() {
        let first = archive_inspection_run_id_v1("owner-1", [1; 16]);
        assert_eq!(first, archive_inspection_run_id_v1("owner-1", [1; 16]));
        assert_ne!(first, archive_inspection_run_id_v1("owner-2", [1; 16]));
        let request = ArchiveInspectionRequestV1 {
            run_id: first,
            operation_id: [1; 16],
            attachment_anchor_id: [2; 16],
        };
        assert_ne!(
            archive_inspection_job_id_v1(&request, [3; 16], [4; 16], [5; 16], [6; 16]),
            archive_inspection_job_id_v1(&request, [3; 16], [4; 16], [5; 16], [7; 16])
        );
        assert_ne!(
            archive_inspection_terminal_evidence_id_v1(
                request.run_id,
                ArchiveInspectionErrorV1::NotZip,
            ),
            archive_inspection_terminal_evidence_id_v1(
                request.run_id,
                ArchiveInspectionErrorV1::CorruptArchive,
            )
        );
    }

    #[test]
    fn client_request_fingerprint_excludes_run_identity() {
        assert_eq!(
            archive_inspection_request_fingerprint_v1([2; 16]),
            archive_inspection_request_fingerprint_v1([2; 16])
        );
        assert_ne!(
            archive_inspection_request_fingerprint_v1([2; 16]),
            archive_inspection_request_fingerprint_v1([3; 16])
        );
    }
}
