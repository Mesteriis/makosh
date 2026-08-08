//! Container lifecycle primitives for Макошь integration-test sessions.
//!
//! This crate deliberately has no dependency on application or domain crates.
//! It can start PostgreSQL, PgBouncer and NATS and expose their session-scoped endpoints;
//! This crate also owns the shared schema migration step; domain-specific
//! fixtures remain in `makosh-backend-testkit`.
//!
//! PostgreSQL remains the administrative endpoint for creating isolated test
//! databases. Runtime pools use the session PgBouncer endpoint when available,
//! so high-concurrency tests exercise the same pooling boundary as production.

pub mod containers;

use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::Mutex;
use uuid::Uuid;

use containers::nats::NatsContainer;
use containers::pgbouncer::PgbouncerContainer;
use containers::postgres::PostgresContainer;

static DATABASE_SETUP_LOCK: Mutex<()> = Mutex::const_new(());
const TEST_POOL_MAX_CONNECTIONS: u32 = 4;

#[derive(Clone)]
pub struct TestDatabase {
    pool: sqlx::PgPool,
    connection_string: String,
}

/// Container-backed infrastructure for one isolated integration-test database.
///
/// This type deliberately owns only technical lifecycle: PostgreSQL, PgBouncer,
/// NATS and schema application. Domain fixtures, vault setup and router
/// composition remain outside this crate.
pub struct TestDatabaseSession {
    _postgres: PostgresContainer,
    pgbouncer: Option<PgbouncerContainer>,
    nats_container: Mutex<Option<NatsContainer>>,
    database: TestDatabase,
}

impl TestDatabaseSession {
    pub async fn new() -> Self {
        let postgres = PostgresContainer::start().await;
        let database_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let _setup_guard = DATABASE_SETUP_LOCK.lock().await;
        let pgbouncer = PgbouncerContainer::start_for_current_session().await;
        let pool = create_database(&postgres, pgbouncer.as_ref(), &database_name).await;
        let connection_string = match pgbouncer.as_ref() {
            Some(pgbouncer) => pgbouncer.connection_string(&database_name),
            None => direct_connection_string(postgres.host_port(), &database_name),
        };

        Self {
            _postgres: postgres,
            pgbouncer,
            nats_container: Mutex::new(None),
            database: TestDatabase::new(pool, connection_string),
        }
    }

    pub fn pool(&self) -> &PgPool {
        self.database.pool()
    }

    pub fn connection_string(&self) -> &str {
        self.database.connection_string()
    }

    pub fn database(&self) -> TestDatabase {
        self.database.clone()
    }

    pub async fn nats_server_url(&self) -> String {
        let mut container = self.nats_container.lock().await;
        if container.is_none() {
            *container = Some(NatsContainer::start().await);
        }
        container
            .as_ref()
            .expect("NATS test container should be initialized")
            .server_url()
    }

    pub fn uses_pgbouncer(&self) -> bool {
        self.pgbouncer.is_some()
    }
}

impl TestDatabase {
    pub fn new(pool: sqlx::PgPool, connection_string: impl Into<String>) -> Self {
        Self {
            pool,
            connection_string: connection_string.into(),
        }
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }
    pub fn clone_pool(&self) -> sqlx::PgPool {
        self.pool.clone()
    }
}

/// Apply the repository schema to an isolated test database.
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    makosh_schema::apply(pool).await
}

async fn create_database(
    container: &PostgresContainer,
    pgbouncer: Option<&PgbouncerContainer>,
    database_name: &str,
) -> PgPool {
    let admin_pool = connect_with_retry(&container.connection_string(), "admin database").await;
    let create_sql = format!("CREATE DATABASE \"{}\"", database_name.replace('"', "\"\""));
    sqlx::query(&create_sql)
        .execute(&admin_pool)
        .await
        .unwrap_or_else(|error| panic!("failed to create database '{database_name}': {error}"));
    admin_pool.close().await;

    let database_url = match pgbouncer {
        Some(pgbouncer) => pgbouncer.connection_string(database_name),
        None => direct_connection_string(container.host_port(), database_name),
    };
    let pool = connect_with_retry(&database_url, "new test database").await;
    run_migrations(&pool)
        .await
        .expect("failed to run migrations");
    pool
}

fn direct_connection_string(host_port: u16, database_name: &str) -> String {
    format!("postgres://testuser:testpass@127.0.0.1:{host_port}/{database_name}")
}

async fn connect_with_retry(database_url: &str, label: &str) -> PgPool {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        match PgPoolOptions::new()
            .max_connections(TEST_POOL_MAX_CONNECTIONS)
            .connect(database_url)
            .await
        {
            Ok(pool) => return pool,
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            Err(error) => panic!("failed to connect to {label}: {error}"),
        }
    }
}
