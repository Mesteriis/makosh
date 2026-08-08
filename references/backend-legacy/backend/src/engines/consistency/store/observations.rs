use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::super::errors::ConsistencyError;
use super::super::evidence::link_consistency_entity_in_transaction;
use super::super::helpers::contradiction_observation_id;
use super::super::models::{ContradictionObservation, NewContradictionObservation};
use super::super::rows::row_to_observation;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

pub(super) async fn upsert(
    pool: &PgPool,
    observation: &NewContradictionObservation,
) -> Result<ContradictionObservation, ConsistencyError> {
    observation.validate()?;
    let observation_id = contradiction_observation_id(observation);
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO contradiction_observations (
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence,
            severity,
            review_state,
            metadata
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            CAST($10 AS NUMERIC(5,4)),
            $11,
            $12,
            $13
        )
        ON CONFLICT (observation_id)
        DO UPDATE SET
            affected_entities = EXCLUDED.affected_entities,
            old_claim = EXCLUDED.old_claim,
            new_claim = EXCLUDED.new_claim,
            confidence = EXCLUDED.confidence,
            severity = EXCLUDED.severity,
            metadata = EXCLUDED.metadata,
            updated_at = now()
        RETURNING
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence::float8 AS confidence,
            severity,
            review_state,
            metadata,
            reviewed_by,
            reviewed_at,
            resolution,
            created_at,
            updated_at
        "#,
    )
    .bind(&observation_id)
    .bind(observation.old_source_kind.as_str())
    .bind(&observation.old_source_id)
    .bind(observation.new_source_kind.as_str())
    .bind(&observation.new_source_id)
    .bind(&observation.affected_entities)
    .bind(&observation.conflict_type)
    .bind(&observation.old_claim)
    .bind(&observation.new_claim)
    .bind(observation.confidence)
    .bind(observation.severity.as_str())
    .bind(observation.review_state.as_str())
    .bind(&observation.metadata)
    .fetch_one(&mut *transaction)
    .await?;

    let stored = row_to_observation(row)?;
    link_contradiction_observation_in_transaction(&mut transaction, &stored).await?;
    transaction.commit().await?;
    Ok(stored)
}

pub(super) async fn list_open(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ContradictionObservation>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence::float8 AS confidence,
            severity,
            review_state,
            metadata,
            reviewed_by,
            reviewed_at,
            resolution,
            created_at,
            updated_at
        FROM contradiction_observations
        WHERE review_state = 'suggested'
        ORDER BY updated_at DESC, observation_id ASC
        LIMIT $1
        "#,
    )
    .bind(limit.clamp(1, 100))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_observation).collect()
}

pub(crate) async fn link_contradiction_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ConsistencyError> {
    let evidence_observation =
        capture_contradiction_observation_in_transaction(transaction, contradiction).await?;
    link_consistency_entity_in_transaction(
        transaction,
        &evidence_observation.observation_id,
        "contradiction_observation",
        contradiction.observation_id.clone(),
        "upsert",
        json!({
            "conflict_type": contradiction.conflict_type,
            "review_state": contradiction.review_state.as_str(),
            "severity": contradiction.severity.as_str(),
            "old_source_kind": contradiction.old_source_kind.as_str(),
            "old_source_id": contradiction.old_source_id,
            "new_source_kind": contradiction.new_source_kind.as_str(),
            "new_source_id": contradiction.new_source_id,
        }),
    )
    .await?;
    Ok(())
}

async fn capture_contradiction_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<makosh_observations_api::models::Observation, ConsistencyError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "CONTRADICTION_OBSERVATION",
            ObservationOriginKind::LocalRuntime,
            contradiction.created_at,
            json!({
                "contradiction_observation_id": contradiction.observation_id,
                "conflict_type": contradiction.conflict_type,
                "old_claim": contradiction.old_claim,
                "new_claim": contradiction.new_claim,
                "severity": contradiction.severity.as_str(),
                "review_state": contradiction.review_state.as_str(),
                "affected_entities": contradiction.affected_entities,
            }),
            format!("contradiction://{}", contradiction.observation_id),
        )
        .confidence(contradiction.confidence)
        .provenance(json!({
            "engine": "consistency",
            "pipeline": "contradiction_observations",
        })),
    )
    .await?;
    Ok(observation)
}
