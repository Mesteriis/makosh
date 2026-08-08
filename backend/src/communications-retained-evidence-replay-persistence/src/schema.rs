use makosh_storage_protocol::{
    v1::{StorageBundleV1, StorageMigrationStepV1},
    validation::validate_storage_bundle,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1: u32 = 17;
pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_retained_evidence_replay.sql");
pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1: u32 = 18;
pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_retained_evidence_replay_delivery.sql");
pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1: u32 = 19;
pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0003_retained_evidence_replay_scan.sql");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsRetainedEvidenceReplaySchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsRetainedEvidenceReplayScanSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

pub fn append_communications_retained_evidence_replay_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, CommunicationsRetainedEvidenceReplaySchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision
            != COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "communications_state"
        || predecessor.owner_id != "communications"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(CommunicationsRetainedEvidenceReplaySchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "communications_retained_evidence_replay".to_owned(),
        forward_sql_utf8: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| CommunicationsRetainedEvidenceReplaySchemaErrorV1::InvalidSuccessor)
}

pub fn append_communications_retained_evidence_replay_delivery_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision
            != COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "communications_state"
        || predecessor.owner_id != "communications"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "communications_retained_evidence_replay_delivery".to_owned(),
        forward_sql_utf8: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1).to_vec(),
    });
    predecessor.revision =
        COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1::InvalidSuccessor)
}

pub fn append_communications_retained_evidence_replay_scan_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, CommunicationsRetainedEvidenceReplayScanSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision
            != COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "communications_state"
        || predecessor.owner_id != "communications"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(CommunicationsRetainedEvidenceReplayScanSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "communications_retained_evidence_replay_scan".to_owned(),
        forward_sql_utf8: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| CommunicationsRetainedEvidenceReplayScanSchemaErrorV1::InvalidSuccessor)
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::v1::StorageMigrationStepV1;

    use super::*;

    fn predecessor(owner_id: &str) -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 16,
            bundle_id: "communications_state".to_owned(),
            owner_id: owner_id.to_owned(),
            steps: (1..=16)
                .map(|revision| {
                    let sql = format!(
                        "CREATE TABLE makosh_data.communications_test_{revision} (id BIGINT);"
                    )
                    .into_bytes();
                    StorageMigrationStepV1 {
                        revision,
                        migration_id: format!("communications_test_{revision}"),
                        sha256: Sha256::digest(&sql).to_vec(),
                        forward_sql_utf8: sql,
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn appends_only_communications_index_and_audit() {
        let bundle = append_communications_retained_evidence_replay_storage_v1(predecessor(
            "communications",
        ))
        .expect("valid successor");
        assert_eq!(bundle.revision, 17);
        let sql = String::from_utf8(bundle.steps[16].forward_sql_utf8.clone()).expect("utf8");
        assert!(sql.contains("communications_retained_evidence_replay_index"));
        assert!(sql.contains("communications_retained_evidence_replay_audit"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("attachment_security_"));
        assert!(!sql.contains("payload"));
        assert!(!sql.contains("subject"));
    }

    #[test]
    fn rejects_wrong_owner_predecessor() {
        assert_eq!(
            append_communications_retained_evidence_replay_storage_v1(predecessor("mail")),
            Err(CommunicationsRetainedEvidenceReplaySchemaErrorV1::InvalidPredecessor)
        );
    }

    #[test]
    fn appends_owner_local_command_inbox_and_result_outbox() {
        let predecessor = append_communications_retained_evidence_replay_storage_v1(predecessor(
            "communications",
        ))
        .expect("replay predecessor");
        let bundle =
            append_communications_retained_evidence_replay_delivery_storage_v1(predecessor)
                .expect("delivery successor");
        assert_eq!(bundle.revision, 18);
        let sql = String::from_utf8(bundle.steps[17].forward_sql_utf8.clone()).expect("utf8");
        assert!(sql.contains("communications_retained_evidence_replay_command_inbox"));
        assert!(sql.contains("communications_retained_evidence_replay_result_outbox"));
        assert!(sql.contains("exact_envelope_bytes"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("UPDATE makosh_data.communications_domain_outbox"));
    }

    #[test]
    fn appends_bounded_owner_local_outbox_scan_ledger() {
        let predecessor = append_communications_retained_evidence_replay_storage_v1(predecessor(
            "communications",
        ))
        .expect("replay predecessor");
        let predecessor =
            append_communications_retained_evidence_replay_delivery_storage_v1(predecessor)
                .expect("delivery predecessor");
        let bundle = append_communications_retained_evidence_replay_scan_storage_v1(predecessor)
            .expect("scan successor");
        assert_eq!(bundle.revision, 19);
        let sql = String::from_utf8(
            bundle
                .steps
                .last()
                .expect("scan step")
                .forward_sql_utf8
                .clone(),
        )
        .expect("utf8");
        assert!(sql.contains("communications_retained_evidence_replay_scan"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("payload"));
        assert!(!sql.contains("subject"));
        assert!(!sql.contains("UPDATE makosh_data.communications_domain_outbox"));
    }
}
