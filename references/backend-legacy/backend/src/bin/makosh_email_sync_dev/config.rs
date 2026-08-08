use std::path::PathBuf;

use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_hub_backend::platform::secrets::models::ResolvedSecret;

use crate::env::{first_env, optional_env, parse_bool, parse_port, parse_usize};
use crate::errors::DevEmailSyncError;
use crate::provider::{default_host, parse_provider_kind};

const DEFAULT_MAILBOX: &str = "INBOX";
const DEFAULT_MAX_MESSAGES: usize = 25;
const DEFAULT_BLOB_ROOT: &str = "docker/data/mail";

pub(super) struct DevEmailSyncConfig {
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) provider_kind: CommunicationProviderKind,
    pub(super) username: String,
    pub(super) password: ResolvedSecret,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) mailbox: String,
    pub(super) max_messages: usize,
    pub(super) blob_root: PathBuf,
    pub(super) import_batch_id: String,
}

impl DevEmailSyncConfig {
    pub(super) fn from_env() -> Result<Self, DevEmailSyncError> {
        let provider_kind = parse_provider_kind(
            optional_env("MAKOSH_EMAIL_SYNC_PROVIDER")
                .unwrap_or_else(|| "icloud".to_owned())
                .as_str(),
        )?;
        let username = first_env([
            "MAKOSH_EMAIL_SYNC_USERNAME",
            "MAKOSH_IMAP_FIXTURE_USERNAME",
            "ICLOUD_LOGIN",
        ])?;
        let external_account_id = optional_env("MAKOSH_EMAIL_SYNC_EXTERNAL_ACCOUNT_ID")
            .unwrap_or_else(|| username.clone());
        let password = ResolvedSecret::new(first_env([
            "MAKOSH_EMAIL_SYNC_PASSWORD",
            "MAKOSH_IMAP_FIXTURE_PASSWORD",
            "ICLOUD_2FA",
        ])?)?;

        Ok(Self {
            account_id: optional_env("MAKOSH_EMAIL_SYNC_ACCOUNT_ID")
                .unwrap_or_else(|| format!("dev-{}-mail-cache", provider_kind.as_str())),
            display_name: optional_env("MAKOSH_EMAIL_SYNC_DISPLAY_NAME")
                .unwrap_or_else(|| "Dev Mail Cache".to_owned()),
            external_account_id,
            provider_kind,
            username,
            password,
            host: optional_env("MAKOSH_EMAIL_SYNC_HOST")
                .unwrap_or_else(|| default_host(provider_kind).to_owned()),
            port: optional_env("MAKOSH_EMAIL_SYNC_PORT")
                .map(|value| parse_port("MAKOSH_EMAIL_SYNC_PORT", &value))
                .transpose()?
                .unwrap_or(crate::provider::DEFAULT_IMAP_PORT),
            tls: optional_env("MAKOSH_EMAIL_SYNC_TLS")
                .map(|value| parse_bool("MAKOSH_EMAIL_SYNC_TLS", &value))
                .transpose()?
                .unwrap_or(true),
            mailbox: optional_env("MAKOSH_EMAIL_SYNC_MAILBOX")
                .unwrap_or_else(|| DEFAULT_MAILBOX.to_owned()),
            max_messages: optional_env("MAKOSH_EMAIL_SYNC_MAX_MESSAGES")
                .map(|value| parse_usize("MAKOSH_EMAIL_SYNC_MAX_MESSAGES", &value))
                .transpose()?
                .unwrap_or(DEFAULT_MAX_MESSAGES),
            blob_root: PathBuf::from(
                optional_env("MAKOSH_EMAIL_SYNC_BLOB_ROOT")
                    .unwrap_or_else(|| DEFAULT_BLOB_ROOT.to_owned()),
            ),
            import_batch_id: optional_env("MAKOSH_EMAIL_SYNC_IMPORT_BATCH_ID")
                .unwrap_or_else(|| format!("email-sync-dev-{}", chrono::Utc::now().timestamp())),
        })
    }
}
