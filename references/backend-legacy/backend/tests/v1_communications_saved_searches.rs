use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::states::WorkflowState;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-saved-search-test-token";

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

fn request(method: Method, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", T);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(Body::from(
            body.map_or_else(String::new, |value| value.to_string()),
        ))
        .expect("request")
}

#[tokio::test]
async fn v1_saved_searches_crud_and_events_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-saved-search-{suffix}");
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-invoice-overdue-{suffix}"),
        "Invoice overdue",
        "The invoice is overdue and needs review",
        WorkflowState::NeedsAction,
    )
    .await;
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-waiting-invoice-{suffix}"),
        "Invoice waiting",
        "The invoice is waiting on a vendor",
        WorkflowState::Waiting,
    )
    .await;
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-unrelated-{suffix}"),
        "Travel plan",
        "No matching finance terms",
        WorkflowState::NeedsAction,
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/saved-searches",
            Some(json!({
                "name": "Action invoices",
                "description": "Invoices that still need attention",
                "account_id": account_id,
                "query": "invoice overdue",
                "workflow_state": "needs_action",
                "local_state": "active",
                "channel_kind": "email",
                "is_smart_folder": true,
                "sort_order": 10
            })),
        ))
        .await
        .expect("create saved search response");
    assert_eq!(response.status(), StatusCode::OK);
    let created = response_json(response).await;
    let saved_search_id = created["saved_search_id"]
        .as_str()
        .expect("saved search id")
        .to_owned();
    assert!(saved_search_id.starts_with("mail_saved_search:"));
    assert_eq!(created["name"], "Action invoices");
    assert_eq!(created["query"], "invoice overdue");
    assert_eq!(created["is_smart_folder"], true);
    assert_eq!(created["message_count"], 1);

    assert_eq!(event_count(&pool, &saved_search_id).await, 1);
    let created_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'saved_search'
           AND entity_id = $1
           AND relationship_kind = 'saved_search_upsert'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&saved_search_id)
    .fetch_one(&pool)
    .await
    .expect("saved search create link");
    let created_observation_id: String = created_link
        .try_get("observation_id")
        .expect("saved search create observation id");
    let created_metadata: Value = created_link.try_get("metadata").expect("created metadata");
    assert_eq!(created_metadata["operation"], "saved_search_create");
    let created_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&created_observation_id)
    .fetch_one(&pool)
    .await
    .expect("saved search create observation");
    let created_origin_kind: String = created_observation
        .try_get("origin_kind")
        .expect("created origin kind");
    let created_payload: Value = created_observation
        .try_get("payload")
        .expect("created payload");
    assert_eq!(created_origin_kind, "manual");
    assert_eq!(created_payload["operation"], "saved_search_create");

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/saved-searches",
            Some(json!({
                "name": "Later invoices",
                "account_id": account_id,
                "query": "invoice",
                "workflow_state": "waiting",
                "local_state": "active",
                "channel_kind": "email",
                "is_smart_folder": true,
                "sort_order": 20
            })),
        ))
        .await
        .expect("create second saved search response");
    assert_eq!(response.status(), StatusCode::OK);
    let second_created = response_json(response).await;
    let second_saved_search_id = second_created["saved_search_id"]
        .as_str()
        .expect("second saved search id")
        .to_owned();

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/communications/saved-searches?smart_folder=true&limit=1",
            None,
        ))
        .await
        .expect("first saved search page response");
    assert_eq!(response.status(), StatusCode::OK);
    let first_page = response_json(response).await;
    assert_eq!(first_page["items"].as_array().expect("first page").len(), 1);
    assert_eq!(first_page["has_more"], true);
    let next_cursor = first_page["next_cursor"]
        .as_str()
        .expect("next saved search cursor");

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/saved-searches?smart_folder=true&limit=1&cursor={next_cursor}"),
            None,
        ))
        .await
        .expect("second saved search page response");
    assert_eq!(response.status(), StatusCode::OK);
    let second_page = response_json(response).await;
    assert_eq!(
        second_page["items"].as_array().expect("second page").len(),
        1
    );
    assert_eq!(second_page["has_more"], false);
    assert!(second_page["next_cursor"].is_null());

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/communications/saved-searches?smart_folder=true",
            None,
        ))
        .await
        .expect("list saved searches response");
    assert_eq!(response.status(), StatusCode::OK);
    let list = response_json(response).await;
    let items = list["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert_eq!(list["has_more"], false);
    assert!(list["next_cursor"].is_null());
    assert_eq!(items[0]["saved_search_id"], saved_search_id);
    assert_eq!(items[0]["message_count"], 1);
    assert_eq!(items[1]["saved_search_id"], second_saved_search_id);
    assert_eq!(items[1]["message_count"], 1);

    let response = app
        .clone()
        .oneshot(request(
            Method::PUT,
            &format!("/api/v1/communications/saved-searches/{saved_search_id}"),
            Some(json!({
                "name": "Waiting invoices",
                "query": "invoice",
                "workflow_state": "waiting",
                "local_state": "all",
                "is_smart_folder": false,
                "sort_order": 20
            })),
        ))
        .await
        .expect("update saved search response");
    assert_eq!(response.status(), StatusCode::OK);
    let updated = response_json(response).await;
    assert_eq!(updated["saved_search_id"], saved_search_id);
    assert_eq!(updated["name"], "Waiting invoices");
    assert_eq!(updated["workflow_state"], "waiting");
    assert_eq!(updated["local_state"], "all");
    assert_eq!(updated["is_smart_folder"], false);
    assert_eq!(updated["message_count"], 1);
    assert_eq!(event_count(&pool, &saved_search_id).await, 2);
    let upsert_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'saved_search'
           AND entity_id = $1
           AND relationship_kind = 'saved_search_upsert'",
    )
    .bind(&saved_search_id)
    .fetch_one(&pool)
    .await
    .expect("saved search upsert count");
    assert_eq!(upsert_count, 2);

    let response = app
        .clone()
        .oneshot(request(
            Method::DELETE,
            &format!("/api/v1/communications/saved-searches/{saved_search_id}"),
            None,
        ))
        .await
        .expect("delete saved search response");
    assert_eq!(response.status(), StatusCode::OK);
    let deleted = response_json(response).await;
    assert_eq!(deleted["deleted"], true);
    assert_eq!(event_count(&pool, &saved_search_id).await, 3);
    let deleted_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'saved_search'
           AND entity_id = $1
           AND relationship_kind = 'saved_search_delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&saved_search_id)
    .fetch_one(&pool)
    .await
    .expect("saved search delete link");
    let deleted_observation_id: String = deleted_link
        .try_get("observation_id")
        .expect("delete observation id");
    let deleted_metadata: Value = deleted_link.try_get("metadata").expect("delete metadata");
    assert_eq!(deleted_metadata["operation"], "saved_search_delete");
    let deleted_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("saved search delete observation");
    let deleted_origin_kind: String = deleted_observation
        .try_get("origin_kind")
        .expect("delete origin kind");
    let deleted_payload: Value = deleted_observation
        .try_get("payload")
        .expect("delete payload");
    assert_eq!(deleted_origin_kind, "manual");
    assert_eq!(deleted_payload["operation"], "saved_search_delete");

    let response = app
        .clone()
        .oneshot(request(
            Method::DELETE,
            &format!("/api/v1/communications/saved-searches/{second_saved_search_id}"),
            None,
        ))
        .await
        .expect("delete second saved search response");
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(request(
            Method::GET,
            "/api/v1/communications/saved-searches",
            None,
        ))
        .await
        .expect("list after delete response");
    assert_eq!(response.status(), StatusCode::OK);
    let list = response_json(response).await;
    assert_eq!(list["items"].as_array().expect("items").len(), 0);
}

#[tokio::test]
async fn v1_saved_search_counts_follow_rules_builder_match_semantics_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-saved-search-rules-{suffix}");
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-quarterly-{suffix}"),
        "Quarterly report",
        "General update",
        WorkflowState::New,
    )
    .await;
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-invoice-{suffix}"),
        "Travel plan",
        "Invoice approved for payment",
        WorkflowState::New,
    )
    .await;
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-unrelated-rules-{suffix}"),
        "Team lunch",
        "Menu discussion only",
        WorkflowState::New,
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/saved-searches",
            Some(json!({
                "name": "Flexible rules",
                "account_id": account_id,
                "query": "mode:any subject:quarterly body:invoice",
                "local_state": "active",
                "channel_kind": "email",
                "is_smart_folder": false,
                "sort_order": 5
            })),
        ))
        .await
        .expect("create rules saved search response");
    assert_eq!(response.status(), StatusCode::OK);
    let created = response_json(response).await;
    assert_eq!(created["message_count"], 2);

    let saved_search_id = created["saved_search_id"]
        .as_str()
        .expect("saved search id");
    let response = app
        .oneshot(request(
            Method::GET,
            "/api/v1/communications/saved-searches",
            None,
        ))
        .await
        .expect("list rules saved searches response");
    assert_eq!(response.status(), StatusCode::OK);
    let list = response_json(response).await;
    let items = list["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["saved_search_id"], saved_search_id);
    assert_eq!(items[0]["message_count"], 2);
}

#[tokio::test]
async fn v1_saved_search_counts_follow_nested_rules_builder_groups_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-saved-search-nested-{suffix}");
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-quarterly-{suffix}"),
        "Quarterly report",
        "General update",
        WorkflowState::New,
    )
    .await;
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-invoice-{suffix}"),
        "Travel plan",
        "Invoice approved for payment",
        WorkflowState::New,
    )
    .await;
    seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-unrelated-nested-{suffix}"),
        "Team lunch",
        "Menu discussion only",
        WorkflowState::New,
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/saved-searches",
            Some(json!({
                "name": "Nested rules",
                "account_id": account_id,
                "query": "(subject:quarterly OR body:invoice) AND sender:sender@example.com",
                "local_state": "active",
                "channel_kind": "email",
                "is_smart_folder": false,
                "sort_order": 6
            })),
        ))
        .await
        .expect("create nested rules saved search response");
    assert_eq!(response.status(), StatusCode::OK);
    let created = response_json(response).await;
    assert_eq!(created["message_count"], 2);
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
    workflow_state: WorkflowState,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Saved Search Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            account_id,
            "email_message",
            provider_record_id,
            format!("sha256:{provider_record_id}"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": body_text
            }),
        ))
        .await
        .expect("record raw source");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");
    message_store
        .transition_workflow_state(&projected.message_id, workflow_state)
        .await
        .expect("transition workflow state");
    projected.message_id
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn event_count(pool: &sqlx::PgPool, saved_search_id: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE subject->>'kind' = 'mail_saved_search' AND subject->>'id' = $1",
    )
    .bind(saved_search_id)
    .fetch_one(pool)
    .await
    .expect("event count")
}

fn uid() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
