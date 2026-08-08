use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::consumers::EventConsumerConfig;
use makosh_events_postgres::consumers::EventConsumerRunner;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::projects::core::models::NewProject;
use makosh_hub_backend::domains::projects::core::store::ProjectStore;
use makosh_hub_backend::domains::projects::link_reviews::models::{
    ProjectLinkReviewCommand, ProjectLinkReviewCommandResult, ProjectLinkReviewState,
    ProjectLinkTargetKind,
};
use makosh_hub_backend::domains::projects::link_reviews::store::ProjectLinkReviewStore;
use makosh_hub_backend::domains::relationships::{
    models::{RelationshipEntityKind, RelationshipReviewState},
    store::RelationshipStore,
};

use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::project_link_review_effects::{
    PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, project_link_review_effect_event,
};

const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";

#[tokio::test]
async fn project_link_review_command_appends_event_and_updates_review_against_postgres() {
    let Some(context) = live_review_context("project link review command").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectLinkReview{suffix}");
    let project_id = format!("project:v1:review:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Link Review {suffix}"),
                "Product Development",
                "Project link review event test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-{suffix}"),
        &format!("{keyword} kickoff"),
        "Review review candidate",
    )
    .await;
    let command_id = format!("link-review-confirm-{suffix}");
    let command = ProjectLinkReviewCommand {
        command_id: command_id.clone(),
        project_id: project_id.clone(),
        target_kind: ProjectLinkTargetKind::Message,
        target_id: message_id.clone(),
        review_state: ProjectLinkReviewState::UserConfirmed,
        actor_id: "reviewer".to_owned(),
    };
    let result = context
        .review_store
        .set_review_state(&command)
        .await
        .expect("set review state");

    assert_eq!(
        result,
        ProjectLinkReviewCommandResult {
            project_id,
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id,
            review_state: ProjectLinkReviewState::UserConfirmed,
            event_id: format!("project_link_review:{command_id}"),
        }
    );

    let review = context
        .review_store
        .explicit_review(&result.project_id, result.target_kind, &result.target_id)
        .await
        .expect("load review row")
        .expect("review exists");
    assert_eq!(review.review_state, ProjectLinkReviewState::UserConfirmed);
    assert_eq!(review.event_id, result.event_id);
}

#[tokio::test]
async fn project_link_review_confirm_materializes_user_confirmed_decision_against_postgres() {
    let Some(context) = live_review_context("project link review decision adapter").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectDecisionAdapter{suffix}");
    let project_id = format!("project:v1:review-decision:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Review Decision {suffix}"),
                "Product Development",
                "Project link review decision adapter test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(64),
        )
        .await
        .expect("upsert review decision project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("decision-reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-decision-{suffix}"),
        &format!("{keyword} proposal"),
        "Review decision adapter body",
    )
    .await;
    let command_id = format!("link-review-decision-confirm-{suffix}");
    let result = context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: command_id.clone(),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("confirm project link review");
    run_project_link_review_effects(&context).await;

    let decisions = context
        .decision_store
        .list_for_entity(DecisionEntityKind::Project, &project_id, 20)
        .await
        .expect("project decisions");
    let decision = decisions
        .iter()
        .find(|item| item.metadata["project_link_review_event_id"] == json!(result.event_id))
        .expect("confirmed project link review should create a durable Decision");

    assert_eq!(decision.review_state, DecisionReviewState::UserConfirmed);
    assert_eq!(
        decision.rationale,
        "User confirmed a message link candidate for this project."
    );
    assert_eq!(decision.metadata["project_id"], json!(project_id));
    assert_eq!(decision.metadata["target_kind"], json!("message"));
    assert_eq!(decision.metadata["target_id"], json!(message_id));

    let impacted_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM decision_impacted_entities
        WHERE decision_id = $1
          AND (
            (entity_kind = 'project' AND entity_id = $2)
            OR (entity_kind = 'communication' AND entity_id = $3)
          )
        "#,
    )
    .bind(&decision.decision_id)
    .bind(&project_id)
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("decision impacted entities");
    assert_eq!(impacted_count, 2);

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, quote FROM decision_evidence WHERE decision_id = $1",
    )
    .bind(&decision.decision_id)
    .fetch_one(&context.pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(
        evidence.2.as_deref(),
        Some("User confirmed message link to project.")
    );

    let observation_kind: String = sqlx::query_scalar(
        "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
    )
    .bind(&evidence.1)
    .fetch_one(&context.pool)
    .await
    .expect("project link review decision observation kind");
    assert_eq!(observation_kind, "PROJECT_LINK_REVIEW");
}

#[tokio::test]
async fn project_link_review_confirm_materializes_relationship_against_postgres() {
    let Some(context) = live_review_context("project link review relationship adapter").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectRelationshipAdapter{suffix}");
    let project_id = format!("project:v1:review-relationship:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Review Relationship {suffix}"),
                "Product Development",
                "Project link review relationship adapter test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(64),
        )
        .await
        .expect("upsert review relationship project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("relationship-reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-relationship-{suffix}"),
        &format!("{keyword} proposal"),
        "Review relationship adapter body",
    )
    .await;
    let command_id = format!("link-review-relationship-confirm-{suffix}");
    let _result = context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: command_id.clone(),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("confirm project link review");
    run_project_link_review_effects(&context).await;

    let relationships = context
        .relationship_store
        .list_for_entity(RelationshipEntityKind::Project, &project_id, 20)
        .await
        .expect("project relationships");
    let relationship = relationships
        .iter()
        .find(|item| {
            item.source_entity_kind == RelationshipEntityKind::Project
                && item.source_entity_id == project_id
                && item.target_entity_kind == RelationshipEntityKind::Communication
                && item.target_entity_id == message_id
                && item.relationship_type == "project_has_message"
        })
        .expect("confirmed project link review should create a durable Relationship");

    assert_eq!(
        relationship.review_state,
        RelationshipReviewState::UserConfirmed
    );
    assert_eq!(relationship.confidence, 1.0);
    assert_eq!(
        relationship.metadata["compatibility_table"],
        json!("project_link_reviews")
    );
    assert_eq!(relationship.metadata["project_id"], json!(project_id));
    assert_eq!(relationship.metadata["target_kind"], json!("message"));
    assert_eq!(relationship.metadata["target_id"], json!(message_id));

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, excerpt FROM relationship_evidence WHERE relationship_id = $1",
    )
    .bind(&relationship.relationship_id)
    .fetch_one(&context.pool)
    .await
    .expect("relationship evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(
        evidence.2.as_deref(),
        Some("User confirmed message link to project.")
    );

    let observation_kind: String = sqlx::query_scalar(
        "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
    )
    .bind(&evidence.1)
    .fetch_one(&context.pool)
    .await
    .expect("project link review relationship observation kind");
    assert_eq!(observation_kind, "PROJECT_LINK_REVIEW");
}

#[tokio::test]
async fn project_link_review_reset_clears_review_and_demotes_relationship_against_postgres() {
    let Some(context) = live_review_context("project link review reset").await else {
        return;
    };
    let suffix = unique_suffix();
    let project_id = format!("project:v1:review-reset:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Review Reset {suffix}"),
                "Product Development",
                "Reset path test",
                "Alex Morgan",
                vec![format!("Reset{suffix}")],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("owner-{suffix}@example.com"),
        &[format!("reviewer-{suffix}@example.com")],
        &format!("provider-link-review-reset-{suffix}"),
        &format!("Reset keyword {suffix}"),
        "Review reset body",
    )
    .await;

    context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("link-review-reject-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::UserRejected,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("reject link first");

    let result = context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("link-review-suggested-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::Suggested,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("reset link");
    run_project_link_review_effects(&context).await;
    assert_eq!(result.review_state, ProjectLinkReviewState::Suggested);

    let review = context
        .review_store
        .explicit_review(&project_id, ProjectLinkTargetKind::Message, &message_id)
        .await
        .expect("load review after reset");
    assert_eq!(review, None);

    let relationships = context
        .relationship_store
        .list_for_entity(RelationshipEntityKind::Project, &project_id, 20)
        .await
        .expect("project relationships after reset");
    let relationship = relationships
        .iter()
        .find(|item| {
            item.source_entity_kind == RelationshipEntityKind::Project
                && item.source_entity_id == project_id
                && item.target_entity_kind == RelationshipEntityKind::Communication
                && item.target_entity_id == message_id
                && item.relationship_type == "project_has_message"
        })
        .expect("reset project link review should retain a suggested Relationship candidate");
    assert_eq!(
        relationship.review_state,
        RelationshipReviewState::Suggested
    );
    assert_eq!(
        relationship.metadata["project_link_review_event_id"],
        json!(result.event_id)
    );

    let reset_evidence: (String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt
        FROM relationship_evidence
        WHERE relationship_id = $1
          AND metadata->>'event_id' = $2
        "#,
    )
    .bind(&relationship.relationship_id)
    .bind(&result.event_id)
    .fetch_one(&context.pool)
    .await
    .expect("reset relationship evidence");
    assert_eq!(reset_evidence.0, "observation");
    assert!(!reset_evidence.1.is_empty());
    assert_eq!(
        reset_evidence.2.as_deref(),
        Some("User reset message link review for project.")
    );

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'relationship_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_relationship'
          AND review_item.metadata->>'relationship_id' = $2
        "#,
    )
    .bind(&reset_evidence.1)
    .bind(&relationship.relationship_id)
    .fetch_one(&context.pool)
    .await
    .expect("project link relationship review mirror");
    assert_eq!(review_item.1, "potential_relationship");
    assert_eq!(review_item.2, "relationships");
    assert_eq!(review_item.3, relationship.relationship_id);
}

#[tokio::test]
async fn project_link_review_projection_rebuilds_review_state_from_event_against_postgres() {
    let Some(context) = live_review_context("project link review projection replay").await else {
        return;
    };
    let suffix = unique_suffix();
    let project_id = format!("project:v1:review-replay:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Review Replay {suffix}"),
                "Product Development",
                "Replay review projection test",
                "Alex Morgan",
                vec![format!("Replay{suffix}")],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("replay-owner-{suffix}@example.com"),
        &[format!("replay-reviewer-{suffix}@example.com")],
        &format!("provider-link-review-replay-{suffix}"),
        &format!("Replay keyword {suffix}"),
        "Replay body",
    )
    .await;
    let actor_id = "projection-reviewer";
    let confirm_event_id = format!("evt_link_review_replay_confirm_{suffix}");
    let confirm = build_review_event(
        &project_id,
        ProjectLinkTargetKind::Message,
        &message_id,
        ProjectLinkReviewState::UserConfirmed,
        actor_id,
        &confirm_event_id,
    );
    let reject_event_id = format!("evt_link_review_replay_reject_{suffix}");
    let reject = build_review_event(
        &project_id,
        ProjectLinkTargetKind::Message,
        &message_id,
        ProjectLinkReviewState::UserRejected,
        actor_id,
        &reject_event_id,
    );
    let _ = context
        .event_store
        .append(&confirm)
        .await
        .expect("append confirm event");
    let _ = context
        .event_store
        .append(&reject)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    let reject_event = context
        .event_store
        .get_by_id(&reject_event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .review_store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm");
    context
        .review_store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject");

    let review = context
        .review_store
        .explicit_review(&project_id, ProjectLinkTargetKind::Message, &message_id)
        .await
        .expect("load replayed review")
        .expect("replayed review exists");
    assert_eq!(review.review_state, ProjectLinkReviewState::UserRejected);
    assert_eq!(review.event_id, reject_event_id);
}

struct LiveReviewContext {
    pool: sqlx::PgPool,
    project_store: ProjectStore,
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
    review_store: ProjectLinkReviewStore,
    decision_store: DecisionStore,
    relationship_store: RelationshipStore,
    event_store: EventStore,
}

async fn live_review_context(_test_name: &str) -> Option<LiveReviewContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let _ = database_url;

    Some(LiveReviewContext {
        pool: pool.clone(),
        project_store: ProjectStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        review_store: ProjectLinkReviewStore::new(pool.clone()),
        decision_store: DecisionStore::new(pool.clone()),
        relationship_store: RelationshipStore::new(pool.clone()),
        event_store: EventStore::new(pool),
    })
}

async fn run_project_link_review_effects(context: &LiveReviewContext) {
    let runner = EventConsumerRunner::new(
        context.pool.clone(),
        EventConsumerConfig::new(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER),
    );
    runner
        .process_next_batch(|event| project_link_review_effect_event(context.pool.clone(), event))
        .await
        .expect("project link review effects consumer");
}

async fn seed_message(
    context: &LiveReviewContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_link_review_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Link Review Gmail",
            format!("link-review-{suffix}@example.com"),
        ))
        .await
        .expect("store link review provider account");

    let raw_record_id = format!("raw_link_review_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:link-review:{suffix}:{provider_record_id}"),
                format!("batch-link-review-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"project_link_review_test"})),
        )
        .await
        .expect("record link review raw message");

    project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project link review message")
        .message_id
}

fn build_review_event(
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    review_state: ProjectLinkReviewState,
    actor_id: &str,
    event_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        event_id,
        PROJECT_LINK_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "project_link_review",
            "provider": "local_api",
            "source_id": event_id,
        }),
        json!({
            "kind": "project_link_review",
            "project_id": project_id,
        }),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "project_id": project_id,
        "target_kind": target_kind.as_str(),
        "target_id": target_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("valid review event")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
use makosh_hub_backend::domains::decisions::models::entity_kind::DecisionEntityKind;
use makosh_hub_backend::domains::decisions::models::states::DecisionReviewState;
use makosh_hub_backend::domains::decisions::store::DecisionStore;
