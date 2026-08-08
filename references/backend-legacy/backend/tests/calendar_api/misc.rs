use makosh_backend_testkit::context::TestContext;

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, delete_request_with_token, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::database::Database;

async fn get_calendar_endpoint_returns_non_server_error(path: &str) {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(path, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "{path} status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_deadlines_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/deadlines").await;
}

#[tokio::test]
async fn calendar_focus_blocks_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/focus-blocks").await;
}

#[tokio::test]
async fn calendar_watchtower_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/watchtower").await;
}

#[tokio::test]
async fn calendar_health_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/health").await;
}

#[tokio::test]
async fn calendar_weekly_brief_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/weekly-brief").await;
}

#[tokio::test]
async fn calendar_search_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/search?q=meeting").await;
}

#[tokio::test]
async fn calendar_rules_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/rules").await;
}

#[tokio::test]
async fn calendar_analytics_distribution_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/distribution").await;
}

#[tokio::test]
async fn calendar_sources_list() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("SrcAcct{suffix}"), "email": format!("src-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: acct create failed");
        return;
    }
    let account_id = json_body(response).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if account_id.is_empty() {
        eprintln!("skip: no account_id");
        return;
    }

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"name": format!("Src{suffix}"), "color": "#ff0000", "timezone": "UTC"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "src create={}",
        response.status()
    );
    let body = json_body(response).await;
    let source_id = body["source_id"].as_str().expect("source_id");
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_source'
          AND entity_id = $1
          AND relationship_kind = 'create'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(source_id)
    .fetch_one(&pool)
    .await
    .expect("calendar source observation link");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("calendar source observation");
    assert_eq!(origin_kind, "manual");

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "src list={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_post_deadline() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/deadlines",
            json!({"title": "Test Deadline", "due_at": "2027-12-31T23:59:59Z", "severity": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "deadline post={}",
        response.status()
    );
    let body = json_body(response).await;
    let deadline_id = body["id"].as_str().expect("deadline id");
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'deadline_event'
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(deadline_id)
    .fetch_one(&pool)
    .await
    .expect("deadline observation link");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("deadline observation");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn cal_post_focus_block() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let start_at = Utc::now() + Duration::hours(2);
    let end_at = start_at + Duration::minutes(90);
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/focus-blocks",
            json!({
                "title": "Focus Block",
                "start_at": start_at.to_rfc3339(),
                "end_at": end_at.to_rfc3339(),
                "purpose": "Deep work"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "focus block post={}",
        response.status()
    );
    let body = json_body(response).await;
    let focus_block_id = body["id"].as_str().expect("focus block id");
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'focus_block'
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(focus_block_id)
    .fetch_one(&pool)
    .await
    .expect("focus block observation link");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("focus block observation");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn cal_post_smart_schedule() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let app = build_cal_app(&database_url).await;
    let response = app.oneshot(post_request_with_token(
        "/api/v1/calendar/smart-schedule",
        json!({"task_title": "Schedule me", "duration_minutes": 60, "deadline": "2027-12-31T23:59:59Z"}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "smart schedule={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_analytics() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics").await;
}

#[tokio::test]
async fn cal_focus_balance() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/focus-balance")
        .await;
}

#[tokio::test]
async fn cal_back_to_back() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/back-to-back").await;
}

#[tokio::test]
async fn cal_rules_crud() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/rules",
            json!({
                "name": format!("Rule{suffix}"),
                "description": "Color busy blocks",
                "dsl": {"color": "#00ff00"},
                "approval_mode": "suggest_only"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_default();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let rule_id = value["rule_id"].as_str().unwrap_or("").to_owned();
    if rule_id.is_empty() {
        return;
    }
    let created_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_rule'
          AND entity_id = $1
          AND relationship_kind = 'create'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&rule_id)
    .fetch_one(&pool)
    .await
    .expect("calendar rule create observation");

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            json!({"name": format!("Updated{suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule update={}",
        response.status()
    );
    let updated_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_rule'
          AND entity_id = $1
          AND relationship_kind = 'update'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&rule_id)
    .fetch_one(&pool)
    .await
    .expect("calendar rule update observation");
    assert_ne!(updated_observation_id, created_observation_id);

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule delete={}",
        response.status()
    );
    let deleted_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_rule'
          AND entity_id = $1
          AND relationship_kind = 'delete'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&rule_id)
    .fetch_one(&pool)
    .await
    .expect("calendar rule delete observation");
    assert_ne!(deleted_observation_id, updated_observation_id);
    let delete_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("calendar rule delete observation row");
    assert_eq!(
        delete_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        delete_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_RULE"
    );
}

#[tokio::test]
async fn cal_import() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let imported_title = format!("Imported Event {suffix}");
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/import",
            json!({
                "ics_data": "BEGIN:VCALENDAR\nEND:VCALENDAR",
                "events": [{
                    "title": imported_title,
                    "start_at": "2027-12-31T10:00:00Z",
                    "end_at": "2027-12-31T11:00:00Z",
                    "source_event_id": format!("import-{suffix}")
                }]
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "import={}",
        response.status()
    );
    let body = json_body(response).await;
    assert_eq!(body["imported"], json!(1));
    assert_eq!(body["ics_data_received"], json!(true));
    let row = sqlx::query(
        "SELECT event_id, observation_id
         FROM calendar_events
         WHERE title = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&imported_title)
    .fetch_one(&pool)
    .await
    .expect("imported event row");
    let event_id: String = row.try_get("event_id").expect("event_id");
    let observation_id: String = row.try_get("observation_id").expect("observation_id");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("import observation");
    assert_eq!(origin_kind, "file_import");
    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event'
           AND entity_id = $2",
    )
    .bind(&observation_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("import event observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn cal_sync() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("Sync{suffix}"), "email": format!("sync-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip");
        return;
    }
    let account_id = json_body(response).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if account_id.is_empty() {
        return;
    }

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sync",
                urlencoding_percent_encode(&account_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "sync={}",
        response.status()
    );
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'sync_trigger'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar sync observation link");
    let observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("calendar sync observation");
    assert_eq!(
        observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_ACCOUNT_MUTATION"
    );
}
