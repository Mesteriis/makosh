//! End-to-end pool and PostgreSQL fencing on the authenticated Docker contour.

use std::future::Future;

use makosh_storage_control::{
    StorageFenceOutcomeV1, StorageLifecycleStateV1, StorageLifecycleV1, StorageRevokerV1,
    StorageVaultLeasePortV1,
};
use makosh_storage_pgbouncer::{
    PLATFORM_ADMIN_USERNAME, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    PgBouncerPoolFenceAdapterV1, TokioPostgresPgBouncerAdminPortV1,
};
use makosh_storage_postgres::{
    PLATFORM_ADMIN_USERNAME as POSTGRES_ADMIN_USERNAME, PostgresAdminConnectorV1,
    PostgresRuntimeFenceAdapterV1, StorageRoleSpecV1, ensure_platform_schemas,
    ensure_storage_roles, read_storage_role_audit,
};
use makosh_storage_protocol::StorageBindingV1;
use makosh_storage_protocol::v1::{
    StorageBindingV1 as StorageBindingMessageV1, StorageDeploymentProfileV1,
    StorageEffectiveBudgetsV1, StorageRuntimeConfigurationV1, StorageRuntimeTopologyV1,
};
use zeroize::Zeroizing;

use crate::admin::apply_staged_pool_configuration;

use super::super::fixtures::storage_role_binding_in_database;

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const DATABASES_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE";
const AUTH_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE";
const PGBOUNCER_PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const PGBOUNCER_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const PGBOUNCER_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";
const POSTGRES_PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE";
const POSTGRES_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST";
const POSTGRES_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT";

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_revoke_fences_the_real_pool_and_postgres_role() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let binding = binding();
    let spec = StorageRoleSpecV1::from_binding(ddl_owner(), binding.clone())
        .expect("valid Storage role specification");
    let postgres_password = Zeroizing::new(read_credential(POSTGRES_PASSWORD_FILE_ENV));
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let connector = runtime.block_on(connect_and_prepare(&postgres_password, &spec));
    let pgbouncer_password = Zeroizing::new(read_credential(PGBOUNCER_PASSWORD_FILE_ENV));
    apply_staged_pool_configuration(
        &configuration(binding_message(&binding)),
        &pgbouncer_password,
    )
    .expect("publish pool before revocation");
    let mut lifecycle = StorageLifecycleV1::default();
    lifecycle.activate(binding).expect("activate binding");
    let mut vault = AppliedVault;
    let report = runtime.block_on(revoke(
        &mut lifecycle,
        &mut vault,
        &connector,
        &spec,
        &pgbouncer_password,
    ));

    assert!(report.is_complete());
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Idle);
    let audit = runtime
        .block_on(read_storage_role_audit(&connector, &spec))
        .expect("audit fenced PostgreSQL role");
    assert!(!audit.can_login);
}

fn binding() -> StorageBindingV1 {
    let suffix = std::process::id();
    storage_role_binding_in_database(
        "makosh_storage_authenticated",
        &format!("storage_revoke_live_{suffix}"),
        &format!("storage_runtime_revoke_live_{suffix}"),
    )
}

fn ddl_owner() -> String {
    format!("storage_ddl_revoke_live_{}", std::process::id())
}

async fn connect_and_prepare(
    password: &Zeroizing<Vec<u8>>,
    spec: &StorageRoleSpecV1,
) -> PostgresAdminConnectorV1 {
    let connector = PostgresAdminConnectorV1::connect_with_password(
        &required(POSTGRES_HOST_ENV),
        port(POSTGRES_PORT_ENV),
        "makosh_storage_authenticated",
        POSTGRES_ADMIN_USERNAME,
        password,
    )
    .await
    .expect("connect authenticated PostgreSQL");
    ensure_platform_schemas(&connector)
        .await
        .expect("reconcile platform schemas");
    ensure_storage_roles(&connector, spec)
        .await
        .expect("reconcile fenced runtime role");
    connector
}

async fn revoke(
    lifecycle: &mut StorageLifecycleV1,
    vault: &mut AppliedVault,
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    pgbouncer_password: &Zeroizing<Vec<u8>>,
) -> makosh_storage_control::StorageRevocationReportV1 {
    let credential = pgbouncer_credential(pgbouncer_password);
    let admin = TokioPostgresPgBouncerAdminPortV1::connect(&pgbouncer_endpoint(), &credential)
        .await
        .expect("connect authenticated PgBouncer admin");
    let mut pool = PgBouncerPoolFenceAdapterV1::new(admin);
    let mut postgres = PostgresRuntimeFenceAdapterV1::new(connector, spec);
    StorageRevokerV1
        .revoke(lifecycle, vault, &mut pool, &mut postgres)
        .await
        .expect("fence every Storage boundary")
}

fn configuration(binding: StorageBindingMessageV1) -> StorageRuntimeConfigurationV1 {
    StorageRuntimeConfigurationV1 {
        topology: Some(StorageRuntimeTopologyV1 {
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
            pgbouncer_port: u32::from(port(PGBOUNCER_PORT_ENV)),
            pgbouncer_postgres_host: "postgres".to_owned(),
            pgbouncer_postgres_port: 5_432,
        }),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 1,
        vault_hpke_public_key_x25519: vec![1; 32],
        desired_bindings: vec![binding],
        pgbouncer_database_config_path: required(DATABASES_FILE_ENV),
        desired_bundles: Vec::new(),
        pgbouncer_auth_file_path: auth_file_path(),
    }
}

fn auth_file_path() -> String {
    required(AUTH_FILE_ENV)
}

fn binding_message(binding: &StorageBindingV1) -> StorageBindingMessageV1 {
    let identity = binding.identity();
    let fences = binding.fences();
    let access = binding.access();
    StorageBindingMessageV1 {
        storage_instance_id: identity.storage_instance_id().to_owned(),
        storage_generation: fences.storage_generation(),
        database_id: identity.database_id().to_owned(),
        owner: identity.owner().to_owned(),
        registration_id: identity.registration_id().to_owned(),
        runtime_instance_id: identity.runtime_instance_id().to_owned(),
        runtime_generation: fences.runtime_generation(),
        grant_epoch: fences.grant_epoch(),
        role_epoch: fences.role_epoch(),
        runtime_principal: access.runtime_principal().to_owned(),
        pool_alias: access.pool_alias().to_owned(),
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: u32::from(access.effective_budgets().max_connections()),
            statement_timeout_millis: access.effective_budgets().statement_timeout_millis(),
        }),
        credential_lease_revision: fences.credential_lease_revision(),
        storage_bundle_revision: fences.storage_bundle_revision(),
        storage_bundle_digest: access.storage_bundle_digest().to_vec(),
    }
}

fn pgbouncer_endpoint() -> PgBouncerAdminEndpointV1 {
    PgBouncerAdminEndpointV1::new(required(PGBOUNCER_HOST_ENV), port(PGBOUNCER_PORT_ENV))
        .expect("valid authenticated PgBouncer endpoint")
}

fn pgbouncer_credential(password: &Zeroizing<Vec<u8>>) -> PgBouncerAdminCredentialV1 {
    let password = String::from_utf8(password.to_vec()).expect("UTF-8 PgBouncer password");
    PgBouncerAdminCredentialV1::new(PLATFORM_ADMIN_USERNAME.to_owned(), Zeroizing::new(password))
        .expect("valid authenticated PgBouncer credential")
}

fn read_credential(name: &str) -> Vec<u8> {
    let path = required(name);
    let metadata = std::fs::symlink_metadata(&path).expect("credential metadata");
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let mut credential = std::fs::read(path).expect("credential bytes");
    while matches!(credential.last(), Some(b'\n' | b'\r')) {
        credential.pop();
    }
    assert!(!credential.is_empty());
    credential
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn port(name: &str) -> u16 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("storage integration test requires a valid {name}"))
}

struct AppliedVault;

impl StorageVaultLeasePortV1 for AppliedVault {
    #[allow(clippy::manual_async_fn)] // The lease port requires a Send future.
    fn invalidate_lease(
        &mut self,
        _: &StorageBindingV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send {
        async { StorageFenceOutcomeV1::Applied }
    }
}
