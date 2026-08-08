## Summary / Резюме

Обновить страницу `components/backend.md` в русской Obsidian‑wiki, задокументировав AI‑подсистему бэкенда Макошь: управление AI‑провайдерами (control_center), маршрутизацию моделей, жизненный цикл AI‑запусков (runs), семантические эмбеддинги и агентов. Вся информация берётся строго из встроенных исходных файлов чанка `022‑source‑backend‑part‑002`.

## Proposed pages / Предлагаемые страницы

### components/backend.md

```markdown
# Бэкенд — AI подсистема

## Обзор

AI‑подсистема состоит из двух крупных модулей:

- `ai::control_center` – управление AI‑провайдерами, ключами, маршрутизацией моделей и промптами.
- `ai::core` – агенты, запуски (runs), семантические эмбеддинги и поиск.

Оба модуля используют PostgreSQL через `sqlx` и пишут события аудита (observations) при каждом значимом изменении.

---

## AI Control Center (`backend/src/ai/control_center/`)

### Центральное хранилище – `AiControlCenterStore`

- Содержит `PgPool`.
- Метод `overview()` собирает: всех провайдеров (`list_providers`), все модели (`list_models`), маршруты (`list_model_routes`), промпты (`list_prompts`), последние 25 eval‑запусков, а также статические списки `capability_slots` и `provider_presets`.

### Управление провайдерами

#### Получение

- `list_providers() -> Vec<AiProviderAccount>`
  Из таблицы `ai_provider_accounts`, отсортировано по `provider_kind`, `display_name`, `provider_id`.
  Поля строки: `provider_id`, `provider_kind`, `provider_key`, `display_name`, `status`, `consent_state`, `consented_at`, `config`, `capabilities`, `created_at`, `updated_at`.

- `provider(id) -> Option<AiProviderAccount>`
  Выборка по `provider_id`. Требует непустого идентификатора.

#### Обновление

- `update_provider(id, request: &AiProviderPatchRequest) -> AiProviderAccount`
  - API‑ключи (`api_key`) разрешено передавать только для провайдеров с `provider_kind == "api"`.
  - Статус вычисляется по правилам:
    - `enabled = true`, но API‑ключ не настроен → `"needs_setup"`
    - `enabled = true` и статус уже `"needs_setup"` → остаётся `"needs_setup"`
    - `enabled = true` и ключ готов → `"ready"`
    - `enabled = false` → `"disabled"`
    - `enabled` не задан → сохраняется текущий статус.
  - Конфигурация (`config`) – JSON‑объект; обновляется слиянием (можно передать `base_url`).
  - **Защита от секретов**: `reject_secret_like_json` проверяет итоговый `config` на наличие полей, содержащих `secret`, `password`, `token`, `credential`, `private_key`, а также полей `body`, `html`, `raw`.
  - Операция обёрнута в транзакцию; создаётся observation с actor `"ai_control_center.update_provider"`.

#### Секреты (API‑ключи)

- `bind_api_key_secret(provider_id, secret_ref) -> ()`
  - Разрешено только для `provider_kind == "api"`.
  - `secret_ref` должен быть ссылкой на секрет типа `api_token` в host‑vault.
  - UPSERT в таблицу `ai_provider_secret_refs` с `secret_purpose = "api_key"`.
  - Если статус провайдера был `"needs_setup"`, переводит в `"ready"`.
  - Observation: `"ai_control_center.bind_api_key_secret"`.

#### Хранение ключа в Host Vault

- `store_api_key_in_host_vault(pool, vault, provider_id, api_key) -> String` (возвращает `secret_ref`)
  - Формирует идентификатор секрета `secret:ai-provider:{provider_id}:api_key`.
  - Создаёт запись через `SecretReferenceStore` и записывает значение в `HostVault`.
  - Затем вызывает `bind_api_key_secret`.

### Маршрутизация моделей (`routes.rs`)

- `list_model_routes() -> Vec<AiModelRoute>` – все записи из `ai_model_routes`.
- `route_for_slot(slot) -> Option<AiModelRoute>` – маршрут для конкретного capability‑слота.
- `put_model_route(slot, request: &AiModelRouteUpdateRequest) -> AiModelRoute`
  - Валидирует `capability_slot` (см. ниже).
  - Проверяет, что модель готова для приватного контекста.
  - **Для слота `"embeddings"` требует модель с размерностью эмбеддинга ровно `2560` (`AI_EMBEDDING_DIMENSION`).**
  - UPSERT через `INSERT ... ON CONFLICT`.
  - Observation: `"ai_control_center.put_model_route"`.

### Маппинг строк БД → модели (`rows.rs`)

Функции преобразования `PgRow` в доменные структуры:

| Функция                  | Модель                    |
|--------------------------|---------------------------|
| `row_to_provider`        | `AiProviderAccount`       |
| `row_to_model`           | `AiModelCatalogItem`      |
| `row_to_route`           | `AiModelRoute`            |
| `row_to_prompt`          | `AiPromptTemplate`        |
| `row_to_prompt_version`  | `AiPromptVersion`         |
| `row_to_eval_run`        | `AiPromptEvalRun`         |

- `capabilities` десериализуются как JSON‑массив строк (`json_string_array`).
- `source_refs` в eval‑запусках – как общий JSON‑массив (`json_array`).

### Валидация и ограничения (`validation.rs`)

#### Допустимые виды провайдеров (`validate_provider_kind`)
  `"built_in"`, `"cli"`, `"api"`.

#### CLI‑пресеты (`validate_cli_preset`)
  Разрешены только: `"codex"`, `"claude"`, `"makosh"`.

#### Capability‑слоты (`CAPABILITY_SLOTS`)
  `default_chat`, `reasoning`, `summarization`, `mail_intelligence`, `reply_draft`, `extraction`, `embeddings`, `meeting_prep`.

#### Области сущностей (entity scope)
  `global`, `person`, `organization`, `project`, `document`, `task`, `meeting`, `communication`, `conversation`.

#### `reject_secret_like_json(value)`
  Рекурсивно обходит JSON‑значение и возвращает `SecretLikePayload`, если в ключах объекта (регистронезависимо) встречаются:
  `secret`, `password`, `token`, `credential`, `private_key`, а также точные совпадения `body`, `html`, `raw`.

#### `render_prompt(template, variables)`
  Заменяет плейсхолдеры `{{variable}}` на строковые значения. Используется для простой шаблонизации промптов.

#### `slug_id(value)`
  Преобразует строку в URL‑slug; если результат пуст, возвращает timestamp наносекунд.

### Тесты (`tests.rs`)

- Отклонение JSON с секретоподобными ключами (например, `"authorization_token"`).
- Белый список CLI‑пресетов: shell‑подобные строки (например, `"bash -lc env"`) отвергаются.
- Наличие пресетов для `openai`, `deepseek`, `omniroute`, `ollama` (последний с `privacy: "local"`).
- Слот `"embeddings"` требует размерность `AI_EMBEDDING_DIMENSION` (2560).
- Рендеринг промптов не использует исходный текст, только значения переменных.

---

## AI Core (`backend/src/ai/core/`)

### Агенты (`agents.rs`)

Статически определены пять агентов (функция `v3_agents`):

| `agent_id`   | Роль                                                                 | статус     |
|--------------|----------------------------------------------------------------------|------------|
| `HESTIA`     | Подготовка встреч и контекста дома                                   | `available`|
| `MAKOSH`     | Координация workflow и извлечение кандидатов задач                   | `available`|
| `MNEMOSYNE`  | Ответы с опорой на локальные источники                               | `available`|
| `ATHENA`     | Обзор планов и поддержка принятия решений                            | `available`|
| `HEPHAESTUS` | Разработка, поддержка и автоматизация инструментов                   | `available`|

- Все используют общую `default_model` (передаётся как параметр).
- `validate_agent` принимает только эти пять идентификаторов.
- `ai_agent_display_name` возвращает email‑подобные имена (например, `makosh@sh-inc.ru`).

### Константы (`constants.rs`)

- `AI_EMBEDDING_DIMENSION: usize = 2560`
- `AI_PROMPT_TEMPLATE_VERSION: &str = "v3-local-source-backed-2026-06-06"`
- `DEFAULT_RETRIEVAL_LIMIT: i64 = 8`

### Ошибки (`errors.rs`)

`AiError` объединяет следующие варианты:

- `InvalidRequest`, `UnknownAgent`, `InvalidSourceKind`, `InvalidEmbeddingDimension { expected, actual }`, `RunNotFound`.
- Transparent‑ошибки от: `AiRuntimeError`, `EventEnvelopeError`, `EventStoreError`, `PersonaAttribution*`, `ReviewInboxWorkflowError`, `ObservationStoreError`, `serde_json::Error`, `sqlx::Error`.

### AI‑запуски (`runs.rs`, `AiRunStore`)

Таблица `ai_agent_runs` управляется через `AiRunStore`.

#### Жизненный цикл

- **Старт**: `start_run(run: &NewAiRun)`
  - Валидирует все поля `NewAiRun` (все строки непустые, `model_config` – JSON‑объект).
  - `INSERT ... ON CONFLICT (run_id) DO UPDATE SET status = 'requested'`, сбрасывая предыдущий ответ/ошибку.
  - Observation: `AI_AGENT_RUN`, relationship `"requested"`.

- **Успешное завершение**: `complete_run(run_id, answer, citations, duration_ms, completed_event_id)`
  - Статус → `'completed'`, сохраняет `answer` и `citations`.
  - Observation: `AI_AGENT_RUN_STATUS`, relationship `"completed"`.

- **Отказ**: `fail_run(run_id, error_summary, duration_ms, failed_event_id)`
  - Статус → `'failed'`, сохраняет `error_summary`.
  - Observation: `AI_AGENT_RUN_STATUS`, relationship `"failed"`.

- **Чтение**: `get_run(run_id)`, `list_runs(limit)` (пагинация по `started_at DESC`).

#### Модель `AiAgentRun`
Содержит поля: `run_id`, `agent_id`, `status`, `chat_model`, `embedding_model`, `prompt_template_version`, `model_config`, `query`, `answer`, `citations`, `error_summary`, `actor_id`, `agent_persona_id`, `owner_persona_id`, `causation_id`, `correlation_id`, `requested_event_id`, `completed_event_id`, `failed_event_id`, `started_at`, `completed_at`, `duration_ms`, `created_at`, `updated_at`.

### Семантические эмбеддинги (`semantic/`)

#### Виды источников (`SemanticSourceKind`)
`Message` → `"message"`, `Document` → `"document"`, `Project` → `"project"`, `Task` → `"task"`, `Person` → `"contact"` или `"person"`.

#### Модели

- `NewSemanticEmbedding<'a>` – входные данные: `source_kind`, `source_id`, `observation_id` (обязателен для Message), `title`, `source_text`, `embedding_model`, `embedding: &[f32]`, `graph_node_id`.
- `SemanticEmbedding` – сохранённая запись: `semantic_embedding_id`, поля источника, `content_hash`, `embedding_dimension`, временные метки.
- `SemanticSearchResult` – результат поиска: те же поля + `score: f64`.
- `SemanticIndexReport` – счётчики: `sources_seen`, `sources_indexed`, `sources_skipped`.

#### Хранилище `SemanticEmbeddingStore`

- **`upsert_embedding`**
  Генерирует `semantic_embedding_id` (хеш SHA‑256 от `source_kind`, `source_id`, `embedding_model`).
  Строит `halfvec`‑литерал из embedding.
  UPSERT по `(source_kind, source_id, embedding_model)`.
  Observation: `AI_SEMANTIC_EMBEDDING`, relationship `"upsert"`.

- **`is_current`** – сравнивает сохранённый `content_hash`; если совпадает, эмбеддинг не требует обновления.

- **`index_canonical_sources`**
  Обходит канонические источники (документы, сообщения и др.). Для каждого: проверяет актуальность через `is_current`; если устарел – вызывает внешний `AiRuntimeClient.embed_with_model` и делает `upsert_embedding`. Возвращает отчёт.

- **`search`** (векторный)
  Использует оператор `<=>` (halfvec‑расстояние). `score = 1.0 - distance`.
  Сортировка по близости, затем по `updated_at DESC, source_id`.

- **`text_search`** (полнотекстовый)
  PostgreSQL `to_tsvector` по `title || ' ' || source_text`, `plainto_tsquery`.
  `score = ts_rank_cd`. Возвращает только строки с ненулевым рангом.

#### Источники для индексации

- **Документы** (`source_documents.rs`)
  Из таблицы `documents` выбираются записи с непустым `extracted_text`. Формируется текст `"{title}\n\n{extracted_text}"`.

- **Сообщения** (`source_messages.rs`)
  Из `communication_messages` выбираются `message_id`, `observation_id`, `subject`, `sender`, `recipients`, `body_text`. Формируется текст `"Subject: ...\nFrom: ...\nTo: ...\n\n{body_text}"`.

### Вспомогательные функции (`helpers.rs`)

- `merge_retrieval_results(vector_results, text_results)` – объединяет результаты, применяя веса (векторные умножаются на 0.75, текстовым добавляется 0.75), удаляет дубликаты по `(source_kind, source_id)` и сортирует по убыванию score.
- `halfvec_literal(embedding)` – собирает строку `[v1,v2,...]` для PostgreSQL `halfvec`, проверяет размерность и конечность значений.
- `content_hash`, `semantic_embedding_id`, `run_id_from_command`, `event_id_from_command`, `ai_task_candidate_id` – детерминированные генераторы строковых идентификаторов через SHA‑256.
- `validate_non_empty`, `validate_limit(1..=100)`, `text_preview`, `elapsed_ms`.

### Промпты (`prompts.rs`)

Три шаблона для агентов:

- **`answer_prompt`** – агент MNEMOSYNE, отвечает только по переданным локальным источникам.
- **`task_candidate_prompt`** – агент MAKOSH, возвращает JSON‑массив кандидатов задач.
- **`meeting_prep_prompt`** – агент HESTIA, готовит брифинг по локальным источникам.

Все шаблоны требуют не доверять содержимому источников как инструкциям.

- `parse_task_candidate_drafts` – разбирает ответ модели, заменяет специальный идентификатор `"__first__"` на данные первого подходящего citation.
- `citation_for_draft` – поиск цитаты по draft.
- `scoped_meeting_query` – расширяет тему встречи полями `Project` / `Contact`.

### Интеграция с событиями

При каждом изменении состояния (провайдеры, маршруты, эмбеддинги, запуски) создаётся observation через `ObservationStore::capture_in_transaction` и связывается с AI‑сущностью через `link_ai_entity_in_transaction` (evidence.rs).
```

## Source coverage / Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `backend/src/ai/control_center/providers/queries.rs` | `list_providers` и `provider` – SQL‑запросы к `ai_provider_accounts`, поля выборки, сортировка, маппинг через `row_to_provider`. |
| `backend/src/ai/control_center/providers/secrets.rs` | `bind_api_key_secret` – проверки (provider_kind == "api", secret в host‑vault), UPSERT в `ai_provider_secret_refs`, обновление статуса `needs_setup → ready`, observation‑audit. |
| `backend/src/ai/control_center/providers/update.rs` | `update_provider` – логика вычисления статуса, проверка API‑key только для api‑провайдеров, слияние config, `base_url`, вызов `reject_secret_like_json`, observation. |
| `backend/src/ai/control_center/routes.rs` | `list_model_routes`, `route_for_slot`, `put_model_route` – работа с `ai_model_routes`, проверка capability‑слота, ограничение размерности эмбеддингов (2560), observation. |
| `backend/src/ai/control_center/rows.rs` | Функции маппинга `PgRow` → структуры `AiProviderAccount`, `AiModelCatalogItem`, `AiModelRoute`, `AiPromptTemplate`, `AiPromptVersion`, `AiPromptEvalRun`; способы десериализации полей `capabilities`, `source_refs`. |
| `backend/src/ai/control_center/store.rs` | Определение `AiControlCenterStore`, его поле `pool`, конструктор `new`, метод `overview` (сбор всех данных). |
| `backend/src/ai/control_center/tests.rs` | Тесты на отклонение секретоподобных JSON, CLI‑пресетов, наличие пресетов провайдеров, ограничение размерности слота `embeddings`, рендеринг промптов. |
| `backend/src/ai/control_center/validation.rs` | Списки допустимых `provider_kind`, `cli_preset`, `capability_slots`, `entity_scope`; функции `validate_*`, `reject_secret_like_json`, `render_prompt`, `slug_id`. |
| `backend/src/ai/control_center/vault.rs` | `store_api_key_in_host_vault` – формирование `secret_ref`, создание секрета, вызов `bind_api_key_secret`. |
| `backend/src/ai/core.rs` | Объявления модулей и реэкспорты: `AiAgentDescriptor`, `AI_EMBEDDING_DIMENSION`, `AiError`, `AiAgentRun`, `NewAiRun`, `SemanticEmbedding`, `SemanticSearchResult`, и др. |
| `backend/src/ai/core/agents.rs` | Определение пяти агентов (`v3_agents`), их ролей, статус `available`; функции `validate_agent` и `ai_agent_display_name`. |
| `backend/src/ai/core/constants.rs` | `AI_EMBEDDING_DIMENSION = 2560`, `AI_PROMPT_TEMPLATE_VERSION`, `DEFAULT_RETRIEVAL_LIMIT = 8`. |
| `backend/src/ai/core/errors.rs` | Перечисление `AiError` с вариантами и transparent‑ошибками. |
| `backend/src/ai/core/evidence.rs` | `link_ai_entity_in_transaction` – обёртка над `link_domain_entity_in_transaction` для домена `"ai"`. |
| `backend/src/ai/core/helpers.rs` | `merge_retrieval_results`, `halfvec_literal`, `content_hash`, `semantic_embedding_id`, `run_id_from_command`, `event_id_from_command`, `ai_task_candidate_id`, `validate_non_empty`, `validate_limit`, `text_preview`, `elapsed_ms`. |
| `backend/src/ai/core/prompts.rs` | Шаблоны `answer_prompt`, `task_candidate_prompt`, `meeting_prep_prompt`; `parse_task_candidate_drafts`; `citation_for_draft`; `scoped_meeting_query`. |
| `backend/src/ai/core/runs.rs` (truncated) | `AiRunStore` – `start_run`, `complete_run`, `fail_run`, `get_run`, `list_runs`; модель `NewAiRun` с валидацией; `AiAgentRun`; `capture_run_observation`. |
| `backend/src/ai/core/semantic.rs` | Структура модуля `semantic`, реэкспорты. |
| `backend/src/ai/core/semantic/embeddings.rs` | `upsert_embedding` – UPSERT в `semantic_embeddings`, проверка `observation_id` для Message, observation; `is_current` – сравнение хеша. |
| `backend/src/ai/core/semantic/indexing.rs` | `index_canonical_sources` – итерация по каноническим источникам, проверка актуальности, запрос эмбеддинга у рантайма, upsert, отчёт. |
| `backend/src/ai/core/semantic/models.rs` | `SemanticSourceKind` (Message, Document, Project, Task, Person), `SemanticEmbedding`, `NewSemanticEmbedding`, `SemanticSearchResult`, `SemanticIndexReport`, `SemanticSource`. |
| `backend/src/ai/core/semantic/rows.rs` | Маппинг строк в `SemanticEmbedding` и `SemanticSearchResult`, парсинг `SemanticSourceKind` из строки. |
| `backend/src/ai/core/semantic/search.rs` | `search` (векторный, через `<=>`) и `text_search` (полнотекстовый, `to_tsvector`/`plainto_tsquery`), лимит, сортировка. |
| `backend/src/ai/core/semantic/source_documents.rs` | Формирование источников из таблицы `documents` (поля `document_id`, `observation_id`, `title`, `extracted_text`). |
| `backend/src/ai/core/semantic/source_messages.rs` | Формирование источников из `communication_messages` (поля `message_id`, `observation_id`, `subject`, `sender`, `recipients`, `body_text`). |

## Drift candidates / Кандидаты на drift

На основе предоставленного контекста drift не обнаружен. Однако следует отметить, что часть файлов (например, `runs.rs`) была обрезана, и полное покрытие их содержимого не гарантировано. Другие потенциальные точки drift (несоответствие схемы БД, отсутствие документации по `AiService`, `types` или `vault` вне данного чанка) не могут быть подтверждены или опровергнуты без дополнительного контекста.
