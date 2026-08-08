---
chunk_id: 023-source-backend-part-003
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 023-source-backend-part-003 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Обновляется страница `components/backend.md` русской Obsidian‑wiki: на основе предоставленных исходных файлов добавляется документация по ядру AI-сервиса (`AiService`), семантическому хранилищу эмбеддингов (`SemanticEmbeddingStore`) и модулям API‑поддержки (`app::api_support`). Все утверждения прямо подтверждены встроенным кодом.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend

## Обзор

Исходный код бэкенда организован в модули. Данный чанк покрывает ядро AI-сервиса (`ai::core`), семантическое хранилище эмбеддингов и модули API-поддержки (`app::api_support`).

## AI Service (AiService)

### Конструкторы

Модуль: `backend/src/ai/core/service/core.rs`

- `new_with_routing(pool, runtime, model_routing)` – принимает готовый `AiModelRouting` и строит `AiHub` поверх runtime-порта.
- `new_with_hub(pool, hub)` – принимает уже собранный `SharedAiHub`.
- `with_persona_attribution(persona_attribution)` – устанавливает порт атрибуции персон для AI-агентов.

### Маршрутизация моделей (AiModelRouting)

Модуль: `backend/src/ai/core/types.rs`

Структура `AiModelRouting` содержит отдельные поля для каждой роли:
- `default_chat`, `reasoning`, `summarization`, `mail_intelligence`, `reply_draft`, `extraction`, `meeting_prep`, `embeddings`.
- Маршруты больше не собираются через legacy fallback: каждый слот должен быть явно назначен в AI Hub settings, иначе запрос завершается ошибкой маршрутизации.

### Конфигурация моделей для записей о запусках

Модуль: `backend/src/ai/core/service/model_config.rs`

Метод `AiService::model_config` возвращает JSON с текущей конфигурацией: runtime, chat_model, embedding_model, embedding_dimension и объект `routes`, перечисляющий названия моделей для каждого маршрута.

### Атрибуция персон

Модули: `backend/src/ai/core/service/attribution_port.rs`, `backend/src/ai/core/service/attribution.rs`

- Порт `AiPersonaAttributionPort` (трейт) предоставляет методы:
  - `upsert_ai_agent_persona(agent_id, display_name)` – создаёт или получает персону агента, возвращает `AiAgentPersonaAttribution` (persona_id, persona_type, persona_email).
  - `owner_persona_id()` – возвращает ID персоны владельца (опционально).
- `AiService::run_attribution` вызывает upsert для агента (имя агента определяется через функцию `ai_agent_display_name`) и получает владельца. Если порт не задан, возвращает ошибку `AiError::PersonaAttributionUnavailable`.

### Поиск цитат (retrieval)

Модуль: `backend/src/ai/core/service/retrieval.rs`

Метод `retrieve_citations(query)` выполняет:
1. Создание `SemanticEmbeddingStore` и индексацию канонических источников (вызов `index_canonical_sources`).
2. Эмбеддинг запроса через `AiHub`, который резолвит слот `embeddings` и вызывает runtime-порт.
3. Проверку размерности эмбеддинга (должна совпадать с `AI_EMBEDDING_DIMENSION`).
4. Векторный поиск (`search`) и текстовый поиск (`text_search`) с лимитом `DEFAULT_RETRIEVAL_LIMIT`.
5. Объединение результатов (функция `merge_retrieval_results`), усечение до лимита и преобразование в `Vec<AiCitation>`.

### Жизненный цикл запусков (run lifecycle) и события

Модуль: `backend/src/ai/core/service/events.rs`

Каждый рабочий процесс (answer, meeting_prep, task_candidates) использует общий механизм:
- Создаётся запись в `AiRunStore` о начале запуска.
- Генерируется событие `ai.run.requested`.
- По завершении – событие `ai.run.completed`.
- При ошибке выполнения или маршрутизации фиксируется `ai.run.failed`.
- Для `refresh_task_candidates` дополнительно генерируется `ai.task_extraction.completed`.
- `append_run_event` пишет событие в `EventStore` с payload, содержащим preview запроса (до 160 символов) и детали, provenance (информация о рантайме и моделях), correlation_id.
- `append_ai_signal_event` диспатчит сигнал `dispatch_ai_runtime_signal` для событий с типами: `ai.run.requested` → сигнал `run_requested`, `ai.run.completed` → `run_completed`, `ai.run.failed` → `run_failed`, `ai.task_extraction.completed` → `task_extraction`. При этом из payload сигнала исключаются поля `query`, `answer`, `briefing`.
- Публичные HTTP-эндпоинты `/api/v1/ai/answers`, `/api/v1/ai/task-candidates/refresh`, `/api/v1/ai/meeting-prep` не возвращают итоговый payload синхронно: они отвечают `202 Accepted` с `AiHubRequestAcceptedResponse { request_id, run_id, status, event_id, correlation_id }`, пишут `ai.hub.requested`, а итог результата читается через `GET /api/v1/ai/runs/{run_id}`.
- `POST /api/v1/ai/model-downloads` обслуживает явный lifecycle скачивания встроенных Ollama-моделей: пишет `ai.hub.model_download.requested`, затем `ai.hub.model_download.completed` или `ai.hub.model_download.failed`. Успех только переводит модель в `is_available = true`, но не создаёт `ai_model_routes`.

### Рабочий процесс Answer

Модуль: `backend/src/ai/core/service/answer.rs`

- Агент по умолчанию: `MNEMOSYNE`.
- Входные параметры: `command_id`, `query`, опционально `agent_id`, `causation_id`, `correlation_id`.
- Порядок действий:
  1. Валидация `command_id`, `query`, `agent_id`.
  2. Запуск run, атрибуция персон.
  3. Извлечение цитат (`retrieve_citations`).
  4. Формирование промпта (`answer_prompt`) и вызов chat-модели через `AiHub`.
  5. Завершение run или перевод в failed run и запись событий.
- Внутренний результат сервиса описывается `AiAnswerResponse`, но наружу public API возвращает accepted-ответ и требует читать итог через `AiRun`.

### Рабочий процесс Meeting Prep

Модуль: `backend/src/ai/core/service/meeting_prep.rs`

- Агент: `HESTIA`.
- Входные параметры: `command_id`, `topic`, опционально `project_id`, `person_id`, `causation_id`, `correlation_id`.
- Формируется скоупированный запрос (`scoped_meeting_query`) с учётом `project_id`/`person_id`.
- Используется модель `self.model_routing.meeting_prep` для чата.
- Промпт генерируется через `meeting_prep_prompt`.
- Внутренний результат сервиса описывается `AiMeetingPrepResponse`, где вместо `answer` используется `briefing`, но public API отдаёт только accepted-ответ и хранит итог в run-проекции.

### Рабочий процесс обновления кандидатов задач (Task Candidates)

Модуль: `backend/src/ai/core/service/task_candidates.rs`

- Агент: `MAKOSH`.
- Входные параметры: `command_id`, `query`, опционально `causation_id`, `correlation_id`.
- Используется модель `self.model_routing.extraction`.
- После получения цитат генерируется промпт `task_candidate_prompt`, ответ парсится в черновики кандидатов (`parse_task_candidate_drafts`).
- Черновики сохраняются через `upsert_ai_task_candidates`.
- Вызывается синхронизация с инбоксом ревью (`sync_ai_run_task_candidates_to_review`).
- Внутренний результат сервиса описывается `AiTaskCandidateRefreshResponse` с `created_count`, но public API отдаёт accepted-ответ и требует читать durable run/result отдельно.

### Сохранение кандидатов задач в БД

Модуль: `backend/src/ai/core/service/task_candidate_persistence.rs`

Метод `upsert_ai_task_candidates(run_id, drafts, citations)`:
- Для каждого черновика ищет связанную цитату (по `source_kind` и excerpt), фильтрует только цитаты с kind `message` или `document`.
- Определяет `observation_id`: для `message` ищет `observation_id` в таблице `communication_messages` по `message_id`; для `document` – в таблице `documents` по `document_id`. Если не найден – пропускает кандидата.
- Выполняет upsert в таблицу `task_candidates`:
  - `task_candidate_id` генерируется функцией `ai_task_candidate_id`.
  - Поля: `source_kind = "observation"`, `source_id = observation_id`, `project_id = NULL`, `title`, `due_text`, `assignee_label`, `confidence` (clamp 0.0–1.0), `review_state = 'suggested'`, `evidence_excerpt`, `agent_run_id = run_id`.
  - При конфликте (по `source_kind, source_id, lower(title)`) обновляются поля, но `review_state` сохраняется, если он `user_confirmed` или `user_rejected`.
- Возвращает количество затронутых строк (созданных/обновлённых).

### Статус AI-сервиса

Модуль: `backend/src/ai/core/service/status.rs`

Метод `status`:
- Получает версию рантайма и список доступных моделей.
- Проверяет наличие `default_chat` и `embeddings` в списке моделей.
- Статус: `"ok"`, если оба вызова успешны и обе модели доступны; иначе `"unavailable"`.
- Возвращает `AiStatusResponse` с информацией о рантайме, моделях, доступности.

## Семантическое хранилище эмбеддингов (SemanticEmbeddingStore)

Модули: `backend/src/ai/core/semantic/store.rs`, `backend/src/ai/core/semantic/sources.rs`, `backend/src/ai/core/semantic/source_persons.rs`, `backend/src/ai/core/semantic/source_projects.rs`, `backend/src/ai/core/semantic/source_tasks.rs`

- `SemanticEmbeddingStore` содержит пул соединений `PgPool`.
- `canonical_sources()` собирает все семантические источники в следующем порядке:
  1. Сообщения (`append_message_sources`)
  2. Документы (`append_document_sources`)
  3. Проекты (`append_project_sources`)
  4. Задачи (`append_task_sources`)
  5. Персоны (`append_person_sources`)
- Источник `Person`:
  - Из таблицы `persons` выбираются `person_id`, `display_name`, `email_address`, сортировка по `updated_at DESC, person_id`.
  - Текст источника: `"{display_name}\nEmail: {email_address}"`.
- Источник `Project`:
  - JOIN с `project_keywords`, агрегация ключевых слов через `string_agg`.
  - Выбираются поля: `project_id`, `name`, `kind`, `status`, `description`, `owner_display_name`, ключевые слова.
  - Текст источника содержит многострочное описание проекта.
  - Дополнительно проставляется `graph_node_id` через вызов `node_id(GraphNodeKind::Project, &project_id)`.
- Источник `Task`:
  - Из таблицы `tasks` выбираются `task_id`, `title`, `source_kind`, `source_id`, `status`.
  - Текст: `"{title}\nStatus: {status}\nSource: {source_kind}:{source_id}"`.
  - `graph_node_id` отсутствует (None).

## Модули API поддержки (app::api_support)

Модуль: `backend/src/app/api_support.rs` и подмодули.

- Файл `api_support.rs` является фасадом, реэкспортирующим подмодули: `automation_calls`, `communications`, `formatting`, `messaging_integrations`, `platform_dtos`, `query_parsing`, `review_commands`, `review_lists`, `stores`, `telegram_capabilities`, `whatsapp_capabilities`.
- Ссылка на ADR-0073 в комментарии указывает на текущую декомпозицию бэкенда.
- Функция `ensure_fixture_routes_enabled` разрешает фикстурные маршруты только в режиме разработки (`config.dev_mode()`) или в тестах (`cfg!(test)`), иначе возвращает ошибку `NotFound`.

### automation_calls

Модуль: `backend/src/app/api_support/automation_calls.rs`

Содержит DTO для автоматизации:
- `PolicyTemplateApiRequest` / `PolicyTemplateListResponse` для шаблонов.
- `PolicyApiRequest` / `PolicyListResponse` для политик (с полями: `policy_id`, `template_id`, `enabled`, `account_id`, `allowed_chat_ids`, `trigger_kind`, `max_sends_per_hour`, `quiet_hours`, `expires_at`, `conditions`).
- `CallApiRequest` для звонков (состояния `CallDirection`, `CallState`).
- `CallTranscriptFixtureApiRequest` для фикстур транскриптов.

### communications

Модуль: `backend/src/app/api_support/communications.rs`

DTO для сообщений:
- `CommunicationMessagesResponse` – список с курсорной пагинацией.
- `CommunicationMessageSummaryResponse` – краткая информация о сообщении (поля: `message_id`, `subject`, `sender`, `recipients`, `body_text_preview`, `delivery_state`, `local_state` и др.).
- `CommunicationMessageDetailResponse` – детальная информация, включая `body_text`, `body_html`, `message_metadata`, `local_state_reason`.
- `CommunicationAttachmentResponse` – информация о вложении (поля: `attachment_id`, `filename`, `content_type`, `size_bytes`, `sha256`, `scan_status`, `storage_kind` и др.).

### formatting

Модуль: `backend/src/app/api_support/formatting.rs`

Хелперы:
- `text_preview` – обрезка текста.
- `default_schema_version()` – возвращает `1`.
- `empty_json_object()` – возвращает `{}`.
- `html_escape(value)` – экранирует HTML-сущности.

### messaging_integrations

Модуль: `backend/src/app/api_support/messaging_integrations.rs`

DTO для интеграций с мессенджерами:
- `TelegramListQuery` (фильтры: `account_id`, `provider_chat_id`, `provider`, `channel_kind`, `limit`).
- `TelegramChatListResponse`, `TelegramMessageListResponse`.
- `WhatsappWebListQuery`, `WhatsappWebSessionListResponse`, `WhatsappWebMessageListResponse`.
- `TelegramReactionDeleteQuery` – запрос на удаление реакции.

### platform_dtos

Модуль: `backend/src/app/api_support/platform_dtos.rs`

DTO для платформенных сущностей:
- `ApplicationSettingsResponse` – список настроек.
- `ApplicationAccountsResponse` – список аккаунтов провайдеров.
- `AppendEventRequest` (с полями: `event_id`, `event_type`, `schema_version`, `occurred_at`, `source`, `actor`, `subject`, `payload`, `provenance`, `causation_id`, `correlation_id`) и метод `into_new_event` для создания `NewEventEnvelope`.
- `AppendEventResponse` – `event_id` и `position`.
- `AuditEventsQuery` – фильтры аудит-событий.
- `V1StatusResponse` с информацией о версии и доступных поверхностях (`surfaces`: `messages`, `persons`, `search`, `documents`, `account_setup`).
```

## Покрытие источников

- **`backend/src/ai/core/semantic/source_persons.rs`** – запрос к таблице `persons`, построение источников `Person` с полями `person_id`, `display_name`, `email_address`, формат текста.
- **`backend/src/ai/core/semantic/source_projects.rs`** – запрос с `LEFT JOIN project_keywords`, агрегация ключевых слов, построение источников `Project`, заполнение `graph_node_id`.
- **`backend/src/ai/core/semantic/source_tasks.rs`** – запрос к таблице `tasks`, построение источников `Task` с полями `task_id`, `title`, `source_kind`, `source_id`, `status`.
- **`backend/src/ai/core/semantic/sources.rs`** – метод `canonical_sources`, порядок вызова функций-дополнителей.
- **`backend/src/ai/core/semantic/store.rs`** – структура `SemanticEmbeddingStore`, конструктор `new`.
- **`backend/src/ai/core/service.rs`** – декларация подмодулей и реэкспорт `AiService`, `AiPersonaAttributionPort`.
- **`backend/src/ai/core/service/answer.rs`** – рабочий процесс `answer`, агент `MNEMOSYNE`, запуск, события, формирование ответа.
- **`backend/src/ai/core/service/attribution.rs`** – метод `run_attribution`, использование порта `persona_attribution`, ошибка `PersonaAttributionUnavailable`.
- **`backend/src/ai/core/service/attribution_port.rs`** – трейт `AiPersonaAttributionPort`, структура `AiAgentPersonaAttribution`, ошибка `AiPersonaAttributionError`.
- **`backend/src/ai/core/service/core.rs`** – структура `AiService`, конструкторы `new`, `new_with_routing`, `with_persona_attribution`.
- **`backend/src/ai/core/service/events.rs`** – структура `AiRunEvent`, метод `append_run_event` и сигнальная логика, маппинг типов событий на сигналы, редкация payload.
- **`backend/src/ai/core/service/meeting_prep.rs`** – рабочий процесс `meeting_prep`, агент `HESTIA`, скоуп запроса, модель `meeting_prep`, поле `briefing`.
- **`backend/src/ai/core/service/model_config.rs`** – метод `model_config`, возвращающий JSON с маршрутами моделей и размерностью эмбеддинга.
- **`backend/src/ai/core/service/retrieval.rs`** – метод `retrieve_citations`, индексация, эмбеддинг, векторный и текстовый поиск, объединение результатов.
- **`backend/src/ai/core/service/status.rs`** – метод `status`, проверка доступности моделей, статус `ok`/`unavailable`.
- **`backend/src/ai/core/service/task_candidate_persistence.rs`** – метод `upsert_ai_task_candidates`, фильтрация по `source_kind`, поиск `observation_id`, upsert в `task_candidates` с логикой сохранения пользовательского решения.
- **`backend/src/ai/core/service/task_candidates.rs`** – рабочий процесс `refresh_task_candidates`, агент `MAKOSH`, модель `extraction`, парсинг черновиков, синхронизация с инбоксом ревью, дополнительное событие `ai.task_extraction.completed`.
- **`backend/src/ai/core/types.rs`** – структуры `AiModelRouting` (и `fallback`), `AiCitation`, `AiAnswerRequest/Response`, `AiTaskCandidateRefreshRequest/Response`, `AiMeetingPrepRequest/Response`, `AiStatusResponse`.
- **`backend/src/ai/mod.rs`** – структура модуля `ai` (api, control_center, core).
- **`backend/src/app/api_support.rs`** – фасад `api_support`, реэкспорт подмодулей, комментарий ADR-0073, функция `ensure_fixture_routes_enabled`.
- **`backend/src/app/api_support/automation_calls.rs`** – DTO для автоматизации: `PolicyTemplateApiRequest`, `PolicyApiRequest`, `CallApiRequest`, `CallTranscriptFixtureApiRequest`.
- **`backend/src/app/api_support/communications.rs`** – DTO для сообщений: `CommunicationMessagesResponse`, `CommunicationMessageSummaryResponse`, `CommunicationMessageDetailResponse`, `CommunicationAttachmentResponse`.
- **`backend/src/app/api_support/formatting.rs`** – хелперы `text_preview`, `default_schema_version`, `empty_json_object`, `html_escape`.
- **`backend/src/app/api_support/messaging_integrations.rs`** – DTO для Telegram и WhatsApp: запросы, ответы, `TelegramReactionDeleteQuery`.
- **`backend/src/app/api_support/platform_dtos.rs`** – DTO для настроек, событий, аудита, статуса (`V1StatusResponse`).

## Исходные файлы

- [`backend/src/ai/core/semantic/source_persons.rs`](../../../../backend/src/ai/core/semantic/source_persons.rs)
- [`backend/src/ai/core/semantic/source_projects.rs`](../../../../backend/src/ai/core/semantic/source_projects.rs)
- [`backend/src/ai/core/semantic/source_tasks.rs`](../../../../backend/src/ai/core/semantic/source_tasks.rs)
- [`backend/src/ai/core/semantic/sources.rs`](../../../../backend/src/ai/core/semantic/sources.rs)
- [`backend/src/ai/core/semantic/store.rs`](../../../../backend/src/ai/core/semantic/store.rs)
- [`backend/src/ai/core/service.rs`](../../../../backend/src/ai/core/service.rs)
- [`backend/src/ai/core/service/answer.rs`](../../../../backend/src/ai/core/service/answer.rs)
- [`backend/src/ai/core/service/attribution.rs`](../../../../backend/src/ai/core/service/attribution.rs)
- [`backend/src/ai/core/service/attribution_port.rs`](../../../../backend/src/ai/core/service/attribution_port.rs)
- [`backend/src/ai/core/service/core.rs`](../../../../backend/src/ai/core/service/core.rs)
- [`backend/src/ai/core/service/events.rs`](../../../../backend/src/ai/core/service/events.rs)
- [`backend/src/ai/core/service/meeting_prep.rs`](../../../../backend/src/ai/core/service/meeting_prep.rs)
- [`backend/src/ai/core/service/model_config.rs`](../../../../backend/src/ai/core/service/model_config.rs)
- [`backend/src/ai/core/service/retrieval.rs`](../../../../backend/src/ai/core/service/retrieval.rs)
- [`backend/src/ai/core/service/status.rs`](../../../../backend/src/ai/core/service/status.rs)
- [`backend/src/ai/core/service/task_candidate_persistence.rs`](../../../../backend/src/ai/core/service/task_candidate_persistence.rs)
- [`backend/src/ai/core/service/task_candidates.rs`](../../../../backend/src/ai/core/service/task_candidates.rs)
- [`backend/src/ai/core/types.rs`](../../../../backend/src/ai/core/types.rs)
- [`backend/src/ai/mod.rs`](../../../../backend/src/ai/mod.rs)
- [`backend/src/app/api_support.rs`](../../../../backend/src/app/api_support.rs)
- [`backend/src/app/api_support/automation_calls.rs`](../../../../backend/src/app/api_support/automation_calls.rs)
- [`backend/src/app/api_support/communications.rs`](../../../../backend/src/app/api_support/communications.rs)
- [`backend/src/app/api_support/formatting.rs`](../../../../backend/src/app/api_support/formatting.rs)
- [`backend/src/app/api_support/messaging_integrations.rs`](../../../../backend/src/app/api_support/messaging_integrations.rs)
- [`backend/src/app/api_support/platform_dtos.rs`](../../../../backend/src/app/api_support/platform_dtos.rs)

## Кандидаты на drift

- **ADR-0073 не верифицирована** – файл `api_support.rs` содержит комментарий, ссылающийся на ADR-0073. Сама ADR не предоставлена, поэтому соответствие текущей реализации этому решению не может быть проверено.
- **Признак переходной архитектуры** – комментарий в `api_support.rs` указывает: «route modules still import this facade during the backend decomposition phase». Это может свидетельствовать о том, что текущая структура фасада является временной, и финальная архитектура может отличаться (потенциальное расхождение между кодом и запланированной архитектурой).
