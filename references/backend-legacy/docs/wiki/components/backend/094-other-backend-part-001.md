---
chunk_id: 094-other-backend-part-001
batch_id: batch-20260628T214902
group: backend
role: other
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 094-other-backend-part-001 — backend/other

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/backend.md` должна быть создана или обновлена, чтобы отразить текущую схему базы данных Макошь, определённую в миграциях с 0001 по 0025. Предыдущая версия (если существовала) могла не включать последние изменения: поддержку Telegram‑звонков, WhatsApp Web, шифрованное хранилище секретов в БД, таблицу прикладных настроек, состояние рабочего процесса сообщений и AI‑рантайм с pgvector. Новая страница даёт русскоязычный обзор всех основных таблиц, их ограничений, индексов и назначения.

## Предложенные страницы

`components/backend.md`

---

```markdown
# Backend

Backend Макошь работает с PostgreSQL и предоставляет API для фронтенда. Схема базы строится последовательными миграциями и реализует следующие подсистемы.

## Event Sourcing

### `event_log`
Основное хранилище событий (append‑only). Каждое событие содержит:

- `event_id` (PK), `event_type`, `schema_version`, `occurred_at`, `recorded_at`
- `source` (JSONB) – источник события (всегда объект)
- `actor` (JSONB, nullable, объект)
- `subject` (JSONB, объект)
- `payload` (JSONB, по умолчанию `{}`)
- `provenance` (JSONB, по умолчанию `{}`)
- `causation_id`, `correlation_id`
- `position` – BIGINT, генерируется автоматически, уникальный

Ограничения:
- `event_id`, `event_type` непустые
- `schema_version > 0`
- `source`, `actor`, `subject`, `payload`, `provenance` – объекты JSONB

Индексы:
- `recorded_at, position`
- `occurred_at, position`
- `event_type, recorded_at`
- `correlation_id` (частичный)
- Уникальный индекс идемпотентности: `(event_type, source->>'kind', COALESCE(source->>'provider',''), source->>'source_id')` при наличии `source->>'source_id'`

Триггеры `prevent_event_log_mutation` запрещают UPDATE и DELETE.

### `projection_cursors`
Отслеживает последнюю обработанную позицию для проекций:
- `projection_name` (PK), `last_processed_position` (BIGINT ≥0), `updated_at`
- Индекс по `updated_at`

## API Audit

### `api_audit_log`
Append‑only журнал операций API:
- `audit_id` (PK, BIGINT GENERATED ALWAYS AS IDENTITY)
- `recorded_at`, `actor_kind`, `operation`, `method`, `path_template`
- `target_kind`, `target_id` (nullable), `metadata` (JSONB)
- Позднее добавлен `actor_id` (nullable, непустой если задан)

Ограничения: все обязательные текстовые поля непустые, `metadata` – объект.
Индексы: `recorded_at`, `operation, recorded_at`, `target_kind, target_id, recorded_at` (частичный), `actor_kind, actor_id, recorded_at` (частичный).
Триггеры `prevent_api_audit_log_mutation` запрещают UPDATE и DELETE.

## Приём коммуникаций

### Аккаунты и сырые записи

#### `communication_provider_accounts`
- `account_id` (PK), `provider_kind`, `display_name`, `external_account_id`, `config` (JSONB)
- `provider_kind` допустимые значения: `gmail`, `icloud`, `imap`, `telegram_user`, `telegram_bot`, `whatsapp_web` (расширялось миграциями)
- Уникальность: `(provider_kind, external_account_id)`

#### `communication_raw_records`
Append‑only сырые записи от провайдеров:
- `raw_record_id` (PK), `account_id` (FK), `record_kind`, `provider_record_id`, `source_fingerprint`, `import_batch_id`
- `occurred_at` (nullable), `captured_at`, `payload` (JSONB), `provenance` (JSONB)
- Уникальность: `(account_id, record_kind, provider_record_id)`
- Триггеры запрещают UPDATE и DELETE

#### `communication_ingestion_checkpoints`
Контрольные точки получения для пары `(account_id, stream_id)`:
- PK: `(account_id, stream_id)`, `checkpoint` (JSONB), `updated_at`

### Сообщения

#### `communication_messages`
Спроецированные сообщения:
- `message_id` (PK), `raw_record_id` (FK), `account_id` (FK), `provider_record_id`
- `subject`, `sender`, `recipients` (JSONB массив), `body_text`
- `occurred_at` (nullable), `projected_at`
- Уникальность: `(account_id, provider_record_id)`
- Позднее добавлены:
  - `channel_kind` (`email`, `telegram_user`, `telegram_bot`, `whatsapp_web`)
  - `conversation_id`, `sender_display_name`
  - `delivery_state` (`received`, `sent`, `send_dry_run`, `send_blocked`)
  - `message_metadata` (JSONB)
  - `workflow_state` (`new`, `reviewed`, `needs_action`, `waiting`, `done`, `archived`, `muted`, `spam`), по умолчанию `new`
  - `importance_score` (SMALLINT 0-100), `ai_category`, `ai_summary`, `ai_summary_generated_at`
- Текстовые поля непустые, `recipients` – массив, `message_metadata` – объект.
- Индексы: `workflow_state, COALESCE(occurred_at, projected_at) DESC`, `importance_score` (частичный)

### Почтовые вложения

#### `communication_mail_blobs`
Бинарные объекты вложений email:
- `blob_id` (PK), `storage_kind` (`local_fs`), `storage_path`, `sha256`, `size_bytes`, `content_type` (nullable)
- Уникальность: `(storage_kind, storage_path)` и `sha256`

#### `communication_attachments`
Вложения сообщений:
- `attachment_id` (PK), `message_id` (FK), `raw_record_id` (FK), `blob_id` (FK)
- `provider_attachment_id`, `filename` (nullable), `content_type`, `size_bytes`, `sha256`, `disposition` (`attachment`, `inline`, `unknown`)
- Уникальность: `(message_id, provider_attachment_id)`
- Позднее добавлены поля сканирования:
  - `scan_status` (`not_scanned`, `clean`, `suspicious`, `malicious`, `failed`)
  - `scan_engine`, `scan_checked_at`, `scan_summary`, `scan_metadata` (JSONB)

### Telegram

#### `telegram_chats`
Чаты Telegram:
- `telegram_chat_id` (PK), `account_id` (FK), `provider_chat_id`, `chat_kind` (`private`, `group`, `channel`, `bot`)
- `title`, `username` (nullable), `sync_state` (`fixture`, `syncing`, `synced`, `degraded`, `error`), `last_message_at`
- Уникальность: `(account_id, provider_chat_id)`

#### `automation_templates`
Шаблоны автоматических сообщений:
- `template_id` (PK), `name`, `body_template`, `required_variables` (JSONB массив)

#### `automation_policies`
Политики автоматической отправки Telegram:
- `policy_id` (PK), `template_id` (FK), `name`, `enabled`
- `account_id` (FK), `allowed_chat_ids` (JSONB массив), `trigger_kind`, `max_sends_per_hour` (>0), `quiet_hours` (JSONB объект), `expires_at`, `conditions` (JSONB объект)

#### `telegram_outbound_messages`
Исходящие сообщения Telegram:
- `outbound_message_id` (PK), `policy_id` (FK nullable), `template_id` (FK nullable), `account_id` (FK)
- `provider_chat_id`, `send_mode` (`dry_run`, `live`), `status` (`allowed`, `blocked`, `sent`, `failed`)
- `rendered_preview_hash`, `variables` (JSONB), `source_context` (JSONB), `actor_id`

#### `telegram_calls`
Звонки Telegram:
- `call_id` (PK), `account_id` (FK), `provider_call_id`, `provider_chat_id`
- `direction` (`incoming`, `outgoing`), `call_state` (`ringing`, `active`, `ended`, `missed`, `declined`, `failed`)
- `started_at`, `ended_at`, `transcription_policy_id` (FK nullable), `metadata` (JSONB)
- Уникальность: `(account_id, provider_call_id)`

#### `call_transcripts`
Транскрипции звонков:
- `transcript_id` (PK), `call_id` (FK), `account_id` (FK), `provider_chat_id`
- `transcript_status` (`queued`, `running`, `succeeded`, `failed`), `stt_provider`, `source_audio_ref` (nullable)
- `language_code` (nullable), `transcript_text`, `segments` (JSONB массив), `provenance` (JSONB)

### WhatsApp Web

#### `whatsapp_web_sessions`
Сессии WhatsApp Web:
- `session_id` (PK), `account_id` (FK, уникальный), `device_name`
- `companion_runtime` (`fixture`, `manual_webview`, `blocked`)
- `link_state` (`fixture`, `qr_pending`, `linked`, `degraded`, `revoked`, `blocked`)
- `local_state_path`, `last_sync_at`, `metadata` (JSONB)

## Контакты

### `contacts`
- `contact_id` (PK), `display_name`, `email_address` (UNIQUE)
- Текстовые поля непустые

## Документы

### `documents`
Импортированные документы:
- `document_id` (PK), `document_kind` (`markdown`, `pdf`), `title`, `source_fingerprint`, `extracted_text`, `imported_at`

### `document_processing_jobs`
Задания обработки:
- `job_id` (PK), `document_id` (FK), `step` (`extract_text`, `ocr`), `status` (`queued`, `running`, `succeeded`, `failed`, `skipped`)
- `attempts`, `max_attempts` (≥1), `last_error_summary`, временные метки
- Уникальность: `(document_id, step)`

### `document_artifacts`
Артефакты обработки:
- `artifact_id` (PK), `document_id` (FK), `job_id` (FK), `artifact_kind` (`extracted_text`, `ocr_text`)
- `content_sha256`, `text_content` (nullable), `storage_kind`, `storage_path` (nullable), `metadata` (JSONB)
- Ограничение: хотя бы одно из `text_content` или `storage_path` не NULL
- Уникальность: `(document_id, artifact_kind)`

## Граф знаний

### `graph_nodes`
Узлы графа:
- `node_id` (PK), `node_kind` (`person`, `email_address`, `message`, `document`, `project`)
- `stable_key`, `label`, `properties` (JSONB объект)
- Уникальность: `(node_kind, stable_key)`

### `graph_edges`
Связи графа:
- `edge_id` (PK), `source_node_id` (FK), `target_node_id` (FK)
- `relationship_type` (`person_has_email_address`, `person_sent_message`, `person_received_message`, `email_address_sent_message`, `email_address_received_message`, `project_has_message`, `project_has_document`, `project_involves_person`, `project_involves_email_address`)
- `confidence` (0.0–1.0), `review_state` (`system_accepted`, `suggested`, `user_confirmed`, `user_rejected`)
- `properties` (JSONB объект), `valid_from`, `valid_to` (nullable)
- Уникальный индекс активных связей: `(source_node_id, target_node_id, relationship_type)` где `valid_to IS NULL`

### `graph_evidence`
Доказательства для связей:
- `evidence_id` (PK), `edge_id` (FK), `source_kind` (`contact`, `message`, `document`, `raw_record`), `source_id`
- `excerpt` (nullable), `metadata` (JSONB объект)
- Уникальность: `(edge_id, source_kind, source_id)`

## Проекты

### `projects`
- `project_id` (PK), `name`, `kind`, `status` (`planning`, `active`, `on_hold`, `completed`, `archived`), `description`, `owner_display_name`
- `progress_percent` (0–100), `start_date`, `target_date` (nullable)
- Seed‑запись: проект «Макошь» (`project:v1:makosh`) с прогрессом 75%

### `project_keywords`
Ключевые слова проекта:
- PK `(project_id, keyword)`, `keyword` непустое
- Seed‑ключи: `Макошь`, `Макошь Project`, `makosh`

### `project_link_reviews`
Ревью связей проектов с сообщениями/документами:
- PK `(project_id, target_kind, target_id)`, где `target_kind` ∈ `message`, `document`
- `review_state` (`user_confirmed`, `user_rejected`), `event_id` (FK event_log), `actor_id`, `reviewed_at`

## Задачи (Tasks)

### `task_candidates`
Кандидаты задач от AI:
- PK `task_candidate_id`, `source_kind` (`message`, `document`), `source_id`, `project_id` (FK nullable)
- `title`, `due_text`, `assignee_label` (nullable)
- `confidence` (0.0–1.0), `review_state` (`suggested`, `user_confirmed`, `user_rejected`), `evidence_excerpt`
- `event_id`, `actor_id`, `generated_at`, `reviewed_at` (nullable)
- Позднее добавлен `agent_run_id` (FK → `ai_agent_runs`)
- Уникальный индекс: `(source_kind, source_id, lower(title))`

### `tasks`
Подтверждённые задачи:
- PK `task_id`, `task_candidate_id` (FK, UNIQUE), `title`, `source_kind`, `source_id`, `project_id` (FK nullable)
- `status` (пока только `active`), `created_from_event_id`, `created_by_actor_id`

## Идентичность контактов

### `contact_identity_candidates`
Предложения AI по контактам:
- PK `identity_candidate_id`, `candidate_kind` (`merge_contacts`, `attach_email_address`, `split_contact`)
- `left_contact_id` (FK), `right_contact_id` (FK nullable), `email_address` (nullable)
- `evidence_summary`, `confidence` (0.0–1.0), `review_state` (`suggested`, `user_confirmed`, `user_rejected`)
- Для `merge_contacts` обязательно наличие `right_contact_id`
- Уникальный индекс для пар слияния (при `candidate_kind = 'merge_contacts'`)

## Управление секретами

### `secret_references`
Ссылки на секреты:
- PK `secret_ref`, `secret_kind` (`oauth_token`, `app_password`, `password`, `api_token`, `private_key`, `other`)
- `store_kind` (`os_keychain`, `encrypted_vault`, `database_encrypted_vault`, `external_vault`, `test_double`)
- `label`, `metadata` (JSONB объект)

### `communication_provider_account_secret_refs`
Привязка аккаунтов к секретам:
- PK `(account_id, secret_purpose)`
- `secret_purpose`: `oauth_token`, `imap_password`, `smtp_password`, `telegram_api_hash`, `telegram_session_key`, `telegram_bot_token`, `whatsapp_web_session_key`
- `secret_ref` (FK)

### `encrypted_secret_vault_entries`
Записи зашифрованного хранилища в БД:
- PK `secret_ref` (FK), `kdf` (`argon2id:v1`), `salt`, `nonce`, `ciphertext`
- Все текстовые поля непустые

## AI‑рантайм

### `ai_agent_runs`
Запуски AI‑агентов:
- PK `run_id`, `agent_id`, `status` (`requested`, `completed`, `failed`)
- `chat_model`, `embedding_model`, `prompt_template_version`, `model_config` (JSONB)
- `query`, `answer` (nullable), `citations` (JSONB массив), `error_summary` (nullable)
- `actor_id`, `causation_id`, `correlation_id`, `requested_event_id`, `completed_event_id`, `failed_event_id`
- `started_at`, `completed_at` (nullable), `duration_ms`

### `semantic_embeddings`
Векторные представления (расширение [`pgvector`](https://github.com/pgvector/pgvector)):
- PK `semantic_embedding_id`, `source_kind` (`message`, `document`, `project`, `task`, `contact`), `source_id`
- `title`, `source_text`, `content_hash`, `embedding_model`, `embedding_dimension` (фиксировано 2560)
- `embedding` – `halfvec(2560)`, `graph_node_id` (nullable)
- Уникальность: `(source_kind, source_id, embedding_model)`
- HNSW‑индекс по `embedding` с `halfvec_cosine_ops` для быстрого поиска

## Прикладные настройки

### `application_settings`
Хранилище настроек приложения:
- PK `setting_key` (формат `^[a-z0-9][a-z0-9_.-]*[a-z0-9]$`, без секретных слов в ключе)
- `category`, `value_kind` (`boolean`, `integer`, `string`, `json`), `value` (JSONB)
- `label`, `description`, `metadata` (JSONB объект), `is_editable`, `updated_by_actor_id`
- Seed‑настройки (миграции 0023 и 0024):
  - `server.http_addr` (`"127.0.0.1:8080"`), `frontend.api_base_url`, `frontend.actor_id`
  - `ai.ollama_base_url`, `ai.chat_model`, `ai.embedding_model`, `ai.timeout_seconds`
  - `ui.theme` (`system`), `ui.density` (`comfortable`)

## Миграции и эволюция схемы

Схема расширялась от базового event‑sourcing (0001–0002) до полноценной системы с AI, Telegram‑звонками, WhatsApp и шифрованным хранилищем. Многие ограничения CHECK пересоздавались для добавления новых провайдеров или видов сущностей. Append‑only таблицы защищены триггерами, гарантируя целостность аудита и событийной модели.
```

## Покрытие источников

- **0001_create_event_log.sql** — таблица `event_log`, колонки, ограничения CHECK, индексы, триггеры append‑only, функция `prevent_event_log_mutation`.
- **0002_create_projection_cursors.sql** — таблица `projection_cursors`, колонки, ограничения, индекс `updated_at`.
- **0003_create_api_audit_log.sql** — таблица `api_audit_log`, колонки, ограничения, индексы, триггеры append‑only.
- **0004_add_api_audit_actor_id.sql** — колонка `actor_id` в `api_audit_log`, её ограничение и индекс.
- **0005_create_communication_ingestion.sql** — таблицы `communication_provider_accounts`, `communication_raw_records`, `communication_ingestion_checkpoints`, их колонки, ограничения, уникальности, индексы и триггеры append‑only для `communication_raw_records`.
- **0006_create_secret_references.sql** — таблицы `secret_references`, `communication_provider_account_secret_refs`, их колонки, CHECK‑ограничения, индексы.
- **0007_create_communication_messages.sql** — таблица `communication_messages`, колонки, ограничения, уникальность `(account_id, provider_record_id)`.
- **0008_create_contacts.sql** — таблица `contacts`, колонки, ограничения, UNIQUE `email_address`.
- **0009_create_documents.sql** — таблица `documents`, колонки, ограничения.
- **0010_create_graph_core.sql** — таблицы `graph_nodes`, `graph_edges`, `graph_evidence`, все колонки, CHECK‑ограничения (включая допустимые значения relationship), уникальности, индексы, уникальный индекс активных связей.
- **0011_create_mail_blob_storage.sql** — таблицы `communication_mail_blobs`, `communication_attachments`, все колонки, ограничения, уникальности, индексы.
- **0012_add_attachment_scan_metadata.sql** — колонки сканирования в `communication_attachments` (`scan_status`, `scan_engine`, `scan_checked_at`, `scan_summary`, `scan_metadata`) и их CHECK‑ограничения.
- **0013_create_projects_and_extend_graph.sql** — таблицы `projects`, `project_keywords`, seed‑запись проекта Макошь, seed‑ключевые слова, расширение CHECK‑ограничений `graph_nodes_kind` и `graph_edges_relationship_type`, новые индексы.
- **0014_create_project_link_reviews.sql** — таблица `project_link_reviews`, все колонки, PK, CHECK‑ограничения, индексы.
- **0015_create_task_candidates.sql** — таблицы `task_candidates`, `tasks`, все колонки, ограничения, уникальности, индексы.
- **0016_create_contact_identity_reviews.sql** — таблица `contact_identity_candidates`, все колонки, ограничения, уникальный индекс для пар merge, прочие индексы.
- **0017_create_document_processing.sql** — таблицы `document_processing_jobs`, `document_artifacts`, все колонки, ограничения, уникальности, индексы.
- **0018_create_ai_runtime.sql** — таблицы `ai_agent_runs`, `semantic_embeddings`, расширение `vector`, колонки, ограничения (включая `embedding_dimension = 2560`), уникальности, индексы (HNSW), добавление `agent_run_id` в `task_candidates` и индекс на него.
- **0019_rebuild_graph_projection_after_v3.sql** — оператор `TRUNCATE graph_evidence, graph_edges, graph_nodes` для перестройки проекций.
- **0020_create_v4_telegram_policy_calls.sql** — расширение `provider_kind` до telegram_user/telegram_bot, расширение `secret_purpose` для telegram, добавление колонок `channel_kind`, `delivery_state`, `message_metadata` в `communication_messages`, новые таблицы `telegram_chats`, `automation_templates`, `automation_policies`, `telegram_outbound_messages`, `telegram_calls`, `call_transcripts` со всеми ограничениями и индексами.
- **0021_create_v5_whatsapp_web_foundation.sql** — расширение `provider_kind` (whatsapp_web), `secret_purpose` (whatsapp_web_session_key), `channel_kind` (whatsapp_web), таблица `whatsapp_web_sessions` со всеми ограничениями и индексами.
- **0022_create_database_encrypted_secret_vault.sql** — расширение `store_kind` на `database_encrypted_vault`, таблица `encrypted_secret_vault_entries` с KDF `argon2id:v1`, ограничениями, индексами.
- **0023_create_application_settings.sql** — таблица `application_settings`, все колонки, ограничения на ключ и исключение секретных слов, seed‑настройки.
- **0024_seed_runtime_application_settings.sql** — seed‑вставки настроек `server.http_addr`, `frontend.api_base_url`, `frontend.actor_id` (ON CONFLICT DO NOTHING).
- **0025_add_message_workflow_state.sql** — колонки `workflow_state`, `importance_score`, `ai_category`, `ai_summary`, `ai_summary_generated_at` в `communication_messages`, ограничения значений, индексы.

## Исходные файлы

- [`backend/migrations/0001_create_event_log.sql`](../../../../backend/migrations/0001_create_event_log.sql)
- [`backend/migrations/0002_create_projection_cursors.sql`](../../../../backend/migrations/0002_create_projection_cursors.sql)
- [`backend/migrations/0003_create_api_audit_log.sql`](../../../../backend/migrations/0003_create_api_audit_log.sql)
- [`backend/migrations/0004_add_api_audit_actor_id.sql`](../../../../backend/migrations/0004_add_api_audit_actor_id.sql)
- [`backend/migrations/0005_create_communication_ingestion.sql`](../../../../backend/migrations/0005_create_communication_ingestion.sql)
- [`backend/migrations/0006_create_secret_references.sql`](../../../../backend/migrations/0006_create_secret_references.sql)
- [`backend/migrations/0007_create_communication_messages.sql`](../../../../backend/migrations/0007_create_communication_messages.sql)
- [`backend/migrations/0008_create_contacts.sql`](../../../../backend/migrations/0008_create_contacts.sql)
- [`backend/migrations/0009_create_documents.sql`](../../../../backend/migrations/0009_create_documents.sql)
- [`backend/migrations/0010_create_graph_core.sql`](../../../../backend/migrations/0010_create_graph_core.sql)
- [`backend/migrations/0011_create_mail_blob_storage.sql`](../../../../backend/migrations/0011_create_mail_blob_storage.sql)
- [`backend/migrations/0012_add_attachment_scan_metadata.sql`](../../../../backend/migrations/0012_add_attachment_scan_metadata.sql)
- [`backend/migrations/0013_create_projects_and_extend_graph.sql`](../../../../backend/migrations/0013_create_projects_and_extend_graph.sql)
- [`backend/migrations/0014_create_project_link_reviews.sql`](../../../../backend/migrations/0014_create_project_link_reviews.sql)
- [`backend/migrations/0015_create_task_candidates.sql`](../../../../backend/migrations/0015_create_task_candidates.sql)
- [`backend/migrations/0016_create_contact_identity_reviews.sql`](../../../../backend/migrations/0016_create_contact_identity_reviews.sql)
- [`backend/migrations/0017_create_document_processing.sql`](../../../../backend/migrations/0017_create_document_processing.sql)
- [`backend/migrations/0018_create_ai_runtime.sql`](../../../../backend/migrations/0018_create_ai_runtime.sql)
- [`backend/migrations/0019_rebuild_graph_projection_after_v3.sql`](../../../../backend/migrations/0019_rebuild_graph_projection_after_v3.sql)
- [`backend/migrations/0020_create_v4_telegram_policy_calls.sql`](../../../../backend/migrations/0020_create_v4_telegram_policy_calls.sql)
- [`backend/migrations/0021_create_v5_whatsapp_web_foundation.sql`](../../../../backend/migrations/0021_create_v5_whatsapp_web_foundation.sql)
- [`backend/migrations/0022_create_database_encrypted_secret_vault.sql`](../../../../backend/migrations/0022_create_database_encrypted_secret_vault.sql)
- [`backend/migrations/0023_create_application_settings.sql`](../../../../backend/migrations/0023_create_application_settings.sql)
- [`backend/migrations/0024_seed_runtime_application_settings.sql`](../../../../backend/migrations/0024_seed_runtime_application_settings.sql)
- [`backend/migrations/0025_add_message_workflow_state.sql`](../../../../backend/migrations/0025_add_message_workflow_state.sql)

## Кандидаты на drift

Не видны из представленного контекста — страница создаётся/обновляется на основе исходников, поэтому расхождений с кодом нет.
