use self::delivery::OutboxSendReceipt;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, PgRow, Postgres};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use makosh_events_postgres::store::EventStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

pub mod attachments;
pub mod delivery;
pub mod delivery_status;
pub mod provider_send_store;
pub mod provider_sender;
pub mod smtp_sender;

pub fn rfc822_message_id_for_outbox(outbox_id: &str) -> String {
    let digest = Sha256::digest(outbox_id.trim().as_bytes());
    format!("<makosh-outbox-{digest:x}@local.invalid>")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationOutboxStatus {
    Queued,
    Scheduled,
    Sending,
    Sent,
    Failed,
    Canceled,
}

impl CommunicationOutboxStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Scheduled => "scheduled",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "queued" => Some(Self::Queued),
            "scheduled" => Some(Self::Scheduled),
            "sending" => Some(Self::Sending),
            "sent" => Some(Self::Sent),
            "failed" => Some(Self::Failed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommunicationOutboxItem {
    pub outbox_id: String,
    pub account_id: String,
    pub draft_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: CommunicationOutboxStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_deadline_at: Option<DateTime<Utc>>,
    pub send_attempts: i32,
    pub claimed_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub provider_message_id: Option<String>,
    pub last_error: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailOutboxListPage {
    pub items: Vec<CommunicationOutboxItem>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct NewCommunicationOutboxItem {
    pub outbox_id: String,
    pub account_id: String,
    pub draft_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: CommunicationOutboxStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_deadline_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewCommunicationOutboxItem {
    fn validate(&self) -> Result<(), CommunicationOutboxError> {
        validate_non_empty("outbox_id", &self.outbox_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        if self
            .to_recipients
            .iter()
            .chain(self.cc_recipients.iter())
            .chain(self.bcc_recipients.iter())
            .all(|recipient| recipient.trim().is_empty())
        {
            return Err(CommunicationOutboxError::Invalid(
                "at least one recipient is required",
            ));
        }
        if !self.metadata.is_object() {
            return Err(CommunicationOutboxError::Invalid(
                "metadata must be a JSON object",
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct CommunicationOutboxStore {
    pool: PgPool,
}

impl CommunicationOutboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        outbox_id: &str,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        let row = sqlx::query(
            r#"
            SELECT
                outbox.outbox_id,
                outbox.account_id,
                outbox.draft_id,
                outbox.to_participants AS to_recipients,
                outbox.cc_participants AS cc_recipients,
                outbox.bcc_participants AS bcc_recipients,
                outbox.subject,
                outbox.body_text,
                outbox.body_html,
                outbox.status,
                outbox.scheduled_send_at,
                outbox.undo_deadline_at,
                outbox.send_attempts,
                outbox.claimed_at,
                outbox.sent_at,
                outbox.provider_message_id,
                outbox.last_error,
                CASE
                    WHEN latest_receipt.latest_read_receipt IS NULL THEN outbox.metadata
                    ELSE jsonb_set(
                        outbox.metadata,
                        '{latest_read_receipt}',
                        latest_receipt.latest_read_receipt,
                        true
                    )
                END AS metadata,
                outbox.created_at,
                outbox.updated_at
            FROM communication_outbox outbox
            LEFT JOIN LATERAL (
                SELECT jsonb_build_object(
                    'receipt_kind', receipt.receipt_kind,
                    'read_at', receipt.read_at,
                    'source_kind', receipt.source_kind
                ) AS latest_read_receipt
                FROM communication_read_receipts receipt
                WHERE receipt.outbox_id = outbox.outbox_id
                ORDER BY receipt.read_at DESC, receipt.receipt_id ASC
                LIMIT 1
            ) latest_receipt ON true
            WHERE outbox.outbox_id = $1
            "#,
        )
        .bind(outbox_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_outbox_item).transpose()
    }

    pub async fn enqueue(
        &self,
        item: &NewCommunicationOutboxItem,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        self.enqueue_with_observation(item, None, "outbox_status_transition", None)
            .await
    }

    pub async fn enqueue_with_observation(
        &self,
        item: &NewCommunicationOutboxItem,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        let mut transaction = self.pool.begin().await?;
        let outbox_item = enqueue_in_transaction(&mut transaction, item).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.trim().is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "status": outbox_item.status.as_str(),
                    "draft_id": outbox_item.draft_id,
                    "scheduled_send_at": outbox_item.scheduled_send_at,
                    "undo_deadline_at": outbox_item.undo_deadline_at,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "outbox_item",
                outbox_item.outbox_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(outbox_item)
    }

    pub async fn list(
        &self,
        account_id: Option<&str>,
        status: Option<CommunicationOutboxStatus>,
        limit: i64,
    ) -> Result<Vec<CommunicationOutboxItem>, CommunicationOutboxError> {
        Ok(self.list_page(account_id, status, None, limit).await?.items)
    }

    pub async fn list_page(
        &self,
        account_id: Option<&str>,
        status: Option<CommunicationOutboxStatus>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<EmailOutboxListPage, CommunicationOutboxError> {
        let limit = validate_limit(limit)?;
        let cursor = cursor.map(decode_outbox_list_cursor).transpose()?;
        let status = status.map(CommunicationOutboxStatus::as_str);
        let rows = sqlx::query(
            r#"
            SELECT
                outbox.outbox_id,
                outbox.account_id,
                outbox.draft_id,
                outbox.to_participants AS to_recipients,
                outbox.cc_participants AS cc_recipients,
                outbox.bcc_participants AS bcc_recipients,
                outbox.subject,
                outbox.body_text,
                outbox.body_html,
                outbox.status,
                outbox.scheduled_send_at,
                outbox.undo_deadline_at,
                outbox.send_attempts,
                outbox.claimed_at,
                outbox.sent_at,
                outbox.provider_message_id,
                outbox.last_error,
                CASE
                    WHEN latest_receipt.latest_read_receipt IS NULL THEN outbox.metadata
                    ELSE jsonb_set(
                        outbox.metadata,
                        '{latest_read_receipt}',
                        latest_receipt.latest_read_receipt,
                        true
                    )
                END AS metadata,
                outbox.created_at,
                outbox.updated_at
            FROM communication_outbox outbox
            LEFT JOIN LATERAL (
                SELECT jsonb_build_object(
                    'receipt_kind', receipt.receipt_kind,
                    'read_at', receipt.read_at,
                    'source_kind', receipt.source_kind
                ) AS latest_read_receipt
                FROM communication_read_receipts receipt
                WHERE receipt.outbox_id = outbox.outbox_id
                ORDER BY receipt.read_at DESC, receipt.receipt_id ASC
                LIMIT 1
            ) latest_receipt ON true
            WHERE ($1::text IS NULL OR outbox.account_id = $1)
              AND ($2::text IS NULL OR outbox.status = $2)
              AND (
                  $3::timestamptz IS NULL
                  OR outbox.created_at < $3
                  OR (outbox.created_at = $3 AND outbox.outbox_id > $4)
              )
            ORDER BY outbox.created_at DESC, outbox.outbox_id ASC
            LIMIT $5
            "#,
        )
        .bind(account_id)
        .bind(status)
        .bind(cursor.as_ref().map(|cursor| cursor.created_at))
        .bind(cursor.as_ref().map(|cursor| cursor.outbox_id.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

        let mut items = rows
            .into_iter()
            .map(row_to_outbox_item)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > limit as usize;
        if has_more {
            items.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            items.last().map(encode_outbox_list_cursor).transpose()?
        } else {
            None
        };

        Ok(EmailOutboxListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn undo(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        self.undo_with_observation(outbox_id, now, None, "outbox_status_transition", None)
            .await
    }

    pub async fn undo_with_observation(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'canceled',
                updated_at = $2
            WHERE outbox_id = $1
              AND status IN ('queued', 'scheduled')
              AND undo_deadline_at IS NOT NULL
              AND undo_deadline_at >= $2
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .fetch_optional(&mut *transaction)
            .await?;

        match row {
            Some(row) => {
                let item = row_to_outbox_item(row)?;
                if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
                    let link_metadata = merge_metadata(
                        json!({
                            "operation": "outbox_undo",
                            "status": item.status.as_str(),
                        }),
                        metadata,
                    );
                    link_mail_entity_in_transaction(
                        &mut transaction,
                        observation_id,
                        "outbox_item",
                        item.outbox_id.clone(),
                        relationship_kind,
                        link_metadata,
                        None,
                    )
                    .await?;
                }
                transaction.commit().await?;
                Ok(item)
            }
            None => {
                transaction.rollback().await?;
                Err(CommunicationOutboxError::UndoUnavailable)
            }
        }
    }

    pub async fn claim_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<CommunicationOutboxItem>, CommunicationOutboxError> {
        let limit = validate_limit(limit)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            WITH due AS (
                SELECT outbox_id
                FROM communication_outbox
                WHERE status IN ('queued', 'scheduled')
                  AND (scheduled_send_at IS NULL OR scheduled_send_at <= $1)
                  AND (undo_deadline_at IS NULL OR undo_deadline_at <= $1)
                ORDER BY COALESCE(scheduled_send_at, created_at) ASC, created_at ASC, outbox_id ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            UPDATE communication_outbox item
            SET status = 'sending',
                send_attempts = item.send_attempts + 1,
                claimed_at = $1,
                updated_at = $1
            FROM due
            WHERE item.outbox_id = due.outbox_id
            "#,
            "item",
        );
        let rows = sqlx::query(&sql)
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *transaction)
            .await?;
        let items = rows
            .into_iter()
            .map(row_to_outbox_item)
            .collect::<Result<Vec<_>, _>>()?;
        for item in &items {
            capture_outbox_transition_observation(
                &mut transaction,
                item,
                "outbox_claim_due",
                json!({
                    "status": item.status.as_str(),
                }),
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(items)
    }

    pub async fn mark_sent(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        receipt: &OutboxSendReceipt,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        validate_non_empty("provider_message_id", &receipt.provider_message_id)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'sent',
                sent_at = $2,
                provider_message_id = $3,
                last_error = NULL,
                updated_at = $2
            WHERE outbox_id = $1
              AND status = 'sending'
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .bind(receipt.provider_message_id.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(item) = row.map(row_to_outbox_item).transpose()? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        capture_outbox_transition_observation(
            &mut transaction,
            &item,
            "outbox_mark_sent",
            json!({
                "status": item.status.as_str(),
                "provider_message_id": item.provider_message_id,
            }),
        )
        .await?;
        let event = outbox_delivery_event("mail.outbox.sent", &item)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(item))
    }

    pub async fn mark_failed(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        error_message: &str,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        validate_non_empty("last_error", error_message)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'failed',
                sent_at = NULL,
                provider_message_id = NULL,
                last_error = $3,
                updated_at = $2
            WHERE outbox_id = $1
              AND status = 'sending'
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .bind(error_message.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(item) = row.map(row_to_outbox_item).transpose()? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        capture_outbox_transition_observation(
            &mut transaction,
            &item,
            "outbox_mark_failed",
            json!({
                "status": item.status.as_str(),
                "last_error": item.last_error,
            }),
        )
        .await?;
        let event = outbox_delivery_event("mail.outbox.failed", &item)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(item))
    }

    pub async fn mark_retry_scheduled(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        next_attempt_at: DateTime<Utc>,
        error_message: &str,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        validate_non_empty("last_error", error_message)?;
        if next_attempt_at <= now {
            return Err(CommunicationOutboxError::Invalid(
                "next_attempt_at must be after now",
            ));
        }

        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'scheduled',
                scheduled_send_at = $3,
                claimed_at = NULL,
                sent_at = NULL,
                provider_message_id = NULL,
                last_error = $4,
                updated_at = $2
            WHERE outbox_id = $1
              AND status = 'sending'
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .bind(next_attempt_at)
            .bind(error_message.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(item) = row.map(row_to_outbox_item).transpose()? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        capture_outbox_transition_observation(
            &mut transaction,
            &item,
            "outbox_retry_scheduled",
            json!({
                "status": item.status.as_str(),
                "scheduled_send_at": item.scheduled_send_at,
                "last_error": item.last_error,
            }),
        )
        .await?;
        let event = outbox_delivery_event("mail.outbox.retry_scheduled", &item)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(item))
    }
}

pub(crate) async fn enqueue_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    item: &NewCommunicationOutboxItem,
) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
    item.validate()?;
    ensure_canonical_account_in_transaction(transaction, Some(item.account_id.as_str())).await?;
    let draft_id =
        existing_draft_id_in_transaction(transaction, item.draft_id.as_deref(), &item.account_id)
            .await?;
    let mut outbox_metadata = item.metadata.clone();
    if let Some(object) = outbox_metadata.as_object_mut() {
        object.insert(
            "rfc822_message_id".to_owned(),
            Value::String(rfc822_message_id_for_outbox(&item.outbox_id)),
        );
    }
    let sql = outbox_returning_query(
        r#"
        INSERT INTO communication_outbox (
            outbox_id,
            account_id,
            draft_id,
            to_participants,
            cc_participants,
            bcc_participants,
            subject,
            body_text,
            body_html,
            status,
            scheduled_send_at,
            undo_deadline_at,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
        "communication_outbox",
    );
    let row = sqlx::query(&sql)
        .bind(item.outbox_id.trim())
        .bind(item.account_id.trim())
        .bind(draft_id.as_deref())
        .bind(serde_json::to_value(&item.to_recipients)?)
        .bind(serde_json::to_value(&item.cc_recipients)?)
        .bind(serde_json::to_value(&item.bcc_recipients)?)
        .bind(&item.subject)
        .bind(&item.body_text)
        .bind(item.body_html.as_deref())
        .bind(item.status.as_str())
        .bind(item.scheduled_send_at)
        .bind(item.undo_deadline_at)
        .bind(&outbox_metadata)
        .fetch_one(&mut **transaction)
        .await?;

    if let Some(draft_id) = draft_id.as_deref() {
        copy_draft_attachments_to_outbox_in_transaction(
            transaction,
            item.outbox_id.trim(),
            draft_id,
        )
        .await?;
    }

    row_to_outbox_item(row)
}

#[derive(Debug, Error)]
pub enum CommunicationOutboxError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("invalid outbox item: {0}")]
    Invalid(&'static str),

    #[error("invalid outbox cursor")]
    InvalidCursor,

    #[error("outbox item cannot be undone")]
    UndoUnavailable,
}
mod support;
use support::*;
