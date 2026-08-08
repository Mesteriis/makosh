use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::email_fixture_pipeline::{
    EmailFixturePipelineRequest, import_fixture_email_messages_for_dev,
    project_fixture_email_messages,
};
use thiserror::Error;

const DEFAULT_FIXTURE_PATH: &str = "tmp/email-fixtures/icloud-inbox-redacted.json";
const DEFAULT_ACCOUNT_ID: &str = "dev-icloud-fixture";
const DEFAULT_DISPLAY_NAME: &str = "iCloud Redacted Fixture";
const DEFAULT_EXTERNAL_ACCOUNT_ID: &str = "redacted-icloud@example.invalid";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    makosh_hub_backend::app::router::init_tracing();

    let config = EmailFixtureDevConfig::from_env()?;
    let app_config = AppConfig::from_env()?;
    let database_url = app_config
        .database_url()
        .ok_or(EmailFixtureDevConfigError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(EmailFixtureDevConfigError::MissingDatabaseUrl)?
        .clone();
    let fixture_json = fs::read_to_string(&config.fixture_path).map_err(|source| {
        EmailFixtureDevConfigError::FixtureRead {
            path: config.fixture_path.clone(),
            source,
        }
    })?;
    let request = EmailFixturePipelineRequest::new(
        config.account_id,
        config.display_name,
        config.external_account_id,
        config.provider_kind,
        config.import_batch_id,
        fixture_json,
    );

    match config.mode {
        EmailFixtureDevMode::Import => {
            let provider_accounts = makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone());
            let communication_evidence =
                makosh_communications_postgres::store::CommunicationIngestionStore::new(
                    pool.clone(),
                );
            let report = import_fixture_email_messages_for_dev(
                &provider_accounts,
                &communication_evidence,
                &request,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        EmailFixtureDevMode::Project => {
            let provider_accounts = makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone());
            let communication_evidence =
                makosh_communications_postgres::store::CommunicationIngestionStore::new(
                    pool.clone(),
                );
            let report = project_fixture_email_messages(
                pool,
                &provider_accounts,
                &communication_evidence,
                &request,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}

struct EmailFixtureDevConfig {
    mode: EmailFixtureDevMode,
    fixture_path: PathBuf,
    account_id: String,
    display_name: String,
    external_account_id: String,
    provider_kind: CommunicationProviderKind,
    import_batch_id: String,
}

impl EmailFixtureDevConfig {
    fn from_env() -> Result<Self, EmailFixtureDevConfigError> {
        let mode = parse_mode(
            optional_env("MAKOSH_EMAIL_FIXTURE_MODE")
                .unwrap_or_else(|| "project".to_owned())
                .as_str(),
        )?;
        let provider_kind = parse_provider_kind(
            optional_env("MAKOSH_EMAIL_FIXTURE_PROVIDER")
                .unwrap_or_else(|| "icloud".to_owned())
                .as_str(),
        )?;

        Ok(Self {
            mode,
            fixture_path: PathBuf::from(
                optional_env("MAKOSH_EMAIL_FIXTURE_PATH")
                    .unwrap_or_else(|| DEFAULT_FIXTURE_PATH.to_owned()),
            ),
            account_id: optional_env("MAKOSH_EMAIL_FIXTURE_ACCOUNT_ID")
                .unwrap_or_else(|| DEFAULT_ACCOUNT_ID.to_owned()),
            display_name: optional_env("MAKOSH_EMAIL_FIXTURE_DISPLAY_NAME")
                .unwrap_or_else(|| DEFAULT_DISPLAY_NAME.to_owned()),
            external_account_id: optional_env("MAKOSH_EMAIL_FIXTURE_EXTERNAL_ACCOUNT_ID")
                .unwrap_or_else(|| DEFAULT_EXTERNAL_ACCOUNT_ID.to_owned()),
            provider_kind,
            import_batch_id: optional_env("MAKOSH_EMAIL_FIXTURE_IMPORT_BATCH_ID")
                .unwrap_or_else(|| format!("fixture-dev-{}", Utc::now().timestamp())),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmailFixtureDevMode {
    Import,
    Project,
}

fn parse_mode(value: &str) -> Result<EmailFixtureDevMode, EmailFixtureDevConfigError> {
    match value.trim() {
        "import" => Ok(EmailFixtureDevMode::Import),
        "project" => Ok(EmailFixtureDevMode::Project),
        other => Err(EmailFixtureDevConfigError::InvalidMode(other.to_owned())),
    }
}

fn parse_provider_kind(
    value: &str,
) -> Result<CommunicationProviderKind, EmailFixtureDevConfigError> {
    CommunicationProviderKind::try_from(value.trim())
        .map_err(|_| EmailFixtureDevConfigError::InvalidProviderKind(value.to_owned()))
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Error)]
enum EmailFixtureDevConfigError {
    #[error("DATABASE_URL is required for email fixture dev commands")]
    MissingDatabaseUrl,

    #[error("failed to read fixture file `{path}`: {source}")]
    FixtureRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid MAKOSH_EMAIL_FIXTURE_MODE `{0}`; expected `import` or `project`")]
    InvalidMode(String),

    #[error("invalid MAKOSH_EMAIL_FIXTURE_PROVIDER `{0}`; expected `gmail`, `icloud` or `imap`")]
    InvalidProviderKind(String),
}
