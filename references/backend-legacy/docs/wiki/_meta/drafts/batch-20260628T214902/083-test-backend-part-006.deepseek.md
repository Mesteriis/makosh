### Summary / Резюме

Создать новую страницу wiki `operations/backend-tests.md` на русском языке. Страница документирует интеграционные тесты бэкенда: общие шаблоны тестирования, тесты событийной платформы (event store, consumers, outbox, API, SSE/WebSocket), тесты графовой платформы (GraphStore, Graph API), тесты коммуникаций (email sync pipeline, enrichment engine, Gmail outbox/send API). Все утверждения подтверждены встроенными исходными файлами тестов.

### Proposed pages / Предлагаемые страницы

#### `operations/backend-tests.md`

```markdown
# Тестирование бэкенда

Интеграционные тесты бэкенда находятся в `backend/tests/`. Они используют библиотеку `testkit` для создания изолированных тестовых окружений (PostgreSQL, NATS) и прямым подключением к временной базе данных через `Database::connect`.

## Общие шаблоны

- **Тестовый контекст**: `TestContext::new().await` предоставляет строку подключения к изолированной базе данных и URL NATS-сервера.
- **Уникальные суффиксы**: для изоляции параллельных тестов используется `SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()` как часть идентификаторов.
- **Конфигурация API**: для тестов API используется конфиг с фиксированным локальным секретом (`LOCAL_API_TOKEN`) через `testkit::app::config_with_secret`.
- **Прямые запросы к БД**: состояние после операций проверяется напрямую через `sqlx` к таблицам, гарантируя точность побочных эффектов.
- **Mock-серверы**: для внешних зависимостей (например, Gmail API) реализованы простые TCP-серверы, которые читают HTTP-запросы и возвращают заранее определённые ответы.
- **Vault**: для тестов с шифрованием (outbox delivery, send API) vault создаётся во временной директории с включённым dev-режимом.

## Событийная система

### Event Store (журнал событий)

- **Валидация конверта**: `NewEventEnvelope::builder` требует непустой `event_type` (пробельный — ошибка) и объектный `source` (строка — ошибка). Пустой `correlation_id` автоматически заменяется на `event_id` с обрезкой пробелов.
- **Контекст трассировки**: `TraceContext::root` создаёт корневой контекст с `correlation_id` и `causation_id: None`; `TraceContext::child_of` наследует `correlation_id` и устанавливает `causation_id` из `event_id` родителя.
- **Добавление и загрузка**: Событие сохраняется в `event_log`, загружается по `event_id`; все поля совпадают, кроме `recorded_at`. Дублирование события с теми же `event_type` и `source_id` источника приводит к ошибке (защита от дублирования). Прямые `UPDATE` в `event_log` блокируются — таблица append-only.
- **Реигрывание**: `list_after_position` возвращает события после заданной позиции с корректными `position` и `event`.
- **Трассировка**: `trace_by_event_id` собирает цепочку по `correlation_id` и `causation_id`, формируя `edges`, `root_event_ids`, `missing_parent_ids`, `orphan_event_ids`. `list_children` возвращает непосредственных потомков.

### Подтверждение обработки (Event Consumers)

- **Курсор и повторы**: Курсор не продвигается до успешной обработки. Счётчик попыток (`failure_attempt_count`) растёт при сбоях. При достижении `max_attempts` событие попадает в dead letter, курсор обновляется.
- **Dead letter**: Запись содержит `attempts`, `review_state` (`Open`), `position`. После запроса `request_dead_letter_replay` и вызова `replay_dead_letter`, `review_state` становится `Replayed`.
- **Идемпотентность**: Повторная обработка уже успешного события пропускается как дубликат без вызова обработчика. Даже после принудительного отката курсора маркер обработки не дублируется.

### Отправка событий из outbox

- **Формирование**: `append_for_dispatch` создаёт запись в `event_outbox` со статусом `pending` и `attempts = 0`.
- **In-Memory шина**: `InMemoryEventBus::broadcast` доставляет событие одному подписчику за вызов.
- **NATS JetStream**: `EventOutboxDispatcher` забирает pending-записи, публикует их в JetStream, обновляет статус на `published`, `attempts = 1`. При наличии `InMemoryEventBus` событие также рассылается в реальном времени.
- **Восстановление**: Записи, оставшиеся в статусе «dispatching» дольше допустимого времени (имитируется `updated_at - 5 minutes`), помечаются как восстановленные и публикуются.

### API событий

- **Защита**: Все эндпойнты (`/api/v1/events`, `/api/v1/events/{id}`, `/api/v1/audit/events`, `/api/events/stream`, `/api/events/ws`) требуют заголовок `x-makosh-secret`, совпадающий с локальным секретом. Неверный или отсутствующий секрет — `403 Forbidden`, ошибка `invalid_api_secret`.
- **Порядок проверок**: Проверка секрета выполняется до проверки actor и доступа к базе данных. При отсутствии БД возвращается `503 Service Unavailable`.
- **Валидация**: Невалидный конверт (пустой `event_type`) — `400 Bad Request`, ошибка `invalid_event_envelope`.
- **Round-trip**: POST создаёт событие, GET возвращает идентичные поля. Аудит (`api_audit_log`) фиксирует `event.append` и `event.get`. Прямые изменения `api_audit_log` блокируются.
- **Not Found**: GET несуществующего события — `404 Not Found`, аудит фиксирует `event.get`.
- **Long poll**: `GET /api/v1/events?after_position=...` возвращает пакет событий, `next_after_position`, `has_more`. Аудит записывает `event.list` с параметрами.
- **SSE**: `GET /api/events/stream?after_position=...` отдаёт события как `text/event-stream` с полями `id`, `event`, JSON-данными.
- **WebSocket**: `GET /api/events/ws?after_position=...&makosh_secret=...` возвращает `101 Switching Protocols`.
- **Trace API**: `GET /api/v1/events/{event_id}/trace` возвращает трассу с `edges`, `root_event_ids`, `missing_parent_ids`, `orphan_event_ids`. `GET /api/v1/events/{event_id}/children` — дочерние события.

## Графовая платформа

### Graph Store

- **Идемпотентность узлов**: Повторный `upsert_node` с теми же параметрами возвращает тот же `node_id`.
- **Идемпотентность рёбер**: `upsert_edge_with_evidence` при совпадении `edge` и `source_kind` + `source_id` доказательства обновляет `excerpt` и `metadata`, не создавая дубликатов.
- **Валидация**:
  - Рёбра без доказательств (включая `suggested`) отклоняются с `SystemEdgeRequiresEvidence`.
  - `confidence` вне диапазона [0.0, 1.0] вызывает `InvalidConfidence`.
  - Рёбра с установленным `valid_to` (закрытые темпоральные) вызывают `TemporalEdgesUnsupported`.
- **Детерминированные идентификаторы**: `edge_id` и `evidence_id` различают входные параметры даже при наличии разделителей внутри компонентов.

### Graph API

- **Аутентификация**: Эндпойнты `/api/v1/graph/summary`, `/search`, `/nodes`, `/neighborhood` защищены локальным секретом. Без него — `403 Forbidden`.
- **Приоритет**: Проверка секрета — до валидации параметров запроса (например, некорректный `depth` не проверяется при отсутствующем секрете).
- **Отсутствие БД**: Корректный секрет без БД — `503 Service Unavailable`.

## Коммуникации

### Email Sync Pipeline

Тест `email_sync_pipeline_records_raw_blob_and_projects_message_persons_against_postgres` проверяет:
- Сохранение учётной записи IMAP через `CommunicationIngestionStore::upsert_provider_account`.
- Проецирование сообщения из `EmailSyncBatch` с base64-закодированным RFC822-телом через `project_email_sync_batch_with_mail_blobs`.
- Запись в `communication_messages` (subject, sender, recipients, body_text).
- Создание персон, email-идентичностей (статус `active`, источник `email_sync`), observation-ссылок для персон, идентичностей, участников, событий отношений.
- Создание организаций, contact links, relationships (`member_of`).
- Запись сигнала `signal.accepted.mail.message` в `event_log`.
- Все проверки выполняются прямыми запросами через `sqlx`.

### Enrichment Engine

- `EnrichmentEngine::persona_favorite_preference` с `true` создаёт черновик предпочтения `ui:favorite`, `value: "true"`, `source` из ID сущности, `confidence: 1.0`. С `false` возвращается `None`.
- `EnrichmentEngine::persona_observation_candidate` создаёт кандидата с `entity_kind: "persona"`, `source`, `extracted_claim`, `review_state: "pending"`, `freshness: "current"`, `conflict_marker: false`. В `data` вкладывается `_enrichment` с `affected_entity_id` и `extracted_claim`. Пустой `source` отклоняется ошибкой `enrichment candidate source must not be empty`.

### Gmail Outbox Delivery

Тест `outbox_delivery_worker_sends_gmail_items_through_gmail_api_against_postgres` проверяет:
- Конфигурацию Gmail-аккаунта с OAuth-учётными данными в `HostVault`.
- Enqueue outbox-элемента со статусом `scheduled`.
- Delivery-воркер отправляет через `LiveGmailOutboxTransport` с вызовом mock Gmail API (`POST /gmail/v1/users/me/messages/send`, заголовок `Authorization: Bearer gmail-access-token`).
- Статус outbox-элемента становится `sent`, `provider_message_id` = `"gmail-api-message-id"`.

### Gmail Send API

Тест `gmail_send_api_queues_outbox_when_send_scope_enabled_against_postgres` проверяет:
- POST `/api/v1/communications/send` с `gmail_send_enabled: true` добавляет запись в `communication_outbox` со статусом `queued`, не вызывая Gmail API.
- Ответ: `transport: "outbox"`, `status: "queued"`, `accepted_recipients`, `outbox_id`, `message_id`.
- В базе создаётся observation_link типа `outbox_status_transition` с метаданными `operation: "outbox_enqueue"`.
```

### Source coverage / Покрытие источников

- **`backend/tests/email_sync_pipeline.rs`** (обрезан): покрывает полный пайплайн email-синхронизации: проецирование сообщения, создание персон, идентичностей, организаций, связей, observation-ссылок, сигнал `signal.accepted.mail.message`. Детали за пределами обрезанного участка не документированы.
- **`backend/tests/enrichment_engine.rs`** (полностью): покрывает создание черновиков избранного и кандидатов наблюдений для персон, валидацию пустого источника.
- **`backend/tests/event_consumers.rs`** (обрезан): покрывает управление курсором, dead letter, replay, идемпотентность дубликатов, эмиссию provider observation событий. Обрезан на этапе описания типов событий.
- **`backend/tests/event_log.rs`** (обрезан): покрывает валидацию конверта, контекст трассировки, append/load, replay, trace edges, блокировку изменений event_log. Обрезан внутри теста `event_store_reports_missing_trace_parent_against_postgres`.
- **`backend/tests/event_platform.rs`** (полностью): покрывает формирование outbox, in-memory шину, диспетчер в NATS JetStream, realtime-шину, восстановление зависших записей.
- **`backend/tests/events_api.rs`** (обрезан): покрывает защиту секретом, порядок проверок, валидацию, round-trip, not found, аудит. Обрезан внутри теста `get_audit_events_returns_records_without_self_auditing_against_postgres`.
- **`backend/tests/events_long_poll_api.rs`** (полностью): покрывает long poll листинг и аудит операции `event.list`.
- **`backend/tests/events_stream_api.rs`** (полностью): покрывает SSE-поток событий и trace/children API.
- **`backend/tests/events_websocket_api.rs`** (полностью): покрывает WebSocket upgrade с передачей секрета через query-параметр.
- **`backend/tests/gmail_outbox_delivery.rs`** (обрезан): покрывает outbox delivery через mock Gmail API, vault, авторизацию. Обрезан внутри реализации mock-сервера.
- **`backend/tests/gmail_send_api.rs`** (обрезан): покрывает API отправки и постановку в outbox. Обрезан внутри реализации mock-сервера.
- **`backend/tests/graph.rs`** (полностью): покрывает идемпотентность узлов и рёбер с доказательствами, валидацию (evidence, confidence, temporal), детерминированные идентификаторы.
- **`backend/tests/graph_api.rs`** (полностью, модульный файл): только декларации подмодулей; фактические тесты предоставлены только для `auth.rs`.
- **`backend/tests/graph_api/auth.rs`** (полностью): покрывает защиту графовых эндпойнтов локальным секретом, приоритет проверок, поведение без БД.

### Drift candidates / Кандидаты на drift

- Несколько файлов были обрезаны из-за ограничения на 12000 символов; оставшаяся часть тестов не видна в данном контексте, поэтому полное покрытие проверить невозможно. Потенциальный drift между документированными тестами и реальным поведением может существовать в обрезанных сегментах (например, дополнительные проверки в `email_sync_pipeline.rs`, `event_consumers.rs`, `event_log.rs`, `events_api.rs`, `gmail_outbox_delivery.rs`, `gmail_send_api.rs`).
- Модуль `backend/tests/graph_api` включает подмодули `neighborhood.rs`, `search.rs`, `support.rs`, не предоставленные в контексте; их фактическое поведение не задокументировано.
- Не подтверждено соответствие между тестами и существующими производственными доменами/хранилищами (например, таблицы БД, сигнатуры методов) — тесты могут ссылаться на код, который был изменён после их написания.
- Не проверено наличие других тестовых файлов вне данного чанка, которые могут документировать оставшиеся части бэкенда.
- Предоставленный контекст не содержит конфигурационных файлов тестовой инфраструктуры (testkit) или документации к ним; описания шаблонов тестов основаны исключительно на видимом коде.
