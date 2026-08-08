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

- Chunk ID / ID чанка: `106-test-crates`
- Group / Группа: `crates`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/crates-tests.md`

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

### `crates/testkit/tests/smoke_test.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/crates/testkit/tests/smoke_test.rs`
- Size bytes / Размер в байтах: `3758`
- Included characters / Включено символов: `3727`
- Truncated / Обрезано: `no`

```rust
//! Smoke test for Testcontainers infrastructure in crates/testkit.
//!
//! Verifies that:
//! - AC1: Testcontainers PostgreSQL контейнер поднимается и проходит health check
//! - Migration runner works against the container
//! - Entity factories can create records
//!
//! This is a characterization test — it captures CURRENT behavior without
//! modifying it. Any change to the infrastructure will cause these tests to
//! fail, alerting the developer to review compatibility.

use testkit::context::TestContext;

/// AC1: Testcontainers PostgreSQL container starts and passes health check.
#[tokio::test]
async fn test_context_creates_isolated_database() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool();

    // Verify basic query works (container is reachable, DB is responsive)
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("basic query against test database must succeed");
    assert_eq!(result, 1, "pgvector container must respond to queries");
}

/// Verify migrations were applied successfully.
#[tokio::test]
async fn test_context_runs_migrations() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool();

    // Check that _sqlx_migrations table exists (proof migrations ran)
    let migration_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true")
            .fetch_one(pool)
            .await
            .expect("migrations table must exist and be queryable");
    assert!(
        migration_count > 0,
        "at least one migration must have been applied (got {migration_count})"
    );
}

/// Verify each test gets a unique, isolated database.
#[tokio::test]
async fn test_context_databases_are_isolated() {
    let ctx_a = TestContext::new().await;
    let ctx_b = TestContext::new().await;

    let a_conn = ctx_a.connection_string();
    let b_conn = ctx_b.connection_string();

    assert_ne!(
        a_conn, b_conn,
        "each TestContext must produce a unique database connection string"
    );
}

/// Verify the ContactFactory works against a real container.
#[tokio::test]
async fn testkit_contact_factory_creates_person() {
    let ctx = TestContext::new().await;
    let factory = testkit::factories::contact::ContactFactory::new(ctx.pool());

    let person_id = factory
        .with_name("Characterization Test Person")
        .with_email("char-test@example.com")
        .create()
        .await
        .expect("ContactFactory must create a person against the test container");

    assert!(
        !person_id.is_empty(),
        "ContactFactory must return a non-empty person ID"
    );

    // Verify the person actually exists in the database
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM persons WHERE person_id = $1)")
            .bind(&person_id)
            .fetch_one(ctx.pool())
            .await
            .expect("query must succeed");
    assert!(exists, "person created by ContactFactory must exist in DB");
}

/// Verify the EmailFactory works against a real container.
#[tokio::test]
async fn testkit_email_factory_creates_email() {
    let ctx = TestContext::new().await;
    let factory = testkit::factories::email::EmailFactory::new(ctx.pool());

    let (_account, raw) = factory
        .with_subject("Characterization Test Email")
        .with_from("sender@example.com")
        .with_body("This is a characterization test email body.")
        .create()
        .await
        .expect("EmailFactory must create an email record against the test container");

    assert!(
        !raw.raw_record_id.is_empty(),
        "EmailFactory must return a non-empty raw_record_id"
    );
}
```
