//! Owner-authorized admission of canonical Storage bundles.

use makosh_gateway_protocol::v1::{
    AdmitBundledStorageArtifactRequestV1, AdmitBundledStorageArtifactResponseV1,
    AdmitStorageBundleRequestV1, AdmitStorageBundleResponseV1,
};
use makosh_kernel_control_store::PlatformStorageBundleV1;
use makosh_kernel_control_store_sqlite::{SqliteControlStore, StoreError};
use makosh_storage_protocol::{v1::StorageBundleV1, validation::validate_storage_bundle};
use prost::Message;
use sha2::{Digest, Sha256};

use super::super::{OwnerControlSessions, OwnerResult};

pub(super) fn admit_bundled(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: AdmitBundledStorageArtifactRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let bytes = crate::platform::macos::bundled_release::read_current_installed_storage_artifact(
        &request.artifact_id,
        &request.expected_distribution_id,
        request.expected_distribution_generation,
    )?;
    let bundle = admit_canonical_bundle(store, &bytes)?;
    Ok(OwnerResult::AdmitBundledStorageArtifact(
        AdmitBundledStorageArtifactResponseV1 {
            owner_id: bundle.owner_id().to_owned(),
            storage_bundle_revision: bundle.revision(),
            storage_bundle_digest: bundle.digest().to_vec(),
            distribution_id: request.expected_distribution_id,
            distribution_generation: request.expected_distribution_generation,
            artifact_id: request.artifact_id,
        },
    ))
}

pub(super) fn admit(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: AdmitStorageBundleRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        admit_canonical_bundle(store, &request.canonical_bundle)
    })()
    .map(|bundle| {
        OwnerResult::AdmitStorageBundle(AdmitStorageBundleResponseV1 {
            owner_id: bundle.owner_id().to_owned(),
            storage_bundle_revision: bundle.revision(),
            storage_bundle_digest: bundle.digest().to_vec(),
        })
    })
}

fn admit_canonical_bundle(
    store: &SqliteControlStore,
    bytes: &[u8],
) -> Result<PlatformStorageBundleV1, String> {
    let bundle = canonical_bundle(bytes)?;
    match store.record_platform_storage_bundle(&bundle) {
        Ok(()) => {}
        Err(StoreError::PlatformStorageBundleRevisionConflict) => {
            let predecessor = store
                .platform_storage_bundle(bundle.owner_id(), bundle.revision())
                .map_err(|_| "Storage bundle cannot be inspected".to_owned())?
                .ok_or_else(|| "Storage bundle cannot be inspected".to_owned())?;
            if !is_exact_legacy_namespace_successor(
                predecessor.canonical_bytes(),
                bundle.canonical_bytes(),
            ) {
                return Err("Storage bundle cannot be recorded".to_owned());
            }
            store
                .migrate_legacy_platform_storage_bundle_namespace(&predecessor, &bundle)
                .map_err(|_| "Storage bundle cannot be recorded".to_owned())?;
        }
        Err(_) => return Err("Storage bundle cannot be recorded".to_owned()),
    }
    Ok(bundle)
}

fn is_exact_legacy_namespace_successor(predecessor: &[u8], successor: &[u8]) -> bool {
    let Ok(predecessor) = StorageBundleV1::decode(predecessor) else {
        return false;
    };
    let Ok(successor) = StorageBundleV1::decode(successor) else {
        return false;
    };
    if validate_storage_bundle(&predecessor).is_err()
        || validate_storage_bundle(&successor).is_err()
        || predecessor.major != successor.major
        || predecessor.revision != successor.revision
        || predecessor.bundle_id != successor.bundle_id
        || predecessor.owner_id != successor.owner_id
        || predecessor.steps.len() != successor.steps.len()
    {
        return false;
    }

    let mut changed = false;
    predecessor
        .steps
        .iter()
        .zip(&successor.steps)
        .all(|(predecessor, successor)| {
            if predecessor.revision != successor.revision
                || predecessor.migration_id != successor.migration_id
            {
                return false;
            }
            let normalized = replace_exact_token(
                &predecessor.forward_sql_utf8,
                b"hermes_data",
                b"makosh_data",
            );
            changed |= normalized != predecessor.forward_sql_utf8;
            normalized == successor.forward_sql_utf8
        })
        && changed
}

fn replace_exact_token(bytes: &[u8], predecessor: &[u8], successor: &[u8]) -> Vec<u8> {
    debug_assert_eq!(predecessor.len(), successor.len());
    if predecessor.is_empty() || bytes.len() < predecessor.len() {
        return bytes.to_vec();
    }
    let mut normalized = bytes.to_vec();
    for offset in 0..=bytes.len() - predecessor.len() {
        if &bytes[offset..offset + predecessor.len()] == predecessor {
            normalized[offset..offset + successor.len()].copy_from_slice(successor);
        }
    }
    normalized
}

fn canonical_bundle(bytes: &[u8]) -> Result<PlatformStorageBundleV1, String> {
    if bytes.is_empty() || bytes.len() > 4 * 1024 * 1024 {
        return Err("Storage bundle is invalid".to_owned());
    }
    let bundle =
        StorageBundleV1::decode(bytes).map_err(|_| "Storage bundle is invalid".to_owned())?;
    validate_storage_bundle(&bundle).map_err(|_| "Storage bundle is invalid".to_owned())?;
    (bundle.encode_to_vec() == bytes)
        .then_some(())
        .ok_or_else(|| "Storage bundle is not canonical".to_owned())?;
    PlatformStorageBundleV1::new(
        bundle.owner_id,
        u64::from(bundle.revision),
        Sha256::digest(bytes).into(),
        bytes.to_vec(),
    )
    .map_err(|_| "Storage bundle is invalid".to_owned())
}

#[cfg(test)]
mod tests {
    use super::is_exact_legacy_namespace_successor;
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use prost::Message;
    use sha2::{Digest, Sha256};

    fn bundle(namespace: &str) -> Vec<u8> {
        let sql = format!("CREATE TABLE {namespace}.events (id bigint PRIMARY KEY);").into_bytes();
        StorageBundleV1 {
            major: 1,
            revision: 7,
            bundle_id: "communications_state".to_owned(),
            owner_id: "communications".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "initial".to_owned(),
                sha256: Sha256::digest(&sql).to_vec(),
                forward_sql_utf8: sql,
            }],
        }
        .encode_to_vec()
    }

    #[test]
    fn accepts_only_the_exact_legacy_storage_namespace_successor() {
        assert!(is_exact_legacy_namespace_successor(
            &bundle("hermes_data"),
            &bundle("makosh_data"),
        ));
        assert!(!is_exact_legacy_namespace_successor(
            &bundle("makosh_data"),
            &bundle("makosh_data"),
        ));

        let mut unrelated = StorageBundleV1::decode(bundle("makosh_data").as_slice()).unwrap();
        unrelated.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b" DROP TABLE users;");
        unrelated.steps[0].sha256 = Sha256::digest(&unrelated.steps[0].forward_sql_utf8).to_vec();
        assert!(!is_exact_legacy_namespace_successor(
            &bundle("hermes_data"),
            &unrelated.encode_to_vec(),
        ));
    }
}
