use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::identity::{
    errors::PersonaIdentityError,
    models::{PersonaIdentityReviewCommand, PersonaIdentityReviewState},
    store::PersonaIdentityReviewStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

const PERSONA_IDENTITY_REVIEW_EVENT_TYPE: &str = "persona_identity.review_state_changed";
const LEGACY_PERSON_IDENTITY_REVIEW_EVENT_TYPE: &str = "person_identity.review_state_changed";

pub(crate) struct PersonaIdentityTestContext {
    _test_context: TestContext,
    pub(crate) pool: PgPool,
    pub(crate) store: PersonaIdentityReviewStore,
    pub(crate) event_store: EventStore,
    pub(crate) persona_store: PersonaProjectionStore,
}

pub(crate) async fn live_persona_identity_context() -> Option<PersonaIdentityTestContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(PersonaIdentityTestContext {
        _test_context: test_context,
        pool: pool.clone(),
        store: PersonaIdentityReviewStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        persona_store: PersonaProjectionStore::new(pool.clone()),
    })
}

pub(crate) async fn seed_normalized_personas(
    context: &PersonaIdentityTestContext,
    left_persona_id: &str,
    right_persona_id: &str,
    display_name: &str,
) -> Result<(), PersonaIdentityError> {
    sqlx::query(
        r#"
        UPDATE personas
        SET display_name = $1
        WHERE persona_id = $2 OR persona_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_persona_id)
    .bind(right_persona_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn confirm_identity_candidate(
    context: &PersonaIdentityTestContext,
    identity_candidate_id: &str,
    command_id: &str,
) -> Result<(), PersonaIdentityError> {
    context
        .store
        .set_review_state(&PersonaIdentityReviewCommand {
            command_id: command_id.to_owned(),
            identity_candidate_id: identity_candidate_id.to_owned(),
            review_state: PersonaIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await?;

    Ok(())
}

pub(crate) async fn exclude_personas_from_name_merge_refresh(
    context: &PersonaIdentityTestContext,
    persona_ids: &[&str],
    suffix: u128,
) -> Result<(), PersonaIdentityError> {
    for (index, persona_id) in persona_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE personas
            SET display_name = $1
            WHERE persona_id = $2
            "#,
        )
        .bind(format!(
            "identity-refresh-skip-{suffix}-{index}@example.com"
        ))
        .bind(persona_id)
        .execute(&context.pool)
        .await?;
    }

    Ok(())
}

pub(crate) async fn promote_identity_candidate(
    context: &PersonaIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonaIdentityError> {
    sqlx::query(
        r#"
        UPDATE persona_identity_candidates
        SET updated_at = clock_timestamp()
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn age_identity_candidate(
    context: &PersonaIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonaIdentityError> {
    sqlx::query(
        r#"
        UPDATE persona_identity_candidates
        SET updated_at = clock_timestamp() - INTERVAL '1 hour'
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn assert_identity_candidate_exists(
    context: &PersonaIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonaIdentityError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM persona_identity_candidates
            WHERE identity_candidate_id = $1
        )
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?
    .then_some(())
    .ok_or(PersonaIdentityError::IdentityCandidateNotFound)
}

pub(crate) async fn identity_candidate_updated_at(
    context: &PersonaIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<chrono::DateTime<Utc>, PersonaIdentityError> {
    let updated_at = sqlx::query_scalar(
        r#"
        SELECT updated_at
        FROM persona_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?;

    Ok(updated_at)
}

pub(crate) fn identity_candidate_id_from_personas(left_id: &str, right_id: &str) -> String {
    let (left_persona_id, right_persona_id) = ordered_persona_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_personas:{left_persona_id}:{right_persona_id}")
}

pub(crate) fn split_identity_candidate_id_from_personas(left_id: &str, right_id: &str) -> String {
    let (left_persona_id, right_persona_id) = ordered_persona_ids(left_id, right_id);
    format!("identity_candidate:v1:split_persona:{left_persona_id}:{right_persona_id}")
}

pub(crate) fn ordered_persona_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

pub(crate) fn build_review_event(
    identity_candidate_id: &str,
    review_state: PersonaIdentityReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    build_review_event_with_type(
        identity_candidate_id,
        review_state,
        actor_id,
        command_id,
        PERSONA_IDENTITY_REVIEW_EVENT_TYPE,
        "persona_identity_review",
        "persona_identity_review:",
    )
}

pub(crate) fn build_legacy_review_event(
    identity_candidate_id: &str,
    review_state: PersonaIdentityReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    build_review_event_with_type(
        identity_candidate_id,
        review_state,
        actor_id,
        command_id,
        LEGACY_PERSON_IDENTITY_REVIEW_EVENT_TYPE,
        "person_identity_review",
        "person_identity_review:",
    )
}

fn build_review_event_with_type(
    identity_candidate_id: &str,
    review_state: PersonaIdentityReviewState,
    actor_id: &str,
    command_id: &str,
    event_type: &str,
    review_kind: &str,
    event_id_prefix: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("{event_id_prefix}{command_id}"),
        event_type,
        Utc::now(),
        json!({
            "kind": review_kind,
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": review_kind}),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "identity_candidate_id": identity_candidate_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("review event")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
