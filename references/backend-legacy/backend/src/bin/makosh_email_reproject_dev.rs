use std::env;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use makosh_communications_api::evidence::StoredRawCommunicationRecord;
use makosh_hub_backend::app::router::init_tracing;
use makosh_hub_backend::domains::communications::messages::projection::{
    parse_raw_email_message_from_blob, project_parsed_raw_email_message,
};
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = ReprojectDevConfig::from_env()?;
    let app_config = AppConfig::from_env()?;
    let database_url = app_config
        .database_url()
        .ok_or(ReprojectDevError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(ReprojectDevError::MissingDatabaseUrl)?
        .clone();
    let blob_store = LocalCommunicationBlobStore::new(&config.blob_root);
    let message_store = MessageProjectionStore::new(pool.clone());
    let raw_records = email_blob_records_for_reprojection(
        &pool,
        config.account_id.as_deref(),
        config.only_corrupt,
    )
    .await?;

    let mut reprojected_messages = 0usize;
    let mut failed_records = 0usize;
    for raw_record in &raw_records {
        match parse_raw_email_message_from_blob(&blob_store, raw_record).await {
            Ok(parsed) => {
                match project_parsed_raw_email_message(&message_store, raw_record, &parsed).await {
                    Ok(_) => reprojected_messages += 1,
                    Err(error) => {
                        failed_records += 1;
                        eprintln!(
                            "mail reproject failed raw_record_id={} error={}",
                            raw_record.raw_record_id, error
                        );
                    }
                }
            }
            Err(error) => {
                failed_records += 1;
                eprintln!(
                    "mail raw parse failed raw_record_id={} error={}",
                    raw_record.raw_record_id, error
                );
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&ReprojectDevReport {
            account_id: config.account_id.as_deref(),
            only_corrupt: config.only_corrupt,
            blob_root: config.blob_root.display().to_string(),
            selected_records: raw_records.len(),
            reprojected_messages,
            failed_records,
        })?
    );

    if failed_records > 0 {
        return Err(ReprojectDevError::FailedRecords {
            count: failed_records,
        }
        .into());
    }

    Ok(())
}

async fn email_blob_records_for_reprojection(
    pool: &sqlx::PgPool,
    account_id: Option<&str>,
    only_corrupt: bool,
) -> Result<Vec<StoredRawCommunicationRecord>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            r.raw_record_id,
            r.observation_id,
            r.account_id,
            r.record_kind,
            r.provider_record_id,
            r.source_fingerprint,
            r.import_batch_id,
            r.occurred_at,
            r.captured_at,
            r.payload,
            r.provenance
        FROM communication_raw_records r
        JOIN communication_messages m ON m.raw_record_id = r.raw_record_id
        WHERE r.record_kind = 'email_message'
          AND r.payload->>'raw_blob_storage_kind' = 'local_fs'
          AND r.payload ? 'raw_blob_storage_path'
          AND ($1::text IS NULL OR r.account_id = $1)
          AND (
              $2::bool = false
              OR m.subject LIKE '%�%'
              OR m.sender LIKE '%�%'
              OR m.body_text LIKE '%�%'
          )
        ORDER BY r.captured_at ASC, r.raw_record_id ASC
        "#,
    )
    .bind(account_id)
    .bind(only_corrupt)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(StoredRawCommunicationRecord {
                raw_record_id: row.try_get("raw_record_id")?,
                observation_id: row.try_get("observation_id")?,
                account_id: row.try_get("account_id")?,
                record_kind: row.try_get("record_kind")?,
                provider_record_id: row.try_get("provider_record_id")?,
                source_fingerprint: row.try_get("source_fingerprint")?,
                import_batch_id: row.try_get("import_batch_id")?,
                occurred_at: row.try_get::<Option<DateTime<Utc>>, _>("occurred_at")?,
                captured_at: row.try_get("captured_at")?,
                payload: row.try_get::<Value, _>("payload")?,
                provenance: row.try_get::<Value, _>("provenance")?,
            })
        })
        .collect()
}

struct ReprojectDevConfig {
    account_id: Option<String>,
    only_corrupt: bool,
    blob_root: PathBuf,
}

impl ReprojectDevConfig {
    fn from_env() -> Result<Self, ReprojectDevError> {
        Ok(Self {
            account_id: optional_env("MAKOSH_EMAIL_REPROJECT_ACCOUNT_ID"),
            only_corrupt: optional_env("MAKOSH_EMAIL_REPROJECT_ONLY_CORRUPT")
                .map(|value| parse_bool("MAKOSH_EMAIL_REPROJECT_ONLY_CORRUPT", &value))
                .transpose()?
                .unwrap_or(true),
            blob_root: PathBuf::from(
                optional_env("MAKOSH_EMAIL_REPROJECT_BLOB_ROOT")
                    .unwrap_or_else(|| DEFAULT_MAIL_SYNC_BLOB_ROOT.to_owned()),
            ),
        })
    }
}

#[derive(Serialize)]
struct ReprojectDevReport<'a> {
    account_id: Option<&'a str>,
    only_corrupt: bool,
    blob_root: String,
    selected_records: usize,
    reprojected_messages: usize,
    failed_records: usize,
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_bool(name: &'static str, value: &str) -> Result<bool, ReprojectDevError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(ReprojectDevError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

#[derive(Debug, Error)]
enum ReprojectDevError {
    #[error("DATABASE_URL is required for email reproject dev command")]
    MissingDatabaseUrl,

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },

    #[error("mail reproject failed for {count} raw records")]
    FailedRecords { count: usize },
}
