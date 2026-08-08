use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use super::constants::{GRAPH_NEIGHBORHOOD_EDGE_LIMIT, GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT};
use super::errors::GraphStoreError;
use super::models::{GraphNeighborhood, GraphNode};
use super::row_mapping::{row_to_count, row_to_edge, row_to_evidence_summary, row_to_node};
use super::store::GraphStore;
use makosh_graph_api::GraphSummary;

impl GraphStore {
    pub async fn summary(&self) -> Result<GraphSummary, GraphStoreError> {
        let node_count_rows = sqlx::query(
            r#"
            SELECT node_kind AS key, count(*) AS count
            FROM graph_nodes
            GROUP BY node_kind
            ORDER BY node_kind
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let node_counts = node_count_rows
            .into_iter()
            .map(row_to_count)
            .collect::<Result<Vec<_>, _>>()?;

        let edge_count_rows = sqlx::query(
            r#"
            SELECT relationship_type AS key, count(*) AS count
            FROM graph_edges
            GROUP BY relationship_type
            ORDER BY relationship_type
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let edge_counts = edge_count_rows
            .into_iter()
            .map(row_to_count)
            .collect::<Result<Vec<_>, _>>()?;

        let evidence_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_evidence")
            .fetch_one(&self.pool)
            .await?;
        let latest_projection_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT max(updated_at)
            FROM (
                SELECT updated_at FROM graph_nodes
                UNION ALL
                SELECT updated_at FROM graph_edges
            ) graph_updates
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        let total_nodes = node_counts.iter().map(|count| count.count).sum::<i64>();

        Ok(GraphSummary {
            node_counts,
            edge_counts,
            evidence_count,
            latest_projection_at,
            is_empty: total_nodes == 0,
        })
    }

    pub async fn search_nodes(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<GraphNode>, GraphStoreError> {
        let search_pattern = format!("%{query}%");
        let rows = sqlx::query(
            r#"
            SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
            FROM graph_nodes
            WHERE label ILIKE $1 OR stable_key ILIKE $1
            ORDER BY node_kind, label
            LIMIT $2
            "#,
        )
        .bind(search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_node).collect()
    }

    pub async fn list_nodes_for_picker(
        &self,
        limit: i64,
    ) -> Result<Vec<GraphNode>, GraphStoreError> {
        let rows = sqlx::query(
            r#"
            WITH node_degree AS (
                SELECT node_id, count(*) AS edge_count
                FROM (
                    SELECT source_node_id AS node_id
                    FROM graph_edges
                    WHERE valid_to IS NULL
                    UNION ALL
                    SELECT target_node_id AS node_id
                    FROM graph_edges
                    WHERE valid_to IS NULL
                ) edge_endpoints
                GROUP BY node_id
            )
            SELECT
                graph_nodes.node_id,
                graph_nodes.node_kind,
                graph_nodes.stable_key,
                graph_nodes.label,
                graph_nodes.properties,
                graph_nodes.created_at,
                graph_nodes.updated_at
            FROM graph_nodes
            LEFT JOIN node_degree ON node_degree.node_id = graph_nodes.node_id
            ORDER BY
                coalesce(node_degree.edge_count, 0) DESC,
                graph_nodes.updated_at DESC,
                graph_nodes.label,
                graph_nodes.node_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_node).collect()
    }

    pub async fn neighborhood(
        &self,
        node_id: &str,
    ) -> Result<Option<GraphNeighborhood>, GraphStoreError> {
        let mut transaction = self.pool.begin().await?;
        // Neighborhood assembles several query results; one read-only snapshot keeps edges,
        // neighbor nodes, and evidence mutually consistent while projections commit.
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *transaction)
            .await?;

        let Some(selected_row) = sqlx::query(
            r#"
            SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
            FROM graph_nodes
            WHERE node_id = $1
            "#,
        )
        .bind(node_id)
        .fetch_optional(&mut *transaction)
        .await?
        else {
            transaction.commit().await?;
            return Ok(None);
        };
        let selected_node = row_to_node(selected_row)?;

        let edge_rows = sqlx::query(
            r#"
            SELECT
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM graph_edges
            WHERE valid_to IS NULL
              AND (source_node_id = $1 OR target_node_id = $1)
            ORDER BY relationship_type, source_node_id, target_node_id
            LIMIT $2
            "#,
        )
        .bind(&selected_node.node_id)
        .bind(GRAPH_NEIGHBORHOOD_EDGE_LIMIT + 1)
        .fetch_all(&mut *transaction)
        .await?;
        let mut edges = edge_rows
            .into_iter()
            .map(row_to_edge)
            .collect::<Result<Vec<_>, _>>()?;
        let truncated = edges.len() > GRAPH_NEIGHBORHOOD_EDGE_LIMIT as usize;
        edges.truncate(GRAPH_NEIGHBORHOOD_EDGE_LIMIT as usize);

        let mut node_ids = BTreeSet::new();
        for edge in &edges {
            if edge.source_node_id != selected_node.node_id {
                node_ids.insert(edge.source_node_id.clone());
            }
            if edge.target_node_id != selected_node.node_id {
                node_ids.insert(edge.target_node_id.clone());
            }
        }
        let node_ids = node_ids.into_iter().collect::<Vec<_>>();
        let nodes = if node_ids.is_empty() {
            Vec::new()
        } else {
            let node_rows = sqlx::query(
                r#"
                SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
                FROM graph_nodes
                WHERE node_id = ANY($1)
                ORDER BY node_kind, label, node_id
                "#,
            )
            .bind(&node_ids)
            .fetch_all(&mut *transaction)
            .await?;

            node_rows
                .into_iter()
                .map(row_to_node)
                .collect::<Result<Vec<_>, _>>()?
        };

        let edge_ids = edges
            .iter()
            .map(|edge| edge.edge_id.clone())
            .collect::<Vec<_>>();
        let (evidence, evidence_truncated) = if edge_ids.is_empty() {
            (Vec::new(), false)
        } else {
            let evidence_rows = sqlx::query(
                r#"
                SELECT edge_id, source_kind, source_id, observation_id, excerpt, metadata
                FROM graph_evidence
                WHERE edge_id = ANY($1)
                ORDER BY edge_id, source_kind, source_id
                LIMIT $2
                "#,
            )
            .bind(&edge_ids)
            .bind(GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT + 1)
            .fetch_all(&mut *transaction)
            .await?;

            let mut evidence = evidence_rows
                .into_iter()
                .map(row_to_evidence_summary)
                .collect::<Result<Vec<_>, _>>()?;
            let evidence_truncated = evidence.len() > GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT as usize;
            evidence.truncate(GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT as usize);
            (evidence, evidence_truncated)
        };

        transaction.commit().await?;

        Ok(Some(GraphNeighborhood {
            selected_node,
            nodes,
            edges,
            evidence,
            edge_limit: GRAPH_NEIGHBORHOOD_EDGE_LIMIT,
            truncated,
            evidence_limit: GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT,
            evidence_truncated,
        }))
    }
}
