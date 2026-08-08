use makosh_events_api::EventEnvelope;
use makosh_signal_hub_api::raw_signals::RawSignalPersistenceError;
use serde_json::Value;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::errors::storage_error;

pub(super) async fn resolve_connection_id(
    pool: &PgPool,
    source_code: &str,
    event: &EventEnvelope,
) -> Result<Option<String>, RawSignalPersistenceError> {
    let Some(account_id) = raw_signal_account_id(event) else {
        return Ok(None);
    };
    let connection_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM signal_connections
        WHERE source_code = $1
          AND settings->>'account_id' = $2
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(source_code)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(storage_error)?;

    Ok(connection_id.map(|value| value.to_string()))
}

fn raw_signal_account_id(event: &EventEnvelope) -> Option<String> {
    string_field(&event.source, "account_id")
        .or_else(|| string_field(&event.subject, "account_id"))
        .or_else(|| string_field(&event.provenance, "account_id"))
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
