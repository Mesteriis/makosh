use super::common::*;
use makosh_kernel_control_store::{
    PlatformStorageBindingInputV1, PlatformStorageBindingV1, PlatformStorageBundleV1,
    PlatformStorageEndpointV1, PlatformStorageTopology, PlatformStorageTopologyInputV1,
    StorageDeploymentProfileV1,
};
use makosh_storage_protocol::v1::{
    StorageBundleV1, StorageMigrationStepV1, StorageRuntimeConfigurationV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::platform::storage::topology::{
    ManagedStorageConfigurationInputV1, encoded_managed_macos,
};

#[test]
fn control_store_persists_only_monotonic_fenced_storage_topology() {
    let root = unique_target_root("makosh-storage-topology");
    std::fs::create_dir_all(&root).expect("create fixture root");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");

    store
        .record_platform_storage_topology(&topology(1, 1))
        .expect("record first topology");
    let stored = store
        .platform_storage_topology()
        .expect("read topology")
        .expect("stored topology");
    assert_eq!(stored.revision(), 1);
    assert_eq!(stored.storage_generation(), 1);
    assert_eq!(
        stored.deployment_profile(),
        StorageDeploymentProfileV1::MacosTauriEmbedded
    );

    assert!(
        store
            .record_platform_storage_topology(&topology(2, 1))
            .is_err()
    );
    store
        .record_platform_storage_topology(&topology(2, 2))
        .expect("advance topology and generation together");
    assert!(
        store
            .record_platform_storage_topology(&topology(2, 3))
            .is_err()
    );

    std::fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn control_store_rejects_an_untrusted_storage_topology_shape() {
    let root = unique_target_root("makosh-storage-topology-invalid");
    std::fs::create_dir_all(&root).expect("create fixture root");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let invalid = PlatformStorageTopology::new(PlatformStorageTopologyInputV1 {
        revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".to_owned(),
        database_id: "makosh".to_owned(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
        postgres_endpoint: endpoint(5_432),
        pgbouncer_endpoint: endpoint(6_432),
        postgres_artifact_sha256: [0; 32],
        pgbouncer_artifact_sha256: [2; 32],
    });

    assert!(store.record_platform_storage_topology(&invalid).is_err());
    let invalid_endpoint = PlatformStorageTopology::new(PlatformStorageTopologyInputV1 {
        revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".to_owned(),
        database_id: "makosh".to_owned(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
        postgres_endpoint: PlatformStorageEndpointV1::new("not/a-host", 5_432),
        pgbouncer_endpoint: endpoint(6_432),
        postgres_artifact_sha256: [1; 32],
        pgbouncer_artifact_sha256: [2; 32],
    });
    assert!(
        store
            .record_platform_storage_topology(&invalid_endpoint)
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn runtime_configuration_stages_only_durable_bindings_for_the_current_topology() {
    let current = topology(1, 1);
    let bundle = storage_bundle();
    let configuration = encoded_managed_macos(ManagedStorageConfigurationInputV1 {
        topology: &current,
        bindings: &[binding(1, 1, *bundle.digest())],
        bundles: &[bundle],
        pgbouncer_database_config_path: &unique_target_root("makosh-storage-pgbouncer")
            .join("databases.ini"),
        pgbouncer_auth_file_path: &unique_target_root("makosh-storage-pgbouncer").join("users.txt"),
        vault_instance_id: "vault_main",
        vault_runtime_generation: 3,
        vault_hpke_public_key_x25519: &[7; 32],
    })
    .expect("encode current fenced binding");
    let configuration = StorageRuntimeConfigurationV1::decode(configuration.as_slice())
        .expect("decode Storage runtime configuration");
    assert_eq!(configuration.desired_bindings.len(), 1);
    assert_eq!(configuration.desired_bundles.len(), 1);
    assert_eq!(
        configuration.desired_bindings[0].runtime_principal,
        "runtime_notes"
    );
    assert!(
        configuration
            .pgbouncer_database_config_path
            .ends_with("databases.ini")
    );

    let stale = encoded_managed_macos(ManagedStorageConfigurationInputV1 {
        topology: &current,
        bindings: &[binding(1, 2, [7; 32])],
        bundles: &[],
        pgbouncer_database_config_path: &unique_target_root("makosh-storage-pgbouncer-stale")
            .join("databases.ini"),
        pgbouncer_auth_file_path: &unique_target_root("makosh-storage-pgbouncer-stale")
            .join("users.txt"),
        vault_instance_id: "vault_main",
        vault_runtime_generation: 3,
        vault_hpke_public_key_x25519: &[7; 32],
    });
    assert!(matches!(
        stale,
        Err(error) if error == "Storage binding is stale for the current topology"
    ));
}

fn topology(revision: u64, generation: u64) -> PlatformStorageTopology {
    PlatformStorageTopology::new(PlatformStorageTopologyInputV1 {
        revision,
        storage_generation: generation,
        storage_instance_id: "storage_main".to_owned(),
        database_id: "makosh".to_owned(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
        postgres_endpoint: endpoint(5_432),
        pgbouncer_endpoint: endpoint(6_432),
        postgres_artifact_sha256: [1; 32],
        pgbouncer_artifact_sha256: [2; 32],
    })
}

fn endpoint(port: u16) -> PlatformStorageEndpointV1 {
    PlatformStorageEndpointV1::new("127.0.0.1", port)
}

fn binding(
    revision: u64,
    topology_revision: u64,
    storage_bundle_digest: [u8; 32],
) -> PlatformStorageBindingV1 {
    PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: "registration_notes".to_owned(),
        capability_id: "storage.access".to_owned(),
        owner_id: "owner_notes".to_owned(),
        binding_revision: revision,
        topology_revision,
        storage_generation: 1,
        runtime_instance_id: "runtime_notes".to_owned(),
        runtime_generation: 7,
        grant_epoch: 3,
        role_epoch: revision,
        runtime_principal: "runtime_notes".to_owned(),
        connection_budget: 4,
        statement_timeout_millis: 5_000,
        credential_lease_revision: revision,
        storage_bundle_revision: 1,
        storage_bundle_digest,
    })
    .expect("valid durable storage binding")
}

fn storage_bundle() -> PlatformStorageBundleV1 {
    let sql = b"CREATE TABLE makosh_data.owner_notes_probe (probe_id uuid);".to_vec();
    let bundle = StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "owner_notes_bundle".to_owned(),
        owner_id: "owner_notes".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "create_probe".to_owned(),
            forward_sql_utf8: sql.clone(),
            sha256: Sha256::digest(sql).to_vec(),
        }],
    };
    let bytes = bundle.encode_to_vec();
    PlatformStorageBundleV1::new("owner_notes", 1, Sha256::digest(&bytes).into(), bytes)
        .expect("valid canonical Storage bundle")
}
