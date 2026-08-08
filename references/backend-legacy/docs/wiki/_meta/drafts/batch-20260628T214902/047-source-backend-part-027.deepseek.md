### Summary / Резюме

Обновить страницу `components/backend.md` русской Obsidian‑wiki: добавить описание backend‑доменов `organizations` (память, сервис команд, workflow‑сущности) и `persons` (API‑проекция, аналитика, участники) на основе представленных исходных файлов. Страница должна отражать только те факты, которые подтверждены встроенным кодом (структуры, методы, SQL‑запросы, ошибки).

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend-компоненты

## Организации (`organizations`)

Домен `organizations` содержит подмодули: `api`, `core`, `enrichment`, `finance`, `health`, `investigator`, `memory`, `ports`, `service`, `workflows`.

### Память организаций (`memory`)

Файл `organizations/memory.rs` реализует хранилища для фактов, памятных карточек, предпочтений и требуемых документов.

#### OrgFact

```rust
pub struct OrgFact {
    pub id: String,
    pub organization_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Хранилище `OrgFactStore`:
- `list(org_id)` – возвращает все факты организации, сортировка `created_at DESC`.
- `upsert(org_id, fact_type, value, source, confidence)` – вставляет факт (`ON CONFLICT DO NOTHING`). Если конфликт (не указан уникальный constraint), новая запись не создаётся; метод возвращает вставленную строку.
- `decay_unverified(threshold_days)` – для записей, где `last_verified_at IS NULL` или старше `threshold_days` дней, уменьшает `confidence` вдвое (`* 0.5`) и обновляет `updated_at`. Возвращает количество затронутых строк.

Таблица: `organization_facts` (поля: id, organization_id, fact_type, value, source, confidence, last_verified_at, valid_from, valid_to, is_active, created_at, updated_at).

#### OrgMemoryCard

```rust
pub struct OrgMemoryCard {
    pub id: String,
    pub organization_id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}
```

Хранилище `OrgMemoryCardStore`:
- `list(org_id)` – записи организации, сортировка `importance DESC, created_at DESC`.
- `upsert(org_id, title, description, source, importance)` – `ON CONFLICT DO NOTHING`.

Таблица: `organization_memory_cards` (поля: id, organization_id, title, description, source, confidence, importance, created_at, last_verified_at).

#### OrgPreference

```rust
pub struct OrgPreference {
    pub id: String,
    pub organization_id: String,
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Хранилище `OrgPreferenceStore`:
- `list(org_id)` – сортировка `preference_type`.
- `upsert(org_id, ptype, value, source)` – `ON CONFLICT (organization_id, preference_type) DO UPDATE SET value, source, updated_at`. При конфликте обновляет существующую запись.

Таблица: `organization_preferences` (поля: id, organization_id, preference_type, value, source, confidence, last_verified_at, created_at, updated_at).

#### OrgRequiredDocument

```rust
pub struct OrgRequiredDocument {
    pub id: String,
    pub organization_id: String,
    pub document_type: String,
    pub description: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}
```

Хранилище `OrgRequiredDocStore`:
- `list(org_id)` – сортировка `document_type`.

Таблица: `organization_required_documents` (поля: id, organization_id, document_type, description, source, confidence, created_at).

#### Ошибки

`OrgMemoryError` – варианты `Sqlx` (прозрачный) и `NotFound`.

### Порты (`ports`)

`organizations/ports.rs` реэкспортирует:
- `OrganizationStore` как `OrganizationCommandPort`
- `OrgContactLinkStore` как `OrganizationContactLinkPort`

### Сервис команд (`service`)

`OrganizationCommandService` (файл `organizations/service.rs`) оборачивает мутации организаций в **наблюдения** (observations). Каждый публичный метод сначала вызывает `capture_manual` для создания записи `Observation` (origin `Manual`), затем вызывает соответствующий store.

Публичные методы:
- `create_organization_manual(display_name, org_type)` – создаёт организацию.
- `update_organization_manual(organization_id, update)` – принимает `OrganizationUpdate`, обновляет организацию.
- `archive_organization_manual(organization_id)` – архивирует организацию.
- `add_identity_manual(organization_id, identity_type, identity_value, requested_source)` – добавляет идентификатор, вызывает `OrgIdentityStore::upsert_with_observation`.
- `add_alias_manual(organization_id, name, alias_type, requested_source)` – добавляет алиас, вызывает `OrgAliasStore::add_with_observation`.
- `add_department_manual(organization_id, name, description, parent_id)` – добавляет отдел, вызывает `OrgDepartmentStore::add_with_observation`.
- `link_contact_manual(organization_id, person_id, role, department, requested_source)` – связывает контакт с персоной, вызывает `OrgContactLinkStore::link_with_observation`.
- `apply_enrichment_manual(organization_id, result_id)` – применяет результат обогащения, вызывает `OrgEnrichmentStore::apply_with_observation`.
- `toggle_watchlist_manual(organization_id)` – переключает статус списка наблюдения, вызывает `OrgHealthStore::toggle_watchlist_with_observation`.

Приватный метод `capture_manual` создаёт `NewObservation` (тип observation_kind указан вручную, origin `Manual`, текущее время) и сохраняет через `ObservationStore::capture`.

Ошибки: `OrganizationCommandServiceError` – прозрачные варианты `Observation`, `Organization`, `Core`, `Enrichment`, `Health`.

### Workflows

Поддомен `organizations/workflows` состоит из модулей: `errors`, `playbooks`, `portals`, `procedures`, `templates`, `timeline`.

#### Playbooks (`playbooks.rs`)

`OrgPlaybook` и хранилище `OrgPlaybookStore` с методом `list(org_id)`.
Таблица: `organization_playbooks`. Поля: id, organization_id, name, trigger_condition, steps (JSON), approval_mode, enabled, last_run_at, created_at, updated_at.

#### Portals (`portals.rs`)

`OrgPortal` и хранилище `OrgPortalStore`:
- `list(org_id)` – сортировка `portal_type, name`.
- `add(org_id, name, url, portal_type)` – вставка записи.

Таблица: `organization_portals`. Поля: id, organization_id, name, url, portal_type, login_hint, secret_reference, last_used_at, notes, created_at.

#### Procedures (`procedures.rs`)

`OrgProcedure` и хранилище `OrgProcedureStore` с методом `list(org_id)`.
Таблица: `organization_procedures`. Поля: id, organization_id, name, description, steps (JSON), source, confidence, last_used_at, created_at, updated_at.

#### Templates (`templates.rs`)

`OrgTemplate` и хранилище `OrgTemplateStore` с методом `list(org_id)`.
Таблица: `organization_templates`. Поля: id, organization_id, name, template_type, subject, body, language, tone, metadata (JSON), created_at, updated_at.

#### Timeline (`timeline.rs`)

`OrgTimelineEvent`:

```rust
pub struct OrgTimelineEvent {
    pub id: String,
    pub organization_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
```

Хранилище `OrgTimelineStore`:
- `list(org_id, limit)` – применяет `TimelineEngine::bounded_entity_limit(limit)` для ограничения количества, сортировка `occurred_at DESC`.
- `add(org_id, event_type, title, occurred_at, source)` – валидирует событие через `TimelineEngine::validate_event`, затем вставляет запись.

Таблица: `organization_timeline_events`. Поля: id, organization_id, event_type, title, description, occurred_at, source, related_entity_id, related_entity_kind, confidence, metadata (JSON), created_at.

#### Ошибки

`OrgWorkflowError` – прозрачные варианты `Sqlx` и `Timeline` (из `crate::engines::timeline::TimelineEngineError`).

## Персоны (`persons`)

Домен `persons/api` предоставляет проекцию персон и сервис аналитики.

### API‑проекция

`PersonProjectionStore` (файл `api/store.rs`) объединяет несколько подмодулей с функциональностью:

#### Чтение (`persona_reads.rs`)

- `list_personas(limit)` – возвращает список персон, `limit` обрезается до диапазона 1–100, сортировка `updated_at DESC, created_at DESC, person_id`.
- `get_persona(persona_id)` – возвращает `Option<Person>`.

Таблица `persons`: поля person_id, display_name, email_address, person_type, is_self, created_at, updated_at.

#### Запись (`persona_writes.rs`)

- `update_persona(persona_id, display_name, set_self)` – в одной транзакции:
  - если передан `display_name`, обновляет его в `persons` и метку в `graph_nodes`.
  - если `set_self`, вызывает `assign_owner_persona_in_transaction`.
- `update_persona_with_observation(...)` – аналогично, но дополнительно связывает персону с наблюдением через `link_persons_entity_in_transaction` (отношение `"persona_update"`).

#### Владелец (`owner.rs`)

- `owner_persona()` – возвращает персону с `is_self = true`.
- `set_owner_persona(person_id)` – назначает указанную персону владельцем; предыдущий владелец теряет флаг `is_self`.
- `set_owner_persona_with_observation(person_id, observation_id)` – то же с привязкой к наблюдению.
- `assign_owner_persona_in_transaction(transaction, person_id)` – вспомогательная функция, сбрасывает `is_self` у всех остальных и устанавливает у целевой.

#### Email‑проекция (`email_projection.rs`)

- `upsert_email_person(email_address)` – создаёт/обновляет запись в `persons` и `person_identities`.
- `upsert_email_person_with_observation(email_address, observation_id)` – дополнительно связывает с наблюдением.
- `upsert_email_person_in_transaction(transaction, email_address)`:
  - нормализует email, определяет `person_id` как `person_id_for_email`.
  - `INSERT ... ON CONFLICT (email_address) DO UPDATE SET display_name`.
  - вставляет идентификатор в `person_identities` (`ON CONFLICT (identity_type, identity_value) WHERE status = 'active' DO UPDATE ...`).
- `link_email_person_projection_in_transaction` – связывает персону и идентификатор с наблюдением через вызовы `link_persons_entity_in_transaction`.

Таблица `person_identities`: поля id, person_id, identity_type, identity_value, source, confidence, status, metadata, last_verified_at, updated_at.

#### AI‑агенты (`ai_agents.rs`)

`upsert_ai_agent_persona(agent_id, display_name)` – в одной транзакции:
- нормализует agent_id, формирует `person_id` и `email_address` на основе agent_id.
- вставляет/обновляет запись в `persons` с `person_type = 'ai_agent'` и `is_self = false`.
- обновляет узел в `graph_nodes` (`node_kind = 'person'`) с метаданными `email_address`, `persona_type=ai_agent`, `agent_id`.
- вставляет/обновляет идентификатор в `person_identities` со статусом `active`, источником `ai_agent_registry` и метаданными.

#### Тип персоны (`persona_type.rs`)

`set_persona_type(person_id, persona_type)` – обновляет `person_type` у персоны.

#### Review‑проекция (`review_projection.rs`)

`upsert_review_person(person_id, display_name)` – создаёт/обновляет запись в `persons` с синтетическим email (`{person_id}@makosh.invalid`) и типом `human`, а также запись в `person_personas`.

Таблица `person_personas`: поля persona_id, person_id, name.

#### Модели (`models.rs`)

```rust
pub enum PersonaType {
    Human,
    AiAgent,
    OrganizationProxy,
    System,
}
```

Метод `as_str()` возвращает строковое представление (`"human"`, `"ai_agent"`, `"organization_proxy"`, `"system"`). Реализован `TryFrom<&str>` с ошибкой `InvalidPersonaType`.

`Person` – основная структура (person_id, display_name, email_address, persona_type, is_self, created_at, updated_at). Тип‑алиас `Persona = Person` (согласно комментарию: ADR‑0084 определяет `Persona` как каноническое имя).

#### Ошибки (`errors.rs`)

`PersonProjectionError`:
- `Sqlx` (прозрачный)
- `Observation` (прозрачный)
- `EmptyEmailAddress`
- `InvalidEmailAddress(String)`
- `EmptyAiAgentId`
- `InvalidAiAgentId(String)`
- `EmptyDisplayName`
- `PersonNotFound(String)`
- `InvalidPersonaType(String)`

### Участники сообщений (`participants.rs`)

Функция `upsert_persons_from_message_participants(store, email_addresses)`:
- нормализует email‑адреса (`normalize_email_addresses`), дедуплицирует.
- для каждого адреса вызывает `store.upsert_email_person`.

### Аналитика (`analytics.rs`)

`PersonAnalyticsService` вычисляет агрегированные показатели для персоны.

Метод `compute(person_id)` возвращает `PersonAnalytics`:
- `relationship_score` – взвешенная сумма `interaction_count` (вес 0.5) и `trust_score` (вес 0.5), ограниченная 100.
- `intelligence_score` – сумма баллов за заполнение полей: language, tone, trust_score, preferred_channel, writing_style, timezone, person_type, primary_role, organization_reference, notes (каждое +10, макс. 100).
- `interaction_heatmap` – агрегация сообщений из `communication_messages` по дням недели и часам (sender/recipients содержат person_id).
- `communication_costs` – средняя длина цепочки (interaction_count/10, макс. 50), среднее время ответа (`avg_response_hours`), частота follow‑up (0.3 если interaction_count>0).
- `shared_context` – количество общих проектов через `graph_edges` с отношением `person_involved_in_project` (остальные поля – общие документы и задачи – всегда 0).

Ошибка: `AnalyticsError` – прозрачный `Sqlx`.
```

### Source coverage / Покрытие источников

1. `backend/src/domains/organizations/memory.rs` – структуры `OrgFact`, `OrgMemoryCard`, `OrgPreference`, `OrgRequiredDocument`; методы хранилищ `list`, `upsert`, `decay_unverified`; SQL‑запросы к таблицам `organization_facts`, `organization_memory_cards`, `organization_preferences`, `organization_required_documents`; `OrgMemoryError`.
2. `backend/src/domains/organizations/mod.rs` – перечень подмодулей `organizations`.
3. `backend/src/domains/organizations/ports.rs` – реэкспорт `OrganizationCommandPort` и `OrganizationContactLinkPort`.
4. `backend/src/domains/organizations/service.rs` – `OrganizationCommandService` и его методы, вспомогательный `capture_manual`, использование observation; `OrganizationCommandServiceError`.
5. `backend/src/domains/organizations/workflows.rs` – реэкспорт `OrgPlaybook`, `OrgPlaybookStore`, `OrgPortal`, `OrgPortalStore`, `OrgProcedure`, `OrgProcedureStore`, `OrgTemplate`, `OrgTemplateStore`, `OrgTimelineEvent`, `OrgTimelineStore`, `OrgWorkflowError`.
6. `backend/src/domains/organizations/workflows/errors.rs` – `OrgWorkflowError` (Sqlx, Timeline).
7. `backend/src/domains/organizations/workflows/playbooks.rs` – `OrgPlaybook`, `OrgPlaybookStore::list`, таблица `organization_playbooks`.
8. `backend/src/domains/organizations/workflows/portals.rs` – `OrgPortal`, `OrgPortalStore::list` и `add`, таблица `organization_portals`.
9. `backend/src/domains/organizations/workflows/procedures.rs` – `OrgProcedure`, `OrgProcedureStore::list`, таблица `organization_procedures`.
10. `backend/src/domains/organizations/workflows/templates.rs` – `OrgTemplate`, `OrgTemplateStore::list`, таблица `organization_templates`.
11. `backend/src/domains/organizations/workflows/timeline.rs` – `OrgTimelineEvent`, `OrgTimelineStore::list` и `add`, использование `TimelineEngine`, таблица `organization_timeline_events`.
12. `backend/src/domains/persons/analytics.rs` – `PersonAnalyticsService`, вычисление `PersonAnalytics`, отдельных метрик; `AnalyticsError`.
13. `backend/src/domains/persons/api.rs` – реэкспорт `Person`, `Persona`, `PersonaType`, `PersonProjectionStore`, `PersonProjectionPort`, `upsert_persons_from_message_participants`.
14. `backend/src/domains/persons/api/errors.rs` – `PersonProjectionError` и его варианты.
15. `backend/src/domains/persons/api/models.rs` – `PersonaType`, `Person`, алиас `Persona = Person`.
16. `backend/src/domains/persons/api/participants.rs` – `upsert_persons_from_message_participants`, нормализация email.
17. `backend/src/domains/persons/api/rows.rs` – `row_to_person`.
18. `backend/src/domains/persons/api/store.rs` – структура `PersonProjectionStore`.
19. `backend/src/domains/persons/api/store/ai_agents.rs` – `upsert_ai_agent_persona`, операции с `persons`, `graph_nodes`, `person_identities`.
20. `backend/src/domains/persons/api/store/email_projection.rs` – `upsert_email_person`, `upsert_email_person_with_observation`, внутренние функции, связь с observation.
21. `backend/src/domains/persons/api/store/owner.rs` – `owner_persona`, `set_owner_persona`, `assign_owner_persona_in_transaction`.
22. `backend/src/domains/persons/api/store/persona_reads.rs` – `list_personas`, `get_persona`.
23. `backend/src/domains/persons/api/store/persona_type.rs` – `set_persona_type`.
24. `backend/src/domains/persons/api/store/persona_writes.rs` – `update_persona`, `update_persona_with_observation`, обновление `graph_nodes`.
25. `backend/src/domains/persons/api/store/review_projection.rs` – `upsert_review_person`, запись в `persons` и `person_personas`.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом, документацией и ADR не видны – отсутствует существующая wiki‑страница `components/backend.md` и тексты ADR для сравнения. Код содержит комментарий о том, что `Persona` определён как каноническое имя в ADR-0084; без содержания этого ADR подтвердить или опровергнуть drift невозможно.
