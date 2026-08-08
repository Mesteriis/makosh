use makosh_events_api::NewEventEnvelope;
use std::collections::HashSet;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use makosh_events_postgres::store::EventStore;

use super::errors::ReviewInboxError;
use super::models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItem, ReviewItemKind, ReviewItemStatus,
    ReviewPromotionTarget, validate_non_empty, validate_review_item_with_evidence,
};
use makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction;

#[derive(Clone)]
pub struct ReviewInboxStore {
    pool: PgPool,
}

impl ReviewInboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_with_evidence(
        &self,
        item: &NewReviewItem,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_review_item_with_evidence(item, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::create_with_evidence_in_transaction(&mut transaction, item, evidence).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_by_status(
        &self,
        status: ReviewItemStatus,
        limit: i64,
    ) -> Result<Vec<ReviewItem>, ReviewInboxError> {
        let sql = review_item_select_sql(
            "WHERE status = $1 ORDER BY updated_at DESC, review_item_id ASC LIMIT $2",
        );
        let rows = sqlx::query(&sql)
            .bind(status.as_str())
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(row_to_review_item).collect()
    }

    pub async fn list_open(&self, limit: i64) -> Result<Vec<ReviewItem>, ReviewInboxError> {
        let sql = review_item_select_sql(
            "WHERE status IN ('new', 'in_review') ORDER BY updated_at DESC, review_item_id ASC LIMIT $1",
        );
        let rows = sqlx::query(&sql)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(row_to_review_item).collect()
    }

    pub async fn list_all(&self, limit: i64) -> Result<Vec<ReviewItem>, ReviewInboxError> {
        let sql = review_item_select_sql("ORDER BY updated_at DESC, review_item_id ASC LIMIT $1");
        let rows = sqlx::query(&sql)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_review_item).collect()
    }

    pub async fn set_status(
        &self,
        review_item_id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        self.set_status_with_observation(review_item_id, status, None, None)
            .await
    }

    pub async fn set_status_with_observation(
        &self,
        review_item_id: &str,
        status: ReviewItemStatus,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;

        let mut transaction = self.pool.begin().await?;
        let item = Self::transition_status_in_transaction_with_observation(
            &mut transaction,
            review_item_id,
            status,
            observation_id,
            None,
        )
        .await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "review",
            "review_item",
            &item.review_item_id,
            "status",
            item.status.as_str(),
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(item)
    }

    pub async fn promote(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewInboxError> {
        self.promote_with_observation(review_item_id, target, None, None, None, None)
            .await
    }

    pub async fn promote_with_observation(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
        observation_id: Option<&str>,
        metadata: Option<Value>,
        causation_id: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;
        target.validate()?;

        let mut transaction = self.pool.begin().await?;
        let correlation_id = match correlation_id {
            Some(value) => Some(value.to_owned()),
            None => Some(
                Self::review_item_trace_id_in_transaction(&mut transaction, review_item_id).await?,
            ),
        };
        let causation_id = causation_id
            .or(observation_id)
            .map(std::string::ToString::to_string);

        let item = Self::promote_in_transaction_with_observation(
            &mut transaction,
            review_item_id,
            target,
            causation_id.as_deref(),
            correlation_id.as_deref(),
        )
        .await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "review",
            "review_item",
            &item.review_item_id,
            "status",
            item.status.as_str(),
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(item)
    }

    pub(crate) async fn create_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        item: &NewReviewItem,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let review_item_id = review_item_id(item, evidence)?;
        let inserted = sqlx::query(review_item_insert_sql())
            .bind(&review_item_id)
            .bind(item.item_kind.as_str())
            .bind(item.title.trim())
            .bind(item.summary.trim())
            .bind(item.confidence)
            .bind(&item.metadata)
            .fetch_optional(&mut **transaction)
            .await?;
        let was_inserted = inserted.is_some();

        for item in evidence {
            sqlx::query(
                r#"
                INSERT INTO review_item_evidence (
                    review_item_id,
                    observation_id,
                    evidence_role,
                    metadata
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (review_item_id, observation_id, evidence_role)
                DO UPDATE SET metadata = EXCLUDED.metadata
                "#,
            )
            .bind(&review_item_id)
            .bind(item.observation_id.trim())
            .bind(item.evidence_role.trim())
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        let stored = if let Some(row) = inserted {
            row_to_review_item(row)?
        } else {
            Self::fetch_review_item_in_transaction(transaction, &review_item_id).await?
        };

        if was_inserted {
            append_candidate_detected_event(transaction, &stored, evidence).await?;
            append_review_available_event(transaction, &stored, evidence).await?;
        }

        Ok(stored)
    }

    pub(crate) async fn attach_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;
        validate_evidence_observations_exist(transaction, evidence).await?;

        for item in evidence {
            sqlx::query(
                r#"
                INSERT INTO review_item_evidence (
                    review_item_id,
                    observation_id,
                    evidence_role,
                    metadata
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (review_item_id, observation_id, evidence_role)
                DO UPDATE SET metadata = EXCLUDED.metadata
                "#,
            )
            .bind(review_item_id)
            .bind(item.observation_id.trim())
            .bind(item.evidence_role.trim())
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Self::fetch_review_item_in_transaction(transaction, review_item_id).await
    }

    pub(crate) async fn transition_status_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        Self::transition_status_in_transaction_with_observation(
            transaction,
            review_item_id,
            status,
            None,
            None,
        )
        .await
    }

    pub(crate) async fn transition_status_in_transaction_with_observation(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        status: ReviewItemStatus,
        causation_id: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        let item = Self::set_status_in_transaction(transaction, review_item_id, status).await?;
        append_review_status_event(transaction, &item, causation_id, correlation_id).await?;
        Ok(item)
    }

    pub(crate) async fn promote_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewInboxError> {
        Self::promote_in_transaction_with_observation(
            transaction,
            review_item_id,
            target,
            None,
            None,
        )
        .await
    }

    pub(crate) async fn promote_in_transaction_with_observation(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        target: ReviewPromotionTarget,
        causation_id: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;
        target.validate()?;

        let sql = review_item_update_returning_sql(
            r#"
            SET
                status = 'promoted',
                target_domain = $2,
                target_entity_kind = $3,
                target_entity_id = $4,
                updated_at = now()
            WHERE review_item_id = $1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(review_item_id)
            .bind(target.target_domain.trim())
            .bind(target.target_entity_kind.trim())
            .bind(target.target_entity_id.trim())
            .fetch_optional(&mut **transaction)
            .await?;
        let item = row
            .map(row_to_review_item)
            .transpose()?
            .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(review_item_id.to_owned()))?;
        append_review_status_event(transaction, &item, causation_id, correlation_id).await?;
        Ok(item)
    }

    async fn review_item_trace_id_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
    ) -> Result<String, ReviewInboxError> {
        let trace_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT COALESCE(correlation_id, event_id)
            FROM event_log
            WHERE subject ->> 'review_item_id' = $1
            ORDER BY position ASC
            LIMIT 1
            "#,
        )
        .bind(review_item_id)
        .fetch_optional(&mut **transaction)
        .await?;

        Ok(trace_id.unwrap_or_else(|| review_item_id.to_owned()))
    }

    pub(crate) async fn find_latest_by_kind_and_metadata_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        item_kind: ReviewItemKind,
        metadata_filter: &Value,
    ) -> Result<Option<ReviewItem>, ReviewInboxError> {
        if !metadata_filter.is_object() {
            return Err(ReviewInboxError::InvalidMetadataFilter);
        }

        let sql = review_item_select_sql(
            "WHERE item_kind = $1 AND metadata @> $2::jsonb ORDER BY updated_at DESC, review_item_id ASC LIMIT 1",
        );
        let row = sqlx::query(&sql)
            .bind(item_kind.as_str())
            .bind(metadata_filter)
            .fetch_optional(&mut **transaction)
            .await?;
        row.map(row_to_review_item).transpose()
    }

    async fn set_status_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        let sql = review_item_update_returning_sql(
            r#"
            SET
                status = $2,
                updated_at = now()
            WHERE review_item_id = $1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(review_item_id)
            .bind(status.as_str())
            .fetch_optional(&mut **transaction)
            .await?;

        row.map(row_to_review_item)
            .transpose()?
            .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(review_item_id.to_owned()))
    }

    pub async fn get(&self, review_item_id: &str) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;

        let mut transaction = self.pool.begin().await?;
        let item = Self::fetch_review_item_in_transaction(&mut transaction, review_item_id).await?;
        transaction.commit().await?;
        Ok(item)
    }

    pub async fn list_evidence(
        &self,
        review_item_id: &str,
    ) -> Result<Vec<ReviewItemEvidenceRecord>, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;

        let mut transaction = self.pool.begin().await?;
        let evidence = load_review_evidence(&mut transaction, review_item_id).await?;
        transaction.commit().await?;
        Ok(evidence)
    }

    async fn fetch_review_item_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
    ) -> Result<ReviewItem, ReviewInboxError> {
        let sql = review_item_select_sql("WHERE review_item_id = $1");
        let row = sqlx::query(&sql)
            .bind(review_item_id)
            .fetch_optional(&mut **transaction)
            .await?;
        row.map(row_to_review_item)
            .transpose()?
            .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(review_item_id.to_owned()))
    }
}

#[derive(Clone, Debug)]
pub struct ReviewItemEvidenceRecord {
    pub observation_id: String,
    pub evidence_role: String,
    pub metadata: Value,
}

fn review_item_insert_sql() -> &'static str {
    r#"
    INSERT INTO review_items (
        review_item_id,
        item_kind,
        title,
        summary,
        confidence,
        metadata
    )
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (review_item_id) DO NOTHING
    RETURNING
        review_item_id,
        item_kind,
        title,
        summary,
        status,
        target_domain,
        target_entity_kind,
        target_entity_id,
        confidence::float8 AS confidence,
        metadata,
        created_at,
        updated_at
    "#
}

fn review_item_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            review_item_id,
            item_kind,
            title,
            summary,
            status,
            target_domain,
            target_entity_kind,
            target_entity_id,
            confidence::float8 AS confidence,
            metadata,
            created_at,
            updated_at
        FROM review_items
        {where_clause}
        "#
    )
}

fn review_item_update_returning_sql(update_clause: &str) -> String {
    format!(
        r#"
        UPDATE review_items
        {update_clause}
        RETURNING
            review_item_id,
            item_kind,
            title,
            summary,
            status,
            target_domain,
            target_entity_kind,
            target_entity_id,
            confidence::float8 AS confidence,
            metadata,
            created_at,
            updated_at
        "#
    )
}

fn row_to_review_item(row: PgRow) -> Result<ReviewItem, ReviewInboxError> {
    let item_kind: String = row.try_get("item_kind")?;
    let status: String = row.try_get("status")?;

    Ok(ReviewItem {
        review_item_id: row.try_get("review_item_id")?,
        item_kind: super::models::ReviewItemKind::parse(item_kind)?,
        title: row.try_get("title")?,
        summary: row.try_get("summary")?,
        status: ReviewItemStatus::parse(status)?,
        target_domain: row.try_get("target_domain")?,
        target_entity_kind: row.try_get("target_entity_kind")?,
        target_entity_id: row.try_get("target_entity_id")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

async fn load_review_evidence(
    transaction: &mut Transaction<'_, Postgres>,
    review_item_id: &str,
) -> Result<Vec<ReviewItemEvidenceRecord>, ReviewInboxError> {
    let rows = sqlx::query(
        r#"
        SELECT observation_id, evidence_role, metadata
        FROM review_item_evidence
        WHERE review_item_id = $1
        ORDER BY created_at ASC, observation_id ASC
        "#,
    )
    .bind(review_item_id)
    .fetch_all(&mut **transaction)
    .await?;

    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        records.push(ReviewItemEvidenceRecord {
            observation_id: row.try_get("observation_id")?,
            evidence_role: row.try_get("evidence_role")?,
            metadata: row.try_get("metadata")?,
        });
    }
    Ok(records)
}

async fn validate_evidence_observations_exist(
    transaction: &mut Transaction<'_, Postgres>,
    evidence: &[NewReviewItemEvidence],
) -> Result<(), ReviewInboxError> {
    let observation_ids: Vec<String> = evidence
        .iter()
        .map(|item| item.observation_id.trim().to_owned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let stored_observation_ids: HashSet<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM observations
        WHERE observation_id = ANY($1)
        "#,
    )
    .bind(&observation_ids)
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .collect();

    for observation_id in observation_ids {
        if !stored_observation_ids.contains(&observation_id) {
            return Err(ReviewInboxError::ObservationNotFound(observation_id));
        }
    }

    Ok(())
}

async fn append_candidate_detected_event(
    transaction: &mut Transaction<'_, Postgres>,
    item: &ReviewItem,
    evidence: &[NewReviewItemEvidence],
) -> Result<(), ReviewInboxError> {
    let Some(event_type) = candidate_event_type(item.item_kind) else {
        return Ok(());
    };
    let evidence_observation_ids: Vec<&str> = evidence
        .iter()
        .map(|candidate| candidate.observation_id.as_str())
        .collect();
    let event = NewEventEnvelope::builder(
        format!(
            "event:v1:{}:{}",
            event_type.replace('.', "-"),
            item.review_item_id
        ),
        event_type,
        item.created_at,
        json!({
            "domain": "ingestion",
            "source_id": format!("{event_type}:{}", item.review_item_id)
        }),
        json!({
            "review_item_id": item.review_item_id,
            "item_kind": item.item_kind.as_str()
        }),
    )
    .payload(json!({
        "title": item.title,
        "summary": item.summary,
        "confidence": item.confidence,
        "evidence_observation_ids": evidence_observation_ids
    }))
    .provenance(json!({
        "observation_ingestion": true
    }))
    .build()?;

    append_event_idempotently(transaction, &event).await
}

async fn append_review_available_event(
    transaction: &mut Transaction<'_, Postgres>,
    item: &ReviewItem,
    evidence: &[NewReviewItemEvidence],
) -> Result<(), ReviewInboxError> {
    let evidence_observation_ids: Vec<&str> = evidence
        .iter()
        .map(|evidence| evidence.observation_id.as_str())
        .collect();
    let event = NewEventEnvelope::builder(
        format!("event:v1:review-item-available:{}", item.review_item_id),
        "review.item.available.v1",
        item.created_at,
        json!({
            "domain": "review",
            "source_id": format!("review.item.available.v1:{}", item.review_item_id)
        }),
        json!({
            "review_item_id": item.review_item_id,
            "item_kind": item.item_kind.as_str()
        }),
    )
    .payload(json!({
        "status": item.status.as_str(),
        "confidence": item.confidence,
        "evidence_observation_ids": evidence_observation_ids
    }))
    .provenance(json!({
        "review_inbox": true
    }))
    .build()?;

    append_event_idempotently(transaction, &event).await?;
    Ok(())
}

async fn append_review_status_event(
    transaction: &mut Transaction<'_, Postgres>,
    item: &ReviewItem,
    causation_id: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<(), ReviewInboxError> {
    let Some(event_type) = review_status_event_type(item.status) else {
        return Ok(());
    };
    let mut builder = NewEventEnvelope::builder(
        format!(
            "event:v1:{}:{}",
            event_type.replace('.', "-"),
            item.review_item_id
        ),
        event_type,
        item.updated_at,
        json!({
            "domain": "review",
            "source_id": format!("{event_type}:{}", item.review_item_id)
        }),
        json!({
            "review_item_id": item.review_item_id,
            "item_kind": item.item_kind.as_str()
        }),
    )
    .payload(json!({
        "status": item.status.as_str(),
        "target_domain": item.target_domain,
        "target_entity_kind": item.target_entity_kind,
        "target_entity_id": item.target_entity_id
    }))
    .provenance(json!({
        "review_inbox": true
    }));
    if let Some(causation_id) = causation_id {
        builder = builder.causation_id(causation_id);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }

    let event = builder.build()?;

    append_event_idempotently(transaction, &event).await?;
    Ok(())
}

async fn append_event_idempotently(
    transaction: &mut Transaction<'_, Postgres>,
    event: &NewEventEnvelope,
) -> Result<(), ReviewInboxError> {
    match EventStore::append_in_transaction(transaction, event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn candidate_event_type(kind: ReviewItemKind) -> Option<&'static str> {
    match kind {
        ReviewItemKind::NewPersona => Some("persona.detected.v1"),
        ReviewItemKind::NewOrganization => Some("organization.detected.v1"),
        ReviewItemKind::IdentityCandidate => Some("identity.candidate.detected.v1"),
        ReviewItemKind::ProjectLinkCandidate => Some("project_link.candidate.detected.v1"),
        ReviewItemKind::ContradictionCandidate => Some("contradiction.detected.v1"),
        ReviewItemKind::PotentialTask => Some("task.candidate.detected.v1"),
        ReviewItemKind::PotentialObligation => Some("obligation.candidate.detected.v1"),
        ReviewItemKind::PotentialDecision => Some("decision.candidate.detected.v1"),
        ReviewItemKind::PotentialRelationship => Some("relationship.candidate.detected.v1"),
        ReviewItemKind::PotentialProject => Some("project.candidate.detected.v1"),
        ReviewItemKind::KnowledgeCandidate => Some("knowledge.candidate.detected.v1"),
    }
}

fn review_status_event_type(status: ReviewItemStatus) -> Option<&'static str> {
    match status {
        ReviewItemStatus::Approved => Some("review.item.approved.v1"),
        ReviewItemStatus::Promoted => Some("review.item.promoted.v1"),
        ReviewItemStatus::Dismissed => Some("review.item.dismissed.v1"),
        ReviewItemStatus::New | ReviewItemStatus::InReview | ReviewItemStatus::Archived => None,
    }
}

fn review_item_id(
    item: &NewReviewItem,
    evidence: &[NewReviewItemEvidence],
) -> Result<String, ReviewInboxError> {
    let mut digest = Sha256::new();
    digest.update(item.item_kind.as_str().as_bytes());
    digest.update(b"\n");
    digest.update(item.title.trim().as_bytes());
    digest.update(b"\n");
    digest.update(item.summary.trim().as_bytes());
    digest.update(b"\n");
    digest.update(serde_json::to_vec(&item.metadata)?);
    for evidence in evidence {
        digest.update(b"\n");
        digest.update(evidence.observation_id.trim().as_bytes());
        digest.update(b"\n");
        digest.update(evidence.evidence_role.trim().as_bytes());
    }
    Ok(format!("review_item:v1:{:x}", digest.finalize()))
}
