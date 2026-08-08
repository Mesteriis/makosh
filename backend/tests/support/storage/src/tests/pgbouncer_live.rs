//! Live transaction-pool conformance for the disposable Compose contour.

use makosh_storage_pgbouncer::{
    PLATFORM_ADMIN_USERNAME, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    PgBouncerAdminPortV1, PoolLifecycleOutcomeV1, TokioPostgresPgBouncerAdminPortV1,
    verify_admin_connection,
};
use makosh_storage_postgres::{
    PostgresAdminConnectorV1, PostgresRuntimeSessionProbeV1, StorageRoleSpecV1,
    ensure_platform_schemas, ensure_storage_roles,
};
use zeroize::Zeroizing;

use super::fixtures::storage_role_binding;

const DATABASE_URL_ENV: &str = "MAKOSH_STORAGE_TEST_DATABASE_URL";
const PGBOUNCER_URL_ENV: &str = "MAKOSH_STORAGE_TEST_PGBOUNCER_URL";
const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const AUTHENTICATED_PASSWORD_FILE_ENV: &str =
    "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const AUTHENTICATED_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const AUTHENTICATED_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";

#[test]
#[ignore = "requires the disposable development PgBouncer contour"]
fn runtime_role_reaches_postgres_only_through_the_transaction_pool() {
    let database_url = required_url(DATABASE_URL_ENV);
    let pgbouncer_url = required_url(PGBOUNCER_URL_ENV);
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async move {
        let connector = PostgresAdminConnectorV1::connect(&database_url)
            .await
            .expect("connect development PostgreSQL");
        ensure_platform_schemas(&connector)
            .await
            .expect("reconcile platform schemas");
        let spec = pool_role_spec();
        ensure_storage_roles(&connector, &spec)
            .await
            .expect("reconcile pooled runtime role");

        let mut pooled = PostgresRuntimeSessionProbeV1::connect(&runtime_url(
            &pgbouncer_url,
            spec.runtime_principal(),
        ))
        .await
        .expect("connect runtime role through PgBouncer");
        assert_eq!(
            pooled
                .current_principal()
                .await
                .expect("query pooled principal"),
            spec.runtime_principal()
        );
    });
}

#[test]
#[ignore = "requires the disposable development PgBouncer contour"]
fn administrative_port_reaches_the_disposable_admin_console() {
    let endpoint = admin_endpoint(&required_url(PGBOUNCER_URL_ENV));
    let credential = PgBouncerAdminCredentialV1::new(
        "makosh_development".to_owned(),
        Zeroizing::new(String::new()),
    )
    .expect("development admin credential shape");
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async move {
        let mut admin = TokioPostgresPgBouncerAdminPortV1::connect(&endpoint, &credential)
            .await
            .expect("connect disposable PgBouncer admin database");
        assert_eq!(
            admin.execute_pool_command("SHOW VERSION").await,
            PoolLifecycleOutcomeV1::Applied
        );
    });
}

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_admin_console_requires_a_file_backed_credential() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let endpoint = PgBouncerAdminEndpointV1::new(
        required_url(AUTHENTICATED_HOST_ENV),
        required_url(AUTHENTICATED_PORT_ENV)
            .parse()
            .expect("authenticated PgBouncer port"),
    )
    .expect("authenticated PgBouncer endpoint");
    let credential = authenticated_credential();
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async move {
        let wrong = PgBouncerAdminCredentialV1::new(
            PLATFORM_ADMIN_USERNAME.to_owned(),
            Zeroizing::new("wrong-password".to_owned()),
        )
        .expect("wrong credential shape");
        assert!(verify_admin_connection(&endpoint, &wrong).await.is_err());
        verify_admin_connection(&endpoint, &credential)
            .await
            .expect("authenticate PgBouncer admin");
    });
}

fn required_url(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn authenticated_credential() -> PgBouncerAdminCredentialV1 {
    let path = required_url(AUTHENTICATED_PASSWORD_FILE_ENV);
    let metadata = std::fs::symlink_metadata(&path).expect("authenticated password metadata");
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let password = std::fs::read_to_string(path).expect("authenticated password file");
    let password = password.trim_end_matches(['\r', '\n']).to_owned();
    PgBouncerAdminCredentialV1::new(PLATFORM_ADMIN_USERNAME.to_owned(), Zeroizing::new(password))
        .expect("authenticated credential shape")
}

fn pool_role_spec() -> StorageRoleSpecV1 {
    let process_id = std::process::id();
    StorageRoleSpecV1::from_binding(
        format!("storage_ddl_pool_{process_id}"),
        storage_role_binding(
            &format!("storage_pool_probe_{process_id}"),
            &format!("storage_runtime_pool_{process_id}"),
        ),
    )
    .expect("valid pooled role specification")
}

fn runtime_url(base_url: &str, runtime_principal: &str) -> String {
    let (_, endpoint) = base_url
        .split_once('@')
        .expect("development PgBouncer URL has an authority separator");
    format!("postgres://{runtime_principal}@{endpoint}")
}

fn admin_endpoint(base_url: &str) -> PgBouncerAdminEndpointV1 {
    let (_, authority_and_database) = base_url
        .split_once('@')
        .expect("development PgBouncer URL has an authority separator");
    let (authority, _) = authority_and_database
        .split_once('/')
        .expect("development PgBouncer URL has a database");
    let (host, port) = authority
        .rsplit_once(':')
        .expect("development PgBouncer URL has a port");
    PgBouncerAdminEndpointV1::new(
        host.to_owned(),
        port.parse().expect("development PgBouncer port"),
    )
    .expect("valid development PgBouncer endpoint")
}
