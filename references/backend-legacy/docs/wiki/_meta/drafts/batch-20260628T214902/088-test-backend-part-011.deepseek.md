### Summary / Резюме

Создаётся страница `operations/backend-tests.md` на русском языке, документирующая структуру и покрытие тестов backend‑директории. Страница описывает тестовые модули (Signal Hub, Task Candidates, Tasks, хранилище, Telegram, снапшоты, архитектурные ограничения), ключевые тестовые сценарии, используемую инфраструктуру (TestContext, роутер, API‑токены) и отмечает файлы, предоставленные в обрезанном виде.

### Proposed pages / Предлагаемые страницы

#### `operations/backend-tests.md`

```markdown
# Тесты backend

## Обзор

Набор тестов для backend расположен в `backend/tests/`. Тесты используют
`testkit::context::TestContext` для создания временной изолированной базы данных
PostgreSQL. Все тесты асинхронные (`#[tokio::test]`). Тесты API используют
`axum::Router` и тестовый HTTP-клиент.

Основные проверяемые домены:
- **Signal Hub** — обработка сырых сигналов, политики, fixture, API.
- **Task Candidates** — генерация, ревью, подтверждение, обязательства, API.
- **Tasks** — CRUD, provenance, обновления, чеклисты, evidence, API.
- **Хранилище** — readiness при отсутствии URL и неполных миграциях.
- **Telegram** — типы провайдеров и секретов.
- **Снапшоты** — стабильность сериализации payload событий.
- **Архитектурные ограничения** — лимит строк в тестовых файлах.

---

## Signal Hub

### `backend/tests/signal_hub.rs`

Тесты доменной логики Signal Hub.

- `signal_hub_restores_canonical_sources_idempotently` — проверяет, что повторное
  восстановление системных источников (14 sources, 4 профиля) не создаёт дубликатов
  (`second.sources_created == 0`). Проверяет коды источников (ai, browser, ...) и
  свойства источника `telegram` (default_enabled, supports_connections, etc.).

- `signal_policy_evaluator_applies_reject_pause_mute_allow_order` — тестирует
  приоритет политик (`SignalPolicyEvaluator`): `Disabled` (event pattern) > `Paused`
  (source) > `Muted` (global). Просроченные политики игнорируются.

- `event_store_queries_signal_events_by_type_source_subject_correlation_and_time` —
  проверяет фильтрацию событий в `EventStore` по `event_type`, `source_code`,
  subject kind/entity_id, `correlation_id` и временному интервалу.

- `signal_hub_accepts_raw_signal_when_no_policy_blocks_it` — `SignalHubSignalService`
  принимает сырой сигнал, если политики не блокируют, и публикует событие
  `signal.accepted.telegram.message` с корректным `causation_id` и payload.

- `signal_hub_pause_policy_buffers_raw_signal_without_accepted_publication` —
  создаётся политика `Paused`, затем сырой сигнал обрабатывается; ожидается
  буферизация без публикации accepted-события. *(Файл обрезан после 12000 символов,
  полное поведение не подтверждено.)*

### `backend/tests/signal_hub_api.rs`

Тесты HTTP API Signal Hub.

- `signal_hub_api_restores_fixture_and_lists_sources` — комплексный тест:
  восстанавливает фикстуры, проверяет список источников (`GET /api/v1/signal-hub/sources`),
  эмитит fixture `fixture_basic_message`, проверяет список фикстур и их свойства,
  список профилей, применение профиля `testing` (is_active: true), список
  capabilities (degraded), отключение источника (`/sources/telegram/disable`)
  переводит capabilities в `blocked`, создание/обновление/удаление пользовательского
  профиля `quiet_hours`, а также отвержение обновления системного профиля (400).

- `signal_hub_connect_api_requires_local_api_secret` — запрос к Connect-роуту
  `/makosh.signal_hub.v1.SignalHubService/ListSources` без секрета возвращает
  403 (`invalid_api_secret`), с секретом (через `post_json`) — 200.

- `signal_hub_api_runs_ai_health_check_against_runtime_status` — проверяет
  health-check AI с тестовым Ollama URL `http://127.0.0.1:9`. *(Файл обрезан,
  полное поведение не подтверждено.)*

---

## Task Candidates

### `backend/tests/task_candidates/refresh.rs`

Тесты генерации кандидатов (refresh).

- `task_candidate_refresh_creates_message_and_document_candidates_against_postgres` —
  после `refresh_deterministic_candidates` создаются записи в `task_candidates` для
  сообщения и документа с `review_state = "suggested"`, `source_kind = "observation"`
  и `observation_id`.

- `task_candidate_refresh_uses_obligation_engine_for_message_commitments_against_postgres` —
  сообщение с фразой «I will ... by Friday 5pm.» порождает obligation-кандидата
  (`candidate_kind = "obligation_task"`) с заголовком-утверждением, due_text и
  confidence > 0.7. При этом задачи и обязательства не создаются автоматически
  (task_count = 0, obligation_count = 0).

- Аналогичный тест для документов: `task_candidate_refresh_uses_obligation_engine_for_document_commitments_against_postgres`.

- `task_candidate_refresh_updates_existing_source_title_candidate_against_postgres` —
  обновление существующего кандидата (UPSERT) по source/title без ошибок
  дублирования ключа.

### `backend/tests/task_candidates/review.rs`

Тесты ревью кандидатов.

- `task_candidate_review_confirm_creates_active_task_against_postgres` — после
  `set_review_state(UserConfirmed)` создаётся задача (`tasks`) с
  `provenance_kind = "observation"`, `source_kind = "observation"` и
  наблюдение с kind `COMMUNICATION_MESSAGE`.

- `task_candidate_store_review_with_observation_materializes_transition_link_against_postgres` —
  `set_review_state_with_observation` создаёт связь `review_transition` в
  `observation_links` с метаданными `review_state` и `event_id`.

- `task_candidate_review_confirm_materializes_obligation_candidate_against_postgres` —
  подтверждение obligation-кандидата создаёт задачу и связанное обязательство
  (`obligation_task_links`) с evidence (quote, observation_id) и persona-ссылкой.

- `obligation_task_candidate_reset_demotes_obligation_review_state_against_postgres` —
  сброс состояния obligation-кандидата понижает review_state обязательства.
  *(Файл обрезан, полное поведение не подтверждено.)*

### `backend/tests/task_candidates/event_replay.rs`

Тест перестроения состояния через события.

- `task_candidate_review_event_rebuilds_state_against_postgres` —
  применяет события `task_candidate_review_state_changed` (UserConfirmed, UserRejected)
  через `apply_review_event`, проверяя, что итоговый `review_state` = "user_rejected"
  и `event_id` соответствует последнему событию.

### `backend/tests/task_candidates/support.rs`

Вспомогательный модуль для тестов кандидатов. Предоставляет:

- `live_task_candidate_context()` — создаёт `TaskCandidateTestContext` с пулом,
  `TaskCandidateStore` и `EventStore` из `TestContext`.
- `seed_message()` — записывает провайдерский аккаунт (Gmail), сырую запись и
  проецирует сообщение через `project_raw_email_message`.
- `seed_document()` — импортирует Markdown-документ.
- `build_review_event()` — строит `NewEventEnvelope` события
  `task_candidate_review_state_changed`.
- `unique_suffix()` — генерирует наносекундный суффикс.

### `backend/tests/task_candidates_api.rs`

Тесты HTTP API кандидатов.

- `task_candidates_reject_missing_local_api_secret` — запрос без `x-makosh-secret`
  получает 403 с ошибкой `invalid_api_secret`.

- `task_candidates_returns_safe_candidate_payload` — после генерации кандидатов
  эндпоинт `GET /api/v1/task-candidates` возвращает `source_kind`, `observation_id`,
  `evidence_excerpt`, **не** возвращает `candidate_kind` и `candidate_metadata`.

- `put_task_candidate_review_confirms_task_with_observation_trail` —
  `PUT /api/v1/task-candidates/{id}/review` с `user_confirmed` создаёт:
  запись `review_transition` в `observation_links`, наблюдение с `origin_kind="manual"`
  и `review_item` со статусом `promoted` и `target_entity_kind="task"`.

- `put_task_candidate_review_rejects_missing_candidate` — 404 для несуществующего
  кандидата с ошибкой `task_candidate_not_found`.

### `backend/tests/task_candidates_architecture.rs`

Архитектурный тест: все файлы в каталоге `task_candidates/` (включая сам
`task_candidates.rs` и `task_candidates_architecture.rs`) не должны превышать
`700` строк (`MAX_TEST_FILE_LINES`). Если лимит нарушен, тест падает с перечнем
файлов и количеством строк.

---

## Tasks

### `backend/tests/tasks.rs`

Тесты домена задач.

- `task_crud_against_postgres` — создание, чтение, обновление (статус, приоритет),
  архивация задачи. Проверяет `task_id` (префикс `task:v1:`), provenance,
  source_type, `completed_at`.

- `task_manual_creation_materializes_explicit_observation_provenance_against_postgres` —
  создание задачи без явного provenance автоматически захватывает observation типа
  `TASK_MUTATION` с полем `captured_from: "task_create"`.

- `task_store_update_with_observation_materializes_task_link_against_postgres` —
  обновление с observation создаёт связь `task_update` в `observation_links`.

- `task_creation_rejects_missing_review_item_provenance_against_postgres` —
  создание задачи с `provenance_kind="review_item"` и несуществующим
  `provenance_id` возвращает ошибку.

- `task_creation_rejects_missing_observation_provenance_against_postgres` —
  аналогично для отсутствующего observation.

- `task_creation_from_explicit_observation_provenance_uses_observation_source_against_postgres` —
  при явном указании observation provenance поля source заполняются из observation.

- `task_creation_rejects_missing_decision_provenance_against_postgres` —
  отвергается создание с отсутствующим decision provenance.

- `task_creation_rejects_missing_obligation_provenance_against_postgres` —
  отвергается создание с отсутствующим obligation provenance.

*(Файл обрезан после 12000 символов; часть тестов не включена.)*

### `backend/tests/tasks_api/`

Набор тестов API задач, разнесённый по файлам.

**`auth.rs`** — `tasks_rejects_missing_local_api_secret`: запрос без
`x-makosh-secret` возвращает 403 `invalid_api_secret`.

**`crud.rs`** — комплексные тесты:

- `tasks_crud_against_postgres` — создание, получение, обновление задачи,
  архивация; проверка `observation_links` для `task_update`.
- `tasks_list_returns_items` — `GET /api/v1/tasks` возвращает список.
- `task_status_transition` — `POST /api/v1/tasks/{id}/status` с `"completed"`
  создаёт связь `status_update` в `observation_links`.
- `task_analyze_runtime_path_captures_observation_against_postgres` —
  `POST /api/v1/tasks/{id}/analyze` создаёт связь `analysis_update`.
- `task_creation_rejects_unknown_review_item_reference_in_api` —
  создание с несуществующим `review_item:v1:does-not-exist` получает 400.
- `task_creation_rejects_decision_without_observation_evidence_in_api` —
  решение без observation evidence отвергается (400).
- `task_checklist_manual_create_path_captures_observation_against_postgres` —
  создание чеклиста захватывает observation `TASK_MUTATION` и связь в
  `observation_links`.
- `task_evidence_manual_create_path_captures_observation_against_postgres` —
  создание evidence захватывает observation. *(Файл обрезан, полное поведение не
  подтверждено.)*

**`mutations.rs`** — тесты мутаций:

- `task_rule_create_and_delete` — создание и удаление правила через API.
- Макрос `task_post_test!` прогоняет POST-запросы к:
  - `context-pack` (контекст-пак),
  - `evidence` (доказательство),
  - `relations` (связи задач),
  - `checklist` (чеклист),
  - `subtasks` (подзадачи).
- `task_post_provider` — создание провайдера задач.
- `task_candidate_review` — PUT-запрос ревью кандидата.

**`reads.rs`** — тесты GET-эндпоинтов:

- `task_context_pack_returns_ok`, `task_evidence_list_returns_empty`,
  `task_relations_list_returns_empty`, `task_checklist_list_returns_empty`,
  `task_subtasks_list_returns_empty`, `task_export_returns_text`,
  `task_external_returns_ok` — требуют созданной задачи.
- `task_providers_list_returns_ok`, `task_search_returns_ok`,
  `task_daily_brief_returns_ok`, `task_rules_list_returns_empty`,
  `task_templates_list_returns_ok`, `task_watchtower_returns_ok`,
  `task_health_returns_ok`, `task_analytics_returns_ok`,
  `task_candidates_list_returns_ok` — не требуют задачи.

**`support.rs`** — вспомогательные функции: `config_with_api_token`,
`test_database_url`, построение HTTP-запросов, `json_body`,
`urlencoding_percent_encode`, `unique_suffix`, `build_tasks_app`, `create_task`.

### `backend/tests/tasks_api_architecture.rs`

Архитектурный тест: все файлы в каталоге `tasks_api/` (включая `tasks_api.rs`
и `tasks_api_architecture.rs`) не должны превышать `700` строк.

---

## Хранилище

### `backend/tests/storage.rs`

- `database_without_url_reports_not_configured` — `Database::connect(None)`
  возвращает readiness `NotConfigured`.
- `database_without_url_reports_migrations_not_configured` — аналогично для
  миграций.
- `migration_readiness_rejects_missing_latest_migration_against_postgres` —
  удаляет запись о последней миграции из `_sqlx_migrations`, проверяет, что
  `migration_readiness()` возвращает `Unavailable` с сообщением
  `"required database migrations are incomplete"`. Требует, чтобы версия миграции
  была >= 4 (actor identity migration). После теста запись восстанавливается.

---

## Telegram

### `backend/tests/telegram.rs`

- `telegram_provider_and_secret_kinds_are_account_scoped` — проверяет, что
  `CommunicationProviderKind::TelegramUser` и `TelegramBot` парсятся из строк,
  и что `ProviderAccountSecretPurpose` принимает только ожидаемые `SecretKind`
  (например, `TelegramApiHash` и `TelegramBotToken` принимают `ApiToken`,
  `TelegramSessionKey` принимает `PrivateKey`, `TelegramBotToken` отвергает
  `Password`).

---

## Снапшоты

### `backend/tests/snapshot_smoke.rs`

- `event_payload_snapshot_remains_stable` — сериализует тестовый payload
  `signal.accepted.telegram.message` и сравнивает с эталонным снапшотом
  (`insta::assert_snapshot!`). Снапшот хранится в файле
  `backend/tests/snapshots/snapshot_smoke__event_payload_snapshot_remains_stable.snap`
  и содержит поля `event_type`, `metadata`, `source`, `subject`.

---

## Архитектурные ограничения

Помимо тестов на лимит строк в `task_candidates_architecture.rs` и
`tasks_api_architecture.rs`, общий принцип — каждый файл тестов не должен
превышать 700 строк для указанных групп тестов. Это проверяется рекурсивным
обходом каталогов и подсчётом строк в исходниках Rust.

---

## Инфраструктура тестов

### TestContext

`testkit::context::TestContext` предоставляет временную базу данных PostgreSQL
(через `connection_string()` и `pool()`). Тесты создают контекст в начале
асинхронной функции и используют его для инициализации хранилищ и сервисов.

### Роутер и HTTP-тесты

API-тесты собирают роутер через `build_router(config)` или
`build_router_with_database(config, database)`. Конфигурация включает тестовый
секрет (`x-makosh-secret`), передаваемый в заголовке. HTTP-запросы выполняются
через `tower::ServiceExt::oneshot`.

Тестовые константы:
- `LOCAL_API_TOKEN` в тестах task candidates и tasks API.

### Миграции

Тесты, использующие `Database::connect(Some(&url))`, полагаются на наличие
миграций, выполненных в тестовой базе. Тест `migration_readiness` проверяет
корректность детекта неполных миграций.

### Снапшот-тестирование

Используется библиотека `insta` для проверки стабильности сериализации JSON.

---

> **Примечание.** Часть файлов предоставлена с обрезанием после 12000 символов,
> поэтому данный документ может не отражать все тесты, присутствующие в этих
> файлах.
```

### Source coverage / Покрытие источников

- **`backend/tests/signal_hub.rs`** (обрезан) — покрыты тесты: идемпотентное восстановление канонических источников, приоритет политик, запросы событий по type/source/subject/correlation/time, приём сырого сигнала без блокирующих политик, буферизация при paused.
- **`backend/tests/signal_hub_api.rs`** (обрезан) — покрыты тесты: восстановление fixture и список источников, требование локального API-секрета в Connect-роуте, AI health-check.
- **`backend/tests/snapshot_smoke.rs`** — покрыт тест стабильности снапшота event payload.
- **`backend/tests/snapshots/snapshot_smoke__event_payload_snapshot_remains_stable.snap`** — содержание эталонного снапшота.
- **`backend/tests/storage.rs`** — покрыты тесты: readiness при отсутствии URL (NotConfigured), readiness миграций при отсутствии URL (NotConfigured), readiness миграций при отсутствии последней миграции (Unavailable).
- **`backend/tests/task_candidates.rs`** — объявления модулей `event_replay`, `refresh`, `review`, `support`.
- **`backend/tests/task_candidates/refresh.rs`** — покрыты тесты: создание message/document кандидатов, obligation engine для сообщений и документов, UPSERT существующего source-title кандидата.
- **`backend/tests/task_candidates/review.rs`** (обрезан) — покрыты тесты: подтверждение создаёт задачу, создание review transition observation link, подтверждение obligation-кандидата, сброс obligation состояния (частично).
- **`backend/tests/task_candidates/event_replay.rs`** — покрыт тест перестроения состояния кандидата через события.
- **`backend/tests/task_candidates/support.rs`** — покрыт вспомогательный код: `live_task_candidate_context`, `seed_message`, `seed_document`, `build_review_event`, `unique_suffix`.
- **`backend/tests/task_candidates_api.rs`** (обрезан) — покрыты тесты: авторизация, безопасная выдача полей кандидата, подтверждение с observation trail, 404 для несуществующего кандидата.
- **`backend/tests/task_candidates_architecture.rs`** — покрыт архитектурный тест лимита 700 строк для группы task_candidates.
- **`backend/tests/tasks.rs`** (обрезан) — покрыты тесты: crud, materialize observation provenance при ручном создании, update with observation link, отвержение отсутствующего review_item/observation/decision/obligation provenance, создание из явного observation provenance (частично).
- **`backend/tests/tasks_api.rs`** — объявления модулей `auth`, `crud`, `mutations`, `reads`, `support`.
- **`backend/tests/tasks_api/auth.rs`** — покрыт тест авторизации.
- **`backend/tests/tasks_api/crud.rs`** (обрезан) — покрыты тесты: crud, список задач, статус-переход, анализирование создаёт observation, отвержение неверного provenance в API, checklist/evidence manual create захватывает observation (частично).
- **`backend/tests/tasks_api/mutations.rs`** — покрыты тесты: create/delete rule, POST к context-pack/evidence/relations/checklist/subtasks, создание провайдера, candidate review.
- **`backend/tests/tasks_api/reads.rs`** — покрыты тесты GET-эндпоинтов: context-pack, evidence, relations, checklist, subtasks, export, external, providers, search, daily-brief, rules, templates, watchtower, health, analytics, candidates.
- **`backend/tests/tasks_api/support.rs`** — покрыт вспомогательный код: config, запросы, `json_body`, `unique_suffix`, `build_tasks_app`, `create_task`.
- **`backend/tests/tasks_api_architecture.rs`** — покрыт архитектурный тест лимита 700 строк для группы tasks_api.
- **`backend/tests/telegram.rs`** — покрыт тест парсинга `CommunicationProviderKind` и допустимых `SecretKind` для `ProviderAccountSecretPurpose`.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения не видны.
