use chrono::{DateTime, Utc};
use makosh_graph_api::{
    GraphCount, GraphEdgeRead, GraphEvidenceRead, GraphNeighborhoodFuture,
    GraphNeighborhoodQueryPort, GraphNodeListFuture, GraphNodeRead, GraphNodeReadPort,
    GraphNodeSearchPort, GraphQueryError, GraphSummary, GraphSummaryFuture, GraphSummaryQueryPort,
};
use sqlx::{PgPool, Row};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct GraphPostgresSummaryQuery {
    pool: PgPool,
}

impl GraphPostgresSummaryQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl GraphSummaryQueryPort for GraphPostgresSummaryQuery {
    fn summary<'a>(&'a self) -> GraphSummaryFuture<'a> {
        Box::pin(async move {
            let node_counts = counts(&self.pool, "node_kind", "graph_nodes").await?;
            let edge_counts = counts(&self.pool, "relationship_type", "graph_edges").await?;
            let evidence_count =
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_evidence")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|error| makosh_graph_api::GraphQueryError(error.to_string()))?;
            let latest_projection_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
                "SELECT max(updated_at) FROM (SELECT updated_at FROM graph_nodes UNION ALL SELECT updated_at FROM graph_edges) graph_updates",
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|error| makosh_graph_api::GraphQueryError(error.to_string()))?;
            Ok(GraphSummary {
                is_empty: node_counts.iter().map(|item| item.count).sum::<i64>() == 0,
                node_counts,
                edge_counts,
                evidence_count,
                latest_projection_at,
            })
        })
    }
}

impl GraphNodeReadPort for GraphPostgresSummaryQuery {
    fn list_nodes<'a>(&'a self, limit: i64) -> GraphNodeListFuture<'a> {
        Box::pin(async move {
            sqlx::query(
                "SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at FROM graph_nodes ORDER BY node_kind, label LIMIT $1",
            )
            .bind(limit.clamp(1, 50))
            .fetch_all(&self.pool)
            .await
            .map_err(query_error)?
            .into_iter()
            .map(to_node)
            .collect()
        })
    }
}

impl GraphNodeSearchPort for GraphPostgresSummaryQuery {
    fn search_nodes<'a>(&'a self, query: &'a str, limit: i64) -> GraphNodeListFuture<'a> {
        Box::pin(async move {
            let pattern = format!("%{query}%");
            sqlx::query(
                "SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at FROM graph_nodes WHERE label ILIKE $1 OR stable_key ILIKE $1 ORDER BY node_kind, label LIMIT $2",
            )
            .bind(pattern)
            .bind(limit.clamp(1, 50))
            .fetch_all(&self.pool)
            .await
            .map_err(query_error)?
            .into_iter()
            .map(to_node)
            .collect()
        })
    }
}

impl GraphNeighborhoodQueryPort for GraphPostgresSummaryQuery {
    fn neighborhood<'a>(&'a self, node_id: &'a str) -> GraphNeighborhoodFuture<'a> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(query_error)?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
                .execute(&mut *tx)
                .await
                .map_err(query_error)?;
            let Some(row) = sqlx::query(NODE_SQL)
                .bind(node_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(query_error)?
            else {
                tx.commit().await.map_err(query_error)?;
                return Ok(None);
            };
            let selected_node = to_node(row)?;
            let mut edges = sqlx::query(EDGE_SQL)
                .bind(&selected_node.node_id)
                .bind(EDGE_LIMIT + 1)
                .fetch_all(&mut *tx)
                .await
                .map_err(query_error)?
                .into_iter()
                .map(to_edge)
                .collect::<Result<Vec<_>, _>>()?;
            let truncated = edges.len() > EDGE_LIMIT as usize;
            edges.truncate(EDGE_LIMIT as usize);
            let mut node_ids = BTreeSet::new();
            for edge in &edges {
                if edge.source_node_id != selected_node.node_id {
                    node_ids.insert(edge.source_node_id.clone());
                }
                if edge.target_node_id != selected_node.node_id {
                    node_ids.insert(edge.target_node_id.clone());
                }
            }
            let nodes = if node_ids.is_empty() {
                Vec::new()
            } else {
                sqlx::query(NODES_BY_ID_SQL)
                    .bind(node_ids.into_iter().collect::<Vec<_>>())
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(query_error)?
                    .into_iter()
                    .map(to_node)
                    .collect::<Result<Vec<_>, _>>()?
            };
            let edge_ids = edges
                .iter()
                .map(|edge| edge.edge_id.clone())
                .collect::<Vec<_>>();
            let (mut evidence, evidence_truncated) = if edge_ids.is_empty() {
                (Vec::new(), false)
            } else {
                let mut values = sqlx::query(EVIDENCE_SQL)
                    .bind(edge_ids)
                    .bind(EVIDENCE_LIMIT + 1)
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(query_error)?
                    .into_iter()
                    .map(to_evidence)
                    .collect::<Result<Vec<_>, _>>()?;
                let truncated = values.len() > EVIDENCE_LIMIT as usize;
                values.truncate(EVIDENCE_LIMIT as usize);
                (values, truncated)
            };
            let result = makosh_graph_api::GraphNeighborhoodRead {
                selected_node,
                nodes,
                edges,
                evidence: std::mem::take(&mut evidence),
                edge_limit: EDGE_LIMIT,
                truncated,
                evidence_limit: EVIDENCE_LIMIT,
                evidence_truncated,
            };
            tx.commit().await.map_err(query_error)?;
            Ok(Some(result))
        })
    }
}

const EDGE_LIMIT: i64 = 100;
const EVIDENCE_LIMIT: i64 = 100;
const NODE_SQL: &str = "SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at FROM graph_nodes WHERE node_id = $1";
const NODES_BY_ID_SQL: &str = "SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at FROM graph_nodes WHERE node_id = ANY($1) ORDER BY node_kind, label, node_id";
const EDGE_SQL: &str = "SELECT edge_id, source_node_id, target_node_id, relationship_type, confidence::float8 AS confidence, review_state, properties, valid_from, valid_to, created_at, updated_at FROM graph_edges WHERE valid_to IS NULL AND (source_node_id = $1 OR target_node_id = $1) ORDER BY relationship_type, source_node_id, target_node_id LIMIT $2";
const EVIDENCE_SQL: &str = "SELECT edge_id, source_kind, source_id, observation_id, excerpt, metadata FROM graph_evidence WHERE edge_id = ANY($1) ORDER BY edge_id, source_kind, source_id LIMIT $2";

fn to_node(row: sqlx::postgres::PgRow) -> Result<GraphNodeRead, GraphQueryError> {
    Ok(GraphNodeRead {
        node_id: row.try_get("node_id").map_err(query_error)?,
        node_kind: row.try_get("node_kind").map_err(query_error)?,
        stable_key: row.try_get("stable_key").map_err(query_error)?,
        label: row.try_get("label").map_err(query_error)?,
        properties: row.try_get("properties").map_err(query_error)?,
        created_at: row.try_get("created_at").map_err(query_error)?,
        updated_at: row.try_get("updated_at").map_err(query_error)?,
    })
}

fn to_edge(row: sqlx::postgres::PgRow) -> Result<GraphEdgeRead, GraphQueryError> {
    Ok(GraphEdgeRead {
        edge_id: row.try_get("edge_id").map_err(query_error)?,
        source_node_id: row.try_get("source_node_id").map_err(query_error)?,
        target_node_id: row.try_get("target_node_id").map_err(query_error)?,
        relationship_type: row.try_get("relationship_type").map_err(query_error)?,
        confidence: row.try_get("confidence").map_err(query_error)?,
        review_state: row.try_get("review_state").map_err(query_error)?,
        properties: row.try_get("properties").map_err(query_error)?,
        valid_from: row.try_get("valid_from").map_err(query_error)?,
        valid_to: row.try_get("valid_to").map_err(query_error)?,
        created_at: row.try_get("created_at").map_err(query_error)?,
        updated_at: row.try_get("updated_at").map_err(query_error)?,
    })
}

fn to_evidence(row: sqlx::postgres::PgRow) -> Result<GraphEvidenceRead, GraphQueryError> {
    Ok(GraphEvidenceRead {
        edge_id: row.try_get("edge_id").map_err(query_error)?,
        source_kind: row.try_get("source_kind").map_err(query_error)?,
        source_id: row.try_get("source_id").map_err(query_error)?,
        observation_id: row.try_get("observation_id").map_err(query_error)?,
        excerpt: row.try_get("excerpt").map_err(query_error)?,
        metadata: row.try_get("metadata").map_err(query_error)?,
    })
}

fn query_error(error: sqlx::Error) -> GraphQueryError {
    GraphQueryError(error.to_string())
}

async fn counts(
    pool: &PgPool,
    key_column: &str,
    table: &str,
) -> Result<Vec<GraphCount>, makosh_graph_api::GraphQueryError> {
    let sql = format!(
        "SELECT {key_column} AS key, count(*) AS count FROM {table} GROUP BY {key_column} ORDER BY {key_column}"
    );
    sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(|error| makosh_graph_api::GraphQueryError(error.to_string()))?
        .into_iter()
        .map(|row| {
            Ok(GraphCount {
                key: row.try_get("key").map_err(|error: sqlx::Error| {
                    makosh_graph_api::GraphQueryError(error.to_string())
                })?,
                count: row.try_get("count").map_err(|error: sqlx::Error| {
                    makosh_graph_api::GraphQueryError(error.to_string())
                })?,
            })
        })
        .collect()
}
