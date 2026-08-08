//! Canonical Communications Storage bundle construction.

use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use makosh_storage_protocol::validation::validate_storage_bundle;
use sha2::{Digest, Sha256};

const INITIAL_SCHEMA: &[u8] = include_bytes!("../../migrations/0001_communications_state.sql");
const SEARCH_PROJECTION_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0002_communications_search_projection.sql");
const SEARCH_JOBS_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0003_communications_search_jobs.sql");
const SEARCH_JOB_BLOB_RANGE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0004_communications_search_job_blob_range.sql");
const SEARCH_JOB_LIFECYCLE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0005_communications_search_job_lifecycle.sql");
const CANONICAL_MESSAGE_BODY_STATE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0006_communications_canonical_message_body_state.sql");
const SEARCH_PROJECTION_TOMBSTONES_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0007_communications_search_projection_tombstones.sql");
const BODY_CUSTODY_TRANSFERS_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0008_communications_body_custody_transfers.sql");
const BODY_CUSTODY_TRANSFER_LIFECYCLE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0009_communications_body_custody_transfer_lifecycle.sql");
const EVIDENCE_AUDIT_LINEAGE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0010_communications_evidence_audit_lineage.sql");
const CANONICAL_READ_V2_INDEXES_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0011_communications_canonical_read_v2_indexes.sql");
const SAVED_SEARCH_PROJECTION_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0012_communications_saved_search_projection.sql");
const SENDER_INSIGHTS_PROJECTION_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0013_communications_sender_insights_projection.sql");
const EVIDENCE_EXPORT_SOURCE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0014_communications_evidence_export_source.sql");
const MESSAGE_SUBJECT_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0015_communications_message_subject.sql");
const BODY_MEDIA_TYPE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0016_communications_body_media_type.sql");

pub const COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 15;
pub const COMMUNICATIONS_BODY_MEDIA_TYPE_STORAGE_BUNDLE_REVISION_V1: u32 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsBodyMediaTypeSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

/// Immutable Communications schema admitted and applied only by Storage Control.
#[must_use]
pub fn communications_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communications_state".to_owned(),
        owner_id: "communications".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "communications_state_initial".to_owned(),
                forward_sql_utf8: INITIAL_SCHEMA.to_vec(),
                sha256: Sha256::digest(INITIAL_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "communications_search_projection".to_owned(),
                forward_sql_utf8: SEARCH_PROJECTION_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_PROJECTION_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 3,
                migration_id: "communications_search_jobs".to_owned(),
                forward_sql_utf8: SEARCH_JOBS_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_JOBS_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 4,
                migration_id: "communications_search_job_blob_range".to_owned(),
                forward_sql_utf8: SEARCH_JOB_BLOB_RANGE_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_JOB_BLOB_RANGE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 5,
                migration_id: "communications_search_job_lifecycle".to_owned(),
                forward_sql_utf8: SEARCH_JOB_LIFECYCLE_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_JOB_LIFECYCLE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 6,
                migration_id: "communications_canonical_message_body_state".to_owned(),
                forward_sql_utf8: CANONICAL_MESSAGE_BODY_STATE_SCHEMA.to_vec(),
                sha256: Sha256::digest(CANONICAL_MESSAGE_BODY_STATE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 7,
                migration_id: "communications_search_projection_tombstones".to_owned(),
                forward_sql_utf8: SEARCH_PROJECTION_TOMBSTONES_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_PROJECTION_TOMBSTONES_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 8,
                migration_id: "communications_body_custody_transfers".to_owned(),
                forward_sql_utf8: BODY_CUSTODY_TRANSFERS_SCHEMA.to_vec(),
                sha256: Sha256::digest(BODY_CUSTODY_TRANSFERS_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 9,
                migration_id: "communications_body_custody_transfer_lifecycle".to_owned(),
                forward_sql_utf8: BODY_CUSTODY_TRANSFER_LIFECYCLE_SCHEMA.to_vec(),
                sha256: Sha256::digest(BODY_CUSTODY_TRANSFER_LIFECYCLE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 10,
                migration_id: "communications_evidence_audit_lineage".to_owned(),
                forward_sql_utf8: EVIDENCE_AUDIT_LINEAGE_SCHEMA.to_vec(),
                sha256: Sha256::digest(EVIDENCE_AUDIT_LINEAGE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 11,
                migration_id: "communications_canonical_read_v2_indexes".to_owned(),
                forward_sql_utf8: CANONICAL_READ_V2_INDEXES_SCHEMA.to_vec(),
                sha256: Sha256::digest(CANONICAL_READ_V2_INDEXES_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 12,
                migration_id: "communications_saved_search_projection".to_owned(),
                forward_sql_utf8: SAVED_SEARCH_PROJECTION_SCHEMA.to_vec(),
                sha256: Sha256::digest(SAVED_SEARCH_PROJECTION_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 13,
                migration_id: "communications_sender_insights_projection".to_owned(),
                forward_sql_utf8: SENDER_INSIGHTS_PROJECTION_SCHEMA.to_vec(),
                sha256: Sha256::digest(SENDER_INSIGHTS_PROJECTION_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 14,
                migration_id: "communications_evidence_export_source".to_owned(),
                forward_sql_utf8: EVIDENCE_EXPORT_SOURCE_SCHEMA.to_vec(),
                sha256: Sha256::digest(EVIDENCE_EXPORT_SOURCE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 15,
                migration_id: "communications_message_subject".to_owned(),
                forward_sql_utf8: MESSAGE_SUBJECT_SCHEMA.to_vec(),
                sha256: Sha256::digest(MESSAGE_SUBJECT_SCHEMA).to_vec(),
            },
        ],
    }
}

pub fn append_communications_body_media_type_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, CommunicationsBodyMediaTypeSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != COMMUNICATIONS_BODY_MEDIA_TYPE_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "communications_state"
        || predecessor.owner_id != "communications"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(CommunicationsBodyMediaTypeSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: COMMUNICATIONS_BODY_MEDIA_TYPE_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "communications_body_media_type".to_owned(),
        forward_sql_utf8: BODY_MEDIA_TYPE_SCHEMA.to_vec(),
        sha256: Sha256::digest(BODY_MEDIA_TYPE_SCHEMA).to_vec(),
    });
    predecessor.revision = COMMUNICATIONS_BODY_MEDIA_TYPE_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| CommunicationsBodyMediaTypeSchemaErrorV1::InvalidSuccessor)
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn storage_bundle_is_structurally_valid_and_owner_scoped() {
        let bundle = communications_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "communications");
        assert_eq!(bundle.revision, COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
    }
}
