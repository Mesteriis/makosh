use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::core::{
    agendas::EventAgendaStore, checklists::EventChecklistStore, context_packs::ContextPackInput,
    context_packs::EventContextPackStore, participants::EventParticipantStore,
    relations::EventRelationStore,
};
use makosh_hub_backend::domains::calendar::events::{
    account_store::CalendarAccountStore, event_store::CalendarEventStore, models::NewCalendarEvent,
};
use serde_json::json;

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn event_participants_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let part_store = EventParticipantStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Part Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Participants {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let participant = part_store
        .add(
            &event.event_id,
            &format!("john-{suffix}@test.com"),
            Some("John"),
            Some("required"),
            None,
            None,
        )
        .await
        .expect("add participant");
    assert_eq!(participant.role, "required");
    assert_eq!(participant.email, format!("john-{suffix}@test.com"));

    let list = part_store
        .list(&event.event_id)
        .await
        .expect("list participants");
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn event_relations_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let rel_store = EventRelationStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Rel Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Relations {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let rel = rel_store
        .link(&event.event_id, "project", "proj-1", "related_to")
        .await
        .expect("link");
    assert_eq!(rel.entity_type, "project");

    let list = rel_store
        .list(&event.event_id)
        .await
        .expect("list relations");
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn event_context_pack_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let ctx_store = EventContextPackStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Ctx Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Context {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let input = ContextPackInput {
        summary: Some("Test summary".into()),
        documents: json!([]),
        tasks: json!([]),
        open_questions: json!(["Q1"]),
        risks: json!(["No agenda"]),
        suggested_agenda: json!([]),
        suggested_actions: json!([]),
        ..Default::default()
    };
    let pack = ctx_store
        .upsert(&event.event_id, &input)
        .await
        .expect("upsert context pack");
    assert_eq!(pack.summary.as_deref(), Some("Test summary"));

    let fetched = ctx_store
        .get(&event.event_id)
        .await
        .expect("get pack")
        .expect("pack exists");
    assert_eq!(fetched.summary.as_deref(), Some("Test summary"));
}

#[tokio::test]
async fn event_agenda_and_checklist_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let agenda_store = EventAgendaStore::new(pool.clone());
    let cl_store = EventChecklistStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Agenda Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Agenda {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let agenda = agenda_store
        .set(&event.event_id, json!(["Item 1", "Item 2"]), "manual")
        .await
        .expect("set agenda");
    assert_eq!(agenda.source, "manual");

    let checklist = cl_store
        .set(
            &event.event_id,
            json!([{"text": "Prepare docs", "done": false}]),
            "manual",
        )
        .await
        .expect("set checklist");
    assert_eq!(checklist.source, "manual");

    let fetched_agenda = agenda_store
        .get(&event.event_id)
        .await
        .expect("get agenda")
        .expect("exists");
    let items = fetched_agenda.items.as_array().expect("array");
    assert_eq!(items.len(), 2);
}
