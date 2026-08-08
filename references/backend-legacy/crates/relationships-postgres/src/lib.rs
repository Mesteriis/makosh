use chrono::{DateTime, Utc};
use makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction;
use makosh_relationships_api::{
    RelationshipEvidence, RelationshipListFuture, RelationshipListQuery, RelationshipQueryError,
    RelationshipQueryPort, RelationshipRead, RelationshipUpsert, RelationshipWriteError,
    RelationshipWriteFuture, RelationshipWritePort,
};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};

#[derive(Clone)]
pub struct RelationshipPostgresQuery {
    pool: PgPool,
}

impl RelationshipPostgresQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship: &RelationshipUpsert,
        evidence: &[RelationshipEvidence],
    ) -> Result<RelationshipRead, RelationshipWriteError> {
        if relationship.relationship_id.trim().is_empty() {
            return Err(RelationshipWriteError::InvalidWrite(
                "relationship_id is required",
            ));
        }
        let row = sqlx::query("INSERT INTO relationships (relationship_id,source_entity_kind,source_entity_id,target_entity_kind,target_entity_id,relationship_type,trust_score,strength_score,confidence,review_state,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,CAST($7 AS NUMERIC(5,4)),CAST($8 AS NUMERIC(5,4)),CAST($9 AS NUMERIC(5,4)),$10,$11,$12,$13) ON CONFLICT (relationship_id) DO UPDATE SET trust_score=EXCLUDED.trust_score,strength_score=EXCLUDED.strength_score,confidence=EXCLUDED.confidence,review_state=EXCLUDED.review_state,valid_from=EXCLUDED.valid_from,valid_to=EXCLUDED.valid_to,metadata=EXCLUDED.metadata,updated_at=now() RETURNING relationship_id,source_entity_kind,source_entity_id,target_entity_kind,target_entity_id,relationship_type,trust_score::float8 AS trust_score,strength_score::float8 AS strength_score,confidence::float8 AS confidence,review_state,valid_from,valid_to,metadata,created_at,updated_at")
            .bind(&relationship.relationship_id).bind(relationship.source_entity_kind.as_str()).bind(&relationship.source_entity_id)
            .bind(relationship.target_entity_kind.as_str()).bind(&relationship.target_entity_id).bind(&relationship.relationship_type)
            .bind(relationship.trust_score).bind(relationship.strength_score).bind(relationship.confidence)
            .bind(relationship.review_state.as_str()).bind(relationship.valid_from).bind(relationship.valid_to).bind(&relationship.metadata)
            .fetch_one(&mut **transaction).await.map_err(map_write_error)?;
        for item in evidence {
            sqlx::query("INSERT INTO relationship_evidence (evidence_id,relationship_id,source_kind,source_id,observation_id,excerpt,metadata) VALUES (md5($1 || ':' || $2 || ':' || $3),$1,$2,$3,$4,$5,$6) ON CONFLICT (relationship_id,source_kind,source_id) DO UPDATE SET observation_id=EXCLUDED.observation_id,excerpt=EXCLUDED.excerpt,metadata=EXCLUDED.metadata")
                .bind(&relationship.relationship_id).bind(item.source_kind.as_str()).bind(&item.source_id)
                .bind(&item.observation_id).bind(&item.excerpt).bind(&item.metadata)
                .execute(&mut **transaction).await.map_err(map_write_error)?;
        }
        to_read(row).map_err(|error| RelationshipWriteError::Failed(error.to_string()))
    }

    pub async fn set_review_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship_id: &str,
        review_state: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<RelationshipRead, RelationshipWriteError> {
        let row = sqlx::query("UPDATE relationships SET review_state=$1,updated_at=now() WHERE relationship_id=$2 RETURNING relationship_id,source_entity_kind,source_entity_id,target_entity_kind,target_entity_id,relationship_type,trust_score::float8 AS trust_score,strength_score::float8 AS strength_score,confidence::float8 AS confidence,review_state,valid_from,valid_to,metadata,created_at,updated_at")
            .bind(review_state).bind(relationship_id).fetch_optional(&mut **transaction)
            .await.map_err(map_write_error)?.ok_or_else(|| RelationshipWriteError::Failed("relationship not found".into()))?;
        materialize_review_transition_link_in_transaction(
            transaction,
            observation_id,
            "relationships",
            "relationship",
            relationship_id,
            "review_state",
            review_state,
            metadata,
        )
        .await
        .map_err(|error| RelationshipWriteError::Failed(error.to_string()))?;
        to_read(row).map_err(|error| RelationshipWriteError::Failed(error.to_string()))
    }
}

impl RelationshipQueryPort for RelationshipPostgresQuery {
    fn list<'a>(&'a self, query: RelationshipListQuery) -> RelationshipListFuture<'a> {
        Box::pin(async move {
            let limit = query.limit.unwrap_or(50);
            if !(1..=100).contains(&limit) {
                return Err(RelationshipQueryError::InvalidQuery(
                    "limit must be between 1 and 100",
                ));
            }

            let rows = match (
                query.review_state.as_deref(),
                query.entity_kind.as_deref(),
                query.entity_id.as_deref(),
            ) {
                (Some(review_state), None, None) => sqlx::query(
                    SELECT_SQL
                        .replace("WHERE_FILTER", "review_state = $1")
                        .replace("LIMIT_PARAM", "$2")
                        .as_str(),
                )
                .bind(review_state)
                .bind(limit)
                .fetch_all(&self.pool)
                .await,
                (None, Some(entity_kind), Some(entity_id)) if !entity_id.trim().is_empty() => {
                    sqlx::query(
                        SELECT_SQL
                            .replace(
                                "WHERE_FILTER",
                                "(source_entity_kind = $1 AND source_entity_id = $2)\n               OR (target_entity_kind = $1 AND target_entity_id = $2)",
                            )
                            .replace("LIMIT_PARAM", "$3")
                            .as_str(),
                    )
                    .bind(entity_kind)
                    .bind(entity_id)
                    .bind(limit)
                    .fetch_all(&self.pool)
                    .await
                }
                (Some(_), _, _) => {
                    return Err(RelationshipQueryError::InvalidQuery(
                        "review_state cannot be combined with entity filters",
                    ));
                }
                (None, _, _) => {
                    return Err(RelationshipQueryError::InvalidQuery(
                        "missing required relationship query field",
                    ));
                }
            }
            .map_err(|error| RelationshipQueryError::Failed(error.to_string()))?;

            rows.into_iter().map(to_read).collect()
        })
    }
}

impl RelationshipWritePort for RelationshipPostgresQuery {
    fn upsert<'a>(
        &'a self,
        relationship: &'a RelationshipUpsert,
        evidence: &'a [RelationshipEvidence],
    ) -> RelationshipWriteFuture<'a> {
        Box::pin(async move {
            if relationship.relationship_id.trim().is_empty() {
                return Err(RelationshipWriteError::InvalidWrite(
                    "relationship_id is required",
                ));
            }
            if relationship.source_entity_id.trim().is_empty()
                || relationship.target_entity_id.trim().is_empty()
            {
                return Err(RelationshipWriteError::InvalidWrite(
                    "entity ids are required",
                ));
            }
            let mut tx = self.pool.begin().await.map_err(map_write_error)?;
            let row = sqlx::query(
                "INSERT INTO relationships (relationship_id,source_entity_kind,source_entity_id,target_entity_kind,target_entity_id,relationship_type,trust_score,strength_score,confidence,review_state,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,CAST($7 AS NUMERIC(5,4)),CAST($8 AS NUMERIC(5,4)),CAST($9 AS NUMERIC(5,4)),$10,$11,$12,$13) ON CONFLICT (relationship_id) DO UPDATE SET trust_score=EXCLUDED.trust_score,strength_score=EXCLUDED.strength_score,confidence=EXCLUDED.confidence,review_state=EXCLUDED.review_state,valid_from=EXCLUDED.valid_from,valid_to=EXCLUDED.valid_to,metadata=EXCLUDED.metadata,updated_at=now() RETURNING relationship_id,source_entity_kind,source_entity_id,target_entity_kind,target_entity_id,relationship_type,trust_score::float8 AS trust_score,strength_score::float8 AS strength_score,confidence::float8 AS confidence,review_state,valid_from,valid_to,metadata,created_at,updated_at",
            )
            .bind(&relationship.relationship_id)
            .bind(relationship.source_entity_kind.as_str())
            .bind(&relationship.source_entity_id)
            .bind(relationship.target_entity_kind.as_str())
            .bind(&relationship.target_entity_id)
            .bind(&relationship.relationship_type)
            .bind(relationship.trust_score)
            .bind(relationship.strength_score)
            .bind(relationship.confidence)
            .bind(relationship.review_state.as_str())
            .bind(relationship.valid_from)
            .bind(relationship.valid_to)
            .bind(&relationship.metadata)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_write_error)?;
            for item in evidence {
                sqlx::query("INSERT INTO relationship_evidence (evidence_id,relationship_id,source_kind,source_id,observation_id,excerpt,metadata) VALUES (md5($1 || ':' || $2 || ':' || $3),$1,$2,$3,$4,$5,$6) ON CONFLICT (relationship_id,source_kind,source_id) DO UPDATE SET observation_id=EXCLUDED.observation_id,excerpt=EXCLUDED.excerpt,metadata=EXCLUDED.metadata")
                    .bind(&relationship.relationship_id)
                    .bind(item.source_kind.as_str())
                    .bind(&item.source_id)
                    .bind(&item.observation_id)
                    .bind(&item.excerpt)
                    .bind(&item.metadata)
                    .execute(&mut *tx).await.map_err(map_write_error)?;
            }
            tx.commit().await.map_err(map_write_error)?;
            to_read(row).map_err(|error| RelationshipWriteError::Failed(error.to_string()))
        })
    }
}

fn map_write_error(error: sqlx::Error) -> RelationshipWriteError {
    RelationshipWriteError::Failed(error.to_string())
}

const SELECT_SQL: &str = r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE WHERE_FILTER
            ORDER BY updated_at DESC, relationship_id ASC
            LIMIT LIMIT_PARAM
            "#;

fn to_read(row: sqlx::postgres::PgRow) -> Result<RelationshipRead, RelationshipQueryError> {
    Ok(RelationshipRead {
        relationship_id: row.try_get("relationship_id").map_err(map_row_error)?,
        source_entity_kind: row.try_get("source_entity_kind").map_err(map_row_error)?,
        source_entity_id: row.try_get("source_entity_id").map_err(map_row_error)?,
        target_entity_kind: row.try_get("target_entity_kind").map_err(map_row_error)?,
        target_entity_id: row.try_get("target_entity_id").map_err(map_row_error)?,
        relationship_type: row.try_get("relationship_type").map_err(map_row_error)?,
        trust_score: row.try_get("trust_score").map_err(map_row_error)?,
        strength_score: row.try_get("strength_score").map_err(map_row_error)?,
        confidence: row.try_get("confidence").map_err(map_row_error)?,
        review_state: row.try_get("review_state").map_err(map_row_error)?,
        valid_from: row
            .try_get::<Option<DateTime<Utc>>, _>("valid_from")
            .map_err(map_row_error)?,
        valid_to: row
            .try_get::<Option<DateTime<Utc>>, _>("valid_to")
            .map_err(map_row_error)?,
        metadata: row.try_get::<Value, _>("metadata").map_err(map_row_error)?,
        created_at: row.try_get("created_at").map_err(map_row_error)?,
        updated_at: row.try_get("updated_at").map_err(map_row_error)?,
    })
}

fn map_row_error(error: sqlx::Error) -> RelationshipQueryError {
    RelationshipQueryError::Failed(error.to_string())
}
