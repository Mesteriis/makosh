use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::integrations::telegram::tdjson::{
    snapshots::TelegramTdlibChatFolderSnapshot, snapshots::TelegramTdlibChatSnapshot,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

use super::TELEGRAM_CHAT_RECORD_KIND;
use super::chat_metadata::tdlib_chat_projection_metadata;
use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::identifiers::{telegram_chat_id, telegram_raw_record_id};
use super::models::chats::{
    NewTelegramChat, TelegramChat, TelegramChatGroupFilter, TelegramChatMember, TelegramSyncState,
};
use super::rows::row_to_telegram_chat;
use super::store::TelegramStore;
use super::validation::validate_chat_list_limit;

#[path = "chats/metadata_flags.rs"]
mod metadata_flags;

async fn capture_chat_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    chat: &TelegramChat,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_CHAT",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "telegram_chat_id": chat.telegram_chat_id,
                "account_id": chat.account_id,
                "provider_chat_id": chat.provider_chat_id,
                "chat_kind": chat.chat_kind,
                "title": chat.title,
                "username": chat.username,
                "sync_state": chat.sync_state,
                "last_message_at": chat.last_message_at,
                "metadata": chat.metadata,
                "operation": relationship_kind,
            }),
            match relationship_kind {
                "upsert" => format!("telegram-chat://{}", chat.telegram_chat_id),
                _ => format!(
                    "telegram-chat://{}/{}",
                    chat.telegram_chat_id, relationship_kind
                ),
            },
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
        "chat",
        chat.telegram_chat_id.clone(),
        relationship_kind,
        json!({
            "account_id": chat.account_id,
            "provider_chat_id": chat.provider_chat_id,
            "chat_kind": chat.chat_kind,
            "sync_state": chat.sync_state,
        }),
    )
    .await?;
    Ok(())
}

impl TelegramStore {
    pub async fn upsert_chat(&self, chat: &NewTelegramChat) -> Result<TelegramChat, TelegramError> {
        chat.validate()?;
        let telegram_chat_id = telegram_chat_id(&chat.account_id, &chat.provider_chat_id);
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_chats (
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            ON CONFLICT (account_id, provider_chat_id)
            DO UPDATE SET
                chat_kind = EXCLUDED.chat_kind,
                title = EXCLUDED.title,
                username = COALESCE(EXCLUDED.username, telegram_chats.username),
                sync_state = EXCLUDED.sync_state,
                last_message_at = EXCLUDED.last_message_at,
                metadata = COALESCE(telegram_chats.metadata, '{}'::jsonb) || EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&telegram_chat_id)
        .bind(chat.account_id.trim())
        .bind(chat.provider_chat_id.trim())
        .bind(chat.chat_kind.as_str())
        .bind(chat.title.trim())
        .bind(
            chat.username
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(chat.sync_state.as_str())
        .bind(chat.last_message_at)
        .bind(&chat.metadata)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_telegram_chat(row)?;
        capture_chat_observation_in_transaction(
            &mut transaction,
            &stored,
            "upsert",
            "telegram.client.chats.upsert_chat",
            stored.updated_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_chats(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let limit = validate_chat_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY COALESCE(last_message_at, updated_at) DESC, telegram_chat_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }

    async fn list_all_chats_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let rows = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE account_id = $1
            ORDER BY updated_at DESC, telegram_chat_id ASC
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }

    pub async fn list_chat_group_filters(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<TelegramChatGroupFilter>, TelegramError> {
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query_as::<_, (String, String, String, i64, String, Option<i64>)>(
            r#"
            SELECT id, label, source, count, icon, provider_folder_id
            FROM (
                SELECT
                    'local:all'::text AS id,
                    'All'::text AS label,
                    'local'::text AS source,
                    COUNT(*)::bigint AS count,
                    'tabler:message'::text AS icon,
                    NULL::bigint AS provider_folder_id
                FROM telegram_chats
                WHERE ($1::text IS NULL OR account_id = $1)

                UNION ALL

                SELECT
                    'folder:' || folder_label AS id,
                    folder_label AS label,
                    'telegram'::text AS source,
                    COUNT(*)::bigint AS count,
                    'tabler:folder'::text AS icon,
                    MIN(provider_folder_id)::bigint AS provider_folder_id
                FROM (
                    SELECT
                        telegram_chat_id,
                        NULLIF(BTRIM(folder_labels.value), '') AS folder_label,
                        COALESCE(
                            NULLIF(BTRIM(provider_folder_ids.value), '')::bigint,
                            NULLIF(BTRIM(metadata->>'provider_folder_id'), '')::bigint
                        ) AS provider_folder_id
                    FROM telegram_chats
                    LEFT JOIN LATERAL jsonb_array_elements_text(COALESCE(metadata->'folder_labels', '[]'::jsonb))
                        WITH ORDINALITY AS folder_labels(value, folder_index) ON true
                    LEFT JOIN LATERAL jsonb_array_elements_text(COALESCE(metadata->'provider_folder_ids', '[]'::jsonb))
                        WITH ORDINALITY AS provider_folder_ids(value, folder_index)
                        ON provider_folder_ids.folder_index = folder_labels.folder_index
                    WHERE ($1::text IS NULL OR account_id = $1)

                    UNION ALL

                    SELECT
                        telegram_chat_id,
                        NULLIF(BTRIM(metadata->>'folder_name'), '') AS folder_label,
                        NULLIF(BTRIM(metadata->>'provider_folder_id'), '')::bigint AS provider_folder_id
                    FROM telegram_chats
                    WHERE ($1::text IS NULL OR account_id = $1)
                      AND jsonb_array_length(COALESCE(metadata->'folder_labels', '[]'::jsonb)) = 0
                ) folder_rows
                WHERE folder_label IS NOT NULL
                GROUP BY folder_label
            ) filters
            ORDER BY
                CASE WHEN source = 'local' THEN 0 ELSE 1 END,
                label ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, label, source, count, icon, provider_folder_id)| TelegramChatGroupFilter {
                    id,
                    label,
                    source,
                    count,
                    icon,
                    provider_folder_id,
                },
            )
            .collect())
    }

    pub(crate) async fn apply_provider_chat_folders(
        &self,
        account_id: &str,
        folders: &[TelegramTdlibChatFolderSnapshot],
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let folder_map = folders
            .iter()
            .map(|folder| (folder.provider_folder_id, folder))
            .collect::<std::collections::HashMap<_, _>>();
        let chats = self.list_all_chats_for_account(account_id).await?;
        let mut updated = Vec::new();

        for chat in chats {
            let Some(chat_metadata) = chat.metadata.as_object() else {
                continue;
            };
            let folder_ids = chat_metadata
                .get("tdlib_chat_positions")
                .and_then(serde_json::Value::as_object)
                .and_then(|positions| positions.get("folder_ids"))
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            if folder_ids.is_empty() {
                continue;
            }

            let mut labels = Vec::new();
            let provider_folder_ids = folder_ids
                .into_iter()
                .filter_map(|value| value.as_i64())
                .collect::<Vec<_>>();
            let mut label_folder_ids = Vec::new();
            for folder_id in provider_folder_ids.iter().copied() {
                let Some(folder) = folder_map.get(&folder_id) else {
                    continue;
                };
                if labels.iter().all(|label: &String| label != &folder.title) {
                    labels.push(folder.title.clone());
                    label_folder_ids.push(folder_id);
                }
            }
            if labels.is_empty() {
                labels.extend(
                    provider_folder_ids
                        .iter()
                        .map(|folder_id| format!("Unknown folder {folder_id}")),
                );
                label_folder_ids = provider_folder_ids.clone();
            }

            let mut next_metadata = chat_metadata.clone();
            if labels.is_empty() {
                next_metadata.remove("folder_labels");
                next_metadata.remove("folder_name");
                next_metadata.remove("provider_folder_id");
                next_metadata.remove("provider_folder_ids");
            } else {
                next_metadata.insert(
                    "folder_labels".to_owned(),
                    serde_json::Value::Array(
                        labels
                            .iter()
                            .cloned()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                );
                next_metadata.insert(
                    "folder_name".to_owned(),
                    serde_json::Value::String(labels[0].clone()),
                );
                next_metadata.insert(
                    "provider_folder_ids".to_owned(),
                    serde_json::Value::Array(
                        label_folder_ids
                            .iter()
                            .copied()
                            .map(|value| serde_json::Value::Number(value.into()))
                            .collect(),
                    ),
                );
                if let Some(value) = label_folder_ids.first().copied() {
                    next_metadata.insert(
                        "provider_folder_id".to_owned(),
                        serde_json::Value::Number(value.into()),
                    );
                } else {
                    next_metadata.remove("provider_folder_id");
                }
            }
            if next_metadata == *chat_metadata {
                continue;
            }

            let metadata = self
                .persist_chat_metadata(
                    &chat.telegram_chat_id,
                    serde_json::Map::from_iter(next_metadata.into_iter()),
                )
                .await?;
            let mut refreshed = chat.clone();
            refreshed.metadata = metadata;
            updated.push(refreshed);
        }

        Ok(updated)
    }

    pub async fn telegram_chat_by_id(
        &self,
        telegram_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_chat).transpose()
    }

    pub(super) async fn chat_metadata_map(
        &self,
        telegram_chat_id: &str,
    ) -> Result<serde_json::Map<String, serde_json::Value>, TelegramError> {
        let row: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT metadata FROM telegram_chats WHERE telegram_chat_id = $1")
                .bind(telegram_chat_id)
                .fetch_optional(&self.pool)
                .await?;
        let metadata = row
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?
            .0;
        Ok(metadata.as_object().cloned().unwrap_or_default())
    }

    pub(super) async fn persist_chat_metadata(
        &self,
        telegram_chat_id: &str,
        metadata: serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, TelegramError> {
        let metadata = serde_json::Value::Object(metadata);
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE telegram_chats
            SET metadata = $2, updated_at = now()
            WHERE telegram_chat_id = $1
            RETURNING
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(telegram_chat_id)
        .bind(&metadata)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let stored = row_to_telegram_chat(row)?;
            capture_chat_observation_in_transaction(
                &mut transaction,
                &stored,
                "metadata_update",
                "telegram.client.chats.persist_chat_metadata",
                stored.updated_at,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(metadata)
    }

    pub async fn list_chat_members(
        &self,
        telegram_chat_id: &str,
        query: Option<&str>,
        role: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<Vec<TelegramChatMember>, TelegramError> {
        let limit = validate_chat_list_limit(limit)?;
        let offset = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.parse::<i64>())
            .transpose()
            .map_err(|_| {
                TelegramError::InvalidRequest("members cursor must be a numeric offset".to_owned())
            })?
            .unwrap_or(0);
        let chat = self
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;

        if super::participants::provider_roster_exists(&self.pool, telegram_chat_id).await? {
            return super::participants::list_provider_chat_members(
                &self.pool,
                telegram_chat_id,
                query,
                role,
                limit,
                offset,
            )
            .await;
        }

        super::participants::list_message_heuristic_members(
            self,
            &chat.account_id,
            &chat.provider_chat_id,
            query,
            role,
            limit,
            offset,
        )
        .await
    }

    pub(crate) async fn ingest_tdlib_chat_snapshot(
        &self,
        account_id: &str,
        snapshot: &TelegramTdlibChatSnapshot,
    ) -> Result<TelegramChat, TelegramError> {
        let provider_account = self.telegram_provider_account(account_id).await?;
        let raw_record_id = telegram_raw_record_id(
            &provider_account.account_id,
            TELEGRAM_CHAT_RECORD_KIND,
            &snapshot.provider_chat_id,
        );
        self.upsert_chat(&NewTelegramChat {
            account_id: provider_account.account_id,
            provider_chat_id: snapshot.provider_chat_id.clone(),
            chat_kind: snapshot.chat_kind,
            title: snapshot.title.clone(),
            username: snapshot.username.clone(),
            sync_state: TelegramSyncState::Synced,
            last_message_at: snapshot.last_message_at,
            metadata: tdlib_chat_projection_metadata(
                snapshot,
                &raw_record_id,
                &provider_account.external_account_id,
            ),
        })
        .await
    }
}
