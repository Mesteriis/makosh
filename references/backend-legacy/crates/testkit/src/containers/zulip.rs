use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use testcontainers::compose::{AutoComposeOptions, ContainerisedComposeOptions, DockerCompose};
use testcontainers::core::{ExecCommand, IntoContainerPort};
use tokio::time::{Instant, MissedTickBehavior, sleep};
use uuid::Uuid;

use makosh_test_session::containers::labels::{SESSION_ID_ENV, session_id_label_value};

const ZULIP_ENABLE_ENV: &str = "MAKOSH_ZULIP_TESTCONTAINERS";
const ZULIP_START_TIMEOUT_ENV: &str = "MAKOSH_ZULIP_START_TIMEOUT_SECS";
const DEFAULT_START_TIMEOUT_SECS: u64 = 600;
const READINESS_POLL_INTERVAL: Duration = Duration::from_secs(2);
const READINESS_LOG_INTERVAL: Duration = Duration::from_secs(15);
const PROVISION_PREFIX: &str = "MAKOSH_ZULIP_PROVISION ";

type ZulipResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct ZulipServer {
    compose: DockerCompose,
    base_url: String,
    admin_email: String,
}

pub struct ProvisionedZulipRealm {
    pub base_url: String,
    pub owner_email: String,
    pub owner_user_id: i64,
    pub owner_api_key: String,
    pub bot_email: String,
    pub bot_user_id: i64,
    pub bot_api_key: String,
    pub human_email: String,
    pub human_user_id: i64,
    pub human_api_key: String,
    pub stream_name: String,
}

#[derive(Deserialize)]
struct ProvisionedZulipRealmPayload {
    owner_email: String,
    owner_user_id: i64,
    owner_api_key: String,
    bot_email: String,
    bot_user_id: i64,
    bot_api_key: String,
    human_email: String,
    human_user_id: i64,
    human_api_key: String,
    stream_name: String,
}

impl ZulipServer {
    pub fn enabled() -> bool {
        env::var(ZULIP_ENABLE_ENV)
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    pub async fn start() -> ZulipResult<Self> {
        if !Self::enabled() {
            return Err(boxed_error(format!(
                "{ZULIP_ENABLE_ENV}=1 is required to start Zulip"
            )));
        }

        let session_id = session_id_label_value();
        let project_name = format!("makosh-zulip-{}", Uuid::new_v4().simple());
        let compose_path = compose_file_path();
        let admin_email = "makosh-admin@example.com".to_owned();

        eprintln!(
            "[zulip-testcontainer] starting project={project_name} compose={}",
            compose_path.display()
        );

        let containerised = ContainerisedComposeOptions::new(&[compose_path.as_path()])
            .with_project_directory(repo_root());
        let options = AutoComposeOptions::from_containerised(containerised);
        let mut compose = DockerCompose::with_auto_client(options)
            .await?
            .with_project_name(project_name)
            .with_wait(false)
            .with_env_vars(compose_env(&session_id));

        compose.with_remove_volumes(true);
        eprintln!(
            "[zulip-testcontainer] running docker compose up; first image pull can take time"
        );
        heartbeat_while(
            "docker compose up for Zulip fixture",
            READINESS_LOG_INTERVAL,
            compose.up(),
        )
        .await?;

        let proxy_service = compose
            .service("proxy")
            .ok_or_else(|| boxed_error("Zulip proxy compose service was not discovered"))?;
        let host_port = proxy_service.get_host_port_ipv4(8080.tcp()).await?;
        let base_url = format!("http://127.0.0.1:{host_port}");

        eprintln!("[zulip-testcontainer] Zulip proxy mapped to {base_url}");
        wait_for_zulip_http(&base_url, start_timeout()).await?;
        eprintln!("[zulip-testcontainer] Zulip API readiness passed");

        Ok(Self {
            compose,
            base_url,
            admin_email,
        })
    }

    pub async fn provision_test_realm(&self) -> ZulipResult<ProvisionedZulipRealm> {
        eprintln!("[zulip-testcontainer] provisioning root realm, owner, bot, human, stream");
        let service = self
            .compose
            .service("zulip")
            .ok_or_else(|| boxed_error("Zulip compose service was not discovered"))?;
        let mut result = heartbeat_while(
            "Zulip realm provisioning",
            READINESS_LOG_INTERVAL,
            service.exec(
                ExecCommand::new(["sh", "-lc", provision_script()]).with_env_vars([
                    ("MAKOSH_OWNER_PASSWORD", "makosh-owner-password"),
                    ("MAKOSH_REALM_NAME", "Макошь Test"),
                    ("MAKOSH_STREAM_NAME", "makosh-lab"),
                    ("MAKOSH_OWNER_EMAIL", "owner@example.com"),
                    ("MAKOSH_OWNER_NAME", "Макошь Owner"),
                    ("MAKOSH_HUMAN_EMAIL", "alice@example.com"),
                    ("MAKOSH_HUMAN_NAME", "Alice Example"),
                    ("MAKOSH_BOT_EMAIL", "makosh-bot@example.com"),
                    ("MAKOSH_BOT_NAME", "Макошь Bot"),
                ]),
            ),
        )
        .await?;
        let stdout = String::from_utf8_lossy(&result.stdout_to_vec().await?).into_owned();
        let stderr = String::from_utf8_lossy(&result.stderr_to_vec().await?).into_owned();
        let exit_code = result.exit_code().await?;

        if exit_code != Some(0) {
            return Err(boxed_error(format!(
                "Zulip provisioning failed with exit code {exit_code:?}: {}",
                redact_provisioning_output(&stderr)
            )));
        }

        let payload = stdout
            .lines()
            .find_map(|line| line.strip_prefix(PROVISION_PREFIX))
            .ok_or_else(|| boxed_error("Zulip provisioning did not return credentials payload"))?;
        let payload: ProvisionedZulipRealmPayload = serde_json::from_str(payload)?;

        eprintln!(
            "[zulip-testcontainer] provisioned realm stream={} owner={} bot={} human={}",
            payload.stream_name, payload.owner_email, payload.bot_email, payload.human_email
        );

        Ok(ProvisionedZulipRealm {
            base_url: self.base_url.clone(),
            owner_email: payload.owner_email,
            owner_user_id: payload.owner_user_id,
            owner_api_key: payload.owner_api_key,
            bot_email: payload.bot_email,
            bot_user_id: payload.bot_user_id,
            bot_api_key: payload.bot_api_key,
            human_email: payload.human_email,
            human_user_id: payload.human_user_id,
            human_api_key: payload.human_api_key,
            stream_name: payload.stream_name,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn admin_email(&self) -> &str {
        &self.admin_email
    }
}

fn compose_env(session_id: &str) -> HashMap<String, String> {
    HashMap::from([
        (SESSION_ID_ENV.to_owned(), session_id.to_owned()),
        (
            "ZULIP__POSTGRES_PASSWORD".to_owned(),
            "makosh-zulip-postgres-password".to_owned(),
        ),
        (
            "ZULIP__MEMCACHED_PASSWORD".to_owned(),
            "makosh-zulip-memcached-password".to_owned(),
        ),
        (
            "ZULIP__RABBITMQ_PASSWORD".to_owned(),
            "makosh-zulip-rabbitmq-password".to_owned(),
        ),
        (
            "ZULIP__REDIS_PASSWORD".to_owned(),
            "makosh-zulip-redis-password".to_owned(),
        ),
        (
            "ZULIP__SECRET_KEY".to_owned(),
            "makosh-zulip-test-secret-key-not-production-000000000000000000000000".to_owned(),
        ),
        (
            "ZULIP__EMAIL_PASSWORD".to_owned(),
            "makosh-zulip-email-password".to_owned(),
        ),
    ])
}

async fn wait_for_zulip_http(base_url: &str, timeout: Duration) -> ZulipResult<()> {
    let readiness_url = format!("{base_url}/api/v1/server_settings");
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(5))
        .build()?;
    let deadline = Instant::now() + timeout;
    let mut next_log = Instant::now();

    loop {
        let last_error = match client.get(&readiness_url).send().await {
            Ok(response)
                if response.status().is_success() || response.status().is_redirection() =>
            {
                return Ok(());
            }
            Ok(response) => format!("unexpected HTTP status {}", response.status()),
            Err(error) => error.to_string(),
        };

        if Instant::now() >= deadline {
            return Err(boxed_error(format!(
                "Zulip did not become ready within {timeout:?}: {last_error}"
            )));
        }

        if Instant::now() >= next_log {
            eprintln!(
                "[zulip-testcontainer] waiting for Zulip API readiness at {readiness_url}; last={}",
                last_error
            );
            next_log = Instant::now() + READINESS_LOG_INTERVAL;
        }

        sleep(READINESS_POLL_INTERVAL).await;
    }
}

async fn heartbeat_while<F, T>(label: &str, interval: Duration, future: F) -> T
where
    F: Future<Output = T>,
{
    tokio::pin!(future);
    let started = Instant::now();
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker.tick().await;

    loop {
        tokio::select! {
            result = &mut future => return result,
            _ = ticker.tick() => {
                eprintln!(
                    "[zulip-testcontainer] still running {label}; elapsed={}s",
                    started.elapsed().as_secs()
                );
            }
        }
    }
}

fn start_timeout() -> Duration {
    let seconds = env::var(ZULIP_START_TIMEOUT_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_START_TIMEOUT_SECS);
    Duration::from_secs(seconds)
}

fn compose_file_path() -> PathBuf {
    repo_root().join("testing/zulip/compose.testcontainers.yml")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn boxed_error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(io::Error::other(message.into()))
}

fn redact_provisioning_output(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.contains(PROVISION_PREFIX))
        .collect::<Vec<_>>()
        .join("\n")
}

fn provision_script() -> &'static str {
    include_str!("../../../../testing/zulip/provision-test-realm.sh")
}
