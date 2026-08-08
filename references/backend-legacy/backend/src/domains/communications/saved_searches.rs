use makosh_events_api::NewEventEnvelope;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::communications::saved_search_counts::{
    count_messages_for_saved_search, load_message_counts_for_saved_searches,
};
use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::errors::ObservationStoreError;

const EVENT_TYPE_CREATED: &str = "mail.saved_search.created";
const EVENT_TYPE_UPDATED: &str = "mail.saved_search.updated";
const EVENT_TYPE_DELETED: &str = "mail.saved_search.deleted";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommunicationSavedSearch {
    pub saved_search_id: String,
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub query: String,
    pub workflow_state: Option<WorkflowState>,
    pub local_state: LocalMessageState,
    pub channel_kind: Option<String>,
    pub is_smart_folder: bool,
    pub sort_order: i32,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationSavedSearch {
    pub saved_search_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub workflow_state: Option<WorkflowState>,
    pub local_state: Option<LocalMessageState>,
    pub channel_kind: Option<String>,
    pub is_smart_folder: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateCommunicationSavedSearch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub workflow_state: Option<WorkflowState>,
    pub local_state: Option<LocalMessageState>,
    pub channel_kind: Option<String>,
    pub is_smart_folder: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationSavedSearchListPage {
    pub items: Vec<CommunicationSavedSearch>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct CommunicationSavedSearchListQuery<'a> {
    pub account_id: Option<&'a str>,
    pub is_smart_folder: Option<bool>,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone)]
pub struct CommunicationSavedSearchStore {
    pool: PgPool,
}

#[derive(Clone, Debug)]
pub(crate) struct SavedSearchRecord {
    pub(crate) saved_search_id: String,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) query: String,
    pub(crate) workflow_state: Option<WorkflowState>,
    pub(crate) local_state: LocalMessageState,
    pub(crate) channel_kind: Option<String>,
    pub(crate) is_smart_folder: bool,
    pub(crate) sort_order: i32,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl CommunicationSavedSearchStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        query: CommunicationSavedSearchListQuery<'_>,
    ) -> Result<CommunicationSavedSearchListPage, CommunicationSavedSearchError> {
        let limit = query.limit.clamp(1, 1000);
        let account_id = normalize_optional(query.account_id.map(str::to_owned))?;
        let cursor = query
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_saved_search_list_cursor)
            .transpose()?;
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                s.saved_search_id,
                s.name,
                s.description,
                s.account_id,
                s.query_text,
                s.workflow_state,
                s.local_state,
                s.channel_kind,
                s.is_smart_folder,
                s.sort_order,
                s.created_at,
                s.updated_at
            FROM communication_saved_searches s
            WHERE ($1::text IS NULL OR s.account_id = $1)
              AND ($2::boolean IS NULL OR s.is_smart_folder = $2)
              AND (
                $3::boolean IS NULL
                OR ($3 = TRUE AND s.is_smart_folder = FALSE)
                OR (
                  s.is_smart_folder = $3
                  AND (
                    s.sort_order > $4
                    OR (s.sort_order = $4 AND lower(s.name) > $5)
                    OR (s.sort_order = $4 AND lower(s.name) = $5 AND s.updated_at < $6)
                    OR (s.sort_order = $4 AND lower(s.name) = $5 AND s.updated_at = $6 AND s.saved_search_id > $7)
                  )
                )
              )
            ORDER BY s.is_smart_folder DESC, s.sort_order ASC, lower(s.name) ASC, s.updated_at DESC, s.saved_search_id ASC
            LIMIT $8
            "#,
        )
        .bind(account_id.as_deref())
        .bind(query.is_smart_folder)
        .bind(cursor.as_ref().map(|value| value.is_smart_folder))
        .bind(cursor.as_ref().map(|value| value.sort_order))
        .bind(cursor.as_ref().map(|value| value.name_lower.as_str()))
        .bind(cursor.as_ref().map(|value| value.updated_at))
        .bind(cursor.as_ref().map(|value| value.saved_search_id.as_str()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        let has_more = rows.len() > limit as usize;
        let records = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_saved_search_record)
            .collect::<Result<Vec<_>, _>>()?;
        let counts = load_message_counts_for_saved_searches(&self.pool, &records).await?;
        let items = records
            .into_iter()
            .map(|record| {
                let count = *counts.get(record.saved_search_id.as_str()).unwrap_or(&0);
                Ok(saved_search_from_record(record, count))
            })
            .collect::<Result<Vec<_>, CommunicationSavedSearchError>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(encode_saved_search_list_cursor)
                .transpose()?
        } else {
            None
        };

        Ok(CommunicationSavedSearchListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn create(
        &self,
        input: NewCommunicationSavedSearch,
    ) -> Result<CommunicationSavedSearch, CommunicationSavedSearchError> {
        self.create_with_observation(input, None, "saved_search_upsert", None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        input: NewCommunicationSavedSearch,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommunicationSavedSearch, CommunicationSavedSearchError> {
        let normalized = NormalizedMailSavedSearchInput::from_new(input)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let saved_search = insert_saved_search(&mut transaction, &normalized).await?;
        let event = saved_search_event(EVENT_TYPE_CREATED, &saved_search)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "saved_search_create",
                    "name": saved_search.name,
                    "is_smart_folder": saved_search.is_smart_folder,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "saved_search",
                saved_search.saved_search_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(saved_search)
    }

    pub async fn update(
        &self,
        saved_search_id: &str,
        update: UpdateCommunicationSavedSearch,
    ) -> Result<Option<CommunicationSavedSearch>, CommunicationSavedSearchError> {
        self.update_with_observation(saved_search_id, update, None, "saved_search_upsert", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        saved_search_id: &str,
        update: UpdateCommunicationSavedSearch,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<CommunicationSavedSearch>, CommunicationSavedSearchError> {
        let saved_search_id = normalize_required("saved_search_id", saved_search_id)?;
        let normalized = NormalizedMailSavedSearchUpdate::from_update(update)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let Some(saved_search) =
            update_saved_search(&mut transaction, &saved_search_id, &normalized).await?
        else {
            transaction.rollback().await?;
            return Ok(None);
        };
        let event = saved_search_event(EVENT_TYPE_UPDATED, &saved_search)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "saved_search_update",
                    "name": saved_search.name,
                    "is_smart_folder": saved_search.is_smart_folder,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "saved_search",
                saved_search.saved_search_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(Some(saved_search))
    }

    pub async fn delete(
        &self,
        saved_search_id: &str,
    ) -> Result<bool, CommunicationSavedSearchError> {
        self.delete_with_observation(saved_search_id, None, "saved_search_delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        saved_search_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<bool, CommunicationSavedSearchError> {
        let saved_search_id = normalize_required("saved_search_id", saved_search_id)?;
        let mut transaction = self.pool.begin().await?;
        let Some(saved_search) = delete_saved_search(&mut transaction, &saved_search_id).await?
        else {
            transaction.rollback().await?;
            return Ok(false);
        };
        let event = saved_search_event(EVENT_TYPE_DELETED, &saved_search)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "saved_search_delete",
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "saved_search",
                saved_search.saved_search_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(true)
    }
}

#[derive(Debug)]
struct NormalizedMailSavedSearchInput {
    saved_search_id: String,
    name: String,
    description: Option<String>,
    account_id: Option<String>,
    query: String,
    workflow_state: Option<WorkflowState>,
    local_state: LocalMessageState,
    channel_kind: Option<String>,
    is_smart_folder: bool,
    sort_order: i32,
}

impl NormalizedMailSavedSearchInput {
    fn from_new(input: NewCommunicationSavedSearch) -> Result<Self, CommunicationSavedSearchError> {
        let saved_search_id = match input.saved_search_id {
            Some(value) => normalize_required("saved_search_id", &value)?,
            None => generate_saved_search_id(),
        };
        let normalized = Self {
            saved_search_id,
            name: normalize_required("name", &input.name)?,
            description: normalize_optional(input.description)?,
            account_id: normalize_optional(input.account_id)?,
            query: normalize_optional(input.query)?.unwrap_or_default(),
            workflow_state: input.workflow_state,
            local_state: input.local_state.unwrap_or(LocalMessageState::Active),
            channel_kind: normalize_optional(input.channel_kind)?,
            is_smart_folder: input.is_smart_folder.unwrap_or(false),
            sort_order: input.sort_order.unwrap_or(0),
        };
        validate_has_filter(
            &normalized.query,
            normalized.workflow_state,
            normalized.local_state,
            normalized.channel_kind.as_deref(),
            normalized.account_id.as_deref(),
        )?;
        Ok(normalized)
    }
}

#[derive(Debug)]
struct NormalizedMailSavedSearchUpdate {
    name: Option<String>,
    description: Option<String>,
    account_id: Option<String>,
    query: Option<String>,
    workflow_state: Option<WorkflowState>,
    local_state: Option<LocalMessageState>,
    channel_kind: Option<String>,
    is_smart_folder: Option<bool>,
    sort_order: Option<i32>,
}

impl NormalizedMailSavedSearchUpdate {
    fn from_update(
        update: UpdateCommunicationSavedSearch,
    ) -> Result<Self, CommunicationSavedSearchError> {
        Ok(Self {
            name: normalize_optional(update.name)?,
            description: normalize_optional(update.description)?,
            account_id: normalize_optional(update.account_id)?,
            query: update.query.map(|value| value.trim().to_owned()),
            workflow_state: update.workflow_state,
            local_state: update.local_state,
            channel_kind: normalize_optional(update.channel_kind)?,
            is_smart_folder: update.is_smart_folder,
            sort_order: update.sort_order,
        })
    }
}

async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationSavedSearchError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn insert_saved_search(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    input: &NormalizedMailSavedSearchInput,
) -> Result<CommunicationSavedSearch, CommunicationSavedSearchError> {
    let row = sqlx::query(
        r#"
        WITH inserted AS (
            INSERT INTO communication_saved_searches (
                saved_search_id, name, description, account_id, query_text, workflow_state,
                local_state, channel_kind, is_smart_folder, sort_order
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING saved_search_id, name, description, account_id, query_text, workflow_state,
                      local_state, channel_kind, is_smart_folder, sort_order, created_at, updated_at
        )
        SELECT
            s.saved_search_id,
            s.name,
            s.description,
            s.account_id,
            s.query_text,
            s.workflow_state,
            s.local_state,
            s.channel_kind,
            s.is_smart_folder,
            s.sort_order,
            s.created_at,
            s.updated_at
        FROM inserted s
        "#,
    )
    .bind(&input.saved_search_id)
    .bind(&input.name)
    .bind(input.description.as_deref())
    .bind(input.account_id.as_deref())
    .bind(&input.query)
    .bind(input.workflow_state.map(|state| state.as_str()))
    .bind(input.local_state.as_str())
    .bind(input.channel_kind.as_deref())
    .bind(input.is_smart_folder)
    .bind(input.sort_order)
    .fetch_one(&mut **transaction)
    .await?;
    let record = row_to_saved_search_record(row)?;
    let message_count = count_messages_for_saved_search(&mut **transaction, &record).await?;
    Ok(saved_search_from_record(record, message_count))
}

async fn update_saved_search(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    saved_search_id: &str,
    update: &NormalizedMailSavedSearchUpdate,
) -> Result<Option<CommunicationSavedSearch>, CommunicationSavedSearchError> {
    let row = sqlx::query(
        r#"
        WITH updated AS (
            UPDATE communication_saved_searches
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                account_id = COALESCE($4, account_id),
                query_text = COALESCE($5, query_text),
                workflow_state = COALESCE($6, workflow_state),
                local_state = COALESCE($7, local_state),
                channel_kind = COALESCE($8, channel_kind),
                is_smart_folder = COALESCE($9, is_smart_folder),
                sort_order = COALESCE($10, sort_order),
                updated_at = now()
            WHERE saved_search_id = $1
            RETURNING saved_search_id, name, description, account_id, query_text, workflow_state,
                      local_state, channel_kind, is_smart_folder, sort_order, created_at, updated_at
        )
        SELECT
            s.saved_search_id,
            s.name,
            s.description,
            s.account_id,
            s.query_text,
            s.workflow_state,
            s.local_state,
            s.channel_kind,
            s.is_smart_folder,
            s.sort_order,
            s.created_at,
            s.updated_at
        FROM updated s
        "#,
    )
    .bind(saved_search_id)
    .bind(update.name.as_deref())
    .bind(update.description.as_deref())
    .bind(update.account_id.as_deref())
    .bind(update.query.as_deref())
    .bind(update.workflow_state.map(|state| state.as_str()))
    .bind(update.local_state.map(|state| state.as_str()))
    .bind(update.channel_kind.as_deref())
    .bind(update.is_smart_folder)
    .bind(update.sort_order)
    .fetch_optional(&mut **transaction)
    .await?;
    let row = row.map(row_to_saved_search_record).transpose()?;
    match row {
        Some(record) => {
            let message_count =
                count_messages_for_saved_search(&mut **transaction, &record).await?;
            Ok(Some(saved_search_from_record(record, message_count)))
        }
        None => Ok(None),
    }
}

async fn delete_saved_search(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    saved_search_id: &str,
) -> Result<Option<CommunicationSavedSearch>, CommunicationSavedSearchError> {
    let row = sqlx::query(
        r#"
        WITH deleted AS (
            DELETE FROM communication_saved_searches
            WHERE saved_search_id = $1
            RETURNING saved_search_id, name, description, account_id, query_text, workflow_state,
                      local_state, channel_kind, is_smart_folder, sort_order, created_at, updated_at
        )
        SELECT
            s.saved_search_id,
            s.name,
            s.description,
            s.account_id,
            s.query_text,
            s.workflow_state,
            s.local_state,
            s.channel_kind,
            s.is_smart_folder,
            s.sort_order,
            s.created_at,
            s.updated_at
        FROM deleted s
        "#,
    )
    .bind(saved_search_id)
    .fetch_optional(&mut **transaction)
    .await?;
    let row = row.map(row_to_saved_search_record).transpose()?;
    match row {
        Some(record) => {
            let message_count =
                count_messages_for_saved_search(&mut **transaction, &record).await?;
            Ok(Some(saved_search_from_record(record, message_count)))
        }
        None => Ok(None),
    }
}

fn saved_search_event(
    event_type: &str,
    saved_search: &CommunicationSavedSearch,
) -> Result<NewEventEnvelope, CommunicationSavedSearchError> {
    Ok(NewEventEnvelope::builder(
        generate_saved_search_event_id(event_type, &saved_search.saved_search_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_saved_search_api" }),
        json!({
            "kind": "mail_saved_search",
            "id": saved_search.saved_search_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(serde_json::to_value(saved_search)?)
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": saved_search.saved_search_id,
    }))
    .correlation_id(saved_search.saved_search_id.clone())
    .build()?)
}

#[derive(Debug, Deserialize, Serialize)]
struct SavedSearchListCursor {
    is_smart_folder: bool,
    sort_order: i32,
    name_lower: String,
    updated_at: DateTime<Utc>,
    saved_search_id: String,
}

fn encode_saved_search_list_cursor(
    saved_search: &CommunicationSavedSearch,
) -> Result<String, CommunicationSavedSearchError> {
    let cursor = SavedSearchListCursor {
        is_smart_folder: saved_search.is_smart_folder,
        sort_order: saved_search.sort_order,
        name_lower: saved_search.name.to_lowercase(),
        updated_at: saved_search.updated_at,
        saved_search_id: saved_search.saved_search_id.clone(),
    };
    let bytes =
        serde_json::to_vec(&cursor).map_err(|_| CommunicationSavedSearchError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_saved_search_list_cursor(
    cursor: &str,
) -> Result<SavedSearchListCursor, CommunicationSavedSearchError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationSavedSearchError::InvalidCursor)?;
    let cursor: SavedSearchListCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationSavedSearchError::InvalidCursor)?;
    if cursor.name_lower.trim().is_empty() || cursor.saved_search_id.trim().is_empty() {
        return Err(CommunicationSavedSearchError::InvalidCursor);
    }
    Ok(cursor)
}

fn row_to_saved_search_record(
    row: PgRow,
) -> Result<SavedSearchRecord, CommunicationSavedSearchError> {
    let workflow_state: Option<String> = row.try_get("workflow_state")?;
    let local_state: String = row.try_get("local_state")?;
    Ok(SavedSearchRecord {
        saved_search_id: row.try_get("saved_search_id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        account_id: row.try_get("account_id")?,
        query: row.try_get("query_text")?,
        workflow_state: workflow_state
            .as_deref()
            .map(str::parse::<WorkflowState>)
            .transpose()
            .map_err(|_| CommunicationSavedSearchError::Invalid("invalid workflow_state"))?,
        local_state: local_state
            .parse::<LocalMessageState>()
            .map_err(|_| CommunicationSavedSearchError::Invalid("invalid local_state"))?,
        channel_kind: row.try_get("channel_kind")?,
        is_smart_folder: row.try_get("is_smart_folder")?,
        sort_order: row.try_get("sort_order")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn saved_search_from_record(
    record: SavedSearchRecord,
    message_count: i64,
) -> CommunicationSavedSearch {
    CommunicationSavedSearch {
        saved_search_id: record.saved_search_id,
        name: record.name,
        description: record.description,
        account_id: record.account_id,
        query: record.query,
        workflow_state: record.workflow_state,
        local_state: record.local_state,
        channel_kind: record.channel_kind,
        is_smart_folder: record.is_smart_folder,
        sort_order: record.sort_order,
        message_count,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn normalize_required(
    field: &'static str,
    value: &str,
) -> Result<String, CommunicationSavedSearchError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CommunicationSavedSearchError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(
    value: Option<String>,
) -> Result<Option<String>, CommunicationSavedSearchError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn validate_has_filter(
    query: &str,
    workflow_state: Option<WorkflowState>,
    local_state: LocalMessageState,
    channel_kind: Option<&str>,
    account_id: Option<&str>,
) -> Result<(), CommunicationSavedSearchError> {
    if query.trim().is_empty()
        && workflow_state.is_none()
        && channel_kind.is_none()
        && account_id.is_none()
        && local_state == LocalMessageState::Active
    {
        return Err(CommunicationSavedSearchError::Invalid(
            "saved search filter",
        ));
    }
    Ok(())
}

fn generate_saved_search_id() -> String {
    format!("mail_saved_search:{:x}", system_time_nanos())
}

fn generate_saved_search_event_id(event_type: &str, saved_search_id: &str) -> String {
    format!(
        "mail_saved_search_event:{event_type}:{saved_search_id}:{:x}",
        system_time_nanos()
    )
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[derive(Debug, Error)]
pub enum CommunicationSavedSearchError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error("invalid mail saved search cursor")]
    InvalidCursor,
    #[error("invalid mail saved search field: {0}")]
    Invalid(&'static str),
}
