use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::models::topics::{NewTelegramTopic, TelegramTopic};
use super::store::TelegramStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

fn row_to_telegram_topic(row: sqlx::postgres::PgRow) -> Result<TelegramTopic, TelegramError> {
    use sqlx::Row;
    Ok(TelegramTopic {
        topic_id: row.try_get("topic_id")?,
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_topic_id: row.try_get("provider_topic_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        title: row.try_get("title")?,
        icon_emoji: row.try_get("icon_emoji")?,
        is_pinned: row.try_get("is_pinned")?,
        is_closed: row.try_get("is_closed")?,
        unread_count: row.try_get("unread_count")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

async fn capture_topic_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    topic: &TelegramTopic,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_TOPIC",
            ObservationOriginKind::LocalRuntime,
            topic.updated_at,
            json!({
                "topic_id": topic.topic_id,
                "telegram_chat_id": topic.telegram_chat_id,
                "account_id": topic.account_id,
                "provider_topic_id": topic.provider_topic_id,
                "provider_chat_id": topic.provider_chat_id,
                "title": topic.title,
                "icon_emoji": topic.icon_emoji,
                "is_pinned": topic.is_pinned,
                "is_closed": topic.is_closed,
                "unread_count": topic.unread_count,
                "last_message_at": topic.last_message_at,
                "metadata": topic.metadata,
                "operation": relationship_kind,
            }),
            format!("telegram-topic://{}/{}", topic.topic_id, relationship_kind),
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
        "topic",
        topic.topic_id.clone(),
        relationship_kind,
        json!({
            "telegram_chat_id": topic.telegram_chat_id,
            "account_id": topic.account_id,
            "provider_topic_id": topic.provider_topic_id,
            "provider_chat_id": topic.provider_chat_id,
            "is_closed": topic.is_closed,
            "is_pinned": topic.is_pinned,
        }),
    )
    .await?;
    Ok(())
}

pub async fn upsert_topic(
    pool: &PgPool,
    topic: &NewTelegramTopic,
) -> Result<TelegramTopic, TelegramError> {
    let now = Utc::now();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r"
        INSERT INTO telegram_topics (
            topic_id, telegram_chat_id, account_id, provider_topic_id, provider_chat_id,
            title, icon_emoji, is_pinned, is_closed, unread_count, last_message_at,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
        ON CONFLICT (telegram_chat_id, provider_topic_id)
        DO UPDATE SET
            title        = EXCLUDED.title,
            icon_emoji   = EXCLUDED.icon_emoji,
            is_pinned    = EXCLUDED.is_pinned,
            is_closed    = EXCLUDED.is_closed,
            unread_count = EXCLUDED.unread_count,
            last_message_at = EXCLUDED.last_message_at,
            updated_at   = EXCLUDED.updated_at
        RETURNING *
        ",
    )
    .bind(&topic.topic_id)
    .bind(&topic.telegram_chat_id)
    .bind(&topic.account_id)
    .bind(topic.provider_topic_id)
    .bind(&topic.provider_chat_id)
    .bind(&topic.title)
    .bind(&topic.icon_emoji)
    .bind(topic.is_pinned)
    .bind(topic.is_closed)
    .bind(topic.unread_count)
    .bind(topic.last_message_at)
    .bind(now)
    .fetch_one(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let stored = row_to_telegram_topic(row)?;
    capture_topic_observation_in_transaction(
        &mut transaction,
        &stored,
        "upsert",
        "telegram.client.topics.upsert_topic",
    )
    .await?;
    transaction.commit().await?;
    Ok(stored)
}

pub async fn list_topics(
    pool: &PgPool,
    telegram_chat_id: &str,
    limit: i64,
) -> Result<Vec<TelegramTopic>, TelegramError> {
    let rows = sqlx::query(
        r"
        SELECT * FROM telegram_topics
        WHERE telegram_chat_id = $1
        ORDER BY is_pinned DESC, last_message_at DESC NULLS LAST, updated_at DESC
        LIMIT $2
        ",
    )
    .bind(telegram_chat_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter().map(row_to_telegram_topic).collect()
}

pub async fn get_topic(
    pool: &PgPool,
    topic_id: &str,
) -> Result<Option<TelegramTopic>, TelegramError> {
    let row = sqlx::query("SELECT * FROM telegram_topics WHERE topic_id = $1")
        .bind(topic_id)
        .fetch_optional(pool)
        .await
        .map_err(TelegramError::from)?;

    row.map(row_to_telegram_topic).transpose()
}

pub async fn search_topics(
    pool: &PgPool,
    telegram_chat_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<TelegramTopic>, TelegramError> {
    let pattern = format!("%{}%", query.trim().to_lowercase());
    let rows = sqlx::query(
        r"
        SELECT * FROM telegram_topics
        WHERE telegram_chat_id = $1
          AND lower(title) LIKE $2
        ORDER BY is_pinned DESC, last_message_at DESC NULLS LAST, updated_at DESC
        LIMIT $3
        ",
    )
    .bind(telegram_chat_id)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter().map(row_to_telegram_topic).collect()
}

pub async fn list_topic_message_ids(
    store: &TelegramStore,
    topic_id: &str,
    limit: i64,
) -> Result<Vec<String>, TelegramError> {
    Ok(store
        .provider_channel_message_store()
        .message_ids_by_metadata_string("forum_topic_id", topic_id, TELEGRAM_CHANNEL_KINDS, limit)
        .await?)
}
