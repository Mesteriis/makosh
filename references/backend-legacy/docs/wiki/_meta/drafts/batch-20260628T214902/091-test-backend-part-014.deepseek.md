### Summary / Резюме

Создать или обновить страницу `operations/backend-tests.md` в русской Obsidian‑wiki, описывающую состав и свойства интеграционных и юнит‑тестов бэкенда из репозитория `makosh`. Страница должна отражать только факты, присутствующие во встроенных исходных файлах тестов, и не добавлять внешних знаний о фреймворках. Структура страницы: обзор тестового набора, разбивка по группам (Telegram‑интеграции, движки, v1 API, тестовая инфраструктура), перечень тестируемых сценариев и используемых хелперов.

### Proposed pages / Предлагаемые страницы

#### `operations/backend-tests.md`

```markdown
# Тесты бэкенда

Набор интеграционных и юнит‑тестов, расположенных в каталоге `backend/tests/`. Тесты используют изолированные базы данных через `testkit::context::TestContext`, маршрутизатор приложения `build_router_with_database` и фикстурные хелперы для Telegram.

## Структура

Каждый файл соответствует определённому компоненту. Основные группы:

- **Telegram‑интеграции** – маршруты, управление runtime, поиск, закреплённые сообщения, capability‑гейты, топики.
- **Движки** – `TimelineEngine`, `TrustEngine`.
- **v1 API** – статус, сообщения, вложения, AI‑состояния, синхронизация, настройки.
- **Поддержка** – общие утилиты (`telegram_support/mod.rs`).

## Telegram‑интеграции

### `telegram_references_members.rs`

Проверяет маршруты цепочек ответов и пересылок.

- Тест-функция: `telegram_reference_routes_return_enriched_message_summaries`.
- Создаются сообщения в чате через фикстурный endpoint `/api/v1/integrations/telegram/fixtures/messages`.
- Программно вставляются связи `reply` и `forward` через вызовы `lifecycle::insert_reply_ref` и `lifecycle::insert_forward_ref` с циклическими и двухуровневыми цепочками.
- Проверяются ответы endpoint’ов:
  - `GET /api/v1/communications/messages/{message_id}/reply-chain` – поле `reply_to` с `target_message_summary`, содержащим `text` и `sender_display_name`.
  - `GET /api/v1/communications/messages/{message_id}/forward-chain` – поле `forwards` с `source_message_summary` и `forward_origin_sender_name`.
- Утверждается, что количество элементов в `reply_to` и `forwards` соответствует созданным связям даже при наличии циклов.

### `telegram_reply_forward_capability_gates.rs`

Проверяет, что фикстурный аккаунт блокирует запись команд reply/forward до выполнения побочных эффектов.

- Тест-функция: `fixture_account_blocks_reply_and_forward_before_side_effects`.
- Pop‑up аккаунт и сообщения, затем попытки reply и forward:
  - `POST /api/v1/integrations/telegram/provider-commands/messages/{message_id}/reply`
  - `POST /api/v1/integrations/telegram/provider-commands/messages/{message_id}/forward`
- Ожидаемый статус: `BAD_REQUEST`.
- Проверки:
  - Количество сообщений в чате не изменилось.
  - В таблице `api_audit_log` нет записей с `operation = 'telegram.message.send'`.
  - В таблице `event_log` нет событий `telegram.command.status_changed` и дополнительных `telegram.message.created`.
  - Уже существующее сообщение‑пересылка (`telegram.message.created`) сохраняется (т.к. оно не связано с заблокированной командой).

### `telegram_runtime_lifecycle.rs`

Охватывает жизненный цикл runtime Telegram‑аккаунта и диагностику.

- Тест-функция: `telegram_fixture_runtime_status_can_start_account_actor`
  - Создаёт аккаунт через `POST /api/v1/integrations/telegram/fixtures/accounts`.
  - Проверяет первоначальный статус (`GET /api/v1/integrations/telegram/runtime/status`): `status: "stopped"`, `runtime_kind: "fixture"`, `live_send_available: false`, `runtime_blockers: []`.
  - Выполняет:
    - `POST /api/v1/integrations/telegram/runtime/start` → ответ содержит `status: "running"`.
    - `POST /api/v1/integrations/telegram/runtime/restart` → ответ содержит `status: "running"`.
    - `POST /api/v1/integrations/telegram/runtime/stop` → ответ содержит `status: "stopped"`.
  - Проверяет записи в `api_audit_log` для операций `telegram.runtime.stop` и `telegram.runtime.restart` с полями `capability`, `action_class`, `account_id`, `runtime_kind`, `status`.

- Тест-функция: `telegram_runtime_status_reports_tdlib_diagnostics_for_qr_authorized_user_accounts`
  - Использует конфигурацию с `MAKOSH_DEV_MODE`, `MAKOSH_TDJSON_PATH` (путь к несуществующей dylib), `MAKOSH_TELEGRAM_API_ID` и `MAKOSH_TELEGRAM_API_HASH`.
  - Создаёт аккаунт через `POST /api/v1/integrations/telegram/accounts`.
  - Проверяет статус runtime:
    - `runtime_kind: "tdlib_qr_authorized"`
    - `tdjson_path` равен заданному пути
    - `tdjson_runtime_available: false`
    - `telegram_api_id_configured: true`, `telegram_api_hash_configured: true`, `telegram_app_credentials_configured: true`
    - `live_send_available: false`
    - `tdjson_probe_error` содержит строку `"unable to load libtdjson"`
    - `runtime_blockers` содержит `"tdjson_runtime_unavailable"`

- Тест-функция: `telegram_account_lifecycle_lists_logs_out_and_removes_without_deleting_evidence` – (текст обрезан; контекст не полностью покрыт).

### `telegram_search_pinning.rs`

Проверяет поиск диалогов, медиа‑поиск и закреплённые сообщения.

- Тест-функция: `telegram_dialog_search_returns_projected_chat_matches`
  - Создаётся аккаунт через фикстуры, два чата с разными названиями (`Project Alpha Ops`, `Beta Support`).
  - Запрос `GET /api/v1/communications/conversations/search?q=Alpha&account_id=…&limit=10`
  - Ответ: `items` содержит один элемент с `provider_chat_id` совпадающего чата и `title: "Project Alpha Ops"`.

- Тест-функция: `telegram_media_search_filters_by_free_text_query`
  - Создаётся сообщение со вложением `invoice-2026.pdf` через прямое обновление `communication_messages.message_metadata`.
  - Запрос `GET /api/v1/communications/search/media?q=invoice&account_id=…&provider_chat_id=…&limit=20`
  - Ответ: `items` содержит одно вложение с `file_name: "invoice-2026.pdf"`, `provider_attachment_id: "attachment-invoice-1"`, `tdlib_file_id: 4201`, `local_path: "/tmp/makosh/invoice-2026.pdf"`. Поле `source: "projection"`, `provider_search_attempted: false`.

- Тест-функция: `telegram_pinned_messages_route_returns_projection_backed_items`
  - Создаются сообщения, часть помечается как `is_pinned: true` в `message_metadata`.
  - Получается `telegram_chat_id` через `GET /api/v1/communications/conversations`.
  - Запрос `GET /api/v1/communications/conversations/{telegram_chat_id}/pinned-messages?limit=10`
  - Ответ: `items` содержит два закреплённых сообщения, отсортированных по времени (сначала новейшее). Текст `"Newest pinned message"` и `"Pinned root message"`.

### `telegram_topic_capability_gates.rs`

Проверяет capability‑гейты для топиков чата.

- Тест-функция: `fixture_account_allows_topic_list_but_blocks_topic_writes`
  - Создаётся аккаунт и чат. Вставляется топик через прямой SQL в `telegram_topics`.
  - `GET /api/v1/communications/conversations/{telegram_chat_id}/topics?limit=10` возвращает список с одним топиком.
  - `POST …/topics` для создания топика возвращает `BAD_REQUEST`.
  - `POST …/topics/{topic_id}/close` возвращает `BAD_REQUEST`.
  - Запись команд `topic_create`, `topic_close`, `topic_reopen` в `telegram_provider_write_commands` отсутствует.

### `telegram_support/mod.rs`

Модуль с общими хелперами для Telegram‑тестов.

- Константа `LOCAL_API_TOKEN = "telegram-api-test-secret"`.
- Функции-строители HTTP‑запросов: `json_post_request_with_actor`, `json_post_request_with_explicit_actor_header`, `get_request_with_token`, `delete_request_with_token`.
- `assert_capability_status` – проверка capability в ответе.
- `assert_ok` – отправка POST‑запроса и проверка статуса 200.
- `ingest_fixture_telegram_message` – вспомогательная отправка фикстурного сообщения и извлечение `message_id`.
- `account_item` – поиск аккаунта в списке по `account_id`.
- `json_body` – десериализация тела ответа в `serde_json::Value`.
- `unique_suffix` – генерация уникального суффикса на основе наносекунд эпохи.
- `vault_entropy_events` – генерация массива энтропийных событий (используется в других тестах).

## Движки

### `timeline_engine.rs`

Юнит‑тесты для `TimelineEngine`. Не зависят от HTTP.

- `timeline_engine_bounds_entity_timeline_limits` – проверка `bounded_entity_limit`: 0 → 1, 25 → 25, 250 → 100.
- `timeline_engine_rejects_unsourced_timeline_event` – событие с пустым (`" "`) источником отклоняется с ошибкой `"timeline event source must not be empty"`.
- `timeline_engine_accepts_source_backed_timeline_event` – событие с заполненным `source` (`"communication_messages:message-1"`) принимается.
- `timeline_engine_builds_period_summary_for_source_backed_events` – сводка за период считает количество событий по `entity_kind` и `event_type`, игнорируя события вне периода.
- `timeline_engine_rejects_invalid_period_summary_range` – неверный диапазон (start > end) вызывает ошибку `"timeline period start must not be after period end"`.
- `timeline_engine_builds_recency_signal_for_source_backed_entity_events` – `recency_signal` возвращает последнее событие до заданного момента (`last_event_at`, `last_event_type`, `last_event_source`, `age_seconds`), игнорируя будущие события.
- `timeline_engine_detects_source_backed_entity_timeline_gaps` – `timeline_gaps` определяет интервал между событиями одной сущности, превышающий порог (259200 с), с указанием границ и источников предшествующего/последующего события.
- `timeline_engine_rejects_invalid_gap_threshold` – порог 0 вызывает ошибку `"timeline gap threshold must be greater than zero"`.
- `timeline_engine_builds_source_backed_change_diff_for_entity_snapshots` – `change_diff` вычисляет добавленные и удалённые события между двумя снимками по источнику, типу и времени.
- `timeline_engine_builds_cross_domain_timeline_for_source_backed_events` – (текст обрезан; видно только объявление).

### `trust_engine.rs`

Юнит‑тесты для `TrustEngine`.

- `trust_engine_maps_persona_compatibility_score_to_relationship_signal`:
  - Вызов `persona_compatibility_score_signal(82)` → сигнал с параметрами: `kind: PersonaCompatibilityScore`, `relationship_type: "trusts"`, `trust_score: 0.82`, `strength_score: 0.5`, `confidence: 1.0`, `explanation: "compatibility persons.trust_score signal"`.
- `trust_engine_clamps_legacy_persona_scores_to_relationship_range`:
  - Отрицательный score (−20) обрезается до `0.0`, превышающий 100 (135) – до `1.0`.
- `trust_engine_builds_source_reliability_signal_for_review`:
  - `source_reliability_signal("person_enrichment:persona:v1:human:alice:trust_score", "trust_score=82", 0.82)` → сигнал с `kind: SourceReliability`, `affected_source`, `evidence`, `confidence: 0.82`, `direction: "positive"`.
- `trust_engine_rejects_unsourced_source_reliability_signal`:
  - Пустой `affected_source` вызывает ошибку `"trust signal affected source must not be empty"`.

## v1 API

### `v1_api.rs`

Интеграционные тесты для endpoint’а статуса, деталей сообщений, аутентификации и CORS.

- `v1_status_returns_enabled_surfaces_against_postgres`:
  - `GET /api/v1/status` с валидным токеном → `version: "1.0"`, `surfaces` содержит `messages`, `persons`, `search`, `documents`, `account_setup` со значением `true`.
- `v1_communications_message_detail_returns_attachment_metadata_against_postgres`:
  - Создаётся Email‑сообщение с вложением через хранилища (ingestion, projection, storage).
  - `GET /api/v1/communications/messages?limit=100` – в списке присутствует сообщение с `subject` и `attachment_count: 1`.
  - `GET /api/v1/communications/messages/{message_id}` – возвращает `message.body_text`, `attachments` с полями `filename`, `content_type`, `scan_status: "not_scanned"`, `storage_kind: "local_fs"`, `storage_path`.
- `v1_status_rejects_missing_local_api_secret_before_database_access`:
  - Запрос без токена → `403 FORBIDDEN`, тело `{"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"}`.
- `v1_status_accepts_local_frontend_cors_preflight_before_auth`:
  - OPTIONS‑запросы от локальных origin (`http://127.0.0.1:5174`, `http://localhost:5173`, `http://tauri.localhost`, `tauri://localhost`) отвечают `200 OK` с `Access-Control-Allow-Origin`, равным запрошенному origin.
- `v1_status_rejects_invalid_local_api_secret_before_database_access`:
  - Неверный токен → `403 FORBIDDEN`.
- `v1_status_accepts_secret_without_actor_header_before_database_access`:
  - Корректный токен без `x-makosh-actor-id` → `503 SERVICE_UNAVAILABLE` с `error: "database_not_configured"`.
- `v1_status_ignores_actor_header_before_database_access`:
  - Корректный токен с невалидным actor → аналогично `503`.
- `v1_status_returns_service_unavailable_after_auth_when_database_is_not_configured`:
  - Полный запрос без базы данных → `503`.

### `v1_communications_api.rs`

Широкая проверка ручек чтения/записи, синхронизации и отправки с guard-ом записи.

- Макрос `v1_read_test!` прогоняет GET‑запросы к перечисленным endpoint’ам и проверяет отсутствие 5xx ошибок:
  `v1_messages_list`, `v1_message_states`, `v1_threads_list`, `v1_thread_messages`, `v1_search`, `v1_personas_list`, `v1_drafts_list`, `v1_invoices_list`, `v1_analytics_health`, `v1_analytics_senders`, `v1_subscriptions`, `v1_dup_attachments`, `v1_legal_list`, `v1_certs_list`, `v1_certs_expiring`, `v1_rich_templates`, `v1_blockers`, `v1_sync_status`.
- Макрос `v1_post_test!` проверяет, что POST‑эндпоинты возвращают успешные или ожидаемые клиентские ошибки (404, 400, 422): `v1_pin_msg`, `v1_snooze_msg`, `v1_mute_msg`, `v1_label_msg`, `v1_render_tpl`.
- `v1_sync_settings_default_update_and_manual_sync_status_against_postgres`:
  - Создаётся IMAP‑аккаунт.
  - `GET /api/v1/integrations/mail/accounts/{account_id}/sync-settings` возвращает настройки по умолчанию: `sync_enabled: true`, `batch_size: 100`, `poll_interval_seconds: 300`.
  - `PUT` с изменёнными значениями (`sync_enabled: false`, `batch_size: 7`, `poll_interval_seconds: 600`) успешно обновляет.
  - `POST …/sync-now` запускает синхронизацию, в теле ответа ожидаются `account_id`, `status`, `phase`, `run_id`. При ошибке конфигурации возвращается `invalid_communication_query`.
  - В `event_log` появляются события `mail.sync.started` и `mail.sync.skipped` с `run_id`, `status: "skipped"`.
  - В таблице `observation_links` фиксируются связи `COMMUNICATION_MAIL_SYNC_RUN` и `COMMUNICATION_MAIL_SYNC_RUN_STATUS`, последняя с `relationship_kind: "skipped"`.
  - `POST …/sync-full-resync` работает аналогично `sync-now`, в ответе также ожидаются `account_id`, `status`, `phase`, `run_id`.
- `v1_send_requires_explicit_provider_write_confirmation`:
  - Отправка `POST /api/v1/communications/send` с actor‑заголовком `"makosh-frontend"` и телом без явного подтверждения записи – проверяется, что возвращается ошибка или специальный статус (текст обрезан, но видно, что тест проверяет guard записи).

### `v1_communications_ai_state.rs`

Проверяет жизненный цикл AI‑состояний сообщений.

- `v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres`:
  - Создаётся спроецированное Email‑сообщение.
  - `GET …/api/v1/communications/messages/{message_id}/ai-state` изначально возвращает `ai_state: "NEW"`.
  - `PUT` с `ai_state: "PROCESSING"` меняет состояние, ответ содержит новый `ai_state` и `updated_at`.
  - В `observations` появляется запись с `kind_code: "COMMUNICATION_MESSAGE"`, `origin_kind: "manual"`, `payload.previous_ai_state: "NEW"`, `relationship_kind: "ai_state_transition"`.
  - В `communication_ai_states` строка с `ai_state: "PROCESSING"`, `review_reason: null`, `last_error: null`.
  - В `event_log` событие `mail.ai_state.changed` с `subject.message_id` и `payload.ai_state: "PROCESSING"`, `previous_ai_state: "NEW"`, без `body_text`.
  - Повторный GET возвращает `ai_state: "PROCESSING"`.

### `v1_communications_archive_inspection.rs`

Проверяет инспекцию ZIP‑архивов вложений.

- `v1_attachment_archive_inspection_reads_local_zip_blob_against_postgres`:
  - Создаётся почтовое сообщение с ZIP‑вложением (`evidence.zip`), содержащим два файла.
  - `GET …/api/v1/communications/attachments/{attachment_id}/archive-inspection` возвращает:
    - `report.archive_kind: "zip"`
    - `report.entry_count: 2`
    - `report.total_uncompressed_bytes: 17`
    - `report.has_nested_archive: false`
    - `report.entries[0].normalized_path: "docs/readme.txt"`
    - `report.entries[1].normalized_path: "invoice.txt"`

### `v1_communications_attachment_preview.rs`

Проверяет предпросмотр вложений и блокировку вредоносных.

- `v1_attachment_preview_reads_bounded_local_text_blob_against_postgres`:
  - Текстовый файл `notes.txt` (23 байта, `text/plain`) → `preview_kind: "text"`, полный текст без обрезки, `max_preview_bytes: 65536`, `byte_count: 23`.
- `v1_attachment_preview_reads_bounded_local_image_blob_against_postgres`:
  - PNG file (8 байт) → `preview_kind: "image"`, `data_url` с base64‑представлением, `byte_count: 8`.
- `v1_attachment_preview_reads_bounded_local_pdf_blob_against_postgres`:
  - PDF‑файл (9 байт) → `preview_kind: "pdf"`, `data_url`, `max_preview_bytes: 16777216`.
- `v1_attachment_preview_rejects_malicious_attachment_metadata`:
  - Вложение со статусом сканирования `Malicious` → `400 BAD_REQUEST`, `error: "invalid_communication_query"`, сообщение `"attachment preview is blocked by attachment scan status"`.

## Тестовая инфраструктура

- Изоляция: каждый тест создаёт экземпляр `TestContext::new().await`, предоставляющий строку подключения к выделенной тестовой БД и пул соединений.
- Сборка приложения: `build_router_with_database(AppConfig, Database)` для интеграционных тестов; `build_router(AppConfig)` для тестов без БД.
- Фикстуры Telegram: endpoint `POST /api/v1/integrations/telegram/fixtures/accounts` создаёт тестовый аккаунт‑заглушку, `POST …/fixtures/messages` – сообщение.
- Вспомогательный модуль `telegram_support/mod.rs` предоставляет единый `LOCAL_API_TOKEN`, функции-хелперы для HTTP‑запросов, проверок capability, генерации уникальных ID.
- В тестах движков (`TimelineEngine`, `TrustEngine`) нет HTTP‑зависимости; они вызывают методы напрямую.
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `backend/tests/telegram_references_members.rs` | Тест `telegram_reference_routes_return_enriched_message_summaries`: создание фикстурных сообщений и связей reply/forward, проверка маршрутов `reply-chain` (поле `target_message_summary`) и `forward-chain` (поле `source_message_summary`, `forward_origin_sender_name`), учёт циклических связей. |
| `backend/tests/telegram_reply_forward_capability_gates.rs` | Тест `fixture_account_blocks_reply_and_forward_before_side_effects`: фикстурный аккаунт блокирует reply и forward с кодом `BAD_REQUEST`, сообщения в чате не увеличиваются, отсутствуют записи в `api_audit_log` и `event_log` для команд отправки, уже созданные сообщения сохраняются. |
| `backend/tests/telegram_runtime_lifecycle.rs` | Тесты: `telegram_fixture_runtime_status_can_start_account_actor` (жизненный цикл start/restart/stop, поля статуса, `api_audit_log`), `telegram_runtime_status_reports_tdlib_diagnostics_for_qr_authorized_user_accounts` (диагностика tdlib, `runtime_kind`, `tdjson_probe_error`, `runtime_blockers`). Частичное покрытие теста `telegram_account_lifecycle_lists_logs_out_and_removes_without_deleting_evidence`. |
| `backend/tests/telegram_search_pinning.rs` | Тесты: `telegram_dialog_search_returns_projected_chat_matches` (поиск диалогов по названию), `telegram_media_search_filters_by_free_text_query` (медиа‑поиск по тексту и полям `file_name`, `tdlib_file_id`, `local_path`), `telegram_pinned_messages_route_returns_projection_backed_items` (список закреплённых сообщений, сортировка). |
| `backend/tests/telegram_support/mod.rs` | Хелперы: константа `LOCAL_API_TOKEN`, `json_post_request_with_actor`, `get_request_with_token`, `delete_request_with_token`, `assert_capability_status`, `assert_ok`, `ingest_fixture_telegram_message`, `account_item`, `json_body`, `unique_suffix`, `vault_entropy_events`. |
| `backend/tests/telegram_topic_capability_gates.rs` | Тест `fixture_account_allows_topic_list_but_blocks_topic_writes`: список топиков доступен, запись (create, close) блокирована с `BAD_REQUEST`, команды не сохраняются в `telegram_provider_write_commands`. |
| `backend/tests/timeline_engine.rs` | Тесты: `bounded_entity_limit` (границы), валидация источника (reject unsourced / accept sourced), `period_summary` (сводка и валидация диапазона), `recency_signal` (последнее событие, возраст), `timeline_gaps` (обнаружение пробелов, валидация порога), `change_diff` (добавленные/удалённые события по источнику). Частичное покрытие теста крос‑доменного таймлайна. |
| `backend/tests/trust_engine.rs` | Тесты: `persona_compatibility_score_signal` (значения и обрезка), `source_reliability_signal` (поля сигнала и валидация пустого affected_source). |
| `backend/tests/v1_api.rs` | Тесты: `v1_status_returns_enabled_surfaces_against_postgres` (статус, surfaces), `v1_communications_message_detail_returns_attachment_metadata_against_postgres` (детали сообщения с вложением и полями storage), отказ при отсутствии/неверном токене, CORS‑preflight для локальных origin, поведение без базы данных (503). |
| `backend/tests/v1_communications_ai_state.rs` | Тест `v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres`: GET/PUT для AI‑state, persistence в `communication_ai_states`, observation и event‑log эмиссия. |
| `backend/tests/v1_communications_api.rs` | Макрос‑тесты для read‑endpoint’ов (smoke), write‑endpoint’ов (graceful error). Тесты: `v1_sync_settings_default_update_and_manual_sync_status_against_postgres` (настройки синхронизации, manual sync, full resync, события и observation), `v1_send_requires_explicit_provider_write_confirmation` (guard записи). |
| `backend/tests/v1_communications_archive_inspection.rs` | Тест `v1_attachment_archive_inspection_reads_local_zip_blob_against_postgres`: инспекция ZIP‑архива (archive_kind, entry_count, entries.normalized_path). |
| `backend/tests/v1_communications_attachment_preview.rs` | Тесты: текстовый, PNG, PDF‑превью (preview_kind, data_url, max_preview_bytes), отклонение Malicious‑вложения с сообщением об ошибке. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста видимых расхождений между кодом тестов и документацией или ADR нет. Тесты ссылаются на конкретные env‑переменные (`MAKOSH_TDJSON_PATH`) и значения (`LOCAL_API_TOKEN`), которые могут измениться при рефакторинге конфигурации, но подтвердить дрейф без дополнительной документации невозможно. Пути к фикстурным endpoint’ам (`/api/v1/integrations/telegram/fixtures/…`) и названия таблиц соответствуют только тому, что зафиксировано в тестах; гарантии их стабильности в других частях системы не предоставлены.
