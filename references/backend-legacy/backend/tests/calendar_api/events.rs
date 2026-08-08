use makosh_backend_testkit::context::TestContext;

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, create_cal_event, delete_request_with_token,
    get_request_with_token, json_body, post_request_with_token, put_request_with_token,
    unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn calendar_events_crud_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
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
    let fetched = json_body(response).await;
    assert_eq!(fetched["event_id"], json!(event_id));

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"title": format!("Updated Event {suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Event {suffix}")));

    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
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
}

#[tokio::test]
async fn calendar_events_list_returns_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    create_cal_event(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/events",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    let _items = body["items"].as_array().expect("items");
}

#[tokio::test]
async fn calendar_event_reschedule() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let new_start = Utc::now() + Duration::hours(3);
    let new_end = Utc::now() + Duration::hours(4);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reschedule",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"start_at": new_start.to_rfc3339(), "end_at": new_end.to_rfc3339()}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_event_cancel() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/cancel",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert_eq!(body["cancelled"], json!(true));
}

#[tokio::test]
async fn calendar_event_participants_crud() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };
    let legacy_persona_id = format!("persona:calendar-participant:{suffix}");

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/participants",
                urlencoding_percent_encode(&event_id)
            ),
            json!({
                "email": format!("participant-{suffix}@example.com"),
                "display_name": format!("Participant {suffix}"),
                "role": "required",
                "person_id": legacy_persona_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let created = json_body(response).await;
    let participant_id = created["id"].as_str().expect("participant id").to_owned();
    assert_eq!(created["persona_id"], json!(legacy_persona_id));
    assert!(created.get("person_id").is_none());

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/participants",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());

    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let participant_source: String =
        sqlx::query_scalar("SELECT source FROM event_participants WHERE id::text = $1")
            .bind(&participant_id)
            .fetch_one(&pool)
            .await
            .expect("participant source");
    assert!(participant_source.starts_with("observation:"));

    let observation_id = participant_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("participant observation");
    assert_eq!(origin_kind, "manual");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event_participant'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&participant_id)
    .fetch_one(&pool)
    .await
    .expect("participant observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn calendar_event_manual_lifecycle_captures_append_only_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };
    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();

    let created_row =
        sqlx::query("SELECT observation_id, status FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("created event row");
    let created_observation_id: String = created_row
        .try_get("observation_id")
        .expect("created observation id");
    let created_status: String = created_row.try_get("status").expect("created status");
    assert_eq!(created_status, "confirmed");
    let created_kind: String = sqlx::query_scalar(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&created_observation_id)
    .fetch_one(&pool)
    .await
    .expect("created observation kind");
    let created_origin: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&created_observation_id)
            .fetch_one(&pool)
            .await
            .expect("created observation origin");
    assert_eq!(created_kind, "CALENDAR_EVENT");
    assert_eq!(created_origin, "manual");

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"title": format!("Updated Event {suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("update response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated_row =
        sqlx::query("SELECT observation_id, title FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("updated event row");
    let updated_observation_id: String = updated_row
        .try_get("observation_id")
        .expect("updated observation id");
    assert_ne!(updated_observation_id, created_observation_id);

    let new_start = Utc::now() + Duration::hours(3);
    let new_end = Utc::now() + Duration::hours(4);
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reschedule",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"start_at": new_start.to_rfc3339(), "end_at": new_end.to_rfc3339()}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reschedule response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let rescheduled_row =
        sqlx::query("SELECT observation_id, status FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("rescheduled event row");
    let rescheduled_observation_id: String = rescheduled_row
        .try_get("observation_id")
        .expect("rescheduled observation id");
    let rescheduled_status: String = rescheduled_row
        .try_get("status")
        .expect("rescheduled status");
    assert_ne!(rescheduled_observation_id, updated_observation_id);
    assert_eq!(rescheduled_status, "rescheduled");

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/cancel",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("cancel response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let cancelled_row =
        sqlx::query("SELECT observation_id, status FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("cancelled event row");
    let cancelled_observation_id: String = cancelled_row
        .try_get("observation_id")
        .expect("cancelled observation id");
    let cancelled_status: String = cancelled_row.try_get("status").expect("cancelled status");
    assert_ne!(cancelled_observation_id, rescheduled_observation_id);
    assert_eq!(cancelled_status, "cancelled");

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let deleted_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("deleted count");
    assert_eq!(deleted_count, 0);

    let deleted_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'calendar'
           AND entity_kind = 'event'
           AND entity_id = $1
           AND metadata ->> 'action' = 'delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("delete observation link");
    let deleted_kind: String = sqlx::query_scalar(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("deleted observation kind");
    assert_eq!(deleted_kind, "CALENDAR_EVENT_DELETED");

    let evidence_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'calendar'
           AND entity_kind = 'event'
           AND entity_id = $1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("event observation links");
    assert!(evidence_count >= 5);
}

#[tokio::test]
async fn calendar_event_intelligence_updates_capture_runtime_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
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
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/classify",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("classify response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let classified_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("classified observation id");
    assert_ne!(classified_observation_id, before_observation_id);
    let classify_origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&classified_observation_id)
            .fetch_one(&pool)
            .await
            .expect("classify observation");
    assert_eq!(classify_origin_kind, "local_runtime");

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/analyze",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("analyze response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let analyzed_row = sqlx::query(
        "SELECT observation_id, importance_score, readiness_score
         FROM calendar_events
         WHERE event_id = $1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("analyzed event row");
    let analyzed_observation_id: String = analyzed_row
        .try_get("observation_id")
        .expect("analyzed observation id");
    assert_ne!(analyzed_observation_id, classified_observation_id);
    let analyze_origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&analyzed_observation_id)
            .fetch_one(&pool)
            .await
            .expect("analyze observation");
    assert_eq!(analyze_origin_kind, "local_runtime");
    let analysis_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event'
           AND entity_id = $2
           AND relationship_kind = 'runtime_update'",
    )
    .bind(&analyzed_observation_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("analysis observation link count");
    assert_eq!(analysis_link_count, 1);
}
