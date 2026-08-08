use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::Utc;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, project_accepted_signal_if_runtime_allows,
};
use makosh_hub_backend::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;

use makosh_backend_testkit::app::{TestApp, delete, get, patch_json, post_json};
use makosh_backend_testkit::composition::router_for_context;
use makosh_backend_testkit::context::TestContext;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

async fn test_app() -> TestApp {
    let context = TestContext::new().await;
    let router = router_for_context(&context);
    TestApp::new(context, router)
}

#[tokio::test]
async fn signal_hub_api_restores_fixture_and_lists_sources() {
    let app = test_app().await;
    let router = app.clone_router();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/system/restore",
            serde_json::json!({}),
        ))
        .await
        .expect("restore request");

    assert_eq!(restore_response.status(), StatusCode::OK);

    let response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/sources"))
        .await
        .expect("sources request");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let body: Value = serde_json::from_slice(&bytes).expect("json body");
    let codes: Vec<&str> = body["items"]
        .as_array()
        .expect("items array")
        .iter()
        .map(|item| item["code"].as_str().expect("source code"))
        .collect();

    assert_eq!(
        codes,
        vec![
            "ai",
            "browser",
            "calendar",
            "filesystem",
            "fixture",
            "github",
            "home_assistant",
            "mail",
            "rss",
            "system",
            "telegram",
            "voice",
            "whatsapp",
            "yandex_telemost",
            "zoom",
            "zulip",
        ]
    );

    let emit_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/fixture_basic_message/emit",
            serde_json::json!({}),
        ))
        .await
        .expect("emit fixture request");
    assert_eq!(emit_response.status(), StatusCode::OK);
    let emit_body = to_bytes(emit_response.into_body(), usize::MAX)
        .await
        .expect("emit fixture body");
    let emit_json: Value = serde_json::from_slice(&emit_body).expect("emit fixture json");
    assert_eq!(emit_json["item"]["fixture_id"], "fixture_basic_message");
    assert_eq!(emit_json["item"]["source_code"], "fixture");

    let fixture_list_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/fixtures"))
        .await
        .expect("fixture list request");
    assert_eq!(fixture_list_response.status(), StatusCode::OK);
    let fixture_list_body = to_bytes(fixture_list_response.into_body(), usize::MAX)
        .await
        .expect("fixture list body");
    let fixture_list_json: Value =
        serde_json::from_slice(&fixture_list_body).expect("fixture list json");
    let fixture = fixture_list_json["items"]
        .as_array()
        .expect("fixture items")
        .iter()
        .find(|item| item["fixture_id"] == "fixture_basic_message")
        .expect("fixture basic message in list");
    assert_eq!(fixture["source_code"], "fixture");
    assert_eq!(fixture["event_type"], "signal.raw.fixture.message.observed");
    assert_eq!(fixture["summary"], "Fixture message");

    let profiles_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/profiles"))
        .await
        .expect("profiles request");
    assert_eq!(profiles_response.status(), StatusCode::OK);
    let profiles_body = to_bytes(profiles_response.into_body(), usize::MAX)
        .await
        .expect("profiles body");
    let profiles_json: Value = serde_json::from_slice(&profiles_body).expect("profiles json");
    assert!(
        profiles_json["items"]
            .as_array()
            .expect("profile items")
            .iter()
            .any(|item| item["code"] == "testing")
    );

    let apply_profile_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/profiles/testing/apply",
            serde_json::json!({}),
        ))
        .await
        .expect("apply profile request");
    assert_eq!(apply_profile_response.status(), StatusCode::OK);
    let apply_profile_body = to_bytes(apply_profile_response.into_body(), usize::MAX)
        .await
        .expect("apply profile body");
    let apply_profile_json: Value =
        serde_json::from_slice(&apply_profile_body).expect("apply profile json");
    assert_eq!(apply_profile_json["code"], "testing");
    assert_eq!(apply_profile_json["is_active"], true);

    let capabilities_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/capabilities?source_code=telegram"))
        .await
        .expect("capabilities request");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = to_bytes(capabilities_response.into_body(), usize::MAX)
        .await
        .expect("capabilities body");
    let capabilities_json: Value =
        serde_json::from_slice(&capabilities_body).expect("capabilities json");
    assert!(
        capabilities_json["items"]
            .as_array()
            .expect("capability items")
            .iter()
            .any(|item| item["capability"] == "runtime.replay")
    );
    assert!(
        capabilities_json["items"]
            .as_array()
            .expect("capability items")
            .iter()
            .all(|item| item["state"] == "degraded")
    );

    let disable_source_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/sources/telegram/disable",
            serde_json::json!({}),
        ))
        .await
        .expect("disable source request");
    assert_eq!(disable_source_response.status(), StatusCode::OK);

    let blocked_capabilities_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/capabilities?source_code=telegram"))
        .await
        .expect("blocked capabilities request");
    assert_eq!(blocked_capabilities_response.status(), StatusCode::OK);
    let blocked_capabilities_body = to_bytes(blocked_capabilities_response.into_body(), usize::MAX)
        .await
        .expect("blocked capabilities body");
    let blocked_capabilities_json: Value =
        serde_json::from_slice(&blocked_capabilities_body).expect("blocked capabilities json");
    assert!(
        blocked_capabilities_json["items"]
            .as_array()
            .expect("blocked capability items")
            .iter()
            .any(|item| item["capability"] == "runtime.replay" && item["state"] == "blocked")
    );

    let create_profile_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/profiles",
            serde_json::json!({
                "code": "quiet_hours",
                "display_name": "Quiet Hours",
                "description": "Mute noisy overnight signals",
                "source_policies": [
                    {
                        "scope": "source",
                        "source_code": "telegram",
                        "mode": "muted",
                        "reason": "night mute"
                    }
                ]
            }),
        ))
        .await
        .expect("create profile request");
    assert_eq!(create_profile_response.status(), StatusCode::OK);
    let create_profile_body = to_bytes(create_profile_response.into_body(), usize::MAX)
        .await
        .expect("create profile body");
    let create_profile_json: Value =
        serde_json::from_slice(&create_profile_body).expect("create profile json");
    assert_eq!(create_profile_json["code"], "quiet_hours");
    assert_eq!(
        create_profile_json["source_policies"][0]["source_code"],
        "telegram"
    );

    let update_profile_response = router
        .clone()
        .oneshot(patch_json(
            "/api/v1/signal-hub/profiles/quiet_hours",
            serde_json::json!({
                "description": "Updated quiet profile",
                "source_policies": [
                    {
                        "scope": "event_pattern",
                        "event_pattern": "signal.raw.mail.*",
                        "mode": "paused",
                        "reason": "overnight mail pause"
                    }
                ]
            }),
        ))
        .await
        .expect("update profile request");
    assert_eq!(update_profile_response.status(), StatusCode::OK);
    let update_profile_body = to_bytes(update_profile_response.into_body(), usize::MAX)
        .await
        .expect("update profile body");
    let update_profile_json: Value =
        serde_json::from_slice(&update_profile_body).expect("update profile json");
    assert_eq!(update_profile_json["description"], "Updated quiet profile");
    assert_eq!(
        update_profile_json["source_policies"][0]["event_pattern"],
        "signal.raw.mail.*"
    );

    let delete_profile_response = router
        .clone()
        .oneshot(delete("/api/v1/signal-hub/profiles/quiet_hours"))
        .await
        .expect("delete profile request");
    assert_eq!(delete_profile_response.status(), StatusCode::OK);
    let delete_profile_body = to_bytes(delete_profile_response.into_body(), usize::MAX)
        .await
        .expect("delete profile body");
    let delete_profile_json: Value =
        serde_json::from_slice(&delete_profile_body).expect("delete profile json");
    assert_eq!(delete_profile_json["code"], "quiet_hours");

    let system_profile_update_response = router
        .clone()
        .oneshot(patch_json(
            "/api/v1/signal-hub/profiles/testing",
            serde_json::json!({
                "description": "should fail"
            }),
        ))
        .await
        .expect("system profile update request");
    assert_eq!(
        system_profile_update_response.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn signal_hub_connect_api_requires_local_api_secret() {
    let app = test_app().await;
    let router = app.clone_router();

    let forbidden_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/makosh.signal_hub.v1.SignalHubService/ListSources")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("connect request without secret"),
        )
        .await
        .expect("connect response without secret");
    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);
    let forbidden_body = to_bytes(forbidden_response.into_body(), usize::MAX)
        .await
        .expect("forbidden body");
    let forbidden_json: Value =
        serde_json::from_slice(&forbidden_body).expect("forbidden json body");
    assert_eq!(forbidden_json["error"], "invalid_api_secret");

    let allowed_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListSources",
            serde_json::json!({}),
        ))
        .await
        .expect("connect response with secret");
    assert_eq!(allowed_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn signal_hub_api_runs_ai_health_check_against_runtime_status() {
    let ctx = TestContext::new().await;
    ApplicationSettingsStore::new(ctx.pool().clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &serde_json::json!("http://127.0.0.1:9"),
            "makosh-test",
        )
        .await
        .expect("unreachable Ollama setting");
    let config = ctx
        .app_config("makosh-test-api-secret")
        .with_test_pairs([("MAKOSH_OLLAMA_BASE_URL", "http://127.0.0.1:9")])
        .expect("ai runtime test config");
    let router = build_router_with_database(config, ctx.database());

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/system/restore",
            serde_json::json!({}),
        ))
        .await
        .expect("restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let health_check_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/health",
            serde_json::json!({
                "source_code": "ai"
            }),
        ))
        .await
        .expect("ai health check request");
    assert_eq!(health_check_response.status(), StatusCode::OK);
    let health_check_body = to_bytes(health_check_response.into_body(), usize::MAX)
        .await
        .expect("ai health check body");
    let health_check_json: Value =
        serde_json::from_slice(&health_check_body).expect("ai health check json");
    assert_eq!(health_check_json["source_code"], "ai");
    assert_eq!(health_check_json["level"], "degraded");
    assert_eq!(
        health_check_json["evidence"]["health_origin"],
        "ai_runtime_status"
    );
    assert_eq!(health_check_json["evidence"]["runtime"], "ollama");
}

#[tokio::test]
async fn signal_hub_policy_api_can_pause_all_raw_signals() {
    let app = test_app().await;
    let router = app.clone_router();

    let create_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/policies",
            serde_json::json!({
                "scope": "event_pattern",
                "event_pattern": "signal.raw.*",
                "mode": "paused",
                "reason": "maintenance window"
            }),
        ))
        .await
        .expect("create policy request");
    assert_eq!(create_response.status(), StatusCode::OK);

    let response = router
        .oneshot(get("/api/v1/signal-hub/policies"))
        .await
        .expect("policies request");
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let body: Value = serde_json::from_slice(&bytes).expect("json body");
    let policy = body["items"]
        .as_array()
        .expect("items array")
        .iter()
        .find(|item| item["event_pattern"] == "signal.raw.*")
        .expect("global raw signal pause policy");

    assert_eq!(policy["scope"], "event_pattern");
    assert_eq!(policy["mode"], "paused");
    assert_eq!(policy["source_code"], Value::Null);
}

#[tokio::test]
async fn signal_hub_api_can_toggle_source_and_scoped_signal_controls() {
    let app = test_app().await;
    let router = app.clone_router();
    let pool = app.context().pool();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/system/restore",
            serde_json::json!({}),
        ))
        .await
        .expect("restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let source_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/sources/telegram"))
        .await
        .expect("get source request");
    assert_eq!(source_response.status(), StatusCode::OK);
    let source_body = to_bytes(source_response.into_body(), usize::MAX)
        .await
        .expect("source body");
    let source_json: Value = serde_json::from_slice(&source_body).expect("source json");
    assert_eq!(source_json["code"], "telegram");

    sqlx::query(
        r#"
        INSERT INTO signal_runtime_states (
            id,
            source_code,
            connection_id,
            runtime_kind,
            state,
            metadata
        )
        VALUES ($1, 'telegram', NULL, 'telegram_command_executor', 'running', '{"scope":"runtime"}'::jsonb)
        "#,
    )
    .bind(Uuid::now_v7())
    .execute(pool)
    .await
    .expect("insert telegram runtime state");

    let disable_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/sources/telegram/disable",
            serde_json::json!({}),
        ))
        .await
        .expect("disable source request");
    assert_eq!(disable_response.status(), StatusCode::OK);
    let disable_body = to_bytes(disable_response.into_body(), usize::MAX)
        .await
        .expect("disable source body");
    let disable_json: Value = serde_json::from_slice(&disable_body).expect("disable source json");
    assert_eq!(disable_json["source_code"], "telegram");
    assert!(disable_json["policy_id"].is_string());

    let disabled_runtime: String = sqlx::query_scalar(
        r#"
        SELECT state
        FROM signal_runtime_states
        WHERE source_code = 'telegram'
          AND runtime_kind = 'telegram_command_executor'
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("load disabled telegram runtime");
    assert_eq!(disabled_runtime, "stopped");

    let mute_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/MuteSignals",
            serde_json::json!({
                "scope": "event_pattern",
                "eventPattern": "signal.raw.telegram.*",
                "reason": "owner mute"
            }),
        ))
        .await
        .expect("connect mute request");
    assert_eq!(mute_response.status(), StatusCode::OK);

    let disable_signals_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/DisableSignals",
            serde_json::json!({
                "scope": "event_pattern",
                "eventPattern": "signal.accepted.telegram.*",
                "reason": "owner disable accepted telegram stream"
            }),
        ))
        .await
        .expect("connect disable signals request");
    assert_eq!(disable_signals_response.status(), StatusCode::OK);

    let pause_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/signals/pause",
            serde_json::json!({
                "scope": "global",
                "reason": "owner pause"
            }),
        ))
        .await
        .expect("pause request");
    assert_eq!(pause_response.status(), StatusCode::OK);

    let policies_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/policies"))
        .await
        .expect("policies request");
    assert_eq!(policies_response.status(), StatusCode::OK);
    let policies_body = to_bytes(policies_response.into_body(), usize::MAX)
        .await
        .expect("policies body");
    let policies_json: Value = serde_json::from_slice(&policies_body).expect("policies json");
    let policies = policies_json["items"].as_array().expect("policy items");
    assert!(policies.iter().any(|item| {
        item["scope"] == "source" && item["mode"] == "disabled" && item["source_code"] == "telegram"
    }));
    assert!(policies.iter().any(|item| {
        item["scope"] == "event_pattern"
            && item["mode"] == "muted"
            && item["event_pattern"] == "signal.raw.telegram.*"
    }));
    assert!(policies.iter().any(|item| {
        item["scope"] == "event_pattern"
            && item["mode"] == "disabled"
            && item["event_pattern"] == "signal.accepted.telegram.*"
    }));
    assert!(
        policies
            .iter()
            .any(|item| item["scope"] == "global" && item["mode"] == "paused")
    );

    let enable_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/EnableSource",
            serde_json::json!({
                "sourceCode": "telegram"
            }),
        ))
        .await
        .expect("enable source request");
    assert_eq!(enable_response.status(), StatusCode::OK);
    let enable_body = to_bytes(enable_response.into_body(), usize::MAX)
        .await
        .expect("enable body");
    let enable_json: Value = serde_json::from_slice(&enable_body).expect("enable json");
    assert_eq!(enable_json["sourceCode"], "telegram");
    assert_eq!(enable_json["clearedCount"], 1);
    let enabled_runtime: String = sqlx::query_scalar(
        r#"
        SELECT state
        FROM signal_runtime_states
        WHERE source_code = 'telegram'
          AND runtime_kind = 'telegram_command_executor'
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("load enabled telegram runtime");
    assert_eq!(enabled_runtime, "paused");

    let unmute_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UnmuteSignals",
            serde_json::json!({
                "scope": "event_pattern",
                "eventPattern": "signal.raw.telegram.*",
                "reason": "owner unmute"
            }),
        ))
        .await
        .expect("unmute request");
    assert_eq!(unmute_response.status(), StatusCode::OK);
    let unmute_body = to_bytes(unmute_response.into_body(), usize::MAX)
        .await
        .expect("unmute body");
    let unmute_json: Value = serde_json::from_slice(&unmute_body).expect("unmute json");
    assert_eq!(unmute_json["clearedCount"], 1);

    let enable_signals_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/EnableSignals",
            serde_json::json!({
                "scope": "event_pattern",
                "eventPattern": "signal.accepted.telegram.*",
                "reason": "owner re-enable accepted telegram stream"
            }),
        ))
        .await
        .expect("enable signals request");
    assert_eq!(enable_signals_response.status(), StatusCode::OK);
    let enable_signals_body = to_bytes(enable_signals_response.into_body(), usize::MAX)
        .await
        .expect("enable signals body");
    let enable_signals_json: Value =
        serde_json::from_slice(&enable_signals_body).expect("enable signals json");
    assert_eq!(enable_signals_json["clearedCount"], 1);

    let resume_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/signals/resume",
            serde_json::json!({
                "scope": "global",
                "reason": "owner resume"
            }),
        ))
        .await
        .expect("resume request");
    assert_eq!(resume_response.status(), StatusCode::OK);
    let resume_body = to_bytes(resume_response.into_body(), usize::MAX)
        .await
        .expect("resume body");
    let resume_json: Value = serde_json::from_slice(&resume_body).expect("resume json");
    assert_eq!(resume_json["cleared_count"], 1);
}

#[tokio::test]
async fn signal_hub_api_lists_connections_and_health() {
    let app = test_app().await;
    let pool = app.context().pool();
    let connection_id = Uuid::now_v7();
    let health_id = Uuid::now_v7();

    let restore_response = app
        .clone_router()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/system/restore",
            serde_json::json!({}),
        ))
        .await
        .expect("restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    sqlx::query(
        r#"
        INSERT INTO signal_connections (
            id,
            source_code,
            display_name,
            status,
            profile,
            settings,
            secret_ref
        )
        VALUES ($1, 'telegram', 'Personal Telegram', 'connected', 'default', '{}'::jsonb, 'vault://telegram')
        "#,
    )
    .bind(connection_id)
    .execute(pool)
    .await
    .expect("insert signal connection");

    sqlx::query(
        r#"
        INSERT INTO signal_health (
            id,
            source_code,
            connection_id,
            level,
            summary,
            failure_count,
            consecutive_failure_count,
            evidence
        )
        VALUES ($1, 'telegram', $2, 'healthy', 'Runtime heartbeat is current', 0, 0, '{"heartbeat":"ok"}'::jsonb)
        "#,
    )
    .bind(health_id)
    .bind(connection_id)
    .execute(pool)
    .await
    .expect("insert signal health");

    let connections_response = app
        .clone_router()
        .oneshot(get("/api/v1/signal-hub/connections"))
        .await
        .expect("connections request");
    assert_eq!(connections_response.status(), StatusCode::OK);
    let connections_body = to_bytes(connections_response.into_body(), usize::MAX)
        .await
        .expect("connections body");
    let connections_json: Value =
        serde_json::from_slice(&connections_body).expect("connections json");
    let connection = connections_json["items"]
        .as_array()
        .expect("connections items")
        .iter()
        .find(|item| item["id"] == connection_id.to_string())
        .expect("telegram connection");
    assert_eq!(connection["status"], "connected");
    assert_eq!(connection["profile"], "default");

    let health_response = app
        .clone_router()
        .oneshot(get("/api/v1/signal-hub/health"))
        .await
        .expect("health request");
    assert_eq!(health_response.status(), StatusCode::OK);
    let health_body = to_bytes(health_response.into_body(), usize::MAX)
        .await
        .expect("health body");
    let health_json: Value = serde_json::from_slice(&health_body).expect("health json");
    let health = health_json["items"]
        .as_array()
        .expect("health items")
        .iter()
        .find(|item| item["id"] == health_id.to_string())
        .expect("telegram health");
    assert_eq!(health["level"], "healthy");
    assert_eq!(health["summary"], "Runtime heartbeat is current");
    assert_eq!(health["connection_id"], connection_id.to_string());

    let health_check_response = app
        .clone_router()
        .oneshot(post_json(
            "/api/v1/signal-hub/health",
            serde_json::json!({
                "source_code": "system",
                "runtime_kind": "signal_hub_raw_signal_dispatcher"
            }),
        ))
        .await
        .expect("health check request");
    assert_eq!(health_check_response.status(), StatusCode::OK);
    let health_check_body = to_bytes(health_check_response.into_body(), usize::MAX)
        .await
        .expect("health check body");
    let health_check_json: Value =
        serde_json::from_slice(&health_check_body).expect("health check json");
    assert_eq!(health_check_json["source_code"], "system");
    assert!(health_check_json["level"].is_string());

    let runtime_response = app
        .clone_router()
        .oneshot(post_json(
            "/api/v1/signal-hub/runtimes",
            serde_json::json!({
                "source_code": "system",
                "runtime_kind": "signal_hub_raw_signal_dispatcher",
                "state": "paused",
                "metadata": {
                    "scope": "consumer"
                }
            }),
        ))
        .await
        .expect("runtime update request");
    assert_eq!(runtime_response.status(), StatusCode::OK);

    let runtimes_response = app
        .clone_router()
        .oneshot(get("/api/v1/signal-hub/runtimes"))
        .await
        .expect("runtimes request");
    assert_eq!(runtimes_response.status(), StatusCode::OK);
    let runtimes_body = to_bytes(runtimes_response.into_body(), usize::MAX)
        .await
        .expect("runtimes body");
    let runtimes_json: Value = serde_json::from_slice(&runtimes_body).expect("runtimes json");
    let runtime = runtimes_json["items"]
        .as_array()
        .expect("runtime items")
        .iter()
        .find(|item| item["runtime_kind"] == "signal_hub_raw_signal_dispatcher")
        .expect("signal hub runtime state");
    assert_eq!(runtime["source_code"], "system");
    assert_eq!(runtime["state"], "paused");

    let replay_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO signal_replay_requests (
            id,
            source_code,
            connection_id,
            event_pattern,
            status,
            requested_by,
            replayed_count,
            metadata
        )
        VALUES ($1, 'telegram', $2, 'signal.raw.telegram.*', 'queued', 'makosh-frontend', 3, '{"trigger":"api"}'::jsonb)
        "#,
    )
    .bind(replay_id)
    .bind(connection_id)
    .execute(pool)
    .await
    .expect("insert replay request");

    let replay_response = app
        .clone_router()
        .oneshot(get("/api/v1/signal-hub/replay"))
        .await
        .expect("replay request");
    assert_eq!(replay_response.status(), StatusCode::OK);
    let replay_body = to_bytes(replay_response.into_body(), usize::MAX)
        .await
        .expect("replay body");
    let replay_json: Value = serde_json::from_slice(&replay_body).expect("replay json");
    let replay = replay_json["items"]
        .as_array()
        .expect("replay items")
        .iter()
        .find(|item| item["id"] == replay_id.to_string())
        .expect("telegram replay request");
    assert_eq!(replay["status"], "queued");
    assert_eq!(replay["event_pattern"], "signal.raw.telegram.*");
    assert_eq!(replay["replayed_count"], 3);
    assert_eq!(replay["from_position"], Value::Null);

    let create_replay_response = app
        .clone_router()
        .oneshot(post_json(
            "/api/v1/signal-hub/replay",
            serde_json::json!({
                "source_code": "telegram",
                "event_pattern": "signal.raw.telegram.*",
                "from_position": 10,
                "to_position": 20,
                "target_projection": "communication_messages",
                "metadata": {
                    "requested_from": "rest_test"
                }
            }),
        ))
        .await
        .expect("replay create request");
    assert_eq!(create_replay_response.status(), StatusCode::OK);
    let create_replay_body = to_bytes(create_replay_response.into_body(), usize::MAX)
        .await
        .expect("replay create body");
    let create_replay_json: Value =
        serde_json::from_slice(&create_replay_body).expect("replay create json");
    assert_eq!(create_replay_json["source_code"], "telegram");
    assert_eq!(create_replay_json["status"], "queued");
    assert_eq!(create_replay_json["from_position"], 10);
    assert_eq!(create_replay_json["to_position"], 20);
    assert_eq!(
        create_replay_json["target_projection"],
        "communication_messages"
    );
}

#[tokio::test]
async fn signal_hub_api_can_create_update_and_remove_connections() {
    let app = test_app().await;
    let router = app.clone_router();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/system/restore",
            serde_json::json!({}),
        ))
        .await
        .expect("restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let create_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/connections",
            serde_json::json!({
                "source_code": "telegram",
                "display_name": "Primary Telegram",
                "status": "connected",
                "profile": "default",
                "settings": {}
            }),
        ))
        .await
        .expect("create connection request");
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("create body");
    let create_json: Value = serde_json::from_slice(&create_body).expect("create json");
    let connection_id = create_json["item"]["id"]
        .as_str()
        .expect("created connection id")
        .to_owned();
    assert_eq!(create_json["item"]["status"], "connected");

    let update_response = router
        .clone()
        .oneshot(patch_json(
            &format!("/api/v1/signal-hub/connections/{connection_id}"),
            serde_json::json!({
                "status": "paused",
                "profile": "maintenance"
            }),
        ))
        .await
        .expect("update connection request");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_body = to_bytes(update_response.into_body(), usize::MAX)
        .await
        .expect("update body");
    let update_json: Value = serde_json::from_slice(&update_body).expect("update json");
    assert_eq!(update_json["item"]["status"], "paused");
    assert_eq!(update_json["item"]["profile"], "maintenance");

    let policies_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/policies"))
        .await
        .expect("list policies request");
    assert_eq!(policies_response.status(), StatusCode::OK);
    let policies_body = to_bytes(policies_response.into_body(), usize::MAX)
        .await
        .expect("policies body");
    let policies_json: Value = serde_json::from_slice(&policies_body).expect("policies json");
    let paused_policy = policies_json["items"]
        .as_array()
        .expect("policy items")
        .iter()
        .find(|item| item["connection_id"] == connection_id && item["mode"] == "paused")
        .expect("paused operator policy for connection");
    assert_eq!(paused_policy["scope"], "connection");
    assert_eq!(paused_policy["source_code"], "telegram");

    let remove_response = router
        .clone()
        .oneshot(delete(&format!(
            "/api/v1/signal-hub/connections/{connection_id}"
        )))
        .await
        .expect("remove connection request");
    assert_eq!(remove_response.status(), StatusCode::OK);
    let remove_body = to_bytes(remove_response.into_body(), usize::MAX)
        .await
        .expect("remove body");
    let remove_json: Value = serde_json::from_slice(&remove_body).expect("remove json");
    assert_eq!(remove_json["item"]["status"], "removed");

    let policies_after_remove_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/policies"))
        .await
        .expect("list policies after remove request");
    assert_eq!(policies_after_remove_response.status(), StatusCode::OK);
    let policies_after_remove_body =
        to_bytes(policies_after_remove_response.into_body(), usize::MAX)
            .await
            .expect("policies after remove body");
    let policies_after_remove_json: Value =
        serde_json::from_slice(&policies_after_remove_body).expect("policies after remove json");
    assert!(
        policies_after_remove_json["items"]
            .as_array()
            .expect("policy items after remove")
            .iter()
            .all(|item| item["connection_id"] != connection_id)
    );
}

#[tokio::test]
async fn signal_hub_connect_api_lists_sources_and_updates_runtime_state() {
    let app = test_app().await;
    let router = app.clone_router();
    let pool = app.context().pool();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RestoreSystemFixture",
            serde_json::json!({}),
        ))
        .await
        .expect("connect restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let list_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListSources",
            serde_json::json!({}),
        ))
        .await
        .expect("connect list sources request");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("connect sources body");
    let list_json: Value = serde_json::from_slice(&list_body).expect("connect sources json");
    let sources = list_json["items"].as_array().expect("connect source items");
    assert!(sources.iter().any(|item| item["code"] == "telegram"));
    assert!(sources.iter().any(|item| item["code"] == "mail"));

    let runtime_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UpdateRuntimeState",
            serde_json::json!({
                "sourceCode": "system",
                "runtimeKind": "signal_hub_raw_signal_dispatcher",
                "state": "paused",
                "metadataJson": "{\"scope\":\"consumer\"}"
            }),
        ))
        .await
        .expect("connect runtime update request");
    assert_eq!(runtime_response.status(), StatusCode::OK);
    let runtime_body = to_bytes(runtime_response.into_body(), usize::MAX)
        .await
        .expect("connect runtime body");
    let runtime_json: Value = serde_json::from_slice(&runtime_body).expect("connect runtime json");
    assert_eq!(runtime_json["item"]["sourceCode"], "system");
    assert_eq!(
        runtime_json["item"]["runtimeKind"],
        "signal_hub_raw_signal_dispatcher"
    );
    assert_eq!(runtime_json["item"]["state"], "paused");
    assert_eq!(
        runtime_json["item"]["metadataJson"],
        "{\"scope\":\"consumer\"}"
    );

    let create_policy_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/CreatePolicy",
            serde_json::json!({
                "scope": "event_pattern",
                "eventPattern": "signal.raw.*",
                "mode": "paused",
                "reason": "connect policy"
            }),
        ))
        .await
        .expect("connect create policy request");
    assert_eq!(create_policy_response.status(), StatusCode::OK);

    let list_policies_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListPolicies",
            serde_json::json!({}),
        ))
        .await
        .expect("connect list policies request");
    assert_eq!(list_policies_response.status(), StatusCode::OK);
    let list_policies_body = to_bytes(list_policies_response.into_body(), usize::MAX)
        .await
        .expect("connect policies body");
    let list_policies_json: Value =
        serde_json::from_slice(&list_policies_body).expect("connect policies json");
    let policy = list_policies_json["items"]
        .as_array()
        .expect("connect policy items")
        .iter()
        .find(|item| item["eventPattern"] == "signal.raw.*")
        .expect("connect-created pause policy");
    assert_eq!(policy["scope"], "event_pattern");
    assert_eq!(policy["mode"], "paused");

    let replay_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO signal_replay_requests (
            id,
            source_code,
            connection_id,
            event_pattern,
            status,
            requested_by,
            replayed_count,
            metadata
        )
        VALUES ($1, 'telegram', NULL, 'signal.raw.telegram.*', 'queued', 'makosh-frontend', 2, '{"trigger":"connect","from_position":11,"to_position":22}'::jsonb)
        "#,
    )
    .bind(replay_id)
    .execute(pool)
    .await
    .expect("insert connect replay request");

    let list_replay_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListReplayRequests",
            serde_json::json!({}),
        ))
        .await
        .expect("connect list replay request");
    assert_eq!(list_replay_response.status(), StatusCode::OK);
    let list_replay_body = to_bytes(list_replay_response.into_body(), usize::MAX)
        .await
        .expect("connect replay body");
    let list_replay_json: Value =
        serde_json::from_slice(&list_replay_body).expect("connect replay json");
    let replay = list_replay_json["items"]
        .as_array()
        .expect("connect replay items")
        .iter()
        .find(|item| item["id"] == replay_id.to_string())
        .expect("connect replay request");
    assert_eq!(replay["status"], "queued");
    assert_eq!(replay["eventPattern"], "signal.raw.telegram.*");
    assert_eq!(replay["fromPosition"], "11");
    assert_eq!(replay["toPosition"], "22");
    assert_eq!(replay["replayedCount"], 2);
    assert_eq!(
        replay["metadataJson"],
        "{\"from_position\":11,\"to_position\":22,\"trigger\":\"connect\"}"
    );

    let request_replay_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RequestReplay",
            serde_json::json!({
                "sourceCode": "telegram",
                "eventPattern": "signal.raw.telegram.*",
                "fromPosition": 30,
                "toPosition": 40,
                "targetProjection": "communication_messages",
                "metadataJson": "{\"requested_from\":\"connect\"}"
            }),
        ))
        .await
        .expect("connect request replay request");
    assert_eq!(request_replay_response.status(), StatusCode::OK);
    let request_replay_body = to_bytes(request_replay_response.into_body(), usize::MAX)
        .await
        .expect("connect request replay body");
    let request_replay_json: Value =
        serde_json::from_slice(&request_replay_body).expect("connect request replay json");
    assert_eq!(request_replay_json["item"]["sourceCode"], "telegram");
    assert_eq!(request_replay_json["item"]["status"], "queued");
    assert_eq!(request_replay_json["item"]["fromPosition"], "30");
    assert_eq!(request_replay_json["item"]["toPosition"], "40");
    assert_eq!(
        request_replay_json["item"]["targetProjection"],
        "communication_messages"
    );

    let request_persona_projection_replay_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RequestReplay",
            serde_json::json!({
                "eventPattern": "persona.role.assigned",
                "fromPosition": 50,
                "toPosition": 50,
                "targetProjection": "persona_derived_evidence",
                "metadataJson": "{\"requested_from\":\"connect\"}"
            }),
        ))
        .await
        .expect("connect request persona projection replay request");
    assert_eq!(
        request_persona_projection_replay_response.status(),
        StatusCode::OK
    );
    let request_persona_projection_replay_body = to_bytes(
        request_persona_projection_replay_response.into_body(),
        usize::MAX,
    )
    .await
    .expect("connect request persona projection replay body");
    let request_persona_projection_replay_json: Value =
        serde_json::from_slice(&request_persona_projection_replay_body)
            .expect("connect request persona projection replay json");
    assert_eq!(
        request_persona_projection_replay_json["item"]["targetProjection"],
        "persona_derived_evidence"
    );

    let run_health_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RunHealthCheck",
            serde_json::json!({
                "sourceCode": "system"
            }),
        ))
        .await
        .expect("connect run health request");
    assert_eq!(run_health_response.status(), StatusCode::OK);
    let run_health_body = to_bytes(run_health_response.into_body(), usize::MAX)
        .await
        .expect("connect run health body");
    let run_health_json: Value =
        serde_json::from_slice(&run_health_body).expect("connect run health json");
    assert_eq!(run_health_json["item"]["sourceCode"], "system");
    assert!(run_health_json["item"]["level"].is_string());

    let list_profiles_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListProfiles",
            serde_json::json!({}),
        ))
        .await
        .expect("connect list profiles request");
    assert_eq!(list_profiles_response.status(), StatusCode::OK);
    let list_profiles_body = to_bytes(list_profiles_response.into_body(), usize::MAX)
        .await
        .expect("connect profiles body");
    let list_profiles_json: Value =
        serde_json::from_slice(&list_profiles_body).expect("connect profiles json");
    assert!(
        list_profiles_json["items"]
            .as_array()
            .expect("connect profile items")
            .iter()
            .any(|item| item["code"] == "production")
    );

    let apply_profile_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ApplyProfile",
            serde_json::json!({
                "code": "testing"
            }),
        ))
        .await
        .expect("connect apply profile request");
    assert_eq!(apply_profile_response.status(), StatusCode::OK);
    let apply_profile_body = to_bytes(apply_profile_response.into_body(), usize::MAX)
        .await
        .expect("connect apply profile body");
    let apply_profile_json: Value =
        serde_json::from_slice(&apply_profile_body).expect("connect apply profile json");
    assert_eq!(apply_profile_json["item"]["code"], "testing");
    assert_eq!(apply_profile_json["item"]["isActive"], true);

    let list_capabilities_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListCapabilities",
            serde_json::json!({
                "sourceCode": "telegram"
            }),
        ))
        .await
        .expect("connect list capabilities request");
    assert_eq!(list_capabilities_response.status(), StatusCode::OK);
    let list_capabilities_body = to_bytes(list_capabilities_response.into_body(), usize::MAX)
        .await
        .expect("connect capabilities body");
    let list_capabilities_json: Value =
        serde_json::from_slice(&list_capabilities_body).expect("connect capabilities json");
    assert!(
        list_capabilities_json["items"]
            .as_array()
            .expect("connect capability items")
            .iter()
            .any(|item| item["capability"] == "runtime.replay")
    );
    assert!(
        list_capabilities_json["items"]
            .as_array()
            .expect("connect capability items")
            .iter()
            .all(|item| item["state"] == "degraded")
    );
    assert!(
        list_capabilities_json["items"]
            .as_array()
            .expect("connect capability items")
            .iter()
            .any(|item| {
                item["capability"] == "runtime.replay"
                    && item["reason"]
                        .as_str()
                        .is_some_and(|reason| reason.contains("muted"))
            })
    );

    let create_profile_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/CreateProfile",
            serde_json::json!({
                "code": "quiet_hours",
                "displayName": "Quiet Hours",
                "description": "Mute noisy overnight signals",
                "sourcePolicies": [
                    {
                        "scope": "source",
                        "sourceCode": "telegram",
                        "mode": "muted",
                        "reason": "night mute"
                    }
                ]
            }),
        ))
        .await
        .expect("connect create profile request");
    assert_eq!(create_profile_response.status(), StatusCode::OK);
    let create_profile_body = to_bytes(create_profile_response.into_body(), usize::MAX)
        .await
        .expect("connect create profile body");
    let create_profile_json: Value =
        serde_json::from_slice(&create_profile_body).expect("connect create profile json");
    assert_eq!(create_profile_json["item"]["code"], "quiet_hours");
    assert_eq!(
        create_profile_json["item"]["sourcePolicies"][0]["sourceCode"],
        "telegram"
    );

    let update_profile_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UpdateProfile",
            serde_json::json!({
                "code": "quiet_hours",
                "description": "Updated quiet profile",
                "updateSourcePolicies": true,
                "sourcePolicies": [
                    {
                        "scope": "event_pattern",
                        "eventPattern": "signal.raw.mail.*",
                        "mode": "paused",
                        "reason": "overnight mail pause"
                    }
                ]
            }),
        ))
        .await
        .expect("connect update profile request");
    assert_eq!(update_profile_response.status(), StatusCode::OK);
    let update_profile_body = to_bytes(update_profile_response.into_body(), usize::MAX)
        .await
        .expect("connect update profile body");
    let update_profile_json: Value =
        serde_json::from_slice(&update_profile_body).expect("connect update profile json");
    assert_eq!(
        update_profile_json["item"]["sourcePolicies"][0]["eventPattern"],
        "signal.raw.mail.*"
    );

    let remove_profile_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RemoveProfile",
            serde_json::json!({
                "code": "quiet_hours"
            }),
        ))
        .await
        .expect("connect remove profile request");
    assert_eq!(remove_profile_response.status(), StatusCode::OK);
    let remove_profile_body = to_bytes(remove_profile_response.into_body(), usize::MAX)
        .await
        .expect("connect remove profile body");
    let remove_profile_json: Value =
        serde_json::from_slice(&remove_profile_body).expect("connect remove profile json");
    assert_eq!(remove_profile_json["item"]["code"], "quiet_hours");

    let emit_fixture_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/EmitFixtureSignal",
            serde_json::json!({
                "fixtureId": "fixture_basic_message"
            }),
        ))
        .await
        .expect("connect emit fixture request");
    assert_eq!(emit_fixture_response.status(), StatusCode::OK);
    let emit_fixture_body = to_bytes(emit_fixture_response.into_body(), usize::MAX)
        .await
        .expect("connect emit fixture body");
    let emit_fixture_json: Value =
        serde_json::from_slice(&emit_fixture_body).expect("connect emit fixture json");
    assert_eq!(emit_fixture_json["fixtureId"], "fixture_basic_message");
    assert_eq!(emit_fixture_json["sourceCode"], "fixture");

    let list_fixture_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListFixtureSources",
            serde_json::json!({}),
        ))
        .await
        .expect("connect list fixture sources request");
    assert_eq!(list_fixture_response.status(), StatusCode::OK);
    let list_fixture_body = to_bytes(list_fixture_response.into_body(), usize::MAX)
        .await
        .expect("connect list fixture body");
    let list_fixture_json: Value =
        serde_json::from_slice(&list_fixture_body).expect("connect list fixture json");
    let fixture = list_fixture_json["items"]
        .as_array()
        .expect("connect fixture items")
        .iter()
        .find(|item| item["fixtureId"] == "fixture_basic_message")
        .expect("connect fixture basic message");
    assert_eq!(fixture["sourceCode"], "fixture");
    assert_eq!(fixture["summary"], "Fixture message");
}

#[tokio::test]
async fn signal_hub_connect_runtime_switch_takes_effect_without_restart() {
    let app = test_app().await;
    let router = app.clone_router();
    let pool = app.context().pool().clone();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RestoreSystemFixture",
            serde_json::json!({}),
        ))
        .await
        .expect("connect restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "telegram-connect-runtime-account",
            CommunicationProviderKind::TelegramUser,
            "Telegram Connect Runtime",
            "telegram-connect-runtime-account",
        ))
        .await
        .expect("provider account");

    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw_signal_hub_connect_runtime_telegram",
                "telegram-connect-runtime-account",
                "telegram_message",
                "telegram-connect-runtime-message-1",
                "sha256:signal-hub:connect-runtime:telegram",
                "signal-hub-connect-runtime",
                serde_json::json!({
                    "provider_chat_id": "telegram-connect-runtime-chat",
                    "chat_title": "Telegram Connect Runtime Chat",
                    "sender_id": "telegram-connect-runtime-sender",
                    "sender_display_name": "Connect Runtime Sender",
                    "text": "runtime switch should apply immediately",
                    "delivery_state": "received"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(serde_json::json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": "telegram-connect-runtime-account",
                "provider_chat_id": "telegram-connect-runtime-chat",
            })),
        )
        .await
        .expect("raw telegram record");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch raw telegram signal")
        .expect("accepted telegram signal");

    let pause_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UpdateRuntimeState",
            serde_json::json!({
                "sourceCode": "system",
                "runtimeKind": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                "state": "paused",
                "metadataJson": "{\"scope\":\"consumer\",\"requested_by\":\"connect_test\"}"
            }),
        ))
        .await
        .expect("connect pause runtime request");
    assert_eq!(pause_response.status(), StatusCode::OK);

    let blocked_projection =
        project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event)
            .await
            .expect("project accepted signal while paused");
    assert!(blocked_projection.is_none());

    let projected_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM communication_messages WHERE raw_record_id = $1")
            .bind(&raw_record.raw_record_id)
            .fetch_one(&pool)
            .await
            .expect("projected message count while paused");
    assert_eq!(projected_count, 0);

    let resume_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UpdateRuntimeState",
            serde_json::json!({
                "sourceCode": "system",
                "runtimeKind": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                "state": "running",
                "metadataJson": "{\"scope\":\"consumer\",\"requested_by\":\"connect_test\"}"
            }),
        ))
        .await
        .expect("connect resume runtime request");
    assert_eq!(resume_response.status(), StatusCode::OK);

    let resumed_projection =
        project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event)
            .await
            .expect("project accepted signal after resume");
    assert!(resumed_projection.is_some());

    let projected_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM communication_messages WHERE raw_record_id = $1")
            .bind(&raw_record.raw_record_id)
            .fetch_one(&pool)
            .await
            .expect("projected message count after resume");
    assert_eq!(projected_count, 1);
}

#[tokio::test]
async fn signal_hub_connect_raw_dispatcher_switch_takes_effect_without_restart() {
    let app = test_app().await;
    let router = app.clone_router();
    let pool = app.context().pool().clone();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/RestoreSystemFixture",
            serde_json::json!({}),
        ))
        .await
        .expect("connect restore request");
    assert_eq!(restore_response.status(), StatusCode::OK);

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "telegram-connect-raw-runtime-account",
            CommunicationProviderKind::TelegramUser,
            "Telegram Connect Raw Runtime",
            "telegram-connect-raw-runtime-account",
        ))
        .await
        .expect("provider account");

    let pause_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UpdateRuntimeState",
            serde_json::json!({
                "sourceCode": "system",
                "runtimeKind": "signal_hub_raw_signal_dispatcher",
                "state": "paused",
                "metadataJson": "{\"scope\":\"consumer\",\"requested_by\":\"connect_test\"}"
            }),
        ))
        .await
        .expect("connect pause raw dispatcher request");
    assert_eq!(pause_response.status(), StatusCode::OK);

    let paused_raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw_signal_hub_connect_paused_telegram",
                "telegram-connect-raw-runtime-account",
                "telegram_message",
                "telegram-connect-paused-message-1",
                "sha256:signal-hub:connect-paused:telegram",
                "signal-hub-connect-paused",
                serde_json::json!({
                    "provider_chat_id": "telegram-connect-paused-chat",
                    "chat_title": "Telegram Connect Paused Chat",
                    "sender_id": "telegram-connect-paused-sender",
                    "sender_display_name": "Connect Paused Sender",
                    "text": "raw dispatcher should pause immediately",
                    "delivery_state": "received"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(serde_json::json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": "telegram-connect-raw-runtime-account",
                "provider_chat_id": "telegram-connect-paused-chat",
            })),
        )
        .await
        .expect("paused raw telegram record");

    let paused_accepted = dispatch_telegram_raw_signal(pool.clone(), &paused_raw_record)
        .await
        .expect("dispatch paused raw telegram signal");
    assert!(paused_accepted.is_none());

    let accepted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.telegram.message' AND correlation_id = $1",
    )
    .bind(&paused_raw_record.observation_id)
    .fetch_one(&pool)
    .await
    .expect("accepted signal count while paused");
    assert_eq!(accepted_count, 0);

    let resume_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/UpdateRuntimeState",
            serde_json::json!({
                "sourceCode": "system",
                "runtimeKind": "signal_hub_raw_signal_dispatcher",
                "state": "running",
                "metadataJson": "{\"scope\":\"consumer\",\"requested_by\":\"connect_test\"}"
            }),
        ))
        .await
        .expect("connect resume raw dispatcher request");
    assert_eq!(resume_response.status(), StatusCode::OK);

    let resumed_raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw_signal_hub_connect_resumed_telegram",
                "telegram-connect-raw-runtime-account",
                "telegram_message",
                "telegram-connect-resumed-message-1",
                "sha256:signal-hub:connect-resumed:telegram",
                "signal-hub-connect-resumed",
                serde_json::json!({
                    "provider_chat_id": "telegram-connect-resumed-chat",
                    "chat_title": "Telegram Connect Resumed Chat",
                    "sender_id": "telegram-connect-resumed-sender",
                    "sender_display_name": "Connect Resumed Sender",
                    "text": "raw dispatcher should resume immediately",
                    "delivery_state": "received"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(serde_json::json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": "telegram-connect-raw-runtime-account",
                "provider_chat_id": "telegram-connect-resumed-chat",
            })),
        )
        .await
        .expect("resumed raw telegram record");

    let resumed_accepted = dispatch_telegram_raw_signal(pool.clone(), &resumed_raw_record)
        .await
        .expect("dispatch resumed raw telegram signal");
    assert!(resumed_accepted.is_some());

    let accepted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.telegram.message' AND correlation_id = $1",
    )
    .bind(&resumed_raw_record.observation_id)
    .fetch_one(&pool)
    .await
    .expect("accepted signal count after resume");
    assert_eq!(accepted_count, 1);
}
