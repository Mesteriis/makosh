use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V4: u32 = 4;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5: u32 = 5;
pub const COMMUNICATION_DELIVERY_INTENT_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_delivery_intent_state.sql");
pub const COMMUNICATION_DELIVERY_INTENT_PROVIDER_EVENTS_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_provider_event_delivery.sql");
pub const COMMUNICATION_DELIVERY_INTENT_CLIENT_REALTIME_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_client_realtime_replay.sql");
pub const COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_SCHEMA_V4: &[u8] =
    include_bytes!("../migrations/0004_event_ingress.sql");
pub const COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CLEANUP_SCHEMA_V5: &[u8] =
    include_bytes!("../migrations/0005_event_ingress_cleanup.sql");

#[must_use]
pub fn communication_delivery_intent_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5,
        bundle_id: "communication_delivery_intent_state".to_owned(),
        owner_id: "communication_delivery_intent".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "communication_delivery_intent_state_initial".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELIVERY_INTENT_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELIVERY_INTENT_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "communication_delivery_intent_provider_events".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELIVERY_INTENT_PROVIDER_EVENTS_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELIVERY_INTENT_PROVIDER_EVENTS_SCHEMA_V2)
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "communication_delivery_intent_client_realtime_replay".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELIVERY_INTENT_CLIENT_REALTIME_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELIVERY_INTENT_CLIENT_REALTIME_SCHEMA_V3)
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "communication_delivery_intent_event_ingress".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_SCHEMA_V4.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_SCHEMA_V4)
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5,
                migration_id: "communication_delivery_intent_event_ingress_cleanup".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CLEANUP_SCHEMA_V5
                    .to_vec(),
                sha256: Sha256::digest(
                    COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CLEANUP_SCHEMA_V5,
                )
                .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_owner_scoped_and_blob_receipt_only() {
        let bundle = communication_delivery_intent_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid owner storage bundle");
        assert_eq!(bundle.owner_id, "communication_delivery_intent");
        assert_eq!(
            bundle.revision,
            COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5
        );
        assert_eq!(bundle.steps.len(), 5);
        let sql = bundle
            .steps
            .iter()
            .map(|step| std::str::from_utf8(&step.forward_sql_utf8).expect("utf8"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(sql.contains("makosh_data.communication_delivery_intent_jobs"));
        assert!(sql.contains("body_reference_id"));
        assert!(sql.contains("body_custody_source_proof"));
        assert!(sql.contains("claim_epoch"));
        assert!(sql.contains("PRIMARY KEY (logical_owner_id, intent_id)"));
        assert!(sql.contains("communication_delivery_intent_provider_outbox"));
        assert!(sql.contains("communication_delivery_intent_result_inbox"));
        assert!(sql.contains("exact_envelope_bytes"));
        assert!(sql.contains("realtime_sequence"));
        assert!(sql.contains("communication_delivery_intent_ingress_inbox"));
        assert!(sql.contains("communication_delivery_intent_ingress_result_outbox"));
        assert!(sql.contains("communication_delivery_intent_ingress_cleanup"));
        assert!(!sql.contains("body_utf8"));
        assert!(!sql.contains("body_ciphertext"));
        assert!(!sql.contains("body_nonce"));
        assert!(!sql.contains("body_key_epoch"));
        assert!(!sql.contains("communications_messages"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("telegram_"));
    }
}
