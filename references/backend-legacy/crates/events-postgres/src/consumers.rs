use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPool;

use super::errors::EventStoreError;
use super::store::EventStore;
use super::validation::validate_non_empty;
use makosh_events_api::StoredEventEnvelope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventConsumerConfig {
    pub consumer_name: String,
    pub batch_size: u32,
    pub max_attempts: i32,
    pub retry_base_seconds: i64,
}

impl EventConsumerConfig {
    pub fn new(consumer_name: impl Into<String>) -> Self {
        Self {
            consumer_name: consumer_name.into(),
            batch_size: 100,
            max_attempts: 5,
            retry_base_seconds: 30,
        }
    }

    fn normalized(&self) -> Self {
        Self {
            consumer_name: self.consumer_name.trim().to_owned(),
            batch_size: self.batch_size.clamp(1, 1000),
            max_attempts: self.max_attempts.max(1),
            retry_base_seconds: self.retry_base_seconds.max(0),
        }
    }
}

#[derive(Clone)]
pub struct EventConsumerRunner {
    event_store: EventStore,
    consumer_store: EventConsumerStore,
    config: EventConsumerConfig,
}

impl EventConsumerRunner {
    pub fn new(pool: PgPool, config: EventConsumerConfig) -> Self {
        Self {
            event_store: EventStore::new(pool.clone()),
            consumer_store: EventConsumerStore::new(pool),
            config: config.normalized(),
        }
    }

    pub async fn process_next_batch<F, Fut>(
        &self,
        mut handler: F,
    ) -> Result<EventConsumerRunReport, EventStoreError>
    where
        F: FnMut(StoredEventEnvelope) -> Fut,
        Fut: Future<Output = Result<(), EventStoreError>>,
    {
        validate_non_empty("consumer_name", &self.config.consumer_name)?;
        self.consumer_store
            .ensure_consumer(&self.config.consumer_name)
            .await?;

        let mut report = EventConsumerRunReport::default();
        let cursor = self
            .consumer_store
            .last_processed_position(&self.config.consumer_name)
            .await?;
        let events = self
            .event_store
            .list_after_position(cursor, self.config.batch_size)
            .await?;

        for event in events {
            let now = Utc::now();
            if self
                .consumer_store
                .has_processed_event(&self.config.consumer_name, event.position)
                .await?
                || self
                    .consumer_store
                    .has_processed_event_id(&self.config.consumer_name, &event.event.event_id)
                    .await?
            {
                self.consumer_store
                    .clear_failure(&self.config.consumer_name, event.position)
                    .await?;
                self.consumer_store
                    .save_position(&self.config.consumer_name, event.position)
                    .await?;
                report.skipped_duplicates += 1;
                report.last_processed_position = event.position;
                continue;
            }

            if self
                .consumer_store
                .next_attempt_at(&self.config.consumer_name, event.position)
                .await?
                .is_some_and(|next_attempt_at| next_attempt_at > now)
            {
                report.pending_retry = true;
                break;
            }

            match handler(event.clone()).await {
                Ok(()) => {
                    let inserted = self
                        .consumer_store
                        .record_processed(&self.config.consumer_name, &event)
                        .await?;
                    self.consumer_store
                        .mark_dead_letter_replayed_for_event(
                            &self.config.consumer_name,
                            event.position,
                        )
                        .await?;
                    self.consumer_store
                        .clear_failure(&self.config.consumer_name, event.position)
                        .await?;
                    self.consumer_store
                        .save_position(&self.config.consumer_name, event.position)
                        .await?;
                    if inserted {
                        report.processed += 1;
                    } else {
                        // Another worker completed the same event after our preflight check.
                        // The handler must already be idempotent, and the marker conflict is not
                        // a processing failure or a reason to poison the consumer retry queue.
                        report.skipped_duplicates += 1;
                    }
                    report.last_processed_position = event.position;
                }
                Err(error) => {
                    let error_message = error.to_string();
                    let attempt_count = self
                        .consumer_store
                        .record_failure(
                            &self.config.consumer_name,
                            &event,
                            &error_message,
                            next_retry_at(now, self.config.retry_base_seconds, 1),
                        )
                        .await?;
                    report.failed += 1;

                    if attempt_count >= self.config.max_attempts {
                        self.consumer_store
                            .dead_letter(
                                &self.config.consumer_name,
                                &event,
                                attempt_count,
                                &error_message,
                            )
                            .await?;
                        self.consumer_store
                            .save_position(&self.config.consumer_name, event.position)
                            .await?;
                        report.dead_lettered += 1;
                        report.last_processed_position = event.position;
                        continue;
                    }

                    let retry_at =
                        next_retry_at(now, self.config.retry_base_seconds, attempt_count);
                    self.consumer_store
                        .update_next_attempt(&self.config.consumer_name, event.position, retry_at)
                        .await?;
                    break;
                }
            }
        }

        Ok(report)
    }

    pub async fn replay_dead_letter<F, Fut>(
        &self,
        dead_letter_id: &str,
        handler: F,
    ) -> Result<(), EventStoreError>
    where
        F: FnOnce(StoredEventEnvelope) -> Fut,
        Fut: Future<Output = Result<(), EventStoreError>>,
    {
        validate_non_empty("dead_letter_id", dead_letter_id)?;
        let dead_letter = self
            .consumer_store
            .dead_letter_by_id(dead_letter_id)
            .await?;
        if dead_letter.review_state != EventDeadLetterReviewState::ReplayRequested {
            return Err(EventStoreError::DeadLetterNotReplayRequested(
                dead_letter_id.to_owned(),
            ));
        }

        handler(dead_letter.event.clone()).await?;
        self.consumer_store
            .record_processed(&dead_letter.consumer_name, &dead_letter.event)
            .await?;
        self.consumer_store
            .mark_dead_letter_replayed(dead_letter_id)
            .await?;
        self.consumer_store
            .clear_failure(&dead_letter.consumer_name, dead_letter.event_position)
            .await?;

        Ok(())
    }

    pub fn store(&self) -> &EventConsumerStore {
        &self.consumer_store
    }
}

#[derive(Clone)]
pub struct EventConsumerStore {
    pool: PgPool,
}

impl EventConsumerStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_consumer(&self, consumer_name: &str) -> Result<(), EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        sqlx::query(
            r#"
            INSERT INTO event_consumers (consumer_name, updated_at)
            VALUES ($1, now())
            ON CONFLICT (consumer_name)
            DO NOTHING
            "#,
        )
        .bind(consumer_name.trim())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn last_processed_position(
        &self,
        consumer_name: &str,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let position = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT last_processed_position
            FROM event_consumers
            WHERE consumer_name = $1
            "#,
        )
        .bind(consumer_name.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.flatten().unwrap_or(0))
    }

    pub async fn save_position(
        &self,
        consumer_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO event_consumers (
                consumer_name,
                last_processed_position,
                updated_at
            )
            VALUES ($1, $2, now())
            ON CONFLICT (consumer_name)
            DO UPDATE SET
                last_processed_position = GREATEST(
                    event_consumers.last_processed_position,
                    EXCLUDED.last_processed_position
                ),
                updated_at = now()
            RETURNING last_processed_position
            "#,
        )
        .bind(consumer_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }

    pub async fn rewind_position(
        &self,
        consumer_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }
        self.ensure_consumer(consumer_name).await?;

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE event_consumers
            SET
                last_processed_position = LEAST(
                    COALESCE(last_processed_position, $2),
                    $2
                ),
                updated_at = now()
            WHERE consumer_name = $1
            RETURNING COALESCE(last_processed_position, 0)
            "#,
        )
        .bind(consumer_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }

    pub async fn next_attempt_at(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<Option<DateTime<Utc>>, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let next_attempt_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT next_attempt_at
            FROM event_consumer_failures
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .fetch_optional(&self.pool)
        .await?;

        Ok(next_attempt_at.flatten())
    }

    pub async fn record_failure(
        &self,
        consumer_name: &str,
        event: &StoredEventEnvelope,
        error_message: &str,
        next_attempt_at: DateTime<Utc>,
    ) -> Result<i32, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        validate_non_empty("error_message", error_message)?;
        self.ensure_consumer(consumer_name).await?;

        let attempt_count = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO event_consumer_failures (
                consumer_name,
                event_position,
                event_id,
                event_type,
                attempt_count,
                next_attempt_at,
                last_attempt_at,
                last_error,
                updated_at
            )
            VALUES ($1, $2, $3, $4, 1, $5, now(), $6, now())
            ON CONFLICT (consumer_name, event_position)
            DO UPDATE SET
                attempt_count = event_consumer_failures.attempt_count + 1,
                next_attempt_at = EXCLUDED.next_attempt_at,
                last_attempt_at = now(),
                last_error = EXCLUDED.last_error,
                updated_at = now()
            RETURNING attempt_count
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event.position)
        .bind(&event.event.event_id)
        .bind(&event.event.event_type)
        .bind(next_attempt_at)
        .bind(error_message.trim())
        .fetch_one(&self.pool)
        .await?;

        Ok(attempt_count)
    }

    pub async fn update_next_attempt(
        &self,
        consumer_name: &str,
        event_position: i64,
        next_attempt_at: DateTime<Utc>,
    ) -> Result<(), EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        sqlx::query(
            r#"
            UPDATE event_consumer_failures
            SET next_attempt_at = $3, updated_at = now()
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .bind(next_attempt_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn clear_failure(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<(), EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        sqlx::query(
            r#"
            DELETE FROM event_consumer_failures
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn clear_failures_for_positions(
        &self,
        consumer_name: &str,
        event_positions: &[i64],
    ) -> Result<u64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if event_positions.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(
            r#"
            DELETE FROM event_consumer_failures
            WHERE consumer_name = $1
              AND event_position = ANY($2)
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_positions.to_vec())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn request_replay_for_positions(
        &self,
        consumer_name: &str,
        event_positions: &[i64],
    ) -> Result<u64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if event_positions.is_empty() {
            return Ok(0);
        }
        self.ensure_consumer(consumer_name).await?;

        self.clear_failures_for_positions(consumer_name, event_positions)
            .await?;

        let min_position = event_positions
            .iter()
            .copied()
            .min()
            .ok_or(EventStoreError::InvalidReplayPosition(0))?;
        self.rewind_position(consumer_name, min_position.saturating_sub(1))
            .await?;

        let dead_letters_updated = sqlx::query(
            r#"
            UPDATE event_dead_letters
            SET
                review_state = CASE
                    WHEN review_state = 'replayed' THEN review_state
                    ELSE 'replay_requested'
                END,
                replay_requested_at = CASE
                    WHEN review_state = 'replayed' THEN replay_requested_at
                    ELSE now()
                END,
                updated_at = now()
            WHERE consumer_name = $1
              AND event_position = ANY($2)
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_positions.to_vec())
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(dead_letters_updated)
    }

    pub async fn clear_processed_for_positions(
        &self,
        consumer_name: &str,
        event_positions: &[i64],
    ) -> Result<u64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if event_positions.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(
            r#"
            DELETE FROM event_consumer_processed_events
            WHERE consumer_name = $1
              AND event_position = ANY($2)
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_positions.to_vec())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn has_processed_event(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<bool, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let processed = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM event_consumer_processed_events
                WHERE consumer_name = $1 AND event_position = $2
            )
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .fetch_one(&self.pool)
        .await?;

        Ok(processed)
    }

    pub async fn has_processed_event_id(
        &self,
        consumer_name: &str,
        event_id: &str,
    ) -> Result<bool, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        validate_non_empty("event_id", event_id)?;
        let processed = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM event_consumer_processed_events
                WHERE consumer_name = $1 AND event_id = $2
            )
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_id.trim())
        .fetch_one(&self.pool)
        .await?;

        Ok(processed)
    }

    pub async fn record_processed(
        &self,
        consumer_name: &str,
        event: &StoredEventEnvelope,
    ) -> Result<bool, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        self.ensure_consumer(consumer_name).await?;

        let inserted = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO event_consumer_processed_events (
                consumer_name,
                event_position,
                event_id,
                event_type
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            RETURNING event_position
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event.position)
        .bind(&event.event.event_id)
        .bind(&event.event.event_type)
        .fetch_optional(&self.pool)
        .await?;

        Ok(inserted.is_some())
    }

    pub async fn processed_event_count(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM event_consumer_processed_events
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn failure_attempt_count(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<Option<i32>, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let attempt_count = sqlx::query_scalar::<_, Option<i32>>(
            r#"
            SELECT attempt_count
            FROM event_consumer_failures
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .fetch_optional(&self.pool)
        .await?;

        Ok(attempt_count.flatten())
    }

    pub async fn dead_letter(
        &self,
        consumer_name: &str,
        event: &StoredEventEnvelope,
        attempts: i32,
        error_message: &str,
    ) -> Result<EventDeadLetter, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        validate_non_empty("error_message", error_message)?;
        self.ensure_consumer(consumer_name).await?;

        let dead_letter_id = dead_letter_id(consumer_name, event.position);
        let event_payload = serde_json::to_value(event)?;
        sqlx::query(
            r#"
            INSERT INTO event_dead_letters (
                dead_letter_id,
                consumer_name,
                event_position,
                event_id,
                event_type,
                attempts,
                last_error,
                event_payload,
                review_state,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'open', now())
            ON CONFLICT (consumer_name, event_position)
            DO UPDATE SET
                attempts = GREATEST(event_dead_letters.attempts, EXCLUDED.attempts),
                last_error = EXCLUDED.last_error,
                event_payload = EXCLUDED.event_payload,
                updated_at = now()
            "#,
        )
        .bind(&dead_letter_id)
        .bind(consumer_name.trim())
        .bind(event.position)
        .bind(&event.event.event_id)
        .bind(&event.event.event_type)
        .bind(attempts)
        .bind(error_message.trim())
        .bind(event_payload)
        .execute(&self.pool)
        .await?;

        self.dead_letter_by_id(&dead_letter_id).await
    }

    pub async fn dead_letter_for_event(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<Option<EventDeadLetter>, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let row = sqlx::query_as::<_, EventDeadLetterRow>(
            r#"
            SELECT
                dead_letter_id,
                consumer_name,
                event_position,
                event_id,
                event_type,
                attempts,
                last_error,
                event_payload,
                review_state
            FROM event_dead_letters
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .fetch_optional(&self.pool)
        .await?;

        row.map(EventDeadLetter::try_from).transpose()
    }

    pub async fn dead_letter_by_id(
        &self,
        dead_letter_id: &str,
    ) -> Result<EventDeadLetter, EventStoreError> {
        validate_non_empty("dead_letter_id", dead_letter_id)?;
        let row = sqlx::query_as::<_, EventDeadLetterRow>(
            r#"
            SELECT
                dead_letter_id,
                consumer_name,
                event_position,
                event_id,
                event_type,
                attempts,
                last_error,
                event_payload,
                review_state
            FROM event_dead_letters
            WHERE dead_letter_id = $1
            "#,
        )
        .bind(dead_letter_id.trim())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| EventStoreError::DeadLetterNotFound(dead_letter_id.to_owned()))?;

        EventDeadLetter::try_from(row)
    }

    pub async fn request_dead_letter_replay(
        &self,
        dead_letter_id: &str,
    ) -> Result<(), EventStoreError> {
        validate_non_empty("dead_letter_id", dead_letter_id)?;
        sqlx::query(
            r#"
            UPDATE event_dead_letters
            SET
                review_state = 'replay_requested',
                replay_requested_at = now(),
                updated_at = now()
            WHERE dead_letter_id = $1
            "#,
        )
        .bind(dead_letter_id.trim())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_dead_letter_replayed(&self, dead_letter_id: &str) -> Result<(), EventStoreError> {
        validate_non_empty("dead_letter_id", dead_letter_id)?;
        sqlx::query(
            r#"
            UPDATE event_dead_letters
            SET
                review_state = 'replayed',
                replayed_at = now(),
                updated_at = now()
            WHERE dead_letter_id = $1
            "#,
        )
        .bind(dead_letter_id.trim())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_dead_letter_replayed_for_event(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<u64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let result = sqlx::query(
            r#"
            UPDATE event_dead_letters
            SET
                review_state = 'replayed',
                replayed_at = now(),
                updated_at = now()
            WHERE consumer_name = $1
              AND event_position = $2
              AND review_state <> 'replayed'
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventConsumerRunReport {
    pub processed: usize,
    pub failed: usize,
    pub dead_lettered: usize,
    pub skipped_duplicates: usize,
    pub pending_retry: bool,
    pub last_processed_position: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventDeadLetter {
    pub dead_letter_id: String,
    pub consumer_name: String,
    pub event_position: i64,
    pub event_id: String,
    pub event_type: String,
    pub attempts: i32,
    pub last_error: String,
    pub event: StoredEventEnvelope,
    pub review_state: EventDeadLetterReviewState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventDeadLetterReviewState {
    Open,
    ReplayRequested,
    Replayed,
    Dismissed,
}

impl EventDeadLetterReviewState {
    fn parse(value: &str) -> Result<Self, EventStoreError> {
        match value {
            "open" => Ok(Self::Open),
            "replay_requested" => Ok(Self::ReplayRequested),
            "replayed" => Ok(Self::Replayed),
            "dismissed" => Ok(Self::Dismissed),
            other => Err(EventStoreError::InvalidDeadLetterReviewState(
                other.to_owned(),
            )),
        }
    }
}

#[derive(sqlx::FromRow)]
struct EventDeadLetterRow {
    dead_letter_id: String,
    consumer_name: String,
    event_position: i64,
    event_id: String,
    event_type: String,
    attempts: i32,
    last_error: String,
    event_payload: Value,
    review_state: String,
}

impl TryFrom<EventDeadLetterRow> for EventDeadLetter {
    type Error = EventStoreError;

    fn try_from(row: EventDeadLetterRow) -> Result<Self, Self::Error> {
        Ok(Self {
            dead_letter_id: row.dead_letter_id,
            consumer_name: row.consumer_name,
            event_position: row.event_position,
            event_id: row.event_id,
            event_type: row.event_type,
            attempts: row.attempts,
            last_error: row.last_error,
            event: serde_json::from_value(row.event_payload)?,
            review_state: EventDeadLetterReviewState::parse(&row.review_state)?,
        })
    }
}

fn next_retry_at(now: DateTime<Utc>, base_seconds: i64, attempt_count: i32) -> DateTime<Utc> {
    let exponent = attempt_count.saturating_sub(1).min(10) as u32;
    let multiplier = 2_i64.saturating_pow(exponent);
    now + Duration::seconds(base_seconds.saturating_mul(multiplier))
}

fn dead_letter_id(consumer_name: &str, event_position: i64) -> String {
    let safe_consumer_name = consumer_name
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("event_dead_letter:{safe_consumer_name}:{event_position}")
}
