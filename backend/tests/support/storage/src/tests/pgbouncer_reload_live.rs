//! Live PgBouncer reload conformance for the Storage-owned database include.

use makosh_storage_pgbouncer::{
    PLATFORM_ADMIN_USERNAME, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    PgBouncerDatabaseConfigFileV1, PgBouncerRuntimeConfigV1, PoolAliasV1,
    TokioPostgresPgBouncerAdminPortV1, database_is_configured, reload_configuration,
};
use zeroize::Zeroizing;

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const DATABASES_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE";
const PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const PGBOUNCER_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const PGBOUNCER_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_pgbouncer_reloads_the_storage_owned_database_include() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let alias = PoolAliasV1::new("registration_reload", 1).expect("pool alias");
    PgBouncerDatabaseConfigFileV1::replace(&database_file(), &[config(alias.clone())])
        .expect("replace database include");

    let endpoint = endpoint();
    let credential = credential();
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    for attempt in 1..=10 {
        if runtime.block_on(reload_once(&endpoint, &credential, &alias)) {
            return;
        }
        if attempt < 10 {
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    panic!("reload storage database include");
}

async fn reload_once(
    endpoint: &PgBouncerAdminEndpointV1,
    credential: &PgBouncerAdminCredentialV1,
    alias: &PoolAliasV1,
) -> bool {
    let Ok(mut admin) = TokioPostgresPgBouncerAdminPortV1::connect(endpoint, credential).await
    else {
        return false;
    };
    reload_configuration(&mut admin).await.is_ok()
        && database_is_configured(endpoint, credential, alias).await == Ok(true)
}

fn config(alias: PoolAliasV1) -> PgBouncerRuntimeConfigV1 {
    PgBouncerRuntimeConfigV1::new(
        alias,
        "postgres".to_owned(),
        5432,
        "makosh_storage_authenticated".to_owned(),
        "runtime_reload".to_owned(),
        8,
    )
    .expect("pool config")
}

fn database_file() -> std::path::PathBuf {
    required(DATABASES_FILE_ENV).into()
}

fn endpoint() -> PgBouncerAdminEndpointV1 {
    PgBouncerAdminEndpointV1::new(required(PGBOUNCER_HOST_ENV), port(PGBOUNCER_PORT_ENV))
        .expect("PgBouncer endpoint")
}

fn credential() -> PgBouncerAdminCredentialV1 {
    let password = std::fs::read_to_string(required(PASSWORD_FILE_ENV))
        .expect("PgBouncer password file")
        .trim_end_matches(['\r', '\n'])
        .to_owned();
    PgBouncerAdminCredentialV1::new(PLATFORM_ADMIN_USERNAME.to_owned(), Zeroizing::new(password))
        .expect("PgBouncer credential")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn port(name: &str) -> u16 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("storage integration test requires a valid {name}"))
}
use std::time::Duration;
