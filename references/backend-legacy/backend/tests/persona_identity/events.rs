use makosh_hub_backend::domains::personas::identity::models::{
    PersonaIdentityReviewCommand, PersonaIdentityReviewState,
};

use super::support::{
    build_legacy_review_event, build_review_event, identity_candidate_id_from_personas,
    live_persona_identity_context, seed_normalized_personas, unique_suffix,
};

#[tokio::test]
async fn persona_identity_reject_suppresses_candidate_against_postgres() {
    let Some(context) = live_persona_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Sam Candidate {suffix}");

    let left = context
        .persona_store
        .upsert_email_persona(&format!("sam.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .persona_store
        .upsert_email_persona(&format!("sam.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_personas(&context, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);

    let _ = context
        .store
        .set_review_state(&PersonaIdentityReviewCommand {
            command_id: format!("identity-reject-{suffix}"),
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: PersonaIdentityReviewState::UserRejected,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reject candidate");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh again");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM persona_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");
}

#[tokio::test]
async fn legacy_person_identity_review_event_still_rebuilds_state_against_postgres() {
    let Some(context) = live_persona_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Legacy Candidate {suffix}");

    let left = context
        .persona_store
        .upsert_email_persona(&format!("legacy.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .persona_store
        .upsert_email_persona(&format!("legacy.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_personas(&context, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);

    let legacy_event = build_legacy_review_event(
        &identity_candidate_id,
        PersonaIdentityReviewState::UserConfirmed,
        "legacy-event-reviewer",
        &format!("identity-event-legacy-{suffix}"),
    );
    context
        .event_store
        .append(&legacy_event)
        .await
        .expect("append legacy event");

    let legacy_event = context
        .event_store
        .get_by_id(&legacy_event.event_id)
        .await
        .expect("load legacy event")
        .expect("legacy event exists");
    context
        .store
        .apply_review_event(&legacy_event)
        .await
        .expect("apply legacy event");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM persona_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_confirmed");
}

#[tokio::test]
async fn persona_identity_review_event_rebuilds_state_against_postgres() {
    let Some(context) = live_persona_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Pat Candidate {suffix}");

    let left = context
        .persona_store
        .upsert_email_persona(&format!("pat.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .persona_store
        .upsert_email_persona(&format!("pat.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_personas(&context, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);

    let confirm_event = build_review_event(
        &identity_candidate_id,
        PersonaIdentityReviewState::UserConfirmed,
        "event-reviewer",
        &format!("identity-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &identity_candidate_id,
        PersonaIdentityReviewState::UserRejected,
        "event-reviewer",
        &format!("identity-event-reject-{suffix}"),
    );

    context
        .event_store
        .append(&confirm_event)
        .await
        .expect("append confirm event");
    context
        .event_store
        .append(&reject_event)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event.event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    context
        .store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm event");

    let reject_event = context
        .event_store
        .get_by_id(&reject_event.event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject event");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM persona_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String = sqlx::query_scalar(
        "SELECT event_id FROM persona_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load event id");
    assert_eq!(
        event_id,
        format!("persona_identity_review:identity-event-reject-{suffix}")
    );
}
