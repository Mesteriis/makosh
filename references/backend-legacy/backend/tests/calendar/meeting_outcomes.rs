use chrono::{Duration, Utc};
use makosh_hub_backend::application::calendar_meeting_outcomes::CalendarMeetingOutcomeApplicationService;
use makosh_hub_backend::domains::calendar::events::{
    account_store::CalendarAccountStore, event_store::CalendarEventStore, models::NewCalendarEvent,
};
use makosh_hub_backend::domains::calendar::meetings::{
    notes::MeetingNoteStore, outcomes::MeetingOutcomeStore,
};
use makosh_hub_backend::domains::obligations::models::entity_kind::ObligationEntityKind;

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn meeting_notes_and_outcomes_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let note_store = MeetingNoteStore::new(pool.clone());
    let outcome_store = MeetingOutcomeStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Meeting Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");
    assert!(event.observation_id.starts_with("observation:v1:"));

    let note = note_store
        .create(
            &event.event_id,
            "# Meeting Notes\n\nDiscussed scope.",
            Some("markdown"),
            Some("manual"),
        )
        .await
        .expect("create note");
    assert!(note.content.contains("Meeting Notes"));

    let outcome = outcome_store
        .add(
            &event.event_id,
            "decision",
            "Use Rust",
            Some("Decided to use Rust for backend"),
            None,
            None,
            Some("manual"),
        )
        .await
        .expect("add outcome");
    assert_eq!(outcome.outcome_type, "decision");
    assert_eq!(outcome.source, "manual");

    let notes = note_store.list(&event.event_id).await.expect("list notes");
    assert_eq!(notes.len(), 1);

    let outcomes = outcome_store
        .list(&event.event_id)
        .await
        .expect("list outcomes");
    assert_eq!(outcomes.len(), 1);
}

#[tokio::test]
async fn meeting_outcome_decision_creates_suggested_decision_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let outcome_service = CalendarMeetingOutcomeApplicationService::new(pool.clone());
    let decision_store = DecisionStore::new(pool.clone());
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Decision Outcome Test {suffix}"), None)
        .await
        .expect("create account");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Decision outcome meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            event_type: Some("meeting".into()),
            ..Default::default()
        })
        .await
        .expect("create event");

    let outcome = outcome_service
        .add_manual(
            &event.event_id,
            "decision",
            &format!("Adopt meeting outcome adapter {suffix}"),
            Some("We decided to persist meeting decisions as reviewable domain Decisions."),
            None,
            None,
        )
        .await
        .expect("add decision outcome");
    let linked_decision_id = outcome
        .linked_entity_id
        .as_deref()
        .expect("decision outcome should link to suggested Decision");

    let decisions = decision_store
        .list_for_entity(DecisionEntityKind::Event, &event.event_id, 10)
        .await
        .expect("event decisions");
    let decision = decisions
        .iter()
        .find(|item| item.decision_id == linked_decision_id)
        .expect("suggested Decision linked to meeting outcome");

    assert_eq!(decision.title, outcome.title);
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(
        decision.rationale,
        "We decided to persist meeting decisions as reviewable domain Decisions."
    );

    let evidence: (String, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, observation_id, quote FROM decision_evidence WHERE decision_id = $1",
    )
    .bind(linked_decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence.0, "event");
    assert_eq!(evidence.1, event.event_id);
    assert_eq!(evidence.2.as_deref(), Some(event.observation_id.as_str()));
    assert_eq!(
        evidence.3.as_deref(),
        Some("We decided to persist meeting decisions as reviewable domain Decisions.")
    );

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'decision_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_decision'
          AND review_item.metadata->>'decision_id' = $2
        "#,
    )
    .bind(&event.observation_id)
    .bind(linked_decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision review mirror");
    assert_eq!(review_item.1, "potential_decision");
    assert_eq!(review_item.2, "decisions");
    assert_eq!(review_item.3, linked_decision_id);
}

#[tokio::test]
async fn meeting_outcome_promise_creates_suggested_obligation_and_review_item_without_task_link_against_postgres()
 {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let outcome_service = CalendarMeetingOutcomeApplicationService::new(pool.clone());
    let obligation_store = ObligationStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner_person_id = format!("person:v1:email:meeting-promise-{suffix}@example.com");
    let due_at = Utc::now() + Duration::days(3);

    let acct = acct_store
        .create("local", &format!("Promise Outcome Test {suffix}"), None)
        .await
        .expect("create account");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Promise outcome meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            event_type: Some("meeting".into()),
            ..Default::default()
        })
        .await
        .expect("create event");

    let outcome = outcome_service
        .add_manual(
            &event.event_id,
            "promise",
            &format!("Send meeting follow-up package {suffix}"),
            Some("Alex promised to send the follow-up package after the meeting."),
            Some(&owner_person_id),
            Some(due_at),
        )
        .await
        .expect("add promise outcome");
    let linked_obligation_id = outcome
        .linked_entity_id
        .as_deref()
        .expect("promise outcome should link to suggested Obligation");

    let obligations = obligation_store
        .list_for_entity(ObligationEntityKind::Persona, &owner_person_id, 10)
        .await
        .expect("owner obligations");
    let obligation = obligations
        .iter()
        .find(|item| item.obligation_id == linked_obligation_id)
        .expect("suggested Obligation linked to meeting outcome");

    assert_eq!(obligation.statement, outcome.title);
    assert_eq!(obligation.review_state, ObligationReviewState::Suggested);
    assert_eq!(
        obligation.due_at.map(|value| value.timestamp_micros()),
        Some(due_at.timestamp_micros())
    );

    let evidence: (String, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, observation_id, quote FROM obligation_evidence WHERE obligation_id = $1",
    )
    .bind(linked_obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence.0, "event");
    assert_eq!(evidence.1, event.event_id);
    assert_eq!(evidence.2.as_deref(), Some(event.observation_id.as_str()));
    assert_eq!(
        evidence.3.as_deref(),
        Some("Alex promised to send the follow-up package after the meeting.")
    );

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'obligation_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_obligation'
          AND review_item.metadata->>'obligation_id' = $2
        "#,
    )
    .bind(&event.observation_id)
    .bind(linked_obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation review mirror");
    assert_eq!(review_item.1, "potential_obligation");
    assert_eq!(review_item.2, "obligations");
    assert_eq!(review_item.3, linked_obligation_id);

    let task_link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(linked_obligation_id)
            .fetch_one(&pool)
            .await
            .expect("task link count");
    assert_eq!(task_link_count, 0);
}
use makosh_hub_backend::domains::decisions::models::{
    entity_kind::DecisionEntityKind, states::DecisionReviewState,
};
use makosh_hub_backend::domains::decisions::store::DecisionStore;
use makosh_hub_backend::domains::obligations::models::states::ObligationReviewState;
use makosh_hub_backend::domains::obligations::store::ObligationStore;
