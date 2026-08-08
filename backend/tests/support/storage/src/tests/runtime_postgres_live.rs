//! Live runtime-adapter conformance for PostgreSQL platform-schema bootstrap.

use makosh_storage_postgres::PostgresRuntimeSessionProbeV1;
use makosh_storage_protocol::v1::{
    StorageBindingV1, StorageBundleV1, StorageDeploymentProfileV1, StorageEffectiveBudgetsV1,
    StorageMigrationStepV1, StorageRuntimeTopologyV1,
};
use makosh_storage_protocol::validation::storage_binding_from_message;
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE";
const POSTGRES_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST";
const POSTGRES_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT";

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_bootstraps_the_platform_postgres_schema() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));

    let wrong = Zeroizing::new(vec![b'f'; 64]);
    assert!(crate::admin::verify_platform_postgres(&topology(), &wrong).is_err());

    let credential = Zeroizing::new(read_platform_credential());
    crate::admin::verify_platform_postgres(&topology(), &credential)
        .expect("runtime adapter bootstraps the authenticated PostgreSQL platform schema");
}

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_reconciles_roles_for_the_kernel_staged_binding() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let credential = Zeroizing::new(read_platform_credential());

    crate::admin::reconcile_authorized_roles(
        &topology(),
        &credential,
        &[runtime_credential(binding([1; 32]))],
    )
    .expect("runtime reconciles the bound Storage role before publishing a pool");
}

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_accepts_only_the_vault_delivered_role_credential() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let credential = Zeroizing::new(read_platform_credential());
    let binding = binding([1; 32]);
    let runtime_password = Zeroizing::new(vec![b'b'; 64]);
    let runtime_credential = runtime_credential_with_password(binding.clone(), runtime_password);

    crate::admin::reconcile_authorized_roles(&topology(), &credential, &[runtime_credential])
        .expect("runtime reconciles the Vault-delivered role credential");

    let topology = topology();
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let mut probe = runtime
        .block_on(PostgresRuntimeSessionProbeV1::connect_with_password(
            &topology.postgres_host,
            u16::try_from(topology.postgres_port).expect("valid PostgreSQL port"),
            &topology.database_id,
            &binding.runtime_principal,
            &Zeroizing::new(vec![b'b'; 64]),
        ))
        .expect("authenticate only with the reconciled role credential");
    let principal = runtime
        .block_on(probe.current_principal())
        .expect("read authenticated principal");
    assert_eq!(principal, binding.runtime_principal);
}

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_applies_the_exact_bound_owner_migration_bundle() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let credential = Zeroizing::new(read_platform_credential());
    let bundle = bundle();
    let digest: [u8; 32] = Sha256::digest(bundle.encode_to_vec()).into();
    let binding = binding(digest);

    crate::admin::reconcile_authorized_roles(
        &topology(),
        &credential,
        &[runtime_credential(binding.clone())],
    )
    .expect("runtime reconciles roles before owner migration");
    crate::admin::apply_authorized_migrations(&topology(), &credential, &[binding], &[bundle])
        .expect("runtime applies the exact owner-local migration bundle");
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
        postgres_host: required(POSTGRES_HOST_ENV),
        postgres_port: port(POSTGRES_PORT_ENV),
        pgbouncer_host: "127.0.0.1".to_owned(),
        pgbouncer_port: 6432,
        pgbouncer_postgres_host: required(POSTGRES_HOST_ENV),
        pgbouncer_postgres_port: port(POSTGRES_PORT_ENV),
    }
}

fn binding(storage_bundle_digest: [u8; 32]) -> StorageBindingV1 {
    StorageBindingV1 {
        storage_instance_id: "storage_main".to_owned(),
        storage_generation: 1,
        database_id: "makosh_storage_authenticated".to_owned(),
        owner: "notes".to_owned(),
        registration_id: "registration_runtime_postgres".to_owned(),
        runtime_instance_id: "runtime_postgres".to_owned(),
        runtime_generation: 1,
        grant_epoch: 1,
        role_epoch: 1,
        runtime_principal: "runtime_postgres".to_owned(),
        pool_alias: "runtime_registration_runtime_postgres_1".to_owned(),
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: 4,
            statement_timeout_millis: 5_000,
        }),
        credential_lease_revision: 1,
        storage_bundle_revision: 1,
        storage_bundle_digest: storage_bundle_digest.to_vec(),
    }
}

fn runtime_credential(binding: StorageBindingV1) -> crate::admin::RuntimeRoleCredentialV1 {
    runtime_credential_with_password(binding, Zeroizing::new(vec![b'a'; 64]))
}

fn runtime_credential_with_password(
    binding: StorageBindingV1,
    password: Zeroizing<Vec<u8>>,
) -> crate::admin::RuntimeRoleCredentialV1 {
    let binding = storage_binding_from_message(&binding).expect("valid Storage binding");
    crate::admin::RuntimeRoleCredentialV1::new(binding, password).expect("valid runtime credential")
}

fn bundle() -> StorageBundleV1 {
    let sql = b"CREATE TABLE makosh_data.notes_runtime_bootstrap_probe (probe_id uuid);".to_vec();
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "notes_runtime_bootstrap_bundle".to_owned(),
        owner_id: "notes".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "create_runtime_bootstrap_probe".to_owned(),
            forward_sql_utf8: sql.clone(),
            sha256: Sha256::digest(sql).to_vec(),
        }],
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
