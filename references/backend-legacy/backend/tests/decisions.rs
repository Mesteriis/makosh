use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::decisions::errors::DecisionStoreError;
use makosh_hub_backend::domains::decisions::models::decision::NewDecision;
use makosh_hub_backend::domains::decisions::models::states::DecisionStatus;
use makosh_hub_backend::domains::documents::core::models::NewDocumentImport;
use makosh_hub_backend::domains::documents::core::store::DocumentImportStore;

use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::graph_projection::service::GraphProjectionService;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::test]
async fn decision_store_upserts_evidence_backed_decision_without_creating_work_against_postgres() {
    let Some((pool, decision_store)) = live_decision_context("decision upsert").await else {
        return;
    };
    let suffix = unique_suffix();
    let decided_by_persona_id = format!("person:v1:email:owner-{suffix}@example.com");
    let project_id = format!("project:v1:decision-{suffix}");
    let evidence_source_id = format!("meeting:decision:{suffix}");

    let decision = NewDecision::new(
        "Use local-first storage for client dossier",
        "Private communication context must remain available offline and under owner control.",
        0.9,
        DecisionReviewState::UserConfirmed,
    )
    .decided_by(DecisionEntityKind::Persona, decided_by_persona_id.clone())
    .decided_at(Utc.with_ymd_and_hms(2026, 6, 12, 10, 30, 0).unwrap())
    .alternatives(json!([
        "remote CRM-backed dossier",
        "provider-only search without local memory"
    ]))
    .metadata(json!({"source": "architecture-review", "scope": "dossier"}));
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, project_id.clone())
        .impact_type("architecture_direction")
        .metadata(json!({"component": "dossier"}));
    let first_evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Event,
        evidence_source_id.clone(),
    )
    .quote("We will use local-first storage for the client dossier.")
    .confidence(0.92)
    .metadata(json!({"meeting_section": "architecture", "revision": 1}));
    let second_evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Event,
        evidence_source_id.clone(),
    )
    .quote("Updated decision evidence for local-first dossier storage.")
    .confidence(0.94)
    .metadata(json!({"meeting_section": "architecture", "revision": 2}));

    let first = decision_store
        .upsert_with_evidence(
            &decision,
            std::slice::from_ref(&first_evidence),
            std::slice::from_ref(&impact),
        )
        .await
        .expect("first decision upsert");
    let second = decision_store
        .upsert_with_evidence(&decision, &[second_evidence], &[impact])
        .await
        .expect("idempotent decision upsert");

    assert_eq!(first.decision_id, second.decision_id);
    assert_eq!(first.title, "Use local-first storage for client dossier");
    assert_eq!(
        first.rationale,
        "Private communication context must remain available offline and under owner control."
    );
    assert_eq!(first.status, DecisionStatus::Active);
    assert_eq!(first.review_state, DecisionReviewState::UserConfirmed);
    assert_eq!(first.confidence, 0.9);
    assert_eq!(
        first.decided_by_entity_kind,
        Some(DecisionEntityKind::Persona)
    );
    assert_eq!(first.decided_by_entity_id, Some(decided_by_persona_id));
    assert_eq!(
        first.alternatives,
        json!([
            "remote CRM-backed dossier",
            "provider-only search without local memory"
        ])
    );

    let evidence_row = sqlx::query(
        r#"
        SELECT quote, confidence::float8 AS confidence, metadata
        FROM decision_evidence
        WHERE decision_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.decision_id)
    .bind(DecisionEvidenceSourceKind::Event.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored decision evidence");
    let quote: Option<String> = evidence_row.try_get("quote").expect("evidence quote");
    let confidence: f64 = evidence_row
        .try_get("confidence")
        .expect("evidence confidence");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(
        quote.as_deref(),
        Some("Updated decision evidence for local-first dossier storage.")
    );
    assert_eq!(confidence, 0.94);
    assert_eq!(
        metadata,
        json!({"meeting_section": "architecture", "revision": 2})
    );

    let impact_row = sqlx::query(
        r#"
        SELECT impact_type, metadata
        FROM decision_impacted_entities
        WHERE decision_id = $1
          AND entity_kind = $2
          AND entity_id = $3
        "#,
    )
    .bind(&first.decision_id)
    .bind(DecisionEntityKind::Project.as_str())
    .bind(&project_id)
    .fetch_one(&pool)
    .await
    .expect("stored impacted entity");
    let impact_type: String = impact_row.try_get("impact_type").expect("impact type");
    let impact_metadata: Value = impact_row.try_get("metadata").expect("impact metadata");
    assert_eq!(impact_type, "architecture_direction");
    assert_eq!(impact_metadata, json!({"component": "dossier"}));

    let project_decisions = decision_store
        .list_for_entity(DecisionEntityKind::Project, &project_id, 10)
        .await
        .expect("project decisions");
    assert!(
        project_decisions
            .iter()
            .any(|item| item.decision_id == first.decision_id)
    );

    GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await
        .expect("project decision graph");

    let decision_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'decision' AND stable_key = $1",
    )
    .bind(&first.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision graph node");
    let project_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'project' AND stable_key = $1",
    )
    .bind(&project_id)
    .fetch_one(&pool)
    .await
    .expect("project graph node");
    let graph_edge_row = sqlx::query(
        r#"
        SELECT edge_id, relationship_type, confidence::float8 AS confidence, review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(&decision_node_id)
    .bind(&project_node_id)
    .fetch_one(&pool)
    .await
    .expect("decision graph edge");
    let graph_edge_id: String = graph_edge_row.try_get("edge_id").expect("edge id");
    let graph_relationship_type: String = graph_edge_row
        .try_get("relationship_type")
        .expect("relationship type");
    let graph_confidence: f64 = graph_edge_row
        .try_get("confidence")
        .expect("graph confidence");
    let graph_review_state: String = graph_edge_row
        .try_get("review_state")
        .expect("graph review state");
    let graph_properties: Value = graph_edge_row
        .try_get("properties")
        .expect("graph properties");

    assert_eq!(graph_relationship_type, "entity_relationship");
    assert_eq!(graph_confidence, 0.9);
    assert_eq!(graph_review_state, "user_confirmed");
    assert_eq!(
        graph_properties,
        json!({
            "domain": "decision",
            "decision_id": first.decision_id,
            "impact_type": "architecture_direction"
        })
    );

    let graph_evidence_row = sqlx::query(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM graph_evidence
        WHERE edge_id = $1
        "#,
    )
    .bind(&graph_edge_id)
    .fetch_one(&pool)
    .await
    .expect("decision graph evidence");
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

    assert_eq!(graph_source_kind, "decision");
    assert_eq!(graph_source_id, first.decision_id);
    assert_eq!(
        graph_excerpt.as_deref(),
        Some("Updated decision evidence for local-first dossier storage.")
    );
    assert_eq!(
        graph_evidence_metadata,
        json!({
            "domain": "decision",
            "source_kind": "event",
            "source_id": evidence_source_id
        })
    );

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&evidence_source_id)
            .fetch_one(&pool)
            .await
            .expect("task count for decision source");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE metadata @> $1")
            .bind(json!({"decision_source_id": evidence_source_id}))
            .fetch_one(&pool)
            .await
            .expect("obligation count for decision source");

    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn decision_store_refresh_persists_explicit_message_decision_candidate_against_postgres() {
    let Some((pool, decision_store)) = live_decision_context("decision candidate refresh").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let title = format!("Use persona dossiers {suffix}");
    let rationale = "relationship context must survive channel changes";
    let quote = format!("Decision: {title} because {rationale}.");
    let message_id = seed_decision_message(
        &pool,
        suffix,
        &format!("decision-candidate-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-decision-candidate-{suffix}"),
        &format!("Decision candidate {suffix}"),
        &quote,
    )
    .await;

    let refreshed = decision_store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh decision candidates");
    assert!(refreshed >= 1);

    let decisions = decision_store
        .list_for_entity(DecisionEntityKind::Communication, &message_id, 10)
        .await
        .expect("communication decisions");
    let decision = decisions
        .iter()
        .find(|item| item.title == title)
        .expect("refreshed decision candidate");

    assert_eq!(decision.rationale, rationale);
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(decision.confidence, 0.83);

    let evidence_row: (String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, quote
        FROM decision_evidence
        WHERE decision_id = $1
        "#,
    )
    .bind(&decision.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence_row.0, "communication");
    assert_eq!(evidence_row.1, message_id);
    assert_eq!(evidence_row.2.as_deref(), Some(quote.as_str()));
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let stored_observation_id: Option<String> =
        sqlx::query_scalar("SELECT observation_id FROM decision_evidence WHERE decision_id = $1")
            .bind(&decision.decision_id)
            .fetch_one(&pool)
            .await
            .expect("stored observation id");
    assert_eq!(
        stored_observation_id.as_deref(),
        Some(message_observation_id.as_str())
    );

    let support_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'decisions'
          AND entity_kind = 'decision'
          AND entity_id = $2
          AND relationship_kind = 'supports'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&decision.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision support link count");
    assert_eq!(support_link_count, 1);
}

#[tokio::test]
async fn decision_store_refresh_persists_explicit_document_decision_candidate_against_postgres() {
    let Some((pool, decision_store)) = live_decision_context("document decision refresh").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let title = format!("Adopt source-backed dossier notes {suffix}");
    let rationale = "documents must cite the original evidence";
    let quote = format!("Decision: {title} because {rationale}.");
    let document_id = seed_decision_document(
        &pool,
        &format!("document_decision_candidate_{suffix}"),
        &format!("Decision document {suffix}"),
        &quote,
    )
    .await;

    let refreshed = decision_store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh decision candidates");
    assert!(refreshed >= 1);

    let decisions = decision_store
        .list_for_entity(DecisionEntityKind::Document, &document_id, 10)
        .await
        .expect("document decisions");
    let decision = decisions
        .iter()
        .find(|item| item.title == title)
        .expect("refreshed document decision candidate");

    assert_eq!(decision.rationale, rationale);
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(decision.confidence, 0.83);

    let evidence_row: (String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, quote
        FROM decision_evidence
        WHERE decision_id = $1
        "#,
    )
    .bind(&decision.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence_row.0, "document");
    assert_eq!(evidence_row.1, document_id);
    assert_eq!(evidence_row.2.as_deref(), Some(quote.as_str()));
    let document_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM documents WHERE document_id = $1")
            .bind(&document_id)
            .fetch_one(&pool)
            .await
            .expect("document observation id");
    let stored_observation_id: Option<String> =
        sqlx::query_scalar("SELECT observation_id FROM decision_evidence WHERE decision_id = $1")
            .bind(&decision.decision_id)
            .fetch_one(&pool)
            .await
            .expect("stored observation id");
    assert_eq!(
        stored_observation_id.as_deref(),
        Some(document_observation_id.as_str())
    );

    let support_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'decisions'
          AND entity_kind = 'decision'
          AND entity_id = $2
          AND relationship_kind = 'supports'
        "#,
    )
    .bind(&document_observation_id)
    .bind(&decision.decision_id)
    .fetch_one(&pool)
    .await
    .expect("document decision support link count");
    assert_eq!(support_link_count, 1);
}

#[tokio::test]
async fn decision_store_refresh_preserves_reviewed_decision_candidate_against_postgres() {
    let Some((pool, decision_store)) = live_decision_context("decision candidate review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let title = format!("Keep review decisions stable {suffix}");
    let rationale = "reviewed memory must not be downgraded by extraction refresh";
    let message_id = seed_decision_message(
        &pool,
        suffix,
        &format!("decision-reviewed-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-decision-reviewed-{suffix}"),
        &format!("Reviewed decision {suffix}"),
        &format!("Decision: {title} because {rationale}."),
    )
    .await;

    let _ = decision_store
        .refresh_deterministic_candidates(100)
        .await
        .expect("initial refresh");
    let decision = decision_store
        .list_for_entity(DecisionEntityKind::Communication, &message_id, 10)
        .await
        .expect("communication decisions")
        .into_iter()
        .find(|item| item.title == title)
        .expect("refreshed decision candidate");

    let confirmed = decision_store
        .set_review_state(&decision.decision_id, DecisionReviewState::UserConfirmed)
        .await
        .expect("confirm decision");
    assert_eq!(confirmed.review_state, DecisionReviewState::UserConfirmed);

    let _ = decision_store
        .refresh_deterministic_candidates(100)
        .await
        .expect("repeat refresh");
    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(&decision.decision_id)
            .fetch_one(&pool)
            .await
            .expect("stored review state");

    assert_eq!(review_state, "user_confirmed");
}

#[tokio::test]
async fn decision_store_rejects_missing_evidence_before_database_write() {
    let store = disconnected_decision_store();
    let decision = NewDecision::new(
        "Keep dossiers source-backed",
        "Generated summaries must cite source records.",
        0.8,
        DecisionReviewState::Suggested,
    );
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, "project:v1:makosh");

    let error = store
        .upsert_with_evidence(&decision, &[], &[impact])
        .await
        .expect_err("decision without evidence must fail before database write");

    assert!(matches!(error, DecisionStoreError::MissingEvidence));
}

#[tokio::test]
async fn decision_store_rejects_invalid_confidence_before_database_write() {
    let store = disconnected_decision_store();
    let decision = NewDecision::new(
        "Keep dossiers source-backed",
        "Generated summaries must cite source records.",
        1.2,
        DecisionReviewState::Suggested,
    );
    let evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Document,
        "document:invalid-confidence",
    );
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, "project:v1:makosh");

    let error = store
        .upsert_with_evidence(&decision, &[evidence], &[impact])
        .await
        .expect_err("invalid confidence must fail before database write");

    assert!(matches!(
        error,
        DecisionStoreError::InvalidScore("confidence", _)
    ));
}

#[tokio::test]
async fn decision_store_rejects_partial_decider_before_database_write() {
    let store = disconnected_decision_store();
    let mut decision = NewDecision::new(
        "Keep dossiers source-backed",
        "Generated summaries must cite source records.",
        0.8,
        DecisionReviewState::Suggested,
    );
    decision.decided_by_entity_kind = Some(DecisionEntityKind::Persona);
    let evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Document,
        "document:partial-decider",
    );
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, "project:v1:makosh");

    let error = store
        .upsert_with_evidence(&decision, &[evidence], &[impact])
        .await
        .expect_err("partial decider must fail before database write");

    assert!(matches!(error, DecisionStoreError::PartialDecider));
}

#[tokio::test]
async fn decision_store_rejects_missing_observation_evidence_against_postgres() {
    let Some((_pool, store)) = live_decision_context("missing decision observation evidence").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let decision = NewDecision::new(
        format!("Observation-backed decision {suffix}"),
        "Decision evidence must point to an existing observation.",
        0.8,
        DecisionReviewState::Suggested,
    );
    let evidence =
        NewDecisionEvidence::observation(format!("observation:v1:missing-decision:{suffix}"));
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, "project:v1:makosh");

    let error = store
        .upsert_with_evidence(&decision, &[evidence], &[impact])
        .await
        .expect_err("missing observation evidence must fail");

    assert!(matches!(error, DecisionStoreError::ObservationNotFound(_)));
}

async fn live_decision_context(_test_name: &str) -> Option<(PgPool, DecisionStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((pool.clone(), DecisionStore::new(pool)))
}

fn disconnected_decision_store() -> DecisionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    DecisionStore::new(pool)
}

async fn seed_decision_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_decision_candidate_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Decision Candidate Gmail",
            format!("decision-candidate-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_decision_candidate_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:decision-candidate:{suffix}:{provider_record_id}"),
                format!("batch-decision-candidate-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"decision_candidate_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

async fn seed_decision_document(
    pool: &PgPool,
    document_id: &str,
    title: &str,
    body: &str,
) -> String {
    let import = NewDocumentImport::markdown(document_id, title, body);
    DocumentImportStore::new(pool.clone())
        .import_document(&import)
        .await
        .expect("document import");
    document_id.to_owned()
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
use makosh_hub_backend::domains::decisions::models::entity_kind::DecisionEntityKind;
use makosh_hub_backend::domains::decisions::models::evidence::NewDecisionEvidence;
use makosh_hub_backend::domains::decisions::models::impacted_entity::NewDecisionImpactedEntity;
use makosh_hub_backend::domains::decisions::models::source_kind::DecisionEvidenceSourceKind;
use makosh_hub_backend::domains::decisions::models::states::DecisionReviewState;
use makosh_hub_backend::domains::decisions::store::DecisionStore;
