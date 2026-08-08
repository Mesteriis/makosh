use makosh_events_api::EventEnvelope;
use makosh_signal_hub_api::raw_signals::{
    RawSignalPersistenceError, RawSignalPersistenceErrorKind,
};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::errors::storage_error;

pub(super) async fn record(
    pool: &PgPool,
    event: &EventEnvelope,
    source_code: &str,
    connection_id: Option<&str>,
    reason: &str,
) -> Result<(), RawSignalPersistenceError> {
    let connection_id = connection_id
        .map(|value| {
            Uuid::parse_str(value).map_err(|error| {
                RawSignalPersistenceError::new(
                    RawSignalPersistenceErrorKind::InvalidConnectionId,
                    error,
                )
            })
        })
        .transpose()?;
    let event_envelope = serde_json::to_value(event).map_err(|error| {
        RawSignalPersistenceError::new(RawSignalPersistenceErrorKind::Serialization, error)
    })?;
    sqlx::query(
        r#"
        INSERT INTO signal_paused_events (
            id,
            event_id,
            source_code,
            connection_id,
            raw_event_type,
            event_envelope,
            reason
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (event_id) DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(&event.event_id)
    .bind(source_code)
    .bind(connection_id)
    .bind(&event.event_type)
    .bind(event_envelope)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(storage_error)?;

    Ok(())
}
