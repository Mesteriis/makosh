use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::{PgPool, Row};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;

use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::integrations::zoom::client::{
    models::{ZoomAccountSetupRequest, ZoomMeetingObservationRequest},
    store::ZoomStore,
};
use makosh_hub_backend::platform::calls::store::CallIntelligenceStore;

use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use makosh_hub_backend::platform::events::bus::InMemoryEventBus;
use makosh_hub_backend::platform::events::bus::zoom_event_types;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::HostVault;
use makosh_hub_backend::vault::models::{
    EntropyEvent, HostVaultConfig, SecretEntryContext, VaultMode,
};

const LOCAL_API_TOKEN: &str = "zoom-provider-test-secret";
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_transcript_download_enabled";
const ZOOM_RECORDING_IMPORT_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.zoom_recording_import_retention_days";
const ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY: &str = "privacy.zoom_transcript_retention_days";
type HmacSha256 = Hmac<Sha256>;

#[test]
fn zoom_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("zoom_user").expect("zoom user provider"),
        CommunicationProviderKind::ZoomUser
    );
    assert_eq!(
        CommunicationProviderKind::try_from("zoom_server_to_server").expect("zoom s2s provider"),
        CommunicationProviderKind::ZoomServerToServer
    );
    assert!(CommunicationProviderKind::ZoomUser.is_zoom());
    assert!(CommunicationProviderKind::ZoomServerToServer.is_zoom());
    assert!(!CommunicationProviderKind::ZoomUser.is_email());
    assert!(!CommunicationProviderKind::ZoomUser.is_telegram());
    assert!(!CommunicationProviderKind::ZoomUser.is_whatsapp());

    assert!(
        ProviderAccountSecretPurpose::ZoomOauthToken.accepts_secret_kind(SecretKind::OauthToken)
    );
    assert!(
        ProviderAccountSecretPurpose::ZoomClientSecret.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::ZoomWebhookSecret.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        !ProviderAccountSecretPurpose::ZoomOauthToken.accepts_secret_kind(SecretKind::ApiToken)
    );
}

#[tokio::test]
async fn zoom_fixture_account_lifecycle_filters_removed_accounts() {
    let (_context, app, _pool) = test_app().await;
    let suffix = unique_suffix();
    let account_id = format!("zoom-fixture-{suffix}");

    let capabilities_response = app
        .clone()
        .oneshot(get("/api/v1/integrations/zoom/capabilities"))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    let capabilities = capabilities_body["capabilities"]
        .as_array()
        .expect("capabilities array");
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("token_maintenance.scheduler")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("provider_sync.recordings.scheduler")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("recording_imports.remove")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("retention.cleanup")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("retention.cleanup.scheduler")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("auth.token_rotation_policy")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("calendar_event_matching")
            && capability["status"] == json!("available")
    }));
    assert!(capabilities.iter().any(|capability| {
        capability["capability"] == json!("meeting_participant_identity_resolution")
            && capability["status"] == json!("available")
    }));
    assert!(
        !capabilities_body["planned_features"]
            .as_array()
            .expect("planned features")
            .iter()
            .any(|feature| {
                matches!(
                    feature.as_str(),
                    Some(
                        "zoom_token_rotation_policy"
                            | "calendar_event_matching"
                            | "meeting_participant_identity_resolution"
                    )
                )
            })
    );

    let account_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/fixtures/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Fixture",
                "external_account_id": format!("zoom-external-{suffix}"),
                "account_email": "fixture@example.test",
                "metadata": { "tenant": "fixture" }
            }),
        ))
        .await
        .expect("fixture account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(account_body["account"]["provider_kind"], json!("zoom_user"));
    assert_eq!(account_body["account"]["auth_shape"], json!("fixture"));

    let initial_status = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/runtime/status?account_id={account_id}"
        )))
        .await
        .expect("runtime status");
    assert_eq!(initial_status.status(), StatusCode::OK);
    let initial_body = json_body(initial_status).await;
    assert_eq!(initial_body["status"], json!("stopped"));
    assert_eq!(initial_body["healthy"], json!(true));

    let start_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime/start",
            json!({ "account_id": account_id }),
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    assert_eq!(json_body(start_response).await["status"], json!("running"));

    let stop_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime/stop",
            json!({ "account_id": account_id, "reason": "test" }),
        ))
        .await
        .expect("runtime stop response");
    assert_eq!(stop_response.status(), StatusCode::OK);
    assert_eq!(json_body(stop_response).await["status"], json!("stopped"));

    let remove_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime/remove",
            json!({ "account_id": account_id, "reason": "test cleanup" }),
        ))
        .await
        .expect("runtime remove response");
    assert_eq!(remove_response.status(), StatusCode::OK);
    assert_eq!(json_body(remove_response).await["removed"], json!(true));

    let active_accounts = app
        .clone()
        .oneshot(get("/api/v1/integrations/zoom/accounts"))
        .await
        .expect("active accounts response");
    assert_eq!(active_accounts.status(), StatusCode::OK);
    assert!(
        json_body(active_accounts).await["items"]
            .as_array()
            .expect("items")
            .is_empty()
    );

    let all_accounts = app
        .oneshot(get(
            "/api/v1/integrations/zoom/accounts?include_removed=true",
        ))
        .await
        .expect("all accounts response");
    assert_eq!(all_accounts.status(), StatusCode::OK);
    let all_body = json_body(all_accounts).await;
    assert_eq!(all_body["items"][0]["account_id"], json!(account_id));
    assert_eq!(all_body["items"][0]["lifecycle_state"], json!("removed"));
}

#[tokio::test]
async fn shared_calls_route_filters_zoom_calls_by_provider_query() {
    let (_context, app, _pool) = test_app().await;
    let suffix = unique_suffix();
    let account_id = format!("zoom-calls-filter-{suffix}");

    let account_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/fixtures/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Calls Filter",
                "external_account_id": format!("zoom-calls-filter-external-{suffix}")
            }),
        ))
        .await
        .expect("fixture account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let zoom_meeting_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/meetings",
            json!({
                "account_id": account_id,
                "meeting_id": "987650001",
                "topic": "Shared calls filter",
                "metadata": { "source": "zoom_test" }
            }),
        ))
        .await
        .expect("zoom meeting response");
    assert_eq!(zoom_meeting_response.status(), StatusCode::OK);

    let generic_call_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/calls",
            json!({
                "call_id": format!("generic-call-{suffix}"),
                "account_id": account_id,
                "provider_call_id": format!("telegram-call-{suffix}"),
                "provider_chat_id": format!("telegram:call:{suffix}"),
                "direction": "incoming",
                "call_state": "ended",
                "metadata": {
                    "provider": "telegram",
                    "source": "test_generic_call"
                }
            }),
        ))
        .await
        .expect("generic call response");
    assert_eq!(generic_call_response.status(), StatusCode::OK);

    let filtered_response = app
        .oneshot(get(&format!(
            "/api/v1/calls?account_id={account_id}&provider=zoom&limit=10"
        )))
        .await
        .expect("filtered calls response");
    assert_eq!(filtered_response.status(), StatusCode::OK);
    let filtered_body = json_body(filtered_response).await;
    let items = filtered_body["items"].as_array().expect("calls items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["provider_call_id"], json!("987650001"));
    assert_eq!(items[0]["metadata"]["provider"], json!("zoom"));
}

#[tokio::test]
async fn zoom_live_account_registration_is_blocked_and_uses_secret_bindings() {
    let (_context, app, pool) = test_app().await;
    let suffix = unique_suffix();
    let oauth_account_id = format!("zoom-oauth-{suffix}");
    let s2s_account_id = format!("zoom-s2s-{suffix}");
    seed_secret_ref(
        &pool,
        &format!("secret:zoom-oauth-token-{suffix}"),
        SecretKind::OauthToken,
    )
    .await;
    seed_secret_ref(
        &pool,
        &format!("secret:zoom-client-secret-{suffix}"),
        SecretKind::ApiToken,
    )
    .await;
    seed_secret_ref(
        &pool,
        &format!("secret:zoom-webhook-secret-{suffix}"),
        SecretKind::ApiToken,
    )
    .await;

    let oauth_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/accounts",
            json!({
                "account_id": oauth_account_id,
                "display_name": "Zoom OAuth",
                "external_account_id": format!("zoom-oauth-external-{suffix}"),
                "auth_shape": "oauth_user",
                "client_id": "zoom-client-id",
                "token_secret_ref": format!("secret:zoom-oauth-token-{suffix}"),
                "client_secret_ref": format!("secret:zoom-client-secret-{suffix}"),
                "webhook_secret_ref": format!("secret:zoom-webhook-secret-{suffix}"),
                "metadata": { "workspace": "oauth" }
            }),
        ))
        .await
        .expect("oauth account response");
    assert_eq!(oauth_response.status(), StatusCode::OK);
    let oauth_body = json_body(oauth_response).await;
    assert_eq!(oauth_body["account"]["provider_kind"], json!("zoom_user"));
    assert_eq!(oauth_body["account"]["lifecycle_state"], json!("blocked"));
    assert_eq!(
        oauth_body["account"]["config"]["credential_refs_bound"],
        json!({
            "zoom_oauth_token": true,
            "zoom_client_secret": true,
            "zoom_webhook_secret": true,
        })
    );
    assert!(
        !oauth_body["account"]["config"]
            .to_string()
            .contains("secret:zoom-oauth-token"),
        "account config must not duplicate secret reference ids"
    );

    let bindings = provider_secret_bindings(&pool, &oauth_account_id).await;
    assert_eq!(bindings.len(), 3);
    assert!(
        bindings
            .iter()
            .any(|(purpose, _)| purpose == "zoom_oauth_token")
    );
    assert!(
        bindings
            .iter()
            .any(|(purpose, _)| purpose == "zoom_client_secret")
    );
    assert!(
        bindings
            .iter()
            .any(|(purpose, _)| purpose == "zoom_webhook_secret")
    );

    let s2s_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/accounts",
            json!({
                "account_id": s2s_account_id,
                "display_name": "Zoom Server-to-Server",
                "external_account_id": format!("zoom-s2s-external-{suffix}"),
                "auth_shape": "server_to_server",
                "client_id": "zoom-s2s-client-id",
                "client_secret_ref": format!("secret:zoom-client-secret-{suffix}"),
                "metadata": { "workspace": "s2s" }
            }),
        ))
        .await
        .expect("s2s account response");
    assert_eq!(s2s_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(s2s_response).await["account"]["provider_kind"],
        json!("zoom_server_to_server")
    );

    let live_start_response = app
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime/start",
            json!({ "account_id": oauth_account_id }),
        ))
        .await
        .expect("live start response");
    assert_eq!(live_start_response.status(), StatusCode::OK);
    let live_start_body = json_body(live_start_response).await;
    assert_eq!(live_start_body["status"], json!("blocked"));
    assert_eq!(live_start_body["healthy"], json!(false));
    assert_eq!(
        live_start_body["runtime_blockers"],
        json!(["zoom_live_authorization_required"])
    );
}

#[tokio::test]
async fn zoom_oauth_authorization_exchanges_code_and_stores_tokens_in_host_vault() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-oauth-live-{suffix}");
    let (token_endpoint, token_request) = spawn_zoom_token_server(json!({
        "access_token": "zoom-oauth-access-token",
        "refresh_token": "zoom-oauth-refresh-token",
        "expires_in": 3600,
        "token_type": "bearer",
        "scope": "meeting:read recording:read"
    }))
    .await;

    let start_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Zoom OAuth Live",
                "external_account_id": format!("zoom-oauth-live-external-{suffix}"),
                "client_id": "zoom-oauth-client",
                "client_secret": "zoom-oauth-client-secret",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/zoom/oauth/callback",
                "authorization_endpoint": "https://zoom.example.test/oauth/authorize",
                "token_endpoint": token_endpoint,
                "scopes": ["meeting:read", "recording:read"],
                "metadata": { "workspace": "oauth-live" }
            }),
        ))
        .await
        .expect("Zoom OAuth start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert!(
        start_body["authorization_url"]
            .as_str()
            .expect("authorization_url")
            .starts_with("https://zoom.example.test/oauth/authorize?")
    );

    let complete_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": start_body["setup_id"],
                "state": start_body["state"],
                "authorization_code": "zoom-auth-code"
            }),
        ))
        .await
        .expect("Zoom OAuth complete response");
    assert_eq!(complete_response.status(), StatusCode::OK);
    let complete_body = json_body(complete_response).await;
    assert_eq!(complete_body["account_id"], json!(account_id));
    assert_eq!(complete_body["auth_shape"], json!("oauth_user"));
    assert_eq!(complete_body["lifecycle_state"], json!("authorized"));
    assert_eq!(complete_body["store_kind"], json!("host_vault"));
    assert_eq!(complete_body["secret_kind"], json!("oauth_token"));
    let token_secret_ref = complete_body["token_secret_ref"]
        .as_str()
        .expect("token secret ref");
    let client_secret_ref = complete_body["client_secret_ref"]
        .as_str()
        .expect("client secret ref");

    let captured_request = token_request.await.expect("token request task");
    assert!(captured_request.contains("POST /oauth/token HTTP/1.1"));
    assert!(captured_request.contains("grant_type=authorization_code"));
    assert!(captured_request.contains("code=zoom-auth-code"));
    assert!(captured_request.contains("redirect_uri=http%3A%2F%2F127.0.0.1"));
    assert!(
        captured_request
            .to_ascii_lowercase()
            .contains("authorization: basic")
    );

    let account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("authorized Zoom account config");
    assert_eq!(account_config["lifecycle_state"], json!("authorized"));
    assert_eq!(account_config["runtime_blockers"], json!([]));
    assert_eq!(
        account_config["authorization"]["token_secret_bound"],
        json!(true)
    );
    let account_config_text = account_config.to_string();
    assert!(!account_config_text.contains("zoom-oauth-client-secret"));
    assert!(!account_config_text.contains("zoom-oauth-access-token"));
    assert!(!account_config_text.contains("zoom-oauth-refresh-token"));

    let references = secret_references_for(&pool, &[client_secret_ref, token_secret_ref]).await;
    assert_eq!(
        references,
        vec![
            (
                client_secret_ref.to_owned(),
                "api_token".to_owned(),
                "host_vault".to_owned()
            ),
            (
                token_secret_ref.to_owned(),
                "oauth_token".to_owned(),
                "host_vault".to_owned()
            )
        ]
    );
    assert_provider_secret_binding(&pool, &account_id, "zoom_oauth_token", token_secret_ref).await;
    assert_provider_secret_binding(&pool, &account_id, "zoom_client_secret", client_secret_ref)
        .await;

    let vault = unlocked_test_vault(&context);
    let token_bundle = vault
        .read_secret(token_secret_ref)
        .expect("read Zoom token bundle");
    assert!(token_bundle.contains("zoom-oauth-access-token"));
    assert!(token_bundle.contains("zoom-oauth-refresh-token"));
    assert_eq!(
        vault
            .read_secret(client_secret_ref)
            .expect("read Zoom client secret"),
        "zoom-oauth-client-secret"
    );

    let start_runtime_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime/start",
            json!({ "account_id": account_id }),
        ))
        .await
        .expect("start authorized Zoom runtime response");
    assert_eq!(start_runtime_response.status(), StatusCode::OK);
    let runtime_body = json_body(start_runtime_response).await;
    assert_eq!(runtime_body["status"], json!("running"));
    assert_eq!(runtime_body["healthy"], json!(true));
    assert_eq!(runtime_body["live_runtime_available"], json!(true));
    assert_eq!(runtime_body["runtime_blockers"], json!([]));

    let audit_items = zoom_audit_events(app.clone(), &account_id, 10).await;
    let authorization_event =
        find_zoom_audit_event(&audit_items, zoom_event_types::AUTHORIZATION_COMPLETED)
            .expect("oauth authorization audit event");
    assert_eq!(authorization_event["subject_kind"], json!("zoom_account"));
    assert_eq!(authorization_event["subject_entity_id"], json!(account_id));
    assert_eq!(
        authorization_event["payload"]["auth_shape"],
        json!("oauth_user")
    );
    assert_eq!(
        authorization_event["provenance"]["action"],
        json!("oauth_complete")
    );
    assert_secret_like_payload_was_stripped(&authorization_event["payload"]);
}

#[tokio::test]
async fn zoom_server_to_server_authorization_exchanges_account_credentials() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-s2s-live-{suffix}");
    let zoom_account_id = format!("zoom-provider-account-{suffix}");
    let (token_endpoint, token_request) = spawn_zoom_token_server(json!({
        "access_token": "zoom-s2s-access-token",
        "expires_in": 1800,
        "token_type": "bearer",
        "scope": "recording:read"
    }))
    .await;

    let account_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zoom S2S Live",
                "external_account_id": zoom_account_id,
                "auth_shape": "server_to_server",
                "client_id": "zoom-s2s-client",
                "metadata": { "workspace": "s2s-live" }
            }),
        ))
        .await
        .expect("Zoom S2S account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let authorize_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/server-to-server/authorize",
            json!({
                "account_id": account_id,
                "client_id": "zoom-s2s-client",
                "client_secret": "zoom-s2s-client-secret",
                "token_endpoint": token_endpoint,
                "metadata": { "safe": "kept" }
            }),
        ))
        .await
        .expect("Zoom S2S authorize response");
    assert_eq!(authorize_response.status(), StatusCode::OK);
    let authorize_body = json_body(authorize_response).await;
    assert_eq!(authorize_body["auth_shape"], json!("server_to_server"));
    assert_eq!(authorize_body["lifecycle_state"], json!("authorized"));
    let token_secret_ref = authorize_body["token_secret_ref"]
        .as_str()
        .expect("token secret ref");

    let captured_request = token_request.await.expect("S2S token request task");
    assert!(captured_request.contains("POST /oauth/token?grant_type=account_credentials"));
    assert!(captured_request.contains(&format!("account_id={zoom_account_id}")));
    assert!(
        captured_request
            .to_ascii_lowercase()
            .contains("authorization: basic")
    );

    let account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("S2S authorized account config");
    assert_eq!(account_config["lifecycle_state"], json!("authorized"));
    assert_eq!(
        account_config["authorization"]["metadata"]["server_to_server"]["zoom_account_id"],
        json!(zoom_account_id)
    );
    let account_config_text = account_config.to_string();
    assert!(!account_config_text.contains("zoom-s2s-client-secret"));
    assert!(!account_config_text.contains("zoom-s2s-access-token"));

    assert_provider_secret_binding(&pool, &account_id, "zoom_oauth_token", token_secret_ref).await;
    let vault = unlocked_test_vault(&context);
    let token_bundle = vault
        .read_secret(token_secret_ref)
        .expect("read Zoom S2S token bundle");
    assert!(token_bundle.contains("zoom-s2s-access-token"));
    assert!(!token_bundle.contains("refresh_token"));

    let audit_items = zoom_audit_events(app.clone(), &account_id, 10).await;
    let authorization_event =
        find_zoom_audit_event(&audit_items, zoom_event_types::AUTHORIZATION_COMPLETED)
            .expect("s2s authorization audit event");
    assert_eq!(authorization_event["subject_kind"], json!("zoom_account"));
    assert_eq!(authorization_event["subject_entity_id"], json!(account_id));
    assert_eq!(
        authorization_event["payload"]["auth_shape"],
        json!("server_to_server")
    );
    assert_eq!(
        authorization_event["provenance"]["action"],
        json!("server_to_server_authorize")
    );
    assert_secret_like_payload_was_stripped(&authorization_event["payload"]);
}

#[tokio::test]
async fn zoom_token_refresh_renews_oauth_and_server_to_server_tokens_in_host_vault() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();

    let oauth_account_id = format!("zoom-oauth-refresh-{suffix}");
    let (oauth_token_endpoint, oauth_token_requests) = spawn_zoom_token_server_sequence(vec![
        json!({
            "access_token": "zoom-oauth-initial-access-token",
            "refresh_token": "zoom-oauth-initial-refresh-token",
            "expires_in": 1,
            "token_type": "bearer",
            "scope": "meeting:read"
        }),
        json!({
            "access_token": "zoom-oauth-refreshed-access-token",
            "refresh_token": "zoom-oauth-refreshed-refresh-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "meeting:read recording:read"
        }),
    ])
    .await;
    let oauth_start = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": oauth_account_id,
                "display_name": "Zoom OAuth Refresh",
                "external_account_id": format!("zoom-oauth-refresh-external-{suffix}"),
                "client_id": "zoom-oauth-refresh-client",
                "client_secret": "zoom-oauth-refresh-client-secret",
                "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback",
                "token_endpoint": oauth_token_endpoint,
                "scopes": ["meeting:read"]
            }),
        ))
        .await
        .expect("Zoom OAuth refresh start response");
    assert_eq!(oauth_start.status(), StatusCode::OK);
    let oauth_start_body = json_body(oauth_start).await;
    let oauth_complete = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": oauth_start_body["setup_id"],
                "state": oauth_start_body["state"],
                "authorization_code": "zoom-oauth-refresh-code"
            }),
        ))
        .await
        .expect("Zoom OAuth refresh complete response");
    assert_eq!(oauth_complete.status(), StatusCode::OK);

    let oauth_refresh = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/refresh",
            json!({
                "account_id": oauth_account_id,
                "force": true
            }),
        ))
        .await
        .expect("Zoom OAuth refresh response");
    assert_eq!(oauth_refresh.status(), StatusCode::OK);
    let oauth_refresh_body = json_body(oauth_refresh).await;
    assert_eq!(oauth_refresh_body["refreshed"], json!(true));
    assert_eq!(
        oauth_refresh_body["refresh_strategy"],
        json!("oauth_refresh_token")
    );
    let oauth_token_secret_ref = oauth_refresh_body["token_secret_ref"]
        .as_str()
        .expect("OAuth token secret ref");

    let oauth_requests = oauth_token_requests
        .await
        .expect("OAuth token request task");
    assert_eq!(oauth_requests.len(), 2);
    assert!(oauth_requests[1].contains("grant_type=refresh_token"));
    assert!(oauth_requests[1].contains("refresh_token=zoom-oauth-initial-refresh-token"));
    assert!(
        oauth_requests[1]
            .to_ascii_lowercase()
            .contains("authorization: basic")
    );

    let vault = unlocked_test_vault(&context);
    let oauth_token_bundle = vault
        .read_secret(oauth_token_secret_ref)
        .expect("read refreshed OAuth token bundle");
    assert!(oauth_token_bundle.contains("zoom-oauth-refreshed-access-token"));
    assert!(oauth_token_bundle.contains("zoom-oauth-refreshed-refresh-token"));

    let oauth_account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&oauth_account_id)
    .fetch_one(&pool)
    .await
    .expect("OAuth refreshed account config");
    assert_eq!(
        oauth_account_config["authorization"]["last_token_refresh"]["status"],
        json!("refreshed")
    );
    let oauth_account_config_text = oauth_account_config.to_string();
    assert!(!oauth_account_config_text.contains("zoom-oauth-refresh-client-secret"));
    assert!(!oauth_account_config_text.contains("zoom-oauth-refreshed-access-token"));
    assert!(!oauth_account_config_text.contains("zoom-oauth-refreshed-refresh-token"));

    let oauth_audit_items = zoom_audit_events(app.clone(), &oauth_account_id, 10).await;
    let oauth_refresh_event =
        find_zoom_audit_event(&oauth_audit_items, zoom_event_types::TOKEN_REFRESHED)
            .expect("oauth refresh audit event");
    assert_eq!(oauth_refresh_event["payload"]["status"], json!("refreshed"));
    assert_eq!(oauth_refresh_event["payload"]["refreshed"], json!(true));
    assert_eq!(
        oauth_refresh_event["provenance"]["action"],
        json!("explicit_refresh")
    );
    assert_secret_like_payload_was_stripped(&oauth_refresh_event["payload"]);

    let s2s_account_id = format!("zoom-s2s-refresh-{suffix}");
    let zoom_account_id = format!("zoom-s2s-refresh-provider-{suffix}");
    let (s2s_token_endpoint, s2s_token_requests) = spawn_zoom_token_server_sequence(vec![
        json!({
            "access_token": "zoom-s2s-initial-access-token",
            "expires_in": 1,
            "token_type": "bearer",
            "scope": "recording:read"
        }),
        json!({
            "access_token": "zoom-s2s-renewed-access-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "recording:read"
        }),
    ])
    .await;
    let s2s_account = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/accounts",
            json!({
                "account_id": s2s_account_id,
                "display_name": "Zoom S2S Refresh",
                "external_account_id": zoom_account_id,
                "auth_shape": "server_to_server",
                "client_id": "zoom-s2s-refresh-client"
            }),
        ))
        .await
        .expect("Zoom S2S refresh account response");
    assert_eq!(s2s_account.status(), StatusCode::OK);
    let s2s_authorize = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/server-to-server/authorize",
            json!({
                "account_id": s2s_account_id,
                "client_id": "zoom-s2s-refresh-client",
                "client_secret": "zoom-s2s-refresh-client-secret",
                "token_endpoint": s2s_token_endpoint
            }),
        ))
        .await
        .expect("Zoom S2S refresh authorize response");
    assert_eq!(s2s_authorize.status(), StatusCode::OK);
    let s2s_refresh = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/refresh",
            json!({
                "account_id": s2s_account_id,
                "force": true
            }),
        ))
        .await
        .expect("Zoom S2S refresh response");
    assert_eq!(s2s_refresh.status(), StatusCode::OK);
    let s2s_refresh_body = json_body(s2s_refresh).await;
    assert_eq!(s2s_refresh_body["refreshed"], json!(true));
    assert_eq!(
        s2s_refresh_body["refresh_strategy"],
        json!("server_to_server_account_credentials")
    );
    let s2s_token_secret_ref = s2s_refresh_body["token_secret_ref"]
        .as_str()
        .expect("S2S token secret ref");
    let s2s_requests = s2s_token_requests.await.expect("S2S token request task");
    assert_eq!(s2s_requests.len(), 2);
    assert!(s2s_requests[1].contains("grant_type=account_credentials"));
    assert!(s2s_requests[1].contains(&format!("account_id={zoom_account_id}")));

    let s2s_token_bundle = vault
        .read_secret(s2s_token_secret_ref)
        .expect("read renewed S2S token bundle");
    assert!(s2s_token_bundle.contains("zoom-s2s-renewed-access-token"));
    assert!(!s2s_token_bundle.contains("refresh_token"));

    let s2s_account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&s2s_account_id)
    .fetch_one(&pool)
    .await
    .expect("S2S refreshed account config");
    assert_eq!(
        s2s_account_config["authorization"]["last_token_refresh"]["status"],
        json!("refreshed")
    );
    let s2s_account_config_text = s2s_account_config.to_string();
    assert!(!s2s_account_config_text.contains("zoom-s2s-refresh-client-secret"));
    assert!(!s2s_account_config_text.contains("zoom-s2s-renewed-access-token"));

    let s2s_audit_items = zoom_audit_events(app.clone(), &s2s_account_id, 10).await;
    let s2s_refresh_event =
        find_zoom_audit_event(&s2s_audit_items, zoom_event_types::TOKEN_REFRESHED)
            .expect("s2s refresh audit event");
    assert_eq!(s2s_refresh_event["payload"]["status"], json!("refreshed"));
    assert_eq!(s2s_refresh_event["payload"]["refreshed"], json!(true));
    assert_eq!(
        s2s_refresh_event["provenance"]["action"],
        json!("explicit_refresh")
    );
    assert_secret_like_payload_was_stripped(&s2s_refresh_event["payload"]);
}

#[tokio::test]
async fn zoom_token_maintenance_refreshes_expiring_authorized_accounts_only() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-oauth-maintenance-{suffix}");
    let (token_endpoint, token_requests) = spawn_zoom_token_server_sequence(vec![
        json!({
            "access_token": "zoom-maintenance-initial-access-token",
            "refresh_token": "zoom-maintenance-refresh-token",
            "expires_in": 1,
            "token_type": "bearer",
            "scope": "meeting:read"
        }),
        json!({
            "access_token": "zoom-maintenance-refreshed-access-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "meeting:read"
        }),
    ])
    .await;

    let start = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Zoom OAuth Maintenance",
                "external_account_id": format!("zoom-maintenance-external-{suffix}"),
                "client_id": "zoom-maintenance-client",
                "client_secret": "zoom-maintenance-client-secret",
                "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback",
                "token_endpoint": token_endpoint,
                "scopes": ["meeting:read"]
            }),
        ))
        .await
        .expect("Zoom OAuth maintenance start response");
    assert_eq!(start.status(), StatusCode::OK);
    let start_body = json_body(start).await;
    let complete = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": start_body["setup_id"],
                "state": start_body["state"],
                "authorization_code": "zoom-maintenance-code"
            }),
        ))
        .await
        .expect("Zoom OAuth maintenance complete response");
    assert_eq!(complete.status(), StatusCode::OK);

    let maintenance = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/maintenance",
            json!({
                "refresh_expiring_within_seconds": 300
            }),
        ))
        .await
        .expect("Zoom token maintenance response");
    assert_eq!(maintenance.status(), StatusCode::OK);
    let maintenance_body = json_body(maintenance).await;
    assert_eq!(maintenance_body["checked_count"], json!(1));
    assert_eq!(maintenance_body["refreshed_count"], json!(1));
    assert_eq!(maintenance_body["failed_count"], json!(0));
    assert_eq!(
        maintenance_body["items"][0]["account_id"],
        json!(account_id)
    );
    assert_eq!(maintenance_body["items"][0]["status"], json!("refreshed"));
    assert_eq!(maintenance_body["items"][0]["refreshed"], json!(true));

    let skip = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/maintenance",
            json!({
                "account_id": account_id,
                "refresh_expiring_within_seconds": 300
            }),
        ))
        .await
        .expect("Zoom token maintenance skip response");
    assert_eq!(skip.status(), StatusCode::OK);
    let skip_body = json_body(skip).await;
    assert_eq!(skip_body["checked_count"], json!(1));
    assert_eq!(skip_body["refreshed_count"], json!(0));
    assert_eq!(skip_body["skipped_count"], json!(1));
    assert_eq!(
        skip_body["items"][0]["status"],
        json!("skipped_not_expired")
    );

    let requests = token_requests.await.expect("maintenance token requests");
    assert_eq!(requests.len(), 2);
    assert!(requests[1].contains("grant_type=refresh_token"));
    assert!(requests[1].contains("refresh_token=zoom-maintenance-refresh-token"));

    let token_secret_ref = maintenance_body["items"][0]["account_id"]
        .as_str()
        .map(|_| format!("secret:provider-account:{account_id}:zoom_oauth_token"))
        .expect("maintenance account id");
    let vault = unlocked_test_vault(&context);
    let token_bundle = vault
        .read_secret(&token_secret_ref)
        .expect("read maintenance token bundle");
    assert!(token_bundle.contains("zoom-maintenance-refreshed-access-token"));

    let account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("maintenance account config");
    assert_eq!(
        account_config["authorization"]["last_token_refresh"]["status"],
        json!("refreshed")
    );
    let account_config_text = account_config.to_string();
    assert!(!account_config_text.contains("zoom-maintenance-client-secret"));
    assert!(!account_config_text.contains("zoom-maintenance-refreshed-access-token"));
    assert!(!account_config_text.contains("zoom-maintenance-refresh-token"));

    let maintenance_audit_items = zoom_audit_events(app.clone(), &account_id, 20).await;
    let refreshed_event =
        find_zoom_audit_event(&maintenance_audit_items, zoom_event_types::TOKEN_REFRESHED)
            .expect("maintenance refreshed audit event");
    assert_eq!(
        refreshed_event["provenance"]["action"],
        json!("token_maintenance")
    );
    let skipped_event = find_zoom_audit_event(
        &maintenance_audit_items,
        zoom_event_types::TOKEN_REFRESH_SKIPPED,
    )
    .expect("maintenance skipped audit event");
    assert_eq!(
        skipped_event["payload"]["status"],
        json!("skipped_not_expired")
    );
    assert_eq!(skipped_event["payload"]["refreshed"], json!(false));
    assert_eq!(
        skipped_event["provenance"]["action"],
        json!("token_maintenance")
    );
    assert_secret_like_payload_was_stripped(&refreshed_event["payload"]);
    assert_secret_like_payload_was_stripped(&skipped_event["payload"]);
}

#[tokio::test]
async fn zoom_oauth_webhook_subscription_management_uses_client_credentials() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-oauth-{suffix}");
    let webhook_secret_ref = format!("secret:zoom-webhook-oauth-{suffix}");
    seed_host_vault_secret_ref(
        &context,
        &pool,
        &account_id,
        &webhook_secret_ref,
        "zoom-webhook-secret-oauth",
        ProviderAccountSecretPurpose::ZoomWebhookSecret,
    )
    .await;
    let (token_endpoint, token_requests) = spawn_zoom_token_server_sequence(vec![
        json!({
            "access_token": "zoom-oauth-access-token",
            "refresh_token": "zoom-oauth-refresh-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "meeting:read recording:read"
        }),
        json!({
            "access_token": "zoom-webhook-management-access-token-1",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "marketplace:read marketplace:write"
        }),
        json!({
            "access_token": "zoom-webhook-management-access-token-2",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "marketplace:read marketplace:write"
        }),
        json!({
            "access_token": "zoom-webhook-management-access-token-3",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "marketplace:read marketplace:write"
        }),
    ])
    .await;
    let managed_subscription = json!({
        "id": format!("zoom-managed-subscription-{suffix}"),
        "subscription_name": "Макошь Zoom Runtime",
        "event_webhook_url": "https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks",
        "event_types": [
            "meeting.started",
            "meeting.ended",
            "meeting.participant_joined",
            "meeting.participant_left",
            "recording.completed"
        ]
    });
    let (api_base_url, api_requests) = spawn_zoom_webhook_subscription_server_sequence(vec![
        (
            200,
            "application/json",
            json!({ "event_subscriptions": [] }).to_string(),
        ),
        (200, "application/json", managed_subscription.to_string()),
        (
            200,
            "application/json",
            json!({ "event_subscriptions": [managed_subscription.clone()] }).to_string(),
        ),
        (204, "application/json", String::new()),
    ])
    .await;

    let start_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Webhook OAuth",
                "external_account_id": format!("zoom-webhook-oauth-external-{suffix}"),
                "client_id": "zoom-webhook-oauth-client",
                "client_secret": "zoom-webhook-oauth-client-secret",
                "webhook_secret_ref": webhook_secret_ref,
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/zoom/oauth/callback",
                "token_endpoint": token_endpoint,
                "scopes": ["meeting:read", "recording:read"]
            }),
        ))
        .await
        .expect("Zoom OAuth start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;

    let complete_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": start_body["setup_id"],
                "state": start_body["state"],
                "authorization_code": "zoom-webhook-oauth-code"
            }),
        ))
        .await
        .expect("Zoom OAuth complete response");
    assert_eq!(complete_response.status(), StatusCode::OK);

    let reconcile_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/webhook-subscriptions/reconcile",
            json!({
                "account_id": account_id,
                "endpoint_url": "https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks",
                "api_base_url": api_base_url
            }),
        ))
        .await
        .expect("Zoom webhook reconcile response");
    assert_eq!(reconcile_response.status(), StatusCode::OK);
    let reconcile_body = json_body(reconcile_response).await;
    assert_eq!(reconcile_body["status"], json!("created"));
    assert_eq!(
        reconcile_body["subscription"]["subscription_name"],
        json!("Макошь Zoom Runtime")
    );
    let encoded_api_base_url =
        url::form_urlencoded::byte_serialize(api_base_url.as_bytes()).collect::<String>();

    let status_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/webhook-subscriptions/status?account_id={account_id}&api_base_url={encoded_api_base_url}"
        )))
        .await
        .expect("Zoom webhook status response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = json_body(status_response).await;
    assert_eq!(
        status_body["managed_subscription_id"],
        managed_subscription["id"]
    );
    assert_eq!(
        status_body["subscriptions"]
            .as_array()
            .expect("subscriptions")
            .len(),
        1
    );

    let remove_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/webhook-subscriptions/remove",
            json!({
                "account_id": account_id,
                "api_base_url": api_base_url
            }),
        ))
        .await
        .expect("Zoom webhook remove response");
    assert_eq!(remove_response.status(), StatusCode::OK);
    let remove_body = json_body(remove_response).await;
    assert_eq!(remove_body["removed"], json!(true));
    assert_eq!(remove_body["subscription_id"], managed_subscription["id"]);

    let token_requests = token_requests.await.expect("Zoom webhook token requests");
    assert_eq!(token_requests.len(), 4);
    assert!(token_requests[1].contains("grant_type=client_credentials"));
    assert!(token_requests[2].contains("grant_type=client_credentials"));
    assert!(token_requests[3].contains("grant_type=client_credentials"));

    let api_requests = api_requests.await.expect("Zoom webhook API requests");
    assert_eq!(api_requests.len(), 4);
    assert!(api_requests[0].contains("GET /marketplace/app/event_subscription HTTP/1.1"));
    assert!(api_requests[1].contains("POST /marketplace/app/event_subscription HTTP/1.1"));
    assert!(api_requests[1].contains("\"subscription_name\":\"Макошь Zoom Runtime\""));
    assert!(api_requests[1].contains("\"event_webhook_url\":\"https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks\""));
    assert!(api_requests[2].contains("GET /marketplace/app/event_subscription HTTP/1.1"));
    assert!(api_requests[3].contains(&format!(
        "DELETE /marketplace/app/event_subscription/zoom-managed-subscription-{suffix} HTTP/1.1"
    )));

    let account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("webhook account config");
    assert_eq!(
        account_config["webhook_subscription"]["status"],
        json!("cleared")
    );
    assert_eq!(
        account_config["webhook_subscription"]["managed_subscription_id"],
        Value::Null
    );
}

#[tokio::test]
async fn zoom_server_to_server_webhook_subscription_management_uses_account_credentials() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-s2s-{suffix}");
    let zoom_account_id = format!("zoom-webhook-s2s-provider-{suffix}");
    let webhook_secret_ref = format!("secret:zoom-webhook-s2s-{suffix}");
    seed_host_vault_secret_ref(
        &context,
        &pool,
        &account_id,
        &webhook_secret_ref,
        "zoom-webhook-secret-s2s",
        ProviderAccountSecretPurpose::ZoomWebhookSecret,
    )
    .await;
    let (token_endpoint, token_requests) = spawn_zoom_token_server_sequence(vec![
        json!({
            "access_token": "zoom-s2s-access-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "recording:read"
        }),
        json!({
            "access_token": "zoom-s2s-webhook-management-access-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "marketplace:read marketplace:write"
        }),
    ])
    .await;
    let created_subscription = json!({
        "id": format!("zoom-s2s-managed-subscription-{suffix}"),
        "subscription_name": "Макошь Zoom Runtime",
        "event_webhook_url": "https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks",
        "event_types": [
            "meeting.started",
            "meeting.ended",
            "meeting.participant_joined",
            "meeting.participant_left",
            "recording.completed"
        ]
    });
    let (api_base_url, api_requests) = spawn_zoom_webhook_subscription_server_sequence(vec![
        (
            200,
            "application/json",
            json!({ "event_subscriptions": [] }).to_string(),
        ),
        (200, "application/json", created_subscription.to_string()),
    ])
    .await;

    let account_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Webhook S2S",
                "external_account_id": zoom_account_id,
                "auth_shape": "server_to_server",
                "client_id": "zoom-webhook-s2s-client",
                "webhook_secret_ref": webhook_secret_ref
            }),
        ))
        .await
        .expect("Zoom S2S account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let authorize_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/server-to-server/authorize",
            json!({
                "account_id": account_id,
                "client_id": "zoom-webhook-s2s-client",
                "client_secret": "zoom-webhook-s2s-client-secret",
                "token_endpoint": token_endpoint
            }),
        ))
        .await
        .expect("Zoom S2S authorize response");
    assert_eq!(authorize_response.status(), StatusCode::OK);

    let reconcile_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/webhook-subscriptions/reconcile",
            json!({
                "account_id": account_id,
                "endpoint_url": "https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks",
                "api_base_url": api_base_url
            }),
        ))
        .await
        .expect("Zoom S2S webhook reconcile response");
    assert_eq!(reconcile_response.status(), StatusCode::OK);
    let reconcile_body = json_body(reconcile_response).await;
    assert_eq!(reconcile_body["status"], json!("created"));
    assert_eq!(
        reconcile_body["subscription"]["subscription_id"],
        created_subscription["id"]
    );

    let token_requests = token_requests
        .await
        .expect("Zoom S2S webhook token requests");
    assert_eq!(token_requests.len(), 2);
    assert!(token_requests[0].contains("grant_type=account_credentials"));
    assert!(token_requests[0].contains(&format!("account_id={zoom_account_id}")));
    assert!(token_requests[1].contains("grant_type=account_credentials"));
    assert!(token_requests[1].contains(&format!("account_id={zoom_account_id}")));

    let api_requests = api_requests.await.expect("Zoom S2S webhook API requests");
    assert_eq!(api_requests.len(), 2);
    assert!(api_requests[0].contains("GET /marketplace/app/event_subscription HTTP/1.1"));
    assert!(api_requests[1].contains("POST /marketplace/app/event_subscription HTTP/1.1"));
}

#[tokio::test]
async fn zoom_provider_sync_recordings_ingests_calls_recordings_and_transcripts() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-provider-sync-{suffix}");
    let (token_endpoint, _token_requests) = spawn_zoom_token_server(json!({
        "access_token": "zoom-sync-access-token",
        "refresh_token": "zoom-sync-refresh-token",
        "expires_in": 3600,
        "token_type": "bearer",
        "scope": "recording:read"
    }))
    .await;
    let (api_base_url, sync_requests) = spawn_zoom_recording_sync_server_with_limit(3).await;
    enable_zoom_remote_recording_downloads(&pool).await;
    enable_zoom_remote_transcript_downloads(&pool).await;

    let start = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Provider Sync",
                "external_account_id": format!("zoom-provider-sync-external-{suffix}"),
                "client_id": "zoom-sync-client",
                "client_secret": "zoom-sync-client-secret",
                "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback",
                "token_endpoint": token_endpoint,
                "scopes": ["recording:read"]
            }),
        ))
        .await
        .expect("Zoom provider sync start response");
    assert_eq!(start.status(), StatusCode::OK);
    let start_body = json_body(start).await;
    let complete = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": start_body["setup_id"],
                "state": start_body["state"],
                "authorization_code": "zoom-provider-sync-code"
            }),
        ))
        .await
        .expect("Zoom provider sync complete response");
    assert_eq!(complete.status(), StatusCode::OK);

    let sync_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/provider-sync/recordings",
            json!({
                "account_id": account_id,
                "from": "2026-06-01",
                "to": "2026-06-30",
                "page_size": 10,
                "max_meetings": 10,
                "api_base_url": api_base_url
            }),
        ))
        .await
        .expect("Zoom provider sync response");
    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["user_id"], json!("me"));
    assert_eq!(sync_body["meetings_seen"], json!(1));
    assert_eq!(sync_body["meetings_recorded"], json!(1));
    assert_eq!(sync_body["recordings_recorded"], json!(2));
    assert_eq!(sync_body["media_downloads_recorded"], json!(1));
    assert_eq!(sync_body["transcripts_recorded"], json!(1));
    assert_eq!(sync_body["failures"], json!([]));

    let account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider sync account config");
    assert_eq!(
        account_config["recording_sync"]["status"],
        json!("completed")
    );
    assert_eq!(
        account_config["recording_sync"]["allow_remote_transcript_downloads"],
        json!(true)
    );
    assert_eq!(
        account_config["recording_sync"]["allow_remote_recording_downloads"],
        json!(true)
    );
    assert_eq!(
        account_config["recording_sync"]["meetings_recorded"],
        json!(1)
    );
    assert_eq!(
        account_config["recording_sync"]["recordings_recorded"],
        json!(2)
    );
    assert_eq!(
        account_config["recording_sync"]["media_downloads_recorded"],
        json!(1)
    );
    assert_eq!(
        account_config["recording_sync"]["transcripts_recorded"],
        json!(1)
    );
    assert_eq!(account_config["last_error"], Value::Null);

    let imported_attachment = sqlx::query(
        r#"
        SELECT i.attachment_id, i.content_type, i.source_kind, i.scan_status, b.storage_kind, b.storage_path
        FROM communication_attachment_imports i
        JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
        WHERE i.account_id = $1
          AND i.source_kind = 'zoom_recording_download'
        ORDER BY i.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("recording media import row");
    assert_eq!(
        imported_attachment.get::<String, _>("content_type"),
        "video/mp4"
    );
    assert_eq!(
        imported_attachment.get::<String, _>("storage_kind"),
        "local_fs"
    );
    assert!(
        imported_attachment
            .get::<String, _>("storage_path")
            .starts_with("sha256/"),
        "recording media import must persist through local blob storage"
    );

    let call_row = sqlx::query(
        r#"
        SELECT metadata
        FROM telegram_calls
        WHERE account_id = $1 AND provider_call_id = '9988776655'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider sync call row");
    let call_metadata = call_row.get::<Value, _>("metadata");
    assert_eq!(call_metadata["topic"], json!("Provider Sync Review"));

    let transcript_row = sqlx::query(
        r#"
        SELECT transcript_text
        FROM call_transcripts
        WHERE account_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider sync transcript row");
    assert_eq!(
        transcript_row.get::<String, _>("transcript_text"),
        "Provider sync transcript line."
    );

    let requests = sync_requests.await.expect("provider sync requests");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].contains("GET /users/me/recordings?"));
    assert!(requests[0].contains("from=2026-06-01"));
    assert!(requests[0].contains("to=2026-06-30"));
    assert!(
        requests[0].contains("Authorization: Bearer zoom-sync-access-token")
            || requests[0].contains("authorization: Bearer zoom-sync-access-token"),
        "recordings sync request must carry Bearer access token: {}",
        requests[0]
    );
    assert!(
        requests[1].contains("Authorization: Bearer zoom-sync-access-token")
            || requests[1].contains("authorization: Bearer zoom-sync-access-token"),
        "recording media download request must carry Bearer access token: {}",
        requests[1]
    );
    assert!(
        requests[1].contains("GET /download/video-1.mp4")
            || requests[1].contains("GET /download/video-1.mp4?"),
        "recording media download request path mismatch: {}",
        requests[1]
    );
    assert!(
        requests[2].contains("GET /download/transcript-1.vtt")
            || requests[2].contains("GET /download/transcript-1.vtt?"),
        "transcript download request path mismatch: {}",
        requests[2]
    );
    assert!(
        requests[2].contains("Authorization: Bearer zoom-sync-access-token")
            || requests[2].contains("authorization: Bearer zoom-sync-access-token"),
        "transcript download request must carry Bearer access token: {}",
        requests[2]
    );
}

#[tokio::test]
async fn zoom_token_refresh_failure_adds_and_clears_rotation_blocker() {
    let context = TestContext::new().await;
    initialize_host_vault(&context);
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let suffix = unique_suffix();
    let account_id = format!("zoom-oauth-refresh-rotation-{suffix}");
    let (token_endpoint, token_requests) = spawn_zoom_token_server_sequence(vec![
        json!({
            "access_token": "zoom-refresh-rotation-initial-access-token",
            "refresh_token": "zoom-refresh-rotation-initial-refresh-token",
            "expires_in": 1,
            "token_type": "bearer",
            "scope": "meeting:read"
        }),
        json!({
            "error": "invalid_grant"
        }),
        json!({
            "access_token": "zoom-refresh-rotation-refreshed-access-token",
            "refresh_token": "zoom-refresh-rotation-refreshed-refresh-token",
            "expires_in": 3600,
            "token_type": "bearer",
            "scope": "meeting:read recording:read"
        }),
    ])
    .await;

    let start = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Zoom OAuth Refresh Rotation",
                "external_account_id": format!("zoom-refresh-rotation-external-{suffix}"),
                "client_id": "zoom-refresh-rotation-client",
                "client_secret": "zoom-refresh-rotation-client-secret",
                "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback",
                "token_endpoint": token_endpoint,
                "scopes": ["meeting:read"]
            }),
        ))
        .await
        .expect("Zoom OAuth refresh rotation start response");
    assert_eq!(start.status(), StatusCode::OK);
    let start_body = json_body(start).await;
    let complete = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": start_body["setup_id"],
                "state": start_body["state"],
                "authorization_code": "zoom-refresh-rotation-code"
            }),
        ))
        .await
        .expect("Zoom OAuth refresh rotation complete response");
    assert_eq!(complete.status(), StatusCode::OK);

    let failed_refresh = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/refresh",
            json!({
                "account_id": account_id,
                "force": true
            }),
        ))
        .await
        .expect("Zoom OAuth refresh rotation failed refresh response");
    assert!(!failed_refresh.status().is_success());

    let failed_status = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/runtime/status?account_id={account_id}"
        )))
        .await
        .expect("runtime status for failed refresh");
    assert_eq!(failed_status.status(), StatusCode::OK);
    let failed_status_body = json_body(failed_status).await;
    let failed_blockers = failed_status_body["runtime_blockers"]
        .as_array()
        .expect("runtime_blockers");
    assert!(
        failed_blockers
            .iter()
            .any(|item| item.as_str() == Some("zoom_token_rotation_required")),
        "refresh failure must set token rotation blocker"
    );
    assert!(failed_status_body["last_error"].is_string());
    assert_eq!(
        failed_status_body["metadata"]["token_rotation_policy"]["status"],
        json!("required")
    );
    assert_eq!(
        failed_status_body["metadata"]["token_rotation_policy"]["last_refresh_status"],
        json!("failed")
    );
    assert_eq!(
        failed_status_body["metadata"]["token_rotation_policy"]["policy"]["maintenance_refresh_threshold_seconds"],
        json!(300)
    );

    let account_config_failed: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("account config after failed refresh");
    assert_eq!(
        account_config_failed["authorization"]["last_token_refresh"]["status"],
        json!("failed")
    );
    assert!(
        !account_config_failed["authorization"]["last_token_refresh"]["error"]
            .as_str()
            .expect("last token refresh error")
            .is_empty(),
    );

    let recovered_refresh = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/refresh",
            json!({
                "account_id": account_id,
                "force": true
            }),
        ))
        .await
        .expect("Zoom OAuth refresh rotation recovery response");
    assert_eq!(recovered_refresh.status(), StatusCode::OK);
    let recovered_refresh_body = json_body(recovered_refresh).await;
    assert_eq!(recovered_refresh_body["status"], json!("refreshed"));

    let recovered_status = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/runtime/status?account_id={account_id}"
        )))
        .await
        .expect("runtime status for recovered refresh");
    assert_eq!(recovered_status.status(), StatusCode::OK);
    let recovered_status_body = json_body(recovered_status).await;
    let recovered_blockers = recovered_status_body["runtime_blockers"]
        .as_array()
        .expect("runtime_blockers");
    assert!(
        !recovered_blockers
            .iter()
            .any(|item| item.as_str() == Some("zoom_token_rotation_required")),
        "refresh success must clear token rotation blocker"
    );
    assert_eq!(recovered_status_body["last_error"], json!(null));
    assert_eq!(
        recovered_status_body["metadata"]["token_rotation_policy"]["status"],
        json!("current")
    );

    let account_config_recovered: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("account config after recovered refresh");
    assert_eq!(
        account_config_recovered["authorization"]["last_token_refresh"]["status"],
        json!("refreshed")
    );
    assert!(account_config_recovered["last_error"].is_null());

    let requests = token_requests.await.expect("token request list");
    assert_eq!(requests.len(), 3);
    assert!(requests[1].contains("grant_type=refresh_token"));
    assert!(requests[1].contains("refresh_token=zoom-refresh-rotation-initial-refresh-token"));

    let audit_items = zoom_audit_events(app.clone(), &account_id, 20).await;
    let failed_event = find_zoom_audit_event(&audit_items, zoom_event_types::TOKEN_REFRESH_FAILED)
        .expect("failed refresh audit event");
    assert_eq!(failed_event["payload"]["status"], json!("failed"));
    assert_eq!(failed_event["payload"]["refreshed"], json!(false));
    assert_eq!(
        failed_event["provenance"]["action"],
        json!("explicit_refresh")
    );
    let refreshed_event = find_zoom_audit_event(&audit_items, zoom_event_types::TOKEN_REFRESHED)
        .expect("recovered refresh audit event");
    assert_eq!(refreshed_event["payload"]["status"], json!("refreshed"));
    assert_eq!(refreshed_event["payload"]["refreshed"], json!(true));
    assert_secret_like_payload_was_stripped(&failed_event["payload"]);
    assert_secret_like_payload_was_stripped(&refreshed_event["payload"]);
}

#[tokio::test]
async fn zoom_webhook_url_validation_uses_host_vault_secret() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-validation-{suffix}");
    let secret_ref = format!("secret:zoom-webhook-validation-{suffix}");
    let webhook_signing_key = "zoom-webhook-signing-key-validation";
    let (_context, app, _pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;
    let validation_nonce = "zoom-validation-nonce";

    let response = app
        .oneshot(json_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            json!({
                "event": "endpoint.url_validation",
                "event_ts": 1_782_500_000_000_i64,
                "payload": {
                "plainToken": validation_nonce
                }
            }),
        ))
        .await
        .expect("url validation response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["plainToken"], json!(validation_nonce));
    assert_eq!(
        body["encryptedToken"],
        json!(zoom_validation_token(webhook_signing_key, validation_nonce))
    );
}

#[tokio::test]
async fn zoom_signed_webhooks_normalize_meetings_recordings_and_reject_bad_signatures() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-{suffix}");
    let secret_ref = format!("secret:zoom-webhook-{suffix}");
    let webhook_signing_key = "zoom-webhook-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;

    let meeting_body = json!({
        "event": "meeting.started",
        "event_ts": 1_782_500_100_000_i64,
        "payload": {
            "account_id": "zoom-provider-account",
            "object": {
                "id": "987654321",
                "uuid": "meeting-webhook-uuid",
                "topic": "Webhook Planning",
                "host_email": "host@example.test",
                "join_url": "https://example.zoom.us/j/987654321",
                "start_time": "2026-06-27T15:00:00Z",
                "password": "leak-password",
                "metadata": {
                    "access_token": "leak-access-token",
                    "safe": "kept"
                }
            }
        }
    });
    let meeting_response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &meeting_body,
            webhook_signing_key,
        ))
        .await
        .expect("meeting webhook response");
    assert_eq!(meeting_response.status(), StatusCode::OK);
    let meeting_response_body = json_body(meeting_response).await;
    assert_eq!(meeting_response_body["status"], json!("recorded"));
    assert_eq!(
        meeting_response_body["meeting"]["meeting_id"],
        json!("987654321")
    );

    let call_metadata: Value = sqlx::query(
        r#"
        SELECT metadata
        FROM telegram_calls
        WHERE account_id = $1 AND provider_call_id = '987654321'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("webhook call row")
    .get("metadata");
    let serialized_call_metadata = call_metadata.to_string();
    assert!(
        !serialized_call_metadata.contains("leak-password")
            && !serialized_call_metadata.contains("leak-access-token")
            && !serialized_call_metadata.contains("access_token"),
        "Zoom call metadata must be sanitized: {serialized_call_metadata}"
    );

    let recording_body = json!({
        "event": "recording.completed",
        "event_ts": 1_782_500_200_000_i64,
        "payload": {
            "object": {
                "id": "987654321",
                "uuid": "meeting-webhook-uuid",
                "recording_files": [
                    {
                        "id": "recording-webhook-1",
                        "file_type": "MP4",
                        "recording_type": "shared_screen_with_speaker_view",
                        "download_url": "https://example.zoom.us/recording/download/1",
                        "download_token": "leak-download-token",
                        "file_size": 12345,
                        "recording_start": "2026-06-27T15:00:00Z"
                    }
                ]
            }
        }
    });
    let recording_response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &recording_body,
            webhook_signing_key,
        ))
        .await
        .expect("recording webhook response");
    assert_eq!(recording_response.status(), StatusCode::OK);
    let recording_response_body = json_body(recording_response).await;
    assert_eq!(recording_response_body["status"], json!("recorded"));
    assert_eq!(
        recording_response_body["recordings"][0]["recording_id"],
        json!("recording-webhook-1")
    );
    let recording_payload = event_payload(
        &pool,
        zoom_event_types::RECORDING_OBSERVED,
        "recording_id",
        "recording-webhook-1",
    )
    .await;
    assert!(
        !recording_payload
            .to_string()
            .contains("leak-download-token"),
        "Zoom recording event payload must be sanitized: {recording_payload}"
    );

    let bad_signature_response = app
        .oneshot(zoom_signed_post_with_signature(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &meeting_body,
            "v0=0000000000000000000000000000000000000000000000000000000000000000",
        ))
        .await
        .expect("bad signature response");
    assert_eq!(bad_signature_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn zoom_recording_webhook_auto_imports_media_files_from_download_urls() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-recording-download-{suffix}");
    let secret_ref = format!("secret:zoom-webhook-recording-download-{suffix}");
    let webhook_signing_key = "zoom-webhook-recording-download-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;
    enable_zoom_remote_recording_downloads(&pool).await;
    set_zoom_recording_import_retention_days(&pool, 14).await;
    let (download_url, request_handle) =
        spawn_zoom_media_download_server(b"fake webhook mp4", "video/mp4").await;

    let recording_body = json!({
        "event": "recording.completed",
        "event_id": format!("zoom-recording-media-{suffix}"),
        "event_ts": 1_782_500_180_000_i64,
        "payload": {
            "object": {
                "id": "987654321",
                "uuid": "meeting-webhook-recording-uuid",
                "recording_files": [
                    {
                        "id": "recording-webhook-media-1",
                        "file_type": "MP4",
                        "file_extension": "MP4",
                        "download_url": download_url,
                        "download_token": "recording-download-token",
                        "recording_start": "2026-06-27T15:55:00Z"
                    }
                ]
            }
        }
    });
    let response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &recording_body,
            webhook_signing_key,
        ))
        .await
        .expect("recording media webhook response");
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = json_body(response).await;
    assert_eq!(response_body["status"], json!("recorded"));
    assert_eq!(
        response_body["recording_imports"][0]["status"],
        json!("recorded")
    );
    assert_eq!(
        response_body["recording_imports"][0]["recording"]["recording_id"],
        json!("recording-webhook-media-1")
    );

    let import_row = sqlx::query(
        r#"
        SELECT i.content_type, b.storage_kind, b.storage_path
        FROM communication_attachment_imports i
        JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
        WHERE i.account_id = $1
          AND i.source_kind = 'zoom_recording_download'
          AND i.metadata ->> 'recording_id' = 'recording-webhook-media-1'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("webhook recording media import row");
    assert_eq!(import_row.get::<String, _>("content_type"), "video/mp4");
    assert_eq!(import_row.get::<String, _>("storage_kind"), "local_fs");
    assert!(
        import_row
            .get::<String, _>("storage_path")
            .starts_with("sha256/"),
        "webhook recording media import must persist through local blob storage"
    );

    let recording_payload = event_payload(
        &pool,
        zoom_event_types::RECORDING_OBSERVED,
        "recording_id",
        "recording-webhook-media-1",
    )
    .await;
    assert!(
        !recording_payload
            .to_string()
            .contains("recording-download-token"),
        "Zoom recording event payload must be sanitized: {recording_payload}"
    );

    let request_text = request_handle
        .await
        .expect("recording media download request text");
    assert!(
        request_text.contains("Authorization: Bearer recording-download-token")
            || request_text.contains("authorization: Bearer recording-download-token"),
        "recording media download request must carry Bearer download token: {request_text}"
    );

    let imports_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/accounts/{account_id}/recording-imports?limit=5"
        )))
        .await
        .expect("recording imports response");
    assert_eq!(imports_response.status(), StatusCode::OK);
    let imports_body = json_body(imports_response).await;
    assert_eq!(imports_body["account_id"], json!(account_id));
    assert_eq!(
        imports_body["items"][0]["recording_id"],
        json!("recording-webhook-media-1")
    );
    assert_eq!(
        imports_body["items"][0]["source"],
        json!("zoom_recording_webhook")
    );
    assert_eq!(imports_body["items"][0]["storage_kind"], json!("local_fs"));
    assert_eq!(
        imports_body["items"][0]["retention_mode"],
        json!("delete_after_n_days")
    );
    assert_eq!(imports_body["items"][0]["retention_days"], json!(14));
    assert!(imports_body["items"][0]["expires_at"].is_string());

    let audit_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/accounts/{account_id}/audit-events?limit=5"
        )))
        .await
        .expect("audit events response");
    assert_eq!(audit_response.status(), StatusCode::OK);
    let audit_body = json_body(audit_response).await;
    assert_eq!(audit_body["account_id"], json!(account_id));
    assert_eq!(
        audit_body["items"][0]["event_type"],
        json!("zoom.recording.observed")
    );
    assert_eq!(
        audit_body["items"][0]["subject_kind"],
        json!("zoom_recording")
    );
}

#[tokio::test]
async fn zoom_recording_import_retention_remove_cleans_up_orphaned_blob_and_audits() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-recording-import-remove-{suffix}");
    let secret_ref = format!("secret:zoom-recording-import-remove-{suffix}");
    let webhook_signing_key = "zoom-recording-import-remove-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;
    enable_zoom_remote_recording_downloads(&pool).await;
    let (download_url, _request_handle) =
        spawn_zoom_media_download_server(b"retention remove mp4", "video/mp4").await;

    let recording_body = json!({
        "event": "recording.completed",
        "event_id": format!("zoom-recording-import-remove-event-{suffix}"),
        "event_ts": 1_782_500_280_000_i64,
        "payload": {
            "object": {
                "id": "987654399",
                "uuid": "meeting-recording-import-remove-uuid",
                "recording_files": [
                    {
                        "id": "recording-import-remove-1",
                        "file_type": "MP4",
                        "file_extension": "MP4",
                        "download_url": download_url,
                        "download_token": "recording-remove-download-token",
                        "recording_start": "2026-06-27T16:05:00Z"
                    }
                ]
            }
        }
    });
    let import_response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &recording_body,
            webhook_signing_key,
        ))
        .await
        .expect("recording import webhook response");
    assert_eq!(import_response.status(), StatusCode::OK);

    let import_row = sqlx::query(
        r#"
        SELECT i.attachment_id, i.blob_id, b.storage_path
        FROM communication_attachment_imports i
        JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
        WHERE i.account_id = $1
          AND i.source_kind = 'zoom_recording_download'
          AND i.metadata ->> 'recording_id' = 'recording-import-remove-1'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("recording import row before removal");
    let attachment_id: String = import_row.get("attachment_id");
    let blob_id: String = import_row.get("blob_id");
    let storage_path: String = import_row.get("storage_path");
    let blob_file_path = std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT).join(&storage_path);
    assert!(
        blob_file_path.is_file(),
        "imported blob file must exist before removal"
    );

    let remove_response = app
        .clone()
        .oneshot(json_post(
            &format!(
                "/api/v1/integrations/zoom/accounts/{account_id}/recording-imports/{attachment_id}/remove"
            ),
            json!({
                "reason": "fixture_cleanup"
            }),
        ))
        .await
        .expect("recording import remove response");
    assert_eq!(remove_response.status(), StatusCode::OK);
    let remove_body = json_body(remove_response).await;
    assert_eq!(remove_body["account_id"], json!(account_id));
    assert_eq!(remove_body["attachment_id"], json!(attachment_id));
    assert_eq!(remove_body["blob_id"], json!(blob_id));
    assert_eq!(
        remove_body["recording_id"],
        json!("recording-import-remove-1")
    );
    assert_eq!(remove_body["removed"], json!(true));
    assert_eq!(remove_body["blob_metadata_removed"], json!(true));
    assert_eq!(remove_body["blob_file_removed"], json!(true));

    let remaining_imports = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_attachment_imports
        WHERE attachment_id = $1
        "#,
    )
    .bind(&attachment_id)
    .fetch_one(&pool)
    .await
    .expect("remaining import count");
    assert_eq!(remaining_imports, 0);

    let remaining_blobs = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_mail_blobs
        WHERE blob_id = $1
        "#,
    )
    .bind(&blob_id)
    .fetch_one(&pool)
    .await
    .expect("remaining blob count");
    assert_eq!(remaining_blobs, 0);
    assert!(
        !blob_file_path.is_file(),
        "orphaned local blob file must be removed after retention cleanup"
    );

    let imports_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/accounts/{account_id}/recording-imports?limit=5"
        )))
        .await
        .expect("recording imports after removal response");
    assert_eq!(imports_response.status(), StatusCode::OK);
    let imports_body = json_body(imports_response).await;
    assert_eq!(imports_body["items"], json!([]));

    let audit_response = app
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/accounts/{account_id}/audit-events?limit=10"
        )))
        .await
        .expect("audit events after removal response");
    assert_eq!(audit_response.status(), StatusCode::OK);
    let audit_body = json_body(audit_response).await;
    assert!(
        audit_body["items"]
            .as_array()
            .expect("audit items")
            .iter()
            .any(|item| {
                item["event_type"] == json!(zoom_event_types::RECORDING_IMPORT_REMOVED)
                    && item["subject_kind"] == json!("zoom_recording_import")
                    && item["subject_entity_id"] == json!(attachment_id)
            }),
        "recording import removal must be visible in account-scoped audit events"
    );
}

#[tokio::test]
async fn zoom_retention_cleanup_prunes_expired_recordings_and_transcripts() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-retention-cleanup-{suffix}");
    let secret_ref = format!("secret:zoom-retention-cleanup-{suffix}");
    let webhook_signing_key = "zoom-retention-cleanup-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;
    enable_zoom_remote_recording_downloads(&pool).await;

    let (download_url, _request_handle) =
        spawn_zoom_media_download_server(b"expired retention cleanup recording bytes", "video/mp4")
            .await;
    let webhook_response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &json!({
                "event": "recording.completed",
                "payload": {
                    "account_id": account_id,
                    "object": {
                        "id": "cleanup-meeting-1",
                        "uuid": "cleanup-meeting-uuid-1",
                        "topic": "Expired cleanup meeting",
                        "host_email": "cleanup@example.test",
                        "start_time": "2026-06-27T14:00:00Z",
                        "share_url": "https://example.zoom.us/rec/share/cleanup",
                        "recording_files": [
                            {
                                "id": "cleanup-recording-1",
                                "recording_type": "shared_screen_with_speaker_view",
                                "status": "completed",
                                "file_type": "MP4",
                                "file_extension": "MP4",
                                "download_url": download_url,
                                "download_token": "cleanup-recording-download-token",
                                "recording_start": "2026-06-27T14:05:00Z"
                            }
                        ]
                    }
                }
            }),
            webhook_signing_key,
        ))
        .await
        .expect("cleanup webhook response");
    assert_eq!(webhook_response.status(), StatusCode::OK);

    let transcript_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
            json!({
                "observation_id": format!("cleanup-transcript-observation-{suffix}"),
                "transcript_id": format!("cleanup-transcript-{suffix}"),
                "account_id": account_id,
                "meeting_id": "cleanup-meeting-1",
                "file_name": "cleanup.vtt",
                "content_type": "text/vtt",
                "file_text": "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nRetention cleanup transcript.\n",
                "metadata": {}
            }),
        ))
        .await
        .expect("cleanup transcript response");
    assert_eq!(transcript_response.status(), StatusCode::OK);

    let recording_attachment_id: String = sqlx::query_scalar(
        r#"
        SELECT attachment_id
        FROM communication_attachment_imports
        WHERE account_id = $1
          AND source_kind = 'zoom_recording_download'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("cleanup recording attachment id");
    let transcript_id = format!("cleanup-transcript-{suffix}");
    let cleanup_call_id: String = sqlx::query_scalar(
        r#"
        SELECT call_id
        FROM call_transcripts
        WHERE transcript_id = $1
        "#,
    )
    .bind(&transcript_id)
    .fetch_one(&pool)
    .await
    .expect("cleanup call id");

    force_zoom_recording_import_retention_expiry(&pool, &recording_attachment_id).await;
    force_zoom_transcript_retention_expiry(&pool, &transcript_id).await;

    let cleanup_response = app
        .clone()
        .oneshot(json_post(
            &format!("/api/v1/integrations/zoom/accounts/{account_id}/retention/prune"),
            json!({
                "remove_recordings": true,
                "remove_transcripts": true,
                "limit": 10
            }),
        ))
        .await
        .expect("cleanup retention response");
    assert_eq!(cleanup_response.status(), StatusCode::OK);
    let cleanup_body = json_body(cleanup_response).await;
    assert_eq!(cleanup_body["account_id"], json!(account_id));
    assert_eq!(cleanup_body["recordings_removed"], json!(1));
    assert_eq!(cleanup_body["transcripts_removed"], json!(1));
    assert_eq!(
        cleanup_body["items"][0]["evidence_kind"],
        json!("recording_import")
    );
    assert_eq!(
        cleanup_body["items"][1]["evidence_kind"],
        json!("transcript")
    );
    assert_eq!(cleanup_body["items"][1]["call_id"], json!(cleanup_call_id));
    assert_eq!(
        cleanup_body["items"][1]["transcript_id"],
        json!(transcript_id)
    );

    let remaining_imports = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_attachment_imports WHERE attachment_id = $1",
    )
    .bind(&recording_attachment_id)
    .fetch_one(&pool)
    .await
    .expect("remaining expired imports");
    assert_eq!(remaining_imports, 0);

    let remaining_transcripts = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM call_transcripts WHERE transcript_id = $1",
    )
    .bind(&transcript_id)
    .fetch_one(&pool)
    .await
    .expect("remaining expired transcripts");
    assert_eq!(remaining_transcripts, 0);

    let audit_items = zoom_audit_events(app.clone(), &account_id, 20).await;
    assert!(
        find_zoom_audit_event(&audit_items, zoom_event_types::RECORDING_IMPORT_REMOVED).is_some()
    );
    assert!(find_zoom_audit_event(&audit_items, zoom_event_types::TRANSCRIPT_REMOVED).is_some());
    let cleanup_event =
        find_zoom_audit_event(&audit_items, zoom_event_types::RETENTION_CLEANUP_COMPLETED)
            .expect("retention cleanup completed audit event");
    assert_eq!(cleanup_event["payload"]["recordings_removed"], json!(1));
    assert_eq!(cleanup_event["payload"]["transcripts_removed"], json!(1));
}

#[tokio::test]
async fn zoom_recording_webhook_auto_imports_transcript_files_from_download_urls() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-transcript-download-{suffix}");
    let secret_ref = format!("secret:zoom-webhook-transcript-download-{suffix}");
    let webhook_signing_key = "zoom-webhook-transcript-download-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;
    enable_zoom_remote_transcript_downloads(&pool).await;
    let (download_url, request_handle) = spawn_zoom_transcript_download_server(
        "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nTranscript from webhook download.",
        "text/vtt",
    )
    .await;

    let recording_body = json!({
        "event": "recording.completed",
        "event_id": format!("zoom-recording-transcript-{suffix}"),
        "event_ts": 1_782_500_200_000_i64,
        "payload": {
            "object": {
                "id": "987654322",
                "uuid": "meeting-webhook-transcript-uuid",
                "recording_files": [
                    {
                        "id": "recording-webhook-transcript-1",
                        "file_type": "TRANSCRIPT",
                        "file_extension": "VTT",
                        "download_url": download_url,
                        "download_token": "download-token-secret",
                        "recording_start": "2026-06-27T16:00:00Z"
                    }
                ]
            }
        }
    });
    let response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &recording_body,
            webhook_signing_key,
        ))
        .await
        .expect("recording transcript webhook response");
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = json_body(response).await;
    assert_eq!(response_body["status"], json!("recorded"));
    assert_eq!(
        response_body["transcript_imports"][0]["status"],
        json!("recorded")
    );

    let transcript_row = sqlx::query(
        r#"
        SELECT transcript_text
        FROM call_transcripts
        WHERE transcript_id = $1
        "#,
    )
    .bind("zoom-transcript-download:987654322:recording-webhook-transcript-1")
    .fetch_one(&pool)
    .await
    .expect("downloaded transcript row");
    assert_eq!(
        transcript_row.get::<String, _>("transcript_text"),
        "Transcript from webhook download."
    );

    let transcript_payload = event_payload(
        &pool,
        zoom_event_types::TRANSCRIPT_OBSERVED,
        "transcript_id",
        "zoom-transcript-download:987654322:recording-webhook-transcript-1",
    )
    .await;
    assert!(
        !transcript_payload
            .to_string()
            .contains("download-token-secret"),
        "Zoom transcript event payload must be sanitized: {transcript_payload}"
    );

    let request_text = request_handle.await.expect("download request text");
    assert!(
        request_text.contains("Authorization: Bearer download-token-secret")
            || request_text.contains("authorization: Bearer download-token-secret"),
        "download request must carry Bearer download token: {request_text}"
    );
}

#[tokio::test]
async fn zoom_remote_transcript_downloads_are_blocked_until_privacy_opt_in_is_enabled() {
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-policy-{suffix}");
    let secret_ref = format!("secret:zoom-webhook-policy-{suffix}");
    let webhook_signing_key = "zoom-webhook-policy-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;

    let recording_body = json!({
        "event": "recording.completed",
        "event_ts": 1_782_500_350_000_i64,
        "payload": {
            "object": {
                "id": "policy-meeting-1",
                "uuid": "policy-meeting-uuid",
                "recording_files": [
                    {
                        "id": "policy-recording-transcript-1",
                        "file_type": "TRANSCRIPT",
                        "file_extension": "VTT",
                        "download_url": "http://127.0.0.1:9/blocked-transcript.vtt",
                        "download_token": "download-token-secret",
                        "recording_start": "2026-06-27T16:30:00Z"
                    }
                ]
            }
        }
    });

    let response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &recording_body,
            webhook_signing_key,
        ))
        .await
        .expect("policy webhook response");
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = json_body(response).await;
    assert_eq!(
        response_body["transcript_imports"][0]["status"],
        json!("blocked")
    );
    assert_eq!(
        response_body["transcript_imports"][0]["error"],
        json!("zoom_remote_transcript_download_not_enabled")
    );

    let transcript_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM call_transcripts
            WHERE transcript_id = $1
        )
        "#,
    )
    .bind("zoom-transcript-download:policy-meeting-1:policy-recording-transcript-1")
    .fetch_one(&pool)
    .await
    .expect("policy transcript existence");
    assert!(!transcript_exists);

    let sync_context = TestContext::new().await;
    initialize_host_vault(&sync_context);
    let database_url = sync_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let sync_pool = database.pool().expect("pool").clone();
    let sync_app = build_router_with_database(sync_context.app_config(LOCAL_API_TOKEN), database);
    let sync_account_id = format!("zoom-provider-sync-policy-{suffix}");
    let (token_endpoint, _token_requests) = spawn_zoom_token_server(json!({
        "access_token": "zoom-sync-policy-access-token",
        "refresh_token": "zoom-sync-policy-refresh-token",
        "expires_in": 3600,
        "token_type": "bearer",
        "scope": "recording:read"
    }))
    .await;
    let (api_base_url, sync_requests) = spawn_zoom_recording_sync_server_with_limit(1).await;

    let start = sync_app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/start",
            json!({
                "account_id": sync_account_id,
                "display_name": "Zoom Provider Sync Policy",
                "external_account_id": format!("zoom-provider-sync-policy-external-{suffix}"),
                "client_id": "zoom-sync-policy-client",
                "client_secret": "zoom-sync-policy-client-secret",
                "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback",
                "token_endpoint": token_endpoint,
                "scopes": ["recording:read"]
            }),
        ))
        .await
        .expect("Zoom provider sync policy start response");
    let start_body = json_body(start).await;
    let complete = sync_app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/oauth/complete",
            json!({
                "setup_id": start_body["setup_id"],
                "state": start_body["state"],
                "authorization_code": "zoom-provider-sync-policy-code"
            }),
        ))
        .await
        .expect("Zoom provider sync policy complete response");
    assert_eq!(complete.status(), StatusCode::OK);

    let sync_response = sync_app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/provider-sync/recordings",
            json!({
                "account_id": sync_account_id,
                "from": "2026-06-01",
                "to": "2026-06-30",
                "page_size": 10,
                "max_meetings": 10,
                "api_base_url": api_base_url
            }),
        ))
        .await
        .expect("Zoom provider sync policy response");
    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["media_downloads_recorded"], json!(0));
    assert_eq!(sync_body["transcripts_recorded"], json!(0));
    assert_eq!(
        sync_body["failures"][0]["step"],
        json!("recording_download_policy")
    );
    assert_eq!(
        sync_body["failures"][0]["error"],
        json!("zoom_remote_recording_download_not_enabled")
    );
    assert_eq!(sync_body["failures"][1]["step"], json!("policy"));
    assert_eq!(
        sync_body["failures"][1]["error"],
        json!("zoom_remote_transcript_download_not_enabled")
    );

    let sync_account_config: Value = sqlx::query_scalar(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&sync_account_id)
    .fetch_one(&sync_pool)
    .await
    .expect("provider sync policy account config");
    assert_eq!(
        sync_account_config["recording_sync"]["status"],
        json!("completed_with_failures")
    );
    assert_eq!(
        sync_account_config["recording_sync"]["failure_count"],
        json!(2)
    );
    assert_eq!(
        sync_account_config["recording_sync"]["allow_remote_recording_downloads"],
        json!(false)
    );
    assert_eq!(sync_account_config["last_error"], Value::Null);

    let sync_transcript_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM call_transcripts
            WHERE account_id = $1
        )
        "#,
    )
    .bind(&sync_account_id)
    .fetch_one(&sync_pool)
    .await
    .expect("sync transcript existence");
    assert!(!sync_transcript_exists);

    let sync_media_import_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM communication_attachment_imports
            WHERE account_id = $1
              AND source_kind = 'zoom_recording_download'
        )
        "#,
    )
    .bind(&sync_account_id)
    .fetch_one(&sync_pool)
    .await
    .expect("sync media import existence");
    assert!(!sync_media_import_exists);

    let requests = sync_requests.await.expect("policy sync requests");
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn zoom_remote_recording_downloads_are_blocked_for_webhooks_until_privacy_opt_in_is_enabled()
{
    let suffix = unique_suffix();
    let account_id = format!("zoom-webhook-recording-policy-{suffix}");
    let secret_ref = format!("secret:zoom-webhook-recording-policy-{suffix}");
    let webhook_signing_key = "zoom-webhook-recording-policy-signing-key";
    let (_context, app, pool) =
        test_app_with_zoom_webhook_secret(&account_id, &secret_ref, webhook_signing_key).await;

    let recording_body = json!({
        "event": "recording.completed",
        "event_ts": 1_782_500_340_000_i64,
        "payload": {
            "object": {
                "id": "policy-media-meeting-1",
                "uuid": "policy-media-meeting-uuid",
                "recording_files": [
                    {
                        "id": "policy-recording-media-1",
                        "file_type": "MP4",
                        "file_extension": "MP4",
                        "download_url": "http://127.0.0.1:9/blocked-recording.mp4",
                        "download_token": "blocked-recording-token",
                        "recording_start": "2026-06-27T16:25:00Z"
                    }
                ]
            }
        }
    });

    let response = app
        .clone()
        .oneshot(zoom_signed_post(
            &format!("/api/v1/integrations/zoom/runtime-bridge/webhooks?account_id={account_id}"),
            &recording_body,
            webhook_signing_key,
        ))
        .await
        .expect("recording media policy webhook response");
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = json_body(response).await;
    assert_eq!(
        response_body["recording_imports"][0]["status"],
        json!("blocked")
    );
    assert_eq!(
        response_body["recording_imports"][0]["error"],
        json!("zoom_remote_recording_download_not_enabled")
    );

    let recording_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM communication_attachment_imports
            WHERE account_id = $1
              AND source_kind = 'zoom_recording_download'
              AND metadata ->> 'recording_id' = 'policy-recording-media-1'
        )
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("webhook recording media existence");
    assert!(!recording_exists);
}

#[tokio::test]
async fn zoom_runtime_bridge_writes_call_transcript_events_and_sanitizes_payloads() {
    let (_context, app, pool) = test_app().await;
    let suffix = unique_suffix();
    let account_id = format!("zoom-bridge-{suffix}");
    create_fixture_account(&app, &account_id, &suffix.to_string()).await;

    let meeting_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/meetings",
            json!({
                "observation_id": format!("obs-meeting-{suffix}"),
                "account_id": account_id,
                "meeting_id": "meeting-123",
                "meeting_uuid": "uuid-123",
                "topic": "Foundation Review",
                "host_email": "host@example.test",
                "started_at": "2026-06-27T10:00:00Z",
                "ended_at": "2026-06-27T10:45:00Z",
                "duration_seconds": 2700,
                "participants": [
                    {
                        "display_name": "Participant",
                        "metadata": {
                            "access_token": "participant-token",
                            "safe": "kept"
                        }
                    }
                ],
                "metadata": {
                    "access_token": "meeting-token",
                    "nested": {
                        "client_secret": "meeting-secret",
                        "safe": "kept"
                    }
                }
            }),
        ))
        .await
        .expect("meeting bridge response");
    assert_eq!(meeting_response.status(), StatusCode::OK);
    let meeting_body = json_body(meeting_response).await;
    let call_id = meeting_body["call_id"].as_str().expect("call_id");
    assert!(call_id.starts_with("zoom_call_"));

    let call_row = sqlx::query(
        r#"
        SELECT call_id, metadata
        FROM telegram_calls
        WHERE account_id = $1 AND provider_call_id = 'meeting-123'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("zoom call row");
    assert_eq!(call_row.get::<String, _>("call_id"), call_id);
    assert_eq!(
        call_row.get::<Value, _>("metadata")["provider"],
        json!("zoom")
    );

    let meeting_event_payload = event_payload(
        &pool,
        zoom_event_types::MEETING_OBSERVED,
        "meeting_id",
        "meeting-123",
    )
    .await;
    assert_secret_like_payload_was_stripped(&meeting_event_payload);
    assert_eq!(
        meeting_event_payload["metadata"]["nested"]["safe"],
        json!("kept")
    );

    let recording_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/recordings",
            json!({
                "observation_id": format!("obs-recording-{suffix}"),
                "account_id": account_id,
                "meeting_id": "meeting-123",
                "recording": {
                    "recording_id": "recording-1",
                    "recording_type": "shared_screen_with_speaker_view",
                    "download_ref": "zoom-recording-ref-1",
                    "metadata": {
                        "download_token": "recording-token",
                        "safe": "kept"
                    }
                },
                "metadata": {
                    "webhook_secret": "recording-secret",
                    "safe": "kept"
                }
            }),
        ))
        .await
        .expect("recording bridge response");
    assert_eq!(recording_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(recording_response).await["recording_id"],
        json!("recording-1")
    );
    let recording_payload = event_payload(
        &pool,
        zoom_event_types::RECORDING_OBSERVED,
        "recording_id",
        "recording-1",
    )
    .await;
    assert_secret_like_payload_was_stripped(&recording_payload);
    assert_eq!(
        recording_payload["recording"]["metadata"]["safe"],
        json!("kept")
    );

    let transcript_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/transcripts",
            json!({
                "observation_id": format!("obs-transcript-{suffix}"),
                "transcript_id": format!("zoom-transcript-{suffix}"),
                "account_id": account_id,
                "meeting_id": "meeting-transcript-only",
                "transcript_text": "Decision: keep Zoom foundation integration-only.",
                "segments": [
                    { "start_ms": 0, "end_ms": 1000, "text": "Decision" }
                ],
                "metadata": {
                    "refresh_token": "transcript-token",
                    "safe": "kept"
                }
            }),
        ))
        .await
        .expect("transcript bridge response");
    assert_eq!(transcript_response.status(), StatusCode::OK);
    let transcript_body = json_body(transcript_response).await;
    let transcript_call_id = transcript_body["call_id"]
        .as_str()
        .expect("transcript call id");
    assert!(transcript_call_id.starts_with("zoom_call_"));

    let transcript_row = sqlx::query(
        r#"
        SELECT transcript_text
        FROM call_transcripts
        WHERE transcript_id = $1
        "#,
    )
    .bind(format!("zoom-transcript-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("transcript row");
    assert_eq!(
        transcript_row.get::<String, _>("transcript_text"),
        "Decision: keep Zoom foundation integration-only."
    );
    let placeholder_call_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM telegram_calls
            WHERE call_id = $1
              AND metadata->>'placeholder' = 'true'
        )
        "#,
    )
    .bind(transcript_call_id)
    .fetch_one(&pool)
    .await
    .expect("placeholder call exists");
    assert!(placeholder_call_exists);

    let transcript_payload = event_payload(
        &pool,
        zoom_event_types::TRANSCRIPT_OBSERVED,
        "transcript_id",
        &format!("zoom-transcript-{suffix}"),
    )
    .await;
    assert_secret_like_payload_was_stripped(&transcript_payload);
    assert_eq!(transcript_payload["metadata"]["safe"], json!("kept"));

    assert_bad_request(
        app.clone(),
        "/api/v1/integrations/zoom/runtime-bridge/meetings",
        json!({ "account_id": account_id, "meeting_id": "", "metadata": {} }),
    )
    .await;
    assert_bad_request(
        app.clone(),
        "/api/v1/integrations/zoom/runtime-bridge/meetings",
        json!({ "account_id": "missing-account", "meeting_id": "meeting-404", "metadata": {} }),
    )
    .await;
    assert_bad_request(
        app.clone(),
        "/api/v1/integrations/zoom/runtime-bridge/recordings",
        json!({
            "account_id": account_id,
            "meeting_id": "meeting-123",
            "recording": { "recording_id": "recording-bad" },
            "metadata": []
        }),
    )
    .await;
    assert_bad_request(
        app,
        "/api/v1/integrations/zoom/runtime-bridge/transcripts",
        json!({
            "transcript_id": "bad-transcript",
            "account_id": account_id,
            "meeting_id": "meeting-123",
            "transcript_text": "",
            "segments": []
        }),
    )
    .await;
}

#[tokio::test]
async fn zoom_transcript_file_import_parses_vtt_srt_plain_and_sanitizes_payloads() {
    let (_context, app, pool) = test_app().await;
    let suffix = unique_suffix();
    let account_id = format!("zoom-transcript-import-{suffix}");
    create_fixture_account(&app, &account_id, &suffix.to_string()).await;
    set_zoom_transcript_retention_days(&pool, 21).await;

    let vtt_transcript_id = format!("zoom-vtt-transcript-{suffix}");
    let vtt_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
            json!({
                "observation_id": format!("obs-vtt-transcript-{suffix}"),
                "transcript_id": vtt_transcript_id,
                "account_id": account_id,
                "meeting_id": "meeting-vtt",
                "file_name": "meeting.vtt",
                "content_type": "text/vtt",
                "file_text": "WEBVTT\n\n00:00:01.000 --> 00:00:02.500\n<v Alice>Hello &amp; welcome</v>\n\n00:00:03.000 --> 00:00:04.000\nDecision: ship import.\n",
                "metadata": {
                    "access_token": "transcript-import-token",
                    "safe": "kept"
                }
            }),
        ))
        .await
        .expect("VTT transcript import response");
    assert_eq!(vtt_response.status(), StatusCode::OK);
    let vtt_body = json_body(vtt_response).await;
    assert_eq!(vtt_body["import_format"], json!("webvtt"));
    assert_eq!(vtt_body["parsed_segment_count"], json!(2));

    let vtt_row = sqlx::query(
        r#"
        SELECT transcript_text, segments, provenance
        FROM call_transcripts
        WHERE transcript_id = $1
        "#,
    )
    .bind(format!("zoom-vtt-transcript-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("VTT transcript row");
    assert_eq!(
        vtt_row.get::<String, _>("transcript_text"),
        "Hello & welcome\nDecision: ship import."
    );
    let vtt_segments = vtt_row.get::<Value, _>("segments");
    assert_eq!(vtt_segments[0]["start_ms"], json!(1000));
    assert_eq!(vtt_segments[0]["end_ms"], json!(2500));
    assert_eq!(vtt_segments[0]["text"], json!("Hello & welcome"));
    let vtt_provenance = vtt_row.get::<Value, _>("provenance");
    assert_eq!(
        vtt_provenance["retention_policy"]["retention_days"],
        json!(21)
    );
    assert_eq!(
        vtt_provenance["retention_policy"]["mode"],
        json!("delete_after_n_days")
    );
    assert_eq!(
        vtt_provenance["retention_policy"]["setting_key"],
        json!(ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY)
    );
    assert!(vtt_provenance["retention_policy"]["expires_at"].is_string());

    let vtt_payload = event_payload(
        &pool,
        zoom_event_types::TRANSCRIPT_OBSERVED,
        "transcript_id",
        &format!("zoom-vtt-transcript-{suffix}"),
    )
    .await;
    assert_secret_like_payload_was_stripped(&vtt_payload);
    assert_eq!(
        vtt_payload["metadata"]["transcript_file_import"]["format"],
        json!("webvtt")
    );
    assert_eq!(vtt_payload["metadata"]["metadata"]["safe"], json!("kept"));

    let srt_transcript_id = format!("zoom-srt-transcript-{suffix}");
    let srt_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
            json!({
                "transcript_id": srt_transcript_id,
                "account_id": account_id,
                "meeting_id": "meeting-srt",
                "file_name": "meeting.srt",
                "file_text": "1\n00:00:00,500 --> 00:00:01,000\nFirst SRT cue\n\n2\n00:00:01,500 --> 00:00:02,000\nSecond SRT cue\n"
            }),
        ))
        .await
        .expect("SRT transcript import response");
    assert_eq!(srt_response.status(), StatusCode::OK);
    let srt_body = json_body(srt_response).await;
    assert_eq!(srt_body["import_format"], json!("srt"));
    assert_eq!(srt_body["parsed_segment_count"], json!(2));

    let srt_text: String = sqlx::query_scalar(
        r#"
        SELECT transcript_text
        FROM call_transcripts
        WHERE transcript_id = $1
        "#,
    )
    .bind(format!("zoom-srt-transcript-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("SRT transcript text");
    assert_eq!(srt_text, "First SRT cue\nSecond SRT cue");

    let plain_transcript_id = format!("zoom-plain-transcript-{suffix}");
    let plain_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
            json!({
                "transcript_id": plain_transcript_id,
                "account_id": account_id,
                "meeting_id": "meeting-plain",
                "file_name": "meeting.txt",
                "file_text": "First plain line\n\nSecond plain line"
            }),
        ))
        .await
        .expect("plain transcript import response");
    assert_eq!(plain_response.status(), StatusCode::OK);
    let plain_body = json_body(plain_response).await;
    assert_eq!(plain_body["import_format"], json!("plain_text"));
    assert_eq!(plain_body["parsed_segment_count"], json!(0));

    let plain_row = sqlx::query(
        r#"
        SELECT transcript_text, segments
        FROM call_transcripts
        WHERE transcript_id = $1
        "#,
    )
    .bind(format!("zoom-plain-transcript-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("plain transcript row");
    assert_eq!(
        plain_row.get::<String, _>("transcript_text"),
        "First plain line\nSecond plain line"
    );
    assert_eq!(plain_row.get::<Value, _>("segments"), json!([]));

    assert_bad_request(
        app.clone(),
        "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
        json!({
            "transcript_id": "bad-empty-transcript-file",
            "account_id": account_id,
            "meeting_id": "meeting-bad",
            "file_text": ""
        }),
    )
    .await;
    assert_bad_request(
        app,
        "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
        json!({
            "transcript_id": "bad-timed-transcript-file",
            "account_id": account_id,
            "meeting_id": "meeting-bad",
            "file_text": "00:bad --> 00:00:01.000\nBroken cue"
        }),
    )
    .await;
}

#[tokio::test]
async fn zoom_store_broadcasts_meeting_events_on_runtime_bus() {
    let context = TestContext::new().await;
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let event_bus = InMemoryEventBus::new();
    let store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool),
        event_bus.clone(),
    );
    let suffix = unique_suffix();
    let account_id = format!("zoom-bus-{suffix}");
    store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Bus".to_owned(),
            external_account_id: format!("zoom-bus-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("setup fixture account");

    let mut receiver = event_bus.subscribe();
    let result = store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("obs-bus-{suffix}")),
            account_id,
            meeting_id: "meeting-bus".to_owned(),
            meeting_uuid: None,
            topic: None,
            host_email: None,
            join_url: None,
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({}),
            causation_id: None,
            correlation_id: None,
        })
        .await
        .expect("observe meeting");
    assert_eq!(result.meeting_id, "meeting-bus");

    let broadcast = timeout(Duration::from_secs(2), receiver.recv())
        .await
        .expect("broadcast timeout")
        .expect("broadcast event");
    assert_eq!(broadcast.event_type, zoom_event_types::MEETING_OBSERVED);
    assert_eq!(broadcast.payload["meeting_id"], json!("meeting-bus"));
}

async fn test_app_with_zoom_webhook_secret(
    account_id: &str,
    secret_ref: &str,
    webhook_secret: &str,
) -> (TestContext, axum::Router, PgPool) {
    let context = TestContext::new().await;
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    seed_host_vault_secret_ref(
        &context,
        &pool,
        account_id,
        secret_ref,
        webhook_secret,
        ProviderAccountSecretPurpose::ZoomWebhookSecret,
    )
    .await;
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    let response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Webhook",
                "external_account_id": format!("{account_id}-external"),
                "auth_shape": "server_to_server",
                "client_id": "zoom-webhook-client",
                "webhook_secret_ref": secret_ref,
                "metadata": { "test": "webhook" }
            }),
        ))
        .await
        .expect("create webhook account response");
    assert_eq!(response.status(), StatusCode::OK);
    (context, app, pool)
}

async fn test_app() -> (TestContext, axum::Router, PgPool) {
    let context = TestContext::new().await;
    let database_url = context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let app = build_router_with_database(context.app_config(LOCAL_API_TOKEN), database);
    (context, app, pool)
}

fn initialize_host_vault(context: &TestContext) {
    let vault = HostVault::new(HostVaultConfig {
        home: context.vault_home().to_path_buf(),
        dev_mode: true,
        dev_key_path: context.dev_key_path().to_path_buf(),
    })
    .expect("host vault");
    vault
        .collect_entropy(host_vault_entropy_events(2_000))
        .expect("collect host vault entropy");
    vault.create().expect("create host vault");
}

fn unlocked_test_vault(context: &TestContext) -> HostVault {
    let vault = HostVault::new(HostVaultConfig {
        home: context.vault_home().to_path_buf(),
        dev_mode: true,
        dev_key_path: context.dev_key_path().to_path_buf(),
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
}

async fn spawn_zoom_token_server(
    response_body: Value,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Zoom token server");
    let address = listener.local_addr().expect("token server address");
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept token request");
        let mut buffer = Vec::new();
        let mut temp = [0_u8; 1024];
        loop {
            let read = stream.read(&mut temp).await.expect("read token request");
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);
            if request_body_complete(&buffer) {
                break;
            }
        }
        let request_text = String::from_utf8_lossy(&buffer).to_string();
        let response = response_body.to_string();
        let http_response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            response.len(),
            response
        );
        stream
            .write_all(http_response.as_bytes())
            .await
            .expect("write token response");
        request_text
    });
    (format!("http://{address}/oauth/token"), handle)
}

async fn spawn_zoom_token_server_sequence(
    response_bodies: Vec<Value>,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Zoom token server");
    let address = listener.local_addr().expect("token server address");
    let handle = tokio::spawn(async move {
        let mut requests = Vec::with_capacity(response_bodies.len());
        for response_body in response_bodies {
            let (mut stream, _) = listener.accept().await.expect("accept token request");
            let mut buffer = Vec::new();
            let mut temp = [0_u8; 1024];
            loop {
                let read = stream.read(&mut temp).await.expect("read token request");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp[..read]);
                if request_body_complete(&buffer) {
                    break;
                }
            }
            let request_text = String::from_utf8_lossy(&buffer).to_string();
            let response = response_body.to_string();
            let http_response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response.len(),
                response
            );
            stream
                .write_all(http_response.as_bytes())
                .await
                .expect("write token response");
            requests.push(request_text);
        }
        requests
    });
    (format!("http://{address}/oauth/token"), handle)
}

async fn spawn_zoom_transcript_download_server(
    response_body: &str,
    content_type: &str,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Zoom transcript download server");
    let address = listener
        .local_addr()
        .expect("transcript download server address");
    let response_body = response_body.to_owned();
    let content_type = content_type.to_owned();
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept transcript download");
        let mut buffer = Vec::new();
        let mut temp = [0_u8; 1024];
        loop {
            let read = stream
                .read(&mut temp)
                .await
                .expect("read transcript download");
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);
            if request_body_complete(&buffer) {
                break;
            }
        }
        let request_text = String::from_utf8_lossy(&buffer).to_string();
        let http_response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(http_response.as_bytes())
            .await
            .expect("write transcript download response");
        request_text
    });
    (
        format!("http://{address}/recordings/transcript.vtt"),
        handle,
    )
}

async fn spawn_zoom_media_download_server(
    response_body: &[u8],
    content_type: &str,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Zoom media download server");
    let address = listener
        .local_addr()
        .expect("media download server address");
    let response_body = response_body.to_vec();
    let content_type = content_type.to_owned();
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept media download");
        let mut buffer = Vec::new();
        let mut temp = [0_u8; 1024];
        loop {
            let read = stream.read(&mut temp).await.expect("read media download");
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);
            if request_body_complete(&buffer) {
                break;
            }
        }
        let request_text = String::from_utf8_lossy(&buffer).to_string();
        let headers = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            response_body.len()
        );
        stream
            .write_all(headers.as_bytes())
            .await
            .expect("write media response headers");
        stream
            .write_all(&response_body)
            .await
            .expect("write media response body");
        request_text
    });
    (format!("http://{address}/recordings/video.mp4"), handle)
}

async fn spawn_zoom_recording_sync_server_with_limit(
    request_count: usize,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Zoom recording sync server");
    let address = listener
        .local_addr()
        .expect("recording sync server address");
    let handle = tokio::spawn(async move {
        let mut requests = Vec::with_capacity(request_count);
        for index in 0..request_count {
            let (mut stream, _) = listener
                .accept()
                .await
                .expect("accept recording sync request");
            let mut buffer = Vec::new();
            let mut temp = [0_u8; 1024];
            loop {
                let read = stream
                    .read(&mut temp)
                    .await
                    .expect("read recording sync request");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp[..read]);
                if request_body_complete(&buffer) {
                    break;
                }
            }
            let request_text = String::from_utf8_lossy(&buffer).to_string();
            let (content_type, response_body) = if index == 0 {
                (
                    "application/json",
                    json!({
                        "meetings": [
                            {
                                "id": "9988776655",
                                "uuid": "provider-sync-meeting-uuid",
                                "topic": "Provider Sync Review",
                                "host_email": "host@example.test",
                                "start_time": "2026-06-27T14:00:00Z",
                                "duration": 45,
                                "recording_files": [
                                    {
                                        "id": "recording-mp4-1",
                                        "file_type": "MP4",
                                        "recording_type": "shared_screen_with_speaker_view",
                                        "download_url": format!("http://{address}/download/video-1.mp4"),
                                        "file_extension": "MP4",
                                        "file_size": 123456,
                                        "recording_start": "2026-06-27T14:00:00Z"
                                    },
                                    {
                                        "id": "recording-transcript-1",
                                        "file_type": "TRANSCRIPT",
                                        "recording_type": "audio_transcript",
                                        "download_url": format!("http://{address}/download/transcript-1.vtt"),
                                        "file_extension": "VTT",
                                        "file_size": 4567,
                                        "recording_start": "2026-06-27T14:00:00Z"
                                    }
                                ]
                            }
                        ],
                        "next_page_token": ""
                    })
                    .to_string(),
                )
            } else if request_text.contains("/download/video-1.mp4") {
                ("video/mp4", "fake-zoom-mp4-data".to_owned())
            } else {
                (
                    "text/vtt",
                    "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nProvider sync transcript line.\n"
                        .to_owned(),
                )
            };
            let http_response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(http_response.as_bytes())
                .await
                .expect("write recording sync response");
            requests.push(request_text);
        }
        requests
    });
    (format!("http://{address}"), handle)
}

async fn spawn_zoom_webhook_subscription_server_sequence(
    responses: Vec<(u16, &'static str, String)>,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Zoom webhook subscription server");
    let address = listener
        .local_addr()
        .expect("Zoom webhook subscription server address");
    let handle = tokio::spawn(async move {
        let mut requests = Vec::with_capacity(responses.len());
        for (status_code, content_type, response_body) in responses {
            let (mut stream, _) = listener
                .accept()
                .await
                .expect("accept Zoom webhook subscription request");
            let mut buffer = Vec::new();
            let mut temp = [0_u8; 1024];
            loop {
                let read = stream
                    .read(&mut temp)
                    .await
                    .expect("read Zoom webhook subscription request");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp[..read]);
                if request_body_complete(&buffer) {
                    break;
                }
            }
            let request_text = String::from_utf8_lossy(&buffer).to_string();
            let status_text = match status_code {
                200 => "OK",
                201 => "Created",
                204 => "No Content",
                400 => "Bad Request",
                404 => "Not Found",
                _ => "OK",
            };
            let http_response = if response_body.is_empty() {
                format!(
                    "HTTP/1.1 {status_code} {status_text}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                )
            } else {
                format!(
                    "HTTP/1.1 {status_code} {status_text}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                )
            };
            stream
                .write_all(http_response.as_bytes())
                .await
                .expect("write Zoom webhook subscription response");
            requests.push(request_text);
        }
        requests
    });
    (format!("http://{address}"), handle)
}

fn request_body_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = find_header_end(buffer) else {
        return false;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    buffer.len() >= header_end + 4 + content_length
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

async fn secret_references_for(
    pool: &PgPool,
    secret_refs: &[&str],
) -> Vec<(String, String, String)> {
    let rows = sqlx::query(
        r#"
        SELECT secret_ref, secret_kind, store_kind
        FROM secret_references
        WHERE secret_ref = ANY($1)
        ORDER BY secret_ref ASC
        "#,
    )
    .bind(secret_refs)
    .fetch_all(pool)
    .await
    .expect("secret references");
    rows.into_iter()
        .map(|row| {
            (
                row.get::<String, _>("secret_ref"),
                row.get::<String, _>("secret_kind"),
                row.get::<String, _>("store_kind"),
            )
        })
        .collect()
}

async fn assert_provider_secret_binding(
    pool: &PgPool,
    account_id: &str,
    purpose: &str,
    secret_ref: &str,
) {
    let stored: String = sqlx::query_scalar(
        r#"
        SELECT secret_ref
        FROM communication_provider_account_secret_refs
        WHERE account_id = $1
          AND secret_purpose = $2
        "#,
    )
    .bind(account_id)
    .bind(purpose)
    .fetch_one(pool)
    .await
    .expect("provider secret binding");
    assert_eq!(stored, secret_ref);
}

async fn seed_host_vault_secret_ref(
    context: &TestContext,
    pool: &PgPool,
    account_id: &str,
    secret_ref: &str,
    value: &str,
    purpose: ProviderAccountSecretPurpose,
) {
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(
            &NewSecretReference::new(
                secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "Zoom webhook secret",
            )
            .metadata(json!({
                "provider": "zoom",
                "account_id": account_id,
                "purpose": purpose.as_str(),
                "test": true,
            })),
        )
        .await
        .expect("seed host vault secret reference");
    let vault = HostVault::new(HostVaultConfig {
        home: context.vault_home().to_path_buf(),
        dev_mode: true,
        dev_key_path: context.dev_key_path().to_path_buf(),
    })
    .expect("host vault");
    if matches!(
        vault.status().expect("host vault status").state,
        VaultMode::Uninitialized
    ) {
        vault
            .collect_entropy(host_vault_entropy_events(2_000))
            .expect("collect host vault entropy");
        vault.create().expect("create host vault");
    } else {
        vault.unlock_existing().expect("unlock existing host vault");
    }
    vault
        .store_secret(
            secret_ref,
            value,
            SecretEntryContext {
                entry_kind: "provider_webhook_secret",
                account_id,
                purpose: purpose.as_str(),
                secret_kind: "api_token",
                label: "Zoom webhook secret",
                metadata: &json!({
                    "provider": "zoom",
                    "test": true,
                }),
            },
        )
        .expect("store Zoom webhook secret");
}

async fn create_fixture_account(app: &axum::Router, account_id: &str, suffix: &str) {
    let response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/zoom/fixtures/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zoom Bridge",
                "external_account_id": format!("zoom-bridge-external-{suffix}"),
                "metadata": {}
            }),
        ))
        .await
        .expect("fixture account response");
    assert_eq!(response.status(), StatusCode::OK);
}

async fn enable_zoom_remote_transcript_downloads(pool: &PgPool) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY,
            &json!(true),
            "zoom-provider-test",
        )
        .await
        .expect("enable zoom remote transcript downloads");
}

async fn enable_zoom_remote_recording_downloads(pool: &PgPool) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "privacy.zoom_remote_recording_download_enabled",
            &json!(true),
            "zoom-provider-test",
        )
        .await
        .expect("enable zoom remote recording downloads");
}

async fn set_zoom_recording_import_retention_days(pool: &PgPool, days: i64) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            ZOOM_RECORDING_IMPORT_RETENTION_DAYS_SETTING_KEY,
            &json!(days),
            "zoom-provider-test",
        )
        .await
        .expect("set zoom recording import retention days");
}

async fn set_zoom_transcript_retention_days(pool: &PgPool, days: i64) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY,
            &json!(days),
            "zoom-provider-test",
        )
        .await
        .expect("set zoom transcript retention days");
}

async fn force_zoom_recording_import_retention_expiry(pool: &PgPool, attachment_id: &str) {
    sqlx::query(
        r#"
        UPDATE communication_attachment_imports
        SET metadata = jsonb_set(
            COALESCE(metadata, '{}'::jsonb),
            '{retention_policy,expires_at}',
            to_jsonb('2020-01-01T00:00:00Z'::text),
            true
        )
        WHERE attachment_id = $1
        "#,
    )
    .bind(attachment_id)
    .execute(pool)
    .await
    .expect("force recording import retention expiry");
}

async fn force_zoom_transcript_retention_expiry(pool: &PgPool, transcript_id: &str) {
    sqlx::query(
        r#"
        UPDATE call_transcripts
        SET provenance = jsonb_set(
            COALESCE(provenance, '{}'::jsonb),
            '{retention_policy,expires_at}',
            to_jsonb('2020-01-01T00:00:00Z'::text),
            true
        )
        WHERE transcript_id = $1
        "#,
    )
    .bind(transcript_id)
    .execute(pool)
    .await
    .expect("force transcript retention expiry");
}

async fn seed_secret_ref(pool: &PgPool, secret_ref: &str, secret_kind: SecretKind) {
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(
            &NewSecretReference::new(
                secret_ref,
                secret_kind,
                SecretStoreKind::TestDouble,
                secret_ref,
            )
            .metadata(json!({
                "provider": "zoom",
                "test": true,
            })),
        )
        .await
        .expect("seed secret reference");
}

fn zoom_signed_post(uri: &str, body: &Value, webhook_secret: &str) -> Request<Body> {
    let body = body.to_string();
    let timestamp = Utc::now().timestamp().to_string();
    let signature = zoom_webhook_signature(webhook_secret, &timestamp, body.as_bytes());
    signed_zoom_request(uri, body, &timestamp, &signature)
}

fn zoom_signed_post_with_signature(uri: &str, body: &Value, signature: &str) -> Request<Body> {
    signed_zoom_request(
        uri,
        body.to_string(),
        &Utc::now().timestamp().to_string(),
        signature,
    )
}

fn signed_zoom_request(uri: &str, body: String, timestamp: &str, signature: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header("x-zm-request-timestamp", timestamp)
        .header("x-zm-signature", signature)
        .body(Body::from(body))
        .expect("signed Zoom webhook request")
}

fn zoom_webhook_signature(webhook_secret: &str, timestamp: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(webhook_secret.as_bytes()).expect("test webhook secret HMAC");
    mac.update(b"v0:");
    mac.update(timestamp.as_bytes());
    mac.update(b":");
    mac.update(body);
    format!("v0={}", bytes_to_lower_hex(&mac.finalize().into_bytes()))
}

fn zoom_validation_token(webhook_secret: &str, plain_token: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(webhook_secret.as_bytes()).expect("test webhook secret HMAC");
    mac.update(plain_token.as_bytes());
    bytes_to_lower_hex(&mac.finalize().into_bytes())
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn host_vault_entropy_events(count: usize) -> Vec<EntropyEvent> {
    (0..count)
        .map(|index| EntropyEvent {
            x: (index % 977) as f64,
            y: (index % 541) as f64,
            dx: ((index % 13) as f64) - 6.0,
            dy: ((index % 17) as f64) - 8.0,
            timestamp_ms: index as f64 * 7.0,
            velocity: (index % 29) as f64 / 10.0,
            acceleration: (index % 31) as f64 / 100.0,
            interval_ms: 7.0,
        })
        .collect()
}

async fn provider_secret_bindings(pool: &PgPool, account_id: &str) -> Vec<(String, String)> {
    sqlx::query(
        r#"
        SELECT secret_purpose, secret_ref
        FROM communication_provider_account_secret_refs
        WHERE account_id = $1
        ORDER BY secret_purpose
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("provider secret bindings")
    .into_iter()
    .map(|row| {
        (
            row.get::<String, _>("secret_purpose"),
            row.get::<String, _>("secret_ref"),
        )
    })
    .collect()
}

async fn event_payload(
    pool: &PgPool,
    event_type: &str,
    subject_key: &str,
    subject_value: &str,
) -> Value {
    sqlx::query(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = $1
          AND subject->>$2 = $3
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(event_type)
    .bind(subject_key)
    .bind(subject_value)
    .fetch_one(pool)
    .await
    .expect("event payload")
    .get("payload")
}

async fn zoom_audit_events(app: axum::Router, account_id: &str, limit: i64) -> Vec<Value> {
    let response = app
        .oneshot(get(&format!(
            "/api/v1/integrations/zoom/accounts/{account_id}/audit-events?limit={limit}"
        )))
        .await
        .expect("zoom audit events response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response)
        .await
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn find_zoom_audit_event<'a>(items: &'a [Value], event_type: &str) -> Option<&'a Value> {
    items
        .iter()
        .find(|item| item["event_type"] == json!(event_type))
}

fn assert_secret_like_payload_was_stripped(payload: &Value) {
    let serialized = payload.to_string();
    for forbidden in [
        "access_token",
        "refresh_token",
        "download_token",
        "client_secret",
        "webhook_secret",
        "participant-token",
        "meeting-token",
        "recording-token",
        "transcript-token",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "payload leaked forbidden token-like field/value `{forbidden}`: {serialized}"
        );
    }
}

async fn assert_bad_request(app: axum::Router, uri: &str, body: Value) {
    let response = app
        .oneshot(json_post(uri, body))
        .await
        .expect("bad request response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

fn json_post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&body).expect("json response")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos()
}
