use makosh_storage_protocol::{
    v1::{StorageBundleV1, StorageMigrationStepV1},
    validation::validate_storage_bundle,
};
use sha2::{Digest, Sha256};

pub const MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1: u32 = 23;
pub const MAIL_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_retained_evidence_replay.sql");
pub const MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1: u32 = 24;
pub const MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_retained_evidence_replay_delivery.sql");
pub const MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1: u32 = 25;
pub const MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0003_retained_evidence_replay_scan.sql");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailRetainedEvidenceReplaySchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailRetainedEvidenceReplayDeliverySchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailRetainedEvidenceReplayScanSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

pub fn append_mail_retained_evidence_replay_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, MailRetainedEvidenceReplaySchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "mail_state"
        || predecessor.owner_id != "mail"
        || predecessor.steps.last().map(|step| step.revision) != Some(20)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(MailRetainedEvidenceReplaySchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "mail_retained_evidence_replay".to_owned(),
        forward_sql_utf8: MAIL_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(MAIL_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| MailRetainedEvidenceReplaySchemaErrorV1::InvalidSuccessor)
}

pub fn append_mail_retained_evidence_replay_delivery_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, MailRetainedEvidenceReplayDeliverySchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision
            != MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "mail_state"
        || predecessor.owner_id != "mail"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(MailRetainedEvidenceReplayDeliverySchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "mail_retained_evidence_replay_delivery".to_owned(),
        forward_sql_utf8: MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| MailRetainedEvidenceReplayDeliverySchemaErrorV1::InvalidSuccessor)
}

pub fn append_mail_retained_evidence_replay_scan_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, MailRetainedEvidenceReplayScanSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "mail_state"
        || predecessor.owner_id != "mail"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(MailRetainedEvidenceReplayScanSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "mail_retained_evidence_replay_scan".to_owned(),
        forward_sql_utf8: MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = MAIL_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| MailRetainedEvidenceReplayScanSchemaErrorV1::InvalidSuccessor)
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::v1::StorageMigrationStepV1;

    use super::*;

    fn predecessor(owner_id: &str) -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 22,
            bundle_id: "mail_state".to_owned(),
            owner_id: owner_id.to_owned(),
            steps: (1..=20)
                .map(|revision| {
                    let sql = format!("CREATE TABLE makosh_data.mail_test_{revision} (id BIGINT);")
                        .into_bytes();
                    StorageMigrationStepV1 {
                        revision,
                        migration_id: format!("mail_test_{revision}"),
                        sha256: Sha256::digest(&sql).to_vec(),
                        forward_sql_utf8: sql,
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn appends_only_mail_index_and_audit() {
        let bundle = append_mail_retained_evidence_replay_storage_v1(predecessor("mail"))
            .expect("valid successor");
        assert_eq!(bundle.revision, 23);
        let sql = String::from_utf8(bundle.steps[20].forward_sql_utf8.clone()).expect("utf8");
        assert!(sql.contains("mail_retained_evidence_replay_index"));
        assert!(sql.contains("mail_retained_evidence_replay_audit"));
        assert!(!sql.contains("communications_"));
        assert!(!sql.contains("payload"));
        assert!(!sql.contains("subject"));
    }

    #[test]
    fn rejects_wrong_owner_predecessor() {
        assert_eq!(
            append_mail_retained_evidence_replay_storage_v1(predecessor("communications")),
            Err(MailRetainedEvidenceReplaySchemaErrorV1::InvalidPredecessor)
        );
    }

    #[test]
    fn appends_owner_local_command_inbox_and_result_outbox() {
        let predecessor = append_mail_retained_evidence_replay_storage_v1(predecessor("mail"))
            .expect("replay predecessor");
        let bundle = append_mail_retained_evidence_replay_delivery_storage_v1(predecessor)
            .expect("delivery successor");
        assert_eq!(bundle.revision, 24);
        let sql = String::from_utf8(
            bundle
                .steps
                .last()
                .expect("delivery step")
                .forward_sql_utf8
                .clone(),
        )
        .expect("utf8");
        assert!(sql.contains("mail_retained_evidence_replay_command_inbox"));
        assert!(sql.contains("mail_retained_evidence_replay_result_outbox"));
        assert!(sql.contains("exact_envelope_bytes"));
        assert!(!sql.contains("communications_"));
        assert!(!sql.contains("UPDATE makosh_data.mail_attachment_security_outbox"));
    }

    #[test]
    fn appends_bounded_owner_local_outbox_scan_ledger() {
        let predecessor = append_mail_retained_evidence_replay_storage_v1(predecessor("mail"))
            .expect("replay predecessor");
        let predecessor = append_mail_retained_evidence_replay_delivery_storage_v1(predecessor)
            .expect("delivery predecessor");
        let bundle = append_mail_retained_evidence_replay_scan_storage_v1(predecessor)
            .expect("scan successor");
        assert_eq!(bundle.revision, 25);
        let sql = String::from_utf8(
            bundle
                .steps
                .last()
                .expect("scan step")
                .forward_sql_utf8
                .clone(),
        )
        .expect("utf8");
        assert!(sql.contains("mail_retained_evidence_replay_scan"));
        assert!(!sql.contains("communications_"));
        assert!(!sql.contains("payload"));
        assert!(!sql.contains("subject"));
        assert!(!sql.contains("UPDATE makosh_data.mail_attachment_security_outbox"));
    }
}
