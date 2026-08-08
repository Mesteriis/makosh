use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant, sleep};

use super::labels::testkit_labels;
use super::postgres::{postgres_container_name, test_session_network_name};

const PGBOUNCER_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const PGBOUNCER_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);
const PGBOUNCER_MAX_CLIENT_CONNECTIONS: &str = "2000";
const PGBOUNCER_DEFAULT_POOL_SIZE: &str = "2";
const PGBOUNCER_MAX_PREPARED_STATEMENTS: &str = "100";
pub const SESSION_PGBOUNCER_HOST_PORT_ENV: &str = "MAKOSH_TEST_PGBOUNCER_HOST_PORT";

/// Session-scoped PgBouncer endpoint for high-concurrency integration tests.
///
/// Client pools connect here; database creation remains on the direct PostgreSQL
/// endpoint because PgBouncer does not support `CREATE DATABASE` in transaction
/// pooling mode.
pub struct PgbouncerContainer {
    _container: Option<ContainerAsync<GenericImage>>,
    host_port: u16,
}

impl PgbouncerContainer {
    pub async fn start_for_current_session() -> Option<Self> {
        let host_port = session_host_port()?;
        wait_for_pgbouncer_port(host_port).await;
        Some(Self {
            _container: None,
            host_port,
        })
    }

    pub async fn start_owned_with_session(session_id: &str) -> Self {
        let container = GenericImage::new("edoburu/pgbouncer", "v1.25.2-p0")
            .with_wait_for(WaitFor::seconds(1))
            .with_exposed_port(5432.tcp())
            .with_labels(testkit_labels("pgbouncer", session_id))
            .with_network(test_session_network_name(session_id))
            .with_container_name(format!("{session_id}-pgbouncer"))
            .with_env_var("DB_HOST", postgres_container_name(session_id))
            .with_env_var("DB_PORT", "5432")
            .with_env_var("DB_USER", "testuser")
            .with_env_var("DB_PASSWORD", "testpass")
            .with_env_var("AUTH_TYPE", "plain")
            .with_env_var("POOL_MODE", "transaction")
            .with_env_var("MAX_CLIENT_CONN", PGBOUNCER_MAX_CLIENT_CONNECTIONS)
            .with_env_var("DEFAULT_POOL_SIZE", PGBOUNCER_DEFAULT_POOL_SIZE)
            .with_env_var("MAX_PREPARED_STATEMENTS", PGBOUNCER_MAX_PREPARED_STATEMENTS)
            .start()
            .await
            .expect("failed to start PgBouncer container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to resolve PgBouncer container port");
        wait_for_pgbouncer_port(host_port).await;

        Self {
            _container: Some(container),
            host_port,
        }
    }

    pub fn connection_string(&self, database_name: &str) -> String {
        format!(
            "postgres://testuser:testpass@127.0.0.1:{}/{database_name}",
            self.host_port
        )
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

fn session_host_port() -> Option<u16> {
    let value = std::env::var(SESSION_PGBOUNCER_HOST_PORT_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.parse::<u16>().unwrap_or_else(|error| {
        panic!("invalid {SESSION_PGBOUNCER_HOST_PORT_ENV} value '{trimmed}': {error}")
    }))
}

async fn wait_for_pgbouncer_port(host_port: u16) {
    let deadline = Instant::now() + PGBOUNCER_CONNECT_TIMEOUT;
    loop {
        match TcpStream::connect(("127.0.0.1", host_port)).await {
            Ok(stream) => {
                drop(stream);
                return;
            }
            Err(_) if Instant::now() < deadline => sleep(PGBOUNCER_CONNECT_RETRY_DELAY).await,
            Err(error) => panic!("failed to connect to PgBouncer test container: {error}"),
        }
    }
}
