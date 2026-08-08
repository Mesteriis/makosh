use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use axum::Router;
pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Request, StatusCode};
pub(crate) use makosh_hub_backend::app::router::{build_router, build_router_with_database};
pub(crate) use makosh_hub_backend::domains::graph::core::models::{
    GraphEvidenceSourceKind, GraphReviewState, NewGraphEdge, NewGraphEvidence, NewGraphNode,
    RelationshipType,
};
pub(crate) use makosh_hub_backend::domains::graph::core::store::GraphStore;
pub(crate) use makosh_hub_backend::platform::config::app_config::AppConfig;
pub(crate) use makosh_hub_backend::platform::graph::GraphNodeKind;
pub(crate) use makosh_hub_backend::platform::storage::database::Database;
pub(crate) use serde_json::{Value, json};
pub(crate) use sqlx::postgres::{PgPool, PgPoolOptions};
pub(crate) use tower::ServiceExt;
pub(crate) use url::Url;

pub(crate) const LOCAL_API_TOKEN: &str = "graph-api-test-token";
pub(crate) const EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT: usize = 100;
pub(crate) const EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT: usize = 100;

pub(crate) struct LiveGraphApiContext {
    pub(crate) app: Router,
    pub(crate) store: GraphStore,
    pool: PgPool,
    admin_pool: PgPool,
    database_name: String,
}

impl LiveGraphApiContext {
    pub(crate) async fn cleanup(self) {
        let Self {
            app,
            store,
            pool,
            admin_pool,
            database_name,
        } = self;
        drop(app);
        drop(store);
        pool.close().await;
        sqlx::query(
            r#"
            SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = $1
              AND pid <> pg_backend_pid()
            "#,
        )
        .bind(&database_name)
        .execute(&admin_pool)
        .await
        .expect("terminate graph API test database sessions");
        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS {}",
            quote_identifier(&database_name)
        ))
        .execute(&admin_pool)
        .await
        .expect("drop graph API test database");
        admin_pool.close().await;
    }
}

pub(crate) async fn live_graph_api_context(_test_name: &str) -> Option<LiveGraphApiContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let admin_database_url = database_url_with_database(&database_url, "postgres");
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&admin_database_url)
        .await
        .expect("admin database connection");
    let database_name = format!("makosh_graph_api_test_{}", unique_suffix());
    assert_safe_identifier(&database_name);
    sqlx::query(&format!(
        "CREATE DATABASE {}",
        quote_identifier(&database_name)
    ))
    .execute(&admin_pool)
    .await
    .expect("create graph API test database");

    let test_database_url = database_url_with_database(&database_url, &database_name);
    let database = Database::connect(Some(&test_database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = GraphStore::new(pool.clone());
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            test_database_url.as_str(),
        ),
        database,
    );

    Some(LiveGraphApiContext {
        app,
        store,
        pool,
        admin_pool,
        database_name,
    })
}

pub(crate) fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub(crate) fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token_without_actor(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn database_url_with_database(database_url: &str, database_name: &str) -> String {
    let mut url = Url::parse(database_url).expect("valid database URL");
    url.set_path(database_name);
    url.set_query(None);
    url.to_string()
}

fn assert_safe_identifier(identifier: &str) {
    assert!(
        identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
        "test database identifier must be simple ASCII"
    );
}

fn quote_identifier(identifier: &str) -> String {
    format!(r#""{}""#, identifier.replace('"', r#""""#))
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
