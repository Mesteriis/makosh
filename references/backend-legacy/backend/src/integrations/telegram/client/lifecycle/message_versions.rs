use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};

use super::ids::new_version_id;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::evidence::link_telegram_entity_in_transaction;
use crate::integrations::telegram::client::models::messages::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageVersion;
use crate::integrations::telegram::client::rows::row_to_telegram_message_version;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

async fn capture_message_version_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    version: &TelegramMessageVersion,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_MESSAGE_VERSION",
            ObservationOriginKind::LocalRuntime,
            version.created_at,
            json!({
                "version_id": version.version_id,
                "message_id": version.message_id,
                "account_id": version.account_id,
                "provider_message_id": version.provider_message_id,
                "provider_chat_id": version.provider_chat_id,
                "version_number": version.version_number,
                "body_text": version.body_text,
                "edit_timestamp": version.edit_timestamp,
                "source_event": version.source_event,
                "raw_diff_payload": version.raw_diff_payload,
                "provenance": version.provenance,
                "operation": relationship_kind,
            }),
            format!(
                "telegram-message-version://{}/{}",
                version.version_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "message_version",
        version.version_id.clone(),
        relationship_kind,
        json!({
            "message_id": version.message_id,
            "account_id": version.account_id,
            "provider_message_id": version.provider_message_id,
            "provider_chat_id": version.provider_chat_id,
            "version_number": version.version_number,
        }),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_message_version(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    version_number: i32,
    body_text: Option<&str>,
    edit_timestamp: DateTime<Utc>,
    source_event: Option<&str>,
    raw_diff: Value,
    provenance: Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    let version_id = new_version_id();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_message_versions
            (version_id, message_id, account_id, provider_message_id, provider_chat_id,
             version_number, body_text, edit_timestamp, source_event,
             raw_diff_payload, provenance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(&version_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(version_number)
    .bind(body_text)
    .bind(edit_timestamp)
    .bind(source_event)
    .bind(&raw_diff)
    .bind(&provenance)
    .fetch_one(&mut *transaction)
    .await?;

    let version = row_to_telegram_message_version(row)?;
    capture_message_version_observation_in_transaction(
        &mut transaction,
        &version,
        "insert",
        "telegram.client.lifecycle.message_versions.insert_message_version",
    )
    .await?;
    transaction.commit().await?;
    Ok(version)
}

pub async fn list_message_versions(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageVersion>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_version)
        .collect()
}

pub async fn latest_message_version(
    pool: &PgPool,
    message_id: &str,
) -> Result<Option<TelegramMessageVersion>, TelegramError> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_telegram_message_version).transpose()
}

pub async fn latest_version_number(pool: &PgPool, message_id: &str) -> Result<i32, TelegramError> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(version_number), 0) as max_ver
        FROM telegram_message_versions
        WHERE message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(0))
}

pub async fn record_provider_edit_observation(
    pool: &PgPool,
    message: &TelegramMessage,
    body_text: &str,
    edit_timestamp: DateTime<Utc>,
    source_event: &str,
    raw_diff: Value,
    provenance: Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    if let Some(existing) = latest_message_version(pool, &message.message_id).await?
        && existing.body_text.as_deref() == Some(body_text)
        && existing.source_event.as_deref() == Some(source_event)
        && existing.edit_timestamp == edit_timestamp
    {
        return Ok(existing);
    }

    let version_number = latest_version_number(pool, &message.message_id).await? + 1;
    insert_message_version(
        pool,
        &message.message_id,
        &message.account_id,
        &message.provider_message_id,
        message.provider_chat_id.as_deref().unwrap_or_default(),
        version_number,
        Some(body_text),
        edit_timestamp,
        Some(source_event),
        raw_diff,
        provenance,
    )
    .await
}

pub(crate) fn local_edit_diff(previous_text: Option<&str>, new_text: &str) -> Value {
    let previous_text_length = previous_text.map(text_len);
    let new_text_length = text_len(new_text);
    let text_length_delta =
        previous_text_length.map(|previous| new_text_length as i64 - previous as i64);

    json!({
        "previous_text_length": previous_text_length,
        "new_text_length": new_text_length,
        "text_length_delta": text_length_delta,
        "changed": previous_text != Some(new_text),
        "previous_preview": previous_text.map(text_preview),
        "new_preview": text_preview(new_text),
        "previous_sha256": previous_text.map(sha256_hex),
        "new_sha256": sha256_hex(new_text),
    })
}

fn text_len(text: &str) -> usize {
    text.chars().count()
}

fn text_preview(text: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 160;
    text.chars().take(MAX_PREVIEW_CHARS).collect()
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::local_edit_diff;
    use serde_json::json;

    #[test]
    fn local_edit_diff_records_previous_and_new_text_metadata() {
        let diff = local_edit_diff(Some("before body"), "after body!");

        assert_eq!(diff["previous_text_length"], json!(11));
        assert_eq!(diff["new_text_length"], json!(11));
        assert_eq!(diff["text_length_delta"], json!(0));
        assert_eq!(diff["changed"], json!(true));
        assert_eq!(diff["previous_preview"], json!("before body"));
        assert_eq!(diff["new_preview"], json!("after body!"));
        assert_eq!(
            diff["previous_sha256"]
                .as_str()
                .expect("previous hash")
                .len(),
            64
        );
        assert_eq!(diff["new_sha256"].as_str().expect("new hash").len(), 64);
    }
}
