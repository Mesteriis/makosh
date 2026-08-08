use sqlx::{Postgres, Transaction};

use super::core::errors::GraphStoreError;
use super::core::models::{GraphEdge, GraphNode, NewGraphEdge, NewGraphEvidence, NewGraphNode};
use super::core::store::GraphStore;

/// Workflow-owned graph persistence port. The wrapper keeps transaction-scoped
/// graph mutations behind a semantic boundary without changing atomicity.
#[derive(Clone)]
pub struct GraphProjectionPort(GraphStore);

impl GraphProjectionPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(GraphStore::new(pool))
    }

    pub async fn upsert_node(
        &self,
        node: &NewGraphNode,
    ) -> Result<GraphNode, GraphProjectionPortError> {
        self.0
            .upsert_node(node)
            .await
            .map_err(GraphProjectionPortError)
    }

    pub async fn summary(
        &self,
    ) -> Result<makosh_graph_api::GraphSummary, GraphProjectionPortError> {
        self.0.summary().await.map_err(GraphProjectionPortError)
    }

    pub async fn upsert_node_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        node: &NewGraphNode,
    ) -> Result<GraphNode, GraphProjectionPortError> {
        GraphStore::upsert_node_in_transaction(transaction, node)
            .await
            .map_err(GraphProjectionPortError)
    }

    pub async fn upsert_edge_with_evidence(
        &self,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphProjectionPortError> {
        self.0
            .upsert_edge_with_evidence(edge, evidence)
            .await
            .map_err(GraphProjectionPortError)
    }

    pub async fn upsert_edge_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphProjectionPortError> {
        GraphStore::upsert_edge_with_evidence_in_transaction(transaction, edge, evidence)
            .await
            .map_err(GraphProjectionPortError)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("graph projection persistence failed: {0}")]
pub struct GraphProjectionPortError(#[from] GraphStoreError);

impl GraphProjectionPortError {
    pub fn into_inner(self) -> GraphStoreError {
        self.0
    }
}
