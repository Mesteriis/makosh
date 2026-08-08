---
chunk_id: 031-source-backend-part-011
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 031-source-backend-part-011 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Необходимо обновить страницу `components/backend.md` русской wiki, чтобы она отражала актуальную структуру HTTP-обработчиков бэкенда, их зоны ответственности, используемые сервисы/хранилища и поведение API — строго на основании исходного кода, предоставленного в чанке `031-source-backend-part-011`. Предыдущая версия страницы отсутствует в контексте, поэтому предлагается новая редакция.

## Предложенные страницы

`components/backend.md`:

```markdown
# Backend (бэкенд)

## Обзор

Бэкенд реализован на Rust с использованием фреймворка **axum**. Обработчики HTTP-запросов сосредоточены в `backend/src/app/handlers/` и сгруппированы по доменам:

- **persons** — работа с персонами, контактами, их профилями и «памятью»;
- **projects** — управление проектами и их связями с сообщениями/документами;
- **relationships** — управление отношениями между сущностями;
- **review** — инбокс для элементов, требующих ревью и принятия решений;
- **settings** — системные настройки и учётные записи провайдеров;
- **signal_hub** — источники сигналов, политики, профили, подключения;
- **tasks** — задачи, подзадачи, кандидаты, здоровье, аналитика, правила.

Каждый обработчик получает состояние приложения через `State<AppState>`, извлекает пул соединений с базой данных и делегирует бизнес-логику соответствующим **сервисам** (`Service`) или напрямую обращается к **хранилищам** (`Store`). Чтение выполняется через методы `Store`, запись — через `Service` (например, `PersonCommandService`, `TaskCommandService`).

> **Примечание:** Настоящий документ основан на исходных файлах чанка `031-source-backend-part-011`. Некоторые подмодули (перечисленные в `mod.rs`, но не включённые в чанк) отсутствуют, и их описание опущено. Также файлы `projects/mod.rs` и `signal_hub.rs` были обрезаны, поэтому описание неполное.

## Структура обработчиков

Все модули обработчиков содержат:
- `mod.rs` — объявление модуля и реэкспорт публичных функций-обработчиков;
- при необходимости `support.rs` — общие импорты;
- дополнительные файлы с реализацией конкретных эндпоинтов и моделей данных.

---

## Обработчики persons

### 1. Профиль (profile)

Реализованы в `profile/`.

#### Чтение и поиск

- **`get_persons`** (`legacy.rs`) — возвращает список `EnrichedPerson` с фильтрацией:
  - `favorites_only` — только избранные;
  - `limit` — лимит (по умолчанию 50).
- **`get_person`** (`legacy.rs`) — конкретная обогащённая персона по `person_id`. При отсутствии возвращает `PersonIdentityNotFound`.
- **`get_person_search`** (`search.rs`) — полнотекстовый поиск персон через `PersonEnrichmentStore.search_persons`. Параметры: `q` (обязательный), `limit` (по умолчанию 20). Пустой запрос вызывает ошибку.

#### Действия

- **`post_person_fingerprint`** (`actions.rs`) — сбор последних 50 сообщений, связанных с персоной, и выполнение ручного снятия «отпечатка» (`PersonCommandService.fingerprint_person_manual`).
- **`post_person_favorite`** (`actions.rs`) — переключение флага «избранное» (`toggle_favorite_manual`). Возвращает `{"is_favorite": true/false}`.
- **`put_person_notes`** (`actions.rs`) — обновление заметок о персоне (`set_notes_manual`). Возвращает `{"saved": true}`.

#### Persona владельца

- **`get_owner_persona`** (`owner.rs`) — возвращает текущую persona владельца системы (или `null`, если не задана).
- **`put_owner_persona`** (`owner.rs`) — устанавливает persona владельца по переданному `person_id`.

#### Управление personas

- **`get_personas`** (`personas.rs`) — список всех personas (с лимитом, по умолчанию 50). Отдаются в виде `PersonaReadModel`.
- **`get_persona`** (`personas.rs`) — персона по `persona_id`.
- **`put_persona`** (`personas.rs`) — обновление persona:
  - `identity.display_name` — новое отображаемое имя;
  - `is_self` — может быть только `true` или отсутствовать; попытка установить `false` вызывает ошибку `InvalidPersonaQuery`.

**Модель `PersonaReadModel`** (описана в `models.rs`):
```rust
struct PersonaReadModel {
    persona_id: String,
    persona_type: PersonaType,
    is_self: bool,
    identity: { display_name: String, email_address: String },
    communication: { primary_email: String },
    compatibility: { legacy_person_id: String, legacy_route: "/api/v1/persons" },
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### 2. Память (memory)

Реализованы в `memory.rs`.

#### Факты (Person Facts)

- **`get_person_facts`** — список фактов `PersonFact` для персоны.
- **`post_person_fact`** — создание/обновление факта.
  - Поля запроса: `fact_type`, `value`, `source` (по умолчанию `"manual"`), `confidence` (по умолчанию `1.0`).
  - Используется `PersonCommandService.upsert_person_fact_manual`.

#### Карточки памяти (Memory Cards)

- **`get_person_memory_cards`** — список `PersonMemoryCard`.
- **`post_person_memory_card`** — создание/обновление карточки.
  - Поля: `title`, `description`, `source` (по умолчанию `"manual"`), `importance` (по умолчанию `5`).
  - Используется `upsert_person_memory_card_manual`.

#### Предпочтения (Preferences)

- **`get_person_preferences`** — список `PersonPreference`.
- **`post_person_preference`** — создание/обновление предпочтения.
  - Поля: `preference_type`, `value`, `source` (по умолчанию `"manual"`).
  - Используется `upsert_person_preference_manual`.

#### Таймлайн отношений (Relationship Timeline)

- **`get_person_timeline`** — хронология событий отношений `RelationshipEvent`. Поддерживает параметр `limit` (по умолчанию 50).
- **`post_relationship_event`** — добавление события отношения.
  - Поля: `event_type`, `title`, `description` (опционально), `occurred_at` (обязательно), `source`, `related_entity_id`, `related_entity_kind`.
  - Вызывает `PersonCommandService.add_relationship_event_manual`, передавая структуру `NewRelationshipEvent`.

---

## Обработчики projects

Реализованы в `projects/mod.rs` (файл обрезан, доступна часть кода).

- **`get_projects`** — список проектов через `ProjectStore.list_projects` с параметром `limit`.
- **`get_project_detail`** — детальная информация о проекте (`ProjectDetail`) по `project_id`. При отсутствии возвращает `ProjectNotFound`.
- **`get_project_link_candidates`** — для указанного проекта собирает кандидатов на привязку:
  1. Сообщения через `ProjectStore.matching_project_messages`;
  2. Документы через `ProjectStore.matching_project_documents`.
  Для каждого кандидата:
  - вычисляется `graph_node_id`;
  - создаётся запись ревью через `application::ensure_project_link_candidate_review_item`;
  - определяется состояние ревью (`review_state`, по умолчанию `Suggested`).
  Результаты сортируются по дате (новые сверху) и обрезаются лимитом (по умолчанию 25).
- **`put_project_link_review`** — принимает команду ревью привязки (код обрезан). Видно, что выполняется аудит и используется `ProjectLinkReviewStore`.

---

## Обработчики relationships

Реализованы в `relationships/handlers.rs`.

- **`get_v1_relationships`** — возвращает список отношений. Поддерживаемые комбинации фильтров:
  - только `review_state` — фильтр по состоянию ревью;
  - `entity_kind` + `entity_id` — фильтр по сущности.
  Запрещено комбинировать `review_state` с сущностными фильтрами. Лимит валидируется: разрешён диапазон 1–100, по умолчанию 50.
- **`put_v1_relationship_review`** — обновляет состояние ревью отношения через `RelationshipReviewApplicationService.review_manual`. Аудируется с actor "makosh-frontend".

Константы: actor — `"makosh-frontend"`, лимит по умолчанию — 50, минимум — 1, максимум — 100.

---

## Обработчики review

Реализованы в `review.rs`. Реализуют рабочий процесс ревью-инбокса.

- **`get_v1_review_items`** — получение элементов с фильтром `status`:
  - `"active"` (по умолчанию) — открытые элементы;
  - `"all"` — все элементы;
  - конкретный статус (напр. `Approved`, `Dismissed`) — фильтр по одному статусу.
  Лимит 1–100, по умолчанию 50.
- **`post_v1_review_items`** — создание нового элемента ревью с доказательствами.
  - Принимает `item_kind`, `title`, `summary`, `confidence`, `metadata`, список `evidence` (каждый с `observation_id`, `evidence_role`, `metadata`).
  - Вызывает `ReviewInboxStore.create_with_evidence`.
- **`post_v1_review_item_approve`** — переводит элемент в статус `Approved`.
- **`post_v1_review_item_dismiss`** — в `Dismissed`.
- **`post_v1_review_item_archive`** — в `Archived`.
- **`post_v1_review_item_take`** — в `InReview`.
- **`post_v1_review_item_promote`** — продвижение элемента в целевую сущность.
  - Принимает `target_domain`, `target_entity_kind`, `target_entity_id`.
  - Создаёт observation типа `REVIEW_TRANSITION` через `ObservationStore.capture`.
  - Делегирует продвижение в `ReviewPromotionService.promote_with_observation`.

Переходы статусов обрабатываются через `ReviewInboxService.transition_status_from_manual`.

---

## Обработчики settings

Реализованы в `settings/mod.rs`.

- **`get_application_settings`** — возвращает все настройки (`list_settings`).
- **`get_application_settings_accounts`** — список аккаунтов провайдеров коммуникаций (`CommunicationProviderAccountStore.list`).
- **`put_application_setting`** — обновляет значение настройки по ключу. Actor: `"makosh-frontend"`. Операция аудируется (`NewApiAuditRecord::application_setting_set`).

---

## Обработчики signal_hub

Реализованы в `signal_hub.rs` (файл обрезан). Предоставляют управление источниками сигналов, профилями, политиками и подключениями.

Наблюдаемые обработчики (частично):

- **`get_signal_hub_sources`**, **`get_signal_hub_source`** — чтение источников.
- **`get_signal_hub_capabilities`** — список возможностей (фильтрация по `source_code`, `connection_id`).
- **`post_signal_hub_restore_system_fixture`** — восстановление системных fixture-источников.
- **`get_signal_hub_fixture_sources`** — список fixture-источников.
- **`get_signal_hub_profiles`** — список профилей.
- **`post_signal_hub_profile`** — создание профиля (поля: `code`, `display_name`, `description`, `source_policies`).
- **`post_signal_hub_apply_profile`** — применение профиля по коду.
- **`patch_signal_hub_profile`** — обновление профиля (поля опциональны: `display_name`, `description`, `source_policies`).
- **`delete_signal_hub_profile`** — удаление профиля.
- **`post_signal_hub_enable_source`** / **`post_signal_hub_disable_source`** — включение/выключение источника.
- **`get_signal_hub_connections`** — список подключений (дальнейший код обрезан).

Используемые сервисы/хранилища: `SignalHubStore`, `SignalHubProfileService`, `SignalHubControlService`, `SignalHubCapabilityService`, `SignalHubConnectionService`, `SignalFixtureSourceService`, `ApplicationSettingsStore`, `EventStore`, `SignalHubHealthService`, `SignalHubReplayService` (упомянуты в импортах).

---

## Обработчики tasks

### 1. CRUD задач (items.rs)

- **`get_tasks`** — список задач с фильтрами (`status`, `project_id`, `source_type`, `limit`).
- **`post_task`** — создание задачи через `TaskCommandService.create_task_manual`.
- **`get_task`** — получение задачи по `task_id`.
- **`put_task`** — обновление задачи (`update_task_manual`).
- **`post_task_status`** — установка статуса (`set_status_manual`).
- **`post_task_archive`** — архивирование (`archive_manual`).

### 2. Контекстные пакеты, доказательства, связи, подзадачи (core_records.rs)

- **Контекстный пакет** (`TaskContextPack`):
  - `get_task_context_pack` / `post_task_context_pack` — чтение и upsert пакета. Поля для upsert: `summary`, `open_questions` (JSON), `blockers`, `risks`, `suggested_next_action`.
- **Доказательства** (`TaskEvidence`):
  - `get_task_evidence` / `post_task_evidence` — список и добавление (`TaskCommandService.add_evidence`). Поля: `source_type`, `source_id`, `quote`, `confidence`.
- **Связи** (`TaskRelation`):
  - `get_task_relations` / `post_task_relation` — список и добавление связи (`add_relation_manual`). Поля: `entity_type`, `entity_id`, `relation_type`.
- **Чеклист** (`TaskChecklist`):
  - `get_task_checklist` / `post_task_checklist` — получение и установка чеклиста (`set_checklist_manual`). Элементы передаются как JSON `items`, опционально `source`.
- **Подзадачи** (`TaskSubtask`):
  - `get_task_subtasks` / `post_task_subtask` — список и добавление подзадачи (`add_subtask_manual`). Поля: `child_task_id`, `sort_order` (по умолчанию 0).
- **Внешние идентификаторы** (`ExternalTaskIdentity`):
  - `get_task_external` — список внешних идентификаторов задачи.

### 3. Кандидаты задач (candidates.rs)

- **`get_task_candidates`** — список кандидатов (с лимитом).
- **`put_task_candidate_review`** — ревью кандидата через `TaskCandidateReviewApplicationService.review_manual`. Аудируется.

### 4. Здоровье (health.rs)

- **`get_task_watchtower`** — сводка:
  - `overdue` — просроченные задачи;
  - `stale` — застойные (без изменений за `days` дней, по умолчанию 14);
  - `without_context` — задачи без контекстного пакета.
- **`get_task_health`** — метрики `workload` и `cycle_time`.
- **`get_task_analytics`** — возвращает ссылку на `/tasks/health` и `/tasks/watchtower`.

Используется `TaskWatchtowerService`.

### 5. Интеллектуальные функции (intelligence.rs)

- **`post_task_analyze`** — анализ задачи (`TaskCommandService.analyze_runtime`): приоритет, риск, готовность, недостающий контекст, следующее действие.
- **`get_task_export`** — экспорт задачи в JSON (по умолчанию) или Markdown (`?format=md`).
- **`post_task_brain`** — объяснение задачи по текстовому запросу (`TaskBrainService.explain_task`).
- **`get_task_search`** — семантический поиск задач (`TaskBrainService.search_tasks`).
- **`get_task_daily_brief`** — ежедневная сводка на основе ИИ (`TaskBrainService.daily_brief`).

### 6. Провайдеры (providers.rs)

- **`get_task_providers`** — список аккаунтов провайдеров (`TaskProviderStore.list`).
- **`post_task_provider`** — создание аккаунта (`TaskProviderStore.create`). Поля: `provider`, `account_name`.

### 7. Правила и шаблоны (rules.rs)

- **`get_task_rules`** — список правил (`TaskRule`).
- **`post_task_rule`** — создание правила (имя, описание, DSL или config, approval_mode).
- **`delete_task_rule`** — удаление правила.
- **`get_task_templates`** — список шаблонов задач (`TaskTemplate`).

---

## Примечание о маршрутах

Конкретные URL-пути и HTTP-методы не извлекаются из представленных исходников — видны только функции-обработчики с сигнатурами, указывающими на использование `Path`, `Query`, `State` и `Json`. Точное сопоставление маршрутов определяется в другом месте (вероятно, в файлах конфигурации маршрутизации axum).
```

## Покрытие источников

- **`backend/src/app/handlers/persons/memory.rs`** — обработчики `get_person_facts`, `post_person_fact`, `get_person_memory_cards`, `post_person_memory_card`, `get_person_preferences`, `post_person_preference`, `get_person_timeline`, `post_relationship_event`; структуры запросов/ответов; вызовы `PersonCommandService` и хранилищ.
- **`backend/src/app/handlers/persons/mod.rs`** — состав подмодулей (compatibility, errors, health, history, identity, intelligence, investigator, memory, profile, support); реэкспорт.
- **`backend/src/app/handlers/persons/profile.rs`** — реэкспорт подмодулей (actions, legacy, models, owner, personas, search).
- **`backend/src/app/handlers/persons/profile/actions.rs`** — обработчики `post_person_fingerprint`, `post_person_favorite`, `put_person_notes`; модели запросов; вызовы `PersonCommandService`.
- **`backend/src/app/handlers/persons/profile/legacy.rs`** — обработчики `get_persons`, `get_person`; использование `PersonEnrichmentStore`.
- **`backend/src/app/handlers/persons/profile/models.rs`** — структуры `PersonListResponse`, `PersonaListResponse`, `PersonaReadModel`, функция `persona_read_model`.
- **`backend/src/app/handlers/persons/profile/owner.rs`** — обработчики `get_owner_persona`, `put_owner_persona`; структуры запроса/ответа; использование `PersonCommandService`, `PersonProjectionStore`.
- **`backend/src/app/handlers/persons/profile/personas.rs`** — обработчики `get_personas`, `get_persona`, `put_persona`; валидация `is_self`; использование `PersonProjectionStore`, `PersonCommandService`.
- **`backend/src/app/handlers/persons/profile/search.rs`** — обработчик `get_person_search`; вызов `PersonEnrichmentStore.search_persons`.
- **`backend/src/app/handlers/persons/support.rs`** — список импортов, отражающий зависимости обработчиков persons.
- **`backend/src/app/handlers/projects/mod.rs`** — обработчики `get_projects`, `get_project_detail`, `get_project_link_candidates`, `put_project_link_review` (частично); структуры запросов/ответов; взаимодействие с `ProjectStore`, `ProjectLinkReviewStore`, `ObservationStore`.
- **`backend/src/app/handlers/relationships/handlers.rs`** — обработчики `get_v1_relationships`, `put_v1_relationship_review`; валидация параметров; вызов `RelationshipStore`, `RelationshipReviewApplicationService`, аудит.
- **`backend/src/app/handlers/relationships/mod.rs`** — реэкспорт обработчиков и моделей.
- **`backend/src/app/handlers/relationships/models.rs`** — типы `RelationshipListQuery`, `RelationshipReviewApiRequest`, `RelationshipListResponse`.
- **`backend/src/app/handlers/review.rs`** — обработчики `get_v1_review_items`, `post_v1_review_items`, `post_v1_review_item_approve`, `post_v1_review_item_dismiss`, `post_v1_review_item_archive`, `post_v1_review_item_take`, `post_v1_review_item_promote`; управление статусами, создание observation, продвижение; использование `ReviewInboxStore`, `ReviewInboxService`, `ReviewPromotionService`, `ObservationStore`.
- **`backend/src/app/handlers/settings/mod.rs`** — обработчики `get_application_settings`, `get_application_settings_accounts`, `put_application_setting`; вызовы `ApplicationSettingsStore`, `CommunicationProviderAccountStore`, аудит.
- **`backend/src/app/handlers/signal_hub.rs`** (truncated) — обработчики для источников, профилей, политик, подключений, возможностей, fixture; структуры запросов/ответов; используемые сервисы (SignalHubStore, SignalHubProfileService, SignalHubControlService, SignalHubCapabilityService и др.).
- **`backend/src/app/handlers/tasks/candidates.rs`** — обработчики `get_task_candidates`, `put_task_candidate_review`; аудит; вызов `TaskCandidateReviewApplicationService`.
- **`backend/src/app/handlers/tasks/core_records.rs`** — обработчики контекстных пакетов, доказательств, связей, чеклистов, подзадач, внешних идентификаторов; вызовы `TaskCommandService` и соответствующих хранилищ.
- **`backend/src/app/handlers/tasks/health.rs`** — обработчики `get_task_watchtower`, `get_task_health`, `get_task_analytics`; метрики overdue, stale, without_context, workload, cycle_time.
- **`backend/src/app/handlers/tasks/intelligence.rs`** — обработчики `post_task_analyze`, `get_task_export`, `post_task_brain`, `get_task_search`, `get_task_daily_brief`; вызовы `TaskCommandService`, `TaskBrainService`.
- **`backend/src/app/handlers/tasks/items.rs`** — CRUD обработчики задач: `get_tasks`, `post_task`, `get_task`, `put_task`, `post_task_status`, `post_task_archive`.
- **`backend/src/app/handlers/tasks/mod.rs`** — реэкспорт подмодулей (candidates, core_records, health, intelligence, items, providers, rules, support).
- **`backend/src/app/handlers/tasks/providers.rs`** — обработчики `get_task_providers`, `post_task_provider`.
- **`backend/src/app/handlers/tasks/rules.rs`** — обработчики `get_task_rules`, `post_task_rule`, `delete_task_rule`, `get_task_templates`.

## Исходные файлы

- [`backend/src/app/handlers/persons/memory.rs`](../../../../backend/src/app/handlers/persons/memory.rs)
- [`backend/src/app/handlers/persons/mod.rs`](../../../../backend/src/app/handlers/persons/mod.rs)
- [`backend/src/app/handlers/persons/profile.rs`](../../../../backend/src/app/handlers/persons/profile.rs)
- [`backend/src/app/handlers/persons/profile/actions.rs`](../../../../backend/src/app/handlers/persons/profile/actions.rs)
- [`backend/src/app/handlers/persons/profile/legacy.rs`](../../../../backend/src/app/handlers/persons/profile/legacy.rs)
- [`backend/src/app/handlers/persons/profile/models.rs`](../../../../backend/src/app/handlers/persons/profile/models.rs)
- [`backend/src/app/handlers/persons/profile/owner.rs`](../../../../backend/src/app/handlers/persons/profile/owner.rs)
- [`backend/src/app/handlers/persons/profile/personas.rs`](../../../../backend/src/app/handlers/persons/profile/personas.rs)
- [`backend/src/app/handlers/persons/profile/search.rs`](../../../../backend/src/app/handlers/persons/profile/search.rs)
- [`backend/src/app/handlers/persons/support.rs`](../../../../backend/src/app/handlers/persons/support.rs)
- [`backend/src/app/handlers/projects/mod.rs`](../../../../backend/src/app/handlers/projects/mod.rs)
- [`backend/src/app/handlers/relationships/handlers.rs`](../../../../backend/src/app/handlers/relationships/handlers.rs)
- [`backend/src/app/handlers/relationships/mod.rs`](../../../../backend/src/app/handlers/relationships/mod.rs)
- [`backend/src/app/handlers/relationships/models.rs`](../../../../backend/src/app/handlers/relationships/models.rs)
- [`backend/src/app/handlers/review.rs`](../../../../backend/src/app/handlers/review.rs)
- [`backend/src/app/handlers/settings/mod.rs`](../../../../backend/src/app/handlers/settings/mod.rs)
- [`backend/src/app/handlers/signal_hub.rs`](../../../../backend/src/app/handlers/signal_hub.rs)
- [`backend/src/app/handlers/tasks/candidates.rs`](../../../../backend/src/app/handlers/tasks/candidates.rs)
- [`backend/src/app/handlers/tasks/core_records.rs`](../../../../backend/src/app/handlers/tasks/core_records.rs)
- [`backend/src/app/handlers/tasks/health.rs`](../../../../backend/src/app/handlers/tasks/health.rs)
- [`backend/src/app/handlers/tasks/intelligence.rs`](../../../../backend/src/app/handlers/tasks/intelligence.rs)
- [`backend/src/app/handlers/tasks/items.rs`](../../../../backend/src/app/handlers/tasks/items.rs)
- [`backend/src/app/handlers/tasks/mod.rs`](../../../../backend/src/app/handlers/tasks/mod.rs)
- [`backend/src/app/handlers/tasks/providers.rs`](../../../../backend/src/app/handlers/tasks/providers.rs)
- [`backend/src/app/handlers/tasks/rules.rs`](../../../../backend/src/app/handlers/tasks/rules.rs)

## Кандидаты на drift

Из предоставленного контекста явных расхождений между кодом и документацией или ADR не обнаружено. Однако:

- Файлы `backend/src/app/handlers/projects/mod.rs` и `backend/src/app/handlers/signal_hub.rs` обрезаны, поэтому часть поведения обработчиков может быть не задокументирована.
- Подмодули `backend/src/app/handlers/persons/` (compatibility, errors, health, history, identity, intelligence, investigator) перечислены в `mod.rs`, но отсутствуют в чанке — их описание не включено. Если wiki ожидает их покрытия, это может быть расхождением.
