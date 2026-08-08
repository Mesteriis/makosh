use chrono::{DateTime, Utc};
use makosh_obligations_api::{
    ObligationEvidence, ObligationListFuture, ObligationListQuery, ObligationQueryError,
    ObligationRead, ObligationReadPort, ObligationUpsert, ObligationWriteError,
    ObligationWriteFuture, ObligationWritePort,
};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction;
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct ObligationPostgresReadQuery {
    pool: PgPool,
}
impl ObligationPostgresReadQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_review_state_with_observation(
        &self,
        obligation_id: &str,
        review_state: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ObligationRead, ObligationWriteError> {
        if obligation_id.trim().is_empty() || review_state.trim().is_empty() {
            return Err(ObligationWriteError::InvalidWrite(
                "obligation id and review state are required",
            ));
        }
        let mut tx = self.pool.begin().await.map_err(write_error)?;
        let result = Self::set_review_state_in_transaction(
            &mut tx,
            obligation_id,
            review_state,
            observation_id,
            metadata,
        )
        .await?;
        tx.commit().await.map_err(write_error)?;
        Ok(result)
    }

    pub async fn set_review_state_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        obligation_id: &str,
        review_state: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ObligationRead, ObligationWriteError> {
        let row = sqlx::query("UPDATE obligations SET review_state=$1, updated_at=now() WHERE obligation_id=$2 RETURNING obligation_id,obligated_entity_kind,obligated_entity_id,beneficiary_entity_kind,beneficiary_entity_id,statement,status,review_state,due_at,condition,risk_state,confidence::float8 AS confidence,metadata,created_at,updated_at")
            .bind(review_state).bind(obligation_id).fetch_optional(&mut **transaction).await.map_err(write_error)?
            .ok_or_else(|| ObligationWriteError::Failed("obligation was not found".to_owned()))?;
        let result =
            to_read(row).map_err(|error| ObligationWriteError::Failed(error.to_string()))?;
        materialize_review_transition_link_in_transaction(
            transaction,
            observation_id,
            "obligations",
            "obligation",
            &result.obligation_id,
            "review_state",
            review_state,
            metadata,
        )
        .await
        .map_err(|error| ObligationWriteError::Failed(error.to_string()))?;
        Ok(result)
    }

    pub async fn upsert_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        obligation: &ObligationUpsert,
        evidence: &[ObligationEvidence],
    ) -> Result<ObligationRead, ObligationWriteError> {
        if obligation.obligation_id.trim().is_empty() || obligation.statement.trim().is_empty() {
            return Err(ObligationWriteError::InvalidWrite(
                "obligation id and statement are required",
            ));
        }
        let row = sqlx::query("INSERT INTO obligations (obligation_id,obligated_entity_kind,obligated_entity_id,beneficiary_entity_kind,beneficiary_entity_id,statement,status,review_state,due_at,condition,risk_state,confidence,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,CAST($12 AS NUMERIC(5,4)),$13) ON CONFLICT (obligation_id) DO UPDATE SET status=EXCLUDED.status,review_state=EXCLUDED.review_state,due_at=EXCLUDED.due_at,condition=EXCLUDED.condition,risk_state=EXCLUDED.risk_state,confidence=EXCLUDED.confidence,metadata=EXCLUDED.metadata,updated_at=now() RETURNING obligation_id,obligated_entity_kind,obligated_entity_id,beneficiary_entity_kind,beneficiary_entity_id,statement,status,review_state,due_at,condition,risk_state,confidence::float8 AS confidence,metadata,created_at,updated_at")
            .bind(&obligation.obligation_id).bind(&obligation.obligated_entity_kind).bind(&obligation.obligated_entity_id)
            .bind(&obligation.beneficiary_entity_kind).bind(&obligation.beneficiary_entity_id).bind(&obligation.statement)
            .bind(&obligation.status).bind(&obligation.review_state).bind(obligation.due_at).bind(&obligation.condition)
            .bind(&obligation.risk_state).bind(obligation.confidence).bind(&obligation.metadata)
            .fetch_one(&mut **transaction).await.map_err(write_error)?;
        for item in evidence {
            sqlx::query("INSERT INTO obligation_evidence (evidence_id,obligation_id,source_kind,source_id,observation_id,quote,confidence,metadata) VALUES (md5($1 || ':' || $2 || ':' || $3),$1,$2,$3,$4,$5,CAST($6 AS NUMERIC(5,4)),$7) ON CONFLICT (obligation_id,source_kind,source_id) DO UPDATE SET observation_id=EXCLUDED.observation_id,quote=EXCLUDED.quote,confidence=EXCLUDED.confidence,metadata=EXCLUDED.metadata")
                .bind(&obligation.obligation_id).bind(&item.source_kind).bind(&item.source_id).bind(&item.observation_id)
                .bind(&item.excerpt).bind(item.confidence).bind(&item.metadata).execute(&mut **transaction).await.map_err(write_error)?;
            if let Some(observation_id) = item.observation_id.as_deref() {
                link_domain_entity_in_transaction(
                    transaction,
                    observation_id,
                    "obligations",
                    "obligation",
                    obligation.obligation_id.clone(),
                    Some("support"),
                    Some(item.confidence),
                    Some(item.metadata.clone()),
                )
                .await
                .map_err(|error| ObligationWriteError::Failed(error.to_string()))?;
            }
        }
        to_read(row).map_err(|error| ObligationWriteError::Failed(error.to_string()))
    }
}

impl ObligationReadPort for ObligationPostgresReadQuery {
    fn list<'a>(&'a self, query: ObligationListQuery) -> ObligationListFuture<'a> {
        Box::pin(async move {
            let limit = query.limit.unwrap_or(50);
            if !(1..=100).contains(&limit) {
                return Err(ObligationQueryError(
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
                _ => return Err(ObligationQueryError("invalid obligation query".to_owned())),
            }
            .map_err(query_error)?;
            rows.into_iter().map(to_read).collect()
        })
    }
}

impl ObligationWritePort for ObligationPostgresReadQuery {
    fn upsert<'a>(
        &'a self,
        obligation: &'a ObligationUpsert,
        evidence: &'a [ObligationEvidence],
    ) -> ObligationWriteFuture<'a> {
        Box::pin(async move {
            if obligation.obligation_id.trim().is_empty() || obligation.statement.trim().is_empty()
            {
                return Err(ObligationWriteError::InvalidWrite(
                    "obligation id and statement are required",
                ));
            }
            let mut tx = self.pool.begin().await.map_err(write_error)?;
            let row = sqlx::query("INSERT INTO obligations (obligation_id,obligated_entity_kind,obligated_entity_id,beneficiary_entity_kind,beneficiary_entity_id,statement,status,review_state,due_at,condition,risk_state,confidence,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,CAST($12 AS NUMERIC(5,4)),$13) ON CONFLICT (obligation_id) DO UPDATE SET status=EXCLUDED.status,review_state=EXCLUDED.review_state,due_at=EXCLUDED.due_at,condition=EXCLUDED.condition,risk_state=EXCLUDED.risk_state,confidence=EXCLUDED.confidence,metadata=EXCLUDED.metadata,updated_at=now() RETURNING obligation_id,obligated_entity_kind,obligated_entity_id,beneficiary_entity_kind,beneficiary_entity_id,statement,status,review_state,due_at,condition,risk_state,confidence::float8 AS confidence,metadata,created_at,updated_at")
                .bind(&obligation.obligation_id).bind(&obligation.obligated_entity_kind).bind(&obligation.obligated_entity_id)
                .bind(&obligation.beneficiary_entity_kind).bind(&obligation.beneficiary_entity_id).bind(&obligation.statement)
                .bind(&obligation.status).bind(&obligation.review_state).bind(obligation.due_at).bind(&obligation.condition)
                .bind(&obligation.risk_state).bind(obligation.confidence).bind(&obligation.metadata)
                .fetch_one(&mut *tx).await.map_err(write_error)?;
            for item in evidence {
                sqlx::query("INSERT INTO obligation_evidence (evidence_id,obligation_id,source_kind,source_id,observation_id,quote,confidence,metadata) VALUES (md5($1 || ':' || $2 || ':' || $3),$1,$2,$3,$4,$5,CAST($6 AS NUMERIC(5,4)),$7) ON CONFLICT (obligation_id,source_kind,source_id) DO UPDATE SET observation_id=EXCLUDED.observation_id,quote=EXCLUDED.quote,confidence=EXCLUDED.confidence,metadata=EXCLUDED.metadata")
                    .bind(&obligation.obligation_id).bind(&item.source_kind).bind(&item.source_id).bind(&item.observation_id)
                    .bind(&item.excerpt).bind(item.confidence).bind(&item.metadata).execute(&mut *tx).await.map_err(write_error)?;
            }
            tx.commit().await.map_err(write_error)?;
            to_read(row).map_err(|error| ObligationWriteError::Failed(error.to_string()))
        })
    }
}

fn write_error(error: sqlx::Error) -> ObligationWriteError {
    ObligationWriteError::Failed(error.to_string())
}

const REVIEW_SQL: &str = "SELECT obligation_id, obligated_entity_kind, obligated_entity_id, beneficiary_entity_kind, beneficiary_entity_id, statement, status, review_state, due_at, condition, risk_state, confidence::float8 AS confidence, metadata, created_at, updated_at FROM obligations WHERE review_state = $1 ORDER BY updated_at DESC, obligation_id ASC LIMIT $2";
const ENTITY_SQL: &str = "SELECT obligation_id, obligated_entity_kind, obligated_entity_id, beneficiary_entity_kind, beneficiary_entity_id, statement, status, review_state, due_at, condition, risk_state, confidence::float8 AS confidence, metadata, created_at, updated_at FROM obligations WHERE (obligated_entity_kind = $1 AND obligated_entity_id = $2) OR (beneficiary_entity_kind = $1 AND beneficiary_entity_id = $2) ORDER BY updated_at DESC, obligation_id ASC LIMIT $3";

fn to_read(row: sqlx::postgres::PgRow) -> Result<ObligationRead, ObligationQueryError> {
    Ok(ObligationRead {
        obligation_id: row.try_get("obligation_id").map_err(query_error)?,
        obligated_entity_kind: row.try_get("obligated_entity_kind").map_err(query_error)?,
        obligated_entity_id: row.try_get("obligated_entity_id").map_err(query_error)?,
        beneficiary_entity_kind: row
            .try_get("beneficiary_entity_kind")
            .map_err(query_error)?,
        beneficiary_entity_id: row.try_get("beneficiary_entity_id").map_err(query_error)?,
        statement: row.try_get("statement").map_err(query_error)?,
        status: row.try_get("status").map_err(query_error)?,
        review_state: row.try_get("review_state").map_err(query_error)?,
        due_at: row
            .try_get::<Option<DateTime<Utc>>, _>("due_at")
            .map_err(query_error)?,
        condition: row.try_get("condition").map_err(query_error)?,
        risk_state: row.try_get("risk_state").map_err(query_error)?,
        confidence: row.try_get("confidence").map_err(query_error)?,
        metadata: row.try_get::<Value, _>("metadata").map_err(query_error)?,
        created_at: row.try_get("created_at").map_err(query_error)?,
        updated_at: row.try_get("updated_at").map_err(query_error)?,
    })
}
fn query_error(error: sqlx::Error) -> ObligationQueryError {
    ObligationQueryError(error.to_string())
}
