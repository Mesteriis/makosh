use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use makosh_signal_hub_api::raw_signals::{
    RawSignalPersistenceError, RawSignalPersistenceErrorKind,
};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use uuid::Uuid;

use super::errors::storage_error;

pub(super) async fn list_active(
    pool: &PgPool,
) -> Result<Vec<SignalPolicy>, RawSignalPersistenceError> {
    let rows = sqlx::query(
        r#"
        SELECT scope, source_code, connection_id, event_pattern, mode, reason, expires_at
        FROM signal_policies
        WHERE expires_at IS NULL OR expires_at > now()
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(storage_error)?;

    rows.into_iter().map(row_to_policy).collect()
}

fn row_to_policy(row: PgRow) -> Result<SignalPolicy, RawSignalPersistenceError> {
    let scope: String = row.try_get("scope").map_err(storage_error)?;
    let mode: String = row.try_get("mode").map_err(storage_error)?;
    let connection_id: Option<Uuid> = row.try_get("connection_id").map_err(storage_error)?;
    let scope = SignalPolicyScope::parse(&scope).ok_or_else(|| {
        RawSignalPersistenceError::new(RawSignalPersistenceErrorKind::InvalidPolicyScope, scope)
    })?;
    let mode = SignalPolicyMode::parse(&mode).ok_or_else(|| {
        RawSignalPersistenceError::new(RawSignalPersistenceErrorKind::InvalidPolicyMode, mode)
    })?;

    Ok(SignalPolicy {
        scope,
        source_code: row.try_get("source_code").map_err(storage_error)?,
        connection_id: connection_id.map(|value| value.to_string()),
        event_pattern: row.try_get("event_pattern").map_err(storage_error)?,
        mode,
        reason: row.try_get("reason").map_err(storage_error)?,
        expires_at: row.try_get("expires_at").map_err(storage_error)?,
    })
}
