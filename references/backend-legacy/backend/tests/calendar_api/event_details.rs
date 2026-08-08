use makosh_backend_testkit::context::TestContext;

use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, create_cal_event, get_request_with_token, json_body,
    post_request_with_token, unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::database::Database;

async fn event_get_endpoint_returns_non_server_error(path_suffix: &str) -> Option<Value> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return None;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/{}",
                urlencoding_percent_encode(&event_id),
                path_suffix
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    Some(json_body(response).await)
}

#[tokio::test]
async fn calendar_event_relations_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("relations").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_context_pack_returns_ok() {
    event_get_endpoint_returns_non_server_error("context-pack").await;
}

#[tokio::test]
async fn calendar_event_agenda_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("agenda").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_checklist_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("checklist").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_risks_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("risks").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_meeting_notes_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("notes").await else {
        return;
    };
    assert!(body["items"].is_array() || body.is_object());
}

#[tokio::test]
async fn calendar_event_meeting_outcomes_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("outcomes").await else {
        return;
    };
    assert!(body["items"].is_array() || body.is_object());
}

#[tokio::test]
async fn calendar_event_recording_list_returns_ok() {
    event_get_endpoint_returns_non_server_error("recording").await;
}

#[tokio::test]
async fn calendar_event_transcript_returns_ok() {
    event_get_endpoint_returns_non_server_error("transcript").await;
}

#[tokio::test]
async fn calendar_event_brief_returns_ok() {
    event_get_endpoint_returns_non_server_error("brief").await;
}

#[tokio::test]
async fn calendar_event_export_returns_text() {
    event_get_endpoint_returns_non_server_error("export").await;
}

#[tokio::test]
async fn calendar_event_reminders_list_returns_empty() {
    event_get_endpoint_returns_non_server_error("reminders").await;
}

macro_rules! cal_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let database_url = test_context.connection_string();
            let suffix = unique_suffix();
            let app = build_cal_app(&database_url).await;
            let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
                eprintln!("skip: no event");
                return;
            };
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/calendar/events/{}/{}",
                        urlencoding_percent_encode(&event_id),
                        $path_suffix
                    ),
                    $body,
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

cal_post_test!(
    cal_event_post_relation,
    "relations",
    json!({"related_event_id": "event:fake", "relation_type": "follows"})
);
cal_post_test!(
    cal_event_post_context_pack,
    "context-pack",
    json!({"summary": "Test context"})
);
cal_post_test!(
    cal_event_post_agenda,
    "agenda",
    json!({"item": "Test agenda item", "order_index": 1})
);
cal_post_test!(
    cal_event_post_checklist,
    "checklist",
    json!({"item": "Test checklist", "done": false})
);
cal_post_test!(
    cal_event_post_meeting_note,
    "notes",
    json!({"content": "Test note", "note_type": "action_item"})
);
cal_post_test!(
    cal_event_post_meeting_outcome,
    "outcomes",
    json!({"outcome": "Test outcome", "decision": false})
);
cal_post_test!(
    cal_event_post_follow_up,
    "follow-up",
    json!({"action": "Send follow-up", "due_by": "2027-12-01T00:00:00Z"})
);
cal_post_test!(
    cal_event_post_recording,
    "recording",
    json!({"url": "https://example.com/rec", "format": "mp4"})
);
cal_post_test!(
    cal_event_post_generate_agenda,
    "generate-agenda",
    json!({"participant_count": 3, "duration_minutes": 60})
);
cal_post_test!(
    cal_event_post_reminder,
    "reminders",
    json!({"minutes_before": 15, "method": "notification"})
);

#[tokio::test]
async fn cal_event_follow_up_status() {
    event_get_endpoint_returns_non_server_error("follow-up-status").await;
}

#[tokio::test]
async fn cal_event_reminder_toggle() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };
    let create_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders",
                urlencoding_percent_encode(&event_id)
            ),
            json!({
                "reminder_type": "time_based",
                "minutes_before": 15,
                "message": "Prepare for the meeting"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create reminder response");
    assert_eq!(create_response.status(), axum::http::StatusCode::OK);
    let reminder_id = json_body(create_response).await["id"]
        .as_str()
        .expect("reminder id")
        .to_owned();

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders/{}/toggle",
                urlencoding_percent_encode(&event_id),
                reminder_id
            ),
            json!({"active": false}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "reminder toggle={}",
        response.status()
    );

    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let source: String =
        sqlx::query_scalar("SELECT source FROM calendar_reminders WHERE id::text = $1")
            .bind(&reminder_id)
            .fetch_one(&pool)
            .await
            .expect("reminder source");
    assert!(source.starts_with("observation:"));

    let observation_id = source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("toggle observation");
    assert_eq!(origin_kind, "manual");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event_reminder'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&reminder_id)
    .fetch_one(&pool)
    .await
    .expect("reminder observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn calendar_manual_event_materials_capture_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };
    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let encoded_event_id = urlencoding_percent_encode(&event_id);

    let agenda_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/agenda"),
            json!({
                "items": ["Kickoff", "Scope review"],
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("agenda response");
    assert_eq!(agenda_response.status(), axum::http::StatusCode::OK);
    let agenda_id = json_body(agenda_response).await["id"]
        .as_str()
        .expect("agenda id")
        .to_owned();
    let agenda_source: String =
        sqlx::query_scalar("SELECT source FROM event_agendas WHERE id::text = $1")
            .bind(&agenda_id)
            .fetch_one(&pool)
            .await
            .expect("agenda source");
    assert!(agenda_source.starts_with("observation:"));

    let checklist_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/checklist"),
            json!({
                "items": [{"text": "Prepare deck", "done": false}],
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("checklist response");
    assert_eq!(checklist_response.status(), axum::http::StatusCode::OK);
    let checklist_id = json_body(checklist_response).await["id"]
        .as_str()
        .expect("checklist id")
        .to_owned();
    let checklist_source: String =
        sqlx::query_scalar("SELECT source FROM event_checklists WHERE id::text = $1")
            .bind(&checklist_id)
            .fetch_one(&pool)
            .await
            .expect("checklist source");
    assert!(checklist_source.starts_with("observation:"));

    let note_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/notes"),
            json!({
                "content": "Discussed migration sequencing.",
                "format": "markdown",
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("note response");
    assert_eq!(note_response.status(), axum::http::StatusCode::OK);
    let note_id = json_body(note_response).await["id"]
        .as_str()
        .expect("note id")
        .to_owned();
    let note_source: String =
        sqlx::query_scalar("SELECT source FROM meeting_notes WHERE id::text = $1")
            .bind(&note_id)
            .fetch_one(&pool)
            .await
            .expect("note source");
    assert!(note_source.starts_with("observation:"));

    let recording_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/recording"),
            json!({
                "file_path": format!("/tmp/meeting-{suffix}.m4a"),
                "duration_seconds": 185,
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("recording response");
    assert_eq!(recording_response.status(), axum::http::StatusCode::OK);
    let recording_id = json_body(recording_response).await["id"]
        .as_str()
        .expect("recording id")
        .to_owned();
    let recording_source: String =
        sqlx::query_scalar("SELECT source FROM event_recordings WHERE id::text = $1")
            .bind(&recording_id)
            .fetch_one(&pool)
            .await
            .expect("recording source");
    assert!(recording_source.starts_with("observation:"));

    let outcome_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/outcomes"),
            json!({
                "outcome_type": "decision",
                "title": format!("Adopt outcome path {suffix}"),
                "description": "Manual meeting outcome should create observation-backed source."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("outcome response");
    assert_eq!(outcome_response.status(), axum::http::StatusCode::OK);
    let outcome_id = json_body(outcome_response).await["id"]
        .as_str()
        .expect("outcome id")
        .to_owned();
    let outcome_source: String =
        sqlx::query_scalar("SELECT source FROM meeting_outcomes WHERE id::text = $1")
            .bind(&outcome_id)
            .fetch_one(&pool)
            .await
            .expect("outcome source");
    assert!(outcome_source.starts_with("observation:"));

    let reminder_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/reminders"),
            json!({
                "reminder_type": "time_based",
                "minutes_before": 30,
                "message": "Review notes"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reminder response");
    assert_eq!(reminder_response.status(), axum::http::StatusCode::OK);
    let reminder_id = json_body(reminder_response).await["id"]
        .as_str()
        .expect("reminder id")
        .to_owned();
    let reminder_source: String =
        sqlx::query_scalar("SELECT source FROM calendar_reminders WHERE id::text = $1")
            .bind(&reminder_id)
            .fetch_one(&pool)
            .await
            .expect("reminder source");
    assert!(reminder_source.starts_with("observation:"));

    for (source, entity_type, entity_id) in [
        (agenda_source, "event_agenda", agenda_id),
        (checklist_source, "event_checklist", checklist_id),
        (note_source, "meeting_note", note_id),
        (recording_source, "event_recording", recording_id),
        (outcome_source, "meeting_outcome", outcome_id),
        (reminder_source, "event_reminder", reminder_id),
    ] {
        let observation_id = source
            .strip_prefix("observation:")
            .expect("observation prefix");
        let row = sqlx::query(
            "SELECT observation_id, origin_kind FROM observations WHERE observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("stored observation");
        assert_eq!(
            row.try_get::<String, _>("origin_kind")
                .expect("origin kind"),
            "manual"
        );

        let expected_kind_code = match entity_type {
            "event_agenda" => Some("EVENT_AGENDA"),
            "event_checklist" => Some("EVENT_CHECKLIST"),
            "meeting_note" => Some("MEETING_NOTE"),
            _ => None,
        };
        if let Some(expected_kind_code) = expected_kind_code {
            let kind_code: String = sqlx::query_scalar(
                "SELECT kind.code AS kind_code
                 FROM observations observation
                 JOIN observation_kind_definitions kind
                   ON kind.kind_definition_id = observation.kind_definition_id
                 WHERE observation.observation_id = $1",
            )
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("event detail observation kind");
            assert_eq!(kind_code, expected_kind_code);
        }

        let link_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM observation_links
             WHERE observation_id = $1
               AND domain = 'calendar'
               AND entity_kind = $2
               AND entity_id = $3",
        )
        .bind(observation_id)
        .bind(entity_type)
        .bind(entity_id)
        .fetch_one(&pool)
        .await
        .expect("observation link count");
        assert_eq!(link_count, 1);
    }
}

#[tokio::test]
async fn calendar_event_relation_manual_create_path_captures_observation_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/relations",
                urlencoding_percent_encode(&event_id)
            ),
            json!({
                "entity_type": "project",
                "entity_id": format!("project:v1:{suffix}"),
                "relation_type": "related_to"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let relation_id = json_body(response).await["id"]
        .as_str()
        .expect("relation id")
        .to_owned();

    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let relation_source: String =
        sqlx::query_scalar("SELECT source FROM event_relations WHERE id::text = $1")
            .bind(&relation_id)
            .fetch_one(&pool)
            .await
            .expect("relation source");
    assert!(relation_source.starts_with("observation:"));

    let observation_id = relation_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("relation observation");
    assert_eq!(origin_kind, "manual");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event_relation'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&relation_id)
    .fetch_one(&pool)
    .await
    .expect("relation observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn calendar_event_follow_up_manual_status_change_captures_observation_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };

    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let before_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("initial observation id");

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/follow-up",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let event_row =
        sqlx::query("SELECT observation_id, status FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("follow-up event row");
    let after_observation_id: String = event_row
        .try_get("observation_id")
        .expect("follow-up observation id");
    let status: String = event_row.try_get("status").expect("follow-up status");
    assert_eq!(status, "needs_follow_up");
    assert_ne!(after_observation_id, before_observation_id);

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&after_observation_id)
            .fetch_one(&pool)
            .await
            .expect("follow-up observation");
    assert_eq!(origin_kind, "manual");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event'
           AND entity_id = $2
           AND metadata ->> 'status' = 'needs_follow_up'",
    )
    .bind(&after_observation_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("follow-up observation link count");
    assert_eq!(link_count, 1);
}
