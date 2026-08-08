use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;

use makosh_hub_backend::platform::settings::models::SettingValueKind;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "settings-api-test-token";

static SETTINGS_DB_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn application_settings_store_lists_seeded_settings_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    let settings = store.list_settings().await.expect("list settings");

    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "ai.chat_model")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "server.http_addr")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.layout")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.sidebar")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.theme")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.locale")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.ui_state")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "ui.theme")
    );
    assert!(
        settings
            .iter()
            .all(|setting| !setting.setting_key.contains("password"))
    );

    let public_settings = store
        .list_public_settings()
        .await
        .expect("list public settings");
    assert!(
        public_settings
            .iter()
            .all(|setting| setting.category != "ai" && !setting.setting_key.starts_with("ai."))
    );
    assert!(
        public_settings
            .iter()
            .any(|setting| setting.setting_key == "ui.theme")
    );
}

#[tokio::test]
async fn application_settings_include_frontend_layout_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let layout_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.layout")
        .expect("frontend layout setting");

    assert_eq!(layout_setting.category, "frontend");
    assert_eq!(layout_setting.value_kind, SettingValueKind::Json);
    assert_eq!(layout_setting.value["schemaVersion"], json!(2));
    assert!(layout_setting.value["views"].is_object());
    assert!(layout_setting.is_editable);
}

#[tokio::test]
async fn application_settings_include_frontend_sidebar_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let sidebar_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.sidebar")
        .expect("frontend sidebar setting");

    assert_eq!(sidebar_setting.category, "frontend");
    assert_eq!(sidebar_setting.value_kind, SettingValueKind::Json);
    assert_eq!(sidebar_setting.metadata["schema_version"], json!(3));
    assert!(sidebar_setting.value["groups"].is_array());
    assert!(sidebar_setting.value["hiddenItemIds"].is_array());
    if sidebar_setting.value["schemaVersion"] == json!(3) {
        assert!(sidebar_setting.value["rootItemIds"].is_array());
        assert_eq!(
            sidebar_setting.value["rootItemIds"][1],
            json!("group:communications")
        );
        assert_eq!(
            sidebar_setting.value["groups"][0]["itemIds"][0],
            json!("communications.mail")
        );
        assert_eq!(
            sidebar_setting.value["groups"][0]["separatorBeforeItemIds"],
            json!([])
        );
    }
    assert!(sidebar_setting.is_editable);
}

#[tokio::test]
async fn application_settings_include_frontend_theme_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let theme_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.theme")
        .expect("frontend theme setting");

    assert_eq!(theme_setting.category, "frontend");
    assert_eq!(theme_setting.value_kind, SettingValueKind::Json);
    assert_eq!(theme_setting.metadata["schema_version"], json!(1));
    assert_eq!(theme_setting.value["schemaVersion"], json!(1));
    assert_eq!(
        theme_setting.value["shellBackground"],
        json!("network-mesh")
    );
    assert_eq!(theme_setting.value["backgroundBrightness"], json!(70));
    assert_eq!(theme_setting.value["accentColor"], json!("teal"));
    assert_eq!(theme_setting.value["panelOpacity"], json!(70));
    assert_eq!(theme_setting.value["panelBlur"], json!(12));
    assert!(theme_setting.is_editable);
}

#[tokio::test]
async fn application_settings_include_frontend_ui_state_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let ui_state_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.ui_state")
        .expect("frontend ui state setting");

    assert_eq!(ui_state_setting.category, "frontend");
    assert_eq!(ui_state_setting.value_kind, SettingValueKind::Json);
    assert_eq!(ui_state_setting.metadata["ui_control"], json!("hidden"));
    assert_eq!(ui_state_setting.metadata["schema_version"], json!(1));
    assert_eq!(
        ui_state_setting.metadata["stores_private_content"],
        json!(false)
    );
    assert_eq!(ui_state_setting.value["schemaVersion"], json!(1));
    assert!(ui_state_setting.is_editable);
}

#[tokio::test]
async fn application_settings_update_repairs_missing_declared_setting_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ApplicationSettingsStore::new(pool.clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    sqlx::query("DELETE FROM application_settings WHERE setting_key = 'frontend.theme'")
        .execute(&pool)
        .await
        .expect("delete declared setting");

    let updated = store
        .update_setting_value(
            "frontend.theme",
            &json!({
                "schemaVersion": 1,
                "shellBackground": "forest-network",
                "backgroundBrightness": 60,
                "accentColor": "cyan",
                "panelOpacity": 80,
                "panelBlur": 16
            }),
            "settings-test",
        )
        .await
        .expect("update repairs missing declared setting");

    assert_eq!(updated.setting_key, "frontend.theme");
    assert_eq!(updated.value["shellBackground"], json!("forest-network"));
    assert_eq!(
        updated.updated_by_actor_id.as_deref(),
        Some("settings-test")
    );

    let restored = store
        .setting("frontend.theme")
        .await
        .expect("fetch repaired setting")
        .expect("frontend theme setting restored");
    assert_eq!(restored.value["accentColor"], json!("cyan"));

    store
        .update_setting_value(
            "frontend.theme",
            &json!({
                "schemaVersion": 1,
                "shellBackground": "network-mesh",
                "backgroundBrightness": 70,
                "accentColor": "teal",
                "panelOpacity": 70,
                "panelBlur": 12
            }),
            "settings-test",
        )
        .await
        .expect("restore frontend theme default");
}

#[tokio::test]
async fn database_startup_repairs_declared_application_settings_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    sqlx::query("DELETE FROM application_settings WHERE setting_key = 'frontend.api_base_url'")
        .execute(&pool)
        .await
        .expect("delete declared setting");
    sqlx::query(
        r#"
        UPDATE application_settings
        SET
            value = '"broken"'::jsonb,
            label = 'Broken density',
            metadata = '{}'::jsonb
        WHERE setting_key = 'ui.density'
        "#,
    )
    .execute(&pool)
    .await
    .expect("corrupt declared setting");
    sqlx::query(
        r#"
        INSERT INTO application_settings (
            setting_key,
            category,
            value_kind,
            value,
            label,
            description,
            metadata
        )
        VALUES (
            'custom.unexpected',
            'custom',
            'string',
            '"manual"'::jsonb,
            'Manual custom setting',
            'This row must not become a supported setting surface.',
            '{}'::jsonb
        )
        ON CONFLICT (setting_key) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert undeclared setting");

    drop(pool);
    drop(database);

    let repaired_database = Database::connect(Some(&database_url))
        .await
        .expect("database reconnect repairs settings");
    let store =
        ApplicationSettingsStore::new(repaired_database.pool().expect("configured pool").clone());
    let settings = store.list_settings().await.expect("list repaired settings");

    let api_base_url_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.api_base_url")
        .expect("frontend API base URL setting restored");
    assert_eq!(api_base_url_setting.value, json!("http://127.0.0.1:8080"));
    assert_eq!(
        api_base_url_setting.updated_by_actor_id.as_deref(),
        Some("system:settings_repair")
    );

    let density_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "ui.density")
        .expect("UI density setting restored");
    assert_eq!(density_setting.label, "UI density");
    assert_eq!(density_setting.value, json!("comfortable"));
    assert_eq!(
        density_setting.updated_by_actor_id.as_deref(),
        Some("system:settings_repair")
    );
    assert!(density_setting.metadata.get("allowed_values").is_some());
    assert!(
        settings
            .iter()
            .all(|setting| setting.setting_key != "custom.unexpected")
    );
}

#[tokio::test]
async fn application_settings_api_updates_existing_setting_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            "/api/v1/settings/ui.theme",
            json!({ "value": "dark" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["setting_key"], json!("ui.theme"));
    assert_eq!(body["value"], json!("dark"));
    assert_eq!(body["updated_by_actor_id"], json!("makosh-frontend"));

    let list_response = app
        .clone()
        .oneshot(get_request_with_token("/api/v1/settings", LOCAL_API_TOKEN))
        .await
        .expect("list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = json_body(list_response).await;
    let items = list_body["items"].as_array().expect("settings items");
    assert!(items.iter().all(|item| {
        item["category"] != json!("ai")
            && item["setting_key"]
                .as_str()
                .is_none_or(|key| !key.starts_with("ai."))
    }));
    assert!(items.iter().any(|item| {
        item["setting_key"] == json!("ui.theme") && item["value"] == json!("dark")
    }));

    let _ = app
        .clone()
        .oneshot(json_put_request_with_actor(
            "/api/v1/settings/ui.theme",
            json!({ "value": "system" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
}

#[tokio::test]
async fn application_settings_api_rejects_secret_like_setting_keys_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            "/api/v1/settings/mail.password",
            json!({ "value": "not-allowed" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_application_setting"));
}

#[tokio::test]
async fn application_settings_api_rejects_private_ui_state_payload_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            "/api/v1/settings/frontend.ui_state",
            json!({
                "value": {
                    "schemaVersion": 1,
                    "savedAt": "2026-06-11T12:00:00Z",
                    "expiresAt": "2026-06-18T12:00:00Z",
                    "communications": {
                        "compose": {
                            "draftId": "draft-1",
                            "body": "private draft body"
                        }
                    }
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_application_setting"));
}

#[tokio::test]
async fn settings_accounts_api_lists_provider_accounts_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_settings_accounts_{suffix}");
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Icloud,
            "Settings iCloud account",
            format!("settings-{suffix}@icloud.com"),
        ))
        .await
        .expect("seed provider account");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/settings/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("account items");
    assert!(items.iter().any(|item| {
        item["account_id"] == json!(account_id) && item["provider_kind"] == json!("icloud")
    }));
}

#[tokio::test]
async fn settings_accounts_api_updates_provider_account_label_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_settings_label_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Icloud,
            "Original iCloud label",
            format!("label-{suffix}@icloud.com"),
        ))
        .await
        .expect("seed provider account");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_patch_request_with_actor(
            format!("/api/v1/settings/accounts/{account_id}").as_str(),
            json!({
                "display_name": "Personal iCloud"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["display_name"], json!("Personal iCloud"));

    let observation = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'vault'
          AND link.entity_kind = 'communication_provider_account'
          AND link.entity_id = $1
          AND link.relationship_kind = 'display_name_update'
        ORDER BY link.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("display name observation");

    assert_eq!(
        observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT_DISPLAY_NAME_MUTATION"
    );
    assert_eq!(
        observation
            .try_get::<String, _>("relationship_kind")
            .expect("relationship kind"),
        "display_name_update"
    );
}

#[tokio::test]
async fn settings_accounts_api_updates_address_book_sync_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_settings_contacts_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::Gmail,
                "Gmail contacts account",
                format!("contacts-{suffix}@gmail.com"),
            )
            .config(json!({
                "connected_services": ["mail", "calendar", "contacts"],
                "address_book_sync_enabled": false,
                "requested_scopes": ["https://www.googleapis.com/auth/contacts"]
            })),
        )
        .await
        .expect("seed provider account");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_patch_request_with_actor(
            format!("/api/v1/settings/accounts/{account_id}").as_str(),
            json!({
                "address_book_sync_enabled": true,
                "address_book_sync_direction": "bidirectional",
                "address_book_remote_write_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["config"]["address_book_sync_enabled"], json!(true));
    assert_eq!(
        body["config"]["address_book_sync_direction"],
        json!("bidirectional")
    );
    assert_eq!(
        body["config"]["address_book_remote_write_enabled"],
        json!(true)
    );

    let stored_config = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT config
        FROM communication_provider_accounts
        WHERE account_id = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("stored config");

    assert_eq!(stored_config["address_book_sync_enabled"], json!(true));
    assert_eq!(
        stored_config["address_book_sync_direction"],
        json!("bidirectional")
    );
    assert_eq!(
        stored_config["address_book_remote_write_enabled"],
        json!(true)
    );
}

#[tokio::test]
async fn settings_accounts_api_rejects_address_book_sync_without_contacts_capability() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_settings_no_contacts_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::Imap,
                "IMAP mail account",
                format!("imap-{suffix}@example.com"),
            )
            .config(json!({
                "connected_services": ["mail"],
                "address_book_sync_enabled": false
            })),
        )
        .await
        .expect("seed provider account");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_patch_request_with_actor(
            format!("/api/v1/settings/accounts/{account_id}").as_str(),
            json!({
                "address_book_sync_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_communication_query"));
}

#[tokio::test]
async fn settings_accounts_api_rejects_address_book_remote_write_without_scope() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_settings_contacts_readonly_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::Gmail,
                "Gmail contacts readonly account",
                format!("contacts-readonly-{suffix}@gmail.com"),
            )
            .config(json!({
                "connected_services": ["mail", "contacts"],
                "address_book_sync_enabled": true,
                "requested_scopes": ["https://www.googleapis.com/auth/contacts.readonly"]
            })),
        )
        .await
        .expect("seed provider account");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_patch_request_with_actor(
            format!("/api/v1/settings/accounts/{account_id}").as_str(),
            json!({
                "address_book_sync_direction": "bidirectional",
                "address_book_remote_write_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_communication_query"));
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request_with_actor(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn json_patch_request_with_actor(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PATCH")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
