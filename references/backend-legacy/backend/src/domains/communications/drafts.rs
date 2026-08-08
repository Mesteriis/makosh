use makosh_events_api::NewEventEnvelope;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::errors::ObservationStoreError;

const EVENT_TYPE_DRAFT_CREATED: &str = "mail.draft.created";
const EVENT_TYPE_DRAFT_UPDATED: &str = "mail.draft.updated";
const EVENT_TYPE_DRAFT_DELETED: &str = "mail.draft.deleted";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationDraft {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub attachment_ids: Vec<String>,
    pub attachments: Vec<CommunicationDraftAttachmentRef>,
    pub status: DraftStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub send_attempts: i32,
    pub last_error: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommunicationDraftAttachmentRef {
    pub attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub scan_status: String,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationDraftListPage {
    pub items: Vec<CommunicationDraft>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Failed,
}

impl DraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DraftStatus::Draft => "draft",
            DraftStatus::Scheduled => "scheduled",
            DraftStatus::Sending => "sending",
            DraftStatus::Sent => "sent",
            DraftStatus::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "draft" => Some(DraftStatus::Draft),
            "scheduled" => Some(DraftStatus::Scheduled),
            "sending" => Some(DraftStatus::Sending),
            "sent" => Some(DraftStatus::Sent),
            "failed" => Some(DraftStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewCommunicationDraft {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub attachment_ids: Option<Vec<String>>,
    pub status: DraftStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewCommunicationDraft {
    fn validate(&self) -> Result<(), CommunicationDraftError> {
        if self.draft_id.trim().is_empty() {
            return Err(CommunicationDraftError::Invalid("draft_id empty"));
        }
        if self.account_id.trim().is_empty() {
            return Err(CommunicationDraftError::Invalid("account_id empty"));
        }
        if let Some(attachment_ids) = &self.attachment_ids {
            validate_attachment_ids(attachment_ids)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CommunicationDraftStore {
    pool: PgPool,
}

impl CommunicationDraftStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        draft: &NewCommunicationDraft,
    ) -> Result<CommunicationDraft, CommunicationDraftError> {
        self.upsert_with_observation(draft, None, "draft_upsert", None)
            .await
    }

    pub async fn upsert_with_observation(
        &self,
        draft: &NewCommunicationDraft,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationDraft, CommunicationDraftError> {
        draft.validate()?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, Some(draft.account_id.as_str()))
            .await?;
        let existed = draft_exists(&mut transaction, &draft.draft_id).await?;
        let row = sqlx::query(
            r#"INSERT INTO communication_drafts (draft_id, account_id, identity_id, to_participants, cc_participants, bcc_participants, subject, body_text, body_html, in_reply_to, message_refs, status, scheduled_send_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (draft_id) DO UPDATE SET
                account_id = EXCLUDED.account_id, identity_id = EXCLUDED.identity_id,
                to_participants = EXCLUDED.to_participants, cc_participants = EXCLUDED.cc_participants,
                bcc_participants = EXCLUDED.bcc_participants, subject = EXCLUDED.subject,
                body_text = EXCLUDED.body_text, body_html = EXCLUDED.body_html,
                in_reply_to = EXCLUDED.in_reply_to, message_refs = EXCLUDED.message_refs,
                status = EXCLUDED.status, scheduled_send_at = EXCLUDED.scheduled_send_at,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at"#,
        )
        .bind(&draft.draft_id).bind(&draft.account_id).bind(draft.persona_id.as_deref())
        .bind(serde_json::to_value(&draft.to_recipients).unwrap_or_default())
        .bind(serde_json::to_value(&draft.cc_recipients).unwrap_or_default())
        .bind(serde_json::to_value(&draft.bcc_recipients).unwrap_or_default())
        .bind(&draft.subject).bind(&draft.body_text).bind(draft.body_html.as_deref())
        .bind(draft.in_reply_to.as_deref())
        .bind(serde_json::to_value(&draft.references).unwrap_or_default())
        .bind(draft.status.as_str()).bind(draft.scheduled_send_at).bind(&draft.metadata)
        .fetch_one(&mut *transaction).await?;
        if let Some(attachment_ids) = &draft.attachment_ids {
            sync_draft_attachments_in_transaction(
                &mut transaction,
                &draft.draft_id,
                &draft.account_id,
                attachment_ids,
            )
            .await?;
        }
        let mut draft = row_to_draft(row)?;
        draft.attachment_ids =
            draft_attachment_ids_in_transaction(&mut transaction, &draft.draft_id).await?;
        draft.attachments =
            draft_attachment_refs_in_transaction(&mut transaction, &draft.draft_id).await?;
        let event_type = if existed {
            EVENT_TYPE_DRAFT_UPDATED
        } else {
            EVENT_TYPE_DRAFT_CREATED
        };
        let event = draft_event(event_type, &draft)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "status": draft.status.as_str(),
                    "operation": if existed { "draft_update" } else { "draft_create" },
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "draft",
                draft.draft_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(draft)
    }

    pub async fn list(
        &self,
        account_id: Option<&str>,
        status: Option<DraftStatus>,
    ) -> Result<Vec<CommunicationDraft>, CommunicationDraftError> {
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                ARRAY(
                    SELECT attachment.attachment_id
                    FROM communication_draft_attachments attachment
                    WHERE attachment.draft_id = communication_drafts.draft_id
                    ORDER BY attachment.sort_order
                ) AS attachment_ids,
                (
                    SELECT COALESCE(jsonb_agg(jsonb_build_object(
                        'attachment_id', attachment.attachment_id,
                        'filename', imported.filename,
                        'content_type', imported.content_type,
                        'size_bytes', imported.size_bytes,
                        'scan_status', imported.scan_status,
                        'scan_engine', imported.scan_engine,
                        'scan_checked_at', imported.scan_checked_at,
                        'scan_summary', imported.scan_summary
                    ) ORDER BY attachment.sort_order), '[]'::jsonb)
                    FROM communication_draft_attachments attachment
                    JOIN communication_attachment_imports imported
                      ON imported.attachment_id = attachment.attachment_id
                    WHERE attachment.draft_id = communication_drafts.draft_id
                ) AS attachments,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at
            FROM communication_drafts
            WHERE ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR status = $2)
            ORDER BY updated_at DESC, draft_id ASC"#,
        )
        .bind(account_id)
        .bind(status_str.as_deref())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_draft).collect()
    }

    pub async fn list_page(
        &self,
        account_id: Option<&str>,
        status: Option<DraftStatus>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<CommunicationDraftListPage, CommunicationDraftError> {
        let limit = validate_limit(limit)?;
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_draft_list_cursor)
            .transpose()?;
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                ARRAY(
                    SELECT attachment.attachment_id
                    FROM communication_draft_attachments attachment
                    WHERE attachment.draft_id = communication_drafts.draft_id
                    ORDER BY attachment.sort_order
                ) AS attachment_ids,
                (
                    SELECT COALESCE(jsonb_agg(jsonb_build_object(
                        'attachment_id', attachment.attachment_id,
                        'filename', imported.filename,
                        'content_type', imported.content_type,
                        'size_bytes', imported.size_bytes,
                        'scan_status', imported.scan_status,
                        'scan_engine', imported.scan_engine,
                        'scan_checked_at', imported.scan_checked_at,
                        'scan_summary', imported.scan_summary
                    ) ORDER BY attachment.sort_order), '[]'::jsonb)
                    FROM communication_draft_attachments attachment
                    JOIN communication_attachment_imports imported
                      ON imported.attachment_id = attachment.attachment_id
                    WHERE attachment.draft_id = communication_drafts.draft_id
                ) AS attachments,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at
            FROM communication_drafts
            WHERE ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR status = $2)
              AND (
                $3::timestamptz IS NULL
                OR updated_at < $3
                OR (updated_at = $3 AND draft_id > $4)
              )
            ORDER BY updated_at DESC, draft_id ASC
            LIMIT $5"#,
        )
        .bind(account_id)
        .bind(status_str.as_deref())
        .bind(cursor.as_ref().map(|value| value.updated_at))
        .bind(cursor.as_ref().map(|value| value.draft_id.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;
        let mut items = rows
            .into_iter()
            .map(row_to_draft)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > limit as usize;
        if has_more {
            items.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            items.last().map(encode_draft_list_cursor).transpose()?
        } else {
            None
        };
        Ok(CommunicationDraftListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn get(
        &self,
        draft_id: &str,
    ) -> Result<Option<CommunicationDraft>, CommunicationDraftError> {
        let row = sqlx::query(
            r#"SELECT
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                ARRAY(
                    SELECT attachment.attachment_id
                    FROM communication_draft_attachments attachment
                    WHERE attachment.draft_id = communication_drafts.draft_id
                    ORDER BY attachment.sort_order
                ) AS attachment_ids,
                (
                    SELECT COALESCE(jsonb_agg(jsonb_build_object(
                        'attachment_id', attachment.attachment_id,
                        'filename', imported.filename,
                        'content_type', imported.content_type,
                        'size_bytes', imported.size_bytes,
                        'scan_status', imported.scan_status,
                        'scan_engine', imported.scan_engine,
                        'scan_checked_at', imported.scan_checked_at,
                        'scan_summary', imported.scan_summary
                    ) ORDER BY attachment.sort_order), '[]'::jsonb)
                    FROM communication_draft_attachments attachment
                    JOIN communication_attachment_imports imported
                      ON imported.attachment_id = attachment.attachment_id
                    WHERE attachment.draft_id = communication_drafts.draft_id
                ) AS attachments,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at
            FROM communication_drafts
            WHERE draft_id = $1"#,
        )
        .bind(draft_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_draft).transpose()
    }

    pub async fn delete(&self, draft_id: &str) -> Result<bool, CommunicationDraftError> {
        self.delete_with_observation(draft_id, None, "draft_delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        draft_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<bool, CommunicationDraftError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"DELETE FROM communication_drafts
            WHERE draft_id = $1
            RETURNING
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                ARRAY[]::text[] AS attachment_ids,
                '[]'::jsonb AS attachments,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at"#,
        )
        .bind(draft_id)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            transaction.rollback().await?;
            return Ok(false);
        };
        let draft = row_to_draft(row)?;
        let event = draft_event(EVENT_TYPE_DRAFT_DELETED, &draft)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "draft_delete",
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "draft",
                draft.draft_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(true)
    }

    pub async fn mark_stale_drafts(
        &self,
        older_than_days: i32,
    ) -> Result<u64, CommunicationDraftError> {
        let result = sqlx::query(
            "UPDATE communication_drafts SET status = 'failed', last_error = 'stale draft expired', updated_at = now() WHERE status = 'draft' AND updated_at < now() - ($1 || ' days')::interval",
        ).bind(older_than_days).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}

async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationDraftError> {
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

async fn draft_exists(
    transaction: &mut Transaction<'_, Postgres>,
    draft_id: &str,
) -> Result<bool, CommunicationDraftError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM communication_drafts WHERE draft_id = $1)",
    )
    .bind(draft_id)
    .fetch_one(&mut **transaction)
    .await?)
}

fn draft_event(
    event_type: &str,
    draft: &CommunicationDraft,
) -> Result<NewEventEnvelope, CommunicationDraftError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_draft_event:{}:{}:{:x}",
            event_type,
            draft.draft_id,
            system_time_nanos()
        ),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_draft_api" }),
        json!({
            "kind": "mail_draft",
            "id": draft.draft_id,
            "account_id": draft.account_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(json!({
        "draft_id": draft.draft_id,
        "account_id": draft.account_id,
        "status": draft.status.as_str(),
        "scheduled_send_at": draft.scheduled_send_at,
        "has_body_text": !draft.body_text.trim().is_empty(),
        "has_body_html": draft
            .body_html
            .as_deref()
            .is_some_and(|body_html| !body_html.trim().is_empty()),
        "to_recipient_count": draft.to_recipients.len(),
        "cc_recipient_count": draft.cc_recipients.len(),
        "bcc_recipient_count": draft.bcc_recipients.len(),
        "in_reply_to_present": draft.in_reply_to.is_some(),
        "reference_count": draft.references.len(),
    }))
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": draft.draft_id,
    }))
    .correlation_id(draft.draft_id.clone())
    .build()?)
}

fn row_to_draft(row: PgRow) -> Result<CommunicationDraft, CommunicationDraftError> {
    let status_str: String = row.try_get("status")?;
    Ok(CommunicationDraft {
        draft_id: row.try_get("draft_id")?,
        account_id: row.try_get("account_id")?,
        persona_id: row.try_get("persona_id")?,
        to_recipients: serde_json::from_value(row.try_get("to_recipients")?).unwrap_or_default(),
        cc_recipients: serde_json::from_value(row.try_get("cc_recipients")?).unwrap_or_default(),
        bcc_recipients: serde_json::from_value(row.try_get("bcc_recipients")?).unwrap_or_default(),
        subject: row.try_get("subject")?,
        body_text: row.try_get("body_text")?,
        body_html: row.try_get("body_html")?,
        in_reply_to: row.try_get("in_reply_to")?,
        references: serde_json::from_value(row.try_get("message_references")?).unwrap_or_default(),
        attachment_ids: row.try_get("attachment_ids").unwrap_or_default(),
        attachments: row
            .try_get::<Value, _>("attachments")
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_default(),
        status: DraftStatus::parse(&status_str).unwrap_or(DraftStatus::Draft),
        scheduled_send_at: row.try_get("scheduled_send_at")?,
        send_attempts: row.try_get("send_attempts")?,
        last_error: row.try_get("last_error")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

const MAX_DRAFT_ATTACHMENTS: usize = 25;

fn validate_attachment_ids(attachment_ids: &[String]) -> Result<(), CommunicationDraftError> {
    if attachment_ids.len() > MAX_DRAFT_ATTACHMENTS {
        return Err(CommunicationDraftError::Invalid(
            "draft attachment count exceeds 25",
        ));
    }
    let mut unique = std::collections::HashSet::with_capacity(attachment_ids.len());
    for attachment_id in attachment_ids {
        let attachment_id = attachment_id.trim();
        if attachment_id.is_empty() {
            return Err(CommunicationDraftError::Invalid("attachment_id empty"));
        }
        if !unique.insert(attachment_id) {
            return Err(CommunicationDraftError::Invalid(
                "duplicate draft attachment_id",
            ));
        }
    }
    Ok(())
}

async fn sync_draft_attachments_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    draft_id: &str,
    account_id: &str,
    attachment_ids: &[String],
) -> Result<(), CommunicationDraftError> {
    validate_attachment_ids(attachment_ids)?;
    sqlx::query("DELETE FROM communication_draft_attachments WHERE draft_id = $1")
        .bind(draft_id)
        .execute(&mut **transaction)
        .await?;
    for (sort_order, attachment_id) in attachment_ids.iter().enumerate() {
        let inserted = sqlx::query(
            r#"
            INSERT INTO communication_draft_attachments (
                draft_id, attachment_id, disposition, sort_order
            )
            SELECT $1, attachment.attachment_id, 'attachment', $4
            FROM communication_attachment_imports attachment
            WHERE attachment.attachment_id = $2
              AND (attachment.account_id IS NULL OR attachment.account_id = $3)
              AND (attachment.channel_kind IS NULL OR attachment.channel_kind = 'mail')
            "#,
        )
        .bind(draft_id)
        .bind(attachment_id.trim())
        .bind(account_id)
        .bind(i32::try_from(sort_order).map_err(|_| {
            CommunicationDraftError::Invalid("draft attachment sort order overflow")
        })?)
        .execute(&mut **transaction)
        .await?;
        if inserted.rows_affected() != 1 {
            return Err(CommunicationDraftError::Invalid(
                "attachment is unavailable for this mail account",
            ));
        }
    }
    Ok(())
}

async fn draft_attachment_ids_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    draft_id: &str,
) -> Result<Vec<String>, CommunicationDraftError> {
    Ok(sqlx::query_scalar(
        r#"
        SELECT attachment_id
        FROM communication_draft_attachments
        WHERE draft_id = $1
        ORDER BY sort_order
        "#,
    )
    .bind(draft_id)
    .fetch_all(&mut **transaction)
    .await?)
}

async fn draft_attachment_refs_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    draft_id: &str,
) -> Result<Vec<CommunicationDraftAttachmentRef>, CommunicationDraftError> {
    let value = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(jsonb_build_object(
            'attachment_id', attachment.attachment_id,
            'filename', imported.filename,
            'content_type', imported.content_type,
            'size_bytes', imported.size_bytes,
            'scan_status', imported.scan_status,
            'scan_engine', imported.scan_engine,
            'scan_checked_at', imported.scan_checked_at,
            'scan_summary', imported.scan_summary
        ) ORDER BY attachment.sort_order), '[]'::jsonb)
        FROM communication_draft_attachments attachment
        JOIN communication_attachment_imports imported
          ON imported.attachment_id = attachment.attachment_id
        WHERE attachment.draft_id = $1
        "#,
    )
    .bind(draft_id)
    .fetch_one(&mut **transaction)
    .await?;
    Ok(serde_json::from_value(value)?)
}

#[derive(Debug, Error)]
pub enum CommunicationDraftError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid draft: {0}")]
    Invalid(&'static str),
    #[error("invalid draft cursor")]
    InvalidCursor,
}

fn validate_limit(limit: i64) -> Result<i64, CommunicationDraftError> {
    if !(1..=500).contains(&limit) {
        return Err(CommunicationDraftError::Invalid(
            "limit must be between 1 and 500",
        ));
    }
    Ok(limit)
}

#[derive(Debug, Deserialize, Serialize)]
struct DraftListCursor {
    updated_at: DateTime<Utc>,
    draft_id: String,
}

fn encode_draft_list_cursor(draft: &CommunicationDraft) -> Result<String, CommunicationDraftError> {
    let cursor = DraftListCursor {
        updated_at: draft.updated_at,
        draft_id: draft.draft_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationDraftError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_draft_list_cursor(cursor: &str) -> Result<DraftListCursor, CommunicationDraftError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationDraftError::InvalidCursor)?;
    let cursor: DraftListCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationDraftError::InvalidCursor)?;
    if cursor.draft_id.trim().is_empty() {
        return Err(CommunicationDraftError::InvalidCursor);
    }
    Ok(cursor)
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn draft_validation_allows_empty_subject_for_autosave() {
        let draft = NewCommunicationDraft {
            draft_id: "draft-autosave".to_owned(),
            account_id: "imap-primary".to_owned(),
            persona_id: None,
            to_recipients: Vec::new(),
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: String::new(),
            body_text: "body typed before subject".to_owned(),
            body_html: None,
            in_reply_to: None,
            references: Vec::new(),
            attachment_ids: None,
            status: DraftStatus::Draft,
            scheduled_send_at: None,
            metadata: json!({ "compose_mode": "compose" }),
        };

        draft.validate().expect("empty subject draft is valid");
    }
}
