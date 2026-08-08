# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `084-test-backend-part-007`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/tests/graph_api/neighborhood.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_api/neighborhood.rs`
- Size bytes / Размер в байтах: `9815`
- Included characters / Включено символов: `9815`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence() {
    let Some(context) = live_graph_api_context("neighborhood").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:alex-neighborhood:{suffix}"),
            format!("Alex Neighborhood {suffix}"),
        ))
        .await
        .expect("person node");
    let email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("alex-neighborhood-{suffix}@example.com"),
            format!("alex-neighborhood-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let edge = context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                person.node_id.clone(),
                email.node_id.clone(),
                RelationshipType::PersonHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[NewGraphEvidence::new(
                GraphEvidenceSourceKind::Person,
                format!("person-source:{suffix}"),
            )
            .excerpt("confirmed by person record")
            .metadata(json!({"source": "graph_api_test"}))],
        )
        .await
        .expect("graph edge");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/graph/neighborhood?node_id={}&depth=1",
                person.node_id
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["selected_node"]["node_id"], json!(person.node_id));
    assert_eq!(body["selected_node"]["label"], json!(person.label));
    assert_eq!(
        body["edge_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
    );
    assert_eq!(body["truncated"], json!(false));
    assert_eq!(
        body["evidence_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
    );
    assert_eq!(body["evidence_truncated"], json!(false));

    let nodes = body["nodes"].as_array().expect("node array");
    assert_eq!(nodes.len(), 1);
    assert!(nodes.iter().all(|node| node["node_id"] != person.node_id));
    assert!(nodes.iter().any(|node| node["node_id"] == email.node_id));

    let edges = body["edges"].as_array().expect("edge array");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["edge_id"], json!(edge.edge_id));
    assert_eq!(edges[0]["source_node_id"], json!(person.node_id));
    assert_eq!(edges[0]["target_node_id"], json!(email.node_id));

    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0]["edge_id"], json!(edge.edge_id));
    assert_eq!(evidence[0]["source_kind"], json!("person"));
    assert_eq!(
        evidence[0]["source_id"],
        json!(format!("person-source:{suffix}"))
    );
    assert_eq!(evidence[0]["excerpt"], json!("confirmed by person record"));
    assert_eq!(evidence[0]["metadata"], json!({"source": "graph_api_test"}));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_caps_depth_one_edges_nodes_and_evidence() {
    let Some(context) = live_graph_api_context("neighborhood cap").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:alex-neighborhood-cap:{suffix}"),
            format!("Alex Neighborhood Cap {suffix}"),
        ))
        .await
        .expect("person node");

    for index in 0..=EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT {
        let email = context
            .store
            .upsert_node(&NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                format!("alex-neighborhood-cap-{suffix}-{index:03}@example.com"),
                format!("alex-neighborhood-cap-{suffix}-{index:03}@example.com"),
            ))
            .await
            .expect("email node");
        context
            .store
            .upsert_edge_with_evidence(
                &NewGraphEdge::new(
                    person.node_id.clone(),
                    email.node_id,
                    RelationshipType::PersonHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Person,
                    format!("person-source:{suffix}:{index:03}"),
                )],
            )
            .await
            .expect("graph edge");
    }

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/graph/neighborhood?node_id={}", person.node_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body["nodes"].as_array().expect("node array");
    let edges = body["edges"].as_array().expect("edge array");
    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(
        body["edge_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
    );
    assert_eq!(body["truncated"], json!(true));
    assert_eq!(
        body["evidence_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
    );
    assert_eq!(body["evidence_truncated"], json!(false));
    assert_eq!(nodes.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);
    assert!(nodes.iter().all(|node| node["node_id"] != person.node_id));
    assert_eq!(edges.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);
    assert_eq!(evidence.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_caps_evidence_for_returned_edges() {
    let Some(context) = live_graph_api_context("neighborhood evidence cap").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:alex-neighborhood-evidence-cap:{suffix}"),
            format!("Alex Neighborhood Evidence Cap {suffix}"),
        ))
        .await
        .expect("person node");
    let email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("alex-neighborhood-evidence-cap-{suffix}@example.com"),
            format!("alex-neighborhood-evidence-cap-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let evidence = (0..=EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
        .map(|index| {
            NewGraphEvidence::new(
                GraphEvidenceSourceKind::Person,
                format!("person-source:{suffix}:{index:03}"),
            )
        })
        .collect::<Vec<_>>();
    context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                person.node_id.clone(),
                email.node_id,
                RelationshipType::PersonHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &evidence,
        )
        .await
        .expect("graph edge with over-limit evidence");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/graph/neighborhood?node_id={}", person.node_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let edges = body["edges"].as_array().expect("edge array");
    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(body["truncated"], json!(false));
    assert_eq!(
        body["evidence_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
    );
    assert_eq!(body["evidence_truncated"], json!(true));
    assert_eq!(edges.len(), 1);
    assert_eq!(evidence.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT);

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_returns_not_found_when_node_id_is_missing() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/graph/neighborhood?depth=1",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "graph_node_not_found",
            "message": "graph node was not found"
        })
    );
}

#[tokio::test]
async fn graph_neighborhood_rejects_unsupported_depth() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/graph/neighborhood?node_id=graph:node:v1:person:alex&depth=2",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_graph_query",
            "message": "depth supports only 1"
        })
    );
}
```

### `backend/tests/graph_api/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_api/search.rs`
- Size bytes / Размер в байтах: `5084`
- Included characters / Включено символов: `5084`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn graph_summary_returns_empty_state_for_empty_database() {
    let Some(context) = live_graph_api_context("empty summary").await else {
        return;
    };

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/graph/summary",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["node_counts"], json!([]));
    assert_eq!(body["edge_counts"], json!([]));
    assert_eq!(body["evidence_count"], json!(0));
    assert_eq!(body["latest_projection_at"], Value::Null);
    assert_eq!(body["is_empty"], json!(true));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_nodes_returns_connected_picker_nodes_first() {
    let Some(context) = live_graph_api_context("node picker").await else {
        return;
    };
    let suffix = unique_suffix();
    let connected_person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:connected-picker:{suffix}"),
            format!("Connected Picker {suffix}"),
        ))
        .await
        .expect("connected person node");
    let connected_email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("connected-picker-{suffix}@example.com"),
            format!("connected-picker-{suffix}@example.com"),
        ))
        .await
        .expect("connected email node");
    let disconnected = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:disconnected-picker:{suffix}"),
            format!("Disconnected Picker {suffix}"),
        ))
        .await
        .expect("disconnected node");
    context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                connected_person.node_id.clone(),
                connected_email.node_id.clone(),
                RelationshipType::PersonHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[NewGraphEvidence::new(
                GraphEvidenceSourceKind::Person,
                format!("person-source:{suffix}"),
            )],
        )
        .await
        .expect("connected picker edge");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/graph/nodes?limit=2",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body.as_array().expect("node array");
    let node_ids = nodes
        .iter()
        .map(|node| node["node_id"].as_str().expect("node id"))
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 2);
    assert!(node_ids.contains(&connected_person.node_id.as_str()));
    assert!(node_ids.contains(&connected_email.node_id.as_str()));
    assert!(!node_ids.contains(&disconnected.node_id.as_str()));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_search_returns_matching_nodes() {
    let Some(context) = live_graph_api_context("search").await else {
        return;
    };
    let suffix = unique_suffix();
    let alex = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:alex:{suffix}"),
            format!("Alex Morgan {suffix}"),
        ))
        .await
        .expect("alex node");
    context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:blair:{suffix}"),
            format!("Blair Morgan {suffix}"),
        ))
        .await
        .expect("blair node");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/graph/search?q=alex",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body.as_array().expect("node array");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["node_id"], json!(alex.node_id));
    assert_eq!(nodes[0]["label"], json!(alex.label));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_search_rejects_empty_query() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/graph/search?q=",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_graph_query",
            "message": "q must not be empty"
        })
    );
}
```

### `backend/tests/graph_api/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_api/support.rs`
- Size bytes / Размер в байтах: `5178`
- Included characters / Включено символов: `5178`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

pub(crate) use axum::Router;
pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Request, StatusCode};
pub(crate) use makosh_hub_backend::app::{build_router, build_router_with_database};
pub(crate) use makosh_hub_backend::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};
pub(crate) use makosh_hub_backend::platform::config::AppConfig;
pub(crate) use makosh_hub_backend::platform::storage::Database;
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
        testkit::app::config_with_secret_and_database_url(
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
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
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
```

### `backend/tests/graph_api_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_api_architecture.rs`
- Size bytes / Размер в байтах: `1942`
- Included characters / Включено символов: `1942`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn graph_api_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_graph_api_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "graph api test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_graph_api_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_graph_api_test_violations(&path, violations);
            continue;
        }
        if !is_graph_api_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_graph_api_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "graph_api.rs" || file_name == "graph_api_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "graph_api")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/graph_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_projection.rs`
- Size bytes / Размер в байтах: `179`
- Included characters / Включено символов: `179`
- Truncated / Обрезано: `no`

```rust
#[path = "graph_projection/idempotence.rs"]
mod idempotence;
#[path = "graph_projection/project_links.rs"]
mod project_links;
#[path = "graph_projection/support.rs"]
mod support;
```

### `backend/tests/graph_projection/idempotence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_projection/idempotence.rs`
- Size bytes / Размер в байтах: `4025`
- Included characters / Включено символов: `4025`
- Truncated / Обрезано: `no`

```rust
use super::support::{
    assert_document_projected, assert_known_person_endpoint_projected, assert_message_edge_count,
    assert_message_edge_with_evidence, assert_unknown_email_endpoint_projected,
    graph_counts_for_suffix, live_projection_context, seed_message,
    seed_person_message_and_document, unique_suffix,
};

#[tokio::test]
async fn graph_projection_is_idempotent_for_v1_sources_against_postgres() {
    let Some(context) = live_projection_context("graph projection idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    seed_person_message_and_document(&context, suffix).await;

    let first = context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection");
    let counts_after_first = graph_counts_for_suffix(&context.pool, suffix).await;
    let second = context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection");
    let counts_after_second = graph_counts_for_suffix(&context.pool, suffix).await;

    assert_eq!(first.nodes_upserted, second.nodes_upserted);
    assert_eq!(first.edges_upserted, second.edges_upserted);
    assert_eq!(first.evidence_upserted, second.evidence_upserted);
    assert_eq!(counts_after_first, counts_after_second);

    let person_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM graph_nodes WHERE node_kind = 'person' AND stable_key LIKE $1",
    )
    .bind(format!("person:v1:email:%unknown-{suffix}%"))
    .fetch_one(&context.pool)
    .await
    .expect("unknown sender person count");
    assert_eq!(person_count, 0);

    assert_unknown_email_endpoint_projected(
        &context.pool,
        &format!("unknown-{suffix}@example.com"),
        &format!("provider-graph-projection-{suffix}"),
        "email_address_sent_message",
    )
    .await;
    assert_unknown_email_endpoint_projected(
        &context.pool,
        &format!("unknown-recipient-{suffix}@example.com"),
        &format!("provider-graph-projection-{suffix}"),
        "email_address_received_message",
    )
    .await;
    assert_known_person_endpoint_projected(&context.pool, suffix).await;
    assert_document_projected(&context.pool, suffix).await;
}

#[tokio::test]
async fn graph_projection_replaces_stale_unknown_message_edges_against_postgres() {
    let Some(context) =
        live_projection_context("graph projection stale message edge replacement").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let sender_email = format!("identity-upgrade-{suffix}@example.com");
    let provider_record_id = format!("provider-graph-identity-upgrade-{suffix}");
    let subject = format!("Graph identity upgrade subject {suffix}");
    let projected = seed_message(
        &context,
        suffix,
        &sender_email,
        &[format!("recipient-upgrade-{suffix}@example.com")],
        &provider_record_id,
        &subject,
    )
    .await;

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection before person exists");
    assert_message_edge_with_evidence(
        &context.pool,
        "email_address",
        &sender_email,
        &provider_record_id,
        "email_address_sent_message",
        &projected,
    )
    .await;

    context
        .person_store
        .upsert_email_person(&sender_email)
        .await
        .expect("upsert exact sender person");
    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection after person exists");

    assert_message_edge_with_evidence(
        &context.pool,
        "person",
        &sender_email,
        &provider_record_id,
        "person_sent_message",
        &projected,
    )
    .await;
    assert_message_edge_count(
        &context.pool,
        "email_address",
        &sender_email,
        &provider_record_id,
        "email_address_sent_message",
        0,
    )
    .await;
}
```

### `backend/tests/graph_projection/project_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_projection/project_links.rs`
- Size bytes / Размер в байтах: `9875`
- Included characters / Включено символов: `9875`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::documents::core::NewDocumentImport;
use makosh_hub_backend::domains::graph::core::{GraphNodeKind, node_id};
use makosh_hub_backend::domains::projects::core::{NewProject, project_graph_node_id};
use makosh_hub_backend::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkTargetKind,
};

use super::support::{
    ExpectedProjectEdge, assert_project_edge_with_evidence, cleanup_project_graph_fixture,
    live_projection_context, project_graph_counts, seed_message, unique_suffix,
};

#[tokio::test]
async fn graph_projection_links_projects_to_keyword_messages_documents_and_people_against_postgres()
{
    let Some(context) = live_projection_context("project graph projection").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphProject{suffix}");
    let project_id = format!("project:v1:graph:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Project {suffix}"),
                "Product Development",
                "Graph project projection test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(55),
        )
        .await
        .expect("upsert graph project");
    let owner = context
        .person_store
        .upsert_email_person(&format!("graph-project-owner-{suffix}@example.com"))
        .await
        .expect("upsert graph project owner");
    let projected = seed_message(
        &context,
        suffix,
        &owner.email_address,
        &[format!("graph-project-reviewer-{suffix}@example.com")],
        &format!("provider-graph-project-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;
    let document_id = format!("doc_graph_project_{suffix}");
    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            format!("{keyword} notes.md"),
            "# Notes\n\nProject graph content.",
        ))
        .await
        .expect("import graph project document");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection");
    let counts_after_first = project_graph_counts(&context.pool, &project_id).await;
    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection");
    let counts_after_second = project_graph_counts(&context.pool, &project_id).await;
    assert_eq!(counts_after_first, counts_after_second);

    let project_node_id = project_graph_node_id(&project_id);
    let owner_node_id = node_id(GraphNodeKind::Person, &owner.person_id);
    let reviewer_node_id = node_id(
        GraphNodeKind::EmailAddress,
        &format!("graph-project-reviewer-{suffix}@example.com"),
    );
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Message, &projected.message_id),
            relationship_type: "project_has_message",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Document, &document_id),
            relationship_type: "project_has_document",
            source_kind: "document",
            source_id: &document_id,
            observation_id: None,
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &owner_node_id,
            relationship_type: "project_involves_person",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &reviewer_node_id,
            relationship_type: "project_involves_email_address",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

#[tokio::test]
async fn graph_projection_omits_rejected_project_link_against_postgres() {
    let Some(context) = live_projection_context("project graph projection rejected link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphReject{suffix}");
    let project_id = format!("project:v1:graph-reject:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Reject Project {suffix}"),
                "Product Development",
                "Graph project rejected link test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(50),
        )
        .await
        .expect("upsert graph reject project");

    let projected = seed_message(
        &context,
        suffix,
        &format!("owner-reject-{suffix}@example.com"),
        &[format!("reviewer-reject-{suffix}@example.com")],
        &format!("provider-graph-reject-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;

    context
        .project_link_review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("graph-reject-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: projected.message_id.clone(),
            review_state: ProjectLinkReviewState::UserRejected,
            actor_id: format!("reviewer-actor-{suffix}"),
        })
        .await
        .expect("set rejected link review");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("project projection for rejected link");

    let project_node_id = project_graph_node_id(&project_id);
    let message_node_id = node_id(GraphNodeKind::Message, &projected.message_id);
    let link_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'project_has_message'
        "#,
    )
    .bind(&project_node_id)
    .bind(&message_node_id)
    .fetch_one(&context.pool)
    .await
    .expect("rejected project link count");
    assert_eq!(link_count, 0);

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

#[tokio::test]
async fn graph_projection_marks_confirmed_project_link_user_confirmed_against_postgres() {
    let Some(context) = live_projection_context("project graph projection confirmed link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphConfirm{suffix}");
    let project_id = format!("project:v1:graph-confirm:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Confirm Project {suffix}"),
                "Product Development",
                "Graph project confirmed link test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(50),
        )
        .await
        .expect("upsert graph confirm project");

    let projected = seed_message(
        &context,
        suffix,
        &format!("owner-confirm-{suffix}@example.com"),
        &[format!("reviewer-confirm-{suffix}@example.com")],
        &format!("provider-graph-confirm-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;

    context
        .project_link_review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("graph-confirm-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: projected.message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: format!("reviewer-actor-{suffix}"),
        })
        .await
        .expect("set confirmed link review");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("project projection for confirmed link");

    let project_node_id = project_graph_node_id(&project_id);
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Message, &projected.message_id),
            relationship_type: "project_has_message",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "user_confirmed",
            confidence: 1.0,
        },
    )
    .await;

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}
```

### `backend/tests/graph_projection/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_projection/support.rs`
- Size bytes / Размер в байтах: `16201`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::projects::core::{ProjectStore, project_graph_node_id};
use makosh_hub_backend::domains::projects::link_reviews::ProjectLinkReviewStore;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::graph_projection::GraphProjectionService;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

pub(crate) struct LiveProjectionContext {
    pub(crate) pool: PgPool,
    pub(crate) person_store: PersonProjectionStore,
    pub(crate) communication_store: CommunicationIngestionStore,
    pub(crate) message_store: MessageProjectionStore,
    pub(crate) document_store: DocumentImportStore,
    pub(crate) project_store: ProjectStore,
    pub(crate) graph_projection: GraphProjectionService,
    pub(crate) project_link_review_store: ProjectLinkReviewStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GraphCounts {
    pub(crate) nodes: i64,
    pub(crate) edges: i64,
    pub(crate) evidence: i64,
}

pub(crate) struct ProjectedMessageFixture {
    pub(crate) message_id: String,
    pub(crate) observation_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
}

pub(crate) struct ExpectedProjectEdge<'a> {
    pub(crate) source_node_id: &'a str,
    pub(crate) target_node_id: &'a str,
    pub(crate) relationship_type: &'a str,
    pub(crate) source_kind: &'a str,
    pub(crate) source_id: &'a str,
    pub(crate) observation_id: Option<&'a str>,
    pub(crate) review_state: &'a str,
    pub(crate) confidence: f64,
}

pub(crate) async fn live_projection_context(_test_name: &str) -> Option<LiveProjectionContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(LiveProjectionContext {
        pool: pool.clone(),
        person_store: PersonProjectionStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        document_store: DocumentImportStore::new(pool.clone()),
        project_store: ProjectStore::new(pool.clone()),
        graph_projection: GraphProjectionService::new(pool.clone()),
        project_link_review_store: ProjectLinkReviewStore::new(pool),
    })
}

pub(crate) async fn seed_person_message_and_document(
    context: &LiveProjectionContext,
    suffix: u128,
) {
    context
        .person_store
        .upsert_email_person(&format!(" Known-{suffix}@Example.com "))
        .await
        .expect("upsert known person");

    seed_message(
        context,
        suffix,
        &format!("Unknown-{suffix}@Example.com"),
        &[
            format!("known-{suffix}@example.com"),
            format!("unknown-recipient-{suffix}@example.com"),
        ],
        &format!("provider-graph-projection-{suffix}"),
        &format!("Graph projection subject {suffix}"),
    )
    .await;

    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_graph_projection_{suffix}"),
            format!("graph-projection-{suffix}.md"),
            "# Graph Projection\n\nDocument body.",
        ))
        .await
        .expect("import graph projection document");
}

pub(crate) async fn seed_message(
    context: &LiveProjectionContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
) -> ProjectedMessageFixture {
    let account_id = format!("acct_graph_projection_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Graph Projection Gmail",
            format!("graph-projection-{suffix}@example.com"),
        ))
        .await
        .expect("store graph projection provider account");

    let raw_record_id = format!("raw_graph_projection_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:graph-projection-{suffix}:{provider_record_id}"),
                format!("batch_graph_projection_{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": "Graph projection body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"graph_projection_test"})),
        )
        .await
        .expect("record graph projection raw message");

    let projected = project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project raw graph projection message");

    ProjectedMessageFixture {
        message_id: projected.message_id,
        observation_id: projected.observation_id,
        raw_record_id: projected.raw_record_id,
        provider_record_id: projected.provider_record_id,
        subject: projected.subject,
    }
}

pub(crate) async fn graph_counts_for_suffix(pool: &PgPool, suffix: u128) -> GraphCounts {
    let pattern = format!("%{suffix}%");
    let nodes =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_nodes WHERE stable_key LIKE $1")
            .bind(&pattern)
            .fetch_one(pool)
            .await
            .expect("graph node count for suffix");
    let edges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.stable_key LIKE $1 OR target.stable_key LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .expect("graph edge count for suffix");
    let evidence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE source_id LIKE $1 OR metadata::text LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .expect("graph evidence count for suffix");

    GraphCounts {
        nodes,
        edges,
        evidence,
    }
}

pub(crate) async fn project_graph_counts(pool: &PgPool, project_id: &str) -> GraphCounts {
    let project_node_id = project_graph_node_id(project_id);
    let nodes = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM graph_nodes WHERE node_id = $1 AND node_kind = 'project'",
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph node count");
    let edges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges
        WHERE source_node_id = $1
          AND relationship_type IN (
              'project_has_message',
              'project_has_document',
              'project_involves_person',
              'project_involves_email_address'
          )
        "#,
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph edge count");
    let evidence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence evidence
        JOIN graph_edges edge ON edge.edge_id = evidence.edge_id
        WHERE edge.source_node_id = $1
        "#,
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph evidence count");

    GraphCounts {
        nodes,
        edges,
        evidence,
    }
}

pub(crate) async fn assert_project_edge_with_evidence(
    pool: &PgPool,
    expected: ExpectedProjectEdge<'_>,
) {
    let row = sqlx::query(
        r#"
        SELECT
            edge.review_state,
            edge.confidence::float8 AS confidence,
            evidence.source_kind,
            evidence.source_id,
            evidence.observation_id
        FROM graph_edges edge
        JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
        WHERE edge.source_node_id = $1
          AND edge.target_node_id = $2
          AND edge.relationship_type = $3
          AND evidence.source_kind = $4
          AND evidence.source_id = $5
        "#,
    )
    .bind(expected.source_node_id)
    .bind(expected.target_node_id)
    .bind(expected.relationship_type)
    .bind(expected.source_kind)
    .bind(expected.source_id)
    .fetch_one(pool)
    .await
    .expect("project edge with evidence");

    let review_state: String = row.try_get("review_state").expect("review state");
    let confidence: f64 = row.try_get("confidence").expect("confidence");
    let stored_source_kind: String = row.try_get("source_kind").expect("source kind");
    let stored_source_id: String = row.try_get("source_id").expect("source id");
    let stored_observation_id: Option<String> =
        row.try_get("observation_id").expect("observation id");
    assert_eq!(review_state, expected.review_state);
    assert!((confidence - expected.confidence).abs() < f64::EPSILON);
    assert_eq!(stored_source_kind, expected.source_kind);
    assert_eq!(stored_source_id, expected.source_id);
    assert_eq!(stored_observation_id.as_deref(), expected.observation_id);
}

pub(crate) async fn cleanup_project_graph_fixture(pool: &PgPool, project_id: &str) {
    sqlx::query("DELETE FROM graph_nodes WHERE node_id = $1")
        .bind(project_graph_node_id(project_id))
        .execute(pool)
        .await
        .expect("cleanup project graph node");
    sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .expect("cleanup graph project");
}

pub(crate) async fn assert_unknown_email_endpoint_projected(
    pool: &PgPool,
    email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
) {
    let message = message_fixture_by_provider_record_id(pool, provider_record_id).await;
    assert_message_edge_with_evidence(
        pool,
        "email_address",
        email_address,
        provider_record_id,
        relationship_type,
        &message,
    )
    .await;

    let person_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_nodes
        WHERE node_kind = 'person'
          AND (stable_key LIKE $1 OR properties->>'email_address' = $2)
        "#,
    )
    .bind(format!("%{email_address}%"))
    .bind(email_address)
    .fetch_one(pool)
    .await
    .expect("unknown email person node count");
    assert_eq!(person_count, 0);
}

pub(crate) async fn assert_message_edge_with_evidence(
    pool: &PgPool,
    source_node_kind: &str,
    source_email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
    message: &ProjectedMessageFixture,
) {
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/graph_projection_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_projection_architecture.rs`
- Size bytes / Размер в байтах: `2012`
- Included characters / Включено символов: `2012`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn graph_projection_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_graph_projection_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "graph projection test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_graph_projection_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_graph_projection_test_violations(&path, violations);
            continue;
        }
        if !is_graph_projection_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_graph_projection_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "graph_projection.rs" || file_name == "graph_projection_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "graph_projection")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/hard_v1_routes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/hard_v1_routes.rs`
- Size bytes / Размер в байтах: `3064`
- Included characters / Включено символов: `3064`
- Truncated / Обрезано: `no`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router;
use makosh_hub_backend::platform::config::AppConfig;

const LOCAL_API_SECRET: &str = "hard-v1-routes-test-secret";

#[tokio::test]
async fn former_versioned_routes_are_not_public_aliases() {
    let app = build_router(config_with_api_secret());

    for path in [
        "/api/v2/tasks",
        "/api/v3/ai/status",
        "/api/v4/capabilities",
        "/api/v5/capabilities",
    ] {
        let response = app
            .clone()
            .oneshot(get_request_with_secret(path))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn telegram_and_whatsapp_capabilities_are_split_under_v1() {
    let app = build_router(config_with_api_secret());

    let telegram = app
        .clone()
        .oneshot(get_request_with_secret(
            "/api/v1/integrations/telegram/capabilities",
        ))
        .await
        .expect("telegram capabilities response");
    assert_eq!(telegram.status(), StatusCode::OK);
    let telegram_body = json_body(telegram).await;
    assert_eq!(telegram_body["version"], json!("2.1"));
    assert_eq!(telegram_body["runtime_mode"], json!("fixture"));
    assert!(telegram_body["planned_features"].is_array());
    assert_has_capability(&telegram_body, "runtime.fixture");

    let whatsapp = app
        .oneshot(get_request_with_secret(
            "/api/v1/integrations/whatsapp/capabilities",
        ))
        .await
        .expect("whatsapp capabilities response");
    assert_eq!(whatsapp.status(), StatusCode::OK);
    let whatsapp_body = json_body(whatsapp).await;
    assert_eq!(whatsapp_body["version"], json!("2.0"));
    assert_eq!(whatsapp_body["runtime_mode"], json!("fixture"));
    assert!(whatsapp_body["planned_features"].is_array());
    assert!(whatsapp_body["provider_shapes"].is_array());
    assert_has_capability(&whatsapp_body, "runtime.fixture");
}

fn config_with_api_secret() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_SECRET)
}

fn get_request_with_secret(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header(
            "x-makosh-secret",
            HeaderValue::from_static(LOCAL_API_SECRET),
        )
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&body).expect("json body")
}

fn assert_has_capability(body: &Value, capability: &str) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    assert!(
        capabilities
            .iter()
            .any(|item| item["capability"] == json!(capability)
                || item["operation"] == json!(capability)),
        "{capability}"
    );
}
```

### `backend/tests/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/health.rs`
- Size bytes / Размер в байтах: `3361`
- Included characters / Включено символов: `3361`
- Truncated / Обрезано: `no`

```rust
use testkit::{app, context::TestContext};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn healthz_returns_ok_status_and_service_name() {
    let app = build_router(app::config());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(
        value,
        json!({
            "status": "ok",
            "service": "makosh-backend"
        })
    );
}

#[tokio::test]
async fn readyz_returns_service_unavailable_when_database_is_not_configured() {
    let app = build_router(app::config());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = to_bytes(response.into_body(), 2048)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(value["status"], json!("degraded"));
    assert_eq!(value["service"], json!("makosh-backend"));
    assert_eq!(
        value["checks"]["database"]["status"],
        json!("not_configured")
    );
    assert!(value["checks"]["database"]["message"].is_string());
    assert_eq!(
        value["checks"]["migrations"]["status"],
        json!("not_configured")
    );
    assert!(value["checks"]["migrations"]["message"].is_string());
}

#[tokio::test]
async fn readyz_reports_database_and_migrations_ok_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(app::config_with_database_url(database_url), database);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(value["status"], "ok");
    assert_eq!(value["checks"]["database"]["status"], "ok");
    assert_eq!(
        value["checks"]["database"]["message"],
        "database is reachable"
    );
    assert_eq!(value["checks"]["migrations"]["status"], "ok");
    assert_eq!(
        value["checks"]["migrations"]["message"],
        "required database migrations are applied"
    );
}
```

### `backend/tests/mail_archive_inspection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/mail_archive_inspection.rs`
- Size bytes / Размер в байтах: `1996`
- Included characters / Включено символов: `1996`
- Truncated / Обрезано: `no`

```rust
use std::io::{Cursor, Write};

use makosh_hub_backend::domains::communications::archive_inspection::{
    ArchiveInspectionError, ArchiveInspectionLimits, inspect_zip_bytes,
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }

    writer.finish().unwrap().into_inner()
}

#[test]
fn inspects_safe_zip_metadata_without_extracting() {
    let bytes = zip_bytes(&[("docs/readme.txt", b"hello"), ("invoice.pdf", b"pdf bytes")]);

    let report = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap();

    assert_eq!(report.entry_count, 2);
    assert_eq!(report.total_uncompressed_bytes, 14);
    assert_eq!(report.entries[0].normalized_path, "docs/readme.txt");
    assert_eq!(report.entries[0].uncompressed_size, 5);
    assert!(!report.has_nested_archive);
}

#[test]
fn rejects_zip_entries_with_path_traversal() {
    let bytes = zip_bytes(&[("../secret.txt", b"secret")]);

    let err = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::UnsafeEntryPath { entry_name }
            if entry_name == "../secret.txt"
    ));
}

#[test]
fn rejects_zip_bombs_by_uncompressed_size_limit() {
    let bytes = zip_bytes(&[("large.txt", b"12345")]);
    let limits = ArchiveInspectionLimits {
        max_uncompressed_bytes: 4,
        ..ArchiveInspectionLimits::default()
    };

    let err = inspect_zip_bytes(&bytes, limits).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::UncompressedSizeExceeded { total, limit }
            if total == 5 && limit == 4
    ));
}
```

### `backend/tests/mail_storage.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/mail_storage.rs`
- Size bytes / Размер в байтах: `6278`
- Included characters / Включено символов: `6278`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanStatus, CommunicationAttachmentDisposition, CommunicationStorageError,
    CommunicationStorageStore, LocalCommunicationBlobStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;

#[tokio::test]
async fn local_mail_blob_store_writes_content_addressed_blob_under_root() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let store = LocalCommunicationBlobStore::new(temp_dir.path());
    let first = store
        .put_blob(b"raw message bytes")
        .await
        .expect("write first blob");
    let second = store
        .put_blob(b"raw message bytes")
        .await
        .expect("write same blob again");

    assert_eq!(first, second);
    assert_eq!(first.storage_kind, "local_fs");
    assert_eq!(first.size_bytes, 17);
    assert!(first.sha256.starts_with("sha256:"));
    assert!(!first.storage_path.starts_with('/'));
    assert!(!first.storage_path.contains(".."));
    assert!(temp_dir.path().join(&first.storage_path).is_file());
}

#[tokio::test]
async fn mail_storage_records_attachment_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let mail_store = CommunicationStorageStore::new(pool.clone());
    let blob_root = tempfile::tempdir().expect("blob root");
    let local_blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let suffix = unique_suffix();
    let account_id = format!("acct_mail_storage_{suffix}");
    let provider_record_id = format!("mail-storage-message-{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Icloud,
            "Mail storage account",
            format!("mail-storage-{suffix}@example.invalid"),
        ))
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-mail-storage-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:raw-mail-storage-{suffix}"),
                format!("batch-mail-storage-{suffix}"),
                json!({
                    "subject": "Attachment storage",
                    "from": "sender@example.invalid",
                    "to": ["recipient@example.invalid"],
                    "body_text": "See attached file."
                }),
            )
            .provenance(json!({"source": "mail_storage_test"})),
        )
        .await
        .expect("raw record");
    let message = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let local_blob = local_blob_store
        .put_blob(b"pdf contents")
        .await
        .expect("write local attachment blob");
    let blob = mail_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type("application/pdf"),
        )
        .await
        .expect("upsert blob");
    let attachment = mail_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message.message_id,
                &raw.raw_record_id,
                &blob.blob_id,
                "part-1",
                "application/pdf",
                local_blob.size_bytes,
                &blob.sha256,
            )
            .filename("invoice.pdf")
            .disposition(CommunicationAttachmentDisposition::Attachment),
        )
        .await
        .expect("upsert attachment");

    assert_eq!(attachment.message_id, message.message_id);
    assert_eq!(attachment.raw_record_id, raw.raw_record_id);
    assert_eq!(attachment.blob_id, blob.blob_id);
    assert_eq!(attachment.filename.as_deref(), Some("invoice.pdf"));
    assert_eq!(attachment.content_type, "application/pdf");
    assert_eq!(attachment.size_bytes, 12);
    assert_eq!(
        attachment.disposition,
        CommunicationAttachmentDisposition::Attachment
    );
    assert_eq!(
        attachment.scan_status,
        AttachmentSafetyScanStatus::NotScanned
    );
    assert!(attachment.scan_engine.is_none());
    assert!(attachment.scan_checked_at.is_none());
    assert!(attachment.scan_summary.is_none());
    assert_eq!(attachment.scan_metadata, json!({}));

    let attachment_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_attachments WHERE message_id = $1",
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await
    .expect("attachment count");
    assert_eq!(attachment_count, 1);
}

#[tokio::test]
async fn mail_blob_metadata_rejects_unsafe_storage_path_before_database_write() {
    let store =
        CommunicationStorageStore::new(sqlx::PgPool::connect_lazy("postgres://unused").unwrap());
    let error = store
        .upsert_blob(&NewCommunicationBlob::new(
            "local_fs",
            "../outside.blob",
            "sha256:unsafe",
            1,
        ))
        .await
        .expect_err("unsafe path must fail");

    assert!(
        matches!(error, CommunicationStorageError::UnsafeStoragePath(ref path) if path == "../outside.blob"),
        "expected UnsafeStoragePath, got {error:?}"
    );
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/memory_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/memory_engine.rs`
- Size bytes / Размер в байтах: `13093`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{Duration, TimeZone, Utc};
use makosh_hub_backend::engines::memory::{
    MemoryCardDraft, MemoryContextSource, MemoryEngine, MemoryEntityRef, MemoryFactDraft,
    MemoryFactState,
};

#[test]
fn memory_engine_builds_persona_notes_memory_card_draft() {
    let draft = MemoryEngine::persona_notes_memory_card(
        "person:v1:email:alice@example.com",
        "  Met Alice at the local-first workshop.  ",
    )
    .expect("notes should create a memory card draft");

    assert_eq!(draft.title, "Compatibility notes");
    assert_eq!(draft.description, "Met Alice at the local-first workshop.");
    assert_eq!(
        draft.source,
        "persons.notes:person:v1:email:alice@example.com"
    );
    assert_eq!(draft.confidence, 1.0);
    assert_eq!(draft.importance, 5);
}

#[test]
fn memory_engine_ignores_empty_persona_notes() {
    let draft = MemoryEngine::persona_notes_memory_card("person:v1:email:alice@example.com", "  ");

    assert!(draft.is_none());
}

#[test]
fn memory_engine_builds_source_backed_persona_fact_draft() {
    let draft = MemoryEngine::persona_fact_memory(
        "person:v1:email:alice@example.com",
        " interest ",
        " local-first systems ",
        " communication_messages:message-1 ",
        0.84,
    )
    .expect("source-backed persona fact should be valid");

    assert_eq!(draft.affected_entity_kind, "persona");
    assert_eq!(
        draft.affected_entity_id,
        "person:v1:email:alice@example.com"
    );
    assert_eq!(draft.fact_type, "interest");
    assert_eq!(draft.value, "local-first systems");
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(draft.confidence, 0.84);
    assert_eq!(draft.review_state, "accepted");
    assert_eq!(draft.produced_by, "memory_engine");
}

#[test]
fn memory_engine_rejects_unsourced_persona_fact_draft() {
    let error = MemoryEngine::persona_fact_memory(
        "person:v1:email:alice@example.com",
        "interest",
        "local-first systems",
        " ",
        0.84,
    )
    .expect_err("fact source should be required");

    assert_eq!(error.to_string(), "memory source must not be empty");
}

#[test]
fn memory_engine_builds_source_backed_context_pack_for_entity() {
    let persona_id = "person:v1:email:alice@example.com";
    let facts = vec![
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "interest".to_owned(),
            value: "local-first systems".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.92,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "project".to_owned(),
            value: "Макошь".to_owned(),
            source: "documents:doc-1".to_owned(),
            confidence: 0.81,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: "person:v1:email:bob@example.com".to_owned(),
            fact_type: "interest".to_owned(),
            value: "unrelated".to_owned(),
            source: "communication_messages:message-2".to_owned(),
            confidence: 0.99,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
    ];
    let cards = vec![MemoryCardDraft {
        title: "Compatibility notes".to_owned(),
        description: "Met Alice at the local-first workshop.".to_owned(),
        source: format!("persons.notes:{persona_id}"),
        confidence: 1.0,
        importance: 5,
    }];

    let pack = MemoryEngine::context_pack("persona", persona_id, &facts, &cards, 10)
        .expect("context pack should be valid");

    assert_eq!(pack.affected_entity_kind, "persona");
    assert_eq!(pack.affected_entity_id, persona_id);
    assert_eq!(pack.items.len(), 3);
    assert_eq!(pack.items[0].item_kind, "memory_card");
    assert_eq!(pack.items[0].title, "Compatibility notes");
    assert_eq!(pack.items[0].source, format!("persons.notes:{persona_id}"));
    assert_eq!(pack.items[1].item_kind, "fact");
    assert_eq!(pack.items[1].title, "interest");
    assert_eq!(pack.items[1].body, "local-first systems");
    assert_eq!(pack.items[2].title, "project");
    assert_eq!(
        pack.source_citations,
        vec![
            format!("persons.notes:{persona_id}"),
            "communication_messages:message-1".to_owned(),
            "documents:doc-1".to_owned(),
        ]
    );
    assert_eq!(pack.produced_by, "memory_engine");
    assert_eq!(pack.confidence, 0.91);
}

#[test]
fn memory_engine_detects_missing_source_backed_fact_types_for_entity() {
    let persona_id = "person:v1:email:alice@example.com";
    let facts = vec![
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "interest".to_owned(),
            value: "local-first systems".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.92,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "project".to_owned(),
            value: "Макошь".to_owned(),
            source: "documents:doc-1".to_owned(),
            confidence: 0.81,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
    ];

    let gaps = MemoryEngine::memory_gaps(
        "persona",
        persona_id,
        &["interest", " project ", "preference", "interest"],
        &facts,
    )
    .expect("memory gaps should be valid");

    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].affected_entity_kind, "persona");
    assert_eq!(gaps[0].affected_entity_id, persona_id);
    assert_eq!(gaps[0].missing_fact_type, "preference");
    assert_eq!(
        gaps[0].source,
        "memory_engine:gap:persona:person:v1:email:alice@example.com:preference"
    );
    assert_eq!(gaps[0].review_state, "suggested");
    assert_eq!(gaps[0].produced_by, "memory_engine");
}

#[test]
fn memory_engine_detects_stale_source_backed_facts_for_entity() {
    let persona_id = "person:v1:email:alice@example.com";
    let as_of = Utc.with_ymd_and_hms(2026, 6, 13, 12, 0, 0).unwrap();
    let facts = vec![
        MemoryFactState {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "interest".to_owned(),
            value: "local-first systems".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.92,
            review_state: "accepted".to_owned(),
            last_verified_at: Some(as_of - Duration::days(120)),
        },
        MemoryFactState {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "project".to_owned(),
            value: "Макошь".to_owned(),
            source: "documents:doc-1".to_owned(),
            confidence: 0.81,
            review_state: "accepted".to_owned(),
            last_verified_at: Some(as_of - Duration::days(4)),
        },
        MemoryFactState {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: "person:v1:email:bob@example.com".to_owned(),
            fact_type: "interest".to_owned(),
            value: "unrelated".to_owned(),
            source: "communication_messages:message-2".to_owned(),
            confidence: 0.99,
            review_state: "accepted".to_owned(),
            last_verified_at: Some(as_of - Duration::days(120)),
        },
    ];

    let stale = MemoryEngine::stale_memory_candidates("persona", persona_id, &facts, as_of, 90)
        .expect("stale candidates should be valid");

    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].affected_entity_kind, "persona");
    assert_eq!(stale[0].affected_entity_id, persona_id);
    assert_eq!(stale[0].fact_type, "interest");
    assert_eq!(stale[0].value, "local-first systems");
    assert_eq!(stale[0].source, "communication_messages:message-1");
    assert_eq!(stale[0].last_verified_at, Some(as_of - Duration::days(120)));
    assert_eq!(stale[0].review_state, "suggested");
    assert_eq!(stale[0].produced_by, "memory_engine");
}

#[test]
fn memory_engine_assembles_cross_domain_context_for_related_entities() {
    let project_id = "project:makosh";
    let related_entities = vec![
        MemoryEntityRef {
            entity_kind: "communication".to_owned(),
            entity_id: "communication:email-1".to_owned(),
            relation_kind: "source_evidence".to_owned(),
        },
        MemoryEntityRef {
            entity_kind: "document".to_owned(),
            entity_id: "document:architecture-note".to_owned(),
            relation_kind: "context_document".to_owned(),
        },
    ];
    let sources = vec![
        MemoryContextSource {
            entity_kind: "project".to_owned(),
            entity_id: project_id.to_owned(),
            item_kind: "fact".to_owned(),
            title: "status".to_owned(),
            body: "Implementation alignment is in progress.".to_owned(),
            source: "projects:makosh".to_owned(),
            confidence: 0.95,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "communication".to_owned(),
            entity_id: "communication:email-1".to_owned(),
            item_kind: "communication_summary".to_owned(),
            title: "Email evidence".to_owned(),
            body: "The message introduced an obligation for the project.".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.91,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "document".to_owned(),
            entity_id: "document:architecture-note".to_owned(),
            item_kind: "decision".to_owned(),
            title: "Architecture decision".to_owned(),
            body: "Memory context must preserve source citations.".to_owned(),
            source: "documents:architecture-note".to_owned(),
            confidence: 0.88,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "organization".to_owned(),
            entity_id: "organization:unrelated".to_owned(),
            item_kind: "fact".to_owned(),
            title: "Unrelated organization".to_owned(),
            body: "This item is outside the requested context graph.".to_owned(),
            source: "organizations:unrelated".to_owned(),
            confidence: 1.0,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "communication".to_owned(),
            entity_id: "communication:email-1".to_owned(),
            item_kind: "fact".to_owned(),
            title: "Rejected evidence".to_owned(),
            body: "Rejected evidence must not enter the context pack.".to_owned(),
            source: "communication_messages:message-rejected".to_owned(),
            confidence: 0.99,
            review_state: "user_rejected".to_owned(),
        },
    ];

    let pack = MemoryEngine::cross_domain_context_pack(
        "project",
        project_id,
        &related_entities,
        &sources,
        10,
    )
    .expect("cross-domain context pac
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/message_flags_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/message_flags_api.rs`
- Size bytes / Размер в байтах: `7078`
- Included characters / Включено символов: `7078`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, NewProjectedMessage,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const TOKEN: &str = "message-flags-api-test-token";

async fn app(ctx: &TestContext) -> axum::Router {
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(TOKEN, ctx.connection_string().as_str()),
        database,
    )
}

fn request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TOKEN)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json")
}

#[tokio::test]
async fn message_important_endpoint_toggles_metadata_flag() {
    let ctx = TestContext::new().await;
    let communication_store = CommunicationIngestionStore::new(ctx.pool().clone());
    let message_store = MessageProjectionStore::new(ctx.pool().clone());
    let suffix = unique_suffix();
    let account_id = format!("acct-important-{suffix}");
    let raw_record_id = format!("raw-important-{suffix}");
    let provider_record_id = format!("provider-important-{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Important IMAP",
            format!("important-{suffix}@example.com"),
        ))
        .await
        .expect("account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "subject": "Important subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Important body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"message_flags_api_test"})),
        )
        .await
        .expect("raw record");

    let projected = message_store
        .upsert_message(&NewProjectedMessage {
            message_id: format!("message-important-{suffix}"),
            raw_record_id: raw.raw_record_id,
            account_id: account_id.clone(),
            provider_record_id,
            subject: "Important subject".to_owned(),
            sender: "alice@example.com".to_owned(),
            recipients: vec!["bob@example.com".to_owned()],
            body_text: "Important body".to_owned(),
            occurred_at: raw.occurred_at,
            channel_kind: "email".to_owned(),
            conversation_id: None,
            sender_display_name: Some("alice@example.com".to_owned()),
            delivery_state: "received".to_owned(),
            message_metadata: json!({}),
        })
        .await
        .expect("message");
    let message_id = projected.message_id;

    let app = app(&ctx).await;
    let uri = format!("/api/v1/communications/messages/{message_id}/important");

    let response = app
        .clone()
        .oneshot(request(Method::POST, &uri))
        .await
        .expect("important response");
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["important"], true);
    let stored = message_store
        .message(&message_id)
        .await
        .expect("stored message")
        .expect("message exists");
    assert_eq!(stored.message_metadata["important"], true);
    let first_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'message_flag_update'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&message_id)
    .fetch_one(ctx.pool())
    .await
    .expect("first observation link");
    let first_observation_id: String = first_link
        .try_get("observation_id")
        .expect("first observation id");
    let first_metadata: Value = first_link.try_get("metadata").expect("first metadata");
    assert_eq!(first_metadata["important"], true);
    let first_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&first_observation_id)
    .fetch_one(ctx.pool())
    .await
    .expect("first observation");
    let first_origin_kind: String = first_observation
        .try_get("origin_kind")
        .expect("first origin kind");
    let first_payload: Value = first_observation.try_get("payload").expect("first payload");
    assert_eq!(first_origin_kind, "manual");
    assert_eq!(first_payload["operation"], "message_important_toggle");
    assert_eq!(first_payload["message_id"], message_id);

    let response = app
        .oneshot(request(Method::POST, &uri))
        .await
        .expect("second important response");
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["important"], false);
    let stored = message_store
        .message(&message_id)
        .await
        .expect("stored message")
        .expect("message exists");
    assert_eq!(stored.message_metadata["important"], false);
    let links_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
             FROM observation_links
             WHERE domain = 'communications'
               AND entity_kind = 'communication_message'
               AND entity_id = $1
               AND relationship_kind = 'message_flag_update'",
    )
    .bind(&message_id)
    .fetch_one(ctx.pool())
    .await
    .expect("message flag observation count");
    assert_eq!(links_count, 2);
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
```

### `backend/tests/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages.rs`
- Size bytes / Размер в байтах: `267`
- Included characters / Включено символов: `267`
- Truncated / Обрезано: `no`

```rust
#[path = "messages/analysis.rs"]
mod analysis;
#[path = "messages/projection_core.rs"]
mod projection_core;
#[path = "messages/projection_queries.rs"]
mod projection_queries;
#[path = "messages/support.rs"]
mod support;
#[path = "messages/workflow.rs"]
mod workflow;
```

### `backend/tests/messages/analysis.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages/analysis.rs`
- Size bytes / Размер в байтах: `4757`
- Included characters / Включено символов: `4757`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::communications::analytics::EmailAnalyticsStore;
use makosh_hub_backend::domains::communications::core::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use testkit::context::TestContext;

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

#[tokio::test]
async fn message_set_ai_analysis_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("ai analysis").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_ai_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "AI Gmail",
        format!("ai-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_ai_{suffix}"),
        &format!("provider-ai-{suffix}"),
        "AI test subject",
        "AI test body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let updated = message_store
        .set_ai_analysis(
            &projected.message_id,
            Some("work"),
            Some("This is a work-related email about project updates."),
            Some(85),
        )
        .await
        .expect("set ai analysis");

    assert_eq!(updated.ai_category.as_deref(), Some("work"));
    assert_eq!(
        updated.ai_summary.as_deref(),
        Some("This is a work-related email about project updates.")
    );
    assert_eq!(updated.importance_score, Some(85));
    assert!(updated.ai_summary_generated_at.is_some());

    let fetched = message_store
        .message(&projected.message_id)
        .await
        .expect("fetch message")
        .expect("message exists");
    assert_eq!(fetched.ai_category.as_deref(), Some("work"));
    assert_eq!(fetched.importance_score, Some(85));
}

#[tokio::test]
async fn message_analytics_decodes_averages_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_analytics_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Analytics Gmail",
        format!("analytics-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_analytics_{suffix}"),
        &format!("provider-analytics-{suffix}"),
        "Analytics subject",
        "Analytics body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");
    message_store
        .set_ai_analysis(
            &projected.message_id,
            Some("work"),
            Some("summary"),
            Some(80),
        )
        .await
        .expect("set ai analysis");

    let analytics = EmailAnalyticsStore::new(pool);
    let health = analytics
        .mailbox_health(Some(&account_id))
        .await
        .expect("mailbox health");
    let senders = analytics
        .top_senders(Some(&account_id), 10)
        .await
        .expect("top senders");

    assert_eq!(health.total_messages, 1);
    assert_eq!(health.average_importance, 80.0);
    assert_eq!(senders.len(), 1);
    assert_eq!(senders[0].avg_importance, 80.0);
}

#[tokio::test]
async fn message_set_ai_analysis_rejects_invalid_score() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("ai analysis invalid score").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_ai_score_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "AI Score Gmail",
        format!("ai-score-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_ai_score_{suffix}"),
        &format!("provider-ai-score-{suffix}"),
        "Score test",
        "Score body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let result = message_store
        .set_ai_analysis(&projected.message_id, None, None, Some(101))
        .await;

    assert!(result.is_err());
}
```

### `backend/tests/messages/projection_core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages/projection_core.rs`
- Size bytes / Размер в байтах: `10881`
- Included characters / Включено символов: `10881`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use makosh_hub_backend::domains::communications::core::NewRawCommunicationRecord;
use makosh_hub_backend::domains::communications::messages::{
    NewProjectedMessage, project_raw_email_message, project_raw_email_message_from_blob,
};
use makosh_hub_backend::domains::communications::storage::LocalCommunicationBlobStore;

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

#[tokio::test]
async fn message_projection_upserts_canonical_message_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_projection_{suffix}");
    let raw_record_id = format!("raw_message_projection_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection Gmail",
        format!("projection-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &format!("provider-message-{suffix}"),
        "Projected subject",
        "Projected body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(projected.observation_id, raw.observation_id);
    assert_eq!(
        projected.provider_record_id,
        format!("provider-message-{suffix}")
    );
    assert_eq!(projected.subject, "Projected subject");
    assert_eq!(projected.sender, "alice@example.com");
    assert_eq!(projected.recipients, vec!["bob@example.com".to_owned()]);
}

#[tokio::test]
async fn message_projection_extracts_canonical_fields_from_raw_blob_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message raw blob projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_blob_projection_{suffix}");
    let raw_record_id = format!("raw_message_blob_projection_{suffix}");
    let provider_record_id = format!("provider-message-blob-{suffix}");
    let blob_root = tempfile::tempdir().expect("blob root");
    let blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let local_blob = blob_store
        .put_blob(
            b"Subject: Real MIME\r\nFrom: Alice <alice@example.com>\r\nTo: Bob <bob@example.com>\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nHello=20from=20real=20mail.",
        )
        .await
        .expect("write raw mail blob");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection raw blob",
        format!("projection-raw-blob-{suffix}@example.com"),
    )
    .await;
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "provider": "imap",
                    "raw_blob_storage_kind": local_blob.storage_kind,
                    "raw_blob_storage_path": local_blob.storage_path,
                    "raw_blob_sha256": local_blob.sha256,
                    "raw_blob_size_bytes": local_blob.size_bytes
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"email_provider_sync"})),
        )
        .await
        .expect("record raw blob message");

    let projected = project_raw_email_message_from_blob(&message_store, &blob_store, &raw)
        .await
        .expect("project message from raw blob");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(projected.observation_id, raw.observation_id);
    assert_eq!(projected.provider_record_id, provider_record_id);
    assert_eq!(projected.subject, "Real MIME");
    assert_eq!(projected.sender, "Alice <alice@example.com>");
    assert_eq!(
        projected.recipients,
        vec!["Bob <bob@example.com>".to_owned()]
    );
    assert_eq!(projected.body_text, "Hello from real mail.");
}

#[tokio::test]
async fn message_projection_distinguishes_delimiter_bearing_identities_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("delimiter-bearing message projection identities").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let base_account_id = format!("acct_message_identity_{suffix}");
    let left_account_id = format!("{base_account_id}:left");

    store_provider_account(
        &communication_store,
        &base_account_id,
        "Projection identity base",
        format!("projection-identity-base-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &left_account_id,
        "Projection identity left",
        format!("projection-identity-left-{suffix}@example.com"),
    )
    .await;

    let base_raw = record_raw_email_message(
        &communication_store,
        &base_account_id,
        &format!("raw_message_identity_base_{suffix}"),
        "left:right",
        "Delimiter subject base",
        "Delimiter body base",
    )
    .await;
    let left_raw = record_raw_email_message(
        &communication_store,
        &left_account_id,
        &format!("raw_message_identity_left_{suffix}"),
        "right",
        "Delimiter subject left",
        "Delimiter body left",
    )
    .await;

    let base_projected = project_raw_email_message(&message_store, &base_raw)
        .await
        .expect("project base delimiter message");
    let left_projected = project_raw_email_message(&message_store, &left_raw)
        .await
        .expect("project left delimiter message");

    assert_ne!(base_projected.message_id, left_projected.message_id);
    assert_eq!(base_projected.account_id, base_account_id);
    assert_eq!(base_projected.provider_record_id, "left:right");
    assert_eq!(left_projected.account_id, left_account_id);
    assert_eq!(left_projected.provider_record_id, "right");

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE account_id IN ($1, $2)
        "#,
    )
    .bind(&base_projected.account_id)
    .bind(&left_projected.account_id)
    .fetch_one(&pool)
    .await
    .expect("projected delimiter message count");
    assert_eq!(count, 2);
}

#[tokio::test]
async fn message_projection_reprojects_same_raw_record_idempotently_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("idempotent message projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_idempotent_{suffix}");
    let provider_record_id = format!("provider-message-idempotent-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection idempotent Gmail",
        format!("projection-idempotent-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_idempotent_{suffix}"),
        &provider_record_id,
        "Idempotent subject",
        "Idempotent body",
    )
    .await;

    let first = project_raw_email_message(&message_store, &raw)
        .await
        .expect("first message projection");
    let second = project_raw_email_message(&message_store, &raw)
        .await
        .expect("second message projection");

    assert_eq!(second.message_id, first.message_id);
    assert_eq!(second.raw_record_id, first.raw_record_id);
    assert_eq!(second.account_id, first.account_id);
    assert_eq!(second.provider_record_id, first.provider_record_id);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("idempotent projected message count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn message_projection_derives_message_id_for_direct_upsert_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("direct message upsert identity derivation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_direct_identity_{suffix}");
    let provider_record_id = format!("provider-message-direct-identity-{suffix}");
    let arbitrary_message_id = format!("not-canonical-message-id-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection direct identity",
        format!("projection-direct-identity-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_direct_identity_{suffix}"),
        &provider_record_id,
        "Direct identity subject",
        "Direct identity body",
    )
    .await;
    let message = NewProjectedMessage {
        message_id: arbitrary_message_id.clone(),
        raw_record_id: raw.raw_record_id,
        account_id,
        provider_record_id,
        subject: "Direct identity subject".to_owned(),
        sender: "alice@example.com".to_owned(),
        recipients: vec!["bob@example.com".to_owned()],
        body_text: "Direct identity body".to_owned(),
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some("alice@example.com".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    let projected = message_store
        .upsert_message(&message)
        .await
        .expect("direct upsert derives canonical message ID");

    assert_ne!(projected.message_id, arbitrary_message_id);
    assert!(projected.message_id.starts_with("msg:v1:"));
    assert_eq!(projected.observation_id, raw.observation_id);

    let arbitrary_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_messages WHERE message_id = $1",
    )
    .bind(&arbitrary_message_id)
    .fetch_one(&pool)
    .await
    .expect("arbitrary message ID count");
    assert_eq!(arbitrary_count, 0);
}
```

### `backend/tests/messages/projection_queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages/projection_queries.rs`
- Size bytes / Размер в байтах: `19893`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;

use makosh_hub_backend::domains::communications::messages::{
    LocalMessageState, MessageProjectionError, MessageSearchMatchMode, MessageSearchQuery,
    NewProjectedMessage, ProjectedMessagePageQuery, WorkflowState, project_raw_email_message,
};

use super::support::{
    disconnected_message_store, live_projection_context, record_raw_email_message,
    store_provider_account, stored_raw_record_with_payload, unique_suffix,
};

#[tokio::test]
async fn message_projection_list_messages_filters_by_account_state_channel_and_query_against_postgres()
 {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message filtered listing").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_left = format!("acct_message_filter_left_{suffix}");
    let account_right = format!("acct_message_filter_right_{suffix}");

    store_provider_account(
        &communication_store,
        &account_left,
        "Filter Left",
        format!("filter-left-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &account_right,
        "Filter Right",
        format!("filter-right-{suffix}@example.com"),
    )
    .await;

    let left_raw = record_raw_email_message(
        &communication_store,
        &account_left,
        &format!("raw_message_filter_left_{suffix}"),
        &format!("provider-filter-left-{suffix}"),
        "Quarterly Alpha Contract",
        "The alpha renewal needs a legal review.",
    )
    .await;
    let right_raw = record_raw_email_message(
        &communication_store,
        &account_right,
        &format!("raw_message_filter_right_{suffix}"),
        &format!("provider-filter-right-{suffix}"),
        "Quarterly Beta Invoice",
        "The beta invoice is already paid.",
    )
    .await;

    let left = project_raw_email_message(&message_store, &left_raw)
        .await
        .expect("project left message");
    let right = project_raw_email_message(&message_store, &right_raw)
        .await
        .expect("project right message");
    message_store
        .transition_workflow_state(&left.message_id, WorkflowState::NeedsAction)
        .await
        .expect("set left state");
    message_store
        .transition_workflow_state(&right.message_id, WorkflowState::Reviewed)
        .await
        .expect("set right state");

    let filtered = message_store
        .list_messages(
            Some(&account_left),
            Some(WorkflowState::NeedsAction),
            Some("email"),
            Some("alpha legal"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list filtered messages");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message.message_id, left.message_id);
    assert_eq!(filtered[0].message.account_id, account_left);

    let no_match = message_store
        .list_messages(
            Some(&account_left),
            Some(WorkflowState::NeedsAction),
            Some("email"),
            Some("beta"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list non-matching messages");
    assert!(no_match.is_empty());
}

#[tokio::test]
async fn message_projection_channel_kind_telegram_alias_matches_user_and_bot_messages_against_postgres()
 {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message telegram channel alias").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_telegram_alias_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Telegram Alias",
        format!("telegram-alias-{suffix}@example.com"),
    )
    .await;

    let user_raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_telegram_alias_user_{suffix}"),
        &format!("provider-telegram-alias-user-{suffix}"),
        "Telegram User",
        "User channel body",
    )
    .await;
    let bot_raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_telegram_alias_bot_{suffix}"),
        &format!("provider-telegram-alias-bot-{suffix}"),
        "Telegram Bot",
        "Bot channel body",
    )
    .await;
    let whatsapp_raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_telegram_alias_whatsapp_{suffix}"),
        &format!("provider-telegram-alias-whatsapp-{suffix}"),
        "WhatsApp",
        "WhatsApp channel body",
    )
    .await;

    let user_message = NewProjectedMessage {
        message_id: format!("msg:telegram-alias:user:{suffix}"),
        raw_record_id: user_raw.raw_record_id.clone(),
        account_id: account_id.clone(),
        provider_record_id: user_raw.provider_record_id.clone(),
        subject: "Telegram User".to_owned(),
        sender: "telegram:user:42".to_owned(),
        recipients: vec![],
        body_text: "User channel body".to_owned(),
        occurred_at: user_raw.occurred_at,
        channel_kind: "telegram_user".to_owned(),
        conversation_id: Some(format!("conversation:telegram-alias:{suffix}")),
        sender_display_name: Some("Ada".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };
    let bot_message = NewProjectedMessage {
        message_id: format!("msg:telegram-alias:bot:{suffix}"),
        raw_record_id: bot_raw.raw_record_id.clone(),
        account_id: account_id.clone(),
        provider_record_id: bot_raw.provider_record_id.clone(),
        subject: "Telegram Bot".to_owned(),
        sender: "telegram:bot:7".to_owned(),
        recipients: vec![],
        body_text: "Bot channel body".to_owned(),
        occurred_at: bot_raw.occurred_at,
        channel_kind: "telegram_bot".to_owned(),
        conversation_id: Some(format!("conversation:telegram-alias:{suffix}")),
        sender_display_name: Some("Build Bot".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };
    let whatsapp_message = NewProjectedMessage {
        message_id: format!("msg:telegram-alias:whatsapp:{suffix}"),
        raw_record_id: whatsapp_raw.raw_record_id.clone(),
        account_id: account_id.clone(),
        provider_record_id: whatsapp_raw.provider_record_id.clone(),
        subject: "WhatsApp".to_owned(),
        sender: "whatsapp:user:9".to_owned(),
        recipients: vec![],
        body_text: "WhatsApp channel body".to_owned(),
        occurred_at: whatsapp_raw.occurred_at,
        channel_kind: "whatsapp_web".to_owned(),
        conversation_id: Some(format!("conversation:whatsapp-alias:{suffix}")),
        sender_display_name: Some("Grace".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    let user = message_store
        .upsert_channel_message(&user_message)
        .await
        .expect("upsert telegram user message");
    let bot = message_store
        .upsert_channel_message(&bot_message)
        .await
        .expect("upsert telegram bot message");
    message_store
        .upsert_channel_message(&whatsapp_message)
        .await
        .expect("upsert whatsapp control message");

    let page = message_store
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: Some(&account_id),
            workflow_state: None,
            channel_kind: Some("telegram"),
            conversation_id: None,
            query: None,
            match_mode: MessageSearchMatchMode::All,
            search: MessageSearchQuery::default(),
            local_state: LocalMessageState::Active,
            cursor: None,
            limit: 10,
        })
        .await
        .expect("list telegram channel alias messages");

    let ids = page
        .items
        .iter()
        .map(|summary| summary.message.message_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&user.message_id.as_str()));
    assert!(ids.contains(&bot.message_id.as_str()));
}

#[tokio::test]
async fn message_local_trash_hides_from_default_lists_and_survives_reprojection_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message local trash").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_local_trash_{suffix}");
    let raw_record_id = format!("raw_message_local_trash_{suffix}");
    let provider_record_id = format!("provider-local-trash-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Local Trash Gmail",
        format!("local-trash-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &provider_record_id,
        "Local trash subject",
        "Local trash body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project local trash message");
    assert_eq!(projected.local_state, LocalMessageState::Active);

    let trashed = message_store
        .move_to_local_trash(&projected.message_id, "user_deleted")
        .await
        .expect("move message to local trash");
    assert_eq!(trashed.local_state, LocalMessageState::Trash);
    assert_eq!(trashed.local_state_reason.as_deref(), Some("user_deleted"));
    assert!(trashed.local_state_changed_at.is_some());

    let default_messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            Some("Local trash"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list active messages");
    assert!(default_messages.is_empty());

    let trash_messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            Some("Local trash"),
            LocalMessageState::Trash,
            10,
        )
        .await
        .expect("list trash messages");
    assert_eq!(trash_messages.len(), 1);
    assert_eq!(trash_messages[0].message.message_id, projected.message_id);

    let reprojected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("reproject local trash message");
    assert_eq!(reprojected.local_state, LocalMessageState::Trash);

    let restored = message_store
        .restore_from_local_trash(&projected.message_id)
        .await
        .expect("restore local trash message");
    assert_eq!(restored.local_state, LocalMessageState::Active);

    let restored_messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            Some("Local trash"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list restored messages");
    assert_eq!(restored_messages.len(), 1);
}

#[tokio::test]
async fn message_search_supports_any_mode_and_field_rules_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message search rules").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_search_rules_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Search Rules Gmail",
        format!("search-rules-{suffix}@example.com"),
    )
    .await;

    let quarterly = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_search_rules_quarterly_{suffix}"),
        &format!("provider-message-search-rules-quarterly-{suffix}"),
        "Quarterly Report",
        "Payment follow-up for the
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/messages/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages/support.rs`
- Size bytes / Размер в байтах: `3664`
- Included characters / Включено символов: `3664`
- Truncated / Обрезано: `no`

```rust
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::MessageProjectionStore;
use makosh_hub_backend::platform::storage::Database;

pub async fn live_projection_context(
    _test_name: &str,
) -> Option<(PgPool, CommunicationIngestionStore, MessageProjectionStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((
        pool.clone(),
        CommunicationIngestionStore::new(pool.clone()),
        MessageProjectionStore::new(pool),
    ))
}

pub async fn store_provider_account(
    store: &CommunicationIngestionStore,
    account_id: &str,
    display_name: &str,
    external_account_id: String,
) {
    store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            display_name,
            external_account_id,
        ))
        .await
        .expect("store provider account");
}

pub async fn record_raw_email_message(
    store: &CommunicationIngestionStore,
    account_id: &str,
    raw_record_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> StoredRawCommunicationRecord {
    store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                raw_record_id,
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "subject": subject,
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record raw message")
}

pub fn disconnected_message_store() -> MessageProjectionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    MessageProjectionStore::new(pool)
}

pub fn stored_raw_record_with_payload(payload: Value) -> StoredRawCommunicationRecord {
    let suffix = unique_suffix();

    StoredRawCommunicationRecord {
        raw_record_id: format!("raw_payload_validation_{suffix}"),
        observation_id: format!("observation:v1:raw-payload-validation-{suffix}"),
        account_id: format!("acct_payload_validation_{suffix}"),
        record_kind: "email_message".to_owned(),
        provider_record_id: format!("provider-payload-validation-{suffix}"),
        source_fingerprint: format!("sha256:payload-validation-{suffix}"),
        import_batch_id: format!("batch_payload_validation_{suffix}"),
        occurred_at: Some(Utc::now()),
        captured_at: Utc::now(),
        payload,
        provenance: json!({"source":"test"}),
    }
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```
