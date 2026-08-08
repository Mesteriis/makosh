use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::documents::core::{
    models::NewDocumentImport, store::DocumentImportStore,
};
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::projects::core::{
    ids::project_graph_node_id, store::ProjectStore,
};
use makosh_hub_backend::domains::projects::link_reviews::store::ProjectLinkReviewStore;

use makosh_hub_backend::workflows::graph_projection::service::GraphProjectionService;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

pub(crate) struct LiveProjectionContext {
    pub(crate) pool: PgPool,
    pub(crate) person_store: PersonaProjectionStore,
    pub(crate) communication_store: CommunicationIngestionStore,
    pub(crate) message_store: MessageProjectionStore,
    pub(crate) document_store: DocumentImportStore,
    pub(crate) project_store: ProjectStore,
    pub(crate) graph_projection: GraphProjectionService,
    pub(crate) project_link_review_store: ProjectLinkReviewStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GraphCounts {
    pub(crate) nodes: i64,
    pub(crate) edges: i64,
    pub(crate) evidence: i64,
}

pub(crate) struct ProjectedMessageFixture {
    pub(crate) message_id: String,
    pub(crate) observation_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
}

pub(crate) struct ExpectedProjectEdge<'a> {
    pub(crate) source_node_id: &'a str,
    pub(crate) target_node_id: &'a str,
    pub(crate) relationship_type: &'a str,
    pub(crate) source_kind: &'a str,
    pub(crate) source_id: &'a str,
    pub(crate) observation_id: Option<&'a str>,
    pub(crate) review_state: &'a str,
    pub(crate) confidence: f64,
}

pub(crate) async fn live_projection_context(_test_name: &str) -> Option<LiveProjectionContext> {
    let test_context = TestContext::new().await;
    let pool = test_context.pool().clone();
    Box::leak(Box::new(test_context));

    Some(LiveProjectionContext {
        pool: pool.clone(),
        person_store: PersonaProjectionStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        document_store: DocumentImportStore::new(pool.clone()),
        project_store: ProjectStore::new(pool.clone()),
        graph_projection: GraphProjectionService::new(pool.clone()),
        project_link_review_store: ProjectLinkReviewStore::new(pool),
    })
}

pub(crate) async fn seed_person_message_and_document(
    context: &LiveProjectionContext,
    suffix: u128,
) {
    context
        .person_store
        .upsert_email_persona(&format!(" Known-{suffix}@Example.com "))
        .await
        .expect("upsert known person");

    seed_message(
        context,
        suffix,
        &format!("Unknown-{suffix}@Example.com"),
        &[
            format!("known-{suffix}@example.com"),
            format!("unknown-recipient-{suffix}@example.com"),
        ],
        &format!("provider-graph-projection-{suffix}"),
        &format!("Graph projection subject {suffix}"),
    )
    .await;

    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_graph_projection_{suffix}"),
            format!("graph-projection-{suffix}.md"),
            "# Graph Projection\n\nDocument body.",
        ))
        .await
        .expect("import graph projection document");
}

pub(crate) async fn seed_message(
    context: &LiveProjectionContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
) -> ProjectedMessageFixture {
    let account_id = format!("acct_graph_projection_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Graph Projection Gmail",
            format!("graph-projection-{suffix}@example.com"),
        ))
        .await
        .expect("store graph projection provider account");

    let raw_record_id = format!("raw_graph_projection_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:graph-projection-{suffix}:{provider_record_id}"),
                format!("batch_graph_projection_{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": "Graph projection body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"graph_projection_test"})),
        )
        .await
        .expect("record graph projection raw message");

    let projected = project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project raw graph projection message");

    ProjectedMessageFixture {
        message_id: projected.message_id,
        observation_id: projected.observation_id,
        raw_record_id: projected.raw_record_id,
        provider_record_id: projected.provider_record_id,
        subject: projected.subject,
    }
}

pub(crate) async fn graph_counts_for_suffix(pool: &PgPool, suffix: u128) -> GraphCounts {
    let pattern = format!("%{suffix}%");
    let nodes =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_nodes WHERE stable_key LIKE $1")
            .bind(&pattern)
            .fetch_one(pool)
            .await
            .expect("graph node count for suffix");
    let edges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.stable_key LIKE $1 OR target.stable_key LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .expect("graph edge count for suffix");
    let evidence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE source_id LIKE $1 OR metadata::text LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .expect("graph evidence count for suffix");

    GraphCounts {
        nodes,
        edges,
        evidence,
    }
}

pub(crate) async fn project_graph_counts(pool: &PgPool, project_id: &str) -> GraphCounts {
    let project_node_id = project_graph_node_id(project_id);
    let nodes = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM graph_nodes WHERE node_id = $1 AND node_kind = 'project'",
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph node count");
    let edges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges
        WHERE source_node_id = $1
          AND relationship_type IN (
              'project_has_message',
              'project_has_document',
              'project_involves_persona',
              'project_involves_email_address'
          )
        "#,
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph edge count");
    let evidence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence evidence
        JOIN graph_edges edge ON edge.edge_id = evidence.edge_id
        WHERE edge.source_node_id = $1
        "#,
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph evidence count");

    GraphCounts {
        nodes,
        edges,
        evidence,
    }
}

pub(crate) async fn assert_project_edge_with_evidence(
    pool: &PgPool,
    expected: ExpectedProjectEdge<'_>,
) {
    let row = sqlx::query(
        r#"
        SELECT
            edge.review_state,
            edge.confidence::float8 AS confidence,
            evidence.source_kind,
            evidence.source_id,
            evidence.observation_id
        FROM graph_edges edge
        JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
        WHERE edge.source_node_id = $1
          AND edge.target_node_id = $2
          AND edge.relationship_type = $3
          AND evidence.source_kind = $4
          AND evidence.source_id = $5
        "#,
    )
    .bind(expected.source_node_id)
    .bind(expected.target_node_id)
    .bind(expected.relationship_type)
    .bind(expected.source_kind)
    .bind(expected.source_id)
    .fetch_one(pool)
    .await
    .expect("project edge with evidence");

    let review_state: String = row.try_get("review_state").expect("review state");
    let confidence: f64 = row.try_get("confidence").expect("confidence");
    let stored_source_kind: String = row.try_get("source_kind").expect("source kind");
    let stored_source_id: String = row.try_get("source_id").expect("source id");
    let stored_observation_id: Option<String> =
        row.try_get("observation_id").expect("observation id");
    assert_eq!(review_state, expected.review_state);
    assert!((confidence - expected.confidence).abs() < f64::EPSILON);
    assert_eq!(stored_source_kind, expected.source_kind);
    assert_eq!(stored_source_id, expected.source_id);
    assert_eq!(stored_observation_id.as_deref(), expected.observation_id);
}

pub(crate) async fn cleanup_project_graph_fixture(pool: &PgPool, project_id: &str) {
    sqlx::query("DELETE FROM graph_nodes WHERE node_id = $1")
        .bind(project_graph_node_id(project_id))
        .execute(pool)
        .await
        .expect("cleanup project graph node");
    sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .expect("cleanup graph project");
}

pub(crate) async fn assert_unknown_email_endpoint_projected(
    pool: &PgPool,
    email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
) {
    let message = message_fixture_by_provider_record_id(pool, provider_record_id).await;
    assert_message_edge_with_evidence(
        pool,
        "email_address",
        email_address,
        provider_record_id,
        relationship_type,
        &message,
    )
    .await;

    let person_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_nodes
        WHERE node_kind = 'persona'
          AND (stable_key LIKE $1 OR properties->>'email_address' = $2)
        "#,
    )
    .bind(format!("%{email_address}%"))
    .bind(email_address)
    .fetch_one(pool)
    .await
    .expect("unknown email person node count");
    assert_eq!(person_count, 0);
}

pub(crate) async fn assert_message_edge_with_evidence(
    pool: &PgPool,
    source_node_kind: &str,
    source_email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
    message: &ProjectedMessageFixture,
) {
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
        JOIN communication_messages message ON message.message_id = evidence.source_id
        WHERE source.node_kind = $1
          AND (
              source.stable_key = $2
              OR source.properties->>'email_address' = $2
          )
          AND target.node_kind = 'message'
          AND target.properties->>'provider_record_id' = $3
          AND edge.relationship_type = $4
          AND evidence.source_kind = 'message'
          AND evidence.source_id = $5
          AND evidence.excerpt = $6
          AND evidence.observation_id = $7
          AND evidence.metadata->>'raw_record_id' = $8
          AND evidence.metadata->>'observation_id' = $7
          AND evidence.metadata->>'provider_record_id' = $9
        "#,
    )
    .bind(source_node_kind)
    .bind(source_email_address)
    .bind(provider_record_id)
    .bind(relationship_type)
    .bind(&message.message_id)
    .bind(&message.subject)
    .bind(&message.observation_id)
    .bind(&message.raw_record_id)
    .bind(&message.provider_record_id)
    .fetch_one(pool)
    .await
    .expect("message edge evidence count");
    assert_eq!(evidence_count, 1);
}

pub(crate) async fn assert_message_edge_count(
    pool: &PgPool,
    source_node_kind: &str,
    source_email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
    expected_count: i64,
) {
    let edge_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.node_kind = $1
          AND (
              source.stable_key = $2
              OR source.properties->>'email_address' = $2
          )
          AND target.node_kind = 'message'
          AND target.properties->>'provider_record_id' = $3
          AND edge.relationship_type = $4
        "#,
    )
    .bind(source_node_kind)
    .bind(source_email_address)
    .bind(provider_record_id)
    .bind(relationship_type)
    .fetch_one(pool)
    .await
    .expect("message edge count");
    assert_eq!(edge_count, expected_count);
}

pub(crate) async fn assert_known_person_endpoint_projected(pool: &PgPool, suffix: u128) {
    let provider_record_id = format!("provider-graph-projection-{suffix}");
    let message = message_fixture_by_provider_record_id(pool, &provider_record_id).await;
    assert_message_edge_with_evidence(
        pool,
        "persona",
        &format!("known-{suffix}@example.com"),
        &provider_record_id,
        "persona_received_message",
        &message,
    )
    .await;
}

async fn message_fixture_by_provider_record_id(
    pool: &PgPool,
    provider_record_id: &str,
) -> ProjectedMessageFixture {
    let row = sqlx::query_as::<_, (String, String, String, String, String)>(
        r#"
        SELECT message_id, observation_id, raw_record_id, provider_record_id, subject
        FROM communication_messages
        WHERE provider_record_id = $1
        "#,
    )
    .bind(provider_record_id)
    .fetch_one(pool)
    .await
    .expect("communication message fixture");

    ProjectedMessageFixture {
        message_id: row.0,
        observation_id: row.1,
        raw_record_id: row.2,
        provider_record_id: row.3,
        subject: row.4,
    }
}

pub(crate) async fn assert_document_projected(pool: &PgPool, suffix: u128) {
    let document_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_nodes
        WHERE node_kind = 'document'
          AND stable_key = $1
          AND label = $2
          AND properties->>'document_kind' = 'markdown'
          AND properties->>'source_fingerprint' LIKE 'local-v1:markdown:%'
          AND properties ? 'imported_at'
        "#,
    )
    .bind(format!("doc_graph_projection_{suffix}"))
    .bind(format!("graph-projection-{suffix}.md"))
    .fetch_one(pool)
    .await
    .expect("projected document node count");
    assert_eq!(document_count, 1);
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
