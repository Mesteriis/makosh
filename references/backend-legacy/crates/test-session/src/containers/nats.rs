use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::time::{Duration, Instant, sleep};

use super::labels::{session_id_label_value, testkit_labels};

const NATS_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const NATS_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);
pub const SESSION_NATS_HOST_PORT_ENV: &str = "MAKOSH_TEST_NATS_HOST_PORT";

pub struct NatsContainer {
    _container: Option<ContainerAsync<GenericImage>>,
    host_port: u16,
}

impl NatsContainer {
    pub async fn start() -> Self {
        if let Some(host_port) = session_host_port() {
            wait_for_nats_port(host_port).await;
            return Self {
                _container: None,
                host_port,
            };
        }

        Self::start_owned().await
    }

    pub async fn start_owned() -> Self {
        Self::start_owned_with_session(&session_id_label_value()).await
    }

    pub async fn start_owned_with_session(session_id: &str) -> Self {
        let container = GenericImage::new("nats", "2.11-alpine")
            .with_exposed_port(4222.tcp())
            .with_cmd(vec!["-js", "-sd", "/data"])
            .with_labels(testkit_labels("nats", session_id))
            .start()
            .await
            .expect("failed to start NATS container");

        let host_port = container
            .get_host_port_ipv4(4222)
            .await
            .expect("failed to resolve NATS container port");

        wait_for_nats_port(host_port).await;

        Self {
            _container: Some(container),
            host_port,
        }
    }

    pub fn server_url(&self) -> String {
        format!("nats://127.0.0.1:{}", self.host_port)
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

fn session_host_port() -> Option<u16> {
    let value = std::env::var(SESSION_NATS_HOST_PORT_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.parse::<u16>().unwrap_or_else(|error| {
        panic!("invalid {SESSION_NATS_HOST_PORT_ENV} value '{trimmed}': {error}")
    }))
}

async fn wait_for_nats_port(host_port: u16) {
    let deadline = Instant::now() + NATS_CONNECT_TIMEOUT;
    let server_url = format!("nats://127.0.0.1:{host_port}");

    loop {
        match async_nats::connect(&server_url).await {
            Ok(client) => {
                client.flush().await.expect("flush NATS test client");
                return;
            }
            Err(_) if Instant::now() < deadline => {
                sleep(NATS_CONNECT_RETRY_DELAY).await;
            }
            Err(error) => panic!("failed to connect to NATS test container: {error}"),
        }
    }
}
