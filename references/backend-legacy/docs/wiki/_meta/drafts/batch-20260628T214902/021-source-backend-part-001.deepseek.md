### Summary / Резюме

Обновление русскоязычной Obsidian-вики для страницы `components/backend.md` — добавляется описание подсистемы AI Control Center, её HTTP API, сущностей (провайдеры, модели, промпты), пресетов, проверок доступности и механики наблюдений (evidence). Вся информация основана исключительно на предоставленных исходных файлах. Предыдущее состояние страницы неизвестно — предлагается полный замещающий контент.

### Proposed pages / Предлагаемые страницы

**wiki-relative path:** `components/backend.md`

```markdown
# Backend

Серверная часть Макошь реализована на Rust с использованием фреймворка Axum. Основной модуль AI Control Center управляет конфигурацией и жизненным циклом AI‑провайдеров, моделей, маршрутов (routes) и промптов.

## Сборка и контракты

Сценарий сборки `backend/build.rs` генерирует код из proto‑контрактов с помощью `connectrpc_build`:

- `../contracts/proto/makosh/common/v1/common.proto`
- `../contracts/proto/makosh/signal_hub/v1/signal_hub.proto`
- `../contracts/proto/makosh/communications/v1/communications.proto`

Перестройка инициируется при изменении любого proto‑файла.

## AI Control Center (`backend/src/ai/`)

Центр управления ИИ — ядро конфигурации AI‑компонентов. Состоит из хранилища (`AiControlCenterStore`), слоя HTTP‑обработчиков и модулей для работы с провайдерами, моделями, промптами и сохранением свидетельств (evidence).

### Хранилище (`AiControlCenterStore`)

`AiControlCenterStore` инкапсулирует доступ к базе данных через пул `sqlx::PgPool`. Предоставляет методы для:

- Управления AI‑провайдерами (создание, обновление, запрос, команды).
- Каталога моделей (список, получение, заполнение «сидируемыми» моделями).
- Работы с промптами (создание, версионирование, активация, тестирование, список).
- Управления маршрутами моделей.
- Проверки доступности моделей и провайдеров для приватного контекста.
- Записи наблюдений (evidence).

Хранилище инициализируется фабрикой `ai_control_center_store` (`backend/src/ai/api/helpers.rs`), которая извлекает пул БД из состояния приложения (`AppState`). При отсутствии настроенной БД возвращается ошибка `ApiError::DatabaseNotConfigured`.

### HTTP API слой (`backend/src/ai/api/`)

API обработчики написаны с использованием `axum`. Все функции получают состояние приложения через `State<AppState>`.

#### Идентификация актора

Для операций с промптами используется заголовок `x-makosh-actor-id`. Если заголовок не задан или пуст, по умолчанию подставляется `"makosh-frontend"`. Реализовано в `request_actor_id` (`helpers.rs`). Операции из рантайма (ответы, подготовка встреч) всегда используют фиксированный идентификатор `"makosh-frontend"`.

#### Управление провайдерами (`control_center.rs`)

- `get_ai_settings_overview` — агрегированный обзор настроек AI (провайдеры, модели, маршруты, промпты, evaluation runs, capability slots, пресеты).
- `get_ai_providers` — список провайдеров.
- `post_ai_provider` — создание провайдера. При передаче API‑ключа выполняются проверки: провайдер должен иметь тип `api`, host vault должен быть разблокирован (`VaultMode::Unlocked`). Ключ сохраняется в vault через `store_api_key_in_host_vault`.
- `patch_ai_provider` — частичное обновление провайдера, аналогичная работа с ключом.
- `post_ai_provider_test` — тестирование провайдера (команда `Test`).
- `post_ai_provider_sync_models` — синхронизация курируемых моделей для провайдера (команда `SyncModels`).
- `post_ai_provider_consent` — запись согласия (`granted`/`revoked`) для API‑провайдеров.

#### Управление моделями и маршрутами

- `get_ai_models` — список моделей из каталога.
- `put_ai_model_route` — назначение модели на capability slot.

#### Управление промптами

- `get_ai_prompts` — список шаблонов промптов.
- `post_ai_prompt` — создание шаблона, использует идентификатор актора из заголовка.
- `post_ai_prompt_version` — создание новой версии промпта.
- `post_ai_prompt_activate` — активация версии промпта.
- `post_ai_prompt_test` — тестирование версии промпта (создаёт запись `AiPromptEvalRun`, см. Evaluation).

#### Рантайм операции (`runtime.rs`)

- `get_ai_status` — проверка статуса AI‑рантайма. Запрашивает настройки из `ai_runtime_settings`, получает список моделей и версию от рантайм‑клиента. Статус `"ok"` если клиент доступен и требуемые модели (chat, embedding) присутствуют. Возвращает `AiStatusResponse` с данными о моделях, embedding dimension и т.д.
- `get_ai_agents` — список AI‑агентов. Агенты формируются функцией `v3_agents`; при наличии сервиса атрибуции персон (`ai_persona_attribution_port_optional`) обогащаются данными персоны (persona_id, persona_type, persona_email).
- `get_ai_runs` — список недавних запусков AI (лимит 1–100). Запрашивает из `ai_run_store`.
- `get_ai_run` — детали конкретного запуска по ID.
- `post_ai_answer` — запрос к AI для получения ответа. Предварительно проверяется `ensure_ai_requests_allowed` (проверка разрешений через Signal Hub policy).
- `post_ai_task_candidates_refresh` — обновление кандидатов задач.
- `post_ai_meeting_prep` — подготовка к встрече.

### Сущности (модели данных) (`models.rs`, `presets.rs`)

#### Провайдеры

- `AiProviderAccount` — основной тип: `provider_id`, `provider_kind` (`built_in`, `cli`, `api`), `provider_key`, `display_name`, `status` (`ready`, `needs_setup`, `disabled`, ...), `consent_state` (`granted`, `revoked`, `required`, `not_required`), `consented_at`, `config` (JSON), `capabilities`.
- `AiProviderCreateRequest` — данные для создания. Валидация требует непустые `provider_kind`, `provider_key`, `display_name`; для CLI — `command_preset`; API‑ключ только для API‑провайдеров.
- `AiProviderPatchRequest` — поля для обновления: `display_name`, `base_url`, `config`, `enabled`, `api_key`.
- `AiProviderConsentRequest` — булево `consented`.
- `AiProviderCommandResponse` — ответ на команду (`status`, `message`). Команды: `Test`, `SyncModels` (`AiProviderCommandKind`).

#### Пресеты провайдеров (`presets.rs`)

Встроенные пресеты провайдеров (через `provider_presets()`):

| Provider Kind | Provider Key | Display Name           | Privacy |
|---------------|--------------|------------------------|---------|
| built_in      | ollama       | Built-in Ollama        | local   |
| cli           | codex        | Codex CLI              | cli     |
| cli           | claude       | Claude CLI             | cli     |
| api           | openai       | OpenAI                 | remote  |
| api           | deepseek     | DeepSeek               | remote  |
| api           | omniroute    | OmniRoute              | remote  |

Константы встроенного Ollama: `BUILT_IN_OLLAMA_PROVIDER_ID = "provider:built_in:ollama"`, `OLLAMA_CHAT_MODEL = "qwen3:4b"`, `OLLAMA_EMBEDDING_MODEL = "qwen3-embedding:4b"`.

#### Модели

- `AiModelCatalogItem` — запись в каталоге моделей: `provider_id`, `model_key`, `display_name`, `category` (chat, embeddings, reasoning…), `privacy` (local, remote, cli), `capabilities`, `context_window`, `embedding_dimension`, `is_available`, `metadata`.
- При создании провайдера (`create_provider`) происходит заполнение каталога курируемыми моделями (`seed_models_for_provider`). Список моделей определяется `curated_models_for` в зависимости от `provider_kind` и `provider_key`. Например, для Ollama: Qwen3 4B (chat) и Qwen3 Embedding 4B; для OpenAI: GPT‑5.5 и Text Embedding 3 Large и т.д.

#### Capability Slots

Предопределённые слоты (`capability_slots()`): `default_chat`, `reasoning`, `summarization`, `mail_intelligence`, `reply_draft`, `extraction`, `embeddings`, `meeting_prep`. Каждый слот имеет метку, описание и для слота `embeddings` — ограничение на размерность вектора (`requires_embedding_dimension`).

#### Маршруты моделей

`AiModelRoute` — связывает capability slot с конкретной моделью (`provider_id`, `model_key`). Обновляется через `put_ai_model_route`.

#### Промпты

- `AiPromptTemplate` — шаблон: `name`, `entity_scope`, `capability_slot`, `description`, `is_system` (системные промпты — только для чтения), `active_version_id`, `metadata`.
- `AiPromptVersion` — версия шаблона: `body_template` (текст с переменными), `variables` (массив имён), `status` (`draft` или `active`), `created_by_actor_id`.
- `AiPromptEvalRun` — запись о тестовом запуске: `output_text`, `score`, `notes`, `actor_id`, `provider_id`, `model_key`, `source_refs`, `variables`.

### Жизненный цикл промптов

1. Создание шаблона (`create_prompt`) — формируется `prompt_id`, сохраняется в таблицу `ai_prompt_templates`.
2. Создание версии (`create_prompt_version`) — создаётся запись в `ai_prompt_template_versions` со статусом `draft`. Версия автоматически получает метку на основе времени, если не задана.
3. Активация (`activate_prompt_version`) — выбранная версия получает статус `active`, все остальные версии этого промпта переводятся в `draft`. Системные промпты (`is_system: true`) нельзя активировать.
4. Тестирование (`test_prompt`) — создаётся `AiPromptEvalRun`. Шаблон рендерится с переданными переменными, но реальный вызов LLM **не выполняется** — результат помечается как «Prompt studio preview».

### Проверки доступности

Механизм `ensure_model_ready_for_private_context` и связанные методы проверяют:

- Модель существует и `is_available = true`.
- Провайдер не `disabled`.
- Для API‑провайдеров: согласие (`consent_state = "granted"`) и настроенный API‑ключ в host vault.
- Статус провайдера `"ready"`.

Эти проверки используются перед тестированием промптов и, вероятно, при реальных вызовах (хотя в контексте явно не показано).

### Хранение API‑ключей и секретов

API‑ключи хранятся в host vault. Связь между провайдером и ключом фиксируется в таблицах `ai_provider_secret_refs` и `secret_references` с параметрами: `secret_purpose = "api_key"`, `secret_kind = "api_token"`, `store_kind = "host_vault"`. Метод `api_key_secret_configured` проверяет наличие такой записи. Сохранение ключа в vault происходит через `store_api_key_in_host_vault` (в контексте доступна только её сигнатура). Доступ к vault требует разблокированного состояния; при заблокированном vault операции с API‑ключами возвращают ошибку `HostVaultError::Locked`.

### Наблюдения (Evidence) (`evidence.rs`)

Все значимые действия (создание/изменение провайдеров, моделей, маршрутов, промптов, версий, тестовых запусков) записываются как «наблюдения» (observations) через `ObservationStore::capture_in_transaction` в рамках транзакций. Каждое наблюдение привязывается к соответствующей доменной сущности через `link_domain_entity_in_transaction`. Действия фиксируются с указанием актора и типа взаимоотношения (например, `"create"`, `"activate"`, `"test"`).

### Обработка ошибок (`errors.rs`)

Ошибки слоя AI Control Center представлены перечислением `AiControlCenterError`:

- `ProviderNotFound`, `ModelNotFound`, `PromptNotFound`, `PromptVersionNotFound`
- `InvalidRequest(String)` — некорректный запрос
- `EmptyField { field }` — пустое поле
- `SecretLikePayload` — данные похожи на секреты
- `SecretReference`, `HostVault`, `ObservationStore`, `Sqlx` — обёртки над ошибками зависимостей

Метод `is_invalid_request` объединяет ошибки 4xx категории.

### Ограничения и допущения

- Реальная связь с AI‑рантаймами (Ollama, OpenAI, CLI) через `ai_runtime_client` не описана в данном контексте; обработчики предполагают наличие клиента.
- Маршрутизация HTTP запросов (конкретные пути) не включена в этот пакет; функции‑обработчики подключаются в вышележащем роутере.
- Дополнительные модули `providers/queries.rs`, `providers/secrets.rs`, `providers/update.rs`, `routes.rs`, `validation.rs` существуют согласно объявлению в `control_center.rs`, но их исходный код не представлен.
```

### Source coverage / Покрытие источников

- `backend/build.rs` — описан сценарий сборки, перечислены контракты и инструмент `connectrpc_build`.
- `backend/src/ai/api.rs` — зафиксированы публичные обработчики, разделение на `control_center` и `runtime`.
- `backend/src/ai/api/control_center.rs` — документированы все хендлеры управления провайдерами, моделями, маршрутами, промптами; обработка API‑ключей, проверка vault, согласие; вспомогательные функции `request_api_key`, `ensure_api_key_provider_kind`, `ensure_host_vault_unlocked_for_api_key`.
- `backend/src/ai/api/helpers.rs` — объяснены `ai_control_center_store` (получение хранилища из БД) и `request_actor_id` (извлечение актора из заголовка).
- `backend/src/ai/api/models.rs` — упомянуты структуры ответов `AiProviderListResponse`, `AiModelListResponse`, `AiPromptListResponse`.
- `backend/src/ai/api/runtime.rs` — описаны все хендлеры рантайма (`get_ai_status`, `get_ai_agents`, `get_ai_runs`, `get_ai_run`, `post_ai_answer`, `post_ai_task_candidates_refresh`, `post_ai_meeting_prep`), проверка `ensure_ai_requests_allowed`, использование `ai_runtime_client`, `ai_runtime_settings`, `v3_agents`, атрибуция персон.
- `backend/src/ai/control_center.rs` — перечислены реэкспортируемые модули и публичные типы, включая `AiControlCenterStore`, `AiControlCenterError`, пресеты и vault‑функцию.
- `backend/src/ai/control_center/availability.rs` — описаны проверки `ensure_model_ready_for_private_context`, `model_ready_for_private_context`, `ensure_provider_ready_for_private_context` и условия для API‑провайдеров.
- `backend/src/ai/control_center/availability/secrets.rs` — документирована проверка наличия API‑ключа через `api_key_secret_configured` (запросы к `ai_provider_secret_refs` и `secret_references`), константы `SECRET_PURPOSE_API_KEY`, `SECRET_KIND_API_TOKEN`, `SECRET_STORE_HOST_VAULT`.
- `backend/src/ai/control_center/catalog.rs` — описаны `list_models`, `model`, `seed_models_for_provider` с upsert‑логикой и записью наблюдений.
- `backend/src/ai/control_center/errors.rs` — перечислены варианты `AiControlCenterError` и метод `is_invalid_request`.
- `backend/src/ai/control_center/evidence.rs` — объяснена фиксация наблюдений для провайдеров, секретных привязок, маршрутов, шаблонов и версий промптов, eval‑запусков, моделей; упомянута связка через `link_domain_entity_in_transaction`.
- `backend/src/ai/control_center/models.rs` — детально описаны все модели данных: провайдеры, запросы, команды, пресеты, capability‑слоты, модели каталога, маршруты, шаблоны и версии промптов, eval‑запуски, валидация запросов.
- `backend/src/ai/control_center/presets.rs` — зафиксированы встроенные пресеты, константы Ollama, `capability_slots`, `curated_models_for` для каждого провайдера, `default_capabilities`, служебные функции `settings_label` и `capability_description`.
- `backend/src/ai/control_center/prompts.rs` — отмечены подмодули управления промптами.
- `backend/src/ai/control_center/prompts/activation.rs` — документирована логика активации версии промпта с переводом остальных в `draft`, блокировка системных промптов, запись наблюдений.
- `backend/src/ai/control_center/prompts/eval_runs.rs` — описан `list_prompt_eval_runs` с лимитом 1‑100.
- `backend/src/ai/control_center/prompts/evaluation.rs` — описан `test_prompt` с проверкой доступности модели, рендерингом шаблона, вставкой eval‑запуска и записью наблюдения; отмечено, что реальный LLM‑вызов не производится.
- `backend/src/ai/control_center/prompts/lookups.rs` — зафиксированы методы `prompt` и `prompt_version` для поиска.
- `backend/src/ai/control_center/prompts/templates.rs` — описаны `list_prompts` и `create_prompt` с генерацией ID, валидацией, сохранением и записью наблюдения.
- `backend/src/ai/control_center/prompts/versions.rs` — описан `create_prompt_version` с авто‑меткой времени, статусом `draft`, записью наблюдения; блокировка системных промптов.
- `backend/src/ai/control_center/providers.rs` — зафиксированы подмодули команд, согласия, создания, запросов, секретов, обновления (детали только для команд, согласия, создания).
- `backend/src/ai/control_center/providers/commands.rs` — документирована команда `provider_command` для `Test` и `SyncModels` с проверками состояния для каждого типа провайдера.
- `backend/src/ai/control_center/providers/consent.rs` — описан `record_consent` для API‑провайдеров.
- `backend/src/ai/control_center/providers/create.rs` — детально описано создание провайдера, формирование ID, установка статуса и consent‑состояния, запись в БД, вызов `seed_models_for_provider`, запись наблюдений.

### Drift candidates / Кандидаты на drift

В предоставленном контексте расхождений между кодом и документацией не выявлено — целевая страница создаётся впервые, а все утверждения непосредственно подтверждены исходными текстами. Однако модули `providers/queries.rs`, `providers/secrets.rs`, `providers/update.rs`, `routes.rs` и `validation.rs` объявлены в `control_center.rs`, но их реализация отсутствует в контексте, поэтому документирование их поведения ограничено косвенными ссылками — при появлении полных исходников возможно уточнение.
