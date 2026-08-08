use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use super::labels::{session_id_label_value, testkit_labels};

const TEST_POSTGRES_MAX_CONNECTIONS: &str = "300";
pub const SESSION_POSTGRES_HOST_PORT_ENV: &str = "MAKOSH_TEST_POSTGRES_HOST_PORT";

pub struct PostgresContainer {
    _container: Option<ContainerAsync<GenericImage>>,
    host_port: u16,
}

impl PostgresContainer {
    pub async fn start() -> Self {
        if let Some(host_port) = session_host_port() {
            return Self {
                _container: None,
                host_port,
            };
        }

        Self::start_owned().await
    }

    pub async fn start_owned() -> Self {
        Self::start_owned_for_session(&session_id_label_value(), false).await
    }

    pub async fn start_owned_with_session(session_id: &str) -> Self {
        Self::start_owned_for_session(session_id, true).await
    }

    async fn start_owned_for_session(session_id: &str, attach_session_network: bool) -> Self {
        // GenericImage methods (return Self) must come BEFORE ImageExt methods (return ContainerRequest)
        let image = GenericImage::new("pgvector/pgvector", "0.8.2-pg16")
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_exposed_port(5432.tcp());
        let request = if attach_session_network {
            image
                .with_labels(testkit_labels("postgres", session_id))
                .with_network(test_session_network_name(session_id))
                .with_container_name(postgres_container_name(session_id))
        } else {
            image.with_labels(testkit_labels("postgres", session_id))
        };
        let container = request
            .with_env_var("POSTGRES_DB", "testdb")
            .with_env_var("POSTGRES_USER", "testuser")
            .with_env_var("POSTGRES_PASSWORD", "testpass")
            .with_cmd(vec![
                "postgres".to_owned(),
                "-c".to_owned(),
                format!("max_connections={TEST_POSTGRES_MAX_CONNECTIONS}"),
            ])
            .start()
            .await
            .expect("failed to start pgvector container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to resolve pgvector container port");

        Self {
            _container: Some(container),
            host_port,
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://testuser:testpass@127.0.0.1:{}/testdb",
            self.host_port
        )
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

pub(crate) fn postgres_container_name(session_id: &str) -> String {
    format!("{session_id}-postgres")
}

pub(crate) fn test_session_network_name(session_id: &str) -> String {
    format!("{session_id}-network")
}

fn session_host_port() -> Option<u16> {
    let value = std::env::var(SESSION_POSTGRES_HOST_PORT_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.parse::<u16>().unwrap_or_else(|error| {
        panic!("invalid {SESSION_POSTGRES_HOST_PORT_ENV} value '{trimmed}': {error}")
    }))
}
