use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::events::{
    account_store::CalendarAccountStore,
    event_store::CalendarEventStore,
    models::{CalendarAccountUpdate, CalendarEventUpdate, NewCalendarEvent},
    queries::CalendarEventListQuery,
    source_store::CalendarSourceStore,
};

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn calendar_account_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = CalendarAccountStore::new(pool);
    let suffix = unique_suffix();

    let acct = store
        .create("local", &format!("Test Account {suffix}"), None)
        .await
        .expect("create account");
    assert_eq!(acct.provider, "local");
    assert!(acct.account_id.starts_with("cal:v1:"));

    let fetched = store
        .get(&acct.account_id)
        .await
        .expect("get account")
        .expect("account exists");
    assert_eq!(fetched.account_name, acct.account_name);

    let update = CalendarAccountUpdate {
        account_name: Some(format!("Updated {suffix}")),
        ..Default::default()
    };
    let updated = store
        .update(&acct.account_id, &update)
        .await
        .expect("update account");
    assert_eq!(updated.account_name, format!("Updated {suffix}"));

    let list = store.list(Some("local")).await.expect("list accounts");
    assert!(list.iter().any(|a| a.account_id == acct.account_id));

    store
        .delete(&acct.account_id)
        .await
        .expect("delete account");
    assert!(
        store
            .get(&acct.account_id)
            .await
            .expect("get deleted")
            .is_none()
    );
}

#[tokio::test]
async fn calendar_source_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let src_store = CalendarSourceStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Src Test {suffix}"), None)
        .await
        .expect("create account");
    let src = src_store
        .create(
            &acct.account_id,
            "Work Calendar",
            Some("gcal-123"),
            Some("#4285f4"),
            Some("Europe/Madrid"),
        )
        .await
        .expect("create source");
    assert!(src.source_id.starts_with("src:v1:"));
    assert_eq!(src.name, "Work Calendar");
    assert_eq!(src.color.as_deref(), Some("#4285f4"));

    let list = src_store
        .list_by_account(&acct.account_id)
        .await
        .expect("list sources");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Work Calendar");
}

#[tokio::test]
async fn calendar_event_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Event Test {suffix}"), None)
        .await
        .expect("create account");
    let now = Utc::now();
    let req = NewCalendarEvent {
        title: format!("Test Event {suffix}"),
        description: Some("Test description".into()),
        start_at: now,
        end_at: now + Duration::hours(1),
        account_id: Some(acct.account_id.clone()),
        event_type: Some("meeting".into()),
        ..Default::default()
    };

    let event = event_store.create(&req).await.expect("create event");
    assert!(event.event_id.starts_with("evt:v1:"));
    assert!(event.observation_id.starts_with("observation:v1:"));
    assert_eq!(event.title, format!("Test Event {suffix}"));
    assert_eq!(event.status, "scheduled");

    let observation_kind: Option<String> = sqlx::query_scalar(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&event.observation_id)
    .fetch_optional(&pool)
    .await
    .expect("calendar event observation");
    assert_eq!(observation_kind.as_deref(), Some("CALENDAR_EVENT"));

    let fetched = event_store
        .get(&event.event_id)
        .await
        .expect("get event")
        .expect("event exists");
    assert_eq!(fetched.event_type.as_deref(), Some("meeting"));

    let update = CalendarEventUpdate {
        title: Some(format!("Updated {suffix}")),
        ..Default::default()
    };
    let updated = event_store
        .update(&event.event_id, &update)
        .await
        .expect("update event");
    assert_eq!(updated.title, format!("Updated {suffix}"));

    let list = event_store
        .list(&CalendarEventListQuery {
            from: Some(now - Duration::hours(1)),
            to: Some(now + Duration::hours(2)),
            limit: Some(50),
            ..Default::default()
        })
        .await
        .expect("list events");
    assert!(list.iter().any(|e| e.event_id == event.event_id));

    event_store
        .delete(&event.event_id)
        .await
        .expect("delete event");
    assert!(
        event_store
            .get(&event.event_id)
            .await
            .expect("get deleted")
            .is_none()
    );
}

#[tokio::test]
async fn event_reschedule_and_status_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Status Test {suffix}"), None)
        .await
        .expect("create account");
    let now = Utc::now();
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Reschedule {suffix}"),
            start_at: now,
            end_at: now + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let new_start = now + Duration::hours(2);
    let rescheduled = event_store
        .reschedule(&event.event_id, new_start, new_start + Duration::hours(1))
        .await
        .expect("reschedule");
    assert_eq!(rescheduled.status, "rescheduled");

    event_store
        .set_status(&event.event_id, "cancelled")
        .await
        .expect("cancel");
    let cancelled = event_store
        .get(&event.event_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(cancelled.status, "cancelled");
}
