use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use super::ids::new_tombstone_id;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::evidence::link_telegram_entity_in_transaction;
use crate::integrations::telegram::client::models::messages::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageTombstone;
use crate::integrations::telegram::client::rows::row_to_telegram_message_tombstone;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

async fn capture_tombstone_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    tombstone: &TelegramMessageTombstone,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_MESSAGE_TOMBSTONE",
            ObservationOriginKind::LocalRuntime,
            tombstone.created_at,
            json!({
                "tombstone_id": tombstone.tombstone_id,
                "message_id": tombstone.message_id,
                "account_id": tombstone.account_id,
                "provider_message_id": tombstone.provider_message_id,
                "provider_chat_id": tombstone.provider_chat_id,
                "reason_class": tombstone.reason_class,
                "actor_class": tombstone.actor_class,
                "observed_at": tombstone.observed_at,
                "source_event": tombstone.source_event,
                "is_provider_delete": tombstone.is_provider_delete,
                "is_local_visible": tombstone.is_local_visible,
                "metadata": tombstone.metadata,
                "provenance": tombstone.provenance,
                "operation": relationship_kind,
            }),
            format!(
                "telegram-message-tombstone://{}/{}",
                tombstone.tombstone_id, relationship_kind
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
        "message_tombstone",
        tombstone.tombstone_id.clone(),
        relationship_kind,
        json!({
            "message_id": tombstone.message_id,
            "account_id": tombstone.account_id,
            "provider_message_id": tombstone.provider_message_id,
            "provider_chat_id": tombstone.provider_chat_id,
            "reason_class": tombstone.reason_class,
            "actor_class": tombstone.actor_class,
            "is_local_visible": tombstone.is_local_visible,
        }),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_tombstone(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    reason_class: &str,
    actor_class: &str,
    observed_at: DateTime<Utc>,
    source_event: Option<&str>,
    is_provider_delete: bool,
    is_local_visible: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let tombstone_id = new_tombstone_id();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(&tombstone_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(reason_class)
    .bind(actor_class)
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(is_local_visible)
    .fetch_one(&mut *transaction)
    .await?;

    let tombstone = row_to_telegram_message_tombstone(row)?;
    capture_tombstone_observation_in_transaction(
        &mut transaction,
        &tombstone,
        "insert",
        "telegram.client.lifecycle.tombstones.insert_tombstone",
    )
    .await?;
    transaction.commit().await?;
    Ok(tombstone)
}

pub async fn list_tombstones(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageTombstone>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_tombstone)
        .collect()
}

pub async fn is_message_visible(pool: &PgPool, message_id: &str) -> Result<bool, TelegramError> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT is_local_visible
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(true))
}

pub async fn record_provider_delete_observation(
    pool: &PgPool,
    message: &TelegramMessage,
    observed_at: DateTime<Utc>,
    source_event: &str,
    is_provider_delete: bool,
    from_cache: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let latest = sqlx::query(
        r#"
        SELECT *
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&message.message_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = latest {
        let tombstone = row_to_telegram_message_tombstone(row)?;
        if tombstone.reason_class == "deleted_by_provider"
            && tombstone.actor_class == "provider"
            && !tombstone.is_local_visible
        {
            return Ok(tombstone);
        }
    }

    let tombstone_id = new_tombstone_id();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible, metadata, provenance)
        VALUES ($1, $2, $3, $4, $5, 'deleted_by_provider', 'provider', $6, $7, $8, false, $9, $10)
        RETURNING *
        "#,
    )
    .bind(&tombstone_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_message_id)
    .bind(message.provider_chat_id.as_deref().unwrap_or_default())
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(json!({
        "from_cache": from_cache,
        "provider_delete": is_provider_delete,
    }))
    .bind(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "source": source_event,
    }))
    .fetch_one(&mut *transaction)
    .await?;

    let tombstone = row_to_telegram_message_tombstone(row)?;
    capture_tombstone_observation_in_transaction(
        &mut transaction,
        &tombstone,
        "provider_delete",
        "telegram.client.lifecycle.tombstones.record_provider_delete_observation",
    )
    .await?;
    transaction.commit().await?;
    Ok(tombstone)
}
