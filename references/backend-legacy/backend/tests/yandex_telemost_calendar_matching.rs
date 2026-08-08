use chrono::{TimeZone, Utc};
use makosh_backend_testkit::context::TestContext;
use makosh_events_api::EventEnvelope;
use makosh_hub_backend::domains::calendar::events::event_store::CalendarEventStore;
use makosh_hub_backend::domains::calendar::events::models::NewCalendarEvent;
use makosh_hub_backend::domains::calendar::ports::EventParticipantPort;
use makosh_hub_backend::platform::events::bus::yandex_telemost_event_types;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::yandex_telemost_calendar_matching::project_yandex_telemost_calendar_matching;
use serde_json::json;

const TELEMOST_PARTICIPANT_SOURCE: &str = "yandex_telemost_cohost_observed";

#[tokio::test]
async fn telemost_cohosts_are_projected_into_matched_calendar_event_participants() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    let start_at = Utc
        .with_ymd_and_hms(2026, 6, 28, 10, 0, 0)
        .single()
        .expect("valid datetime");
    let end_at = Utc
        .with_ymd_and_hms(2026, 6, 28, 11, 0, 0)
        .single()
        .expect("valid datetime");
    let event = CalendarEventStore::new(pool.clone())
        .create_manual(&NewCalendarEvent {
            title: "Telemost planning".to_owned(),
            start_at,
            end_at,
            conference_url: Some("https://telemost.yandex.ru/j/abcdef".to_owned()),
            conference_provider: Some("yandex_telemost".to_owned()),
            ..NewCalendarEvent::default()
        })
        .await
        .expect("calendar event");

    let projection_event = EventEnvelope {
        event_id: "evt-telemost-cohosts".to_owned(),
        event_type: yandex_telemost_event_types::COHOSTS_OBSERVED.to_owned(),
        schema_version: 1,
        occurred_at: end_at,
        recorded_at: end_at,
        source: json!({}),
        actor: Some(json!({})),
        subject: json!({}),
        payload: json!({
            "account_id": "telemost-main",
            "conference_id": "abcdef",
            "cohosts": [
                { "email": "cohost1@yandex.ru" },
                { "email": "COHOST1@YANDEX.RU" },
                { "email": "cohost2@yandex.ru" }
            ]
        }),
        provenance: json!({}),
        causation_id: None,
        correlation_id: None,
    };

    project_yandex_telemost_calendar_matching(&pool, &projection_event)
        .await
        .expect("calendar participant projection");

    let participants = EventParticipantPort::new(pool)
        .list(&event.event_id)
        .await
        .expect("event participants");
    let projected = participants
        .into_iter()
        .filter(|participant| participant.source == TELEMOST_PARTICIPANT_SOURCE)
        .collect::<Vec<_>>();

    assert_eq!(projected.len(), 2);
    assert_eq!(projected[0].role, "attendee");
    assert_eq!(projected[0].email, "cohost1@yandex.ru");
    assert_eq!(projected[1].email, "cohost2@yandex.ru");
}
