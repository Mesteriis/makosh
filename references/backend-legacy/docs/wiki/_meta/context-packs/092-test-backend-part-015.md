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

- Chunk ID / ID чанка: `092-test-backend-part-015`
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

### `backend/tests/v1_communications_attachment_search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_attachment_search.rs`
- Size bytes / Размер в байтах: `8238`
- Included characters / Включено символов: `8238`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
    CommunicationStorageStore, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-attachment-search-test-token";

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

#[tokio::test]
async fn v1_attachment_search_filters_and_paginates_metadata_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-attachment-search-{suffix}");
    let first_message_id = seed_message_with_attachment(
        pool.clone(),
        SeedAttachmentMessage {
            account_id: account_id.clone(),
            provider_record_id: format!("provider-attachment-search-{suffix}-1"),
            subject: "Invoice Q1".to_owned(),
            filename: "invoice-q1.pdf".to_owned(),
            content_type: "application/pdf".to_owned(),
            hex_digit: "a".to_owned(),
            scan_status: AttachmentSafetyScanStatus::NotScanned,
        },
    )
    .await;
    let second_message_id = seed_message_with_attachment(
        pool,
        SeedAttachmentMessage {
            account_id: account_id.clone(),
            provider_record_id: format!("provider-attachment-search-{suffix}-2"),
            subject: "Invoice Q2".to_owned(),
            filename: "invoice-q2.xlsx".to_owned(),
            content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                .to_owned(),
            hex_digit: "b".to_owned(),
            scan_status: AttachmentSafetyScanStatus::Failed,
        },
    )
    .await;

    let app = router(&context.connection_string()).await;
    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/search?account_id={account_id}&q=invoice&limit=1"
        )))
        .await
        .expect("search response");
    assert_eq!(response.status(), StatusCode::OK);
    let first_page = response_json(response).await;
    assert_eq!(first_page["items"].as_array().expect("items").len(), 1);
    assert_eq!(first_page["has_more"], true);
    let next_cursor = first_page["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_owned();
    assert_eq!(first_page["items"][0]["filename"], "invoice-q2.xlsx");
    assert_eq!(first_page["items"][0]["message_id"], second_message_id);
    assert_eq!(first_page["items"][0]["message_subject"], "Invoice Q2");
    assert_eq!(first_page["items"][0]["storage_kind"], "local_fs");
    assert!(
        first_page["items"][0]["storage_path"]
            .as_str()
            .expect("storage path")
            .contains("invoice-q2")
    );

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/search?account_id={account_id}&q=invoice&limit=1&cursor={next_cursor}"
        )))
        .await
        .expect("second search response");
    assert_eq!(response.status(), StatusCode::OK);
    let second_page = response_json(response).await;
    assert_eq!(second_page["items"].as_array().expect("items").len(), 1);
    assert_eq!(second_page["has_more"], false);
    assert_eq!(second_page["next_cursor"], Value::Null);
    assert_eq!(second_page["items"][0]["filename"], "invoice-q1.pdf");
    assert_eq!(second_page["items"][0]["message_id"], first_message_id);

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/search?account_id={account_id}&content_type=pdf&scan_status=not_scanned"
        )))
        .await
        .expect("filtered search response");
    assert_eq!(response.status(), StatusCode::OK);
    let filtered = response_json(response).await;
    assert_eq!(filtered["items"].as_array().expect("items").len(), 1);
    assert_eq!(filtered["items"][0]["filename"], "invoice-q1.pdf");
}

struct SeedAttachmentMessage {
    account_id: String,
    provider_record_id: String,
    subject: String,
    filename: String,
    content_type: String,
    hex_digit: String,
    scan_status: AttachmentSafetyScanStatus,
}

async fn seed_message_with_attachment(pool: sqlx::PgPool, seed: SeedAttachmentMessage) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &seed.account_id,
            EmailProviderKind::Gmail,
            "Attachment Search Gmail",
            format!("{}@example.com", seed.account_id),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{}", seed.provider_record_id),
            &seed.account_id,
            "email_message",
            &seed.provider_record_id,
            format!("sha256:{:0>64}", seed.hex_digit),
            format!("batch-{}", seed.provider_record_id),
            json!({
                "subject": seed.subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Body for attachment search API"
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;
    let sha256 = format!("sha256:{:0>64}", seed.hex_digit);
    let blob = storage_store
        .upsert_blob(
            &NewCommunicationBlob::new(
                "local_fs",
                format!("attachments/{}/{}", seed.provider_record_id, seed.filename),
                &sha256,
                1024,
            )
            .content_type(&seed.content_type),
        )
        .await
        .expect("store blob");
    storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                format!("part-{}", seed.filename),
                &seed.content_type,
                1024,
                sha256,
            )
            .filename(&seed.filename)
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: seed.scan_status,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store attachment");
    message_id
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/v1_communications_attachment_translation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_attachment_translation.rs`
- Size bytes / Размер в байтах: `10081`
- Included characters / Включено символов: `10079`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
    CommunicationStorageStore, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "v1comms-attachment-translation-test-token";

#[tokio::test]
async fn v1_attachment_translation_uses_provided_extracted_text_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "contrato.txt");
    assert_eq!(body["original_language"], "es");
    assert_eq!(body["translated"], false);
    assert_eq!(body["target"], "en");
    assert_eq!(body["text"], Value::Null);
    assert_eq!(body["model"], Value::Null);
    assert_eq!(body["reason"], "translation runtime unavailable");
    assert_eq!(body["source"], "caller_provided_extracted_text");
}

#[tokio::test]
async fn v1_attachment_translation_emits_signal_hub_ai_events_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(context.pool(), &ollama_base_url).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["translated"], true);

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.attachment_translation.observed',
            'signal.accepted.ai.attachment_translation'
        )
          AND subject->>'attachment_id' = $1
        "#,
    )
    .bind(&seeded.attachment_id)
    .fetch_one(context.pool())
    .await
    .expect("attachment translation signal count");
    assert_eq!(signal_count, 2);
}

#[tokio::test]
async fn v1_attachment_translation_rejects_empty_source_text_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "   "
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn seed_message_with_attachment(pool: sqlx::PgPool) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-attachment-translation-{suffix}");
    let provider_record_id = format!("provider-attachment-translation-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Attachment Translation Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            &account_id,
            "email_message",
            &provider_record_id,
            format!("sha256:{:0>64}", "c"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Contrato",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Please review the attached contract."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;
    let sha256 = format!("sha256:{:0>64}", "d");
    let blob = storage_store
        .upsert_blob(
            &NewCommunicationBlob::new(
                "local_fs",
                format!("attachments/{provider_record_id}/contrato.txt"),
                &sha256,
                512,
            )
            .content_type("text/plain"),
        )
        .await
        .expect("store blob");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                "part-contrato",
                "text/plain",
                512,
                sha256,
            )
            .filename("contrato.txt")
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::NotScanned,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
        message_id,
    }
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    )
}

fn post(uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn spawn_fake_ollama() -> String {
    let app = axum::Router::new()
        .route(
            "/api/version",
            axum::routing::get(|| async { axum::Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            axum::routing::get(|| async {
                axum::Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/chat",
            axum::routing::post(|axum::Json(_body): axum::Json<Value>| async move {
                axum::Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": "Translated content from fake Ollama." },
                    "done": true,
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 16u32,
                    "eval_count": 8u32
                }))
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}

async fn configure_fake_ollama_setting(pool: &sqlx::PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama setting");
}
```

### `backend/tests/v1_communications_bilingual_reply_flow.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_bilingual_reply_flow.rs`
- Size bytes / Размер в байтах: `10274`
- Included characters / Включено символов: `10139`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "v1comms-bilingual-reply-flow-test-token";

#[tokio::test]
async fn v1_bilingual_reply_flow_returns_review_contract_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо, мы проверим контракт сегодня.",
                "tone": "business"
            }),
        ))
        .await
        .expect("bilingual reply flow response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["subject"], "Re: Contrato");
    assert_eq!(body["tone"], "business");
    assert_eq!(body["reply_language"], "ru");
    assert_eq!(body["send_ready"], false);
    assert_eq!(body["original"]["language"], "es");
    assert!(
        body["original"]["text"]
            .as_str()
            .expect("original text")
            .contains("Hola equipo")
    );
    assert_eq!(body["translation"]["target"], "ru");
    assert_eq!(body["translation"]["translated"], false);
    assert_eq!(body["translation"]["text"], Value::Null);
    assert_eq!(body["translation"]["model"], Value::Null);
    assert_eq!(
        body["translation"]["reason"],
        "translation runtime unavailable"
    );
    assert_eq!(body["reply"]["language"], "ru");
    assert_eq!(body["reply"]["tone"], "business");
    assert_eq!(
        body["reply"]["text"],
        "Спасибо, мы проверим контракт сегодня."
    );
    assert_eq!(body["back_translation"]["target"], "es");
    assert_eq!(body["back_translation"]["translated"], false);
    assert_eq!(body["back_translation"]["text"], Value::Null);
    assert_eq!(
        body["back_translation"]["reason"],
        "translation runtime unavailable"
    );
}

#[tokio::test]
async fn v1_bilingual_reply_flow_rejects_unsupported_tone_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо.",
                "tone": "casual"
            }),
        ))
        .await
        .expect("bilingual reply flow rejection response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(body["message"], "unsupported bilingual reply tone");
}

#[tokio::test]
async fn v1_bilingual_reply_flow_emits_signal_hub_ai_events_when_runtime_runs() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(context.pool(), &ollama_base_url).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо, мы проверим контракт сегодня.",
                "tone": "business"
            }),
        ))
        .await
        .expect("bilingual reply flow response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["send_ready"], true);
    assert_eq!(body["translation"]["translated"], true);
    assert_eq!(body["back_translation"]["translated"], true);

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.bilingual_reply_inbound_translation.observed',
            'signal.accepted.ai.bilingual_reply_inbound_translation',
            'signal.raw.ai.bilingual_reply_back_translation.observed',
            'signal.accepted.ai.bilingual_reply_back_translation'
        )
          AND subject->>'message_id' = $1
        "#,
    )
    .bind(&seeded.message_id)
    .fetch_one(context.pool())
    .await
    .expect("bilingual reply flow signal count");
    assert_eq!(signal_count, 4);
}

struct SeededMessage {
    message_id: String,
}

async fn seed_message(pool: sqlx::PgPool) -> SeededMessage {
    let suffix = uid();
    let account_id = format!("acct-bilingual-reply-flow-{suffix}");
    let provider_record_id = format!("provider-bilingual-reply-flow-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Bilingual Reply Flow Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            &account_id,
            "email_message",
            &provider_record_id,
            format!("sha256:{:0>64}", "e"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Contrato",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Hola equipo, gracias por enviar el contrato. Saludos."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    SeededMessage { message_id }
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    )
}

fn post(uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn spawn_fake_ollama() -> String {
    let app = axum::Router::new()
        .route(
            "/api/version",
            axum::routing::get(|| async { axum::Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            axum::routing::get(|| async {
                axum::Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/chat",
            axum::routing::post(|axum::Json(body): axum::Json<Value>| async move {
                let text = body["messages"]
                    .as_array()
                    .and_then(|messages| messages.last())
                    .and_then(|message| message["content"].as_str())
                    .unwrap_or_default();
                let content = if text.contains("Translate the following text to ru") {
                    "Спасибо, вот перевод входящего письма."
                } else {
                    "Thanks, we will review the contract today."
                };
                axum::Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": content },
                    "done": true,
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 16u32,
                    "eval_count": 8u32
                }))
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}

async fn configure_fake_ollama_setting(pool: &sqlx::PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama setting");
}
```

### `backend/tests/v1_communications_folders.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_folders.rs`
- Size bytes / Размер в байтах: `12837`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-folder-test-token";

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
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
async fn v1_custom_folders_copy_move_and_events_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-folders-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-folders-{suffix}"),
        "Folder candidate",
    )
    .await;

    let app = router(&context.connection_string()).await;
    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/folders",
            Some(json!({
                "name": "Clients",
                "description": "Client project mail",
                "account_id": account_id,
                "color": "#3b82f6",
                "sort_order": 10
            })),
        ))
        .await
        .expect("create first folder");
    assert_eq!(response.status(), StatusCode::OK);
    let first_folder = response_json(response).await;
    let first_folder_id = first_folder["folder_id"]
        .as_str()
        .expect("first folder id")
        .to_owned();
    assert!(first_folder_id.starts_with("mail_folder:"));
    assert_eq!(first_folder["name"], "Clients");
    assert_eq!(first_folder["message_count"], 0);

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/folders",
            Some(json!({ "name": "Archive copy", "account_id": account_id, "sort_order": 20 })),
        ))
        .await
        .expect("create second folder");
    assert_eq!(response.status(), StatusCode::OK);
    let second_folder = response_json(response).await;
    let second_folder_id = second_folder["folder_id"]
        .as_str()
        .expect("second folder id")
        .to_owned();
    assert_eq!(second_folder["message_count"], 0);
    let response = app
        .clone()
        .oneshot(request(
            Method::PUT,
            &format!("/api/v1/communications/folders/{first_folder_id}"),
            Some(json!({
                "name": "Clients updated",
                "color": "#2563eb"
            })),
        ))
        .await
        .expect("update first folder");
    assert_eq!(response.status(), StatusCode::OK);
    let updated_folder = response_json(response).await;
    assert_eq!(updated_folder["name"], "Clients updated");
    assert_eq!(updated_folder["color"], "#2563eb");

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            &format!("/api/v1/communications/folders/{first_folder_id}/messages/{message_id}/copy"),
            None,
        ))
        .await
        .expect("copy message");
    assert_eq!(response.status(), StatusCode::OK);
    let copied = response_json(response).await;
    assert_eq!(copied["operation"], "copy");
    assert_eq!(copied["folder_id"], first_folder_id);
    assert_eq!(copied["message_id"], message_id);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders?account_id={account_id}"),
            None,
        ))
        .await
        .expect("list folders after copy");
    assert_eq!(response.status(), StatusCode::OK);
    let folders_after_copy = response_json(response).await;
    assert_folder_count(&folders_after_copy, &first_folder_id, 1);
    assert_folder_count(&folders_after_copy, &second_folder_id, 0);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders/{first_folder_id}/messages?limit=1"),
            None,
        ))
        .await
        .expect("list first folder messages");
    assert_eq!(response.status(), StatusCode::OK);
    let first_list = response_json(response).await;
    assert_eq!(first_list["items"].as_array().expect("items").len(), 1);
    assert_eq!(first_list["items"][0]["message_id"], message_id);
    assert_eq!(first_list["items"][0]["subject"], "Folder candidate");
    assert_eq!(first_list["has_more"], false);

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            &format!(
                "/api/v1/communications/folders/{second_folder_id}/messages/{message_id}/move"
            ),
            None,
        ))
        .await
        .expect("move message");
    assert_eq!(response.status(), StatusCode::OK);
    let moved = response_json(response).await;
    assert_eq!(moved["operation"], "move");
    assert_eq!(moved["folder_id"], second_folder_id);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders?account_id={account_id}"),
            None,
        ))
        .await
        .expect("list folders after move");
    assert_eq!(response.status(), StatusCode::OK);
    let folders_after_move = response_json(response).await;
    assert_folder_count(&folders_after_move, &first_folder_id, 0);
    assert_folder_count(&folders_after_move, &second_folder_id, 1);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders/{first_folder_id}/messages"),
            None,
        ))
        .await
        .expect("first folder after move");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await["items"]
            .as_array()
            .expect("items")
            .len(),
        0
    );

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders/{second_folder_id}/messages"),
            None,
        ))
        .await
        .expect("second folder after move");
    assert_eq!(response.status(), StatusCode::OK);
    let second_list = response_json(response).await;
    assert_eq!(second_list["items"].as_array().expect("items").len(), 1);
    assert_eq!(second_list["items"][0]["message_id"], message_id);
    let response = app
        .clone()
        .oneshot(request(
            Method::DELETE,
            &format!("/api/v1/communications/folders/{second_folder_id}"),
            None,
        ))
        .await
        .expect("delete second folder");
    assert_eq!(response.status(), StatusCode::OK);
    let deleted = response_json(response).await;
    assert_eq!(deleted["deleted"], true);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE subject->>'kind' IN ('mail_folder', 'mail_folder_message')",
    )
    .fetch_one(&pool)
    .await
    .expect("event count");
    assert_eq!(event_count, 6);
    let folder_links = sqlx::query(
        "SELECT observation_id, entity_id, relationship_kind, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'mail_folder'
         ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("folder observation links");
    assert_eq!(folder_links.len(), 4);
    let folder_operations: Vec<String> = folder_links
        .iter()
        .map(|row| {
            row.try_get::<Value, _>("metadata")
                .expect("folder metadata")["operation"]
                .as_str()
                .expect("folder operation")
                .to_owned()
        })
        .collect();
    assert_eq!(
        folder_operations,
        vec![
            "folder_create".to_owned(),
            "folder_create".to_owned(),
            "folder_update".to_owned(),
            "folder_delete".to_owned()
        ]
    );
    let folder_observation_id: String = folder_links[0]
        .try_get("observation_id")
        .expect("folder observation id");
    let folder_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&folder_observation_id)
    .fetch_one(&pool)
    .await
    .expect("folder observation");
    let folder_origin_kind: String = folder_observation
        .try_get("origin_kind")
        .expect("folder origin kind");
    let folder_payload: Value = folder_observation
        .try_get("payload")
        .expect("folder payload");
    assert_eq!(folder_origin_kind, "manual");
    assert_eq!(folder_payload["operation"], "folder_create");

    let message_links = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'folder_message_transition'
         ORDER BY created_at ASC",
    )
    .bind(&message_id)
    .fetch_all(&pool)
    .await
    .expect("folder message observation links");
    assert_eq!(message_links.len(), 2);
    let message_operations: Vec<String> = message_links
        .iter()
        .map(|row| {
            row.try_get::<Value, _>("metadata")
                .expect("message metadata")["operation"]
                .as_str()
                .expect("message operation")
                .to_owned()
        })
        .collect();
    assert_eq!(
        message_operations,
        vec!["copy".to_owned(), "move".to_owned()]
    );
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Folder Gmail",
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
                "body_text": "Body for folder API"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_message_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_message_actions.rs`
- Size bytes / Размер в байтах: `27528`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-action-test-token";

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
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

macro_rules! v1_msg_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let r = router(&db).await;
            let response = r
                .oneshot(post(
                    &format!("/api/v1/communications/messages/msg:fake/{}", $path_suffix),
                    $body,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

v1_msg_post_test!(
    v1_send,
    "send",
    json!({"to": "test@example.com", "subject": "Test", "body": "Hello"})
);
v1_msg_post_test!(v1_reply, "reply", json!({"body": "Reply text"}));
v1_msg_post_test!(v1_reply_all, "reply-all", json!({"body": "Reply all text"}));
v1_msg_post_test!(v1_forward, "forward", json!({"to": "fwd@example.com"}));
v1_msg_post_test!(
    v1_forward_eml,
    "forward-eml",
    json!({"to": "fwd@example.com"})
);
v1_msg_post_test!(
    v1_redirect_missing_message,
    "redirect",
    json!({"to": ["redirect@example.com"], "confirmed_provider_write": true})
);
v1_msg_post_test!(v1_imap_mark_read, "imap-mark-read", json!({}));
v1_msg_post_test!(v1_imap_delete, "imap-delete", json!({}));
v1_msg_post_test!(v1_translate, "translate", json!({"target_language": "es"}));
v1_msg_post_test!(v1_ai_reply, "ai-reply", json!({"prompt": "Reply to this"}));
v1_msg_post_test!(
    v1_ai_reply_variants,
    "ai-reply-variants",
    json!({"prompt": "Reply variants"})
);
v1_msg_post_test!(v1_extract_tasks, "extract-tasks", json!({}));
v1_msg_post_test!(v1_extract_notes, "extract-notes", json!({}));
v1_msg_post_test!(
    v1_message_analyze,
    "analyze",
    json!({"analysis_type": "sentiment"})
);

#[tokio::test]
async fn v1_message_analyze_returns_structured_ai_summary_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-analyze-summary-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("provider-analyze-summary-{suffix}"),
        "Action Required: Contract review deadline",
        "From: Ada Lovelace <ada@acme.example>\nPlease review the attached MSA and NDA by Friday. The payment risk remains open. Meeting on Monday at 10:00 with Acme Corp. Confirm approval before EOD.",
    )
    .await;
    let r = router(&context.connection_string()).await;

    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/analyze"),
            json!({}),
        ))
        .await
        .expect("analyze response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert!(
        body["summary_contract"]["key_points"]
            .as_array()
            .expect("key points")
            .iter()
            .any(|item| item.as_str() == Some("Action Required: Contract review deadline"))
    );
    assert!(
        body["summary_contract"]["action_items"]
            .as_array()
            .expect("action items")
            .iter()
            .any(|item| {
                let item = item.as_str().unwrap_or("");
                item.contains("Please review") && item.contains("NDA")
            })
    );
    assert!(
        body["summary_contract"]["risks"]
            .as_array()
            .expect("risks")
            .iter()
            .any(|item| item.as_str().unwrap_or("").contains("payment risk"))
    );
    assert!(
        body["summary_contract"]["deadlines"]
            .as_array()
            .expect("deadlines")
            .iter()
            .any(|item| item.as_str().unwrap_or("").contains("Friday"))
    );
    assert!(
        body["summary_contract"]["event_candidates"]
            .as_array()
            .expect("event candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("Meeting on Monday"))
    );
    assert!(
        body["summary_contract"]["persona_candidates"]
            .as_array()
            .expect("persona candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("Ada Lovelace"))
    );
    assert!(
        body["summary_contract"]["organization_candidates"]
            .as_array()
            .expect("organization candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("acme.example"))
    );
    assert!(
        body["summary_contract"]["document_candidates"]
            .as_array()
            .expect("document candidates")
            .iter()
            .any(|item| item["title"].as_str().unwrap_or("").contains("MSA"))
    );
    assert!(
        body["summary_contract"]["agreement_candidates"]
            .as_array()
            .expect("agreement candidates")
            .iter()
            .any(|item| item["title"].as_str().unwrap_or("").contains("NDA"))
    );

    let metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message metadata");
    assert_eq!(
        metadata["ai_summary_contract"], body["summary_contract"],
        "analyze response must persist the structured summary contract"
    );

    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let knowledge_review_items = sqlx::query(
        r#"
        SELECT metadata
        FROM review_items
        WHERE item_kind = 'knowledge_candidate'
          AND review_item_id IN (
              SELECT review_item_id
              FROM review_item_evidence
              WHERE observation_id = $1
          )
        ORDER BY metadata->>'candidate_group', title
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&pool)
    .await
    .expect("knowledge review items");
    let mut candidate_groups = knowledge_review_items
        .into_iter()
        .map(|row| {
            let metadata: Value = row.try_get("metadata").expect("metadata");
            metadata["candidate_group"]
                .as_str()
                .expect("candidate group")
                .to_owned()
        })
        .collect::<Vec<_>>();
    candidate_groups.sort();
    candidate_groups.dedup();
    assert_eq!(
        candidate_groups,
        vec!["agreement".to_owned(), "document".to_owned()]
    );
}

#[tokio::test]
async fn v1_bulk_actions_mark_read_and_trash_messages_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-bulk-actions-{suffix}");
    let first_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-actions-{suffix}-1"),
        "Bulk actions first",
    )
    .await;
    let second_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-actions-{suffix}-2"),
        "Bulk actions second",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/messages/bulk-actions",
            json!({
                "action": "mark_read",
                "message_ids": [first_id, second_id]
            }),
        ))
        .await
        .expect("bulk mark-read response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["action"], "mark_read");
    assert_eq!(body["matched_count"], 2);
    assert_eq!(body["updated_count"], 2);
    assert_eq!(body["not_found"].as_array().expect("not_found").len(), 0);

    let first_workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&first_id)
    .fetch_one(&pool)
    .await
    .expect("first workflow state");
    let second_workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&second_id)
    .fetch_one(&pool)
    .await
    .expect("second workflow state");
    assert_eq!(first_workflow_state, "reviewed");
    assert_eq!(second_workflow_state, "reviewed");
    let read_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.message.read'
          AND payload->>'action' = 'mark_read'
          AND payload->>'updated_count' = '2'
          AND payload->'message_ids' ? $1
          AND payload->'message_ids' ? $2
        "#,
    )
    .bind(&first_id)
    .bind(&second_id)
    .fetch_one(&pool)
    .await
    .expect("read event count");
    assert_eq!(read_event_count, 1);
    let workflow_links = sqlx::query(
        "SELECT observation_id, entity_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND relationship_kind = 'workflow_state_transition'
           AND entity_id = ANY($1)
         ORDER BY entity_id ASC, created_at ASC",
    )
    .bind(vec![first_id.clone(), second_id.clone()])
    .fetch_all(&pool)
    .await
    .expect("workflow links");
    assert_eq!(workflow_links.len(), 2);
    for row in &workflow_links {
        let metadata: Value = row.try_get("metadata").expect("workflow metadata");
        assert_eq!(metadata["workflow_state"], "reviewed")
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_read_receipts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_read_receipts.rs`
- Size bytes / Размер в байтах: `25036`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
    OutboxSendReceipt,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-read-receipt-test-token";

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
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

#[tokio::test]
async fn v1_read_receipt_records_correlation_and_realtime_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-read-receipt-{suffix}");
    let provider_message_id = format!("provider-message-read-receipt-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let read_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/communications/read-receipts",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "recipient": "reader@example.com",
                "read_at": read_at,
                "source_kind": "mdn",
                "provider_record_id": format!("mdn-{suffix}"),
                "metadata": {
                    "user_agent": "fixture-mdn"
                }
            }),
        ))
        .await
        .expect("read receipt response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["outbox_id"], outbox_id);
    assert_eq!(body["provider_message_id"], provider_message_id);
    assert_eq!(body["recipient"], "reader@example.com");
    assert_eq!(body["receipt_kind"], "read");
    assert_eq!(body["source_kind"], "mdn");

    let persisted = sqlx::query(
        r#"
        SELECT account_id, outbox_id, provider_message_id, recipient, receipt_kind, metadata
        FROM communication_read_receipts
        WHERE provider_record_id = $1
        "#,
    )
    .bind(format!("mdn-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("persisted read receipt");
    assert_eq!(
        persisted.try_get::<String, _>("account_id").unwrap(),
        account_id
    );
    assert_eq!(
        persisted.try_get::<Option<String>, _>("outbox_id").unwrap(),
        Some(outbox_id.clone())
    );
    assert_eq!(
        persisted
            .try_get::<String, _>("provider_message_id")
            .unwrap(),
        provider_message_id
    );
    assert_eq!(
        persisted.try_get::<String, _>("recipient").unwrap(),
        "reader@example.com"
    );
    assert_eq!(
        persisted.try_get::<String, _>("receipt_kind").unwrap(),
        "read"
    );
    assert_eq!(
        persisted.try_get::<Value, _>("metadata").unwrap(),
        json!({"user_agent": "fixture-mdn"})
    );

    let event = sqlx::query(
        r#"
        SELECT subject, payload
        FROM event_log
        WHERE event_type = 'mail.read_receipt.recorded'
          AND subject->>'kind' = 'mail_read_receipt'
          AND subject->>'outbox_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("read receipt event");
    let subject = event.try_get::<Value, _>("subject").unwrap();
    let payload = event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(subject["outbox_id"], outbox_id);
    assert_eq!(payload["account_id"], account_id);
    assert_eq!(payload["provider_message_id"], provider_message_id);
    assert_eq!(payload["receipt_kind"], "read");
    assert!(payload.get("recipient").is_none());
    assert!(payload.get("body_text").is_none());
    let receipt_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'read_receipt'
           AND relationship_kind = 'read_receipt_recorded'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("read receipt observation link");
    let receipt_observation_id: String = receipt_link
        .try_get("observation_id")
        .expect("read receipt observation id");
    let receipt_metadata: Value = receipt_link.try_get("metadata").expect("receipt metadata");
    assert_eq!(receipt_metadata["receipt_kind"], "read");
    let receipt_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&receipt_observation_id)
    .fetch_one(&pool)
    .await
    .expect("read receipt observation");
    let receipt_origin_kind: String = receipt_observation
        .try_get("origin_kind")
        .expect("receipt origin kind");
    let receipt_payload: Value = receipt_observation
        .try_get("payload")
        .expect("receipt payload");
    assert_eq!(receipt_origin_kind, "local_runtime");
    assert_eq!(receipt_payload["operation"], "read_receipt_recorded");
}

#[tokio::test]
async fn v1_outbox_list_includes_latest_read_receipt_summary_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-outbox-read-summary-{suffix}");
    let provider_message_id = format!("provider-message-outbox-read-summary-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let read_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let receipt_response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/read-receipts",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "recipient": "reader@example.com",
                "read_at": read_at,
                "source_kind": "mdn",
                "provider_record_id": format!("mdn-outbox-summary-{suffix}")
            }),
        ))
        .await
        .expect("read receipt response");
    assert_eq!(receipt_response.status(), StatusCode::OK);

    let list_response = r
        .oneshot(get(&format!(
            "/api/v1/communications/outbox?account_id={account_id}&status=sent"
        )))
        .await
        .expect("outbox list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let body = response_json(list_response).await;
    let items = body["items"].as_array().expect("outbox items");
    let item = items
        .iter()
        .find(|candidate| candidate["outbox_id"] == outbox_id)
        .expect("sent outbox item in response");

    let latest_read_receipt = &item["metadata"]["latest_read_receipt"];
    assert_eq!(latest_read_receipt["receipt_kind"], "read");
    assert_eq!(latest_read_receipt["source_kind"], "mdn");
    let listed_read_at = latest_read_receipt["read_at"]
        .as_str()
        .expect("latest read receipt read_at");
    assert_eq!(
        DateTime::parse_from_rfc3339(listed_read_at)
            .expect("parse listed read_at")
            .with_timezone(&Utc),
        read_at
    );
    assert!(latest_read_receipt.get("recipient").is_none());
    assert!(latest_read_receipt.get("provider_record_id").is_none());
}

#[tokio::test]
async fn v1_provider_delivery_event_records_delivery_status_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-provider-delivery-{suffix}");
    let provider_message_id = format!("provider-message-provider-delivery-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let occurred_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/integrations/mail/provider-delivery-events",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "event_kind": "delivered",
                "occurred_at": occurred_at,
                "source_kind": "gmail_history",
                "provider_record_id": format!("gmail-history-delivered-{suffix}")
            }),
        ))
        .await
        .expect("provider delivery event response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["notification_kind"], "delivery_status");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["outbox_id"], outbox_id);
    assert_eq!(body["provider_message_id"], provider_message_id);
    assert_eq!(body["delivery_status"], "delivered");
    assert_eq!(body["source_kind"], "gmail_history");

    let metadata: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_outbox WHERE outbox_id = $1")
            .bind(&outbox_id)
            .fetch_one(&pool)
            .await
            .expect("outbox metadata");
    assert_eq!(metadata["delivery_status"]["delivery_status"], "delivered");
    assert_eq!(metadata["delivery_status"]["source_kind"], "gmail_history");
    let delivery_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'delivery_status_observed'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("delivery status observation link");
    let delivery_observation_id: String = delivery_link
        .try_get("observation_id")
        .expect("delivery observation id");
    let delivery_metadata: Value = delivery_link
        .try_get("metadata")
        .expect("delivery metadata");
    assert_eq!(delivery_metadata["delivery_status"], "delivered");
    let delivery_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&delivery_observation_id)
    .fetch_one(&pool)
    .await
    .expect("delivery observation");
    let delivery_origin_kind: String = delivery_observation
        .try_get("origin_kind")
        .expect("delivery origin kind");
    let delivery_payload: Value = delivery_observation
        .try_get("payload")
        .expect("delivery payload");
    assert_eq!(delivery_origin_kind, "local_runtime");
    assert_eq!(delivery_payload["operation"], "delivery_status_recorded");

    l
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_regressions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_regressions.rs`
- Size bytes / Размер в байтах: `298`
- Included characters / Включено символов: `298`
- Truncated / Обрезано: `no`

```rust
#[path = "v1_communications_regressions/analytics.rs"]
mod analytics;
#[path = "v1_communications_regressions/drafts_outbox.rs"]
mod drafts_outbox;
#[path = "v1_communications_regressions/messages_threads.rs"]
mod messages_threads;
#[path = "v1_communications_regressions/support.rs"]
mod support;
```

### `backend/tests/v1_communications_regressions/analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_regressions/analytics.rs`
- Size bytes / Размер в байтах: `4231`
- Included characters / Включено символов: `4231`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use testkit::context::TestContext;
use tower::ServiceExt;

use super::support::{get, response_json, router, seed_projected_message_from_sender, uid};

#[tokio::test]
async fn v1_subscriptions_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-sub-page-{suffix}");
    for index in 0..3 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sub-a-{suffix}-{index}"),
            "Weekly digest",
            "newsletter-a@example.com",
            "Newsletter body with unsubscribe link",
        )
        .await;
    }
    for index in 0..2 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sub-b-{suffix}-{index}"),
            "Product newsletter",
            "newsletter-b@example.com",
            "Newsletter body with manage preferences link",
        )
        .await;
    }
    let router = router(&context.connection_string()).await;
    let response = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/subscriptions?account_id={account_id}&limit=1"
        )))
        .await
        .expect("subscriptions first page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = router
        .oneshot(get(&format!(
            "/api/v1/communications/subscriptions?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("subscriptions second page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_top_senders_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-senders-page-{suffix}");
    for index in 0..3 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sender-a-{suffix}-{index}"),
            "Message from A",
            "sender-a@example.com",
            "Regular mail body",
        )
        .await;
    }
    for index in 0..2 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sender-b-{suffix}-{index}"),
            "Message from B",
            "sender-b@example.com",
            "Regular mail body",
        )
        .await;
    }

    let router = router(&context.connection_string()).await;
    let response = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/analytics/senders?account_id={account_id}&limit=1"
        )))
        .await
        .expect("top senders first page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = router
        .oneshot(get(&format!(
            "/api/v1/communications/analytics/senders?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("top senders second page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}
```

### `backend/tests/v1_communications_regressions/drafts_outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_regressions/drafts_outbox.rs`
- Size bytes / Размер в байтах: `16547`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use serde_json::json;
use sqlx::Row;
use testkit::context::TestContext;
use tower::ServiceExt;

use super::support::{delete, get, post, post_with_actor, response_json, router, uid};

#[tokio::test]
async fn v1_post_draft_allows_empty_subject_for_autosave_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-draft-autosave-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Draft Autosave IMAP",
            format!("draft-autosave-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let router = router(&context.connection_string()).await;
    let draft_id = format!("draft-autosave-{suffix}");
    let response = router
        .clone()
        .oneshot(post(
            "/api/v1/communications/drafts",
            json!({
                "draft_id": draft_id,
                "account_id": account_id,
                "to_recipients": [],
                "cc_recipients": [],
                "bcc_recipients": [],
                "subject": "",
                "body_text": "Body typed before subject",
                "body_html": null,
                "metadata": {"compose_mode": "compose"}
            }),
        ))
        .await
        .expect("draft autosave response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["draft_id"], draft_id);
    assert_eq!(body["subject"], "");
    assert_eq!(body["body_text"], "Body typed before subject");

    let created_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.created'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft created event count");
    assert_eq!(created_event_count, 1);
    let created_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'draft'
           AND entity_id = $1
           AND relationship_kind = 'draft_upsert'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&draft_id)
    .fetch_one(&pool)
    .await
    .expect("draft create observation link");
    let created_observation_id: String = created_link
        .try_get("observation_id")
        .expect("draft create observation id");
    let created_metadata: serde_json::Value = created_link
        .try_get("metadata")
        .expect("draft create metadata");
    assert_eq!(created_metadata["operation"], "draft_create");
    let created_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&created_observation_id)
    .fetch_one(&pool)
    .await
    .expect("draft create observation");
    let created_origin_kind: String = created_observation
        .try_get("origin_kind")
        .expect("draft create origin kind");
    let created_payload: serde_json::Value = created_observation
        .try_get("payload")
        .expect("draft create payload");
    assert_eq!(created_origin_kind, "manual");
    assert_eq!(created_payload["operation"], "draft_create");

    let response = router
        .clone()
        .oneshot(post(
            "/api/v1/communications/drafts",
            json!({
                "draft_id": draft_id,
                "account_id": account_id,
                "to_recipients": ["recipient@example.com"],
                "subject": "",
                "body_text": "Updated autosave body",
                "body_html": "<p>Updated autosave body</p>",
                "metadata": {"compose_mode": "compose"}
            }),
        ))
        .await
        .expect("draft autosave update response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["draft_id"], draft_id);
    assert_eq!(body["body_text"], "Updated autosave body");

    let updated_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.updated'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND payload->>'has_body_html' = 'true'
          AND payload->>'to_recipient_count' = '1'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft updated event count");
    assert_eq!(updated_event_count, 1);
    let upsert_links_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'draft'
           AND entity_id = $1
           AND relationship_kind = 'draft_upsert'",
    )
    .bind(&draft_id)
    .fetch_one(&pool)
    .await
    .expect("draft upsert observation count");
    assert_eq!(upsert_links_count, 2);

    let response = router
        .oneshot(delete(&format!("/api/v1/communications/drafts/{draft_id}")))
        .await
        .expect("draft delete response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["deleted"], true);

    let deleted_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.deleted'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft deleted event count");
    assert_eq!(deleted_event_count, 1);
    let deleted_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'draft'
           AND entity_id = $1
           AND relationship_kind = 'draft_delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&draft_id)
    .fetch_one(&pool)
    .await
    .expect("draft delete observation link");
    let deleted_observation_id: String = deleted_link
        .try_get("observation_id")
        .expect("draft delete observation id");
    let deleted_metadata: serde_json::Value = deleted_link
        .try_get("metadata")
        .expect("draft delete metadata");
    assert_eq!(deleted_metadata["operation"], "draft_delete");
    let deleted_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("draft delete observation");
    let deleted_origin_kind: String = deleted_observation
        .try_get("origin_kind")
        .expect("draft delete origin kind");
    let deleted_payload: serde_json::Value = deleted_observation
        .try_get("payload")
        .expect("draft delete payload");
    assert_eq!(deleted_origin_kind, "manual");
    assert_eq!(deleted_payload["operation"], "draft_delete");
}

#[tokio::test]
async fn v1_drafts_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-draft-page-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Draft Pagination IMAP",
            format!("draft-page-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let router = router(&context.connection_string()).await;
    for index in 0..2 {
        let response = router
            .clone()
            .oneshot(post(
                "/api/v1/communications/drafts",
                json!({
                    "draft_id": format!("draft-page-{suffix}-{index}"),
                    "account_id": account_id,
                    "to_recipients": ["recipient@example.com"],
                    "subject": format!("Paged draft {index}"),
                    "body_text": format!("Draft body {index}"),
                    "metadata": {"compose_mode": "compose"}
                }),
            ))
            .await
            .expect("draft create response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    let response = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/drafts?account_id={account_id}&limit=1"
        )))
        .await
        .expect("draft list first page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = router
        .oneshot(get(&format!(
            "/api/v1/communications/drafts?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("draft list second page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_send_schedules_outbox_message_and_allows_undo_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-outbox-api-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Outbox API IMAP",
            format!("outbox-api-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let router = router(&context.connection_string()).await;
    let scheduled_send_at = Utc::now() + Duration::hours(1);
    let send = router
        .clone()
        .oneshot(post_with_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Scheduled outbox",
                "body_text": "This should be queued, not sent immediately.",
                "scheduled_send_at": scheduled_send_at,
                "undo_send_seconds": 30,
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("scheduled send response");

    assert_eq!(send.status(), StatusCode::OK);
    let send_body = response_json(send).await;
    assert_eq!(send_body["transport"], "outbox");
    assert_eq!(send_body["status"], "scheduled");
    let outbox_id = send_body["outbox_id"].as_str().expect("outbox id");
    assert!(!outbox_id.trim().is_empty());

    let list = router
        .c
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_regressions/messages_threads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_regressions/messages_threads.rs`
- Size bytes / Размер в байтах: `23362`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::{get as axum_get, post as axum_post};
use axum::{Json, Router};
use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::signal_hub::{
    SignalHubStore, SignalPolicy, SignalPolicyMode, SignalPolicyScope,
};
use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use testkit::context::TestContext;
use tokio::net::TcpListener;
use tower::ServiceExt;

use super::support::{
    T, get, post, response_json, router, seed_projected_message, seed_projected_message_with_body,
    uid,
};

#[tokio::test]
async fn v1_messages_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-cursor-api-{suffix}");
    let mut seeded_message_ids = Vec::new();

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-cursor-api-{suffix}-{index}"),
            &format!("Cursor page subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic message ordering");
        seeded_message_ids.push(message_id);
    }

    let router = router(&context.connection_string()).await;
    let first = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = router
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"].as_array().expect("second items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
    for message_id in seeded_message_ids {
        assert!(returned_ids.contains(&message_id), "missing {message_id}");
    }
}

#[tokio::test]
async fn v1_threads_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-cursor-api-{suffix}");

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-thread-cursor-api-{suffix}-{index}"),
            &format!("Thread Cursor Subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic thread ordering");
    }

    let router = router(&context.connection_string()).await;
    let first = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first thread cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first thread items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next thread cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = router
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second thread cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"]
        .as_array()
        .expect("second thread items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["thread_id"].as_str().expect("thread id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
}

#[tokio::test]
async fn v1_translate_thread_returns_per_message_fallbacks_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-translate-{suffix}");
    let subject = "Thread Translation";
    let first_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-1-{suffix}"),
        subject,
        "Привет, нужна проверка договора.",
    )
    .await;
    let second_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-2-{suffix}"),
        &format!("Re: {subject}"),
        "Hello, the agreement is attached.",
    )
    .await;
    let router = router(&context.connection_string()).await;
    let response = router
        .oneshot(post(
            &format!(
                "/api/v1/communications/threads/translate?account_id={account_id}&subject=Thread%20Translation"
            ),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("thread translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["subject"], subject);
    assert_eq!(body["target_language"], "en");
    let items = body["items"].as_array().expect("translation items");
    assert_eq!(items.len(), 2);
    let returned_ids = items
        .iter()
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert!(returned_ids.contains(&first_id));
    assert!(returned_ids.contains(&second_id));
    assert!(
        items
            .iter()
            .any(|item| item["original_language"] == "ru" && item["translated"] == false)
    );
    assert!(items.iter().all(|item| {
        item["reason"]
            .as_str()
            .map(|reason| !reason.trim().is_empty())
            .unwrap_or(false)
    }));
}

#[tokio::test]
async fn v1_translate_thread_emits_signal_hub_ai_events_per_message() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-translate-signals-{suffix}");
    let subject = "Thread Translation";
    let first_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-signal-1-{suffix}"),
        subject,
        "Привет, нужна проверка договора.",
    )
    .await;
    let second_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-signal-2-{suffix}"),
        &format!("Re: {subject}"),
        "Hola equipo, revisemos el acuerdo.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!(
                "/api/v1/communications/threads/translate?account_id={account_id}&subject=Thread%20Translation"
            ),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("thread translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    let items = body["items"].as_array().expect("translation items");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| item["translated"] == true));

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.thread_message_translation.observed',
            'signal.accepted.ai.thread_message_translation'
        )
          AND subject->>'message_id' = ANY($1)
        "#,
    )
    .bind(vec![first_id.clone(), second_id.clone()])
    .fetch_one(&pool)
    .await
    .expect("thread translation signal count");
    assert_eq!(signal_count, 4);
}

#[tokio::test]
async fn v1_message_translate_returns_fallback_when_ai_source_is_muted() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-message-translate-muted-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("message-translate-muted-{suffix}"),
        "Translate me",
        "Hola equipo, necesitamos revisar el contrato hoy.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    SignalHubStore::new(pool)
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("ai".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "mute ai message translation".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create ai mute policy");

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/translate"),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["translated"], false);
    assert_eq!(body["reason"], "no LLM configured");
}

#[tokio::test]
async fn v1_message_translate_emits_signal
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_regressions/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_regressions/support.rs`
- Size bytes / Размер в байтах: `6795`
- Included characters / Включено символов: `6795`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use chrono::Utc;
use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};

pub(crate) const T: &str = "v1comms-regression-test-token";

pub(crate) fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn post_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .header("x-makosh-actor-id", "makosh-frontend")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub(crate) async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

pub(crate) async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

pub(crate) async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
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
                    "body_text": "Body for cursor pagination API"
                }),
            )
            .occurred_at(Utc::now()),
        )
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub(crate) async fn seed_projected_message_with_body(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Thread Translate Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
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
            )
            .occurred_at(Utc::now()),
        )
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub(crate) async fn seed_projected_message_from_sender(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    sender: &str,
    body_text: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Paged Analytics Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{provider_record_id}"),
                format!("batch-{provider_record_id}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": ["recipient@example.com"],
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now()),
        )
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
```

### `backend/tests/v1_communications_regressions_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_regressions_architecture.rs`
- Size bytes / Размер в байтах: `2147`
- Included characters / Включено символов: `2147`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn v1_communications_regression_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_v1_communications_regression_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "v1 communications regression test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_v1_communications_regression_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_v1_communications_regression_test_violations(&path, violations);
            continue;
        }
        if !is_v1_communications_regression_test_file(&path) {
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

fn is_v1_communications_regression_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "v1_communications_regressions.rs"
        || file_name == "v1_communications_regressions_architecture.rs"
    {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "v1_communications_regressions")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/v1_communications_saved_searches.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_saved_searches.rs`
- Size bytes / Размер в байтах: `18240`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, WorkflowState, project_raw_email_message,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-saved-search-test-token";

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
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
    l
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
