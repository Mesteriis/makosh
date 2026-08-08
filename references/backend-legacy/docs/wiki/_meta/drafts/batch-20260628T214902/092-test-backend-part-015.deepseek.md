### Summary / Резюме

Создать страницу `operations/backend-tests.md` в русской Obsidian wiki проекта makosh. В неё войдёт описание структуры интеграционных тестов бэкенда, используемых паттернов, а также обзор каждого тестового файла/модуля, основанный исключительно на предоставленных исходниках. Это закроет пробел в документации: ранее wiki не описывала покрытие и контракты тестов V1 Communications API.

### Proposed pages / Предлагаемые страницы

#### `operations/backend-tests.md`

```markdown
# Тестирование бэкенда

Директория `backend/tests/` содержит интеграционные тесты API v1. Тесты работают с изолированной базой PostgreSQL через `TestContext`.

## Инфраструктура тестов

- База данных: `TestContext::new().await` предоставляет временный инстанс PostgreSQL.
- Роутер: `build_router_with_database`, конфигурация — `testkit::app::config_with_secret_and_database_url`.
- Аутентификация: HTTP-заголовок `x-makosh-secret` с тестовым значением (определяется константой в каждом файле, например `"v1comms-attachment-search-test-token"`).
- Проверка состояния БД: прямые SQL-запросы через `sqlx::query`, `sqlx::query_scalar`, `sqlx::query_scalar` и методы `Row::try_get`.
- Искусственные AI-зависимости: Fake-сервер Ollama поднимается как `axum::Router` на случайном порту; его URL записывается в настройку `ai.ollama_base_url` через `ApplicationSettingsStore`.
- Общий вспомогательный модуль: `v1_communications_regressions/support.rs` (хелперы `get`, `post`, `delete`, `post_with_actor`, `uid`, `response_json`, `router`, функции посева сообщений).

## Паттерн теста

1. Инициализация: `let context = TestContext::new().await; let pool = context.pool().clone();`
2. Уникальный суффикс: `let suffix = uid();` (наносекунды с UNIX-эпохи).
3. Посев данных: использование доменных хранилищ (`CommunicationIngestionStore`, `MessageProjectionStore`, `CommunicationStorageStore` и др.).
4. Получение роутера: `let app = router(&context.connection_string()).await;`
5. Отправка HTTP-запроса: `app.oneshot(request)`.
6. Проверка ответа: статус, JSON-тело (`serde_json::Value`).
7. Опциональная проверка побочных эффектов: таблицы `event_log`, `observation_links`, `communication_messages`, `communication_outbox` и др.

## Обзор тестовых файлов

### `v1_communications_attachment_search.rs`

Тест: `v1_attachment_search_filters_and_paginates_metadata_against_postgres`

- Эндпоинт: `GET /api/v1/communications/attachments/search`
- Параметры запроса: `account_id`, `q`, `limit`, `cursor`, `content_type`, `scan_status`
- Проверяет постраничную навигацию (курсор), фильтрацию по подстроке (`q`), MIME-типу (`content_type`) и статусу сканирования (`scan_status`)
- Ожидаемые поля ответа: `filename`, `message_id`, `message_subject`, `storage_kind`, `storage_path`
- При `limit=1` и наличии >1 результата `has_more` = `true`, поле `next_cursor` — непустая строка
- Курсор, возвращённый первым запросом, передаётся во втором; второй возвращает оставшиеся элементы и `has_more` = `false`
- При посеве создаются сообщения с вложениями, имеющими разные статусы сканирования (`NotScanned`, `Failed`)

### `v1_communications_attachment_translation.rs`

Тесты:

- `v1_attachment_translation_uses_provided_extracted_text_against_postgres`
- `v1_attachment_translation_emits_signal_hub_ai_events_against_postgres`
- `v1_attachment_translation_rejects_empty_source_text_against_postgres`

- Эндпоинт: `POST /api/v1/communications/attachments/{attachment_id}/translate`
- Тело запроса: `{ "target_language": "en", "source_text": "..." }`
- Без настроенного AI‑рантайма: `"translated": false`, `"reason": "translation runtime unavailable"`, `"source": "caller_provided_extracted_text"`
- Пустой `source_text` (только пробелы) → статус `400`
- При работающем fake‑Ollama: `"translated": true`, и в `event_log` появляются события `signal.raw.ai.attachment_translation.observed` и `signal.accepted.ai.attachment_translation` с `subject.attachment_id` равным ID вложения

### `v1_communications_bilingual_reply_flow.rs`

Тесты:

- `v1_bilingual_reply_flow_returns_review_contract_against_postgres`
- `v1_bilingual_reply_flow_rejects_unsupported_tone_against_postgres`
- `v1_bilingual_reply_flow_emits_signal_hub_ai_events_when_runtime_runs`

- Эндпоинт: `POST /api/v1/communications/messages/{message_id}/bilingual-reply-flow`
- Тело запроса: `{ "reply_text_ru": "...", "tone": "business" }`
- Ответ без AI: `"send_ready": false`, вложенный `translation` и `back_translation` имеют `"translated": false`, `"reason": "translation runtime unavailable"`
- Тон `casual` не поддерживается → статус `400`, телом `"error": "invalid_communication_query"`, `"message": "unsupported bilingual reply tone"`
- При работающем fake‑Ollama: `"send_ready": true`, оба перевода `"translated": true`, и в `event_log` появляются 4 сигнала — по паре `observed`/`accepted` для `bilingual_reply_inbound_translation` и `bilingual_reply_back_translation`

### `v1_communications_folders.rs`

Тест: `v1_custom_folders_copy_move_and_events_against_postgres`

- CRUD папок:
  - Создание: `POST /api/v1/communications/folders` с полями `name`, `description`, `account_id`, `color`, `sort_order`; возвращает `folder_id` с префиксом `mail_folder:`, `name`, `message_count`
  - Обновление: `PUT /api/v1/communications/folders/{folder_id}` — изменяет `name`, `color`
  - Удаление: `DELETE /api/v1/communications/folders/{folder_id}` → `{ "deleted": true }`
- Операции над сообщениями в папках:
  - Копирование: `POST /api/v1/communications/folders/{folder_id}/messages/{message_id}/copy` → `{ "operation": "copy", "folder_id": ..., "message_id": ... }`
  - Перемещение: `POST .../{message_id}/move` → `{ "operation": "move", "folder_id": ... }`
- Список папок: `GET /api/v1/communications/folders?account_id=...`
- Сообщения папки: `GET /api/v1/communications/folders/{folder_id}/messages`
- Проверка событий: 6 записей в `event_log` для папок; в `observation_links` — связи `folder_create`, `folder_update`, `folder_delete` для папок и `copy`/`move` для сообщений; `origin_kind` наблюдений — `"manual"`

### `v1_communications_message_actions.rs`

- Макро‑генерируемые smoke‑тесты (`v1_send`, `v1_reply`, …, `v1_message_analyze`) проверяют, что POST-запросы к фиктивному `msg:fake` на эндпоинты `send`, `reply`, `reply-all`, `forward`, `forward-eml`, `redirect`, `imap-mark-read`, `imap-delete`, `translate`, `ai-reply`, `ai-reply-variants`, `extract-tasks`, `extract-notes`, `analyze` не возвращают 5xx-ошибок
- `v1_message_analyze_returns_structured_ai_summary_against_postgres`:
  - Эндпоинт `POST /api/v1/communications/messages/{message_id}/analyze` (тело пустое)
  - Ответ содержит `summary_contract` с полями `key_points`, `action_items`, `risks`, `deadlines`, `event_candidates`, `persona_candidates`, `organization_candidates`, `document_candidates`, `agreement_candidates`
  - Результат сохраняется в `communication_messages.message_metadata` → `ai_summary_contract`
  - Создаются `review_items` типа `knowledge_candidate` с группами `agreement` и `document`
- `v1_bulk_actions_mark_read_and_trash_messages_against_postgres`:
  - `POST /api/v1/communications/messages/bulk-actions` с `{ "action": "mark_read", "message_ids": [...] }`
  - Ответ: `{ "action": "mark_read", "matched_count": 2, "updated_count": 2, "not_found": [] }`
  - `workflow_state` сообщений становится `"reviewed"`
  - В `event_log` событие `mail.message.read`, в `observation_links` — связи `workflow_state_transition` с метаданными `workflow_state: "reviewed"`

### `v1_communications_read_receipts.rs`

- `v1_read_receipt_records_correlation_and_realtime_event_against_postgres`:
  - `POST /api/v1/communications/read-receipts` с полями `account_id`, `provider_message_id`, `recipient`, `read_at`, `source_kind` (`"mdn"`), `provider_record_id`, `metadata`
  - Ответ: `receipt_kind: "read"`, `source_kind: "mdn"`, `outbox_id` найден
  - Запись сохраняется в `communication_read_receipts`
  - Событие `mail.read_receipt.recorded` в `event_log`
  - Связь в `observation_links`: `entity_kind = "read_receipt"`, `relationship_kind = "read_receipt_recorded"`, `metadata.receipt_kind = "read"`, `origin_kind = "local_runtime"`, `payload.operation = "read_receipt_recorded"`
- `v1_outbox_list_includes_latest_read_receipt_summary_against_postgres`:
  - После записи уведомления `GET /api/v1/communications/outbox?account_id=...&status=sent` включает `metadata.latest_read_receipt` с `receipt_kind`, `source_kind`, `read_at` (без `recipient` и `provider_record_id`)
- `v1_provider_delivery_event_records_delivery_status_against_postgres`:
  - `POST /api/v1/integrations/mail/provider-delivery-events` с `{ "event_kind": "delivered", "source_kind": "gmail_history", ... }`
  - Ответ: `notification_kind: "delivery_status"`, `delivery_status: "delivered"`, `outbox_id`
  - Метаданные outbox обновляются полем `delivery_status`
  - Связь в `observation_links`: `entity_kind = "outbox_item"`, `relationship_kind = "delivery_status_observed"`, `metadata.delivery_status = "delivered"`, `origin_kind = "local_runtime"`, `payload.operation = "delivery_status_recorded"`

### `v1_communications_regressions.rs`

Модульная точка входа, объединяющая подмодули `analytics`, `drafts_outbox`, `messages_threads`, `support`.

### `v1_communications_regressions/analytics.rs`

- `v1_subscriptions_list_is_cursor_paginated_against_postgres` – курсорная пагинация `GET /api/v1/communications/subscriptions?account_id=...&limit=...`
- `v1_top_senders_list_is_cursor_paginated_against_postgres` – курсорная пагинация `GET /api/v1/communications/analytics/senders?account_id=...&limit=...`

### `v1_communications_regressions/drafts_outbox.rs`

- `v1_post_draft_allows_empty_subject_for_autosave_against_postgres` – `POST /api/v1/communications/drafts` с пустой темой (автосохранение), затем обновление и удаление; проверка событий `mail.draft.created`, `mail.draft.updated`, `mail.draft.deleted` и связей `draft_upsert`/`draft_delete` (`origin_kind = "manual"`, операции `"draft_create"`, `"draft_delete"`); тела событий **не** содержат `body_text` и `subject`
- `v1_drafts_list_is_cursor_paginated_against_postgres` – пагинация списка черновиков
- `v1_send_schedules_outbox_message_and_allows_undo_against_postgres` – `POST /api/v1/communications/send` с `scheduled_send_at` и `undo_send_seconds`; ответ: `transport: "outbox"`, `status: "scheduled"`, содержит `outbox_id`

### `v1_communications_regressions/messages_threads.rs`

- `v1_messages_list_uses_cursor_pagination_without_duplicates_against_postgres` – курсорная пагинация `GET /api/v1/communications/messages`, отсутствие дубликатов между страницами
- `v1_threads_list_uses_cursor_pagination_without_duplicates_against_postgres` – то же для `GET /api/v1/communications/threads`
- `v1_translate_thread_returns_per_message_fallbacks_against_postgres` – `POST /api/v1/communications/threads/translate?account_id=...&subject=...` (`{ "target_language": "en" }`) возвращает элементы с `message_id`, `original_language`, `translated` = `false`, непустым `reason`
- `v1_translate_thread_emits_signal_hub_ai_events_per_message` – с fake‑Ollama проверяет `translated: true` и 4 события в `event_log` типов `signal.raw.ai.thread_message_translation.observed` / `signal.accepted.ai.thread_message_translation`
- `v1_message_translate_returns_fallback_when_ai_source_is_muted` – при активной `SignalPolicy` с `mode: Muted` для `source: "ai"` эндпоинт `POST .../messages/{message_id}/translate` возвращает `translated: false` и `reason: "no LLM configured"`

### `v1_communications_regressions/support.rs`

Общие функции:

- `get(uri)` – GET‑запрос с заголовком `x-makosh-secret`
- `post(uri, body)` / `post_with_actor(uri, body)` – POST‑запросы с JSON‑телом
- `delete(uri)` – DELETE‑запрос
- `uid()` – генерация уникального суффикса (наносекунды)
- `response_json(response)` – извлечение JSON из ответа
- `router(database_url)` – создание роутера через `build_router_with_database`
- `seed_projected_message(pool, account_id, provider_record_id, subject)` – посев сообщения с дефолтным телом
- `seed_projected_message_with_body(pool, ...)` – посев с заданным телом
- `seed_projected_message_from_sender(pool, ..., sender, body_text)` – посев с конкретным отправителем

### `v1_communications_regressions_architecture.rs`

Проверяет, что ни один файл регрессионных тестов (включая файл‑объявления и файлы внутри директории `v1_communications_regressions`) не превышает `MAX_TEST_FILE_LINES = 700` строк. Рекурсивно обходит директорию тестов, подсчитывает строки `.rs`‑файлов и завершается `assert!` с перечнем нарушений.

### `v1_communications_saved_searches.rs`

- `v1_saved_searches_crud_and_events_against_postgres`:
  - CRUD: `POST` / `PUT` / `DELETE` на `/api/v1/communications/saved-searches`
  - При создании `saved_search_id` начинается с `mail_saved_search:`, возвращается `message_count`
  - Список с фильтром `smart_folder=true` и курсорной пагинацией (`limit`, `cursor`)
  - Проверка `event_log` (количество событий) и `observation_links`: связи `saved_search_upsert` (операция `"saved_search_create"`) и `saved_search_delete`, `origin_kind = "manual"`
- `v1_saved_search_counts_follow_rules_builder_match_semantics_against_postgres` – проверка семантики подсчёта сообщений для сохранённого поиска
```

### Source coverage / Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/tests/v1_communications_attachment_search.rs` | Постраничный поиск вложений с фильтрацией по `q`, `content_type`, `scan_status`; формат ответа (`items`, `has_more`, `next_cursor`, поля `filename`, `message_id`, `message_subject`, `storage_kind`, `storage_path`); использование `CommunicationIngestionStore`, `MessageProjectionStore`, `CommunicationStorageStore` для посева; эндпоинт `GET /api/v1/communications/attachments/search` |
| `backend/tests/v1_communications_attachment_translation.rs` | POST‑перевод вложения с `source_text`; fallback при отсутствии AI‑рантайма, отказ при пустом `source_text` (400); эмиссия signal‑событий `signal.raw.ai.attachment_translation.observed` и `signal.accepted.ai.attachment_translation` при работающем fake‑Ollama; настройка `ai.ollama_base_url` через `ApplicationSettingsStore`; использование fake‑Ollama сервера |
| `backend/tests/v1_communications_bilingual_reply_flow.rs` | POST‑bilingual‑reply‑flow с `reply_text_ru` и `tone`; fallback‑ответ без AI; отклонение неподдерживаемого тона (`casual` → 400, `invalid_communication_query`); полный путь с fake‑Ollama и 4 signal‑событиями (`inbound_translation` и `back_translation` по паре observed/accepted) |
| `backend/tests/v1_communications_folders.rs` | CRUD‑папок, copy‑move‑операций над сообщениями, пагинация сообщений папки; события (`event_log`) и observation‑связи (`folder_create`, `folder_update`, `folder_delete`, `copy`, `move`); `origin_kind = "manual"`; формат `folder_id` |
| `backend/tests/v1_communications_message_actions.rs` | Смоук‑тесты для основных эндпоинтов; структурированный анализ (`summary_contract`) с персистенсом в `message_metadata` и созданием `review_items`; bulk‑actions `mark_read` с переходом `workflow_state` в `reviewed`, событием `mail.message.read` и связями `workflow_state_transition`; использование макроса `v1_msg_post_test!` |
| `backend/tests/v1_communications_read_receipts.rs` | Запись read‑receipts через `POST /api/v1/communications/read-receipts`; персистенс в `communication_read_receipts`; события `mail.read_receipt.recorded` и observation‑связи `read_receipt_recorded` с `origin_kind = "local_runtime"`; включение последнего read‑receipt в outbox‑список; запись provider‑delivery‑events через `POST /api/v1/integrations/mail/provider-delivery-events` с observation‑связью `delivery_status_observed` |
| `backend/tests/v1_communications_regressions.rs` | Модульная структура: включение подмодулей `analytics`, `drafts_outbox`, `messages_threads`, `support` |
| `backend/tests/v1_communications_regressions/analytics.rs` | Курсорная пагинация эндпоинтов `subscriptions` и `analytics/senders` |
| `backend/tests/v1_communications_regressions/drafts_outbox.rs` | Автосохранение черновика с пустой темой, события `draft.*` и связи `draft_upsert`/`draft_delete` с redacted‑полями; пагинация списка черновиков; расписание отправки с `transport: "outbox"` и undo |
| `backend/tests/v1_communications_regressions/messages_threads.rs` | Курсорная пагинация messages и threads без дубликатов; перевод треда с fallback‑ами и AI‑сигналами; подавление AI‑перевода через SignalPolicy `Muted` для `source: "ai"` с результатом `reason: "no LLM configured"` |
| `backend/tests/v1_communications_regressions/support.rs` | Сигнатуры общих хелперов (`get`, `post`, `delete`, `post_with_actor`, `uid`, `response_json`, `router`, функции посева сообщений); используемая конфигурация роутера и токена |
| `backend/tests/v1_communications_regressions_architecture.rs` | Архитектурное правило: максимум 700 строк на файл регрессионных тестов, рекурсивная проверка |
| `backend/tests/v1_communications_saved_searches.rs` | CRUD сохранённых поисков с `saved_search_id` формата `mail_saved_search:`; пагинация с фильтром `smart_folder`; проверка `event_log` и observation‑связей `saved_search_upsert`/`saved_search_delete` с `origin_kind = "manual"` |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом тестов и документацией/ADR не видны. Тесты непротиворечиво описывают собственное поведение, и внешние артефакты (спецификации API, prod‑код, существующие wiki‑страницы) в контекст не включены — сравнение невозможно.
