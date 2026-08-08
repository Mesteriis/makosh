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

- Chunk ID / ID чанка: `093-test-backend-part-016`
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

### `backend/tests/v1_communications_templates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_templates.rs`
- Size bytes / Размер в байтах: `7625`
- Included characters / Включено символов: `7625`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-template-test-token";

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, db),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn delete_req(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

#[tokio::test]
async fn rich_template_save_list_render_and_delete_uses_durable_template_store() {
    let context = TestContext::new().await;
    let db = context.connection_string();
    let suffix = uid();
    let template_id = format!("mail-template-{suffix}");
    let r = router(&db).await;

    let save_resp = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/templates/rich",
            json!({
                "template_id": template_id,
                "name": "Project update",
                "subject_template": "Hello {{name}}",
                "body_template": "Project {{project}} is {{status}}.",
                "variables": ["name", "project", "status"],
                "language": "en"
            }),
        ))
        .await
        .expect("save template");
    assert_eq!(save_resp.status(), StatusCode::OK);
    let saved_body: Value =
        serde_json::from_slice(&to_bytes(save_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert_eq!(saved_body["template"]["template_id"], template_id);
    assert_eq!(
        saved_body["template"]["placeholder_variables"],
        json!(["name", "project", "status"])
    );
    assert_eq!(saved_body["template"]["undeclared_variables"], json!([]));
    assert_eq!(saved_body["template"]["unused_variables"], json!([]));
    assert_eq!(saved_body["template"]["malformed_placeholders"], json!([]));

    let list_resp = r
        .clone()
        .oneshot(get("/api/v1/communications/templates/rich"))
        .await
        .expect("list templates");
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: Value =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert!(
        list_body["templates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["template_id"] == template_id)
    );
    let listed_template = list_body["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|template| template["template_id"] == template_id)
        .unwrap();
    assert_eq!(
        listed_template["placeholder_variables"],
        json!(["name", "project", "status"])
    );

    let render_resp = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/templates/rich/render",
            json!({
                "template_id": template_id,
                "variables": {
                    "name": "Alex",
                    "project": "Макошь",
                    "status": "green"
                }
            }),
        ))
        .await
        .expect("render template");
    assert_eq!(render_resp.status(), StatusCode::OK);
    let render_body: Value = serde_json::from_slice(
        &to_bytes(render_resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(render_body["rendered"]["subject"], "Hello Alex");
    assert_eq!(render_body["rendered"]["body"], "Project Макошь is green.");
    assert_eq!(render_body["rendered"]["missing_variables"], json!([]));
    assert_eq!(render_body["rendered"]["unresolved_variables"], json!([]));
    assert_eq!(render_body["rendered"]["malformed_placeholders"], json!([]));

    let preview_resp = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/templates/rich/mail-merge-preview",
            json!({
                "template_id": template_id,
                "rows": [
                    {
                        "row_id": "r1",
                        "variables": {
                            "name": "Alex",
                            "project": "Макошь",
                            "status": "green"
                        }
                    },
                    {
                        "row_id": "r2",
                        "variables": {
                            "name": "Sam",
                            "project": "Макошь"
                        }
                    }
                ]
            }),
        ))
        .await
        .expect("mail merge preview");
    assert_eq!(preview_resp.status(), StatusCode::OK);
    let preview_body: Value = serde_json::from_slice(
        &to_bytes(preview_resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(preview_body["template_id"], template_id);
    assert_eq!(preview_body["row_count"], 2);
    assert_eq!(preview_body["ready_count"], 1);
    assert_eq!(preview_body["blocked_count"], 1);
    assert_eq!(preview_body["items"][0]["row_id"], "r1");
    assert_eq!(preview_body["items"][0]["ready"], true);
    assert_eq!(
        preview_body["items"][0]["rendered"]["subject"],
        "Hello Alex"
    );
    assert_eq!(
        preview_body["items"][0]["rendered"]["body"],
        "Project Макошь is green."
    );
    assert_eq!(preview_body["items"][1]["row_id"], "r2");
    assert_eq!(preview_body["items"][1]["ready"], false);
    assert_eq!(
        preview_body["items"][1]["rendered"]["missing_variables"],
        json!(["status"])
    );

    let delete_path = format!("/api/v1/communications/templates/rich/{template_id}");
    let delete_resp = r
        .clone()
        .oneshot(delete_req(&delete_path))
        .await
        .expect("delete template");
    assert_eq!(delete_resp.status(), StatusCode::OK);
    let delete_body: Value = serde_json::from_slice(
        &to_bytes(delete_resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(delete_body["template_id"], template_id);
    assert_eq!(delete_body["deleted"], true);

    let list_resp = r
        .oneshot(get("/api/v1/communications/templates/rich"))
        .await
        .expect("list templates after delete");
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: Value =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert!(
        !list_body["templates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["template_id"] == template_id)
    );
}
```

### `backend/tests/v1_workflow_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_workflow_actions.rs`
- Size bytes / Размер в байтах: `22104`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const T: &str = "v1-workflow-action-test-token";

fn cfg() -> AppConfig {
    testkit::app::config_with_secret(T)
}

fn post_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .header("x-makosh-actor-id", "makosh-frontend")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

#[tokio::test]
async fn workflow_action_endpoint_exists_without_database() {
    let app = build_router(cfg());
    let response = app
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": "workflow-action-no-db",
                "action": "reply",
                "source": { "kind": "communication_message", "id": "msg:no-db" }
            }),
        ))
        .await
        .expect("workflow action no-db response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn v1_put_workflow_state_captures_observation_trail() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-state-{suffix}"),
        &format!("provider-workflow-state-{suffix}"),
        &format!("Workflow state {suffix}"),
    )
    .await;
    let r = router(&db).await;
    let response = r
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/workflow-state"),
            json!({"workflow_state": "reviewed"}),
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], json!(message_id));
    assert_eq!(body["workflow_state"], "reviewed");
    assert_eq!(body["previous_state"], "new");

    let workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state");
    assert_eq!(workflow_state, "reviewed");

    let row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'workflow_state_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state observation link");
    let observation_id: String = row.try_get("observation_id").expect("observation id");
    let metadata: Value = row.try_get("metadata").expect("metadata");
    assert_eq!(metadata["previous_state"], "new");
    assert_eq!(metadata["workflow_state"], "reviewed");

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("observation origin");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn workflow_action_create_task_is_idempotent_and_records_safe_event() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-action-{suffix}"),
        &format!("provider-workflow-action-{suffix}"),
        &format!("Workflow action task {suffix}"),
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let r = router(&db).await;
    let command_id = format!("workflow-action-task-{suffix}");
    let body = json!({
        "command_id": command_id,
        "action": "create_task",
        "source": { "kind": "communication_message", "id": message_id },
        "input": { "title": "Confirm integration access" }
    });

    let first = r
        .clone()
        .oneshot(post_with_actor("/api/v1/workflow-actions", body.clone()))
        .await
        .expect("first workflow action response");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    assert_eq!(
        first_body["event_id"],
        json!(format!("workflow_action:{command_id}"))
    );
    assert_eq!(first_body["target"]["kind"], "task");
    assert_eq!(first_body["provenance"]["source_id"], message_id);

    let second = r
        .oneshot(post_with_actor("/api/v1/workflow-actions", body))
        .await
        .expect("second workflow action response");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    assert_eq!(second_body["target"], first_body["target"]);

    let task_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tasks WHERE source_id = $1 AND source_type = 'observation' AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("task count");
    assert_eq!(task_count, 1);

    let task_id = first_body["target"]["id"]
        .as_str()
        .expect("task id")
        .to_owned();

    let task_create_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task'
          AND entity_id = $2
          AND relationship_kind = 'task_create'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task create observation links");
    assert_eq!(task_create_link_count, 1);

    let workflow_projection_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task'
          AND entity_id = $2
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task workflow projection observation links");
    assert_eq!(workflow_projection_link_count, 1);

    let event_payload: Value =
        sqlx::query_scalar("SELECT payload FROM event_log WHERE event_id = $1")
            .bind(format!("workflow_action:{command_id}"))
            .fetch_one(&pool)
            .await
            .expect("workflow event payload");
    assert!(
        !event_payload
            .to_string()
            .contains("Body for local trash API")
    );
}

#[tokio::test]
async fn workflow_action_create_contact_reuses_message_observation_for_person_projection() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-contact-{suffix}"),
        &format!("provider-workflow-contact-{suffix}"),
        &format!("Workflow action contact {suffix}"),
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let r = router(&db).await;
    let command_id = format!("workflow-action-contact-{suffix}");

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "create_contact",
                "source": { "kind": "communication_message", "id": message_id }
            }),
        ))
        .await
        .expect("workflow contact response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "person");
    let person_id = body["target"]["id"].as_str().expect("person id").to_owned();

    let persona_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'persons'
          AND entity_kind = 'persona'
          AND entity_id = $2
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&person_id)
    .fetch_one(&pool)
    .await
    .expect("persona workflow action observation links");
    assert_eq!(persona_link_count, 1);

    let identity_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'persons'
          AND entity_kind = 'identity'
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("identity workflow action observation links");
    assert_eq!(identity_link_count, 1);
}

#[tokio::test]
async fn workflow_action_create_note_creates_markdown_document() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let r = router(&db).await;
    let command_id = format!("workflow-action-note-{suffix}");

    let response = r
        .oneshot(post_wi
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v2_domain_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v2_domain_api.rs`
- Size bytes / Размер в байтах: `6170`
- Included characters / Включено символов: `6170`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::tasks::api::{NewTask, TaskStore};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const LOCAL_API_TOKEN: &str = "v2-domain-api-test-token";

#[tokio::test]
async fn domain_routes_build_and_require_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/tasks"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        json_body(response).await,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );

    let secret_only_response = build_router(config_with_api_token())
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("secret-only response");

    assert_eq!(
        secret_only_response.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let secret_only_body = json_body(secret_only_response).await;
    assert_eq!(secret_only_body["error"], json!("database_not_configured"));
    assert!(secret_only_body["message"].is_string());
}

#[tokio::test]
async fn tasks_endpoint_returns_first_class_task_payload_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let task = TaskStore::new(pool)
        .create(&NewTask {
            title: format!("V1 first-class task {suffix}"),
            description: Some("contract test task".to_owned()),
            source_type: Some("manual".to_owned()),
            makosh_status: Some("ready".to_owned()),
            priority_score: Some(0.7),
            tags: Some(json!(["api-test"])),
            ..Default::default()
        })
        .await
        .expect("seed task");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/tasks?limit=100",
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let item = body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["task_id"] == task.task_id)
        .expect("seeded task item");

    assert_eq!(item["title"], json!(task.title));
    assert_eq!(item["source_type"], json!("observation"));
    assert_eq!(item["makosh_status"], json!("ready"));
    assert_eq!(item["confidentiality"], json!("private_local"));
    assert_eq!(item["task_metadata"], json!({}));
}

#[tokio::test]
async fn person_health_endpoint_returns_single_person_health_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("health-{suffix}@example.com"))
        .await
        .expect("seed person");
    seed_person_health(&pool, &person.person_id).await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token_and_actor(
            &format!("/api/v1/persons/{}/health", person.person_id),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["person_id"], json!(person.person_id));
    assert_eq!(body["health_status"], json!("at_risk"));
    assert_eq!(body["communication_gap_days"], json!(42));
    assert!(body.get("items").is_none());
}

fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token_and_actor(uri: &str, token: &str, _actor_id: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn seed_person_health(pool: &PgPool, person_id: &str) {
    sqlx::query(
        "UPDATE persons SET health_status = 'at_risk', communication_gap_days = 42, watchlist = true WHERE person_id = $1",
    )
    .bind(person_id)
    .execute(pool)
    .await
    .expect("update person health");
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}
```

### `backend/tests/whatsapp.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/whatsapp.rs`
- Size bytes / Размер в байтах: `608209`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::engines::timeline::TimelineEngine;
use makosh_hub_backend::platform::events::{EventLogQuery, EventStore};
use makosh_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{EntropyEvent, HostVault, HostVaultConfig, SecretEntryContext};

const LOCAL_API_TOKEN: &str = "whatsapp-api-test-secret";

#[test]
fn whatsapp_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("whatsapp_web").expect("whatsapp web provider"),
        CommunicationProviderKind::WhatsappWeb
    );
    assert_eq!(
        CommunicationProviderKind::try_from("whatsapp_business_cloud")
            .expect("whatsapp business cloud provider"),
        CommunicationProviderKind::WhatsappBusinessCloud
    );
    assert!(CommunicationProviderKind::WhatsappWeb.is_whatsapp());
    assert!(CommunicationProviderKind::WhatsappBusinessCloud.is_whatsapp());
    assert!(!CommunicationProviderKind::WhatsappWeb.is_email());
    assert!(!CommunicationProviderKind::WhatsappWeb.is_telegram());

    assert!(
        ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::PrivateKey)
    );
    assert!(
        ProviderAccountSecretPurpose::WhatsappWebSessionKey.accepts_secret_kind(SecretKind::Other)
    );
    assert!(
        !ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::Password)
    );
    assert!(
        !ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::ApiToken)
    );
}

#[tokio::test]
async fn whatsapp_business_cloud_fixture_account_uses_api_credential_session_surface() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-business-cloud-fixture-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_business_cloud",
                "display_name": "WhatsApp Business Cloud Fixture",
                "external_account_id": format!("wa-business-cloud-fixture-{suffix}"),
                "device_name": "Fixture API Credential Surface",
                "local_state_path": format!("docker/data/whatsapp/business-cloud-fixture-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture business cloud account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(
        account_body["provider_kind"],
        json!("whatsapp_business_cloud")
    );
    assert_eq!(account_body["runtime"], json!("fixture"));
    assert_eq!(
        account_body["session"]["companion_runtime"],
        json!("api_credentials")
    );
    assert_eq!(account_body["session"]["link_state"], json!("fixture"));
    assert_eq!(
        account_body["session"]["metadata"]["provider_shape"],
        json!("whatsapp_business_cloud")
    );
    assert_eq!(
        account_body["session"]["metadata"]["setup_semantics"],
        json!("business_cloud")
    );
    assert_eq!(
        account_body["session"]["metadata"]["session_mode"],
        json!("api_credentials")
    );
}

#[tokio::test]
async fn whatsapp_native_md_fixture_account_preserves_provider_shape_and_appears_in_aggregate_routes()
 {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-native-fixture-{suffix}");
    let chat_id = format!("wa-native-fixture-chat-{suffix}");

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_native_md",
                "display_name": "WhatsApp Native MD Fixture",
                "external_account_id": format!("wa-native-fixture-{suffix}"),
                "device_name": "Макошь Native MD Fixture",
                "local_state_path": format!("docker/data/whatsapp/native-fixture-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("native fixture account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(account_body["provider_kind"], json!("whatsapp_web"));
    assert_eq!(account_body["runtime"], json!("fixture"));
    assert_eq!(
        account_body["session"]["metadata"]["provider_shape"],
        json!("whatsapp_native_md")
    );

    let status_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("native fixture status response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = json_body(status_response).await;
    assert_eq!(status_body["provider_shape"], json!("whatsapp_native_md"));
    assert_eq!(status_body["runtime_kind"], json!("fixture"));

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("wa-native-fixture-message-{suffix}"),
                "chat_title": "Native fixture chat",
                "sender_id": format!("wa-native-fixture-sender-{suffix}"),
                "sender_display_name": "WhatsApp Native Sender",
                "text": "Native MD fixture message",
                "import_batch_id": format!("whatsapp-native-fixture-{suffix}"),
                "occurred_at": "2026-06-06T13:00:00Z",
                "delivery_state": "delivered"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("native fixture message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let sessions_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/whatsapp/sessions?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("native aggregate sessions response");
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_body = json_body(sessions_response).await;
    let session_items = sessions_body["items"]
        .as_array()
        .expect("native aggregate session items");
    assert!(
        session_items.iter().any(|item| {
            item["account_id"] == json!(account_id)
                && item["metadata"]["provider_shape"] == json!("whatsapp_native_md")
                && item["companion_runtime"] == json!("fixture")
        }),
        "expected native-md fixture session in aggregate list: {sessions_body}"
    );

    let messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/communications/messages?limit=10&channel_kind=whatsapp",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("native aggregate messages response");
    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_body = json_body(messages_response).await;
    let message_items = messages_body["items"]
        .as_array()
        .expect("native aggregate message items");
    assert!(
        message_items.iter().any(|item| {
            item["account_id"] == json!(account_id)
                && item["conversation_id"] == json!(chat_id)
                && item["channel_kind"] == json!("whatsapp_web")
        }),
        "expected native-md fixture message in route response: {messages_body}"
    );
}

#[tokio::test]
async fn whatsapp_provider_neutral_communications_routes_dispatch_to_whatsapp_commands() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-communications-routes-{suffix}");
    let chat_id = format!("wa-communications-chat-{suffix}");
    let provider_message_id = format!("wa-communications-message-{suffix}");

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Communications Routes",
                "external_account_id": format!("wa-communications-routes-{suffix}"),
                "device_name": "Макошь Communications Routes",
                "local_state_path": format!("docker/data/whatsapp/communications-routes-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "WhatsApp Communications",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "WhatsApp Sender",
                "text": "Projected message for provider-neutral dispatch.",
                "import_batch_id": format!("whatsapp-communications-routes-{suffix}"),
                "occurred_at": "2026-06-06T13:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["messag
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/whatsapp_signal_hub.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/whatsapp_signal_hub.rs`
- Size bytes / Размер в байтах: `56104`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::fs;
use std::path::{Path, PathBuf};

use makosh_hub_backend::platform::events::bus::{sanitize_event_payload, whatsapp_event_types};
use serde_json::json;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend has repository parent")
        .to_path_buf()
}

#[test]
fn whatsapp_signal_hub_fixture_matrix_covers_event_families() {
    let root = repo_root();
    let signal_hub_whatsapp = read(root.join("backend/src/domains/signal_hub/whatsapp.rs"));
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let event_producers =
        provider_handler.clone() + &read_all_sources(root.join("backend/src/application"));
    let fixture_matrix = read(root.join("docs/integrations/whatsapp/fixture-test-matrix.md"));

    for fixture in signal_hub_fixtures() {
        if let Some(raw_record_kind) = fixture.raw_record_kind {
            assert!(
                signal_hub_whatsapp.contains(&format!(
                    "\"{raw_record_kind}\" => \"{}\"",
                    fixture.signal_kind
                )),
                "Signal Hub raw WhatsApp mapping missing for {raw_record_kind}"
            );
        } else {
            assert!(
                signal_hub_whatsapp.contains("_ => \"message\""),
                "Signal Hub raw WhatsApp mapping must default unknown message-like records to message"
            );
        }

        assert!(
            signal_hub_whatsapp.contains("signal.raw.whatsapp.{event_kind}.observed"),
            "WhatsApp raw signals must use the Signal Hub raw observed event family"
        );
        assert!(
            fixture_matrix.contains(fixture.matrix_label),
            "WhatsApp fixture-test matrix must document {}",
            fixture.matrix_label
        );
        assert!(
            event_producers.contains(fixture.realtime_event_constant),
            "WhatsApp event producers must emit realtime event constant {}",
            fixture.realtime_event_constant
        );
    }
}

#[test]
fn whatsapp_runtime_lifecycle_event_fixtures_are_sanitized_and_complete() {
    let root = repo_root();
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let event_producers =
        provider_handler + &read_all_sources(root.join("backend/src/application"));
    let event_bus = read(root.join("backend/src/platform/events/bus.rs"));

    for (required, handler_marker) in [
        (
            whatsapp_event_types::RUNTIME_STATUS_CHANGED,
            "RUNTIME_STATUS_CHANGED",
        ),
        (whatsapp_event_types::RUNTIME_EVENT, "RUNTIME_EVENT"),
        (
            whatsapp_event_types::SESSION_LINK_STATE_CHANGED,
            "SESSION_LINK_STATE_CHANGED",
        ),
        (whatsapp_event_types::SYNC_STARTED, "SYNC_STARTED"),
        (whatsapp_event_types::SYNC_PROGRESS, "SYNC_PROGRESS"),
        (whatsapp_event_types::SYNC_COMPLETED, "SYNC_COMPLETED"),
        (whatsapp_event_types::SYNC_FAILED, "SYNC_FAILED"),
        (
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            "COMMAND_STATUS_CHANGED",
        ),
        (
            whatsapp_event_types::COMMAND_RECONCILED,
            "COMMAND_RECONCILED",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_REQUESTED,
            "MEDIA_UPLOAD_REQUESTED",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_STARTED,
            "whatsapp.media.upload.started",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_PROGRESS,
            "whatsapp.media.upload.progress",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_COMPLETED,
            "whatsapp.media.upload.completed",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_FAILED,
            "MEDIA_UPLOAD_FAILED",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_REQUESTED,
            "MEDIA_DOWNLOAD_REQUESTED",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_STARTED,
            "whatsapp.media.download.started",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_PROGRESS,
            "whatsapp.media.download.progress",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_COMPLETED,
            "whatsapp.media.download.completed",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_FAILED,
            "MEDIA_DOWNLOAD_FAILED",
        ),
    ] {
        assert!(
            event_bus.contains(required),
            "WhatsApp event bus constants must expose {required}"
        );
        assert!(
            event_producers.contains(handler_marker),
            "WhatsApp provider handler/runtime bridge or worker must emit {required}"
        );
    }

    let sanitized = sanitize_event_payload(json!({
        "account_id": "wa-1",
        "session_key": "secret-session",
        "access_token": "secret-token",
        "password": "secret-password",
        "raw_body": "private body",
        "safe": "kept"
    }));

    assert_eq!(sanitized["safe"], json!("kept"));
    for secret_key in ["session_key", "access_token", "password", "raw_body"] {
        assert!(
            sanitized.get(secret_key).is_none(),
            "sanitized WhatsApp event payload must remove {secret_key}"
        );
    }
}

#[test]
fn whatsapp_live_smoke_evidence_requires_typed_sanitized_refs() {
    let root = repo_root();
    let evidence_validator = read(root.join("scripts/whatsapp-live-smoke-evidence.mjs"));
    let readiness = read(root.join("scripts/whatsapp-live-smoke-readiness.mjs"));
    let checklist = read(root.join("docs/integrations/whatsapp/live-smoke-checklist.md"));
    let status = read(root.join("docs/integrations/whatsapp/status.md"));

    assert!(
        evidence_validator.contains("allowedEvidenceRefPrefixes")
            && evidence_validator.contains("requiredEvidenceRefPrefixGroups")
            && evidence_validator.contains("evidence_refs")
            && evidence_validator.contains("'raw_record:'")
            && evidence_validator.contains("'event_log:'")
            && evidence_validator.contains("'signal_hub:'")
            && evidence_validator.contains("'command:'")
            && evidence_validator.contains("'vault_binding:'")
            && evidence_validator.contains("'blob:'")
            && evidence_validator.contains("'edge_proxy:'")
            && evidence_validator.contains("'log_scan:'"),
        "Live-smoke evidence must require typed sanitized references for raw evidence, events, commands, vault bindings, media, edge proxy and redaction checks"
    );
    assert!(
        evidence_validator
            .contains("'commands.no_completion_without_provider_observed_evidence': [")
            && evidence_validator.contains("['event_log:', 'signal_hub:']")
            && evidence_validator.contains("gateId.startsWith('outbound.')")
            && evidence_validator.contains("return [['command:'], ['event_log:', 'signal_hub:']]")
            && evidence_validator.contains("weak_reconciliation_refs_fail")
            && evidence_validator.contains("placeholder_refs_fail")
            && evidence_validator
                .contains("evidence.${gateId}.evidence_refs must include at least one"),
        "Live-smoke evidence must reject weak provider-write evidence that lacks command plus provider-observed event refs"
    );
    assert!(
        checklist.contains("Each passed gate must also include concrete sanitized `evidence_refs`")
            && checklist.contains("`command:` plus observed event refs")
            && checklist.contains("`vault_binding:` for session/credential binding")
            && checklist.contains("Placeholder refs")
            && readiness.contains("strict live-smoke evidence references")
            && status.contains("65. `strict live-smoke evidence references`"),
        "Docs and readiness must expose the stricter live-smoke evidence contract used by the closure audit"
    );
}

#[test]
fn whatsapp_live_smoke_evidence_collector_is_not_a_bypass() {
    let root = repo_root();
    let collector = read(root.join("scripts/whatsapp-live-smoke-collect-evidence.mjs"));
    let evidence_validator = read(root.join("scripts/whatsapp-live-smoke-evidence.mjs"));
    let makefile = read(root.join("Makefile"));
    let readiness = read(root.join("scripts/whatsapp-live-smoke-readiness.mjs"));
    let checklist = read(root.join("docs/integrations/whatsapp/live-smoke-checklist.md"));
    let status = read(root.join("docs/integrations/whatsapp/status.md"));

    assert!(
        evidence_validator.contains("--provider-shape")
            && evidence_validator.contains("templateEvidence(providerShape, status)")
            && evidence_validator.contains("template status must be pending or passed"),
        "Evidence validator templates must be provider-shape aware so each runtime shape can produce its own smoke artifact"
    );
    assert!(
        collector
            .contains("defaultObservationsPath = '.local/whatsapp/live-smoke-observations.json'")
            && collector.contains("whatsapp-live-smoke-evidence.mjs")
            && collector.contains("--observations-template")
            && collector.contains("MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID")
            && collector.contains("sha256Fingerprint")
            && collector.contains("assertNoSecretLikeContent")
            && collector.contains("Gates without operator-provided sanitized refs remain pending")
            && collector.contains("mergeEvidence(template, observations)")
            && collector.contains("validateEvidence(filePath)")
            && collector.contains("process.exitCode = 1"),
        "Live-smoke collector must normalize sanitized observations and then fail through the strict validator until all gates are genuinely evidenced"
    );
    assert!(
        makefile.contains("whatsapp-live-smoke-collect-evidence:")
            && readiness.contains("manual_smoke_evidence_collector_contract")
            && checklist.contains("make whatsapp-live-smoke-collect-evidence")
            && checklist.contains("normalizer, not a bypass")
            && checklist.contains("Gates without operator-provided sanitized")
            && status.contains("67. `live-smoke evidence collector`")
            && status.contains("synthetic passed")
            && status.contains("gates."),
        "Makefile, readiness and docs must expose the collector as a non-bypass path for producing live-smoke artifacts"
    );
}

#[test]
fn whatsapp_native_md_upgrade_path_is_executable_evidence_not_assumption() {
    let root = repo_root();
    let gap_readiness = read(root.join("scripts/whatsapp-native-md-sdk-gap-readiness.mjs"));
    let cargo_toml = read(root.join("backend/Cargo.toml"));
    let cargo_lock = read(root.join("Cargo.lock"));
    let status = read(root.join("docs/integrations/whatsapp/status.md"));

    assert!(
        gap_readiness.contains("verifyRustAndCrateUpgradeContext()")
            && gap_readiness.contains("native_md_rust_baseline_context")
            && gap_readiness.contains("native_md_wa_rs_dependency_context")
            && gap_readiness.contains("native_md_crates_io_probe")
            && gap_readiness.contains("native_md_upgrade_requires_provider_api_not_toolchain_only")
            && gap_readiness.contains("MAKOSH_WA_RS_CRATES_IO_PROBE=1")
            && gap_readiness.contains("cargo info"),
        "Native MD gap readiness must make the Rust/wa-rs upgrade path executable evidence, not an assumption"
    );
    assert!(
        cargo_toml.contains("rust-version = \"1.89\"")
            && cargo_toml.contains("wa-rs = { version = \"0.2.0\"")
            && cargo_toml.contains("wa-rs-core = { version = \"0.2.0\"")
            && cargo_lock.contains("name = \"wa-rs\"\nversion = \"0.2.0\"")
            && cargo_lock.contains("name = \"wa-rs-core\"\nversion = \"0.2.0\""),
        "
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/yandex_telemost_calendar_matching.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/yandex_telemost_calendar_matching.rs`
- Size bytes / Размер в байтах: `3055`
- Included characters / Включено символов: `3055`
- Truncated / Обрезано: `no`

```rust
use chrono::{TimeZone, Utc};
use makosh_hub_backend::domains::calendar::core::EventParticipantPort;
use makosh_hub_backend::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use makosh_hub_backend::platform::events::EventEnvelope;
use makosh_hub_backend::platform::events::bus::yandex_telemost_event_types;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::yandex_telemost_calendar_matching::project_yandex_telemost_calendar_matching;
use serde_json::json;
use testkit::context::TestContext;

const TELEMOST_PARTICIPANT_SOURCE: &str = "yandex_telemost_cohost_observed";

#[tokio::test]
async fn telemost_cohosts_are_projected_into_matched_calendar_event_participants() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    let start_at = Utc
        .with_ymd_and_hms(2026, 6, 28, 10, 0, 0)
        .single()
        .expect("valid datetime");
    let end_at = Utc
        .with_ymd_and_hms(2026, 6, 28, 11, 0, 0)
        .single()
        .expect("valid datetime");
    let event = CalendarEventStore::new(pool.clone())
        .create_manual(&NewCalendarEvent {
            title: "Telemost planning".to_owned(),
            start_at,
            end_at,
            conference_url: Some("https://telemost.yandex.ru/j/abcdef".to_owned()),
            conference_provider: Some("yandex_telemost".to_owned()),
            ..NewCalendarEvent::default()
        })
        .await
        .expect("calendar event");

    let projection_event = EventEnvelope {
        event_id: "evt-telemost-cohosts".to_owned(),
        event_type: yandex_telemost_event_types::COHOSTS_OBSERVED.to_owned(),
        schema_version: 1,
        occurred_at: end_at,
        recorded_at: end_at,
        source: json!({}),
        actor: Some(json!({})),
        subject: json!({}),
        payload: json!({
            "account_id": "telemost-main",
            "conference_id": "abcdef",
            "cohosts": [
                { "email": "cohost1@yandex.ru" },
                { "email": "COHOST1@YANDEX.RU" },
                { "email": "cohost2@yandex.ru" }
            ]
        }),
        provenance: json!({}),
        causation_id: None,
        correlation_id: None,
    };

    project_yandex_telemost_calendar_matching(&pool, &projection_event)
        .await
        .expect("calendar participant projection");

    let participants = EventParticipantPort::new(pool)
        .list(&event.event_id)
        .await
        .expect("event participants");
    let projected = participants
        .into_iter()
        .filter(|participant| participant.source == TELEMOST_PARTICIPANT_SOURCE)
        .collect::<Vec<_>>();

    assert_eq!(projected.len(), 2);
    assert_eq!(projected[0].role, "attendee");
    assert_eq!(projected[0].email, "cohost1@yandex.ru");
    assert_eq!(projected[1].email, "cohost2@yandex.ru");
}
```

### `backend/tests/zoom_calendar_matching.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/zoom_calendar_matching.rs`
- Size bytes / Размер в байтах: `6153`
- Included characters / Включено символов: `6153`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::core::EventRelationStore;
use makosh_hub_backend::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_hub_backend::integrations::zoom::client::{ZoomMeetingObservationRequest, ZoomStore};
use makosh_hub_backend::platform::calls::CallIntelligenceStore;
use makosh_hub_backend::platform::events::{EventBus, EventLogQuery, EventStore};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::zoom_calendar_matching::{
    ZOOM_CALENDAR_RELATION_TYPE, project_zoom_calendar_matching,
};
use serde_json::json;
use testkit::context::TestContext;

#[tokio::test]
async fn zoom_meeting_events_match_calendar_events_into_call_relations() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let started_at = Utc::now();
    let ended_at = started_at + Duration::minutes(45);
    let join_url = format!("https://example.zoom.us/j/987654321?pwd={suffix}");

    let calendar_event = CalendarEventStore::new(pool.clone())
        .create(&NewCalendarEvent {
            title: format!("Zoom Match {suffix}"),
            start_at: started_at - Duration::minutes(5),
            end_at: ended_at + Duration::minutes(5),
            conference_url: Some(join_url.clone()),
            conference_provider: Some("zoom".to_owned()),
            event_type: Some("meeting".to_owned()),
            ..Default::default()
        })
        .await
        .expect("calendar event");

    let event_bus = EventBus::new();
    let zoom_store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::storage::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool.clone()),
        event_bus,
    );
    let account_id = format!("zoom-calendar-match-{suffix}");
    zoom_store
        .setup_fixture_account(
            &makosh_hub_backend::integrations::zoom::client::ZoomAccountSetupRequest {
                account_id: account_id.clone(),
                display_name: "Zoom Calendar Match Fixture".to_owned(),
                external_account_id: format!("zoom-calendar-match-external-{suffix}"),
                account_email: None,
                metadata: json!({}),
            },
        )
        .await
        .expect("fixture account");

    let meeting_id = "987654321".to_owned();
    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-calendar-match-observation-{suffix}")),
            account_id: account_id.clone(),
            meeting_id: meeting_id.clone(),
            meeting_uuid: Some(format!("zoom-calendar-match-uuid-{suffix}")),
            topic: Some("Weekly review".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(join_url.clone()),
            started_at: Some(started_at),
            ended_at: Some(ended_at),
            duration_seconds: Some(45 * 60),
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({ "source": "zoom_calendar_matching_test" }),
            causation_id: None,
            correlation_id: Some(format!("zoom-calendar-match-correlation-{suffix}")),
        })
        .await
        .expect("observe zoom meeting");
    let call_id = observed.call_id;

    let direct_match = CalendarEventStore::new(pool.clone())
        .find_zoom_conference_match(
            Some(&join_url),
            &meeting_id,
            Some(started_at),
            Some(ended_at),
        )
        .await
        .expect("direct calendar match");
    assert_eq!(
        direct_match.as_ref().map(|event| event.event_id.as_str()),
        Some(calendar_event.event_id.as_str())
    );

    let stored_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(format!("zoom-calendar-match-correlation-{suffix}")),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("stored zoom events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(call_id))
        .expect("matched stored zoom event");
    assert_eq!(stored_event.event.payload["meeting_id"], json!(meeting_id));
    assert_eq!(stored_event.event.payload["join_url"], json!(join_url));
    assert_eq!(stored_event.event.payload["started_at"], json!(started_at));

    project_zoom_calendar_matching(&pool, &stored_event.event)
        .await
        .expect("zoom calendar matching projection");

    let relations = EventRelationStore::new(pool.clone())
        .list(&calendar_event.event_id)
        .await
        .expect("calendar event relations");
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].entity_type, "call");
    assert_eq!(relations[0].entity_id, call_id);
    assert_eq!(relations[0].relation_type, ZOOM_CALENDAR_RELATION_TYPE);
    assert_eq!(relations[0].source, "zoom.meeting.observed");

    project_zoom_calendar_matching(&pool, &stored_event.event)
        .await
        .expect("repeat zoom calendar matching");

    let relations_after_repeat = EventRelationStore::new(pool)
        .list(&calendar_event.event_id)
        .await
        .expect("calendar event relations after repeat");
    assert_eq!(relations_after_repeat.len(), 1);
}
```

### `backend/tests/zoom_participant_identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/zoom_participant_identity.rs`
- Size bytes / Размер в байтах: `8088`
- Included characters / Включено символов: `8088`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::integrations::zoom::client::{
    ZoomAccountSetupRequest, ZoomMeetingObservationRequest, ZoomParticipantSnapshot, ZoomStore,
};
use makosh_hub_backend::platform::calls::CallIntelligenceStore;
use makosh_hub_backend::platform::events::{EventBus, EventLogQuery, EventStore};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::review_inbox::project_person_identity_review_event;
use makosh_hub_backend::workflows::zoom_participant_identity::project_zoom_participant_identity;
use serde_json::json;
use sqlx::Row;
use testkit::context::TestContext;

#[tokio::test]
async fn zoom_participant_identity_candidates_flow_into_review_inbox() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());

    let person_store = PersonProjectionStore::new(pool.clone());
    let matched_person = person_store
        .upsert_email_person(&format!("existing-zoom-person-{suffix}@example.com"))
        .await
        .expect("upsert existing person");
    let display_name = format!("Zoom Person {suffix}");
    sqlx::query("UPDATE persons SET display_name = $1 WHERE person_id = $2")
        .bind(&display_name)
        .bind(&matched_person.person_id)
        .execute(&pool)
        .await
        .expect("seed display name");

    let event_bus = EventBus::new();
    let zoom_store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::storage::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool.clone()),
        event_bus,
    );
    let account_id = format!("zoom-participant-identity-{suffix}");
    zoom_store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Participant Identity Fixture".to_owned(),
            external_account_id: format!("zoom-participant-identity-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("fixture account");

    let participant_email = format!("zoom.person.{suffix}@example.com").to_ascii_lowercase();
    let correlation_id = format!("zoom-participant-identity-correlation-{suffix}");
    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-participant-identity-observation-{suffix}")),
            account_id: account_id.clone(),
            meeting_id: format!("participant-identity-{suffix}"),
            meeting_uuid: Some(format!("participant-identity-uuid-{suffix}")),
            topic: Some("Identity candidate meeting".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(format!("https://example.zoom.us/j/{suffix}")),
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![ZoomParticipantSnapshot {
                participant_id: Some(format!("participant-{suffix}")),
                display_name: Some(display_name.clone()),
                email: Some(participant_email.clone()),
                joined_at: None,
                left_at: None,
                metadata: json!({ "source": "zoom_participant_identity_test" }),
            }],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({}),
            causation_id: None,
            correlation_id: Some(correlation_id.clone()),
        })
        .await
        .expect("observe zoom meeting");

    let stored_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("stored zoom events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(observed.call_id))
        .expect("matched stored zoom event");

    project_zoom_participant_identity(&pool, &stored_event.event)
        .await
        .expect("zoom participant identity projection");

    let expected_candidate_id = format!(
        "identity_candidate:v1:attach_email_address:{}:{}:{}",
        matched_person.person_id,
        participant_email.len(),
        participant_email
    );
    let candidate = sqlx::query(
        r#"
        SELECT
            identity_candidate_id,
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            review_state,
            evidence_summary
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&expected_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("zoom participant identity candidate");
    assert_eq!(
        candidate
            .try_get::<String, _>("candidate_kind")
            .expect("candidate kind"),
        "attach_email_address"
    );
    assert_eq!(
        candidate
            .try_get::<String, _>("left_person_id")
            .expect("left person id"),
        matched_person.person_id
    );
    assert_eq!(
        candidate
            .try_get::<Option<String>, _>("right_person_id")
            .expect("right person id"),
        None
    );
    assert_eq!(
        candidate
            .try_get::<Option<String>, _>("email_address")
            .expect("email address"),
        Some(participant_email.clone())
    );
    assert_eq!(
        candidate
            .try_get::<String, _>("review_state")
            .expect("review state"),
        "suggested"
    );
    assert!(
        candidate
            .try_get::<String, _>("evidence_summary")
            .expect("evidence summary")
            .contains(&participant_email)
    );

    let candidate_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("person_identity.candidate.detected".to_owned()),
            limit: Some(20),
            ..Default::default()
        })
        .await
        .expect("person identity candidate events")
        .into_iter()
        .find(|event| event.event.payload["identity_candidate_id"] == json!(expected_candidate_id))
        .expect("candidate detected event");

    project_person_identity_review_event(pool.clone(), candidate_event)
        .await
        .expect("person identity review inbox projection");

    let review_item = sqlx::query(
        r#"
        SELECT review_item_id, item_kind, metadata
        FROM review_items
        WHERE item_kind = 'identity_candidate'
          AND metadata->>'identity_candidate_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&expected_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("review item");
    assert_eq!(
        review_item
            .try_get::<String, _>("item_kind")
            .expect("item kind"),
        "identity_candidate"
    );
    let metadata: serde_json::Value = review_item.try_get("metadata").expect("metadata");
    assert_eq!(metadata["candidate_kind"], json!("attach_email_address"));
    assert_eq!(metadata["left_person_id"], json!(matched_person.person_id));
    assert_eq!(metadata["email_address"], json!(participant_email));
}
```

### `backend/tests/zoom_provider_foundation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/zoom_provider_foundation.rs`
- Size bytes / Размер в байтах: `153137`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::{PgPool, Row};
use testkit::context::TestContext;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderKind,
    CommunicationProviderSecretBindingStore, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::integrations::zoom::client::{
    ZoomAccountSetupRequest, ZoomMeetingObservationRequest, ZoomStore,
};
use makosh_hub_backend::platform::calls::CallIntelligenceStore;
use makosh_hub_backend::platform::events::bus::zoom_event_types;
use makosh_hub_backend::platform::events::{EventBus, EventStore};
use makosh_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{
    EntropyEvent, HostVault, HostVaultConfig, SecretEntryContext, VaultMode,
};
use makosh_hub_backend::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;

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
        &fo
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/zoom_signal_detection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/zoom_signal_detection.rs`
- Size bytes / Размер в байтах: `10440`
- Included characters / Включено символов: `10440`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_hub_backend::domains::signal_hub::{SignalHubProfileService, SignalHubStore};
use makosh_hub_backend::integrations::zoom::client::{
    ZoomAccountSetupRequest, ZoomMeetingObservationRequest, ZoomStore,
};
use makosh_hub_backend::platform::calls::CallIntelligenceStore;
use makosh_hub_backend::platform::events::{EventBus, EventLogQuery, EventStore};
use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::zoom_signal_detection::project_zoom_signal_detection;
use serde_json::json;
use testkit::context::TestContext;

#[tokio::test]
async fn zoom_meeting_events_flow_into_signal_hub_detection_events() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore signal hub sources");

    let event_bus = EventBus::new();
    let zoom_store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::storage::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool.clone()),
        event_bus,
    );
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let account_id = format!("zoom-signal-detection-{suffix}");
    let correlation_id = format!("zoom-signal-detection-correlation-{suffix}");
    zoom_store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Signal Detection Fixture".to_owned(),
            external_account_id: format!("zoom-signal-detection-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("fixture account");

    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-signal-detection-observation-{suffix}")),
            account_id: account_id.clone(),
            meeting_id: format!("meeting-{suffix}"),
            meeting_uuid: Some(format!("meeting-uuid-{suffix}")),
            topic: Some("Signal detection meeting".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(format!("https://example.zoom.us/j/{suffix}")),
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({
                "source": "zoom_signal_detection_test",
            }),
            causation_id: None,
            correlation_id: Some(correlation_id.clone()),
        })
        .await
        .expect("observe meeting");

    let zoom_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id.clone()),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom meeting events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(observed.call_id))
        .expect("stored zoom event");

    project_zoom_signal_detection(&pool, &zoom_event.event)
        .await
        .expect("project zoom signal detection");

    let signal_store = EventStore::new(pool.clone());
    let raw_signal = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.raw.zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id.clone()),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom raw signal events")
        .into_iter()
        .next()
        .expect("raw signal");
    assert_eq!(raw_signal.event.subject["source_code"], json!("zoom"));
    assert_eq!(raw_signal.event.subject["account_id"], json!(account_id));
    assert_eq!(
        raw_signal.event.subject["entity_id"],
        json!(observed.call_id)
    );
    assert_eq!(
        raw_signal.event.subject["meeting_id"],
        json!(format!("meeting-{suffix}"))
    );
    assert_eq!(
        raw_signal.event.payload["meeting_id"],
        json!(format!("meeting-{suffix}"))
    );
    assert_eq!(
        raw_signal.event.provenance["source"],
        json!("zoom_signal_detection")
    );

    let accepted_signal = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.accepted.zoom.meeting".to_owned()),
            correlation_id: Some(correlation_id),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom accepted signal events")
        .into_iter()
        .next()
        .expect("accepted signal");
    assert_eq!(
        accepted_signal.event.subject["entity_id"],
        json!(observed.call_id)
    );
    assert_eq!(
        accepted_signal.event.provenance["signal_hub"]["decision"],
        json!("accepted")
    );
    assert_eq!(
        accepted_signal.event.provenance["raw_event_id"],
        json!(raw_signal.event.event_id)
    );
}

#[tokio::test]
async fn zoom_meeting_signal_detection_respects_testing_profile_muting() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore signal hub sources");
    SignalHubProfileService::new(
        signal_store.clone(),
        ApplicationSettingsStore::new(pool.clone()),
        EventStore::new(pool.clone()),
    )
    .apply_profile("testing")
    .await
    .expect("apply testing profile");

    let event_bus = EventBus::new();
    let zoom_store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::storage::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool.clone()),
        event_bus,
    );
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let account_id = format!("zoom-signal-muted-{suffix}");
    let correlation_id = format!("zoom-signal-muted-correlation-{suffix}");
    zoom_store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Signal Muted Fixture".to_owned(),
            external_account_id: format!("zoom-signal-muted-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("fixture account");

    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-signal-muted-observation-{suffix}")),
            account_id,
            meeting_id: format!("muted-meeting-{suffix}"),
            meeting_uuid: Some(format!("muted-meeting-uuid-{suffix}")),
            topic: Some("Muted meeting".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(format!("https://example.zoom.us/j/muted-{suffix}")),
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({}),
            causation_id: None,
            correlation_id: Some(correlation_id.clone()),
        })
        .await
        .expect("observe meeting");

    let zoom_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id.clone()),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom meeting events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(observed.call_id))
        .expect("stored zoom event");

    project_zoom_signal_detection(&pool, &zoom_event.event)
        .await
        .expect("project zoom signal detection");

    let signal_store = EventStore::new(pool);
    let muted_signal = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.muted.zoom.meeting".to_owned()),
            correlation_id: Some(correlation_id),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom muted signal events")
        .into_iter()
        .next()
        .expect("muted signal");
    assert_eq!(
        muted_signal.event.subject["entity_id"],
        json!(observed.call_id)
    );
    assert_eq!(
        muted_signal.event.provenance["signal_hub"]["decision"],
        json!("muted")
    );
    assert_eq!(
        muted_signal.event.provenance["signal_hub"]["reason"],
        json!("testing profile mutes Zoom signals")
    );

    let accepted = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.accepted.zoom.meeting".to_owned()),
            correlation_id: Some(format!("zoom-signal-muted-correlation-{suffix}")),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("accepted signal query");
    assert!(accepted.is_empty());
}
```
