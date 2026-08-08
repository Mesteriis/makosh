## Summary / Резюме

Создать/обновить страницу `operations/backend-tests.md` в русской Obsidian‑wiki, документирующую интеграционные backend‑тесты из репозитория `makosh`. На основе предоставленных исходных файлов описаны структура тестовых наборов, ключевые проверяемые сценарии и используемая тестовая инфраструктура. Страница охватывает тесты API противоречий, движка решений, хранилища решений, API решений, обработки документов, API обработки документов, импорта документов, управления почтовыми аккаунтами и настройки почтовых аккаунтов.

## Proposed pages / Предлагаемые страницы

### `operations/backend-tests.md`

```markdown
# Тесты backend

## Обзор

Backend‑тесты в `makosh` — это интеграционные тесты, работающие с временной базой PostgreSQL через `testkit::context::TestContext`. Тесты проверяют бизнес‑логику через HTTP‑эндпоинты (используя `axum::Router`) и прямые вызовы хранилищ (`Store`), включая запись в таблицы БД, создание наблюдений (`observation_links`, `observations`, `observation_kind_definitions`), элементов ревью (`review_items`), проекции графа (`graph_nodes`, `graph_edges`, `graph_evidence`) и аудит‑логи (`api_audit_log`).

Все тесты изолированы: каждый генерирует уникальный суффикс через наносекунды `SystemTime`, чтобы избежать коллизий между параллельными запусками.

## Тестовая инфраструктура

- **`TestContext::new().await`** — создаёт временную PostgreSQL и возвращает строку подключения.
- **`app_and_pool(database_url)`** / **`live_context(test_name)`** — создают `PgPool` и экземпляр `axum::Router` с маршрутами `build_router_with_database`, сконфигурированный с тестовым секретом (`LOCAL_API_TOKEN`).
- **`unique_suffix() -> u128`** — уникальный суффикс на основе `SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()`.
- **`get_request_with_token(uri, token)`** — `GET`‑запрос с заголовком `x-makosh-secret`.
- **`json_put_request(uri, value, token)`** — `PUT`‑запрос с JSON‑телом и заголовками `Content-Type: application/json` и `x-makosh-secret`.
- **`json_body(response) -> Value`** — десериализация тела ответа в `serde_json::Value`.
- **`path_segment(value)`** — URL‑кодирование идентификатора для подстановки в путь запроса.
- **`quiesce_*`** — вспомогательные функции для «заморозки» состояний обработки документов (перевод в `skipped`), чтобы изолировать тестируемые задания.
- **`fail_processing_job`**, **`create_failed_extract_text_job`** — ручная установка задания обработки документов в состояние `failed` для тестов повторных попыток.
- **`append_retry_event_for_job`** — запись события `document_processing.retry_requested` через `EventStore`, используемая для проверки идемпотентности и коллизий команд повтора.
- **`step_name`** — отображение варианта `DocumentProcessingStep` в строку (`"extract_text"` / `"ocr"`).
- **`disconnected_document_store`** — создаёт `DocumentImportStore` с ленивым соединением, используемый для unit‑тестов валидации полей без реальной БД.

## Тестовые наборы

### 1. Противоречия (contradictions_api.rs)

Файл: `backend/tests/contradictions_api.rs`

Тесты проверяют API‑маршруты для работы с наблюдениями противоречий.

- **`contradictions_list_returns_open_reviewable_observations`**
  - Создаёт наблюдение противоречия через `ContradictionObservationStore::upsert`.
  - Выполняет `GET /api/v1/contradictions?limit=10`.
  - Проверяет, что ответ содержит `items`, среди которых есть созданное наблюдение.
  - Утверждает поля: `conflict_type = "direct_contradiction"`, `old_claim`, `new_claim`, `review_state = "suggested"`.
  - Проверяет, что в таблице `review_items` создана запись с `item_kind = "contradiction_candidate"`, `status = "new"` и `metadata->>'contradiction_observation_id'`, равным `observation_id`.
  - Проверяет наличие материализованной связи (`observation_links`) типа `upsert` от наблюдения противоречия к основной записи `observations`; в `observation_links.metadata` присутствует `conflict_type: "direct_contradiction"`, а связанное наблюдение имеет `kind.code = "CONTRADICTION_OBSERVATION"`.
  - Функция `sync_contradiction_review_item` вызывается при заполнении тестовых данных.

- **`put_contradiction_review_updates_review_state_with_observation_trail`**
  - Создаёт наблюдение противоречия.
  - Выполняет `PUT /api/v1/contradictions/{observation_id}/review` с JSON `{ "review_state": "user_confirmed", "resolution": "confirmed from source review" }`.
  - Ожидает `200 OK` и тело ответа с `observation_id`, `review_state = "user_confirmed"`, `reviewed_by = "makosh-frontend"`, `resolution`.
  - Проверяет, что в `contradiction_observations` обновлены `review_state` и `resolution`.
  - Проверяет, что количество записей в `person_facts` с `value` равным `new_claim` остаётся равным 0 (перезапись памяти не происходит).
  - Проверяет создание записи‑наблюдения `review_transition` в `observation_links` для домена `consistency`, `entity_kind = 'contradiction_observation'`.
    - `metadata` ссылки содержит `review_state` и `resolution`.
    - Связанное наблюдение (`observations`) имеет `origin_kind = "manual"`, `payload.contradiction_observation_id` и `payload.review_state`.
  - Проверяет, что элемент ревью (`review_items`) обновлён: `status = "approved"`.

### 2. Движок решений (decision_engine.rs)

Файл: `backend/tests/decision_engine.rs`

Unit‑тесты для `DecisionEngine::detect_candidates`.

- **`decision_engine_detects_explicit_communication_decision_candidate`**
  - Создаёт `DecisionExtractionInput::communication` с текстом `"Decision: Use local-first storage because private context must work offline."`.
  - Ожидает ровно один обнаруженный кандидат (`result.decisions.len() == 1`).
  - Проверяет поля кандидата:
    - `kind = DecisionCandidateKind::ExplicitDecision`
    - `title = "Use local-first storage"`
    - `rationale = "private context must work offline"`
    - `quote` равен исходному предложению
    - `decided_by_entity_kind = Some(DecisionEntityKind::Persona)`
    - `decided_by_entity_id = Some("person:v1:email:owner@example.com")`
    - `confidence = 0.83`
    - `review_state = DecisionReviewState::Suggested`
    - `evidence_source_kind = DecisionEvidenceSourceKind::Communication`
    - `evidence_source_id = "message:decision-engine"`
    - один элемент в `impacted_entities` с `entity_kind = DecisionEntityKind::Project` и `entity_id = "project:v1:makosh"`
  - Вызывает `to_decision_draft()` и проверяет возвращаемые черновики решения, доказательства и затронутые сущности (включая `impact_type = "decision_context"`, `metadata` с `engine: "decision"`).

- **`decision_engine_ignores_non_decision_evidence`**
  - Подаёт текст без явного решения: `"The team discussed storage options but no decision was made."`.
  - Ожидает пустой список кандидатов (`result.decisions.is_empty()`).

- **`decision_engine_rejects_empty_source_evidence_before_detection`**
  - Подаёт текст, состоящий из пробелов.
  - Ожидает ошибку `DecisionEngineError::EmptyField("text")`.

### 3. Хранилище решений (decisions.rs)

Файл: `backend/tests/decisions.rs` (полный файл обрезан, доступно 12000 символов).

Тесты используют `DecisionStore` с реальной PostgreSQL.

- **`decision_store_upserts_evidence_backed_decision_without_creating_work_against_postgres`**
  - Создаёт решение с `decided_by`, `alternatives`, `metadata`.
  - Выполняет два последовательных `upsert_with_evidence` с одинаковым `NewDecision`, но разными `NewDecisionEvidence` (разные цитаты, `confidence` 0.92 и 0.94).
  - Проверяет идемпотентность: `decision_id` одинаковый.
  - Проверяет сохранённые поля решения: `title`, `rationale`, `status = "active"`, `review_state = "user_confirmed"`, `confidence = 0.9`, `decided_by_entity_kind` и `decided_by_entity_id`.
  - В `decision_evidence` хранится последняя цитата (`"Updated decision evidence for local-first dossier storage."`), `confidence = 0.94`.
  - В `decision_impacted_entities` хранится запись с `impact_type = "architecture_direction"`, `metadata = { "component": "dossier" }`.
  - `list_for_entity` находит решение по `project_id`.
  - Запускает `GraphProjectionService::project_from_v1` и проверяет созданные узлы и ребра:
    - `graph_nodes` содержат узел `decision` и узел `project`.
    - `graph_edges` с `relationship_type = "entity_relationship"`, `confidence = 0.9`, `review_state = "user_confirmed"`, свойствами `{ domain: "decision", decision_id, impact_type }`.
    - `graph_evidence` с `source_kind = "decision"`, `source_id = decision_id`, excerpt из последнего доказательства, `metadata` с `domain: "decision"`, `source_kind: "event"`, `source_id: evidence_source_id`.
  - Проверяет, что не созданы задачи (`tasks`) и обязательства (`obligations`) для данного `source_id`.

- **`decision_store_refresh_persists_explicit_message_decision_candidate_against_postgres`**
  - Создаёт сообщение‑источник (seed_decision_message), содержащее фразу вида `"Decision: {title} because {rationale}."`.
  - Вызывает `refresh_deterministic_candidates(100)` — количество обновлённых кандидатов >= 1.
  - Проверяет, что среди решений для `entity_kind = "Communication"` и `message_id` присутствует решение с ожидаемым `title`, `rationale`, `review_state = "Suggested"`, `confidence = 0.83`.
  - В `decision_evidence` для этого решения `source_kind = "communication"`, `source_id = message_id`, `quote` равен исходной строке.
  - (Далее файл обрезан; полная проверка `message_observation_id` не видна в данном контексте.)

### 4. API решений (decisions_api.rs)

Файл: `backend/tests/decisions_api.rs`

- **`decisions_list_returns_entity_scoped_decisions`**
  - Создаёт решение для конкретного проекта.
  - `GET /api/v1/decisions?entity_kind=project&entity_id={project_id}&limit=10`.
  - Находит созданное решение в `items`, проверяет `title`, `status = "active"`, `review_state = "suggested"`, `decided_by_entity_kind = "persona"`.

- **`decisions_list_returns_global_suggested_review_items`**
  - Создаёт два решения: одно `Suggested`, другое `UserConfirmed`.
  - `GET /api/v1/decisions?review_state=suggested&limit=10`.
  - Утверждает, что в ответе присутствует только `Suggested` решение, и все элементы имеют `review_state = "suggested"`.

- **`put_decision_review_updates_review_state_with_observation_trail`**
  - Создаёт решение со статусом `Suggested`.
  - `PUT /api/v1/decisions/{decision_id}/review` с `{ "review_state": "user_confirmed" }`.
  - Проверяет `200 OK`, тело ответа содержит `decision_id` и `review_state = "user_confirmed"`.
  - Проверяет обновление `review_state` в таблице `decisions`.
  - Проверяет создание `observation_links` для домена `decisions`, `entity_kind = 'decision'`, `relationship_kind = 'review_transition'`, с `metadata.review_state = "user_confirmed"`.
  - Связанное наблюдение (`observations`) имеет `origin_kind = "manual"`, `payload.decision_id` и `payload.review_state`.
  - Проверяет, что `review_items` обновлён: `status = "promoted"`, `target_entity_kind = "decision"`.
  - Утверждает, что `tasks` и `obligations` не созданы (count = 0).

### 5. Обработка документов (document_processing/)

Файлы: `document_processing/enqueue_run.rs`, `document_processing/retry.rs`, `document_processing/support.rs`.

**Постановка в очередь и выполнение (enqueue_run.rs)**

- **`enqueue_for_document_creates_extract_text_and_ocr_jobs`**
  - Импортирует Markdown‑документ.
  - Вызывает `enqueue_for_document` — возвращает два задания: `extract_text` и `ocr`.
  - Проверяет наличие observation‑связей для каждого задания (в `observation_links`, domain `documents`, entity_kind `document_processing_job`).

- **`enqueue_for_document_does_not_reset_terminal_jobs`**
  - Импортирует PDF, ставит задания в очередь, запускает `run_queued_jobs` (оба задания пропускаются со статусом `skipped`, так как нет реализации OCR/extract_text для PDF).
  - Повторно вызывает `enqueue_for_document` и проверяет, что терминальное состояние заданий не изменилось (защита от сброса).

- **`run_queued_jobs_for_markdown_populates_extracted_text_artifact`**
  - Импортирует Markdown, ставит в очередь, выполняет `run_queued_jobs`.
  - Проверяет, что `extract_text` задание перешло в `succeeded`.
  - В таблице `document_artifacts` появляется одна запись с `artifact_kind = 'extracted_text'`.
  - Проверяет, что создано не менее двух наблюдений `DOCUMENT_PROCESSING_JOB_STATUS` для этого задания.

- **`run_queued_jobs_skips_non_markdown_text_extraction_with_summary`**
  - Импортирует PDF, выполняет `run_queued_jobs`.
  - Задание `extract_text` имеет непустой `last_error_summary`, указывающий на пропуск обработки.

**Повторные попытки (retry.rs)**

- **`document_processing_retry_failed_job_requeues_job_against_postgres`**
  - Создаёт документ, ставит задания в очередь, вручную помечает `extract_text` задание как `failed` (2 попытки, `last_error_summary = "temporary extractor failure"`).
  - Вызывает `retry_failed_job` с уникальным `command_id`.
  - Результат: `job_id`, `status = "queued"`, `event_id = "document_processing_retry:{command_id}"`.
  - В БД задание переведено в `queued`, `attempts` сброшены в 0, `last_error_summary` очищен.
  - Создана observation‑связь `requeued` с `kind.code = "DOCUMENT_PROCESSING_JOB_STATUS"`.

- **`run_queued_jobs_requires_retry_command_for_failed_jobs`**
  - Создаёт упавшее задание, «замораживает» остальные.
  - `run_queued_jobs` без команды повтора не видит заданий: `jobs_seen = 0`, `jobs_queued = 0`.
  - Состояние задания остаётся `failed` (попытки = 2, артефакт `extracted_text` отсутствует).
  - После `retry_failed_job` повторный `run_queued_jobs` выполняет задание: `jobs_seen = 1`, `jobs_queued = 1`, `jobs_succeeded = 1`.
  - Задание переходит в `succeeded`, артефакт создаётся.

- **`document_processing_retry_duplicate_same_command_is_idempotent`**
  - Два вызова `retry_failed_job` с одинаковым `DocumentProcessingRetryCommand` (один и тот же `command_id`) возвращают одинаковый результат, задание остаётся в `queued`.

- **`document_processing_retry_duplicate_command_for_different_job_is_rejected`**
  - Регистрирует событие повтора для одного задания, затем пытается использовать тот же `command_id` для другого задания.
  - Ожидается `DocumentProcessingError::RetryCommandConflict`, состояние целевого задания не меняется.

- **`document_processing_retry_non_failed_job_requires_failed_status`**
  - Попытка повтора для задания в статусе `queued` возвращает `DocumentProcessingError::RetryRequiresFailedJob`.

- **`document_processing_retry_missing_job_returns_job_not_found`**
  - Попытка повтора для несуществующего `job_id` возвращает `DocumentProcessingError::JobNotFound`.

### 6. API обработки документов (document_processing_api.rs)

Файл: `backend/tests/document_processing_api.rs` (полный файл обрезан).

- **`get_document_processing_jobs_rejects_missing_local_api_secret`**
  - Запрос без секрета возвращает `403 FORBIDDEN`, тело: `{ "error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header" }`.

- **`get_document_processing_for_missing_document_returns_404`**
  - Запрос для несуществующего `document_id` возвращает `404 NOT_FOUND`, тело: `{ "error": "document_processing_store_error", "message": "document processing job was not found" }`.

- **`document_processing_api_returns_expected_payloads`**
  - `GET /api/v1/document-processing/jobs?limit=10` возвращает список заданий, включая только что созданный документ.
  - `GET /api/v1/documents/{document_id}/processing` возвращает детали обработки: `document_id`, массив `jobs` (не менее 2 элементов).

- **`post_document_processing_job_retry_requeues_failed_job`**
  - Создаёт документ, вручную делает задание `extract_text` проваленным.
  - `POST /api/v1/document-processing/jobs/{job_id}/retry` с `{ "command_id": "..." }`.
  - Ответ `200 OK`: `job_id`, `status = "queued"`, `event_id = "document_processing_retry:{command_id}"`.
  - Проверяет запись в `api_audit_log`: `operation = "document_processing.job.retry"`, `actor_id = "makosh-frontend"`, `method = "POST"`, `path_template = "/api/v1/document-processing/jobs/{job_id}/retry"`, `target_kind = "document_processing_job"`, `target_id = job_id`.
  - Проверяет observation‑связь `retry_command` с `kind.code = "DOCUMENT_PROCESSING_JOB_STATUS"`.

- **`post_document_processing_job_retry_rejects_non_failed_job_with_stable_body`**
  - Попытка повтора для задания в статусе `queued` возвращает `400 BAD_REQUEST`, тело содержит `"error": "document_processing_store_error"` (остаток сообщения обрезан в предоставленном контексте).

### 7. Архитектурный тест обработки документов (document_processing_architecture.rs)

Файл: `backend/tests/document_processing_architecture.rs`

- **`document_processing_tests_stay_below_architecture_line_limit`**
  - Проверяет, что ни один файл тестов обработки документов (включая подкаталог `document_processing/`) не превышает 700 строк.
  - Использует `fs::read_dir` и подсчёт строк, падает с перечнем нарушителей, если лимит превышен.

### 8. Импорт документов (documents.rs)

Файл: `backend/tests/documents.rs`

- **`document_import_stores_markdown_text_against_postgres`**
  - Импортирует Markdown‑документ с текстом `"# Notes\n\nBudget review notes."`.
  - Проверяет `document_kind = "markdown"`, `title = "notes.md"`, `extracted_text = "Notes\n\nBudget review notes."` (заголовки очищены от `#`, но сохранены в тексте).
  - Проверяет ровно одну observation‑связь с `relationship_kind = "import"`, `domain = "documents"`, `entity_kind = "document"`.

- **`document_import_stores_pdf_metadata_against_postgres`**
  - Импортирует PDF с `source_fingerprint = "sha256:contract"`.
  - Проверяет `document_kind = "pdf"`, `title = "contract.pdf"`, `extracted_text = ""`, `source_fingerprint = "sha256:contract"`.

- **`document_import_rejects_blank_required_fields`**
  - Для каждого обязательного поля (`document_id`, `document_kind`, `title`, `source_fingerprint`, `extracted_text`) создаёт документ с пробельным значением.
  - Ожидает `DocumentImportError::EmptyField` с именем поля.

- **`document_import_rejects_invalid_kind`**
  - Импорт с `document_kind = "docx"` возвращает `DocumentImportError::InvalidDocumentKind("docx")`.

- **`markdown_import_helper_derives_deterministic_local_fingerprint`**
  - Два вызова `NewDocumentImport::markdown` с одинаковыми параметрами дают одинаковый `source_fingerprint`, начинающийся с `"local-v1:markdown:"`.
  - `extracted_text` равен `"Notes\n\nBody."`.

- **`document_import_extracts_multiple_markdown_heading_levels_against_postgres`**
  - Текст с заголовками `#`, `##`, `###` извлекается с удалением маркеров, оставляя только текст заголовков и тело.

- **`document_import_preserves_hash_prefixed_non_headings_against_postgres`**
  - Строки вида `#hashtag`, `#include <x>`, `###not heading`, `####### Too many hashes` сохраняются как есть, без удаления `#`.

- **`document_import_reimports_same_kind_idempotently_against_postgres`**
  - Повторный импорт Markdown с тем же `document_id`, но другим контентом, обновляет `title`, `source_fingerprint` и `extracted_text`, сохраняя исходный `imported_at`.
  - В таблице `documents` остаётся ровно одна запись.

- **`document_import_rejects_existing_document_kind_change_against_postgres`**
  - Попытка импорта PDF для существующего Markdown‑документа возвращает `DocumentImportError::DocumentKindChange`.
  - Исходные данные документа в БД не изменяются.

### 9. Управление почтовыми аккаунтами (email_account_management_api.rs)

Файл: `backend/tests/email_account_management_api.rs` (полный файл обрезан).

- **`email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account`**
  - Создаёт IMAP‑аккаунт `fastmail-primary`, привязывает секрет (`imap_password`), восстанавливает источники сигналов (`SignalHubStore::restore_system_sources`).
  - `GET /api/v1/integrations/mail/accounts` — возвращает список из одного аккаунта с `account_id = "fastmail-primary"`, `capabilities.send = true`, `capabilities.local_trash = true`.
  - `GET /api/v1/integrations/mail/accounts/fastmail-primary` — `external_account_id = "alex@example.com"`.
  - `GET /api/v1/integrations/mail/accounts/fastmail-primary/export` — возвращает аккаунт без полей `password`, `secret_ref`, `token` (секреты не экспортируются).
  - `POST /api/v1/integrations/mail/accounts/fastmail-primary/logout` — `auth_state` становится `"logged_out"`, `sync_enabled = false`.
    - Статус `signal_connections` для `source_code = 'mail'` и `account_id = 'fastmail-primary'` меняется на `"disconnected"`.
    - Создаётся observation‑запись `config_update` с `origin_kind = "local_runtime"`, `kind_code = "COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION"`, `payload.action = "logout"`.
  - `DELETE /api/v1/integrations/mail/accounts/fastmail-primary` — `deleted = true`, разорванные секреты: `["secret:fastmail-primary:imap-password"]`.
    - Статус `signal_connections` меняется на `"removed"`.
    - Создаются observations: `delete` для `COMMUNICATION_PROVIDER_ACCOUNT_DELETED`, `remove` для `COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED` (обе с `origin_kind = "local_runtime"`).
  - Повторный `GET` удалённого аккаунта возвращает `404 NOT FOUND`.

- **`email_account_delete_rejects_accounts_with_retained_raw_records`**
  - Создаёт аккаунт и записывает сырую запись (`raw:mail-account-delete`).
  - `DELETE` возвращает `409 CONFLICT` (далее файл обрезан; детали ошибки не видны в данном контексте).

### 10. Настройка почтовых аккаунтов (email_account_setup/)

Файлы: `email_account_setup/config.rs`, `email_account_setup/gmail_api.rs` (gmail_api.rs обрезан).

**Конфигурация OAuth (config.rs)**

- **`gmail_oauth_setup_defaults_to_mail_send_calendar_and_contacts_scopes`**
  - `GmailOAuthSetupRequest::new` по умолчанию включает scopes: `gmail.readonly`, `gmail.send`, `calendar.readonly`, `contacts.readonly`.

- **`app_config_accepts_google_oauth_client_credentials`**
  - `AppConfig::from_pairs` с `MAKOSH_GOOGLE_OAUTH_CLIENT_ID` и `MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET` корректно сохраняет и возвращает эти значения (секрет оборачивается в `SecretString`).

- **`app_config_accepts_google_oauth_installed_client_json`**
  - `AppConfig` парсит JSON с `"installed": { client_id, client_secret, auth_uri, token_uri, redirect_uris }`.
  - `google_oauth_client()` возвращает `GoogleOAuthClientType::Installed` с ожидаемыми значениями.

**Gmail OAuth API (gmail_api.rs, частично обрезан)**

- **`gmail_oauth_start_api_uses_configured_google_desktop_client_against_postgres`**
  - Создаёт хранилище (`HostVault`), разблокирует (`unlock_test_vault`).
  - `POST /api/v1/integrations/mail/accounts/gmail/oauth/start` с `account_id`, `display_name`, `redirect_uri`.
  - Ответ `200 OK` содержит `authorization_url`, начинающийся с `https://accounts.google.com/o/oauth2/auth?`, включающий `client_id=desktop-client-id.apps.googleusercontent.com` и все требуемые scopes.

- **`gmail_oauth_start_api_requires_initialized_host_vault_against_postgres`**
  - Если хранилище не инициализировано, возвращается `503 SERVICE UNAVAILABLE` с `error: "host_vault_error"`, `message: "host vault is not initialized"`.

- **`gmail_oauth_start_api_reopens_initialized_host_vault_after_restart_against_postgres`**
  - После инициализации хранилища и перезапуска приложения (новый роутер) OAuth‑старт успешен (`200 OK`).

- **`gmail_oauth_callback_completes_pending_grant_without_api_secret`**
  - Запускает mock‑сервер токенов (`MockTokenServer`).
  - Начинает OAuth (`start_response`), получает `state`.
  - Переходит по `callback?code=...&state=...`.
  - Ответ `200 OK`, тело содержит HTML/текст с `"Google"` (файл обрезан, полный текст проверки не виден).

## Примечания

- Для тестов, требующих реального выполнения заданий обработки документов, используются вспомогательные функции `quiesce_*`, которые массово переводят «лишние» задания в статус `skipped`, чтобы изолировать тестируемое задание.
- Контекст безопасности: тестовый секрет (`LOCAL_API_TOKEN`) используется для аутентификации локального API, в production‑среде должен заменяться реальным секретом.
- Тест `document_processing_architecture.rs` накладывает архитектурное ограничение на длину файлов (до 700 строк), что не связано с бизнес‑логикой, а является правилом поддерживаемости.
- Для файлов, обрезанных в предоставленном контексте (`decisions.rs`, `document_processing_api.rs`, `email_account_management_api.rs`, `email_account_setup/gmail_api.rs`), доступна только часть тестовых сценариев; полный охват не подтверждён.
```

## Source coverage / Покрытие источников

| Исходный файл | Факты, покрытые в предложенной странице |
|---|---|
| `backend/tests/contradictions_api.rs` | Тесты `contradictions_list_returns_open_reviewable_observations` и `put_contradiction_review_updates_review_state_with_observation_trail`: проверка эндпоинтов, полей ответа, состояния `review_items`, `observation_links`, `contradiction_observations`, `person_facts`. |
| `backend/tests/decision_engine.rs` | Тесты `decision_engine_detects_explicit_communication_decision_candidate`, `decision_engine_ignores_non_decision_evidence`, `decision_engine_rejects_empty_source_evidence_before_detection`: проверка `DecisionEngine::detect_candidates`, полей кандидатов, `DecisionEngineError::EmptyField`. |
| `backend/tests/decisions.rs` (частично) | Тесты `decision_store_upserts_evidence_backed_decision_without_creating_work_against_postgres` (полный) и `decision_store_refresh_persists_explicit_message_decision_candidate_against_postgres` (начало): проверка `upsert_with_evidence`, `refresh_deterministic_candidates`, полей решения, `decision_evidence`, `decision_impacted_entities`, графовой проекции, отсутствия задач и обязательств. |
| `backend/tests/decisions_api.rs` | Тесты `decisions_list_returns_entity_scoped_decisions`, `decisions_list_returns_global_suggested_review_items`, `put_decision_review_updates_review_state_with_observation_trail`: проверка эндпоинтов, полей ответа, `review_items`, `observation_links`, `tasks`/`obligations` count. |
| `backend/tests/document_processing/enqueue_run.rs` | Тесты `enqueue_for_document_creates_extract_text_and_ocr_jobs`, `enqueue_for_document_does_not_reset_terminal_jobs`, `run_queued_jobs_for_markdown_populates_extracted_text_artifact`, `run_queued_jobs_skips_non_markdown_text_extraction_with_summary`: проверка очередей, статусов, артефактов, observation‑связей. |
| `backend/tests/document_processing/retry.rs` | Тесты повторов: `document_processing_retry_failed_job_requeues_job_against_postgres`, `run_queued_jobs_requires_retry_command_for_failed_jobs`, идемпотентность, коллизии команд, отклонение не‑failed заданий, несуществующих заданий. |
| `backend/tests/document_processing/support.rs` | Вспомогательные функции: `live_context`, `unique_suffix`, `step_name`, `create_failed_extract_text_job`, `fail_processing_job`, `append_retry_event_for_job`, `job_retry_state`, `quiesce_*` — описаны в разделе «Тестовая инфраструктура». |
| `backend/tests/document_processing_api.rs` (частично) | Тесты `get_document_processing_jobs_rejects_missing_local_api_secret`, `get_document_processing_for_missing_document_returns_404`, `document_processing_api_returns_expected_payloads`, `post_document_processing_job_retry_requeues_failed_job`, `post_document_processing_job_retry_rejects_non_failed_job_with_stable_body` (начало): проверка статусов, тел ошибок, `api_audit_log`, observation‑связей. |
| `backend/tests/document_processing_architecture.rs` | Тест `document_processing_tests_stay_below_architecture_line_limit`: ограничение в 700 строк на файлы тестов обработки документов. |
| `backend/tests/documents.rs` | Тесты `document_import_stores_markdown_text_against_postgres`, `document_import_stores_pdf_metadata_against_postgres`, `document_import_rejects_blank_required_fields`, `document_import_rejects_invalid_kind`, `markdown_import_helper_derives_deterministic_local_fingerprint`, тесты извлечения заголовков, идемпотентности, запрета смены kind. |
| `backend/tests/email_account_management_api.rs` (частично) | Тесты `email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account` (полный) и `email_account_delete_rejects_accounts_with_retained_raw_records` (начало): проверка CRUD, логаута, удаления, observation‑записей, статусов сигналов, экспорта без секретов. |
| `backend/tests/email_account_setup/config.rs` | Тесты `gmail_oauth_setup_defaults_to_mail_send_calendar_and_contacts_scopes`, `app_config_accepts_google_oauth_client_credentials`, `app_config_accepts_google_oauth_installed_client_json`: проверка scopes, конфигурации Google OAuth. |
| `backend/tests/email_account_setup/gmail_api.rs` (частично) | Тесты `gmail_oauth_start_api_uses_configured_google_desktop_client_against_postgres`, `gmail_oauth_start_api_requires_initialized_host_vault_against_postgres`, `gmail_oauth_start_api_reopens_initialized_host_vault_after_restart_against_postgres`, `gmail_oauth_callback_completes_pending_grant_without_api_secret` (начало): проверка инициализации хранилища, успешного старта OAuth, callback. |
| `backend/tests/document_processing.rs` | Только декларации подмодулей; покрыто структурой страницы. |
| `backend/tests/email_account_setup.rs` | Только декларации подмодулей; покрыто структурой страницы. |

## Drift candidates / Кандидаты на drift

На основе предоставленного контекста расхождения между кодом и документацией не выявлены. Однако:

- Часть файлов (`decisions.rs`, `document_processing_api.rs`, `email_account_management_api.rs`, `email_account_setup/gmail_api.rs`) обрезана, поэтому полный охват тестовых утверждений этих файлов не может быть подтверждён. Возможен drift в непокрытых тестовых сценариях.
- Тест `decision_store_refresh_persists_explicit_message_decision_candidate_against_postgres` оборван на проверке `message_observation_id`; продолжение может содержать дополнительные проверки, не учтённые в wiki.
- Тест `post_document_processing_job_retry_rejects_non_failed_job_with_stable_body` оборван на теле ошибки; точное сообщение может отличаться от зафиксированного в wiki.
- Тест `email_account_delete_rejects_accounts_with_retained_raw_records` оборван на коде ошибки; дальнейшие детали ответа отсутствуют.
- Тест `gmail_oauth_callback_completes_pending_grant_without_api_secret` оборван на проверке тела ответа; полный набор проверок неизвестен.
