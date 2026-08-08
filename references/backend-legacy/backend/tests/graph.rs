use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_hub_backend::domains::graph::core::{
    errors::GraphStoreError,
    ids::{edge_id, evidence_id},
    models::{
        GraphEvidenceSourceKind, GraphReviewState, NewGraphEdge, NewGraphEvidence, NewGraphNode,
        RelationshipType,
    },
    store::GraphStore,
};
use makosh_hub_backend::platform::graph::GraphNodeKind;
use serde_json::{Value, json};
use sqlx::Row;

#[tokio::test]
async fn graph_store_upserts_node_idempotently_against_postgres() {
    let Some(store) = live_graph_store("node idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    let node = NewGraphNode::new(
        GraphNodeKind::Persona,
        format!("person-{suffix}"),
        format!("Alex {suffix}"),
    )
    .properties(json!({"email_address": format!("alex-{suffix}@example.com")}));

    let first = store.upsert_node(&node).await.expect("first node upsert");
    let second = store.upsert_node(&node).await.expect("second node upsert");

    assert_eq!(first.node_id, second.node_id);
    assert_eq!(first.node_kind, GraphNodeKind::Persona);
    assert_eq!(first.stable_key, format!("person-{suffix}"));
}

#[tokio::test]
async fn graph_store_upserts_edge_with_evidence_idempotently_against_postgres() {
    let Some((pool, store)) = live_graph_context("edge idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Persona,
            format!("person-{suffix}"),
            format!("Person {suffix}"),
        ))
        .await
        .expect("person node");
    let email = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("person-{suffix}@example.com"),
            format!("person-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let edge = NewGraphEdge::new(
        person.node_id.clone(),
        email.node_id.clone(),
        RelationshipType::PersonaHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );
    let evidence_source_id = format!("person-{suffix}");
    let first_evidence =
        NewGraphEvidence::new(GraphEvidenceSourceKind::Persona, evidence_source_id.clone())
            .excerpt("initial person evidence")
            .metadata(json!({"projection": "first"}));
    let second_evidence =
        NewGraphEvidence::new(GraphEvidenceSourceKind::Persona, evidence_source_id.clone())
            .excerpt("updated person evidence")
            .metadata(json!({"projection": "second", "source": "duplicate-upsert"}));

    let first = store
        .upsert_edge_with_evidence(&edge, std::slice::from_ref(&first_evidence))
        .await
        .expect("first edge");
    let second = store
        .upsert_edge_with_evidence(&edge, &[second_evidence])
        .await
        .expect("second edge");

    assert_eq!(first.edge_id, second.edge_id);
    assert_eq!(
        first.relationship_type,
        RelationshipType::PersonaHasEmailAddress
    );
    assert_eq!(first.review_state, GraphReviewState::SystemAccepted);

    let evidence_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_evidence WHERE edge_id = $1")
            .bind(&first.edge_id)
            .fetch_one(&pool)
            .await
            .expect("evidence count");
    assert_eq!(evidence_count, 1);

    let evidence_row = sqlx::query(
        r#"
        SELECT excerpt, metadata
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.edge_id)
    .bind(GraphEvidenceSourceKind::Persona.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored evidence row");

    let excerpt: Option<String> = evidence_row.try_get("excerpt").expect("evidence excerpt");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(excerpt.as_deref(), Some("updated person evidence"));
    assert_eq!(
        metadata,
        json!({"projection": "second", "source": "duplicate-upsert"})
    );
}

#[tokio::test]
async fn graph_store_rejects_system_edge_without_evidence_against_postgres() {
    let Some(store) = live_graph_store("evidence requirement").await else {
        return;
    };
    let suffix = unique_suffix();
    let left = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Persona,
            format!("left-{suffix}"),
            "Left",
        ))
        .await
        .expect("left node");
    let right = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("right-{suffix}@example.com"),
            "right@example.com",
        ))
        .await
        .expect("right node");
    let edge = NewGraphEdge::new(
        left.node_id,
        right.node_id,
        RelationshipType::PersonaHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );

    let error = store
        .upsert_edge_with_evidence(&edge, &[])
        .await
        .expect_err("system edge without evidence must fail");

    assert!(matches!(error, GraphStoreError::SystemEdgeRequiresEvidence));
}

#[tokio::test]
async fn graph_store_rejects_suggested_edge_without_evidence_before_database_write() {
    let store = disconnected_graph_store();
    let edge = NewGraphEdge::new(
        "graph:node:v1:persona:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonaHasEmailAddress,
        0.5,
        GraphReviewState::Suggested,
    );

    let error = store
        .upsert_edge_with_evidence(&edge, &[])
        .await
        .expect_err("suggested edge without evidence must fail");

    assert!(matches!(error, GraphStoreError::SystemEdgeRequiresEvidence));
}

#[tokio::test]
async fn graph_store_rejects_invalid_confidence_before_database_write() {
    let store = disconnected_graph_store();
    let edge = NewGraphEdge::new(
        "graph:node:v1:persona:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonaHasEmailAddress,
        1.1,
        GraphReviewState::SystemAccepted,
    );
    let evidence = NewGraphEvidence::new(GraphEvidenceSourceKind::Persona, "person-id");

    let error = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect_err("invalid confidence must fail");

    assert!(matches!(error, GraphStoreError::InvalidConfidence(_)));
}

#[tokio::test]
async fn graph_store_rejects_closed_temporal_edge_before_database_write() {
    let store = disconnected_graph_store();
    let mut edge = NewGraphEdge::new(
        "graph:node:v1:persona:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonaHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );
    edge.valid_to = Some(Utc::now());
    let evidence = NewGraphEvidence::new(GraphEvidenceSourceKind::Persona, "person-id");

    let error = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect_err("closed temporal edge must fail");

    assert!(matches!(error, GraphStoreError::TemporalEdgesUnsupported));
}

#[test]
fn graph_deterministic_ids_distinguish_delimiter_bearing_components() {
    let relationship_type = RelationshipType::PersonaHasEmailAddress;

    assert_ne!(
        edge_id("a:b", relationship_type, "c"),
        edge_id("a", relationship_type, "b:c")
    );
    assert_ne!(
        evidence_id("edge:a:b", GraphEvidenceSourceKind::Persona, "c"),
        evidence_id("edge:a", GraphEvidenceSourceKind::Persona, "b:c")
    );
}

async fn live_graph_store(test_name: &str) -> Option<GraphStore> {
    live_graph_context(test_name)
        .await
        .map(|(_pool, store)| store)
}

async fn live_graph_context(_test_name: &str) -> Option<(sqlx::postgres::PgPool, GraphStore)> {
    let test_context = TestContext::new().await;
    let pool = test_context.pool().clone();
    Box::leak(Box::new(test_context));
    Some((pool.clone(), GraphStore::new(pool)))
}

fn disconnected_graph_store() -> GraphStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    GraphStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
