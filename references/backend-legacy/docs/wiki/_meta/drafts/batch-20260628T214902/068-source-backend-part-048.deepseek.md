### Summary / Резюме

Страница `components/backend.md` наполняется описанием бэкенда Макошь: точка входа, структура модулей, AI‑порт, система аудита, модели звонков и интеграция с Zoom (аккаунты, формы авторизации, жизненный цикл, наблюдение встреч, константы и валидация). Информация основана исключительно на предоставленных исходных файлах.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend

Бэкенд Макошь — асинхронное приложение на Rust (tokio), запускаемое через `#[tokio::main]`.
Настройки читаются из окружения через `AppConfig::from_env()`, поток выполнения оборачивается в `tracing` span с идентификатором `MAKOSH_FLOW_ID`.
Для диагностики используется `color_eyre`.

Файлы: `backend/src/main.rs`, `backend/src/lib.rs`.

## Модульная структура (`lib.rs`)

```rust
pub mod ai;
pub mod app;
pub mod application;
pub mod contracts;
pub mod domains;
pub mod engines;
pub mod integrations;
pub mod platform;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod vault;
pub mod workflows;
```

> Описание модулей `ai`, `app`, `application`, `contracts`, `domains`, `engines`, `vault`, `workflows` не подтверждено предоставленным контекстом.

## Platform

### AI Runtime

Файл: `backend/src/platform/ai_runtime.rs`.

Трейт `AiRuntimePort` (Send + Sync) определяет контракт для AI‑провайдеров:

- `runtime_name() -> &'static str`
- `chat(prompt) -> Future<Output = Result<AiChatResult, AiRuntimePortError>>`
- `embed_with_model(input, model) -> Future<Output = Result<AiEmbedResult, AiRuntimePortError>>`

Структуры результатов:
- `AiChatResult` — `model`, `content`, `total_duration_ns`
- `AiEmbedResult` — `model`, `embedding: Vec<f32>`, `total_duration_ns`

Ошибка: `AiRuntimePortError { runtime, message }`.

### Audit

Файлы: `backend/src/platform/audit/` (models, store, events, reviews, settings, communication, documents, telegram, telegram_dialogs, telegram_participants, constants, helpers, errors).

Система аудита фиксирует операции, инициированные через фронтенд.
Основные модели:

- `NewApiAuditRecord` — структура для создания записи; поля: `actor_kind` (всегда `"frontend"`), `actor_id`, `operation`, `method`, `path_template`, `target_kind`, `target_id`, `metadata` (JSON).
- `ApiAuditRecord` — хранимая запись, дополнительно содержит `audit_id` и `recorded_at`.

Хранилище `ApiAuditLog` (pg) сохраняет записи в таблицу `api_audit_log` (метод `record`) и извлекает события по фильтрам (`list_event_records`, где `target_kind = 'event'`).

Фабричные методы `NewApiAuditRecord` покрывают различные доменные операции:

- События: `event_append`, `event_get`, `event_list`.
- Коммуникация: `communication_email_send`.
- Документы: `document_processing_job_retry`.
- Настройки: `application_setting_set`.
- Рецензии: `project_link_review_set`, `task_candidate_review_set`, `obligation_review_set`, `decision_review_set`, `relationship_review_set`, `contradiction_review_set`, `message_workflow_state_set`, `person_identity_review_set`.
- Telegram: отправка (`telegram_message_send`, `automation_telegram_send_dry_run`/`rejected`), работа с диалогами (pin, unpin, archive, unarchive, mute, unmute, folder_add/remove/reassign, mark_read/unmark_read, join, leave), синхронизация участников (`telegram_participants_sync`), управление аккаунтом (`telegram_account_logout`, `telegram_account_remove`) и рантаймом (`telegram_runtime_stop`, `telegram_runtime_restart`).

Каждая запись встраивает метаданные разрешений из `CapabilityDecision`.

### Calls

Файлы: `backend/src/platform/calls/` (models, errors, store, stt, validation; публичный модуль `calls.rs`).

Типы звонков:

- `NewTelegramCall` / `TelegramCall` — поля: `call_id`, `account_id`, `provider_call_id`, `provider_chat_id`, `direction`, `call_state`, `started_at`, `ended_at`, `transcription_policy_id`, `metadata`.
- Алиасы: `NewProviderCall = NewTelegramCall`, `ProviderCall = TelegramCall`.
- Перечисления:
  - `CallDirection`: `Incoming`, `Outgoing` (сериализация `snake_case`).
  - `CallState`: `Ringing`, `Active`, `Ended`, `Missed`, `Declined`, `Failed`.
- Транскрипты:
  - `NewCallTranscript` / `CallTranscript` — поля: `transcript_id`, `call_id`, `account_id`, `provider_chat_id`, `transcript_status`, `stt_provider`, `source_audio_ref`, `language_code`, `transcript_text`, `segments` (JSON-массив), `provenance` (JSON-объект).
  - `TranscriptStatus`: `Queued`, `Running`, `Succeeded`, `Failed`.

Ошибки: `CallError` — варианты `InvalidRequest` и `Sqlx(transparent)`.

## Интеграции

### Zoom

Файлы: `backend/src/integrations/zoom/` (модули `client` и `runtime`; `runtime` реэкспортирует модели из `client`).

#### Константы и провайдер

- `ZOOM_PROVIDER_KIND` = `CommunicationProviderKind::ZoomUser`
- `ZOOM_PROVIDER_KIND_STR` = `"zoom_user"`
- `ZOOM_RUNTIME_KIND` = `"zoom_fixture_runtime"`
- `ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND` = `"zoom_live_authorized_runtime"`
- URL по умолчанию: авторизация (`DEFAULT_ZOOM_AUTHORIZATION_ENDPOINT`), токены (`DEFAULT_ZOOM_TOKEN_ENDPOINT`), API (`DEFAULT_ZOOM_API_BASE_URL`).
- Токен‑пороги: safety margin, explicit refresh, maintenance refresh, max refresh.
- Лимиты синхронизации: `ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE = 30`, `ZOOM_PROVIDER_SYNC_MAX_PAGE_SIZE = 100`, `ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS = 100`, `ZOOM_PROVIDER_SYNC_MAX_MEETINGS = 500`.
- `ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES = 268_435_456` (256 MiB).
- События вебхуков по умолчанию: `meeting.started`, `meeting.ended`, `meeting.participant_joined`, `meeting.participant_left`, `recording.completed`.

#### Формы авторизации (`ZoomAuthShape`)

- `Fixture`
- `OAuthUser` (сериализуется как `oauth_user`)
- `ServerToServer` (сериализуется как `server_to_server`)

#### Модели аккаунтов

- `ZoomAccountSetupRequest` — для fixture‑аккаунтов; поля `account_id`, `display_name`, `external_account_id`, `account_email`, `metadata`. Валидация: все строки непустые, `metadata` — объект. Метод `account_config()` формирует JSON с `provider_kind = "zoom_user"`, `runtime_kind = "zoom_fixture_runtime"`, `lifecycle_state = "fixture_ready"`.
- `ZoomLiveAccountSetupRequest` — для живых аккаунтов; дополнительно содержит `auth_shape`, `client_id`, ссылки на секреты (`token_secret_ref`, `client_secret_ref`, `webhook_secret_ref`), `metadata`. Валидация запрещает `auth_shape = Fixture`, проверяет непустоту `client_id` и опциональные ссылки. Метод `provider_kind()` возвращает `ZoomServerToServer` для S2S, иначе `ZoomUser`. `account_config()` задаёт `runtime_kind = "zoom_live_blocked_runtime"`, `lifecycle_state = "blocked"`, `runtime_blockers = ["zoom_live_authorization_required"]` и карту привязок секретов (`credential_refs_bound`).
- `ZoomAccount` — проекция `ProviderAccount` с вычисляемыми полями `auth_shape`, `lifecycle_state`, `runtime_kind`, `account_email` из JSON‑конфига.

#### Жизненный цикл аккаунта (store)

Методы `ZoomStore` (использует `PgPool`, `ProviderAccountCommandPort`, `ProviderSecretBindingCommandPort`, `ImportedAttachmentStoragePort`, `CallIntelligenceStore`, `EventStore`, `EventBus`, `reqwest::Client`):

- `setup_fixture_account` / `setup_live_blocked_account` — создание аккаунта через `upsert_runtime_account`, для live дополнительно привязываются секреты.
- `start_oauth` — запускает OAuth, сначала создаёт live‑аккаунт, генерирует `setup_id` и `state`, возвращает `ZoomOAuthPendingGrant` с `authorization_url`.
- `list_accounts` — отфильтровывает zoom‑аккаунты, при `include_removed = false` исключает `lifecycle_state = "removed"`.
- `runtime_status` — получает `ZoomRuntimeStatus` из полей аккаунта.
- `start_runtime` — выставляет `lifecycle_state` в `"running"` (если авторизован) или `"blocked"`, `runtime_kind` — соответствующий, очищает/устанавливает `runtime_blockers`, записывает `last_runtime_action`.
- `stop_runtime` — переводит в `"stopped"` (если не `removed`), записывает `last_runtime_action`.
- `remove_runtime` — устанавливает `lifecycle_state = "removed"`, `removed_at`, `remove_reason`.
- Все мутации публикуют событие через `EventBus` (типы: `zoom.runtime.start_requested`, `zoom.runtime.stop_requested`, `zoom.runtime.remove_requested`).

`ZoomRuntimeStatus` содержит поля: `account_id`, `provider_kind`, `runtime_kind`, `status`, `healthy`, `auth_shape`, `live_runtime_available`, `recording_ingest_available`, `transcript_ingest_available`, `runtime_blockers: Vec<String>`, `last_error`, `checked_at`, `metadata`.

#### Наблюдение встреч

`ZoomMeetingObservationRequest` — поля: `observation_id`, `account_id`, `meeting_id`, `meeting_uuid`, `topic`, `host_email`, `join_url`, `started_at`, `ended_at`, `duration_seconds`, `participants: Vec<ZoomParticipantSnapshot>`, `recording_refs: Vec<ZoomRecordingRef>`, `transcript_ref`, `metadata`, `causation_id`, `correlation_id`.

Валидация: `account_id` и `meeting_id` непустые, `metadata` — объект.

Методы:
- `provider_chat_id() -> String` = `"zoom:meeting:{meeting_id}"`
- `event_subject_id() -> String` — `meeting_uuid`, если задан и непустой, иначе `meeting_id`.
- `into_call(call_id, observed_at) -> NewProviderCall` — преобразует в вызов, санируя JSON‑поля `participants`, `recording_refs`, `metadata`.

При поступлении `observe_meeting` сохраняет `NewProviderCall` через `call_store.upsert_call`.

#### Дополнительные модели (частично видны)

- `ZoomParticipantSnapshot`: `participant_id`, `display_name`, `email`, `joined_at`, `left_at`, `metadata`.
- `ZoomRecordingRef`: `recording_id`, `recording_type`, `download_ref`, `file_extension`, `file_size_bytes`, `recorded_at`, `metadata`.

#### Валидация

Файл: `backend/src/integrations/zoom/client/validation.rs`.

Вспомогательные функции `validate_non_empty`, `validate_object`, `validate_array` используются в `validate()`-методах моделей и возвращают `ZoomError::InvalidRequest`.

#### Настройки хранения

Ключи `privacy.zoom_recording_import_retention_days` и `privacy.zoom_transcript_retention_days` доступны как константы в `store.rs`.

## База данных

Во всех хранилищах (`ZoomStore`, `ApiAuditLog`, `CallIntelligenceStore`) используется PostgreSQL через `sqlx::PgPool`.

## События

`EventBus` и `EventStore` участвуют в публикации событий изменения состояния аккаунтов (на примере Zoom).
```

### Source coverage / Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/main.rs` | точка входа: `#[tokio::main]`, `color_eyre`, `tracing`, `MAKOSH_FLOW_ID`, `AppConfig::from_env()`, вызов `app::run(config)` |
| `backend/src/lib.rs` | список публичных модулей, атрибут `allow(dead_code, unused_imports, unused_variables)` |
| `backend/src/platform/ai_runtime.rs` | трейт `AiRuntimePort`, `AiChatResult`, `AiEmbedResult`, `AiRuntimePortError` |
| `backend/src/platform/audit.rs` | реэкспорт моделей и хранилища аудита |
| `backend/src/platform/audit/models.rs` | `ApiAuditRecord`, `NewApiAuditRecord`, поле `actor_kind = "frontend"` |
| `backend/src/platform/audit/store.rs` | `ApiAuditLog`, методы `record` (INSERT), `list_event_records` (`target_kind = 'event'`) |
| `backend/src/platform/audit/events.rs` | фабрики `event_append`, `event_get`, `event_list` |
| `backend/src/platform/audit/communication.rs` | фабрика `communication_email_send` |
| `backend/src/platform/audit/documents.rs` | фабрика `document_processing_job_retry` |
| `backend/src/platform/audit/settings.rs` | фабрика `application_setting_set` |
| `backend/src/platform/audit/reviews.rs` | фабрики review‑set для проектов, задач, обязательств, решений, связей, противоречий, сообщений, идентичности |
| `backend/src/platform/audit/telegram.rs` (truncated) | фабрики `automation_telegram_send_dry_run`, `telegram_message_send`, `telegram_media_upload`, `telegram_account_logout`, `telegram_account_remove`, `telegram_runtime_stop`, `telegram_runtime_restart` |
| `backend/src/platform/audit/telegram_dialogs.rs` | фабрики действий с диалогами (pin, unpin, archive, unarchive, mute, unmute, folder_add/remove/reassign, mark_read/unmark_read, join, leave) |
| `backend/src/platform/audit/telegram_participants.rs` | фабрика `telegram_participants_sync`, тест аудита |
| `backend/src/platform/audit/helpers.rs` | вспомогательные функции `insert_non_empty`, `insert_optional`, `non_empty_optional` |
| `backend/src/platform/audit/constants.rs` | `API_FRONTEND_ACTOR_KIND`, `EVENT_TARGET_KIND` |
| `backend/src/platform/audit/errors.rs` | `ApiAuditError::Sqlx` |
| `backend/src/platform/calls.rs` | реэкспорт моделей звонков, транскриптов, `CallIntelligenceStore`, фикстур STT |
| `backend/src/platform/calls/models.rs` | `NewTelegramCall`, `TelegramCall`, алиасы `NewProviderCall`/`ProviderCall`, `CallDirection`, `CallState`, `NewCallTranscript`, `CallTranscript`, `TranscriptStatus`, валидация |
| `backend/src/platform/calls/errors.rs` | `CallError` с вариантами `InvalidRequest` и `Sqlx` |
| `backend/src/integrations/zoom/mod.rs` | объявление модулей `client` и `runtime` |
| `backend/src/integrations/zoom/runtime.rs` | реэкспорт запросов управления рантаймом |
| `backend/src/integrations/zoom/client/models.rs` (truncated) | константы Zoom, `ZoomAuthShape`, `ZoomAccountSetupRequest`, `ZoomLiveAccountSetupRequest`, `ZoomAccount`, `ZoomRuntimeStatus`, запросы старта/стопа/удаления рантайма, `ZoomParticipantSnapshot`, `ZoomRecordingRef`, `ZoomMeetingObservationRequest` с методами `provider_chat_id`, `event_subject_id`, `into_call` |
| `backend/src/integrations/zoom/client/store.rs` (truncated) | структура `ZoomStore` и её зависимости, методы `setup_fixture_account`, `setup_live_blocked_account`, `start_oauth`, `list_accounts`, `runtime_status`, `start_runtime`, `stop_runtime`, `remove_runtime`, `observe_meeting`, ключи retention‑настроек |
| `backend/src/integrations/zoom/client/validation.rs` | функции `validate_non_empty`, `validate_object`, `validate_array` |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно. Файлы усечены (`models.rs` и `store.rs`), но те части, что встроены, не содержат видимых противоречий с внутренними контрактами.
