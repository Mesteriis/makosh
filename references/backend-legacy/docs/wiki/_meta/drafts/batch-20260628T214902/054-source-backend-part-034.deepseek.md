### Summary / Резюме

Обновить страницу `components/backend.md` в русской Obsidian‑wiki, добавив описание домена задач (`tasks`) и движка автоматизации (`automation`), основанное на предоставленных исходных файлах бэкенда `makosh‑hub`. Описать структуру модуля `tasks` (ядро, мониторинг, аналитика, правила, экспорт, порты), ключевые хранилища, модели, сервисы и их поведение. Описать модуль `automation` (политики, шаблоны, dry‑run‑отправка Telegram, проверка политик, сохранение наблюдений). Указать используемые крейты и интеграции.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend

Backend‑сервис `makosh‑hub` написан на Rust и состоит из **доменов** (domains) и **движков** (engines). В данном контексте представлены:

- **Домен задач** — `backend/src/domains/tasks`
- **Движок автоматизации** — `backend/src/engines/automation`

Данные хранятся в PostgreSQL через `sqlx`. Важные системные понятия: наблюдения (`observations`), связи сущностей (`entity links`) и события (`events`).

---

## Домен задач (`tasks`)

Модуль предоставляет полный жизненный цикл задач: создание, классификация, приоритизация, подзадачи, связи с другими сущностями, мониторинг здоровья и экспорт.

### Структура модуля

`mod.rs` объявляет подмодули:

- `api` – (в контексте не раскрыт)
- `brain` – (в контексте не раскрыт)
- `candidates` – (в контексте не раскрыт)
- `command_service` – (в контексте не раскрыт)
- `core` – ядро: ошибки, хранилища, модели, связи
- `health` – мониторинг состояния задач
- `intelligence` – аналитические метрики и рекомендации
- `ports` – публичные порты модуля
- `rules` – правила и шаблоны задач
- `service` – реэкспорт из `command_service`
- `sync` – экспорт в Markdown / JSON

### Ядро (`core`)

#### Ошибки
`TaskCoreError` (файл `errors.rs`) содержит варианты:
- `Sqlx` (transparent)
- `ContextPack` (transparent)
- `Relationship` (transparent)
- `Observation` (transparent)
- `NotFound`

#### Доказательства (`TaskEvidence`)
Хранилище `TaskEvidenceStore` (`evidence.rs`) управляет доказательствами, привязанными к задаче.

- **`list(task_id)`** – возвращает все доказательства задачи, отсортированные по дате создания.
- **`add(task_id, source_type, source_id, quote, confidence)`** – добавляет доказательство. Если `source_type == "observation"`, автоматически создаёт в транзакции две связи с observation: одну для самой записи `task_evidence`, другую для задачи с relationship‑kind `"supports"` и confidence доказательства.

#### Внешние идентификаторы (`ExternalTaskIdentity`)
`ExternalTaskIdentityStore` (`external_identities.rs`) хранит внешние идентификаторы задачи у разных провайдеров.

- **`list(task_id)`** – возвращает список записей с полями: `provider`, `account_id`, `external_project_id`, `external_task_id`, `external_url`, `external_status`, `sync_status` и др.

#### Связи обязательств (`ObligationTaskLink`)
`ObligationTaskLinkStore` (`obligation_links.rs`) связывает обязательство (obligation) с задачей.

- **`link_fulfillment_task(obligation_id, task_id)`** – вставляет запись в `obligation_task_links` с `link_kind = 'fulfillment_task'`. Использует `ON CONFLICT DO NOTHING`.

#### Связи с наблюдениями (`observation_links`)
Модуль `observation_links.rs` содержит вспомогательные функции, работающие в транзакции:

- **`materialize_task_observation_link_in_transaction`** – связывает observation с задачей (entity_kind `"task"`).
- **`materialize_task_entity_link_in_transaction`** – связывает observation с произвольной сущностью домена `"tasks"`.

Обе функции используют `link_domain_entity_in_transaction` из `platform::observations`.

#### Учётные записи провайдеров (`TaskProviderStore`)
`TaskProviderStore` (`provider_store.rs`) управляет учётными записями провайдеров задач (`TaskProviderAccount`).

- **`list()`** – возвращает все учётные записи.
- **`create(provider, account_name)`** – создаёт запись с origin `LocalRuntime` и актором `"tasks_api.post_task_provider"`.
- **`create_with_origin(provider, account_name, origin_kind, actor)`** – создаёт запись с заданным origin и актором. В одной транзакции:
  1. Генерируется `account_id` с префиксом `tprov`.
  2. Создаётся observation типа `"TASK_PROVIDER_ACCOUNT"`.
  3. Вставляется строка в `task_provider_accounts`.
  4. Вызывается внутренняя `link_domain_owned_entity_in_transaction`, которая связывает observation с сущностью `"task_provider_account"` в домене `"tasks"`.

Модель `TaskProviderAccount` (`providers.rs`) содержит поля: `account_id`, `provider`, `account_name`, `credentials_reference`, `sync_mode`, `capabilities`, `created_at`, `updated_at`.

#### Связи задач (`TaskRelation`)
`TaskRelationStore` (`relations.rs`) управляет связями задачи с другими сущностями (любого типа).

- **`list(task_id)`** – возвращает связи, отсортированные по `relation_type`.
- **`link(task_id, entity_type, entity_id, relation_type, source)`** – добавляет связь (с `ON CONFLICT DO NOTHING`). В одной транзакции:
  1. Если `source` начинается с `"observation:"`, создаётся связь с observation через `materialize_task_entity_link_in_transaction`.
  2. Материализуется **отношение** (relationship) в домене `relationships`:
     - Парсится `RelationshipEntityKind` из `entity_type`.
     - Если observation не был передан, создаётся новое наблюдение типа `"TASK_MUTATION"`.
     - Формируется `NewRelationship` с `review_state = UserConfirmed` и trust/strength/confidence из поля `confidence` связи.
     - Добавляется evidence с фиксированным excerpt `"Task relation was recorded through compatibility task relation data."`.
     - Вызывается `RelationshipReviewPort::upsert_with_evidence_in_transaction`.

#### Подзадачи (`TaskSubtask`)
`TaskSubtaskStore` (`subtasks.rs`) управляет деревом подзадач.

- **`list(parent_id)`** – возвращает подзадачи, отсортированные по `sort_order`.
- **`add(parent_id, child_id, order)`** – добавляет подзадачу с источником `"manual"`.
- **`add_with_source(parent_id, child_id, order, source)`** – добавляет подзадачу с указанным источником. Если `source` начинается с `"observation:"`, создаётся связь с observation через `materialize_task_entity_link_in_transaction`. Использует `ON CONFLICT ... DO UPDATE` (обновляет `sort_order` и `source` при повторе родитель‑потомок).

### Мониторинг (`health`)

`TaskWatchtowerService` содержит шесть диагностических методов, возвращающих JSON.

- **`overdue`** – просроченные задачи (статус не `done/cancelled/archived` и `due_at < now`), до 30 штук.
- **`waiting_too_long(days)`** – задачи в статусе `'waiting'` дольше N дней, до 20 штук.
- **`without_context`** – активные задачи (до 50), у которых отсутствует контекстный пакет (`ContextPackKind::Task`); возвращает до 20.
- **`stale_tasks(days)`** – активные задачи без обновлений N дней, до 20 штук.
- **`cycle_time`** – среднее время выполнения (часы) последних 50 завершённых задач.
- **`workload`** – количество активных задач и количество просроченных.

Ошибки: `TaskHealthError` с вариантами `Sqlx` и `ContextPack`.

### Интеллектуальные функции (`intelligence`)

`TaskIntelligenceService` предоставляет статические методы для аналитики и рекомендаций.

- **`calculate_priority(...)`** – вычисляет приоритет (0..1) на основе: дедлайна (чем ближе, тем выше), юридических/налоговых флагов, блокировок, наличия контакта, организации, проекта. Базовый score = 0.2.
- **`calculate_risk(...)`** – оценивает риск на основе: близости дедлайна, отсутствия документов, отсутствия владельца, внешних зависимостей, юридического флага и ключевых слов («urgent», «asap», «срочно») в названии.
- **`calculate_readiness(...)`** – готовность к выполнению: описание, контекст, документы, дедлайн, нет блокировок, разрешены контакты.
- **`detect_missing_context(...)`** – возвращает список отсутствующих элементов: описание, контекстный пакет, дедлайн, контакт, проект.
- **`suggest_next_action(status, has_blockers, waiting_reason)`** – предлагает следующее действие в зависимости от статуса задачи (`new`/`triaged` → установить приоритет, `blocked` → разрешить блокеры и т.д.).

Ошибки: `TaskIntelligenceError::AnalysisFailed`.

### Правила и шаблоны (`rules`)

- **`TaskRule`** – правило с полями: `rule_id`, `name`, `natural_language_description`, `compiled_dsl` (JSON), `enabled`, `approval_mode`, `last_run_at`, `created_at`, `updated_at`. `TaskRuleStore` поддерживает `list`, `create`, `delete`. При создании устанавливается `approval_mode` по умолчанию в `"suggest_only"`. Идентификатор генерируется как `taskrule:v1:{timestamp_nanos:x}`.
- **`TaskTemplate`** – шаблон задачи с полями: `template_id`, `name`, `description`, `default_fields` (JSON), `default_checklist` (JSON), `default_priority`, `default_energy_type`, `required_documents` (JSON), `created_at`, `updated_at`. `TaskTemplateStore` поддерживает `list`.

Ошибки: `TaskRuleError` с вариантами `Sqlx` и `NotFound`.

### Экспорт (`sync`)

- **`export_task_md(title, description, status, why, outcome)`** – формирует Markdown задачу с полями `# Title`, статусом, блоком «Why», описанием и «Outcome».
- **`export_task_json(title, description, status, priority, due_at)`** – формирует JSON-объект из указанных полей.

Ошибки: `TaskSyncError::SyncFailed`.

### Публичные порты (`ports`)

В `ports.rs` определены реэкспорты:

- `TaskCommandPort` → `api::TaskStore`
- `TaskCandidatePort` → `candidates::TaskCandidateStore`
- `ObligationTaskLinkPort` → `core::ObligationTaskLinkStore`

### Сервис (`service`)

`service.rs` реэкспортирует всё из `command_service` (реализация не включена в контекст).

---

## Движок автоматизации (`automation`)

Модуль `engines::automation` управляет автоматической отправкой сообщений (например, Telegram‑сообщений) согласно политикам.

### Публичный интерфейс

Из `automation/mod.rs` экспортируются:

- Типы: `AutomationPolicy`, `AutomationTemplate`, `NewAutomationPolicy`, `NewAutomationTemplate`, `TelegramSendDryRunRequest`, `TelegramSendDryRunResponse`, `object_from_pairs`
- Хранилище: `AutomationStore` (реализация не включена в контекст)
- Ошибки: `AutomationError`

### Модели (`models.rs`)

- **`AutomationTemplate`** – шаблон сообщения: `template_id`, `name`, `body_template`, `required_variables` (Vec<String>), `created_at`, `updated_at`.
- **`AutomationPolicy`** – политика отправки: `policy_id`, `template_id`, `name`, `enabled`, `account_id`, `allowed_chat_ids`, `trigger_kind`, `max_sends_per_hour`, `quiet_hours` (JSON), `expires_at`, `conditions` (JSON), `created_at`, `updated_at`.
- **`TelegramSendDryRunRequest`** – запрос на «сухой прогон»: `command_id`, `policy_id`, `provider_chat_id`, `variables` (JSON), `source_context` (JSON).
- **`TelegramSendDryRunResponse`** – результат прогона: `outbound_message_id`, `policy_id`, `template_id`, `account_id`, `provider_chat_id`, `rendered_text`, `rendered_preview_hash`, `status` (`"allowed"`), `event_id`.

Функция `object_from_pairs` конструирует `serde_json::Value` из пар (ключ, значение).

### Ошибки (`errors.rs`)

`AutomationError` включает:
- `InvalidRequest`, `PolicyNotFound`, `PolicyDisabled`, `ChatNotAllowed`, `MissingTemplateVariable`, `UndeclaredTemplateVariable`
- Transparent‑варианты: `EventEnvelope`, `EventStore`, `ObservationStore`, `Sqlx`.

### Dry‑run‑отправка (`dry_run.rs`)

Основная функция `dry_run_send`:

1. Валидирует запрос и actor_id.
2. Загружает политику и шаблон через `AutomationStore::policy_with_template`.
3. Вызывает `evaluate_policy` для проверки и рендеринга текста.
4. Генерирует `outbound_message_id` на основе SHA‑256 от конкатенации `command_id`, `policy_id`, `chat_id` и хеша рендеренного текста.
5. В транзакции:
   - Вставляет запись в `telegram_outbound_messages` с `send_mode = 'dry_run'` и `status = 'allowed'` (с `ON CONFLICT DO NOTHING`).
   - Формирует событие `NewEventEnvelope` с типом `automation.telegram_send.dry_run`, связывая политику и outbound‑сообщение.
   - Отправляет событие через `EventStore::append_in_transaction`.
   - Вызывает `capture_dry_run_observation`.
6. Фиксирует транзакцию и возвращает `TelegramSendDryRunResponse`.

### Проверка политики (`policy.rs`)

`evaluate_policy(policy, template, request)`:
1. Проверяет, что политика `enabled`.
2. Проверяет `expires_at` (если задан и наступил – ошибка).
3. Проверяет, что `provider_chat_id` входит в `allowed_chat_ids`.
4. Проверяет, что все ключи в `request.variables` присутствуют в `template.required_variables` (иначе `UndeclaredTemplateVariable`).
5. Для каждой требуемой переменной ищет её в `variables`; значение должно быть непустой строкой (иначе `MissingTemplateVariable`).
6. Производит замену `{{variable}}` в `body_template` и возвращает итоговый текст.

### Фиксация наблюдений (`evidence.rs`)

Три функции создают observation и привязывают его к сущности домена `"automation"`:

- **`capture_template_observation`** – observation типа `AUTOMATION_TEMPLATE`, связь с entity_kind `"template"`.
- **`capture_policy_observation`** – observation типа `AUTOMATION_POLICY`, связь с entity_kind `"policy"`.
- **`capture_dry_run_observation`** – observation типа `TELEGRAM_OUTBOUND_MESSAGE`, связь с entity_kind `"telegram_outbound_message"`.

### Константы (`constants.rs`)

- `AUTOMATION_SEND_DRY_RUN_EVENT_TYPE` – `"automation.telegram_send.dry_run"`
- `AUTOMATION_SOURCE_KIND` – `"automation_policy"`
- `AUTOMATION_SOURCE_PROVIDER` – `"local_policy_engine"`

### Генерация идентификаторов (`ids.rs`)

`sha256_hex(bytes)` – возвращает строку вида `sha256:{hex}`, вычисленную через `Sha256`.

### Преобразование строк (`rows.rs`)

- `row_to_template(PgRow)` – собирает `AutomationTemplate`.
- `row_to_policy(PgRow)` – собирает `AutomationPolicy`; строковые массивы извлекаются через `string_vec_from_value`, которая ожидает JSON-массив строк.

---

## Технический контекст

- **Язык**: Rust
- **База данных**: PostgreSQL через `sqlx` (асинхронные запросы)
- **Сериализация**: `serde` / `serde_json`
- **Дата/время**: `chrono`
- **Обработка ошибок**: `thiserror`
- **Хеширование**: `sha2`
- **Наблюдения**: фиксируются через `platform::observations` (тип `ObservationStore`) и связываются с сущностями через `link_domain_entity_in_transaction`.
- **События**: сохраняются через `platform::events::EventStore`.
- **Отношения между сущностями**: управляются через `domains::relationships` (порт `RelationshipReviewPort`).
- **Контекстные пакеты**: проверяются через `engines::context_packs::ContextPackStore`.

Всё взаимодействие с базой данных внутри домена обёрнуто в соответствующие Error-типы (часто с transparent-передачей ошибок `sqlx` и других доменов).
```

### Source coverage / Покрытие источников

| Source file | Facts covered |
|-------------|---------------|
| `backend/src/domains/tasks/core/errors.rs` | Перечисление вариантов `TaskCoreError` |
| `backend/src/domains/tasks/core/evidence.rs` | Структура `TaskEvidence`, методы `list`/`add`, логика связывания с observation при `source_type = "observation"` |
| `backend/src/domains/tasks/core/external_identities.rs` | Модель `ExternalTaskIdentity`, метод `list` хранилища |
| `backend/src/domains/tasks/core/obligation_links.rs` | Структура `ObligationTaskLinkStore`, метод `link_fulfillment_task` |
| `backend/src/domains/tasks/core/observation_links.rs` | Функции `materialize_task_observation_link_in_transaction`, `materialize_task_entity_link_in_transaction` |
| `backend/src/domains/tasks/core/provider_store.rs` | `TaskProviderStore`, методы `list`/`create`/`create_with_origin`, логика создания observation и связывания |
| `backend/src/domains/tasks/core/providers.rs` | Поля модели `TaskProviderAccount` |
| `backend/src/domains/tasks/core/relations.rs` | `TaskRelationStore`, методы `list`/`link`, материализация observation и relationship (с `UserConfirmed`, evidence) |
| `backend/src/domains/tasks/core/subtasks.rs` | `TaskSubtaskStore`, методы `list`/`add`/`add_with_source`, связывание с observation |
| `backend/src/domains/tasks/health.rs` | `TaskWatchtowerService`: `overdue`, `waiting_too_long`, `without_context`, `stale_tasks`, `cycle_time`, `workload`; ошибки `TaskHealthError` |
| `backend/src/domains/tasks/intelligence.rs` | `TaskIntelligenceService`: `calculate_priority`, `calculate_risk`, `calculate_readiness`, `detect_missing_context`, `suggest_next_action`; ошибки |
| `backend/src/domains/tasks/mod.rs` | Перечень подмодулей домена `tasks` |
| `backend/src/domains/tasks/ports.rs` | Псевдонимы `TaskCommandPort`, `TaskCandidatePort`, `ObligationTaskLinkPort` |
| `backend/src/domains/tasks/rules.rs` | `TaskRule`, `TaskRuleStore` (`list`/`create`/`delete`), `TaskTemplate`, `TaskTemplateStore` (`list`), ошибки `TaskRuleError` |
| `backend/src/domains/tasks/service.rs` | Реэкспорт `command_service` |
| `backend/src/domains/tasks/sync.rs` | `export_task_md`, `export_task_json`, ошибки `TaskSyncError` |
| `backend/src/engines/automation.rs` | Публичный API модуля автоматизации |
| `backend/src/engines/automation/constants.rs` | Константы `AUTOMATION_SEND_DRY_RUN_EVENT_TYPE`, `AUTOMATION_SOURCE_KIND`, `AUTOMATION_SOURCE_PROVIDER` |
| `backend/src/engines/automation/dry_run.rs` | Функция `dry_run_send`: загрузка политики, рендеринг, запись outbound‑сообщения, событие, observation |
| `backend/src/engines/automation/errors.rs` | Варианты `AutomationError` |
| `backend/src/engines/automation/evidence.rs` | Функции `capture_template_observation`, `capture_policy_observation`, `capture_dry_run_observation` |
| `backend/src/engines/automation/ids.rs` | Функция `sha256_hex` |
| `backend/src/engines/automation/models.rs` | Структуры `AutomationPolicy`, `AutomationTemplate`, `TelegramSendDryRunRequest`, `TelegramSendDryRunResponse`, `object_from_pairs` |
| `backend/src/engines/automation/policy.rs` | Логика `evaluate_policy`: проверки enabled/expired/chat/variables, рендеринг |
| `backend/src/engines/automation/rows.rs` | Функции `row_to_template`, `row_to_policy`, `string_vec_from_value` |

### Drift candidates / Кандидаты на drift

В предоставленном наборе исходных файлов явных расхождений кода с документацией не обнаружено. Однако стоит отметить следующую неполноту, которая может привести к drift в будущем:

1. **Отсутствующие подмодули домена `tasks`**
   `mod.rs` объявляет модули `api`, `brain`, `candidates`, `command_service`, но их реализация не включена в контекст. Если в вики уже существуют страницы для этих подмодулей, они могут не отражать актуальное состояние кода.

2. **Хранилище `AutomationStore`**
   Используется в `dry_run.rs` (метод `policy_with_template`) и публично экспортируется, но его исходный код отсутствует в чанке. Полноценное описание `AutomationStore` в вики без исходников невозможно.

С точки зрения данного чанка дрифт не визуализируется, так как нет опубликованной русской вики для сравнения.
