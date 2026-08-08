use crate::support::*;
use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use sqlx::Row;

#[tokio::test]
async fn pgvector_semantic_store_indexes_and_searches_sources_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let store = SemanticEmbeddingStore::new(pool.clone());
    let observation_store = ObservationStore::new(pool.clone());
    let embedding_model = format!("qwen3-embedding:4b-semantic-{suffix}");
    let message_observation = observation_store
        .capture(&NewObservation::new(
            "COMMUNICATION_MESSAGE",
            ObservationOriginKind::TestFixture,
            Utc::now(),
            json!({"source": "semantic_store_test", "suffix": suffix.to_string()}),
            format!("semantic-store-test://message/{suffix}"),
        ))
        .await
        .expect("message observation");

    let extension_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(&pool)
            .await
            .expect("vector extension");
    assert!(extension_exists);

    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            observation_id: Some(message_observation.observation_id.as_str()),
            title: "Roadmap planning",
            source_text: "Discussed Макошь AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("upsert first embedding");
    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Document,
            source_id: &format!("document-semantic-{suffix}"),
            observation_id: None,
            title: "Garden notes",
            source_text: "Tomatoes need watering this weekend.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(8),
            graph_node_id: None,
        })
        .await
        .expect("upsert second embedding");

    let indexed = store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            observation_id: Some(message_observation.observation_id.as_str()),
            title: "Roadmap planning",
            source_text: "Discussed Макошь AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("idempotent upsert");
    assert_eq!(indexed.source_kind, "message");
    assert_eq!(
        indexed.observation_id.as_deref(),
        Some(message_observation.observation_id.as_str())
    );
    assert_eq!(indexed.embedding_dimension, 2560);
    let semantic_observations = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'semantic_embedding'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&indexed.semantic_embedding_id)
    .fetch_all(&pool)
    .await
    .expect("semantic embedding observations");
    assert!(
        semantic_observations.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_SEMANTIC_EMBEDDING"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<serde_json::Value, _>("payload")["observation_id"]
                    == serde_json::Value::String(message_observation.observation_id.clone())
        }),
        "semantic embedding upsert observation must exist"
    );

    let results = store
        .search(&embedding_model, &unit_embedding(0), 5)
        .await
        .expect("search");
    assert_eq!(results[0].source_kind, "message");
    assert_eq!(results[0].source_id, format!("message-semantic-{suffix}"));
    assert_eq!(
        results[0].observation_id.as_deref(),
        Some(message_observation.observation_id.as_str())
    );
    assert!(results[0].score > results[1].score);
    assert_eq!(
        results[0].graph_node_id,
        Some(format!("graph:message:{suffix}"))
    );
}
