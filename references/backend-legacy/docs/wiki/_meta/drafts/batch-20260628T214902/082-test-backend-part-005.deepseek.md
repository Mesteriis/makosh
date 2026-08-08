### Summary / Резюме

Страница `operations/backend-tests.md` не была включена в этот context pack, поэтому я предлагаю её полное содержимое заново — структурированный справочник по тестам email-подсистемы бэкенда, основанный на всех предоставленных исходных файлах. Каждое утверждение подтверждается именем тестовой функции и проверяемыми утверждениями (`assert!`, `assert_eq!`) из кода.

### Proposed pages / Предлагаемые страницы

#### `operations/backend-tests.md`

````markdown
---
title: Backend-тесты
type: operations
tags: [testing, backend, email, rust]
date: 2026-06-28
---

# Backend-тесты

Набор интеграционных и unit-тестов для email-подсистемы `makosh-backend`.

Все тесты выполняются в `#[tokio::test]` (асинхронно) либо как синхронные `#[test]` и используют изолированную базу данных PostgreSQL, предоставляемую `TestContext` из `testkit`.

## Структура тестовых файлов

```
backend/tests/
├── email_account_setup/
│   ├── gmail_service.rs         # Gmail OAuth-настройка
│   ├── imap_api.rs              # IMAP-настройка и API создания учётной записи iCloud
│   ├── send_api.rs              # Отправка через API (outbox)
│   ├── support.rs               # Вспомогательные mock-серверы и утилиты
│   └── vault_reconciliation.rs  # Восстановление аккаунта из HostVault после wipe
├── email_account_setup_architecture.rs  # Архитектурный guard на длину файлов
├── email_fixture_export.rs      # Экспорт фикстур (обезличивание писем)
├── email_fixture_pipeline.rs    # Полный пайплайн импорта фикстур (сообщения → персоны/граф)
├── email_import.rs              # Парсинг и импорт фикстур email
├── email_outbox.rs              # Исходящая очередь (outbox): отправка, отказы, retry
├── email_provider_network.rs    # Сетевые клиенты провайдеров (Gmail API, IMAP)
├── email_rfc822.rs              # Парсинг RFC 822 (MIME, вложения, кодировки)
└── email_sync.rs                # Планирование синхронизации с почтовыми ящиками
```

## Общая инфраструктура

- **`TestContext::new().await`** – запускает временный экземпляр PostgreSQL.
- **`live_setup_context(test_name)`** – возвращает `(Database, CommunicationIngestionStore, SecretReferenceStore, u128 unique_suffix)`.
- **`json_request_with_token_and_actor`** – формирует POST-запрос с JSON-телом и заголовком `x-makosh-secret`.
- **`unlock_test_vault(app)`** – выполняет сбор энтропии и создание HostVault через API `/api/v1/vault/collect-entropy` и `/api/v1/vault/create`.
- Мок-серверы **`MockTokenServer`** (OAuth token endpoint) и **`MockSmtpServer`** (SMTP) запускаются на случайных портах и собирают запросы/команды для проверок.

## Категории тестов

### 1. Настройка учётной записи (account setup)

#### Gmail OAuth-настройка (`gmail_service.rs`)

- **`gmail_oauth_setup_builds_pkce_url_and_persists_token_bundle_against_postgres`**
  - `EmailAccountSetupService::start_gmail_oauth` генерирует URL авторизации, который содержит:
    - `code_challenge=`
    - `code_challenge_method=S256`
    - `access_type=offline`
    - `prompt=consent`
    - scope `gmail.readonly` и `gmail.send`
    - `code_verifier` отсутствует в URL
  - `complete_gmail_oauth` завершает обмен кода на токены и сохраняет:
    - `ProviderAccount` с `config["auth"] = "oauth"`, `config["api"] = "gmail"`, `config["oauth_client_id"] = "desktop-client-id"`, **без** access/refresh token
    - `ProviderAccountSecretBinding` для `ProviderAccountSecretPurpose::OauthToken`, ссылающуюся на `completed.secret_ref`
    - `SecretReference` с `store_kind = DatabaseEncryptedVault`, `secret_kind = OauthToken`
    - токен-бандл в `DatabaseEncryptedSecretVault`, содержащий `access_token: "gmail-access-token"`, `refresh_token: "gmail-refresh-token"`, `client_id: "desktop-client-id"`
  - Mock-сервер зафиксировал один запрос с `grant_type=authorization_code`, `code=authorization-code`, `code_verifier=`

- **`gmail_oauth_refresh_returns_runtime_access_token_and_updates_vault`**
  - Помещает в хранилище истёкший токен (`expires_at = "2000-01-01T00:00:00Z"`)
  - `EmailAccountSetupService::refresh_gmail_access_token` возвращает `"gmail-refreshed-access-token"` и обновляет бандл в хранилище (access_token новый, refresh_token сохранён)
  - Mock-сервер получил запрос с `grant_type=refresh_token` и `refresh_token=gmail-refresh-token`

#### IMAP-настройка (`imap_api.rs`)

- **`imap_account_setup_stores_encrypted_secret_in_database_against_postgres`**
  - `EmailAccountSetupService::setup_imap_account` (для `EmailProviderKind::Icloud`) создаёт аккаунт с корректными `host`, `port`, `tls`, `mailbox`, `username` и **не содержит** `password`/`app_password` в `config`
  - `SecretReference` сохраняется с `store_kind = DatabaseEncryptedVault`, `secret_kind = AppPassword`
  - `DatabaseEncryptedSecretVault` отдаёт пароль `"icloud-app-password"` при resolve

- **`icloud_account_setup_api_creates_calendar_account_against_postgres`**
  - API `POST /api/v1/integrations/mail/accounts/imap` для iCloud-аккаунта возвращает `200` и `account_id`
  - Аккаунт сохраняется с `connected_services = ["mail", "calendar", "contacts"]`, `smtp_host = "smtp.mail.me.com"`, `smtp_port = 587`, `smtp_tls = true`, `smtp_starttls = true`, `smtp_username` из запроса, **без** паролей в конфиге
  - Создаётся `signal_connection` с `source_code = "mail"`, `status = "connected"`, `secret_ref = "secret:provider-account:icloud-primary:imap_password"`
  - Создаются привязки секретов `ImapPassword` и `SmtpPassword`
  - SMTP-секрет сохраняется как `SecretStoreKind::HostVault`, `SecretKind::AppPassword`
  - В `observation_links` появляются записи о создании аккаунта (`COMMUNICATION_PROVIDER_ACCOUNT`) и привязке секретов (`COMMUNICATION_PROVIDER_SECRET_BINDING`) с `origin_kind = "local_runtime"`

> **Важно:** остаток файла `imap_api.rs` обрезан в контекст-пакете. Видимых проверок календарного аккаунта и дополнительных сценариев в предоставленном фрагменте нет.

#### Отправка через API (`send_api.rs`)

- **`imap_send_api_queues_outbox_without_direct_smtp_against_postgres`**
  - Настроенный IMAP-аккаунт.
  - `POST /api/v1/communications/send` с `confirmed_provider_write: true` возвращает `transport = "outbox"`, `status = "queued"`, `outbox_id`, `message_id`, `accepted_recipients`
  - Запись в `communication_outbox` имеет `status = "queued"`, корректные участников и тему
  - `observation_link` для `outbox_item` с `relationship_kind = "outbox_status_transition"` содержит `metadata.operation = "outbox_enqueue"`, `metadata.status = "queued"`
  - Mock SMTP-сервер **не получил** команд `AUTH LOGIN`, `MAIL FROM`, `RCPT TO`, `DATA` — отправка только в очередь

- **`gmail_send_api_queues_outbox_without_direct_gmail_client_against_postgres`**
  - Gmail-аккаунт без OAuth-потока.
  - Отправка через `POST /api/v1/communications/send` даёт `transport = "outbox"`, `status = "queued"`, заполненный `outbox_id`

- **`imap_send_api_queues_without_smtp_password_binding_against_postgres`**
  - После удаления привязки `smtp_password` из БД отправка по-прежнему приводит к `transport = "outbox"`, `status = "queued"`
  - Mock SMTP не получает `MAIL FROM`

- **`imap_send_api_does_not_send_when_audit_record_fails_against_postgres`**
  - Имя теста заявляет, что при сбое аудит-записи письмо **не отправляется**. Остаток тела недоступен в этом контекст-пакете.

### 2. Восстановление из хранилища (vault reconciliation)

- **`startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe`**
  - Создаётся iCloud-аккаунт через API.
  - В HostVault записывается sparse-запись манифеста с `SecretEntryContext`.
  - Затем из PostgreSQL удаляются: calendar_accounts, secret bindings, provider_accounts, secret_references.
  - После перезапуска приложения аккаунт **восстанавливается** на основе манифеста:
    - `ProviderAccount` с `display_name = "Recovered iCloud"`, `connected_services = ["mail", "calendar", "contacts"]`
    - `observation_link` для аккаунта имеет `origin_kind = "vault_source"`
    - `SecretReference` с `store_kind = "host_vault"`, `secret_kind = AppPassword`
    - Привязка секрета восстанавливается с тем же `secret_ref`
    - `CalendarAccount` с `provider = "apple"`, `email = "recover@icloud.com"`, `credentials_reference` на тот же `secret_ref`, `observation_link` с `origin_kind = "vault_source"`
  - Значение секрета в HostVault остаётся `"icloud-app-password"`

### 3. Экспорт фикстур (fixture export)

- **`imap_raw_message_exports_redacted_fixture_without_personal_content`**
  - `export_fixture_messages_from_sync_batch` преобразует сырое письмо в фикстуру
  - `subject` начинается с `"Redacted subject "`, `from` и `to` заменены на `@example.invalid`, `body_text` содержит `"Redacted body fixture"`
  - В итоговом JSON отсутствуют оригинальные адреса и текст

- **`imap_multipart_quoted_printable_message_exports_redacted_fixture`**
  - Многочастное quoted-printable письмо успешно обезличивается
  - `body_text` содержит `"original_chars=12"` (мета-информация о длине исходного тела)

### 4. Пайплайн фикстур (fixture pipeline)

- **`fixture_email_pipeline_imports_projects_persons_and_graph_against_postgres`**
  - `project_fixture_email_messages` загружает JSON-фикстуру и выдаёт отчёт с `imported_records = 1`, `projected_messages = 1`, `upserted_persons = 2`, `graph_summary.is_empty = false`, `total_graph_nodes >= 4`, `total_graph_edges >= 3`
  - В `event_log` появляется событие `signal.accepted.mail.message`
  - Прослеживается цепочка причинности в `event_log`:
    1. `observation.captured.v1`
    2. `signal.raw.mail.message.observed` (`causation_id` → observation event)
    3. `signal.accepted.mail.message` (`causation_id` → raw event)
    4. `communication.message.recorded` (`causation_id` → accepted event)

### 5. Импорт фикстур (fixture import)

- **`fixture_email_source_parses_account_scoped_messages`**
  - `parse_fixture_email_messages` парсит JSON в вектор `FixtureCommunicationSourceMessage` с ожидаемыми значениями полей

- **`fixture_email_import_records_raw_messages_idempotently_against_postgres`**
  - `import_fixture_email_messages` идемпотентен: два импорта одной фикстуры дают по 1 `inserted_or_existing_records`, но в `communication_raw_records` только одна запись

- **`fixture_email_import_records_delimiter_bearing_identities_distinctly_against_postgres`**
  - `provider_record_id`, содержащий `:`, не вызывает коллизий:
    - в одном аккаунте две записи с разными суффиксами после `:` дают разные `raw_record_id`
    - два аккаунта с общим префиксом `provider_record_id` (например, `"left:right"` и `"right"`) также получают разные `raw_record_id`

- **`fixture_email_import_preserves_missing_sent_at_as_null_against_postgres`**
  - Если в фикстуре нет `sent_at`, поле `occurred_at` записи в `communication_raw_records` — `NULL`

- **`fixture_email_import_returns_raw_records_for_projection_against_postgres`** (файл обрезан)
  - Название указывает на тест `import_fixture_email_messages_with_records`, возвращающий сырые записи для последующего проецирования

### 6. Исходящая очередь (outbox)

- **`outbox_claim_due_waits_for_schedule_and_undo_deadline_against_postgres`**
  - `CommunicationOutboxStore::claim_due` не выбирает элементы до истечения `undo_deadline_at`
  - После `undo_deadline_at` запись переходит в `Sending`, `send_attempts = 1`, `claimed_at` установлен

- **`outbox_delivery_worker_marks_sent_and_appends_event_against_postgres`**
  - `EmailOutboxDeliveryWorker` с успешным отправителем переводит элемент в `Sent`, заполняет `provider_message_id`, `sent_at`
  - В `event_log` появляется `mail.outbox.sent`
  - В `observation_links` — запись `outbox_status_transition` с `metadata.status = "sent"`, `origin_kind = "local_runtime"`

- **`outbox_delivery_worker_marks_failed_and_appends_event_against_postgres`**
  - С отправителем-неудачником элемент переходит в `Failed`, `last_error` содержит сообщение об ошибке
  - Событие `mail.outbox.failed`, `observation_link` с `metadata.status = "failed"`

- **`outbox_delivery_worker_schedules_retry_with_backoff_against_postgres`**
  - При retry-политике (3 попытки, backoff 60s) после неудачи элемент становится `Scheduled`, `scheduled_send_at` = время доставки + 60s, `send_attempts = 1`
  - Преждевременный `claim_due` не видит запись; через 60 секунд `claim_due` захватывает её, `send_attempts = 2`
  - Событие `mail.outbox.retry_scheduled` присутствует в `event_log`

> **Примечание:** файл `email_outbox.rs` обрезан. Оставшиеся тесты (например, достижение лимита попыток) не отражены в этом контекст-пакете.

### 7. Сетевое взаимодействие с провайдерами (provider networking)

- **`gmail_api_client_fetches_raw_messages_with_bearer_token`**
  - `GmailApiClient` делает два HTTP-запроса к API Gmail:
    1. список сообщений с параметрами `maxResults=2`, `includeSpamTrash=true`, `q=is:unread`
    2. raw-сообщение с `format=raw`
  - В заголовке `Authorization` передаётся `Bearer gmail-access-token`
  - Возвращаемый `EmailSyncBatch` содержит `provider_kind = Gmail`, `stream_id = "gmail:history"`, `checkpoint` с `history_id` и `next_page_token`

- **`imap_network_client_fetches_raw_messages_by_uid_without_mutating_mailbox`**
  - `ImapNetworkClient` подключается, выполняет `LOGIN`, `EXAMINE`, `UID SEARCH UID 43:*`, `UID FETCH 43 (UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)`
  - **Не выполняет** мутирующих команд: `SELECT`, `STORE`, `EXPUNGE`, `COPY`, `MOVE`, `DELETE`
  - `EmailSyncBatch` содержит `stream_id = "imap:Archive"`, `checkpoint` с `uid_validity` и `last_seen_uid`

- **`email_sync_records_provider_network_batch_against_postgres`**
  - `record_email_sync_batch` сохраняет raw-запись с проверяемым провайдером (`payload.provider = "gmail"`) и checkpoint

- **`email_sync_records_provider_batches_with_mail_blobs_against_postgres`**
  - `record_email_sync_batch_with_mail_blobs` для Gmail и IMAP:
    - `blobs_upserted = 1`
    - В `payload` удалены `raw_base64url` / `raw_rfc822_base64`, вместо них `raw_blob_id` с префиксом `blob:v1:sha256:` и `raw_blob_storage_kind = "local_fs"`

> **Примечание:** файл `email_provider_network.rs` обрезан. Оставшиеся тесты (например, проверка storage_path) не включены.

### 8. Парсинг RFC 822 (`email_rfc822.rs`)

- **`rfc822_parser_extracts_nested_multipart_attachments_for_current_basic_slice`**
  - `parse_rfc822_message` разбирает многокомпонентное письмо с вложенным `multipart/alternative`:
    - `body_text` извлечён из quoted-printable plain части
    - `body_html` содержит HTML
    - два вложения: PDF с `disposition = Attachment`, текстовый файл с `Inline`, корректные `body_bytes`

- **`rfc822_parser_preserves_html_links_for_rich_mail_rendering`**
  - HTML-ссылки с quoted-printable экранированием раскодируются корректно (например, `href="https://click.example.invalid/privacy?qs=abc"`)

- **`rfc822_parser_preserves_source_headers_with_folded_values`**
  - Свёрнутые заголовки (folding) объединяются: `X-Макошь-Trace: first line\n\tcontinued line` → `"first line continued line"`

- **`rfc822_parser_extracts_rfc2231_continued_attachment_filenames`**
  - Имена файлов, закодированные по RFC 2231 (`name*0*=…`, `name*1*=…`), корректно собираются с поддержкой Unicode (в т.ч. кириллицы)

- **`rfc822_parser_decodes_legacy_cyrillic_message_bytes`**
  - Письмо в кодировке `windows-1251` (кириллица, 8-битные байты) декодируется без заменяющих символов (`\u{fffd}`)

### 9. Планирование синхронизации (`email_sync.rs`)

- **`email_sync_plan_selects_provider_specific_credentials_and_streams_against_postgres`**
  - `plan_email_sync` выбирает правильный тип учётных данных и конфигурацию адаптера:
    - Gmail → `credential_purpose = OauthToken`, `stream_id = "gmail:history:primary"`, `adapter_config = EmailSyncAdapterConfig::Gmail`
    - iCloud → `credential_purpose = ImapPassword`, `stream_id = "imap:Archive"`, `adapter_config = Imap` с хостом из конфига
    - IMAP без mailbox → `stream_id = "imap:INBOX"` (по умолчанию)

- **`email_sync_plan_keeps_multiple_accounts_isolated_against_postgres`**
  - Два Gmail-аккаунта с разными `history_stream_id` получают независимые планы с разными `account_id` и `stream_id`

- **`email_sync_plan_rejects_invalid_imap_config`**
  - `plan_email_sync` возвращает `EmailSyncPlanError::InvalidProviderConfig` для:
    - пустого хоста (`" "`)
    - порта `0`
    - небулевого значения `tls`
    - имени ящика с переносом строки (`"Inbox\nArchive"`)

- **`email_sync_plan_rejects_secret_like_account_config`**
  - Ключи `oauth_token` и вложенный `adapter.oauth_token` в конфиге аккаунта вызывают `EmailSyncPlanError::SecretLikeConfigKey`

- **`email_sync_plan_uses_delimiter_safe_imap_stream_id`**
  - Почтовый ящик `"Projects:2026%Q2"` превращается в `stream_id = "imap:Projects%3A2026%25Q2"`, но в `adapter_config.mailbox` сохраняется исходное значение

### 10. Архитектурные ограничения (`email_account_setup_architecture.rs`)

- **`email_account_setup_tests_stay_below_architecture_line_limit`**
  - Проверяет, что все файлы в подкаталогах `email_account_setup` имеют не более 700 строк (нарушители перечисляются в сообщении об ошибке)

## Паттерны утверждений

- Все проверки секретов: `config.get("password").is_none()`, `config.get("access_token").is_none()` — ни один секрет не хранится открытым текстом в конфиге аккаунта.
- Каждое создание сущности сопровождается проверкой `observation_link` с правильными `domain`, `entity_kind`, `entity_id`, `relationship_kind` и `origin_kind`.
- События в `event_log` проверяются через `count(*)` и цепочки `causation_id`.
- Mock-серверы верифицируют отсутствие нежелательных сетевых вызовов (`assert!(commands.iter().all(...))`).
````

### Source coverage / Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `backend/tests/email_account_setup/gmail_service.rs` | Gmail OAuth PKCE URL, сохранение аккаунта/привязки/секрета, отсутствие секретов в конфиге, refresh токена с обновлением хранилища |
| `backend/tests/email_account_setup/imap_api.rs` (частично) | IMAP-настройка сервис-слоя, API `/api/v1/integrations/mail/accounts/imap` для iCloud: сохранение аккаунта, connected_services, SMTP-конфиг, signal_connection, привязки секретов, observations |
| `backend/tests/email_account_setup/send_api.rs` (частично) | Отправка через `/api/v1/communications/send`: outbox-очередь, observation_link, отсутствие прямого SMTP/Gmail-вызова, поведение без SMTP-привязки |
| `backend/tests/email_account_setup/support.rs` (частично) | Вспомогательные структуры `MockTokenServer`, `MockSmtpServer`, хелперы `live_setup_context`, `unlock_test_vault`, `json_request_with_token_and_actor` |
| `backend/tests/email_account_setup/vault_reconciliation.rs` | Восстановление iCloud-аккаунта, привязок секретов, календарного аккаунта из HostVault после полной очистки Postgres-метаданных; `origin_kind = "vault_source"` |
| `backend/tests/email_account_setup_architecture.rs` | Guard на 700 строк в файлах email_account_setup |
| `backend/tests/email_fixture_export.rs` | Экспорт фикстур: обезличивание полей `subject`, `from`, `to`, тела; поддержка multipart/quoted-printable |
| `backend/tests/email_fixture_pipeline.rs` | Пайплайн `project_fixture_email_messages`: отчёт о импорте/проекции, события в `event_log`, причинно-следственная цепочка из 4 событий |
| `backend/tests/email_import.rs` (частично) | Парсинг `parse_fixture_email_messages`, идемпотентность импорта, коллизии `provider_record_id` с `:`, обработка отсутствующего `sent_at` |
| `backend/tests/email_outbox.rs` (частично) | Outbox: `claim_due` с учётом `undo_deadline_at`, delivered → `Sent` + event, failed → `Failed` + event, retry с backoff и отложенной `scheduled_send_at`, observation_links |
| `backend/tests/email_provider_network.rs` (частично) | Gmail API client (параметры, Bearer token, checkpoint), IMAP client (не мутирующие команды), `record_email_sync_batch`, `record_email_sync_batch_with_mail_blobs` |
| `backend/tests/email_rfc822.rs` | Парсер RFC 822: вложения, HTML-ссылки, folded-заголовки, RFC 2231 continued filenames, `windows-1251` legacy |
| `backend/tests/email_sync.rs` | План синхронизации: выбор credentials/adapter, изоляция аккаунтов, валидация IMAP-конфига, запрет secret-like ключей, delimiter-safe stream_id |

### Drift candidates / Кандидаты на drift

- Файлы `imap_api.rs`, `send_api.rs`, `email_import.rs`, `email_outbox.rs`, `email_provider_network.rs` были обрезаны по лимиту символов. Полное покрытие оставшихся тестов в этих файлах не может быть подтверждено из данного контекста.
- Иных несоответствий между кодом, документацией или ADR в предоставленном контексте не обнаружено.
