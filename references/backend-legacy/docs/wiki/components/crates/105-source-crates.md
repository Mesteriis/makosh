---
chunk_id: 105-source-crates
batch_id: batch-20260628T214902
group: crates
role: source
source_status: pending
source_count: 16
generated_by: code-wiki-ru
---

# 105-source-crates — crates/source

- Target index: [[components/crates]]
- Batch: `batch-20260628T214902`
- Source files: `16`

## Резюме

Страница `components/crates.md` в русской Obsidian‑wiki нуждается в документировании крейта `testkit`, чтобы отразить его текущую архитектуру, модули и пример использования на основе исходного кода. Предыдущая версия страницы (если существовала) не содержит этих деталей или информация устарела.

## Предложенные страницы

`components/crates.md`

```markdown
# Пакеты (crates)

## testkit

`testkit` — инфраструктурный крейт для интеграционного тестирования Макошь.

Предоставляет:

- Программное управление контейнерами (PostgreSQL, NATS) с автоматическим переиспользованием в рамках тестовой сессии.
- Изолированные тестовые базы данных с автоматическим запуском миграций.
- Фабрики сущностей для быстрого наполнения тестовой БД.
- Тестовое приложение (`TestApp`) с готовым маршрутизатором для HTTP-тестов.

### Модули

#### `app`

- `TestApp` — обёртка над `axum::Router` и `TestContext`, создаётся вызовом `TestApp::new().await`.
- `config()` и варианты — создают `AppConfig` с тестовым API-секретом `makosh-test-api-secret` и опционально с URL базы данных.
- `router_for_context` строит маршрутизатор через `build_router_with_database`.
- Вспомогательные функции для HTTP-запросов: `get`, `post_json`, `put_json`, `patch_json`, `delete` — добавляют заголовок `x-makosh-secret`.

#### `context`

- `TestContext` — изолированное тестовое окружение.
- При создании переиспользует `PostgresContainer` (один на тестовую сессию через `OnceCell`) и создаёт уникальную базу данных с именем `test_<uuid>`.
- Запускает `sqlx`-миграции и восстанавливает настройки приложения.
- Предоставляет `pool()` (PgPool), `connection_string()`, пути хранилища (`vault_home`, `dev_key_path`, `vault_database_path`), методы для построения `AppConfig` с БД и NATS, а также `database()` (обёртка `Database` для тестов).
- Для сериализации создания баз данных используется `Mutex`.

#### `containers`

- `PostgresContainer` — запускает образ `pgvector/pgvector:0.8.2-pg16`, ожидает сообщение о готовности БД. Параметры БД: `testdb/testuser/testpass`.
- `NatsContainer` — запускает образ `nats:2.11-alpine` с флагами `-js -sd /data`.
- Оба контейнера могут переиспользовать порты, заданные переменными окружения `MAKOSH_TEST_POSTGRES_HOST_PORT` и `MAKOSH_TEST_NATS_HOST_PORT` (режим тестовой сессии).

#### `factories`

Фабрики с паттерном «строитель» (`with_*`). Находятся в подмодулях:

- `calendar_event` — `CalendarEventFactory` (события календаря).
- `contact` — `ContactFactory` (персоны и персоны-персоны).
- `document` — `DocumentFactory` (документы, fingerprint через SHA-256).
- `email` — `EmailFactory` (аккаунт провайдера Gmail и сырая запись).
- `organization` — `OrganizationFactory` (организации, тип по умолчанию "company").
- `project` — `ProjectFactory` (проекты, статус по умолчанию "active").
- `task` — `TaskFactory` (задачи, статус по умолчанию "new", приоритет 0.5).

#### `vault`

- `TestVault` — временная директория с путями `vault_home`, `dev_key_path` и `vault_database_path`.
- Функции `retain_test_vault_and_apply` удерживают `TestVault` в глобальном векторе до завершения процесса, чтобы временные файлы не удалились во время выполнения конфигурационных функций.

### Утилита `makosh-test-session`

Бинарный файл `makosh-test-session` запускает контейнеры PostgreSQL и NATS, экспортирует переменные окружения (`MAKOSH_TEST_SESSION_ID`, `MAKOSH_TEST_POSTGRES_HOST_PORT`, `MAKOSH_TEST_NATS_HOST_PORT`) и выполняет указанную команду. Завершается с кодом возврата команды.

### Пример использования

```rust
use testkit::context::TestContext;
use testkit::factories::task::TaskFactory;

#[tokio::test]
async fn my_integration_test() {
    let ctx = TestContext::new().await;
    let task = TaskFactory::new(ctx.pool())
        .with_title("Review Q1 report")
        .create()
        .await
        .unwrap();
    assert_eq!(task.title, "Review Q1 report");
}
```
```

## Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `crates/testkit/src/lib.rs` | Назначение крейта, пример использования с `TestContext` и `TaskFactory`, список публичных модулей |
| `crates/testkit/src/app.rs` | `TestApp`, конфигурации `AppConfig`, роутер, строители HTTP-запросов с заголовком `x-makosh-secret` |
| `crates/testkit/src/context.rs` | `TestContext`: изоляция баз, `OnceCell` для контейнеров, `database()`, `nats_server_url()`, `app_config*`, `Mutex` для создания БД |
| `crates/testkit/src/containers/mod.rs` | Подмодули `nats` и `postgres` |
| `crates/testkit/src/containers/nats.rs` | `NatsContainer`: образ `nats:2.11-alpine`, порт 4222, переменная `MAKOSH_TEST_NATS_HOST_PORT`, ожидание подключения с таймаутом |
| `crates/testkit/src/containers/postgres.rs` | `PostgresContainer`: образ `pgvector/pgvector:0.8.2-pg16`, пользователь/пароль/БД, создание баз, миграции, восстановление настроек, переменные окружения сессии |
| `crates/testkit/src/factories/mod.rs` | Перечень фабрик (calendar_event, contact, document, email, organization, project, task) |
| `crates/testkit/src/factories/calendar_event.rs` | `CalendarEventFactory`, параметры по умолчанию |
| `crates/testkit/src/factories/contact.rs` | `ContactFactory`, создание идентичности и персоны через `PersonsIdentityStore` и `PersonPersonaStore` |
| `crates/testkit/src/factories/document.rs` | `DocumentFactory`, тип по умолчанию `markdown`, fingerprint SHA-256 |
| `crates/testkit/src/factories/email.rs` | `EmailFactory`, аккаунт Gmail, сырая запись через `CommunicationIngestionStore` |
| `crates/testkit/src/factories/organization.rs` | `OrganizationFactory`, тип по умолчанию `company` |
| `crates/testkit/src/factories/project.rs` | `ProjectFactory`, статус по умолчанию `active`, префикс `proj:v1:test:` |
| `crates/testkit/src/factories/task.rs` | `TaskFactory`, статус `new`, приоритет `0.5`, область `general` |
| `crates/testkit/src/vault.rs` | `TestVault`, временная директория, удержание через `retain_test_vault_and_apply` |
| `crates/testkit/src/bin/makosh_test_session.rs` | Бинарная утилита, переменные сессии, запуск команды с наследованием stdio |

## Исходные файлы

- [`crates/testkit/src/app.rs`](../../../../crates/testkit/src/app.rs)
- [`crates/testkit/src/bin/makosh_test_session.rs`](../../../../crates/testkit/src/bin/makosh_test_session.rs)
- [`crates/testkit/src/containers/mod.rs`](../../../../crates/testkit/src/containers/mod.rs)
- [`crates/testkit/src/containers/nats.rs`](../../../../crates/testkit/src/containers/nats.rs)
- [`crates/testkit/src/containers/postgres.rs`](../../../../crates/testkit/src/containers/postgres.rs)
- [`crates/testkit/src/context.rs`](../../../../crates/testkit/src/context.rs)
- [`crates/testkit/src/factories/calendar_event.rs`](../../../../crates/testkit/src/factories/calendar_event.rs)
- [`crates/testkit/src/factories/contact.rs`](../../../../crates/testkit/src/factories/contact.rs)
- [`crates/testkit/src/factories/document.rs`](../../../../crates/testkit/src/factories/document.rs)
- [`crates/testkit/src/factories/email.rs`](../../../../crates/testkit/src/factories/email.rs)
- [`crates/testkit/src/factories/mod.rs`](../../../../crates/testkit/src/factories/mod.rs)
- [`crates/testkit/src/factories/organization.rs`](../../../../crates/testkit/src/factories/organization.rs)
- [`crates/testkit/src/factories/project.rs`](../../../../crates/testkit/src/factories/project.rs)
- [`crates/testkit/src/factories/task.rs`](../../../../crates/testkit/src/factories/task.rs)
- [`crates/testkit/src/lib.rs`](../../../../crates/testkit/src/lib.rs)
- [`crates/testkit/src/vault.rs`](../../../../crates/testkit/src/vault.rs)

## Кандидаты на drift

Явных расхождений в предоставленных исходниках не обнаружено. Однако:

- В doc-комментарии `TestContext` упоминается `make backend-test`, но сам Makefile в этом контексте отсутствует — нельзя подтвердить, актуальна ли таргетная команда.
- Код опирается на API `makosh_hub_backend` (например, `build_router_with_database`, `run_migrations`). Изменение сигнатур этих функций могло бы привести к дрейфу, но исходные тексты backend в данном чанке не представлены, поэтому дрейф с ними не проверяем.
