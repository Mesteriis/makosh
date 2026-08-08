use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_hub_backend::domains::communications::fixtures::export::export_fixture_messages_from_sync_batch;
use makosh_hub_backend::domains::communications::fixtures::export::models::EmailFixtureExportOptions;
use makosh_hub_backend::integrations::mail::gmail::client::{
    imap::ImapNetworkClient, options::ImapFetchOptions,
};
use makosh_hub_backend::platform::secrets::models::ResolvedSecret;
use serde::Serialize;
use thiserror::Error;

const DEFAULT_ICLOUD_IMAP_HOST: &str = "imap.mail.me.com";
const DEFAULT_IMAP_PORT: u16 = 993;
const DEFAULT_MAILBOX: &str = "INBOX";
const DEFAULT_MAX_MESSAGES: usize = 10;
const DEFAULT_OUTPUT_PATH: &str = "tmp/email-fixtures/icloud-inbox-redacted.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    makosh_hub_backend::app::router::init_tracing();

    let config = LiveImapFixtureConfig::from_env()?;
    ensure_parent_dir(&config.output_path)?;

    let password = ResolvedSecret::new(config.password)?;
    let mut options = ImapFetchOptions::new(
        config.host,
        config.port,
        config.tls,
        config.mailbox.clone(),
        config.username,
    )
    .provider_kind(CommunicationProviderKind::Icloud)
    .max_messages(config.max_messages)
    .latest_messages();

    if let Some(last_seen_uid) = config.last_seen_uid {
        options = options.last_seen_uid(last_seen_uid);
    }

    let batch = ImapNetworkClient::new()
        .fetch_raw_messages(&password, &options)
        .await?;
    let checkpoint = batch.checkpoint.clone();
    let fixtures =
        export_fixture_messages_from_sync_batch(&batch, EmailFixtureExportOptions::default())?;
    let exported_messages = fixtures.len();

    fs::write(
        &config.output_path,
        serde_json::to_string_pretty(&fixtures)?,
    )?;

    println!(
        "{}",
        serde_json::to_string_pretty(&LiveImapFixtureExportReport {
            provider: "icloud",
            mailbox: &config.mailbox,
            exported_messages,
            output_path: config.output_path.display().to_string(),
            redaction: "redacted",
            checkpoint,
        })?
    );

    Ok(())
}

struct LiveImapFixtureConfig {
    username: String,
    password: String,
    host: String,
    port: u16,
    tls: bool,
    mailbox: String,
    max_messages: usize,
    last_seen_uid: Option<u32>,
    output_path: PathBuf,
}

impl LiveImapFixtureConfig {
    fn from_env() -> Result<Self, LiveImapFixtureConfigError> {
        Ok(Self {
            username: first_env(["MAKOSH_IMAP_FIXTURE_USERNAME", "ICLOUD_LOGIN"])?,
            password: first_env(["MAKOSH_IMAP_FIXTURE_PASSWORD", "ICLOUD_2FA"])?,
            host: optional_env("MAKOSH_IMAP_FIXTURE_HOST")
                .unwrap_or_else(|| DEFAULT_ICLOUD_IMAP_HOST.to_owned()),
            port: optional_env("MAKOSH_IMAP_FIXTURE_PORT")
                .map(|value| parse_port("MAKOSH_IMAP_FIXTURE_PORT", &value))
                .transpose()?
                .unwrap_or(DEFAULT_IMAP_PORT),
            tls: optional_env("MAKOSH_IMAP_FIXTURE_TLS")
                .map(|value| parse_bool("MAKOSH_IMAP_FIXTURE_TLS", &value))
                .transpose()?
                .unwrap_or(true),
            mailbox: optional_env("MAKOSH_IMAP_FIXTURE_MAILBOX")
                .unwrap_or_else(|| DEFAULT_MAILBOX.to_owned()),
            max_messages: optional_env("MAKOSH_IMAP_FIXTURE_MAX_MESSAGES")
                .map(|value| parse_usize("MAKOSH_IMAP_FIXTURE_MAX_MESSAGES", &value))
                .transpose()?
                .unwrap_or(DEFAULT_MAX_MESSAGES),
            last_seen_uid: optional_env("MAKOSH_IMAP_FIXTURE_LAST_SEEN_UID")
                .map(|value| parse_u32("MAKOSH_IMAP_FIXTURE_LAST_SEEN_UID", &value))
                .transpose()?,
            output_path: PathBuf::from(
                optional_env("MAKOSH_IMAP_FIXTURE_OUTPUT")
                    .unwrap_or_else(|| DEFAULT_OUTPUT_PATH.to_owned()),
            ),
        })
    }
}

#[derive(Serialize)]
struct LiveImapFixtureExportReport<'a> {
    provider: &'a str,
    mailbox: &'a str,
    exported_messages: usize,
    output_path: String,
    redaction: &'a str,
    checkpoint: Option<serde_json::Value>,
}

fn first_env<const N: usize>(
    names: [&'static str; N],
) -> Result<String, LiveImapFixtureConfigError> {
    for name in names.iter().copied() {
        if let Some(value) = optional_env(name) {
            return Ok(value);
        }
    }

    Err(LiveImapFixtureConfigError::MissingEnv(names.join(" or ")))
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_port(name: &'static str, value: &str) -> Result<u16, LiveImapFixtureConfigError> {
    let port = parse_u16(name, value)?;
    if port == 0 {
        return Err(LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(port)
}

fn parse_bool(name: &'static str, value: &str) -> Result<bool, LiveImapFixtureConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

fn parse_usize(name: &'static str, value: &str) -> Result<usize, LiveImapFixtureConfigError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected positive integer",
        })?;
    if parsed == 0 {
        return Err(LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(parsed)
}

fn parse_u32(name: &'static str, value: &str) -> Result<u32, LiveImapFixtureConfigError> {
    value
        .parse::<u32>()
        .map_err(|_| LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u32 integer",
        })
}

fn parse_u16(name: &'static str, value: &str) -> Result<u16, LiveImapFixtureConfigError> {
    value
        .parse::<u16>()
        .map_err(|_| LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u16 integer",
        })
}

fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[derive(Debug, Error)]
enum LiveImapFixtureConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },
}
