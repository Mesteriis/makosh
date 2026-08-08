use serde_json::json;
use tempfile::tempdir;
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::ai::control_center::{
    models::AiProviderAccount, store::AiControlCenterStore,
};
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::secrets::{
    models::SecretKind, resolver::SecretResolver, store::SecretReferenceStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::{HostVault, models::HostVaultConfig};

use super::support::{
    LOCAL_API_TOKEN, json_body, json_request_with_token_and_actor, unlock_test_vault,
    wait_for_manifest_metadata_key, wait_for_secret_reference,
};

#[tokio::test]
async fn startup_reconciles_ai_api_provider_from_host_vault_after_postgres_metadata_wipe() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_pairs([
        ("MAKOSH_DEV_MODE", "true"),
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app.clone()).await;

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/ai/providers",
            json!({
                "provider_kind": "api",
                "provider_key": "omniroute",
                "display_name": "Recovered OmniRoute",
                "base_url": "https://ai.sh-inc.ru/v1",
                "capabilities": ["chat", "reasoning", "embeddings"],
                "enabled": true,
                "remote_context_consent": true,
                "api_key": "omniroute-api-key"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = json_body(response).await;
    let provider_id = body["provider_id"]
        .as_str()
        .expect("provider id")
        .to_owned();
    let secret_ref = format!("secret:ai-provider:{provider_id}:api_key");

    let pool = database.pool().expect("configured pool").clone();
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    wait_for_manifest_metadata_key(&vault, &secret_ref, "provider_key").await;

    sqlx::query("DELETE FROM ai_provider_accounts WHERE provider_id = $1")
        .bind(&provider_id)
        .execute(&pool)
        .await
        .expect("delete ai provider metadata");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(&secret_ref)
        .execute(&pool)
        .await
        .expect("delete ai secret reference");

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let ai_store = AiControlCenterStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool.clone());

    let provider = wait_for_ai_provider(&ai_store, &provider_id).await;
    assert_eq!(provider.provider_kind, "api");
    assert_eq!(provider.provider_key, "omniroute");
    assert_eq!(provider.display_name, "Recovered OmniRoute");
    assert_eq!(provider.status, "ready");
    assert_eq!(provider.consent_state, "granted");
    assert_eq!(
        provider.config["base_url"],
        json!("https://ai.sh-inc.ru/v1")
    );
    assert!(provider.capabilities.contains(&"chat".to_owned()));

    assert_eq!(
        ai_store
            .api_key_secret_ref(&provider_id)
            .await
            .expect("ai api key ref"),
        Some(secret_ref.clone())
    );
    let reference = wait_for_secret_reference(&secret_store, &secret_ref).await;
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored ai secret")
            .expose_for_runtime(),
        "omniroute-api-key"
    );
    wait_for_ai_model_catalog(&restarted_pool, &provider_id).await;
}

async fn wait_for_ai_provider(
    store: &AiControlCenterStore,
    provider_id: &str,
) -> AiProviderAccount {
    for _ in 0..50 {
        if let Some(provider) = store.provider(provider_id).await.expect("load ai provider") {
            return provider;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("AI provider {provider_id} was not reconciled");
}

async fn wait_for_ai_model_catalog(pool: &sqlx::PgPool, provider_id: &str) {
    for _ in 0..50 {
        let model_count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM ai_model_catalog WHERE provider_id = $1")
                .bind(provider_id)
                .fetch_one(pool)
                .await
                .expect("model count");
        if model_count > 0 {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("AI provider {provider_id} model catalog was not seeded");
}
