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

- Chunk ID / ID чанка: `105-source-crates`
- Group / Группа: `crates`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/crates.md`

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

### `crates/testkit/src/app.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/app.rs`
- Size bytes / Размер в байтах: `3037`
- Included characters / Включено символов: `3037`
- Truncated / Обрезано: `no`

```rust
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, header};
use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use serde_json::Value;

use crate::context::TestContext;
use crate::vault;

pub const TEST_API_SECRET: &str = "makosh-test-api-secret";

pub struct TestApp {
    router: Router,
    context: TestContext,
}

impl TestApp {
    pub async fn new() -> Self {
        let context = TestContext::new().await;
        let router = router_for_context(&context);
        Self { context, router }
    }

    pub fn context(&self) -> &TestContext {
        &self.context
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn into_router(self) -> Router {
        self.router
    }

    pub fn clone_router(&self) -> Router {
        self.router.clone()
    }
}

pub fn config() -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret(TEST_API_SECRET))
}

pub fn config_with_database_url(database_url: impl Into<String>) -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret_and_database_url(
        TEST_API_SECRET,
        database_url,
    ))
}

pub fn config_with_secret(api_secret: impl Into<String>) -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret(api_secret))
}

pub fn config_with_secret_and_database_url(
    api_secret: impl Into<String>,
    database_url: impl Into<String>,
) -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret_and_database_url(
        api_secret,
        database_url,
    ))
}

pub fn database_for_context(context: &TestContext) -> Database {
    context.database()
}

pub fn router_for_context(context: &TestContext) -> Router {
    build_router_with_database(
        context.app_config(TEST_API_SECRET),
        database_for_context(context),
    )
}

pub fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TEST_API_SECRET)
        .body(Body::empty())
        .expect("request")
}

pub fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TEST_API_SECRET)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn get(uri: &str) -> Request<Body> {
    empty_request(Method::GET, uri)
}

pub fn post_json(uri: &str, body: Value) -> Request<Body> {
    json_request(Method::POST, uri, body)
}

pub fn put_json(uri: &str, body: Value) -> Request<Body> {
    json_request(Method::PUT, uri, body)
}

pub fn patch_json(uri: &str, body: Value) -> Request<Body> {
    json_request(Method::PATCH, uri, body)
}

pub fn delete(uri: &str) -> Request<Body> {
    empty_request(Method::DELETE, uri)
}
```

### `crates/testkit/src/bin/makosh_test_session.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/bin/makosh_test_session.rs`
- Size bytes / Размер в байтах: `1460`
- Included characters / Включено символов: `1460`
- Truncated / Обрезано: `no`

```rust
use std::env;
use std::process::{Command, Stdio};

use testkit::containers::nats::{NatsContainer, SESSION_NATS_HOST_PORT_ENV};
use testkit::containers::postgres::{
    PostgresContainer, SESSION_ID_ENV, SESSION_POSTGRES_HOST_PORT_ENV,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let command_args = env::args().skip(1).collect::<Vec<_>>();
    if command_args.is_empty() {
        eprintln!("usage: makosh-test-session <command> [args...]");
        std::process::exit(2);
    }

    let session_id = format!("makosh-test-{}", Uuid::new_v4());
    let postgres_container = PostgresContainer::start_owned().await;
    let nats_container = NatsContainer::start_owned().await;
    let status = Command::new(&command_args[0])
        .args(&command_args[1..])
        .env(SESSION_ID_ENV, &session_id)
        .env(
            SESSION_POSTGRES_HOST_PORT_ENV,
            postgres_container.host_port().to_string(),
        )
        .env(
            SESSION_NATS_HOST_PORT_ENV,
            nats_container.host_port().to_string(),
        )
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run test session command '{}': {error}",
                command_args[0]
            )
        });

    drop(nats_container);
    drop(postgres_container);
    std::process::exit(status.code().unwrap_or(1));
}
```

### `crates/testkit/src/containers/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/containers/mod.rs`
- Size bytes / Размер в байтах: `32`
- Included characters / Включено символов: `32`
- Truncated / Обрезано: `no`

```rust
pub mod nats;
pub mod postgres;
```

### `crates/testkit/src/containers/nats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/containers/nats.rs`
- Size bytes / Размер в байтах: `2528`
- Included characters / Включено символов: `2528`
- Truncated / Обрезано: `no`

```rust
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::time::{Duration, Instant, sleep};

const NATS_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const NATS_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);
pub const SESSION_NATS_HOST_PORT_ENV: &str = "MAKOSH_TEST_NATS_HOST_PORT";

pub struct NatsContainer {
    _container: Option<ContainerAsync<GenericImage>>,
    host_port: u16,
}

impl NatsContainer {
    pub async fn start() -> Self {
        if let Some(host_port) = session_host_port() {
            wait_for_nats_port(host_port).await;
            return Self {
                _container: None,
                host_port,
            };
        }

        Self::start_owned().await
    }

    pub async fn start_owned() -> Self {
        let container = GenericImage::new("nats", "2.11-alpine")
            .with_exposed_port(4222.tcp())
            .with_cmd(vec!["-js", "-sd", "/data"])
            .start()
            .await
            .expect("failed to start NATS container");

        let host_port = container
            .get_host_port_ipv4(4222)
            .await
            .expect("failed to resolve NATS container port");

        wait_for_nats_port(host_port).await;

        Self {
            _container: Some(container),
            host_port,
        }
    }

    pub fn server_url(&self) -> String {
        format!("nats://127.0.0.1:{}", self.host_port)
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

fn session_host_port() -> Option<u16> {
    let value = std::env::var(SESSION_NATS_HOST_PORT_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.parse::<u16>().unwrap_or_else(|error| {
        panic!("invalid {SESSION_NATS_HOST_PORT_ENV} value '{trimmed}': {error}")
    }))
}

async fn wait_for_nats_port(host_port: u16) {
    let deadline = Instant::now() + NATS_CONNECT_TIMEOUT;
    let server_url = format!("nats://127.0.0.1:{host_port}");

    loop {
        match async_nats::connect(&server_url).await {
            Ok(client) => {
                client.flush().await.expect("flush NATS test client");
                return;
            }
            Err(_) if Instant::now() < deadline => {
                sleep(NATS_CONNECT_RETRY_DELAY).await;
            }
            Err(error) => panic!("failed to connect to NATS test container: {error}"),
        }
    }
}
```

### `crates/testkit/src/containers/postgres.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/containers/postgres.rs`
- Size bytes / Размер в байтах: `4271`
- Included characters / Включено символов: `4271`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::time::{Duration, Instant, sleep};

const POSTGRES_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const POSTGRES_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);
const TEST_POOL_MAX_CONNECTIONS: u32 = 2;
pub const SESSION_POSTGRES_HOST_PORT_ENV: &str = "MAKOSH_TEST_POSTGRES_HOST_PORT";
pub const SESSION_ID_ENV: &str = "MAKOSH_TEST_SESSION_ID";

pub struct PostgresContainer {
    _container: Option<ContainerAsync<GenericImage>>,
    host_port: u16,
}

impl PostgresContainer {
    pub async fn start() -> Self {
        if let Some(host_port) = session_host_port() {
            return Self {
                _container: None,
                host_port,
            };
        }

        Self::start_owned().await
    }

    pub async fn start_owned() -> Self {
        // GenericImage methods (return Self) must come BEFORE ImageExt methods (return ContainerRequest)
        let container = GenericImage::new("pgvector/pgvector", "0.8.2-pg16")
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_exposed_port(5432.tcp())
            // ImageExt methods return ContainerRequest<GenericImage>
            .with_env_var("POSTGRES_DB", "testdb")
            .with_env_var("POSTGRES_USER", "testuser")
            .with_env_var("POSTGRES_PASSWORD", "testpass")
            .start()
            .await
            .expect("failed to start pgvector container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to resolve pgvector container port");

        Self {
            _container: Some(container),
            host_port,
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://testuser:testpass@127.0.0.1:{}/testdb",
            self.host_port
        )
    }

    pub async fn create_database(&self, db_name: &str) -> PgPool {
        let admin_url = format!(
            "postgres://testuser:testpass@127.0.0.1:{}/testdb",
            self.host_port
        );

        let admin_pool = connect_with_retry(&admin_url, "admin database").await;

        let create_sql = format!("CREATE DATABASE \"{}\"", db_name.replace('"', "\"\""));
        sqlx::query(&create_sql)
            .execute(&admin_pool)
            .await
            .unwrap_or_else(|e| panic!("failed to create database '{db_name}': {e}"));

        admin_pool.close().await;

        let db_url = format!(
            "postgres://testuser:testpass@127.0.0.1:{}/{}",
            self.host_port, db_name
        );

        let pool = connect_with_retry(&db_url, "new test database").await;

        makosh_hub_backend::platform::events::run_migrations(&pool)
            .await
            .expect("failed to run migrations");
        makosh_hub_backend::platform::settings::ApplicationSettingsStore::new(pool.clone())
            .repair_declared_settings()
            .await
            .expect("failed to repair application settings");

        pool
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

fn session_host_port() -> Option<u16> {
    let value = std::env::var(SESSION_POSTGRES_HOST_PORT_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.parse::<u16>().unwrap_or_else(|error| {
        panic!("invalid {SESSION_POSTGRES_HOST_PORT_ENV} value '{trimmed}': {error}")
    }))
}

async fn connect_with_retry(database_url: &str, label: &str) -> PgPool {
    let deadline = Instant::now() + POSTGRES_CONNECT_TIMEOUT;
    loop {
        match PgPoolOptions::new()
            .max_connections(TEST_POOL_MAX_CONNECTIONS)
            .connect(database_url)
            .await
        {
            Ok(pool) => return pool,
            Err(_error) if Instant::now() < deadline => {
                sleep(POSTGRES_CONNECT_RETRY_DELAY).await;
            }
            Err(error) => panic!("failed to connect to {label}: {error}"),
        }
    }
}
```

### `crates/testkit/src/context.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/context.rs`
- Size bytes / Размер в байтах: `3929`
- Included characters / Включено символов: `3929`
- Truncated / Обрезано: `no`

````rust
use sqlx::postgres::PgPool;
use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

use crate::containers::nats::NatsContainer;
use crate::containers::postgres::PostgresContainer;
use crate::vault::{TestVault, new_test_vault};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use std::path::Path;

static POSTGRES_CONTAINER: OnceCell<PostgresContainer> = OnceCell::const_new();
static NATS_CONTAINER: OnceCell<NatsContainer> = OnceCell::const_new();
static DATABASE_SETUP_LOCK: Mutex<()> = Mutex::const_new(());

/// Isolated test environment with a fresh migrated database.
///
/// Each `TestContext` creates its own unique database on the PostgreSQL
/// container owned by the current test session. `make backend-test` starts one
/// pgvector container and one NATS container before nextest, then removes them
/// after the session.
///
/// # Usage
///
/// ```ignore
/// #[tokio::test]
/// async fn my_test() {
///     let ctx = TestContext::new().await;
///     let pool = ctx.pool();
///     // ... use pool ...
///     // The pool is dropped with the context.
/// }
/// ```
pub struct TestContext {
    container: &'static PostgresContainer,
    db_name: String,
    vault: TestVault,
    pool: PgPool,
}

impl TestContext {
    /// Create a new isolated test environment.
    ///
    /// 1. Reuses the pgvector container for this test session
    /// 2. Creates a unique database (uuid-based name)
    /// 3. Runs all sqlx migrations
    /// 4. Returns a ready-to-use pool
    pub async fn new() -> Self {
        let container = POSTGRES_CONTAINER
            .get_or_init(PostgresContainer::start)
            .await;
        let db_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let _setup_guard = DATABASE_SETUP_LOCK.lock().await;
        let pool = container.create_database(&db_name).await;
        let vault = new_test_vault();

        Self {
            container,
            db_name,
            vault,
            pool,
        }
    }

    /// The connection pool to the isolated test database.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// The full connection string for this test database.
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://testuser:testpass@127.0.0.1:{}/{}",
            self.container.host_port(),
            self.db_name
        )
    }

    pub fn vault_home(&self) -> &Path {
        self.vault.vault_home()
    }

    pub fn dev_key_path(&self) -> &Path {
        self.vault.dev_key_path()
    }

    pub fn vault_database_path(&self) -> std::path::PathBuf {
        self.vault.vault_database_path()
    }

    pub fn app_config(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault
            .apply_to_config(AppConfig::test_with_api_secret_and_database_url(
                api_secret,
                self.connection_string(),
            ))
    }

    pub async fn nats_server_url(&self) -> String {
        NATS_CONTAINER
            .get_or_init(NatsContainer::start)
            .await
            .server_url()
    }

    pub async fn app_config_with_nats(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault.apply_to_config(
            AppConfig::test_with_api_secret_and_database_url(api_secret, self.connection_string())
                .with_test_pairs([("MAKOSH_NATS_SERVER_URL", self.nats_server_url().await)])
                .expect("test NATS config must be valid"),
        )
    }

    pub fn app_config_without_database(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault
            .apply_to_config(AppConfig::test_with_api_secret(api_secret))
    }

    /// Database runtime wrapper backed by this context's migrated pool.
    pub fn database(&self) -> Database {
        Database::from_test_pool(self.pool.clone(), self.connection_string())
    }
}
````

### `crates/testkit/src/factories/calendar_event.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/calendar_event.rs`
- Size bytes / Размер в байтах: `2351`
- Included characters / Включено символов: `2351`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct CalendarEventFactory<'a> {
    pool: &'a PgPool,
    title: String,
    start_at: chrono::DateTime<Utc>,
    duration_minutes: i64,
    description: Option<String>,
}

impl<'a> CalendarEventFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        let now = Utc::now();
        Self {
            pool,
            title: format!("test-event-{}", Uuid::new_v4()),
            start_at: now + Duration::hours(1),
            duration_minutes: 60,
            description: Some("Auto-generated test calendar event".into()),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_start(mut self, start: chrono::DateTime<Utc>) -> Self {
        self.start_at = start;
        self
    }

    pub fn with_duration_minutes(mut self, minutes: i64) -> Self {
        self.duration_minutes = minutes;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::calendar::events::CalendarEvent,
        makosh_hub_backend::domains::calendar::events::CalendarError,
    > {
        let store = CalendarEventStore::new(self.pool.clone());
        let end_at = self.start_at + Duration::minutes(self.duration_minutes);
        let new_event = NewCalendarEvent {
            source_event_id: None,
            account_id: None,
            source_id: None,
            title: self.title,
            description: self.description,
            location: None,
            start_at: self.start_at,
            end_at,
            timezone: Some("UTC".into()),
            all_day: Some(false),
            recurrence_rule: None,
            status: Some("confirmed".into()),
            visibility: Some("default".into()),
            event_type: Some("meeting".into()),
            conference_url: None,
            conference_provider: None,
            preparation_reminder_minutes: None,
            travel_buffer_minutes: None,
        };
        store.create(&new_event).await
    }
}
```

### `crates/testkit/src/factories/contact.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/contact.rs`
- Size bytes / Размер в байтах: `2911`
- Included characters / Включено символов: `2911`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::core::{
    NewPersonPersona, PersonPersonaStore, PersonsIdentityStore,
};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct ContactFactory<'a> {
    pool: &'a PgPool,
    display_name: String,
    email: Option<String>,
    person_id: Option<String>,
}

impl<'a> ContactFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            display_name: format!(
                "Test Person {}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ),
            email: None,
            person_id: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_person_id(mut self, id: impl Into<String>) -> Self {
        self.person_id = Some(id.into());
        self
    }

    /// Create a person identity and a default persona. Returns the person ID.
    pub async fn create(
        self,
    ) -> Result<String, makosh_hub_backend::domains::persons::core::PersonCoreError> {
        let identity_store = PersonsIdentityStore::new(self.pool.clone());
        let persona_store = PersonPersonaStore::new(self.pool.clone());

        let person_id = self
            .person_id
            .unwrap_or_else(|| format!("person:{}", Uuid::new_v4()));
        let email = self
            .email
            .unwrap_or_else(|| format!("{}@example.test", Uuid::new_v4()));

        sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (person_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                email_address = EXCLUDED.email_address,
                updated_at = now()
            "#,
        )
        .bind(&person_id)
        .bind(&self.display_name)
        .bind(&email)
        .execute(self.pool)
        .await?;

        // Create identity via upsert
        identity_store
            .upsert(&person_id, "email", &email, "testkit")
            .await?;

        // Create a default persona
        let persona = NewPersonPersona {
            persona_id: format!("persona:{}", Uuid::new_v4()),
            person_id: person_id.clone(),
            name: self.display_name,
            context: Some("test".into()),
            default_tone: Some("neutral".into()),
            default_language: Some("en".into()),
            preferred_channel: Some(email),
        };
        persona_store.upsert(&persona).await?;

        Ok(person_id)
    }
}
```

### `crates/testkit/src/factories/document.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/document.rs`
- Size bytes / Размер в байтах: `1623`
- Included characters / Включено символов: `1623`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct DocumentFactory<'a> {
    pool: &'a PgPool,
    title: String,
    doc_kind: String,
    text: String,
}

impl<'a> DocumentFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            title: "Test Document".into(),
            doc_kind: "markdown".into(),
            text: "# Test Document\n\nAuto-generated for integration testing.".into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.doc_kind = kind.into();
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::documents::core::ImportedDocument,
        makosh_hub_backend::domains::documents::core::DocumentImportError,
    > {
        let store = DocumentImportStore::new(self.pool.clone());
        let fingerprint = format!("{:x}", Sha256::digest(self.text.as_bytes()));
        let new_doc = NewDocumentImport {
            document_id: format!("doc:{}", Uuid::new_v4()),
            document_kind: self.doc_kind,
            title: self.title,
            source_fingerprint: fingerprint,
            extracted_text: self.text,
        };
        store.import_document(&new_doc).await
    }
}
```

### `crates/testkit/src/factories/email.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/email.rs`
- Size bytes / Размер в байтах: `3038`
- Included characters / Включено символов: `3038`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct EmailFactory<'a> {
    pool: &'a PgPool,
    account_id: Option<String>,
    subject: String,
    from_address: String,
    body_text: String,
}

impl<'a> EmailFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            account_id: None,
            subject: "Test Email Subject".into(),
            from_address: "test@example.com".into(),
            body_text: "This is a test email body for integration testing.".into(),
        }
    }

    pub fn with_account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from_address = from.into();
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body_text = body.into();
        self
    }

    /// Create a provider account and a raw communication record for an email.
    /// Returns the raw record.
    pub async fn create(
        self,
    ) -> Result<
        (NewProviderAccount, NewRawCommunicationRecord),
        makosh_hub_backend::domains::communications::core::CommunicationIngestionError,
    > {
        let store = CommunicationIngestionStore::new(self.pool.clone());

        let account_id = self
            .account_id
            .unwrap_or_else(|| format!("acct:{}", Uuid::new_v4()));
        let account = NewProviderAccount {
            account_id: account_id.clone(),
            provider_kind: EmailProviderKind::Gmail,
            display_name: "Test Gmail Account".into(),
            external_account_id: format!("gm-{}", Uuid::new_v4()),
            config: serde_json::json!({"email": self.from_address}),
        };
        store.upsert_provider_account(&account).await?;

        let record_id = format!("rec:{}", Uuid::new_v4());
        let raw = NewRawCommunicationRecord {
            raw_record_id: record_id.clone(),
            account_id,
            record_kind: "email".into(),
            provider_record_id: format!("msg-{}", Uuid::new_v4()),
            source_fingerprint: format!("fp-{}", Uuid::new_v4()),
            import_batch_id: format!("batch-{}", Uuid::new_v4()),
            occurred_at: Some(Utc::now()),
            payload: serde_json::json!({
                "subject": self.subject,
                "from": self.from_address,
                "to": ["recipient@example.com"],
                "body_text": self.body_text,
            }),
            provenance: serde_json::json!({
                "source": "EmailFactory",
                "provider": "gmail",
            }),
        };

        Ok((account, raw))
    }
}
```

### `crates/testkit/src/factories/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/mod.rs`
- Size bytes / Размер в байтах: `127`
- Included characters / Включено символов: `127`
- Truncated / Обрезано: `no`

```rust
pub mod calendar_event;
pub mod contact;
pub mod document;
pub mod email;
pub mod organization;
pub mod project;
pub mod task;
```

### `crates/testkit/src/factories/organization.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/organization.rs`
- Size bytes / Размер в байтах: `1298`
- Included characters / Включено символов: `1298`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::organizations::api::OrganizationStore;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct OrganizationFactory<'a> {
    pool: &'a PgPool,
    display_name: String,
    org_type: Option<String>,
}

impl<'a> OrganizationFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            display_name: format!(
                "Test Org {}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ),
            org_type: Some("company".into()),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn with_type(mut self, org_type: impl Into<String>) -> Self {
        self.org_type = Some(org_type.into());
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::organizations::api::Organization,
        makosh_hub_backend::domains::organizations::api::OrganizationError,
    > {
        let store = OrganizationStore::new(self.pool.clone());
        store
            .create(&self.display_name, self.org_type.as_deref())
            .await
    }
}
```

### `crates/testkit/src/factories/project.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/project.rs`
- Size bytes / Размер в байтах: `1745`
- Included characters / Включено символов: `1745`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::projects::core::{NewProject, ProjectStore};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct ProjectFactory<'a> {
    pool: &'a PgPool,
    name: String,
    kind: String,
    status: String,
    description: String,
}

impl<'a> ProjectFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            name: format!("test-project-{}", Uuid::new_v4()),
            kind: "software".into(),
            status: "active".into(),
            description: "Auto-generated test project".into(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::projects::core::Project,
        makosh_hub_backend::domains::projects::core::ProjectStoreError,
    > {
        let store = ProjectStore::new(self.pool.clone());
        let project_id = format!("proj:v1:test:{}", Uuid::new_v4());
        let new_project = NewProject {
            project_id,
            name: self.name,
            kind: self.kind,
            status: self.status,
            description: self.description,
            owner_display_name: "Test Owner".into(),
            progress_percent: 0,
            start_date: None,
            target_date: None,
            keywords: vec!["test".into(), "factory".into()],
        };
        store.upsert_project(&new_project).await
    }
}
```

### `crates/testkit/src/factories/task.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/factories/task.rs`
- Size bytes / Размер в байтах: `3323`
- Included characters / Включено символов: `3323`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use makosh_hub_backend::domains::tasks::api::{NewTask, TaskStore};
use sqlx::postgres::PgPool;
use uuid::Uuid;

/// Factory for creating test tasks with sensible defaults.
pub struct TaskFactory<'a> {
    pool: &'a PgPool,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority_score: Option<f64>,
    area: Option<String>,
    due_at: Option<chrono::DateTime<Utc>>,
    project_id: Option<String>,
    linked_person_id: Option<String>,
    linked_organization_id: Option<String>,
}

impl<'a> TaskFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            title: format!("test-task-{}", Uuid::new_v4()),
            description: Some("Auto-generated test task".into()),
            status: Some("new".into()),
            priority_score: Some(0.5),
            area: Some("general".into()),
            due_at: None,
            project_id: None,
            linked_person_id: None,
            linked_organization_id: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_priority(mut self, score: f64) -> Self {
        self.priority_score = Some(score);
        self
    }

    pub fn with_area(mut self, area: impl Into<String>) -> Self {
        self.area = Some(area.into());
        self
    }

    pub fn with_due_date(mut self, due: chrono::DateTime<Utc>) -> Self {
        self.due_at = Some(due);
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::tasks::api::Task,
        makosh_hub_backend::domains::tasks::api::TaskError,
    > {
        let store = TaskStore::new(self.pool.clone());
        let new_task = NewTask {
            title: self.title,
            description: self.description,
            provenance_kind: Some("observation".into()),
            provenance_id: Some(format!("observation:v1:test-factory:{}", Uuid::new_v4())),
            source_kind: Some("import".into()),
            source_id: Some(format!("test-src-{}", Uuid::new_v4())),
            source_type: Some("import".into()),
            project_id: self.project_id,
            makosh_status: self.status,
            priority_score: self.priority_score,
            area: self.area,
            why: Some("Created by TaskFactory for integration testing".into()),
            due_at: self.due_at,
            energy_type: Some("medium".into()),
            confidentiality: Some("private_local".into()),
            tags: Some(serde_json::json!(["test", "factory"])),
            linked_person_id: self.linked_person_id,
            linked_organization_id: self.linked_organization_id,
            created_from_event_id: None,
            created_by_actor_id: Some("testkit".into()),
        };
        store.create(&new_task).await
    }
}
```

### `crates/testkit/src/lib.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/lib.rs`
- Size bytes / Размер в байтах: `698`
- Included characters / Включено символов: `698`
- Truncated / Обрезано: `no`

````rust
//! Test infrastructure for Макошь.
//!
//! Provides programmatic container management, isolated test databases,
//! and entity factories for integration testing.
//!
//! # Usage
//!
//! ```ignore
//! use testkit::context::TestContext;
//! use testkit::factories::task::TaskFactory;
//!
//! #[tokio::test]
//! async fn my_integration_test() {
//!     let ctx = TestContext::new().await;
//!     let task = TaskFactory::new(ctx.pool())
//!         .with_title("Review Q1 report")
//!         .create()
//!         .await
//!         .unwrap();
//!     assert_eq!(task.title, "Review Q1 report");
//! }
//! ```

pub mod app;
pub mod containers;
pub mod context;
pub mod factories;
pub mod vault;
````

### `crates/testkit/src/vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/src/vault.rs`
- Size bytes / Размер в байтах: `1731`
- Included characters / Включено символов: `1731`
- Truncated / Обрезано: `no`

```rust
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use makosh_hub_backend::platform::config::AppConfig;
use tempfile::TempDir;

static RETAINED_VAULTS: OnceLock<Mutex<Vec<TestVault>>> = OnceLock::new();

fn retained_vaults() -> &'static Mutex<Vec<TestVault>> {
    RETAINED_VAULTS.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Debug)]
pub struct TestVault {
    _dir: TempDir,
    vault_home: PathBuf,
    dev_key_path: PathBuf,
}

impl TestVault {
    pub fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("makosh-test-vault-")
            .tempdir()
            .expect("create temporary test vault directory");

        let vault_home = dir.path().join("vault");
        let dev_key_path = dir.path().join("dev").join("master.key");

        Self {
            _dir: dir,
            vault_home,
            dev_key_path,
        }
    }

    pub fn vault_home(&self) -> &Path {
        &self.vault_home
    }

    pub fn dev_key_path(&self) -> &Path {
        &self.dev_key_path
    }

    pub fn vault_database_path(&self) -> PathBuf {
        self.vault_home.join("vault.db")
    }

    pub fn apply_to_config(&self, config: AppConfig) -> AppConfig {
        config.with_test_dev_vault_paths(self.vault_home.clone(), self.dev_key_path.clone())
    }
}

impl Default for TestVault {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_test_vault() -> TestVault {
    TestVault::new()
}

pub fn retain_test_vault_and_apply(config: AppConfig) -> AppConfig {
    let vault = TestVault::new();
    let config = vault.apply_to_config(config);

    retained_vaults()
        .lock()
        .expect("test vault retention lock")
        .push(vault);

    config
}
```
