---
chunk_id: 079-test-backend-part-002
batch_id: batch-20260628T214902
group: backend
role: test
source_status: pending
source_count: 23
generated_by: code-wiki-ru
---

# 079-test-backend-part-002 — backend/test

- Target index: [[operations/backend-tests]]
- Batch: `batch-20260628T214902`
- Source files: `23`

## Резюме

Предлагаю создать (или полностью заменить) wiki-страницу `operations/backend-tests.md`, документирующую организацию, покрытие и ключевые паттерны интеграционных тестов бэкенда Макошь. Текущий контекст не содержит существующей версии этой страницы, поэтому документ строится исключительно на основе встроенных исходных файлов тестов. Цель — дать обзор доменов, протестированных на уровне сервисов и HTTP API, структуры каталога тестов и используемой тестовой инфраструктуры.

## Предложенные страницы

#### `operations/backend-tests.md`

```markdown
# Бэкенд-тесты Макошь

## Обзор

Интеграционные тесты бэкенда покрывают следующие домены:
- AI-контрольный центр и дымовой тест AI-моделей
- Календарь (доменный слой и HTTP API)
- Коммуникации (характеризационные тесты текущего поведения)
- Персоны / Peronas (характеризационные тесты)
- Звонки (Calls API)
- Ингестия коммуникационных данных (контракты провайдеров, сохранение чекпоинтов)
- Архитектурные тесты на соблюдение лимитов строк

Тесты используют `testkit::context::TestContext` для поднятия изолированной PostgreSQL БД (вероятно, через testcontainers), подключаются через `Database::connect`, а для API-тестов аутентифицируются заголовком `x-makosh-secret`.

## Структура тестов

Файлы расположены в `backend/tests/`. Основные группы:

- `ai_control_center.rs` — AI Control Center
- `ai_smoke.rs` — дымовой тест Ollama
- `calendar/` — тесты доменного слоя календаря
- `calendar_api/` — тесты HTTP API календаря
- `calls_api.rs` — Calls API
- `characterization_communication.rs` — характеризационные тесты Communications API
- `characterization_person.rs` — характеризационные тесты Person / Persona API
- `communication_ingestion/` — тесты ингестии (контракты, аккаунты, чекпоинты)
- `calendar_architecture.rs` — проверка лимита строк для файлов домена календаря
- `calendar_api_architecture.rs` — проверка лимита строк для файлов calendar_api

## AI и контрольный центр

### `ai_control_center.rs`

- Эндпоинты `/api/v1/ai/settings/overview`, `/api/v1/ai/providers`, `/api/v1/ai/models`, `/api/v1/ai/prompts` возвращают `503 SERVICE_UNAVAILABLE` и `{"error":"database_not_configured"}`, если база данных не подключена (тесты `*_without_database`).
- Эндпоинты на запись (POST провайдеров, PATCH, тестирование, синхронизация моделей, consent, PUT маршрутов, POST промптов, версий, активации) также возвращают `503` при отсутствии БД.
- Для удалённых API-провайдеров (тип `api`) обязательно наличие привязанного host-vault секрета перед использованием в private context (маршрут модели или тестирование промпта) — иначе возвращается ошибка `"host-vault API key"` (`remote_api_provider_models_require_host_vault_secret_before_private_context_use`).
- После создания секретной ссылки (`secret:ai-provider:...:api_key`) и привязки (`bind_api_key_secret`) удалённый API-провайдер успешно принимает маршрут модели (`remote_api_provider_model_route_accepts_host_vault_secret_binding`).
- non-API провайдеры (`built_in`) отклоняют привязку API-ключа (`non_api_provider_rejects_api_key_secret_binding`) и изменение consent (`non_api_provider_consent_mutation_is_rejected`).
- (Файл обрезан, часть тестов может быть не видна.)

### `ai_smoke.rs`

- Опциональный живой дымовой тест с Ollama. Зависит от переменных окружения:
  - `MAKOSH_OLLAMA_BASE_URL` — если не задана, тест пропускается.
  - `MAKOSH_OLLAMA_CHAT_MODEL` (по умолчанию `qwen3:4b`)
  - `MAKOSH_OLLAMA_EMBED_MODEL` (по умолчанию `qwen3-embedding:4b`)
  - `MAKOSH_OLLAMA_TIMEOUT_SECONDS` (по умолчанию 120)
- Проверяет версию Ollama, наличие chat и embedding моделей в списке `tags`, успешный chat-ответ с токеном `makosh-ai-smoke-ok` и размер embedding-вектора 2560.

## Календарь (доменный слой)

### `calendar/account_event.rs`

- `CalendarAccountStore`: CRUD аккаунтов (create, get, update, list, delete). ID начинается с `cal:v1:`.
- `CalendarSourceStore`: создание источника с привязкой к аккаунту, список источников по аккаунту.
- `CalendarEventStore`: создание события через `NewCalendarEvent`, событие получает `event_id` (`evt:v1:…`) и `observation_id` (`observation:v1:…`), статус по умолчанию `scheduled`, тип — `CALENDAR_EVENT`. Поддерживаются get, update, delete, list с временным диапазоном.
- `reschedule` меняет статус на `rescheduled`; `set_status` может установить `cancelled`.

### `calendar/event_context.rs`

- `EventParticipantStore`: добавление участника (email, роль), список участников.
- `EventRelationStore`: связывание события с внешней сущностью (`link`), список связей.
- `EventContextPackStore`: upsert контекстного пакета (summary, documents, tasks, вопросы, риски и т.д.).
- `EventAgendaStore`: установка повестки с указанием источника (`manual`).
- `EventChecklistStore`: установка чеклиста с указанием источника.

### `calendar/intelligence_sync.rs`

- `CalendarIntelligenceService::classify_event`: по названию, количеству участников и длительности возвращает один из типов: `meeting`, `deadline`, `focus`, `travel`, `personal`, `planning`.
- `calculate_importance`: учитывает ключевые слова (`URGENT`), количество участников, флаги.
- `calculate_readiness`: от 0.0 до 1.0 в зависимости от наличия компонентов (повестка, контекст и т.д.).
- `detect_risks`: возвращает список рисков при отсутствии подготовки.
- `CalendarWatchtowerService`: методы `weekly_brief`, `events_needing_preparation`, `events_without_outcomes`, `meeting_load_analysis` корректно выполняются с пустой/тестовой БД.
- `CalendarBrainService`: методы `answer` и `search_events` также выполняются без ошибок.
- Функции `export_event_ics` и `export_event_md` генерируют iCalendar и Markdown представления события.
- Создание хранилищ (EventParticipantStore, EventRelationStore, EventContextPackStore, EventAgendaStore, EventChecklistStore, DeadlineStore, FocusBlockStore, CalendarRuleStore) успешно выполняется с `disconnected_pool()` (ленивое подключение).

### `calendar/meeting_outcomes.rs`

- `MeetingNoteStore`: создание заметок к событию с форматом и источником.
- `MeetingOutcomeStore`: добавление исходов (тип `decision` или `promise`).
- `CalendarMeetingOutcomeApplicationService::add_manual` для исхода `decision` создаёт связанный `Decision` в домене решений с состоянием `Suggested`, evidence-запись с quote и review item типа `potential_decision`, зеркалированную из `decisions`.
- Для исхода `promise` создаёт `Obligation` с состоянием `Suggested`, датой выполнения, evidence-записью и review item `potential_obligation` из `obligations`. При этом связи с задачей (`obligation_task_links`) не создаётся (количество 0).

### `calendar/scheduling_rules.rs`

- `DeadlineStore`: создание дедлайна с severity и статусом `active`, список дедлайнов.
- `FocusBlockStore`: создание фокус-блоков с уровнем защиты `high`, список блоков с фильтрацией.
- `CalendarRuleStore`: CRUD правил с ID `rule:v1:…`, approval_mode `suggest_only`, список и удаление.

## Календарь (HTTP API)

Тесты используют вспомогательные функции из `calendar_api/support.rs`: `build_cal_app`, `create_cal_event`, хелперы запросов с `x-makosh-secret`, `unique_suffix`, `urlencoding_percent_encode`.

### `calendar_api/auth.rs`

- Обращение к `/api/v1/calendar/accounts` и `/api/v1/calendar/events` без заголовка `x-makosh-secret` возвращает `403 FORBIDDEN` с телом `{"error":"invalid_api_secret","message":"missing or invalid x-makosh-secret header"}`.

### `calendar_api/accounts.rs`

- CRUD аккаунтов через API: POST создаёт аккаунт с `provider`, `account_name`, `email`; возвращает `account_id`.
- После создания, обновления и удаления в БД проверяется наличие observation-связей (`observation_links`) с соответствующими `relationship_kind` (`create`, `update`, `delete`).
- Тип observation: `CALENDAR_ACCOUNT_MUTATION`, origin_kind: `manual`.
- GET `/api/v1/calendar/accounts` возвращает список с полем `items`.

### `calendar_api/events.rs` (обрезан)

- CRUD событий: создание (с привязкой к аккаунту), получение, обновление, удаление.
- `POST .../reschedule` с новым `start_at`/`end_at` и `POST .../cancel` успешно выполняются.
- Участники: добавление через POST, получение списка; источник участника содержит observation и manual origin.
- Проверка append-only observations: каждое изменение события (создание, обновление заголовка, перенос) приводит к новому `observation_id` в строке события. Статус после создания — `confirmed`, после переноса — `rescheduled` (дальнейшее обрезано).

### `calendar_api/event_details.rs` (обрезан)

- GET эндпоинты для подресурсов события (`relations`, `context-pack`, `agenda`, `checklist`, `risks`, `notes`, `outcomes`, `recording`, `transcript`, `brief`, `export`, `reminders`) возвращают не-серверные ошибки и корректные JSON-тела (например, массивы `items`).
- Макрос `cal_post_test!` генерирует тесты для POST-эндпоинтов тех же подресурсов с тестовыми телами, проверяя отсутствие серверных ошибок.
- `reminder toggle`: создание напоминания с источником `observation:…`, переключение через `POST .../toggle` с проверкой observation-записи (origin_kind=`manual`) и связи `observation_links`.
- `materials capture`: создание agenda, checklist, meeting note с `source: "manual"` приводит к сохранению их с source `observation:…`.

### `calendar_api/misc.rs` (обрезан)

- GET-эндпоинты: `/api/v1/calendar/deadlines`, `/focus-blocks`, `/watchtower`, `/health`, `/weekly-brief`, `/search`, `/rules`, `/analytics/distribution` возвращают не-серверные ошибки (пустые списки или ok).
- Создание deadline и focus block через POST с последующей проверкой observation-связей (origin_kind=`manual`).
- POST `/api/v1/calendar/smart-schedule` успешно принимает тело задачи.
- CRUD правил через API: создание, обновление, удаление; проверка observation-связей для create и update.
- Работа с источниками календаря: POST к аккаунту создаёт source, проверяется observation.

## Коммуникации и персоны (характеризационные тесты)

### `characterization_communication.rs`

- Фиксирует текущее поведение Communications API (фаза 3+ выравнивания):
  - `GET /api/v1/communications/messages` возвращает 200, тело содержит `items` или массив.
  - `GET /api/v1/communications/search?q=test` возвращает 200.
  - `GET /api/v1/communications/threads` возвращает 200.
  - `GET /api/v1/communications/messages/states` возвращает 200.
  - `GET /api/v1/communications/drafts` возвращает 200.
  - `GET /api/v1/communications/messages/rec:nonexistent` возвращает 404.
  - `POST /api/v1/workflow-actions` с валидным телом принимает либо 200, либо 4xx.

### `characterization_person.rs`

- Фиксирует текущее поведение Person / Persona API (фаза 2+ выравнивания):
  - `GET /api/v1/persons` возвращает 200, поле `items`, дефолтный лимит ≤ 50.
  - `GET /api/v1/personas` возвращает 200 с `items`.
  - Оба эндпоинта сосуществуют.
  - `GET /api/v1/persons/owner` возвращает 200 и поле `owner_persona` (может быть null или объект с `person_id`, `is_self`).
  - `GET /api/v1/persons/search` без параметра `q` возвращает 4xx.
  - `GET /api/v1/persons/person:nonexistent` — 404.
  - `PUT /api/v1/personas/persona:nonexistent` — 4xx.

## Звонки (Calls API)

### `calls_api.rs`

- Запрос без `x-makosh-secret` к `/api/v1/calls` — `403 FORBIDDEN`.
- `GET /api/v1/calls` с токеном возвращает не-серверную ошибку, тело содержит `items`.
- `POST /api/v1/calls` с полями `call_type`, `chat_id`, `direction`, `state`, `initiated_at`, `duration_seconds` создаёт запись без серверной ошибки.
- `GET /api/v1/calls/call:nonexistent-.../transcript` возвращает либо `404`, либо успех.

## Ингестия коммуникационных данных

### `communication_ingestion/contracts.rs`

- `EmailProviderKind`: поддерживаются `gmail`, `icloud`, `imap`; `exchange` не поддерживается.
- `ProviderAccountSecretPurpose`: `OauthToken` принимает только `SecretKind::OauthToken`; `ImapPassword` и `SmtpPassword` принимают `Password` и `AppPassword`, но не `OauthToken`.
- `CommunicationIngestionStore::upsert_provider_account` сохраняет учётные записи провайдеров (Gmail, iCloud, IMAP) с конфигурацией.
- `save_checkpoint` создаёт и обновляет чекпоинты ингестии с полями `last_seen_uid`.
- `checkpoint` читает сохранённый чекпоинт.

## Архитектурные тесты

### `calendar_architecture.rs`

- Все тестовые файлы в папке `calendar/` и сам `calendar.rs` не должны превышать 700 строк.
- Исключения: файл с большим количеством строк вызывает панику с сообщением о нарушении лимита.

### `calendar_api_architecture.rs`

- Все тестовые файлы внутри `calendar_api/` не должны превышать 700 строк. Аналогичный механизм проверки.

## Инфраструктура и паттерны

- **Тестовый контекст**: `testkit::context::TestContext` предоставляет строку подключения к временной БД (вероятно testcontainers), автоматически применяет миграции.
- **Observation-трекинг**: множество тестов проверяют, что мутации (создание, обновление, удаление) порождают observation-записи с `origin_kind = "manual"` и соответствующие связи `observation_links`.
- **Аутентификация API**: заголовок `x-makosh-secret` с токеном, заданным в конфигурации (`testkit::app::config_with_secret`).
- **Отключённый пул**: для тестов, не требующих реального подключения, используется `disconnected_pool()` (lazy connect к несуществующему хосту).

## Ограничения контекста

Некоторые исходные файлы были обрезаны при встраивании (лимит 12000 символов):
- `ai_control_center.rs` — конец файла не виден.
- `calendar_api/event_details.rs` — часть POST-тестов и проверок materials capture не полностью доступна.
- `calendar_api/events.rs` — окончание проверок lifecycle observations отсутствует.
- `calendar_api/misc.rs` — часть проверок (например, удаление правил) не видна.

Поэтому представленное описание может быть неполным для этих файлов.
```

## Покрытие источников

- `backend/tests/ai_control_center.rs` — существование эндпоинтов без БД (`503`), требование host-vault секрета для удалённых API-провайдеров, привязка секрета, отклонение consent/secret для built-in провайдеров (обрезано).
- `backend/tests/ai_smoke.rs` — дымовой тест Ollama (env-переменные, модели qwen3:4b и qwen3-embedding:4b, chat и embedding проверки).
- `backend/tests/calendar.rs` — декларация субмодулей календарных тестов.
- `backend/tests/calendar/account_event.rs` — CRUD аккаунтов, источников, событий; reschedule/статус.
- `backend/tests/calendar/event_context.rs` — участники, связи, контекстные пакеты, повестка, чеклисты.
- `backend/tests/calendar/intelligence_sync.rs` — classify/importance/readiness/risks, health/brain сервисы, ICS/Markdown экспорт, disconnected pool.
- `backend/tests/calendar/meeting_outcomes.rs` — заметки, исходы, зеркало Decision/Obligation с review items.
- `backend/tests/calendar/scheduling_rules.rs` — дедлайны, фокус-блоки, правила.
- `backend/tests/calendar/support.rs` — хелперы unique_suffix, live_pool, disconnected_pool.
- `backend/tests/calendar_api.rs` — декларация субмодулей calendar_api.
- `backend/tests/calendar_api/accounts.rs` — CRUD аккаунтов через API, observation-связи.
- `backend/tests/calendar_api/auth.rs` — отказ без заголовка секрета (403).
- `backend/tests/calendar_api/event_details.rs` — GET/POST подресурсов события, reminder toggle, materials capture (обрезано).
- `backend/tests/calendar_api/events.rs` — CRUD, reschedule, cancel, participants, lifecycle observations (обрезано).
- `backend/tests/calendar_api/misc.rs` — эндпоинты планировщика, дедлайны/фокус-блоки/правила/аналитика, smart-schedule, источники (обрезано).
- `backend/tests/calendar_api/support.rs` — вспомогательные функции для API-тестов.
- `backend/tests/calendar_api_architecture.rs` — лимит 700 строк для calendar_api тестов.
- `backend/tests/calendar_architecture.rs` — лимит 700 строк для календарных тестов.
- `backend/tests/calls_api.rs` — auth, list, create, transcript 404.
- `backend/tests/characterization_communication.rs` — характеризация Communications API (messages, search, threads, states, drafts, message by id, workflow-actions).
- `backend/tests/characterization_person.rs` — характеризация Person/Persona API (persons, personas, owner, search, update).
- `backend/tests/communication_ingestion.rs` — декларация субмодулей.
- `backend/tests/communication_ingestion/contracts.rs` — provider kinds, secret purposes, upsert аккаунтов, checkpoints.

## Исходные файлы

- [`backend/tests/ai_control_center.rs`](../../../../backend/tests/ai_control_center.rs)
- [`backend/tests/ai_smoke.rs`](../../../../backend/tests/ai_smoke.rs)
- [`backend/tests/calendar.rs`](../../../../backend/tests/calendar.rs)
- [`backend/tests/calendar/account_event.rs`](../../../../backend/tests/calendar/account_event.rs)
- [`backend/tests/calendar/event_context.rs`](../../../../backend/tests/calendar/event_context.rs)
- [`backend/tests/calendar/intelligence_sync.rs`](../../../../backend/tests/calendar/intelligence_sync.rs)
- [`backend/tests/calendar/meeting_outcomes.rs`](../../../../backend/tests/calendar/meeting_outcomes.rs)
- [`backend/tests/calendar/scheduling_rules.rs`](../../../../backend/tests/calendar/scheduling_rules.rs)
- [`backend/tests/calendar/support.rs`](../../../../backend/tests/calendar/support.rs)
- [`backend/tests/calendar_api.rs`](../../../../backend/tests/calendar_api.rs)
- [`backend/tests/calendar_api/accounts.rs`](../../../../backend/tests/calendar_api/accounts.rs)
- [`backend/tests/calendar_api/auth.rs`](../../../../backend/tests/calendar_api/auth.rs)
- [`backend/tests/calendar_api/event_details.rs`](../../../../backend/tests/calendar_api/event_details.rs)
- [`backend/tests/calendar_api/events.rs`](../../../../backend/tests/calendar_api/events.rs)
- [`backend/tests/calendar_api/misc.rs`](../../../../backend/tests/calendar_api/misc.rs)
- [`backend/tests/calendar_api/support.rs`](../../../../backend/tests/calendar_api/support.rs)
- [`backend/tests/calendar_api_architecture.rs`](../../../../backend/tests/calendar_api_architecture.rs)
- [`backend/tests/calendar_architecture.rs`](../../../../backend/tests/calendar_architecture.rs)
- [`backend/tests/calls_api.rs`](../../../../backend/tests/calls_api.rs)
- [`backend/tests/characterization_communication.rs`](../../../../backend/tests/characterization_communication.rs)
- [`backend/tests/characterization_person.rs`](../../../../backend/tests/characterization_person.rs)
- [`backend/tests/communication_ingestion.rs`](../../../../backend/tests/communication_ingestion.rs)
- [`backend/tests/communication_ingestion/contracts.rs`](../../../../backend/tests/communication_ingestion/contracts.rs)

## Кандидаты на drift

На основе предоставленного контекста расхождений между кодом тестов и документацией не выявлено — встроенные файлы являются исходным кодом, а целевая wiki-страница отсутствует в контексте для сравнения.
