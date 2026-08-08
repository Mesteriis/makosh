use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_hub_backend::domains::obligations::errors::ObligationStoreError;
use makosh_hub_backend::domains::obligations::models::states::ObligationReviewState;
use makosh_hub_backend::domains::obligations::models::{
    entity_kind::ObligationEntityKind,
    evidence::NewObligationEvidence,
    obligation::NewObligation,
    source_kind::ObligationEvidenceSourceKind,
    states::{ObligationRiskState, ObligationStatus},
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::graph_projection::service::GraphProjectionService;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::test]
async fn obligation_store_upserts_evidence_backed_obligation_without_creating_task_against_postgres()
 {
    let Some((pool, obligation_store)) =
        live_obligation_context("evidence backed obligation upsert").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let obligated_persona_id = format!("person:v1:email:owner-{suffix}@example.com");
    let beneficiary_project_id = format!("project:v1:obligation-{suffix}");
    let evidence_source_id = format!("message:obligation:{suffix}");

    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        obligated_persona_id.clone(),
        "Send the revised project proposal",
        0.88,
        ObligationReviewState::UserConfirmed,
    )
    .beneficiary(
        ObligationEntityKind::Project,
        beneficiary_project_id.clone(),
    )
    .condition("before the stakeholder review")
    .risk_state(ObligationRiskState::Watch)
    .metadata(json!({"channel": "email", "scope": "proposal"}));
    let first_evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .quote("I will send the revised project proposal before the review.")
    .confidence(0.91)
    .metadata(json!({"message_part": "body", "revision": 1}));
    let second_evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .quote("Updated evidence for the revised proposal commitment.")
    .confidence(0.93)
    .metadata(json!({"message_part": "body", "revision": 2}));

    let first = obligation_store
        .upsert_with_evidence(&obligation, std::slice::from_ref(&first_evidence))
        .await
        .expect("first obligation upsert");
    let second = obligation_store
        .upsert_with_evidence(&obligation, &[second_evidence])
        .await
        .expect("idempotent obligation upsert");

    assert_eq!(first.obligation_id, second.obligation_id);
    assert_eq!(first.obligated_entity_kind, ObligationEntityKind::Persona);
    assert_eq!(first.obligated_entity_id, obligated_persona_id);
    assert_eq!(
        first.beneficiary_entity_kind,
        Some(ObligationEntityKind::Project)
    );
    assert_eq!(first.beneficiary_entity_id, Some(beneficiary_project_id));
    assert_eq!(first.statement, "Send the revised project proposal");
    assert_eq!(first.status, ObligationStatus::Open);
    assert_eq!(first.review_state, ObligationReviewState::UserConfirmed);
    assert_eq!(first.risk_state, ObligationRiskState::Watch);
    assert_eq!(
        first.condition.as_deref(),
        Some("before the stakeholder review")
    );
    assert_eq!(first.confidence, 0.88);

    let evidence_row = sqlx::query(
        r#"
        SELECT quote, confidence::float8 AS confidence, metadata
        FROM obligation_evidence
        WHERE obligation_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.obligation_id)
    .bind(ObligationEvidenceSourceKind::Communication.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored obligation evidence");
    let quote: Option<String> = evidence_row.try_get("quote").expect("evidence quote");
    let evidence_confidence: f64 = evidence_row
        .try_get("confidence")
        .expect("evidence confidence");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(
        quote.as_deref(),
        Some("Updated evidence for the revised proposal commitment.")
    );
    assert_eq!(evidence_confidence, 0.93);
    assert_eq!(metadata, json!({"message_part": "body", "revision": 2}));

    let listed = obligation_store
        .list_for_entity(
            ObligationEntityKind::Persona,
            &first.obligated_entity_id,
            10,
        )
        .await
        .expect("obligations for obligated persona");
    assert!(
        listed
            .iter()
            .any(|item| item.obligation_id == first.obligation_id)
    );

    GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await
        .expect("project obligation graph");

    let obligation_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'obligation' AND stable_key = $1",
    )
    .bind(&first.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation graph node");
    let obligated_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'persona' AND stable_key = $1",
    )
    .bind(&first.obligated_entity_id)
    .fetch_one(&pool)
    .await
    .expect("obligated persona graph node");
    let beneficiary_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'project' AND stable_key = $1",
    )
    .bind(first.beneficiary_entity_id.as_deref().expect("beneficiary"))
    .fetch_one(&pool)
    .await
    .expect("beneficiary project graph node");

    let obligated_edge_id: String = sqlx::query_scalar(
        r#"
        SELECT edge_id
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND review_state = 'user_confirmed'
          AND properties @> $3
          AND valid_to IS NULL
        "#,
    )
    .bind(&obligation_node_id)
    .bind(&obligated_node_id)
    .bind(json!({"domain": "obligation", "link_role": "obligated_entity"}))
    .fetch_one(&pool)
    .await
    .expect("obligation to obligated entity graph edge");
    let beneficiary_edge_id: String = sqlx::query_scalar(
        r#"
        SELECT edge_id
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND review_state = 'user_confirmed'
          AND properties @> $3
          AND valid_to IS NULL
        "#,
    )
    .bind(&obligation_node_id)
    .bind(&beneficiary_node_id)
    .bind(json!({"domain": "obligation", "link_role": "beneficiary_entity"}))
    .fetch_one(&pool)
    .await
    .expect("obligation to beneficiary entity graph edge");

    for edge_id in [obligated_edge_id, beneficiary_edge_id] {
        let graph_evidence_row = sqlx::query(
            r#"
            SELECT source_kind, source_id, excerpt, metadata
            FROM graph_evidence
            WHERE edge_id = $1
            "#,
        )
        .bind(edge_id)
        .fetch_one(&pool)
        .await
        .expect("obligation graph evidence");
        let graph_source_kind: String = graph_evidence_row
            .try_get("source_kind")
            .expect("graph evidence source kind");
        let graph_source_id: String = graph_evidence_row
            .try_get("source_id")
            .expect("graph evidence source id");
        let graph_excerpt: Option<String> = graph_evidence_row
            .try_get("excerpt")
            .expect("graph evidence excerpt");
        let graph_evidence_metadata: Value = graph_evidence_row
            .try_get("metadata")
            .expect("graph evidence metadata");

        assert_eq!(graph_source_kind, "obligation");
        assert_eq!(graph_source_id, first.obligation_id);
        assert_eq!(
            graph_excerpt.as_deref(),
            Some("Updated evidence for the revised proposal commitment.")
        );
        assert_eq!(
            graph_evidence_metadata,
            json!({
                "domain": "obligation",
                "source_kind": "communication",
                "source_id": evidence_source_id
            })
        );
    }

    let task_link_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1",
    )
    .bind(&first.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation task link count");
    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&evidence_source_id)
            .fetch_one(&pool)
            .await
            .expect("task count for obligation evidence source");

    assert_eq!(task_link_count, 0);
    assert_eq!(task_count, 0);
}

#[tokio::test]
async fn obligation_store_rejects_missing_evidence_before_database_write() {
    let store = disconnected_obligation_store();
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        "Reply with the signed agreement",
        0.8,
        ObligationReviewState::Suggested,
    );

    let error = store
        .upsert_with_evidence(&obligation, &[])
        .await
        .expect_err("obligation without evidence must fail before database write");

    assert!(matches!(error, ObligationStoreError::MissingEvidence));
}

#[tokio::test]
async fn obligation_store_rejects_invalid_confidence_before_database_write() {
    let store = disconnected_obligation_store();
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        "Reply with the signed agreement",
        1.1,
        ObligationReviewState::Suggested,
    );
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        "message:invalid-obligation-confidence",
    );

    let error = store
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect_err("invalid confidence must fail before database write");

    assert!(matches!(
        error,
        ObligationStoreError::InvalidScore("confidence", _)
    ));
}

#[tokio::test]
async fn obligation_store_rejects_partial_beneficiary_before_database_write() {
    let store = disconnected_obligation_store();
    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        "Reply with the signed agreement",
        0.8,
        ObligationReviewState::Suggested,
    );
    obligation.beneficiary_entity_kind = Some(ObligationEntityKind::Organization);
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        "message:partial-beneficiary",
    );

    let error = store
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect_err("partial beneficiary must fail before database write");

    assert!(matches!(error, ObligationStoreError::PartialBeneficiary));
}

#[tokio::test]
async fn obligation_store_rejects_missing_observation_evidence_against_postgres() {
    let Some((_pool, store)) =
        live_obligation_context("missing obligation observation evidence").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        format!("Observation-backed obligation {suffix}"),
        0.8,
        ObligationReviewState::Suggested,
    );
    let evidence =
        NewObligationEvidence::observation(format!("observation:v1:missing-obligation:{suffix}"));

    let error = store
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect_err("missing observation evidence must fail");

    assert!(matches!(
        error,
        ObligationStoreError::ObservationNotFound(_)
    ));
}

#[tokio::test]
async fn obligation_store_materializes_support_link_for_observation_evidence_against_postgres() {
    let Some((pool, store)) = live_obligation_context("obligation support link").await else {
        return;
    };
    let suffix = unique_suffix();
    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "subject": format!("Obligation support {suffix}"),
                    "body": "I will send the signed agreement tomorrow."
                }),
                format!("manual://obligation-support/{suffix}"),
            )
            .confidence(0.92),
        )
        .await
        .expect("support observation");

    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        format!("person:v1:email:obligation-support-{suffix}@example.com"),
        format!("Send the signed agreement {suffix}"),
        0.86,
        ObligationReviewState::UserConfirmed,
    );
    let stored = store
        .upsert_with_evidence(
            &obligation,
            &[
                NewObligationEvidence::observation(observation.observation_id.clone())
                    .quote("I will send the signed agreement tomorrow.")
                    .confidence(0.91),
            ],
        )
        .await
        .expect("obligation upsert with observation evidence");

    let support_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'obligations'
          AND entity_kind = 'obligation'
          AND entity_id = $2
          AND relationship_kind = 'supports'
        "#,
    )
    .bind(&observation.observation_id)
    .bind(&stored.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation support link count");
    assert_eq!(support_link_count, 1);
}

async fn live_obligation_context(_test_name: &str) -> Option<(PgPool, ObligationStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((pool.clone(), ObligationStore::new(pool)))
}

fn disconnected_obligation_store() -> ObligationStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    ObligationStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
use makosh_hub_backend::domains::obligations::store::ObligationStore;
