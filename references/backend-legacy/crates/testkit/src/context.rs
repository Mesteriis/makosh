use crate::vault::{TestVault, new_test_vault};
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use std::path::Path;

/// Isolated test environment with a fresh migrated database.
///
/// Each `TestContext` creates its own unique database on the PostgreSQL
/// container owned by the current test session. `make backend-test` starts one
/// pgvector container, PgBouncer and one NATS container before nextest, then
/// removes them after the session.
///
/// # Usage
///
/// ```ignore
/// #[tokio::test]
/// async fn my_test() {
///     let ctx = TestContext::new().await;
///     let pool = ctx.pool();
///     // ... use pool ...
///     // The pool is dropped with the context.
/// }
/// ```
pub struct TestContext {
    session: makosh_test_session::TestDatabaseSession,
    vault: TestVault,
}

impl TestContext {
    /// Create a new isolated test environment.
    ///
    /// 1. Reuses the pgvector container for this test session
    /// 2. Creates a unique database (uuid-based name)
    /// 3. Reuses the session PgBouncer endpoint when a session is active
    /// 4. Runs migrations through that endpoint (or directly for standalone tests)
    /// 5. Returns a ready-to-use pool
    pub async fn new() -> Self {
        let session = makosh_test_session::TestDatabaseSession::new().await;
        let vault = new_test_vault();
        makosh_hub_backend::platform::settings::store::ApplicationSettingsStore::new(
            session.pool().clone(),
        )
        .repair_declared_settings()
        .await
        .expect("failed to repair application settings");

        Self { session, vault }
    }

    /// The connection pool to the isolated test database.
    pub fn pool(&self) -> &sqlx::PgPool {
        self.session.pool()
    }

    /// The full connection string for this test database.
    pub fn connection_string(&self) -> String {
        self.session.connection_string().to_owned()
    }

    pub fn vault_home(&self) -> &Path {
        self.vault.vault_home()
    }

    pub fn dev_key_path(&self) -> &Path {
        self.vault.dev_key_path()
    }

    pub fn vault_database_path(&self) -> std::path::PathBuf {
        self.vault.vault_database_path()
    }

    pub fn app_config(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault
            .apply_to_config(AppConfig::test_with_api_secret_and_database_url(
                api_secret,
                self.connection_string(),
            ))
    }

    pub async fn nats_server_url(&self) -> String {
        self.session.nats_server_url().await
    }

    pub async fn app_config_with_nats(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault.apply_to_config(
            AppConfig::test_with_api_secret_and_database_url(api_secret, self.connection_string())
                .with_test_pairs([("MAKOSH_NATS_SERVER_URL", self.nats_server_url().await)])
                .expect("test NATS config must be valid"),
        )
    }

    pub fn app_config_without_database(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault
            .apply_to_config(AppConfig::test_with_api_secret(api_secret))
    }

    /// Database runtime wrapper backed by this context's migrated pool.
    pub fn database(&self) -> Database {
        Database::from_test_pool(
            self.session.database().clone_pool(),
            self.connection_string(),
        )
    }
}
