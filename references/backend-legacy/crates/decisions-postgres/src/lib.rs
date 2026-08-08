use chrono::{DateTime, Utc};
use makosh_decisions_api::{
    DecisionEvidence, DecisionImpactedEntity, DecisionListFuture, DecisionListQuery,
    DecisionQueryError, DecisionRead, DecisionReadPort, DecisionUpsert, DecisionWriteError,
    DecisionWriteFuture, DecisionWritePort,
};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct DecisionPostgresReadQuery {
    pool: PgPool,
}

impl DecisionPostgresReadQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        decision: &DecisionUpsert,
        evidence: &[DecisionEvidence],
        impacted_entities: &[DecisionImpactedEntity],
    ) -> Result<DecisionRead, DecisionWriteError> {
        if decision.decision_id.trim().is_empty() || decision.title.trim().is_empty() {
            return Err(DecisionWriteError::InvalidWrite(
                "decision id and title are required",
            ));
        }
        let row = sqlx::query("INSERT INTO decisions (decision_id,title,status,rationale,alternatives,decided_by_entity_kind,decided_by_entity_id,decided_at,review_state,confidence,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,CAST($10 AS NUMERIC(5,4)),$11) ON CONFLICT (decision_id) DO UPDATE SET title=EXCLUDED.title,status=EXCLUDED.status,rationale=EXCLUDED.rationale,alternatives=EXCLUDED.alternatives,decided_by_entity_kind=EXCLUDED.decided_by_entity_kind,decided_by_entity_id=EXCLUDED.decided_by_entity_id,decided_at=EXCLUDED.decided_at,review_state=EXCLUDED.review_state,confidence=EXCLUDED.confidence,metadata=EXCLUDED.metadata,updated_at=now() RETURNING decision_id,title,status,rationale,alternatives,decided_by_entity_kind,decided_by_entity_id,decided_at,review_state,confidence::float8 AS confidence,metadata,created_at,updated_at")
            .bind(&decision.decision_id).bind(&decision.title).bind(&decision.status).bind(&decision.rationale)
            .bind(&decision.alternatives).bind(&decision.decided_by_entity_kind).bind(&decision.decided_by_entity_id)
            .bind(decision.decided_at).bind(&decision.review_state).bind(decision.confidence).bind(&decision.metadata)
            .fetch_one(&mut **transaction).await.map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
        for item in evidence {
            sqlx::query("INSERT INTO decision_evidence (evidence_id,decision_id,source_kind,source_id,observation_id,quote,confidence,metadata) VALUES (md5($1 || ':' || $2 || ':' || $3),$1,$2,$3,$4,$5,CAST($6 AS NUMERIC(5,4)),$7) ON CONFLICT (decision_id,source_kind,source_id) DO UPDATE SET observation_id=EXCLUDED.observation_id,quote=EXCLUDED.quote,confidence=EXCLUDED.confidence,metadata=EXCLUDED.metadata")
                .bind(&decision.decision_id).bind(&item.source_kind).bind(&item.source_id).bind(&item.observation_id).bind(&item.excerpt).bind(item.confidence).bind(&item.metadata)
                .execute(&mut **transaction).await.map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
            if let Some(observation_id) = item.observation_id.as_deref() {
                link_domain_entity_in_transaction(
                    transaction,
                    observation_id,
                    "decisions",
                    "decision",
                    decision.decision_id.clone(),
                    Some("support"),
                    Some(item.confidence),
                    Some(item.metadata.clone()),
                )
                .await
                .map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
            }
        }
        for item in impacted_entities {
            sqlx::query("INSERT INTO decision_impacted_entities (decision_id,entity_kind,entity_id,impact_type,metadata) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (decision_id,entity_kind,entity_id) DO UPDATE SET impact_type=EXCLUDED.impact_type,metadata=EXCLUDED.metadata")
                .bind(&decision.decision_id).bind(&item.entity_kind).bind(&item.entity_id).bind(&item.impact_type).bind(&item.metadata)
                .execute(&mut **transaction).await.map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
        }
        to_read(row).map_err(|e| DecisionWriteError::Failed(e.to_string()))
    }

    pub async fn set_review_state_with_observation(
        &self,
        decision_id: &str,
        review_state: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<DecisionRead, DecisionWriteError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
        let row = sqlx::query("UPDATE decisions SET review_state=$1, updated_at=now() WHERE decision_id=$2 RETURNING decision_id,title,status,rationale,alternatives,decided_by_entity_kind,decided_by_entity_id,decided_at,review_state,confidence::float8 AS confidence,metadata,created_at,updated_at").bind(review_state).bind(decision_id).fetch_optional(&mut *tx).await.map_err(|e| DecisionWriteError::Failed(e.to_string()))?.ok_or_else(|| DecisionWriteError::Failed("decision was not found".to_owned()))?;
        let result = to_read(row).map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
        makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction(&mut tx, observation_id, "decisions", "decision", &result.decision_id, "review_state", review_state, metadata).await.map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
        Ok(result)
    }
}

impl DecisionWritePort for DecisionPostgresReadQuery {
    fn upsert<'a>(
        &'a self,
        decision: &'a DecisionUpsert,
        evidence: &'a [DecisionEvidence],
        impacted_entities: &'a [DecisionImpactedEntity],
    ) -> DecisionWriteFuture<'a> {
        Box::pin(async move {
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
            let value =
                Self::upsert_in_transaction(&mut tx, decision, evidence, impacted_entities).await?;
            tx.commit()
                .await
                .map_err(|e| DecisionWriteError::Failed(e.to_string()))?;
            Ok(value)
        })
    }
}

impl DecisionReadPort for DecisionPostgresReadQuery {
    fn list<'a>(&'a self, query: DecisionListQuery) -> DecisionListFuture<'a> {
        Box::pin(async move {
            let limit = query.limit.unwrap_or(50);
            if !(1..=100).contains(&limit) {
                return Err(DecisionQueryError(
                    "limit must be between 1 and 100".to_owned(),
                ));
            }
            let rows = match (
                query.review_state.as_deref(),
                query.entity_kind.as_deref(),
                query.entity_id.as_deref(),
            ) {
                (Some(state), None, None) => {
                    sqlx::query(REVIEW_SQL)
                        .bind(state)
                        .bind(limit)
                        .fetch_all(&self.pool)
                        .await
                }
                (None, Some(kind), Some(id)) if !id.trim().is_empty() => {
                    sqlx::query(ENTITY_SQL)
                        .bind(kind)
                        .bind(id)
                        .bind(limit)
                        .fetch_all(&self.pool)
                        .await
                }
                _ => return Err(DecisionQueryError("invalid decision query".to_owned())),
            }
            .map_err(query_error)?;
            rows.into_iter().map(to_read).collect()
        })
    }
}

const REVIEW_SQL: &str = "SELECT decision_id, title, status, rationale, alternatives, decided_by_entity_kind, decided_by_entity_id, decided_at, review_state, confidence::float8 AS confidence, metadata, created_at, updated_at FROM decisions WHERE review_state = $1 ORDER BY updated_at DESC, decision_id ASC LIMIT $2";
const ENTITY_SQL: &str = "SELECT DISTINCT decision.decision_id, decision.title, decision.status, decision.rationale, decision.alternatives, decision.decided_by_entity_kind, decision.decided_by_entity_id, decision.decided_at, decision.review_state, decision.confidence::float8 AS confidence, decision.metadata, decision.created_at, decision.updated_at FROM decisions decision LEFT JOIN decision_impacted_entities impacted ON impacted.decision_id = decision.decision_id WHERE (decision.decided_by_entity_kind = $1 AND decision.decided_by_entity_id = $2) OR (impacted.entity_kind = $1 AND impacted.entity_id = $2) ORDER BY decision.updated_at DESC, decision.decision_id ASC LIMIT $3";

fn to_read(row: sqlx::postgres::PgRow) -> Result<DecisionRead, DecisionQueryError> {
    Ok(DecisionRead {
        decision_id: row.try_get("decision_id").map_err(query_error)?,
        title: row.try_get("title").map_err(query_error)?,
        status: row.try_get("status").map_err(query_error)?,
        rationale: row.try_get("rationale").map_err(query_error)?,
        alternatives: row
            .try_get::<Value, _>("alternatives")
            .map_err(query_error)?,
        decided_by_entity_kind: row.try_get("decided_by_entity_kind").map_err(query_error)?,
        decided_by_entity_id: row.try_get("decided_by_entity_id").map_err(query_error)?,
        decided_at: row
            .try_get::<Option<DateTime<Utc>>, _>("decided_at")
            .map_err(query_error)?,
        review_state: row.try_get("review_state").map_err(query_error)?,
        confidence: row.try_get("confidence").map_err(query_error)?,
        metadata: row.try_get::<Value, _>("metadata").map_err(query_error)?,
        created_at: row.try_get("created_at").map_err(query_error)?,
        updated_at: row.try_get("updated_at").map_err(query_error)?,
    })
}

fn query_error(error: sqlx::Error) -> DecisionQueryError {
    DecisionQueryError(error.to_string())
}
