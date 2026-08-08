use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

use super::evidence::link_calendar_entity_in_transaction;

// ── DeadlineEvent ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadlineEvent {
    pub id: String,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub title: String,
    pub due_at: DateTime<Utc>,
    pub severity: String,
    pub status: String,
    pub linked_calendar_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DeadlineStore {
    pool: PgPool,
}

impl DeadlineStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<DeadlineEvent>, SchedulingError> {
        let limit = limit.clamp(1, 200);
        let rows = if let Some(s) = status {
            sqlx::query("SELECT id::text, source_entity_type, source_entity_id, title, due_at, severity, status, linked_calendar_event_id, created_at, updated_at FROM deadline_events WHERE status=$1 ORDER BY due_at ASC LIMIT $2")
                .bind(s).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT id::text, source_entity_type, source_entity_id, title, due_at, severity, status, linked_calendar_event_id, created_at, updated_at FROM deadline_events ORDER BY due_at ASC LIMIT $1")
                .bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter()
            .map(|r| {
                Ok(DeadlineEvent {
                    id: r.try_get("id")?,
                    source_entity_type: r.try_get("source_entity_type")?,
                    source_entity_id: r.try_get("source_entity_id")?,
                    title: r.try_get("title")?,
                    due_at: r.try_get("due_at")?,
                    severity: r.try_get("severity")?,
                    status: r.try_get("status")?,
                    linked_calendar_event_id: r.try_get("linked_calendar_event_id")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        title: &str,
        due_at: DateTime<Utc>,
        severity: Option<&str>,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
    ) -> Result<DeadlineEvent, SchedulingError> {
        self.create_with_observation(
            title,
            due_at,
            severity,
            entity_type,
            entity_id,
            None,
            "create",
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        title: &str,
        due_at: DateTime<Utc>,
        severity: Option<&str>,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<DeadlineEvent, SchedulingError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO deadline_events (title, due_at, severity, source_entity_type, source_entity_id) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, source_entity_type, source_entity_id, title, due_at, severity, status, linked_calendar_event_id, created_at, updated_at")
            .bind(title).bind(due_at).bind(severity.unwrap_or("medium")).bind(entity_type).bind(entity_id).fetch_one(&mut *transaction).await?;
        let deadline = DeadlineEvent {
            id: row.try_get("id")?,
            source_entity_type: row.try_get("source_entity_type")?,
            source_entity_id: row.try_get("source_entity_id")?,
            title: row.try_get("title")?,
            due_at: row.try_get("due_at")?,
            severity: row.try_get("severity")?,
            status: row.try_get("status")?,
            linked_calendar_event_id: row.try_get("linked_calendar_event_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "deadline_event",
                deadline.id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "deadline_id": deadline.id,
                    "source_entity_type": deadline.source_entity_type,
                    "source_entity_id": deadline.source_entity_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(deadline)
    }
}

// ── FocusBlock ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FocusBlock {
    pub id: String,
    pub title: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub purpose: Option<String>,
    pub linked_project_id: Option<String>,
    pub protection_level: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FocusBlockStore {
    pool: PgPool,
}

impl FocusBlockStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<FocusBlock>, SchedulingError> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query("SELECT id::text, title, start_at, end_at, purpose, linked_project_id, protection_level, status, created_at, updated_at FROM focus_blocks WHERE ($1::timestamptz IS NULL OR end_at>=$1) AND ($2::timestamptz IS NULL OR start_at<=$2) ORDER BY start_at ASC LIMIT $3")
            .bind(from).bind(to).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(FocusBlock {
                    id: r.try_get("id")?,
                    title: r.try_get("title")?,
                    start_at: r.try_get("start_at")?,
                    end_at: r.try_get("end_at")?,
                    purpose: r.try_get("purpose")?,
                    linked_project_id: r.try_get("linked_project_id")?,
                    protection_level: r.try_get("protection_level")?,
                    status: r.try_get("status")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        purpose: Option<&str>,
        project_id: Option<&str>,
        protection: Option<&str>,
    ) -> Result<FocusBlock, SchedulingError> {
        self.create_with_observation(
            title, start_at, end_at, purpose, project_id, protection, None, "create", None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        purpose: Option<&str>,
        project_id: Option<&str>,
        protection: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<FocusBlock, SchedulingError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO focus_blocks (title, start_at, end_at, purpose, linked_project_id, protection_level) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id::text, title, start_at, end_at, purpose, linked_project_id, protection_level, status, created_at, updated_at")
            .bind(title).bind(start_at).bind(end_at).bind(purpose).bind(project_id).bind(protection.unwrap_or("medium")).fetch_one(&mut *transaction).await?;
        let focus_block = FocusBlock {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            start_at: row.try_get("start_at")?,
            end_at: row.try_get("end_at")?,
            purpose: row.try_get("purpose")?,
            linked_project_id: row.try_get("linked_project_id")?,
            protection_level: row.try_get("protection_level")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "focus_block",
                focus_block.id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "focus_block_id": focus_block.id,
                    "linked_project_id": focus_block.linked_project_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(focus_block)
    }
}

// ── SmartSchedulingService ─────────────────────────────────────────────────

pub struct SmartSchedulingService;

impl SmartSchedulingService {
    /// Heuristic: find free slots by checking event gaps today+tomorrow
    pub fn find_slots(
        existing_events: &[(DateTime<Utc>, DateTime<Utc>)],
        duration_min: i64,
        lookahead_hours: i64,
    ) -> Vec<Slot> {
        let now = Utc::now();
        let end = now + Duration::hours(lookahead_hours);
        let mut slots = Vec::new();
        let mut sorted: Vec<_> = existing_events
            .iter()
            .filter(|(s, e)| *e > now && *s < end)
            .collect();
        sorted.sort_by_key(|(s, _)| *s);

        let mut cursor = now;
        for (s, e) in sorted {
            if *s > cursor {
                let gap = (*s - cursor).num_minutes();
                if gap >= duration_min {
                    slots.push(Slot {
                        start: cursor,
                        end: *s,
                        duration_minutes: gap,
                    });
                }
            }
            cursor = cursor.max(*e);
        }
        if end > cursor {
            let gap = (end - cursor).num_minutes();
            if gap >= duration_min {
                slots.push(Slot {
                    start: cursor,
                    end,
                    duration_minutes: gap,
                });
            }
        }
        slots
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_minutes: i64,
}

#[derive(Debug, Error)]
pub enum SchedulingError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
