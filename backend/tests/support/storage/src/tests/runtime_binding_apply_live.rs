//! Live conformance for Storage Runtime's authorized PgBouncer binding apply.

use makosh_storage_protocol::v1::{
    StorageBindingV1, StorageDeploymentProfileV1, StorageEffectiveBudgetsV1,
    StorageRuntimeConfigurationV1, StorageRuntimeTopologyV1,
};
use zeroize::Zeroizing;

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const DATABASES_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE";
const AUTH_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE";
const PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const PGBOUNCER_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const PGBOUNCER_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_applies_the_kernel_staged_binding_to_pgbouncer() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let credential = Zeroizing::new(read_platform_credential());

    crate::admin::apply_staged_pool_configuration(&configuration(), &credential)
        .expect("runtime applies and reloads the staged PgBouncer binding");
}

fn configuration() -> StorageRuntimeConfigurationV1 {
    StorageRuntimeConfigurationV1 {
        topology: Some(topology()),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 1,
        vault_hpke_public_key_x25519: vec![1; 32],
        desired_bindings: vec![binding()],
        pgbouncer_database_config_path: required(DATABASES_FILE_ENV),
        desired_bundles: Vec::new(),
        pgbouncer_auth_file_path: auth_file_path(),
    }
}

fn auth_file_path() -> String {
    required(AUTH_FILE_ENV)
}

fn topology() -> StorageRuntimeTopologyV1 {
    StorageRuntimeTopologyV1 {
        topology_revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".to_owned(),
        database_id: "makosh_storage_authenticated".to_owned(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
        postgres_artifact_sha256: vec![1; 32],
        pgbouncer_artifact_sha256: vec![2; 32],
        postgres_host: "postgres".to_owned(),
        postgres_port: 5_432,
        pgbouncer_host: required(PGBOUNCER_HOST_ENV),
        pgbouncer_port: port(PGBOUNCER_PORT_ENV),
        pgbouncer_postgres_host: "postgres".to_owned(),
        pgbouncer_postgres_port: 5_432,
    }
}

fn binding() -> StorageBindingV1 {
    StorageBindingV1 {
        storage_instance_id: "storage_main".to_owned(),
        storage_generation: 1,
        database_id: "makosh_storage_authenticated".to_owned(),
        owner: "notes".to_owned(),
        registration_id: "registration_live".to_owned(),
        runtime_instance_id: "runtime_live".to_owned(),
        runtime_generation: 1,
        grant_epoch: 1,
        role_epoch: 1,
        runtime_principal: "runtime_live".to_owned(),
        pool_alias: "runtime_registration_live_1".to_owned(),
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: 4,
            statement_timeout_millis: 5_000,
        }),
        credential_lease_revision: 1,
        storage_bundle_revision: 1,
        storage_bundle_digest: vec![1; 32],
    }
}

fn read_platform_credential() -> Vec<u8> {
    let path = required(PASSWORD_FILE_ENV);
    let metadata = std::fs::symlink_metadata(&path).expect("platform credential metadata");
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let mut credential = std::fs::read(path).expect("platform credential file");
    while matches!(credential.last(), Some(b'\n' | b'\r')) {
        credential.pop();
    }
    assert!(!credential.is_empty());
    credential
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn port(name: &str) -> u32 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("storage integration test requires a valid {name}"))
}
