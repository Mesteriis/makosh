//! Fail-closed bootstrap quarantine for policy-invalid owner bundles.

use std::collections::BTreeSet;

use makosh_storage_migrations::admit_storage_bundle;
use makosh_storage_protocol::v1::{StorageBundleV1, StorageRuntimeConfigurationV1};

type BundleKeyV1 = (String, u64);

pub(super) fn quarantine_invalid_desired_bindings(
    configuration: &StorageRuntimeConfigurationV1,
) -> StorageRuntimeConfigurationV1 {
    let admissible_bundle_keys = configuration
        .desired_bundles
        .iter()
        .filter(|bundle| admit_storage_bundle(bundle).is_ok())
        .map(bundle_key)
        .collect::<BTreeSet<_>>();
    let desired_bindings = configuration
        .desired_bindings
        .iter()
        .filter(|binding| {
            admissible_bundle_keys
                .contains(&(binding.owner.clone(), binding.storage_bundle_revision))
        })
        .cloned()
        .collect::<Vec<_>>();
    let referenced_bundle_keys = desired_bindings
        .iter()
        .map(|binding| (binding.owner.clone(), binding.storage_bundle_revision))
        .collect::<BTreeSet<_>>();
    let desired_bundles = configuration
        .desired_bundles
        .iter()
        .filter(|bundle| referenced_bundle_keys.contains(&bundle_key(bundle)))
        .cloned()
        .collect::<Vec<_>>();
    let quarantined = configuration
        .desired_bindings
        .len()
        .saturating_sub(desired_bindings.len());
    if quarantined > 0 && std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_storage_quarantined_invalid_bindings={quarantined}");
    }
    StorageRuntimeConfigurationV1 {
        desired_bindings,
        desired_bundles,
        ..configuration.clone()
    }
}

fn bundle_key(bundle: &StorageBundleV1) -> BundleKeyV1 {
    (bundle.owner_id.clone(), u64::from(bundle.revision))
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::v1::{
        StorageBindingV1, StorageBundleV1, StorageMigrationStepV1, StorageRuntimeConfigurationV1,
    };
    use sha2::{Digest, Sha256};

    use super::quarantine_invalid_desired_bindings;

    #[test]
    fn invalid_owner_bundle_is_removed_without_affecting_valid_owner() {
        let valid = bundle(
            "notes",
            b"CREATE TABLE makosh_data.notes_entries (entry_id UUID PRIMARY KEY);",
        );
        let invalid = bundle("tasks", b"UPDATE makosh_data.tasks_entries SET state = 1;");
        let configuration = StorageRuntimeConfigurationV1 {
            desired_bindings: vec![binding("notes"), binding("tasks")],
            desired_bundles: vec![valid.clone(), invalid],
            ..Default::default()
        };

        let admitted = quarantine_invalid_desired_bindings(&configuration);

        assert_eq!(admitted.desired_bindings, vec![binding("notes")]);
        assert_eq!(admitted.desired_bundles, vec![valid]);
    }

    fn binding(owner: &str) -> StorageBindingV1 {
        StorageBindingV1 {
            owner: owner.to_owned(),
            storage_bundle_revision: 1,
            ..Default::default()
        }
    }

    fn bundle(owner: &str, sql: &[u8]) -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: format!("{owner}_state"),
            owner_id: owner.to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: format!("{owner}_initial"),
                forward_sql_utf8: sql.to_vec(),
                sha256: Sha256::digest(sql).to_vec(),
            }],
        }
    }
}
