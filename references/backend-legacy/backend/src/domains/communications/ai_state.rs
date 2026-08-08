use makosh_events_api::NewEventEnvelope;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::errors::ObservationStoreError;

const EVENT_TYPE_CHANGED: &str = "mail.ai_state.changed";
pub const MAIL_AI_MAX_ATTEMPTS: i32 = 3;
const MAIL_AI_PROCESSING_LEASE_SECONDS: i64 = 300;
const MAIL_AI_RETRY_BASE_SECONDS: i64 = 30;
use super::evidence::link_mail_entity_in_transaction;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommunicationAiState {
    New,
    Processing,
    Processed,
    ReviewRequired,
    Failed,
    Archived,
}

impl CommunicationAiState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Processing => "PROCESSING",
            Self::Processed => "PROCESSED",
            Self::ReviewRequired => "REVIEW_REQUIRED",
            Self::Failed => "FAILED",
            Self::Archived => "ARCHIVED",
        }
    }
}

impl TryFrom<&str> for CommunicationAiState {
    type Error = CommunicationAiStateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "NEW" => Ok(Self::New),
            "PROCESSING" => Ok(Self::Processing),
            "PROCESSED" => Ok(Self::Processed),
            "REVIEW_REQUIRED" => Ok(Self::ReviewRequired),
            "FAILED" => Ok(Self::Failed),
            "ARCHIVED" => Ok(Self::Archived),
            _ => Err(CommunicationAiStateError::Invalid("ai_state")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationAiStateRecord {
    pub message_id: String,
    pub ai_state: CommunicationAiState,
    pub review_reason: Option<String>,
    pub last_error: Option<String>,
    pub retry_count: i32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub processing_lease_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommunicationAiStateTransitionRequest {
    pub ai_state: CommunicationAiState,
    pub review_reason: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct CommunicationAiStateStore {
    pool: PgPool,
}

impl CommunicationAiStateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn current(
        &self,
        message_id: &str,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                m.message_id,
                COALESCE(s.ai_state, 'NEW') AS ai_state,
                s.review_reason,
                s.last_error,
                COALESCE(s.retry_count, 0) AS retry_count,
                s.next_attempt_at,
                s.processing_lease_expires_at,
                COALESCE(s.created_at, m.projected_at) AS created_at,
                COALESCE(s.updated_at, m.projected_at) AS updated_at
            FROM communication_messages m
            LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
            WHERE m.message_id = $1
            "#,
        )
        .bind(&message_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_ai_state).transpose()
    }

    pub async fn transition(
        &self,
        message_id: &str,
        request: CommunicationAiStateTransitionRequest,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        self.transition_with_observation(message_id, request, None, "ai_state_transition", None)
            .await
    }

    pub async fn pending_mail_message_ids(
        &self,
        limit: i64,
    ) -> Result<Vec<String>, CommunicationAiStateError> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT m.message_id
            FROM communication_messages m
            LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
            WHERE m.local_state = 'active'
              AND m.channel_kind IN ('mail', 'email')
              AND COALESCE(s.ai_state, 'NEW') = 'NEW'
            ORDER BY COALESCE(m.occurred_at, m.projected_at) ASC, m.message_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn transition_with_observation(
        &self,
        message_id: &str,
        request: CommunicationAiStateTransitionRequest,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let update = NormalizedCommunicationAiStateTransition::from_request(request)?;
        let mut transaction = self.pool.begin().await?;

        let Some(previous) = select_current_ai_state(&mut transaction, &message_id).await? else {
            transaction.rollback().await?;
            return Ok(None);
        };

        let record = write_ai_state_in_transaction(
            &mut transaction,
            AiStateWrite {
                message_id: &message_id,
                ai_state: update.ai_state,
                review_reason: update.review_reason.as_deref(),
                last_error: update.last_error.as_deref(),
                retry_count: manual_retry_count(previous.retry_count, update.ai_state),
                next_attempt_at: None,
                processing_lease_expires_at: processing_lease_for(update.ai_state, Utc::now()),
            },
        )
        .await?;
        let event = ai_state_changed_event(&record, previous.ai_state, "makosh-frontend")?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                record.message_id.clone(),
                relationship_kind,
                json!({
                    "previous_ai_state": previous.ai_state.as_str(),
                    "ai_state": record.ai_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(Some(record))
    }

    pub async fn recover_expired_mail_processing(
        &self,
        now: DateTime<Utc>,
    ) -> Result<usize, CommunicationAiStateError> {
        let mut transaction = self.pool.begin().await?;
        let message_ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT s.message_id
            FROM communication_ai_states s
            JOIN communication_messages m ON m.message_id = s.message_id
            WHERE m.local_state = 'active'
              AND m.channel_kind IN ('mail', 'email')
              AND s.ai_state = 'PROCESSING'
              AND s.processing_lease_expires_at IS NOT NULL
              AND s.processing_lease_expires_at <= $1
            ORDER BY s.processing_lease_expires_at ASC, s.message_id ASC
            FOR UPDATE OF m SKIP LOCKED
            "#,
        )
        .bind(now)
        .fetch_all(&mut *transaction)
        .await?;

        for message_id in &message_ids {
            let Some(previous) = select_current_ai_state(&mut transaction, message_id).await?
            else {
                continue;
            };
            if previous.ai_state != CommunicationAiState::Processing {
                continue;
            }
            let retry_count = previous.retry_count.saturating_add(1);
            let record = write_ai_state_in_transaction(
                &mut transaction,
                AiStateWrite {
                    message_id,
                    ai_state: CommunicationAiState::Failed,
                    review_reason: None,
                    last_error: Some("AI processing lease expired"),
                    retry_count,
                    next_attempt_at: retry_next_attempt_at(now, message_id, retry_count),
                    processing_lease_expires_at: None,
                },
            )
            .await?;
            let event =
                ai_state_changed_event(&record, previous.ai_state, "makosh-mail-ai-worker")?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }

        transaction.commit().await?;
        Ok(message_ids.len())
    }

    pub async fn claim_due_mail_messages(
        &self,
        limit: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<String>, CommunicationAiStateError> {
        let mut transaction = self.pool.begin().await?;
        let message_ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT m.message_id
            FROM communication_messages m
            WHERE m.local_state = 'active'
              AND m.channel_kind IN ('mail', 'email')
              AND (
                NOT EXISTS (
                    SELECT 1
                    FROM communication_ai_states s
                    WHERE s.message_id = m.message_id
                )
                OR EXISTS (
                    SELECT 1
                    FROM communication_ai_states s
                    WHERE s.message_id = m.message_id
                      AND (
                        s.ai_state = 'NEW'
                        OR (s.ai_state = 'FAILED' AND s.next_attempt_at IS NOT NULL AND s.next_attempt_at <= $1)
                      )
                )
              )
            ORDER BY COALESCE(m.occurred_at, m.projected_at) ASC, m.message_id ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(now)
        .bind(limit.clamp(1, 100))
        .fetch_all(&mut *transaction)
        .await?;

        for message_id in &message_ids {
            let Some(previous) = select_current_ai_state(&mut transaction, message_id).await?
            else {
                continue;
            };
            let record = write_ai_state_in_transaction(
                &mut transaction,
                AiStateWrite {
                    message_id,
                    ai_state: CommunicationAiState::Processing,
                    review_reason: None,
                    last_error: None,
                    retry_count: previous.retry_count,
                    next_attempt_at: None,
                    processing_lease_expires_at: processing_lease_for(
                        CommunicationAiState::Processing,
                        now,
                    ),
                },
            )
            .await?;
            let event =
                ai_state_changed_event(&record, previous.ai_state, "makosh-mail-ai-worker")?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }

        transaction.commit().await?;
        Ok(message_ids)
    }

    pub async fn record_mail_processing_failure(
        &self,
        message_id: &str,
        error: &str,
        retryable: bool,
        now: DateTime<Utc>,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let error = normalize_required("error", error)?;
        let mut transaction = self.pool.begin().await?;
        let Some(previous) = select_current_ai_state(&mut transaction, &message_id).await? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        if previous.ai_state != CommunicationAiState::Processing {
            transaction.rollback().await?;
            return Ok(Some(previous));
        }

        let retry_count = previous.retry_count.saturating_add(1);
        let record = write_ai_state_in_transaction(
            &mut transaction,
            AiStateWrite {
                message_id: &message_id,
                ai_state: CommunicationAiState::Failed,
                review_reason: None,
                last_error: Some(&error),
                retry_count,
                next_attempt_at: retryable
                    .then(|| retry_next_attempt_at(now, &message_id, retry_count))
                    .flatten(),
                processing_lease_expires_at: None,
            },
        )
        .await?;
        let event = ai_state_changed_event(&record, previous.ai_state, "makosh-mail-ai-worker")?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(record))
    }
}

#[derive(Debug)]
struct NormalizedCommunicationAiStateTransition {
    ai_state: CommunicationAiState,
    review_reason: Option<String>,
    last_error: Option<String>,
}

impl NormalizedCommunicationAiStateTransition {
    fn from_request(
        request: CommunicationAiStateTransitionRequest,
    ) -> Result<Self, CommunicationAiStateError> {
        let review_reason = normalize_optional(request.review_reason)?;
        let last_error = normalize_optional(request.last_error)?;

        match request.ai_state {
            CommunicationAiState::ReviewRequired if review_reason.is_none() => {
                return Err(CommunicationAiStateError::Invalid("review_reason"));
            }
            CommunicationAiState::Failed if last_error.is_none() => {
                return Err(CommunicationAiStateError::Invalid("last_error"));
            }
            _ => {}
        }

        Ok(Self {
            ai_state: request.ai_state,
            review_reason: if request.ai_state == CommunicationAiState::ReviewRequired {
                review_reason
            } else {
                None
            },
            last_error: if request.ai_state == CommunicationAiState::Failed {
                last_error
            } else {
                None
            },
        })
    }
}

async fn select_current_ai_state(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: &str,
) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
    let row = sqlx::query(
        r#"
        SELECT
            m.message_id,
            COALESCE(s.ai_state, 'NEW') AS ai_state,
            s.review_reason,
            s.last_error,
            COALESCE(s.retry_count, 0) AS retry_count,
            s.next_attempt_at,
            s.processing_lease_expires_at,
            COALESCE(s.created_at, m.projected_at) AS created_at,
            COALESCE(s.updated_at, m.projected_at) AS updated_at
        FROM communication_messages m
        LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
        WHERE m.message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_ai_state).transpose()
}

struct AiStateWrite<'a> {
    message_id: &'a str,
    ai_state: CommunicationAiState,
    review_reason: Option<&'a str>,
    last_error: Option<&'a str>,
    retry_count: i32,
    next_attempt_at: Option<DateTime<Utc>>,
    processing_lease_expires_at: Option<DateTime<Utc>>,
}

async fn write_ai_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    write: AiStateWrite<'_>,
) -> Result<CommunicationAiStateRecord, CommunicationAiStateError> {
    let row = sqlx::query(
        r#"
        INSERT INTO communication_ai_states (
            message_id,
            ai_state,
            review_reason,
            last_error,
            retry_count,
            next_attempt_at,
            processing_lease_expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (message_id)
        DO UPDATE SET
            ai_state = EXCLUDED.ai_state,
            review_reason = EXCLUDED.review_reason,
            last_error = EXCLUDED.last_error,
            retry_count = EXCLUDED.retry_count,
            next_attempt_at = EXCLUDED.next_attempt_at,
            processing_lease_expires_at = EXCLUDED.processing_lease_expires_at,
            updated_at = now()
        RETURNING
            message_id,
            ai_state,
            review_reason,
            last_error,
            retry_count,
            next_attempt_at,
            processing_lease_expires_at,
            created_at,
            updated_at
        "#,
    )
    .bind(write.message_id)
    .bind(write.ai_state.as_str())
    .bind(write.review_reason)
    .bind(write.last_error)
    .bind(write.retry_count)
    .bind(write.next_attempt_at)
    .bind(write.processing_lease_expires_at)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_ai_state(row)
}

fn row_to_ai_state(row: PgRow) -> Result<CommunicationAiStateRecord, CommunicationAiStateError> {
    let ai_state: String = row.try_get("ai_state")?;
    Ok(CommunicationAiStateRecord {
        message_id: row.try_get("message_id")?,
        ai_state: CommunicationAiState::try_from(ai_state.as_str())?,
        review_reason: row.try_get("review_reason")?,
        last_error: row.try_get("last_error")?,
        retry_count: row.try_get("retry_count")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        processing_lease_expires_at: row.try_get("processing_lease_expires_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn ai_state_changed_event(
    record: &CommunicationAiStateRecord,
    previous_ai_state: CommunicationAiState,
    actor_id: &str,
) -> Result<NewEventEnvelope, CommunicationAiStateError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_ai_state_event:{}:{:x}",
            record.message_id,
            system_time_nanos()
        ),
        EVENT_TYPE_CHANGED,
        Utc::now(),
        json!({ "kind": "mail_ai_state_api" }),
        json!({
            "kind": "mail_ai_state",
            "id": record.message_id,
            "message_id": record.message_id,
        }),
    )
    .actor(json!({ "actor_id": actor_id }))
    .payload(json!({
        "message_id": record.message_id,
        "ai_state": record.ai_state.as_str(),
            "previous_ai_state": previous_ai_state.as_str(),
            "review_required": record.review_reason.is_some(),
            "failed": record.last_error.is_some(),
            "retry_count": record.retry_count,
            "next_attempt_at": record.next_attempt_at,
            "processing_lease_expires_at": record.processing_lease_expires_at,
    }))
    .provenance(json!({
        "source_kind": if actor_id == "makosh-frontend" { "local_api" } else { "automation" },
        "source_id": record.message_id,
    }))
    .correlation_id(record.message_id.clone())
    .build()?)
}

fn manual_retry_count(previous_retry_count: i32, ai_state: CommunicationAiState) -> i32 {
    match ai_state {
        CommunicationAiState::New
        | CommunicationAiState::Processed
        | CommunicationAiState::ReviewRequired
        | CommunicationAiState::Archived => 0,
        CommunicationAiState::Processing | CommunicationAiState::Failed => previous_retry_count,
    }
}

fn processing_lease_for(
    ai_state: CommunicationAiState,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    (ai_state == CommunicationAiState::Processing)
        .then_some(now + chrono::Duration::seconds(MAIL_AI_PROCESSING_LEASE_SECONDS))
}

fn retry_next_attempt_at(
    now: DateTime<Utc>,
    message_id: &str,
    retry_count: i32,
) -> Option<DateTime<Utc>> {
    if retry_count >= MAIL_AI_MAX_ATTEMPTS {
        return None;
    }

    let retry_index = retry_count.saturating_sub(1).clamp(0, 10) as u32;
    let delay_seconds = MAIL_AI_RETRY_BASE_SECONDS.saturating_mul(1_i64 << retry_index);
    let jitter_seconds = stable_retry_jitter_seconds(message_id, retry_count);
    Some(now + chrono::Duration::seconds(delay_seconds + jitter_seconds))
}

fn stable_retry_jitter_seconds(message_id: &str, retry_count: i32) -> i64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    message_id.hash(&mut hasher);
    retry_count.hash(&mut hasher);
    (hasher.finish() % 15) as i64
}

fn normalize_required(
    field: &'static str,
    value: &str,
) -> Result<String, CommunicationAiStateError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CommunicationAiStateError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(value: Option<String>) -> Result<Option<String>, CommunicationAiStateError> {
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

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[derive(Debug, Error)]
pub enum CommunicationAiStateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error("invalid mail AI state field: {0}")]
    Invalid(&'static str),
}
