use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::mail_resources::{
    MailProviderResourceKind, MailProviderSemanticRole,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::provider_resources::{
    MailProviderResourceStore, NewMailProviderResource,
};

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const TOKEN: &str = "mail-provider-resource-api-test-token";

async fn app(context: &TestContext) -> axum::Router {
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            TOKEN,
            context.connection_string(),
        ),
        database,
    )
}

fn request(method: Method, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TOKEN);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("JSON body")
}

#[tokio::test]
async fn account_scoped_resource_mapping_api_lists_and_manually_maps_discovered_resources() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-resource-api";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Imap,
            "Resource API fixture",
            "resource-api@example.test",
        ))
        .await
        .expect("provider account");
    let store = MailProviderResourceStore::new(pool);
    let discovered = store
        .upsert_discovered(
            &NewMailProviderResource::new(
                account_id,
                MailProviderResourceKind::Folder,
                "Sent Messages",
                "Sent Messages",
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("discovered resource");
    CommunicationProviderAccountStore::new(context.pool().clone())
        .upsert(&NewProviderAccount::new(
            "mail-resource-api-other",
            CommunicationProviderKind::Imap,
            "Other resource API fixture",
            "other-resource-api@example.test",
        ))
        .await
        .expect("other provider account");
    let other = store
        .upsert_discovered(
            &NewMailProviderResource::new(
                "mail-resource-api-other",
                MailProviderResourceKind::Folder,
                "Other Sent",
                "Other Sent",
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("other discovered resource");
    let app = app(&context).await;

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/integrations/mail/accounts/{account_id}/provider-resources"),
            None,
        ))
        .await
        .expect("list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["items"][0]["mapping_id"], discovered.mapping_id);
    assert_eq!(body["items"][0]["mapping_source"], "discovered");

    let response = app
        .clone()
        .oneshot(request(
            Method::PUT,
            &format!(
                "/api/v1/integrations/mail/accounts/{account_id}/provider-resources/{}",
                discovered.mapping_id
            ),
            Some(json!({ "semantic_role": "sent", "local_folder_id": null })),
        ))
        .await
        .expect("mapping response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["mapping_source"], "manual");
    assert_eq!(body["semantic_role"], "sent");

    let response = app
        .oneshot(request(
            Method::PUT,
            &format!(
                "/api/v1/integrations/mail/accounts/{account_id}/provider-resources/{}",
                other.mapping_id
            ),
            Some(json!({ "semantic_role": "sent", "local_folder_id": null })),
        ))
        .await
        .expect("cross-account mapping response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
