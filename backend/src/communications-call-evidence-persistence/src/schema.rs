use makosh_storage_protocol::{
    v1::{StorageBundleV1, StorageMigrationStepV1},
    validation::validate_storage_bundle,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1: u32 = 16;
pub const COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_call_evidence.sql");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsCallEvidenceSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

pub fn append_communications_call_evidence_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, CommunicationsCallEvidenceSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "communications_state"
        || predecessor.owner_id != "communications"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(CommunicationsCallEvidenceSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "communications_call_evidence_initial".to_owned(),
        forward_sql_utf8: COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| CommunicationsCallEvidenceSchemaErrorV1::InvalidSuccessor)
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::v1::StorageMigrationStepV1;

    use super::*;

    #[test]
    fn bundle_is_additive_owner_local_and_private_content_negative() {
        let predecessor = StorageBundleV1 {
            major: 1,
            revision: 15,
            bundle_id: "communications_state".to_owned(),
            owner_id: "communications".to_owned(),
            steps: (1..=15)
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
        };
        let bundle =
            append_communications_call_evidence_storage_v1(predecessor).expect("successor");
        assert_eq!(bundle.owner_id, "communications");
        assert_eq!(bundle.revision, 16);
        assert_eq!(bundle.steps.len(), 16);
        let sql = String::from_utf8(bundle.steps[15].forward_sql_utf8.clone()).expect("utf8");
        for table in [
            "communications_call_evidence_inbox",
            "communications_call_evidence_projection",
            "communications_call_evidence_history",
            "communications_call_evidence_realtime_sequence",
            "communications_call_evidence_realtime_frames",
        ] {
            assert!(sql.contains(table));
        }
        for forbidden in [
            "provider_call_id",
            "provider_account_id",
            "chat_id",
            "phone_number",
            "username",
            "encryption_key",
            "signaling",
            "pcm",
            "audio_bytes",
            "transcript_text",
            "raw_json",
        ] {
            assert!(!sql.contains(forbidden));
        }
    }

    #[test]
    fn bundle_rejects_a_parallel_or_wrong_owner_predecessor() {
        let wrong = StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "communications_call_evidence_state".to_owned(),
            owner_id: "communications".to_owned(),
            steps: Vec::new(),
        };
        assert_eq!(
            append_communications_call_evidence_storage_v1(wrong),
            Err(CommunicationsCallEvidenceSchemaErrorV1::InvalidPredecessor)
        );
    }
}
