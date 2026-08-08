use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_hub_backend::application::organization_persona_links::OrganizationPersonaLinkApplicationService;
use makosh_hub_backend::application::relationship_graph::RelationshipGraphCoordinator;
use makosh_hub_backend::domains::organizations::api::OrganizationStore;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::relationships::{
    errors::RelationshipStoreError,
    models::{
        NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
        RelationshipEvidenceSourceKind, RelationshipReviewState,
    },
    store::RelationshipStore,
};
use makosh_hub_backend::platform::graph::{GraphNodeKind, node_id};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

#[tokio::test]
async fn relationship_store_upserts_persona_relationship_with_evidence_against_postgres() {
    let Some((_test_context, pool, person_store, relationship_store)) =
        live_relationship_context("persona relationship upsert").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source = person_store
        .upsert_email_persona(&format!("relationship-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_persona(&format!("relationship-target-{suffix}@example.com"))
        .await
        .expect("target persona");

    let relationship = NewRelationship::between_personas(
        &source.persona_id,
        &target.persona_id,
        "collaborates_with",
        0.82,
        0.64,
        0.91,
        RelationshipReviewState::UserConfirmed,
    )
    .metadata(json!({"project": "relationship-store-test"}));
    let evidence_source_id = format!("message:{suffix}");
    let first_evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .excerpt("We agreed to collaborate on the Макошь relationship model.")
    .metadata(json!({"channel": "email", "revision": 1}));
    let second_evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .excerpt("Updated relationship evidence.")
    .metadata(json!({"channel": "email", "revision": 2}));

    let first = relationship_store
        .upsert_with_evidence(&relationship, std::slice::from_ref(&first_evidence))
        .await
        .expect("first relationship upsert");
    let second = relationship_store
        .upsert_with_evidence(&relationship, &[second_evidence])
        .await
        .expect("second relationship upsert");

    assert_eq!(first.relationship_id, second.relationship_id);
    assert_eq!(first.source_entity_kind, RelationshipEntityKind::Persona);
    assert_eq!(first.source_entity_id, source.persona_id);
    assert_eq!(first.target_entity_kind, RelationshipEntityKind::Persona);
    assert_eq!(first.target_entity_id, target.persona_id);
    assert_eq!(first.relationship_type, "collaborates_with");
    assert_eq!(first.trust_score, 0.82);
    assert_eq!(first.strength_score, 0.64);
    assert_eq!(first.confidence, 0.91);
    assert_eq!(first.review_state, RelationshipReviewState::UserConfirmed);

    let evidence_row = sqlx::query(
        r#"
        SELECT excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.relationship_id)
    .bind(RelationshipEvidenceSourceKind::Communication.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored relationship evidence");
    let excerpt: Option<String> = evidence_row.try_get("excerpt").expect("evidence excerpt");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(excerpt.as_deref(), Some("Updated relationship evidence."));
    assert_eq!(metadata, json!({"channel": "email", "revision": 2}));

    let source_relationships = relationship_store
        .list_for_entity(RelationshipEntityKind::Persona, &source.persona_id, 10)
        .await
        .expect("source relationships");
    let target_relationships = relationship_store
        .list_for_entity(RelationshipEntityKind::Persona, &target.persona_id, 10)
        .await
        .expect("target relationships");

    assert!(
        source_relationships
            .iter()
            .any(|item| item.relationship_id == first.relationship_id)
    );
    assert!(
        target_relationships
            .iter()
            .any(|item| item.relationship_id == first.relationship_id)
    );
}

#[tokio::test]
async fn relationship_store_projects_persona_relationship_into_graph_against_postgres() {
    let Some((_test_context, pool, person_store, _relationship_store)) =
        live_relationship_context("persona relationship graph projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source = person_store
        .upsert_email_persona(&format!("graph-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_persona(&format!("graph-target-{suffix}@example.com"))
        .await
        .expect("target persona");

    let relationship = NewRelationship::between_personas(
        &source.persona_id,
        &target.persona_id,
        "knows",
        0.77,
        0.58,
        0.83,
        RelationshipReviewState::Suggested,
    );
    let stored = RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(
            &relationship,
            &[NewRelationshipEvidence::new(
                RelationshipEvidenceSourceKind::Communication,
                format!("message:graph-projection:{suffix}"),
            )
            .excerpt("Introduced during a project discussion.")],
        )
        .await
        .expect("relationship upsert with graph projection");

    let edge_row = sqlx::query(
        r#"
        SELECT edge.edge_id, edge.confidence::float8 AS confidence, edge.review_state, edge.properties
        FROM graph_edges edge
        WHERE edge.source_node_id = $1
          AND edge.target_node_id = $2
          AND edge.relationship_type = 'entity_relationship'
          AND edge.valid_to IS NULL
        "#,
    )
    .bind(node_id(GraphNodeKind::Persona, &source.persona_id))
    .bind(node_id(GraphNodeKind::Persona, &target.persona_id))
    .fetch_one(&pool)
    .await
    .expect("relationship graph edge");

    let confidence: f64 = edge_row.try_get("confidence").expect("graph confidence");
    let review_state: String = edge_row
        .try_get("review_state")
        .expect("graph review state");
    let properties: Value = edge_row.try_get("properties").expect("graph properties");
    assert_eq!(confidence, 0.83);
    assert_eq!(review_state, "suggested");
    assert_eq!(properties["relationship_id"], json!(stored.relationship_id));
    assert_eq!(properties["relationship_type"], json!("knows"));
    assert_eq!(properties["trust_score"], json!(0.77));
    assert_eq!(properties["strength_score"], json!(0.58));

    let edge_id: String = edge_row.try_get("edge_id").expect("graph edge id");
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = 'relationship'
          AND source_id = $2
        "#,
    )
    .bind(edge_id)
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship graph evidence count");

    assert_eq!(evidence_count, 1);
}

#[tokio::test]
async fn relationship_store_projects_supported_cross_domain_relationship_into_graph_against_postgres()
 {
    let Some((_test_context, pool, _person_store, _relationship_store)) =
        live_relationship_context("cross-domain relationship graph projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let decision_id = format!("decision:v1:relationship-graph:{suffix}");
    let project_id = format!("project:v1:relationship-graph:{suffix}");
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Decision,
        source_entity_id: decision_id.clone(),
        target_entity_kind: RelationshipEntityKind::Project,
        target_entity_id: project_id.clone(),
        relationship_type: "sets_direction_for".to_owned(),
        trust_score: 0.7,
        strength_score: 0.62,
        confidence: 0.86,
        review_state: RelationshipReviewState::Suggested,
        valid_from: None,
        valid_to: None,
        metadata: json!({"source": "relationships_cross_domain_test"}),
    };
    let stored = RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(
            &relationship,
            &[NewRelationshipEvidence::new(
                RelationshipEvidenceSourceKind::Decision,
                decision_id.clone(),
            )
            .excerpt("This decision sets direction for the project.")
            .metadata(json!({"source": "relationships_cross_domain_test"}))],
        )
        .await
        .expect("cross-domain relationship upsert with graph projection");

    let decision_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'decision' AND stable_key = $1",
    )
    .bind(&decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision relationship graph node");
    let project_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'project' AND stable_key = $1",
    )
    .bind(&project_id)
    .fetch_one(&pool)
    .await
    .expect("project relationship graph node");
    let graph_edge_row = sqlx::query(
        r#"
        SELECT edge_id, confidence::float8 AS confidence, review_state, properties
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
    .expect("cross-domain relationship graph edge");

    let confidence: f64 = graph_edge_row
        .try_get("confidence")
        .expect("graph confidence");
    let review_state: String = graph_edge_row
        .try_get("review_state")
        .expect("graph review state");
    let properties: Value = graph_edge_row
        .try_get("properties")
        .expect("graph properties");

    assert_eq!(confidence, 0.86);
    assert_eq!(review_state, "suggested");
    assert_eq!(properties["relationship_id"], json!(stored.relationship_id));
    assert_eq!(properties["relationship_type"], json!("sets_direction_for"));
    assert_eq!(properties["source_entity_kind"], json!("decision"));
    assert_eq!(properties["target_entity_kind"], json!("project"));

    let edge_id: String = graph_edge_row.try_get("edge_id").expect("graph edge id");
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = 'relationship'
          AND source_id = $2
        "#,
    )
    .bind(edge_id)
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship graph evidence count");

    assert_eq!(evidence_count, 1);
}

#[tokio::test]
async fn relationship_store_projects_organization_task_relationship_into_graph_against_postgres() {
    let Some((_test_context, pool, _person_store, _relationship_store)) =
        live_relationship_context("organization task relationship graph projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let organization_id = format!("organization:v1:relationship-graph:{suffix}");
    let task_id = format!("task:v1:relationship-graph:{suffix}");
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Organization,
        source_entity_id: organization_id.clone(),
        target_entity_kind: RelationshipEntityKind::Task,
        target_entity_id: task_id.clone(),
        relationship_type: "owns_work_item".to_owned(),
        trust_score: 0.68,
        strength_score: 0.57,
        confidence: 0.81,
        review_state: RelationshipReviewState::Suggested,
        valid_from: None,
        valid_to: None,
        metadata: json!({"source": "relationships_organization_task_graph_test"}),
    };
    let stored = RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(
            &relationship,
            &[NewRelationshipEvidence::new(
                RelationshipEvidenceSourceKind::Organization,
                organization_id.clone(),
            )
            .excerpt("This organization owns the referenced work item.")
            .metadata(json!({"source": "relationships_organization_task_graph_test"}))],
        )
        .await
        .expect("organization task relationship upsert with graph projection");

    let organization_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'organization' AND stable_key = $1",
    )
    .bind(&organization_id)
    .fetch_one(&pool)
    .await
    .expect("organization relationship graph node");
    let task_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'task' AND stable_key = $1",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task relationship graph node");
    let graph_edge_row = sqlx::query(
        r#"
        SELECT edge_id, confidence::float8 AS confidence, review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(&organization_node_id)
    .bind(&task_node_id)
    .fetch_one(&pool)
    .await
    .expect("organization task relationship graph edge");

    let confidence: f64 = graph_edge_row
        .try_get("confidence")
        .expect("graph confidence");
    let review_state: String = graph_edge_row
        .try_get("review_state")
        .expect("graph review state");
    let properties: Value = graph_edge_row
        .try_get("properties")
        .expect("graph properties");

    assert_eq!(confidence, 0.81);
    assert_eq!(review_state, "suggested");
    assert_eq!(properties["relationship_id"], json!(stored.relationship_id));
    assert_eq!(properties["relationship_type"], json!("owns_work_item"));
    assert_eq!(properties["source_entity_kind"], json!("organization"));
    assert_eq!(properties["target_entity_kind"], json!("task"));

    let edge_id: String = graph_edge_row.try_get("edge_id").expect("graph edge id");
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = 'relationship'
          AND source_id = $2
        "#,
    )
    .bind(edge_id)
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship graph evidence count");

    assert_eq!(evidence_count, 1);
}

#[tokio::test]
async fn organization_persona_link_materializes_member_of_relationship_against_postgres() {
    let Some((
        _test_context,
        pool,
        person_store,
        relationship_store,
        organization_store,
        persona_link_service,
    )) = live_organization_person_relationship_context("organization-persona relationship").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("org-person-relationship-{suffix}@example.com"))
        .await
        .expect("persona");
    let organization = organization_store
        .create(
            &format!("Organization Person Relationship {suffix}"),
            Some("company"),
        )
        .await
        .expect("organization");

    let link = persona_link_service
        .link_persona_manual(
            &organization.organization_id,
            &person.persona_id,
            Some("advisor"),
            Some("strategy"),
            "manual",
        )
        .await
        .expect("organization-person link");

    let relationships = relationship_store
        .list_for_entity(RelationshipEntityKind::Persona, &person.persona_id, 20)
        .await
        .expect("persona relationships");
    let relationship = relationships
        .iter()
        .find(|item| {
            item.source_entity_kind == RelationshipEntityKind::Persona
                && item.source_entity_id == person.persona_id
                && item.target_entity_kind == RelationshipEntityKind::Organization
                && item.target_entity_id == organization.organization_id
                && item.relationship_type == "member_of"
        })
        .expect("organization-person link should create member_of relationship");

    assert_eq!(
        relationship.review_state,
        RelationshipReviewState::UserConfirmed
    );
    assert_eq!(relationship.confidence, link.confidence);
    assert_eq!(
        relationship.metadata["compatibility_table"],
        json!("organization_persona_links")
    );
    assert_eq!(relationship.metadata["role"], json!("advisor"));
    assert_eq!(relationship.metadata["department"], json!("strategy"));

    let evidence_row = sqlx::query(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship evidence");
    let source_kind: String = evidence_row.try_get("source_kind").expect("source kind");
    let source_id: String = evidence_row.try_get("source_id").expect("source id");
    let excerpt: Option<String> = evidence_row.try_get("excerpt").expect("excerpt");
    let metadata: Value = evidence_row.try_get("metadata").expect("metadata");

    assert_eq!(
        source_kind,
        RelationshipEvidenceSourceKind::Observation.as_str()
    );
    assert!(source_id.starts_with("observation:v1:"));
    assert_eq!(
        excerpt.as_deref(),
        Some("Persona is linked to organization through organization-persona compatibility data.")
    );
    assert_eq!(
        metadata["organization_id"],
        json!(organization.organization_id)
    );
    assert_eq!(metadata["persona_id"], json!(person.persona_id));
    assert!(metadata.get("person_id").is_none());
}

#[tokio::test]
async fn relationship_store_rejects_missing_evidence_before_database_write() {
    let store = disconnected_relationship_store();
    let relationship = NewRelationship::between_personas(
        "person:v1:email:source@example.com",
        "person:v1:email:target@example.com",
        "knows",
        0.5,
        0.5,
        0.5,
        RelationshipReviewState::Suggested,
    );

    let error = store
        .upsert_with_evidence(&relationship, &[])
        .await
        .expect_err("relationship without evidence must fail before database write");

    assert!(matches!(error, RelationshipStoreError::MissingEvidence));
}

#[tokio::test]
async fn relationship_store_rejects_invalid_scores_before_database_write() {
    let store = disconnected_relationship_store();
    let relationship = NewRelationship::between_personas(
        "person:v1:email:source@example.com",
        "person:v1:email:target@example.com",
        "knows",
        1.1,
        0.5,
        0.5,
        RelationshipReviewState::Suggested,
    );
    let evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        "message:invalid-score",
    );

    let error = store
        .upsert_with_evidence(&relationship, &[evidence])
        .await
        .expect_err("invalid score must fail before database write");

    assert!(matches!(
        error,
        RelationshipStoreError::InvalidScore("trust_score", _)
    ));
}

#[tokio::test]
async fn relationship_store_rejects_identical_persona_endpoints_before_database_write() {
    let store = disconnected_relationship_store();
    let relationship = NewRelationship::between_personas(
        "person:v1:email:same@example.com",
        "person:v1:email:same@example.com",
        "knows",
        0.5,
        0.5,
        0.5,
        RelationshipReviewState::Suggested,
    );
    let evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        "message:identical-endpoints",
    );

    let error = store
        .upsert_with_evidence(&relationship, &[evidence])
        .await
        .expect_err("identical endpoints must fail before database write");

    assert!(matches!(error, RelationshipStoreError::IdenticalEndpoints));
}

#[tokio::test]
async fn relationship_store_rejects_missing_observation_evidence_against_postgres() {
    let Some((_test_context, _pool, _persons, store)) =
        live_relationship_context("missing relationship observation evidence").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let relationship = NewRelationship::between_personas(
        format!("person:v1:email:source-{suffix}@example.com"),
        format!("person:v1:email:target-{suffix}@example.com"),
        "supports",
        0.5,
        0.5,
        0.5,
        RelationshipReviewState::Suggested,
    );
    let evidence = NewRelationshipEvidence::observation(format!(
        "observation:v1:missing-relationship:{suffix}"
    ));

    let error = store
        .upsert_with_evidence(&relationship, &[evidence])
        .await
        .expect_err("missing observation evidence must fail");

    assert!(matches!(
        error,
        RelationshipStoreError::ObservationNotFound(_)
    ));
}

#[tokio::test]
async fn relationship_store_materializes_support_link_for_observation_evidence_against_postgres() {
    let Some((_test_context, pool, person_store, store)) =
        live_relationship_context("relationship support link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source = person_store
        .upsert_email_persona(&format!("relationship-support-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_persona(&format!("relationship-support-target-{suffix}@example.com"))
        .await
        .expect("target persona");
    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "subject": format!("Relationship support {suffix}"),
                    "body": "These two people collaborate closely on Макошь."
                }),
                format!("manual://relationship-support/{suffix}"),
            )
            .confidence(0.9),
        )
        .await
        .expect("support observation");

    let stored = store
        .upsert_with_evidence(
            &NewRelationship::between_personas(
                &source.persona_id,
                &target.persona_id,
                "collaborates_with",
                0.73,
                0.61,
                0.88,
                RelationshipReviewState::UserConfirmed,
            ),
            &[
                NewRelationshipEvidence::observation(observation.observation_id.clone())
                    .excerpt("These two people collaborate closely on Макошь."),
            ],
        )
        .await
        .expect("relationship upsert with observation evidence");

    let support_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'relationships'
          AND entity_kind = 'relationship'
          AND entity_id = $2
          AND relationship_kind = 'supports'
        "#,
    )
    .bind(&observation.observation_id)
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship support link count");
    assert_eq!(support_link_count, 1);
}

async fn live_relationship_context(
    _test_name: &str,
) -> Option<(
    TestContext,
    PgPool,
    PersonaProjectionStore,
    RelationshipStore,
)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((
        test_context,
        pool.clone(),
        PersonaProjectionStore::new(pool.clone()),
        RelationshipStore::new(pool),
    ))
}

async fn live_organization_person_relationship_context(
    _test_name: &str,
) -> Option<(
    TestContext,
    PgPool,
    PersonaProjectionStore,
    RelationshipStore,
    OrganizationStore,
    OrganizationPersonaLinkApplicationService,
)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((
        test_context,
        pool.clone(),
        PersonaProjectionStore::new(pool.clone()),
        RelationshipStore::new(pool.clone()),
        OrganizationStore::new(pool.clone()),
        OrganizationPersonaLinkApplicationService::new(pool),
    ))
}

fn disconnected_relationship_store() -> RelationshipStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    RelationshipStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
