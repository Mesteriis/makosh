use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const CALENDAR_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const CALENDAR_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_calendar_owner.sql");

#[must_use]
pub fn calendar_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: CALENDAR_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "calendar".to_owned(),
        owner_id: "calendar".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: CALENDAR_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "calendar_owner_initial".to_owned(),
            forward_sql_utf8: CALENDAR_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(CALENDAR_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn all_owner_tables_are_force_rls_and_provider_neutral() {
        validate_storage_bundle(&calendar_storage_bundle_v1()).expect("calendar storage bundle");
        let sql = std::str::from_utf8(CALENDAR_SCHEMA_V1).expect("utf8");
        for table in [
            "calendar_events",
            "calendar_participants",
            "calendar_constraints",
            "calendar_reminders",
            "calendar_outcomes",
            "calendar_client_operations",
            "calendar_scheduler_inbox",
            "calendar_outbox",
        ] {
            assert!(
                sql.contains(&format!("CREATE TABLE makosh_data.{table}")),
                "{table}"
            );
            assert!(
                sql.contains(&format!(
                    "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
                )),
                "{table}"
            );
        }
        for forbidden in ["google", "apple", "caldav", "credential", "private_locator"] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
