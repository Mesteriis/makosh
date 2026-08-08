## Summary / Резюме

Добавляется страница `operations/backend-tests.md` в русскую Obsidian‑wiki. Страница описывает текущий набор тестов бэкенда на основе предоставленных исходных файлов: структуру тестов, тестовую инфраструктуру (временные базы данных, `TestContext`), охватываемые домены (graph API, graph projection, health, маршруты, архивная инспекция, почтовое хранилище, memory engine, сообщения), ключевые эндпоинты и проверяемые инварианты, а также архитектурные тесты на лимит строк.

## Proposed pages / Предлагаемые страницы

### `operations/backend-tests.md`

```markdown
# Тесты бекенда

## Обзор

Тесты бекенда расположены в каталоге `backend/tests` и используют асинхронный фреймворк `tokio` с макросом `#[tokio::test]`. Интеграционные тесты, требующие PostgreSQL, опираются на `testkit::context::TestContext`, который предоставляет изолированную тестовую базу данных для каждого сценария. Тесты разделены по доменам: Graph API, проекция графа, проверки работоспособности и маршрутов, инспекция почтовых архивов, почтовое хранилище, движок памяти (memory engine) и сообщения.

Общий подход:
- Тестовый контекст создаёт свежую базу данных через `live_graph_api_context` / `live_projection_context` и удаляет её при завершении (`cleanup()`).
- Для изоляции данных используется `unique_suffix()` (наносекунды системного времени).
- Взаимодействие с API выполняется через `axum::Router` с заголовком `x-makosh-secret` и заданным токеном.

## Graph API Tests

Тесты Graph API находятся в `graph_api/neighborhood.rs`, `graph_api/search.rs` и `graph_api/support.rs`.

### Лимиты и константы

- `EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT = 100`
- `EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT = 100`
- `LOCAL_API_TOKEN = "graph-api-test-token"`

### Окрестность узла (`/api/v1/graph/neighborhood`)

- Возвращает выбранный узел (`selected_node`), его соседей (`nodes`), рёбра (`edges`) и свидетельства (`evidence`).
- При превышении лимита рёбер устанавливает `truncated: true`, при превышении лимита свидетельств — `evidence_truncated: true`.
- Запрос без `node_id` возвращает `NOT_FOUND` с телом:
  ```json
  {"error":"graph_node_not_found","message":"graph node was not found"}
  ```
- Глубина `depth` принимает только значение `1`. Параметр `depth=2` вызывает `BAD_REQUEST`:
  ```json
  {"error":"invalid_graph_query","message":"depth supports only 1"}
  ```

### Сводка графа (`/api/v1/graph/summary`)

- На пустой базе возвращает пустые массивы `node_counts`, `edge_counts`, `evidence_count: 0`, `latest_projection_at: null`, `is_empty: true`.

### Список узлов (`/api/v1/graph/nodes`)

- Первыми возвращаются узлы, имеющие хотя бы одно ребро (connected picker nodes). Несвязанные узлы в выдачу не включаются.

### Поиск (`/api/v1/graph/search`)

- Возвращает узлы, соответствующие параметру `q`.
- Пустой запрос `q=` приводит к `BAD_REQUEST`:
  ```json
  {"error":"invalid_graph_query","message":"q must not be empty"}
  ```

## Graph Projection Tests

Тесты проекции графа содержатся в `graph_projection.rs` и подкаталоге `graph_projection/`.

### Идемпотентность (`graph_projection/idempotence.rs`)

- Два последовательных вызова `GraphProjectionService::project_from_v1()` возвращают одинаковые счётчики `nodes_upserted`, `edges_upserted`, `evidence_upserted`.
- После проекции неизвестный отправитель не создаёт узел `person` (count = 0 для узлов типа `person` по stable_key, содержащему email).
- При появлении известной персоны ребро заменяется: убирается связь с `email_address` и создаётся связь с `person`.

### Связи проектов (`graph_projection/project_links.rs`)

- Проекция создаёт связи проекта с:
  - сообщениями (`project_has_message`),
  - документами (`project_has_document`),
  - персонами (`project_involves_person`),
  - email-адресами (`project_involves_email_address`).
- По умолчанию для предложенных связей устанавливается `review_state: "suggested"` и `confidence: 0.75`.
- Отклонённые пользователем связи (`UserRejected`) не попадают в граф (количество link count = 0).
- Подтверждённые связи (`UserConfirmed`) получают `review_state: "user_confirmed"` и `confidence: 1.0`.

### Вспомогательные утилиты (`graph_projection/support.rs`)

- `LiveProjectionContext` содержит все хранилища: `PersonProjectionStore`, `CommunicationIngestionStore`, `MessageProjectionStore`, `DocumentImportStore`, `ProjectStore`, `GraphProjectionService`, `ProjectLinkReviewStore`.
- `assert_project_edge_with_evidence` проверяет в базе наличие ребра с заданными параметрами (review_state, confidence, source_kind, source_id, observation_id).
- `assert_unknown_email_endpoint_projected` проверяет, что узел `email_address` имеет ребро заданного типа и что нет узла `person`.
- `cleanup_project_graph_fixture` удаляет узел и проект после теста.

## Health & Routes

### Проверки здоровья (`health.rs`)

- `GET /healthz` всегда возвращает `200` с телом:
  ```json
  {"status":"ok","service":"makosh-backend"}
  ```
- `GET /readyz` без сконфигурированной базы данных возвращает `503` и `"status": "degraded"`. Проверки `database` и `migrations` имеют статус `"not_configured"`.
- С базой данных `readyz` возвращает `200`, статус `"ok"`, сообщения `"database is reachable"` и `"required database migrations are applied"`.

### Версионные маршруты (`hard_v1_routes.rs`)

- Бывшие версионные пути `/api/v2/tasks`, `/api/v3/ai/status`, `/api/v4/capabilities`, `/api/v5/capabilities` возвращают `NOT_FOUND`.
- Конечные точки `/api/v1/integrations/telegram/capabilities` и `/api/v1/integrations/whatsapp/capabilities` возвращают `200` с ключами `version`, `runtime_mode`, `planned_features`, `capabilities`. Для WhatsApp дополнительно присутствует `provider_shapes`. Оба ответа содержат capability `runtime.fixture`.

## Mail Archive Inspection

Тесты в `mail_archive_inspection.rs` проверяют функцию `inspect_zip_bytes`:

- Корректный архив: возвращается `entry_count`, `total_uncompressed_bytes`, `entries[].normalized_path`, `entries[].uncompressed_size`. Флаг `has_nested_archive` выставляется в `false`.
- Архив с относительным путём `../secret.txt` вызывает ошибку `ArchiveInspectionError::UnsafeEntryPath`.
- Превышение `max_uncompressed_bytes` (например, лимит 4 при реальном размере 5) вызывает ошибку `UncompressedSizeExceeded`.

## Mail Storage

Тесты в `mail_storage.rs` проверяют:

- `LocalCommunicationBlobStore` записывает контентно-адресуемые блобы (sha256) с дедупликацией по содержимому. Хранит `storage_kind: "local_fs"`, `size_bytes`, `sha256`, относительный `storage_path`.
- `CommunicationStorageStore` сохраняет метаданные вложения через `NewCommunicationAttachment`: поля `message_id`, `raw_record_id`, `blob_id`, `part_id`, `content_type`, `size_bytes`, `sha256`, `filename`, `disposition` (`CommunicationAttachmentDisposition::Attachment`), `scan_status` (по умолчанию `AttachmentSafetyScanStatus::NotScanned`), `scan_metadata` (пустой объект).
- Попытка записать блоб с путём `../outside.blob` приводит к ошибке `CommunicationStorageError::UnsafeStoragePath`.

## Memory Engine

Тесты в `memory_engine.rs` (модульные) покрывают `MemoryEngine`:

- **Заметки персоны**: `persona_notes_memory_card` создаёт черновик `MemoryCardDraft` с заголовком `"Compatibility notes"`, уверенностью `1.0` и важностью `5`. Пустые / пробельные заметки возвращают `None`.
- **Факты персон**: `persona_fact_memory` создаёт `MemoryFactDraft`, требуя непустой `source`. Состояние ревью — `"accepted"`. Пустой источник вызывает ошибку `"memory source must not be empty"`.
- **Контекстный пакет**: `context_pack` собирает факты и карты по заданной сущности, считает среднюю уверенность и собирает `source_citations`.
- **Пробелы памяти**: `memory_gaps` находит отсутствующие типы фактов из переданного списка желаемых типов, создавая элементы с `review_state: "suggested"`.
- **Устаревшие факты**: `stale_memory_candidates` выявляет факты, у которых `last_verified_at` старше заданного порога (например, 90 дней), помечая их `review_state: "suggested"`.
- **Междоменный контекст**: `cross_domain_context_pack` объединяет факты из связанных сущностей, отбрасывая элементы с `review_state: "user_rejected"`.

## Message Tests

### Проекция сообщений (`messages/projection_core.rs`)

- `project_raw_email_message` создаёт каноническое сообщение с полями `subject`, `sender`, `recipients`, `body_text`.
- `project_raw_email_message_from_blob` парсит MIME-заголовки (Subject, From, To, Content-Type, Content-Transfer-Encoding) из блоба, сохранённого через `LocalCommunicationBlobStore`, и экстрагирует `body_text` (quoted-printable декодируется).
- Проекция различает аккаунты с делимитерами в идентификаторе (например, `base_account_id` и `base_account_id:left`), порождая разные `message_id`.
- Повторная проекция одного и того же `raw_record` идемпотентна — возвращает тот же `message_id`.
- `upsert_message` при прямом вставлении (с произвольным `message_id`) генерирует канонический `message_id` с префиксом `"msg:v1:"`.

### Запросы и фильтрация (`messages/projection_queries.rs`)

- `list_messages` фильтрует по `account_id`, `WorkflowState`, `channel_kind`, поисковому запросу и `LocalMessageState`.
- Канал `"telegram"` является алиасом, объединяющим сообщения с `channel_kind = "telegram_user"` и `"telegram_bot"`.
- `move_to_local_trash` и `restore_from_local_trash` изменяют `LocalMessageState` между `Active` и `Trash`. Повторная проекция сохраняет состояние корзины. Сообщения в корзине не возвращаются при запросе `LocalMessageState::Active`.
- Поиск поддерживает `MessageSearchMatchMode::All`.

### AI‑анализ (`messages/analysis.rs`)

- `set_ai_analysis` записывает `ai_category`, `ai_summary` и `importance_score` (целое число 0–100). Значение `101` отклоняется (ошибка).
- `EmailAnalyticsStore::mailbox_health` возвращает `total_messages` и `average_importance`.
- `top_senders` возвращает список отправителей с `avg_importance`.

### Флаги сообщений (`message_flags_api.rs`)

- `POST /api/v1/communications/messages/{message_id}/important` переключает булев флаг `important` в `message_metadata`.
- Каждое переключение создаёт:
  - `observation` с `origin_kind: "manual"` и payload, содержащим `operation: "message_important_toggle"` и `message_id`.
  - `observation_link` с `relationship_kind: "message_flag_update"`.

## Architecture Tests

Файлы `graph_api_architecture.rs` и `graph_projection_architecture.rs` проверяют, что тестовые файлы в соответствующих каталогах не превышают лимит в 700 строк. При нарушении тест падает с перечислением файлов и количеством строк. Проверка охватывает все файлы с расширением `.rs` внутри директорий `graph_api` и `graph_projection`, а также файлы с именами `graph_api.rs` и `graph_projection.rs`.

## Тестовые утилиты

- `testkit::context::TestContext` — управление временной тестовой базой PostgreSQL.
- `live_graph_api_context` / `live_projection_context` — создание изолированного контекста с маршрутизатором и хранилищами.
- `unique_suffix()` — генерация уникального суффикса из `SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()` для уникальности тестовых данных.
- `json_body` — десериализация тела ответа axum в `serde_json::Value`.
```

## Source coverage / Покрытие источников

| Исходный файл | Факты, покрытые на странице |
|---|---|
| `backend/tests/graph_api/neighborhood.rs` | эндпоинт `/api/v1/graph/neighborhood`, структура ответа, лимиты `EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT` и `_EVIDENCE_LIMIT` (100), флаги `truncated`/`evidence_truncated`, обработка отсутствующего `node_id` (NOT_FOUND), ограничение `depth=1` (BAD_REQUEST) |
| `backend/tests/graph_api/search.rs` | эндпоинты `/api/v1/graph/summary` (пустое состояние), `/api/v1/graph/nodes` (connected picker), `/api/v1/graph/search` (поиск узлов, отклонение пустого запроса) |
| `backend/tests/graph_api/support.rs` | константы `LOCAL_API_TOKEN`, лимиты, структура `LiveGraphApiContext`, создание/удаление тестовых баз, хелперы `config_with_api_token`, `get_request_with_token`, `json_body`, `unique_suffix` |
| `backend/tests/graph_api_architecture.rs` | архитектурный тест на лимит 700 строк для файлов в `graph_api` |
| `backend/tests/graph_projection.rs` | модульная структура тестов проекции |
| `backend/tests/graph_projection/idempotence.rs` | идемпотентность `project_from_v1`, отсутствие узла `person` для неизвестного отправителя, замена рёбер при появлении известной персоны |
| `backend/tests/graph_projection/project_links.rs` | связи проектов (типы рёбер), состояния `suggested`/`user_confirmed`, уверенность 0.75/1.0, пропуск `UserRejected` связей |
| `backend/tests/graph_projection/support.rs` (обрезан) | `LiveProjectionContext`, `assert_project_edge_with_evidence`, `assert_unknown_email_endpoint_projected`, `cleanup_project_graph_fixture` |
| `backend/tests/graph_projection_architecture.rs` | архитектурный тест на лимит 700 строк для файлов в `graph_projection` |
| `backend/tests/hard_v1_routes.rs` | 404 для `/api/v2..v5`, эндпоинты `/api/v1/integrations/{telegram,whatsapp}/capabilities`, структура ответа (version, runtime_mode, planned_features, capabilities, provider_shapes) |
| `backend/tests/health.rs` | `/healthz` ответ, `/readyz` без и с базой данных (статусы not_configured / ok) |
| `backend/tests/mail_archive_inspection.rs` | `inspect_zip_bytes`, безопасный парсинг метаданных, `UnsafeEntryPath`, `UncompressedSizeExceeded` |
| `backend/tests/mail_storage.rs` | `LocalCommunicationBlobStore` (контентная адресация, дедупликация), `CommunicationStorageStore` (вложения, scan_status `NotScanned`), `UnsafeStoragePath` |
| `backend/tests/memory_engine.rs` (обрезан) | `persona_notes_memory_card` (заголовок, пустые заметки), `persona_fact_memory` (требование источника), `context_pack` (средняя уверенность, цитирования), `memory_gaps`, `stale_memory_candidates`, `cross_domain_context_pack` |
| `backend/tests/message_flags_api.rs` | `POST .../important`, переключение флага, создание observation и observation_link с `message_flag_update` |
| `backend/tests/messages.rs` | модульная декомпозиция тестов сообщений |
| `backend/tests/messages/analysis.rs` | `set_ai_analysis` (ограничение score), `mailbox_health`, `top_senders` |
| `backend/tests/messages/projection_core.rs` | `project_raw_email_message`, `project_raw_email_message_from_blob`, делимитеры в аккаунтах, идемпотентность, генерация `message_id` с префиксом `"msg:v1:"` |
| `backend/tests/messages/projection_queries.rs` (обрезан) | `list_messages` (фильтры), алиас канала `"telegram"`, корзина (`move_to_local_trash`/`restore_from_local_trash`), `MessageSearchMatchMode::All` |
| `backend/tests/messages/support.rs` | `live_projection_context`, `store_provider_account`, `record_raw_email_message`, `disconnected_message_store`, `unique_suffix` |

## Drift candidates / Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом тестов, документацией или ADR. Контекст не содержит содержимого существующих wiki-страниц или ADR, поэтому дрифт не может быть подтверждён или опровергнут.
