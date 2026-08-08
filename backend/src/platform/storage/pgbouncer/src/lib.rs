//! PgBouncer configuration and fenced administrative lifecycle boundary.

mod configuration;
mod lifecycle;

pub use configuration::{
    PgBouncerAuthEntryV1, PgBouncerAuthFileV1, PgBouncerDatabaseConfigFileV1,
    PgBouncerRuntimeConfigV1, PoolAliasV1, PoolConfigErrorV1,
};
pub use lifecycle::{
    PgBouncerAdminConnectionErrorV1, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    PgBouncerAdminPortV1, PgBouncerPoolFenceAdapterV1, PoolLifecycleCommandV1,
    PoolLifecycleErrorV1, PoolLifecycleOutcomeV1, PoolRevokePlanV1,
    TokioPostgresPgBouncerAdminPortV1, database_is_configured, reload_configuration,
    verify_admin_connection,
};

pub const MINIMUM_VERSION: &str = "1.25.2";
pub const PLATFORM_ADMIN_USERNAME: &str = "makosh_pgbouncer_admin";
