---
chunk_id: 046-source-backend-part-026
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 046-source-backend-part-026 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Необходимо обновить страницу `components/backend.md`, добавив описание ключевых компонентов бэкенда, относящихся к доменам **обязательств (obligations)** и **организаций (organizations)**. Предоставленные исходные файлы раскрывают модели данных, перечисления состояний, хранилища (stores), сервис команд, порты, вспомогательные функции валидации и маппинга, а также механизмы связывания сущностей с observation-записями для provenance. Страница должна отражать текущее состояние кода без добавления внешних или предположительных сведений.

---

## Предложенные страницы

## `components/backend.md`

```markdown
# Backend-компоненты

В этом документе описаны серверные компоненты системы makosh, охватывающие домены **обязательств (obligations)** и **организаций (organizations)**, согласно текущему исходному коду.

## Обязательства (obligations)

### Модели

- **`ObligationEntityKind`** — перечисление типов сущностей-участников обязательства:
  `Persona`, `Organization`, `Project`, `Communication`, `Document`, `Task`, `Event`, `Decision`, `Obligation`, `Knowledge`.
  Методы: `as_str()` (возвращает snake_case), `parse(value)` (парсит строку, иначе `UnknownEntityKind`).

- **`ObligationEvidenceSourceKind`** — источники доказательств:
  `Observation`, `Communication`, `Document`, `Event`, `Memory`, `Knowledge`, `Decision`, `Obligation`, `Task`, `Project`, `Organization`, `Persona`.
  Метод `as_str()` возвращает snake_case.

- **`NewObligation`** — структура для создания/обновления обязательства (builder pattern).
  Поля: `obligated_entity_kind`, `obligated_entity_id`, `beneficiary_entity_kind`, `beneficiary_entity_id`, `statement`, `status`, `review_state`, `due_at`, `condition`, `risk_state`, `confidence`, `metadata`.
  Конструкторы: `new()`, `beneficiary()`, `status()`, `due_at()`, `condition()`, `risk_state()`, `metadata()`.
  Метод `validate()` проверяет обязательные поля, score в диапазоне [0.0, 1.0], JSON-объект `metadata`, согласованность beneficiary (если задан kind, то id обязателен).

- **`NewObligationEvidence`** — доказательство для привязки к обязательству.
  Поля: `source_kind`, `source_id`, `observation_id`, `quote`, `confidence`, `metadata`.
  Конструкторы: `new()`, `observation()`, `quote()`, `confidence()`, `metadata()`, `with_observation_id()`.
  Валидация (`validate()`): проверяет `source_id`, `observation_id` (если есть), соответствие `Observation` источника своему observation_id, score, JSON-объект `metadata`.

- **`Obligation`** (read-модель) — структура, получаемая из БД. Включает все поля `NewObligation` плюс `obligation_id`, `created_at`, `updated_at`.

- **Состояния**:
  - `ObligationStatus`: `Open`, `Fulfilled`, `Waived`, `Disputed`, `Canceled`.
  - `ObligationReviewState`: `Suggested`, `UserConfirmed`, `UserRejected`.
  - `ObligationRiskState`: `None`, `Watch`, `AtRisk`, `Breached`.
  Каждое перечисление имеет метод `as_str()`. `ObligationReviewState` также поддерживает `parse()`.

### Хранилище (`ObligationStore`)

Использует `PgPool`. Основные методы:

- **`upsert_with_evidence`** — вставляет или обновляет обязательство и связанные доказательства в одной транзакции.
  - Генерирует `obligation_id` на основе полей обязательства (функция `obligation_id()`).
  - Выполняет `INSERT ... ON CONFLICT (obligation_id) DO UPDATE` в таблицу `obligations`, обновляя `status`, `review_state`, `due_at`, `condition`, `risk_state`, `confidence`, `metadata` и `updated_at`.
  - Для каждого элемента `evidence` вставляет/обновляет запись в `obligation_evidence` (конфликт по `(obligation_id, source_kind, source_id)`).
  - Если evidence содержит `observation_id`, вызывает `link_obligation_support_in_transaction` для связывания обязательства с наблюдением.
  - Предварительно проверяет существование всех observation_id в таблице `observations`.

- **`list_for_entity`** — возвращает обязательства, где сущность выступает как обязанная или выгодоприобретатель. Фильтр `WHERE (obligated_entity_kind = $1 AND obligated_entity_id = $2) OR (beneficiary_entity_kind = $1 AND beneficiary_entity_id = $2)`. Лимит ограничен диапазоном [1, 100].

- **`list_by_review_state`** — обязательства по состоянию проверки, лимит [1, 100].

- **`set_review_state` / `set_review_state_with_observation`** — обновляет `review_state` и вызывает `link_obligation_review_transition_in_transaction` для записи перехода в evidence. Принимает опциональные `observation_id` и `metadata`.

Внутренний хелпер `validate_evidence_observations_exist` проверяет наличие всех переданных observation_id в таблице `observations`, иначе возвращает `ObservationNotFound`.

### Сервис команд (`ObligationCommandService`)

- Создаётся с `PgPool`.
- **`review_manual`**: принимает `obligation_id` и `review_state`, создаёт observation с origin `Manual`, типом события `REVIEW_TRANSITION`, и вызывает `set_review_state_with_observation`.
- Ошибки: `ObligationCommandServiceError` объединяет `ObservationStoreError` и `ObligationStoreError`.

### Порты

- `ObligationReviewPort` — реэкспорт `ObligationStore` как порта для использования в других слоях.

### Маппинг строк

- `row_to_obligation` преобразует `PgRow` из PostgreSQL в структуру `Obligation`, вызывая вспомогательные функции парсинга `parse_entity_kind`, `parse_status`, `parse_review_state`, `parse_risk_state`.

### Валидация

- `validate_obligation_with_evidence` — объединяет валидацию обязательства и всех доказательств, требует хотя бы одно доказательство.
- `validate_non_empty` — проверяет, что строковое значение не пусто после `trim()`.
- `validate_score` — значение должно быть в диапазоне [0.0, 1.0].
- `validate_json_object` — значение должно быть JSON-объектом.

---

## Организации (organizations)

### Основное API (`api`)

#### Структура `Organization`

Содержит множество полей (наблюдаемое по предоставленному фрагменту, файл усечён): `organization_id`, `display_name`, `legal_name`, `org_type`, `status`, `country`, `city`, `address`, `website`, `industry`, `description`, `primary_language`, `timezone`, `trust_score`, `health_status`, `priority`, `notes`, `tags`, `org_metadata`, `last_interaction_at`, `interaction_count`, `registration_number`, `country_of_registration`, `vat`, `cif`, `nif`, `tax_id`, `legal_address`, `registry_source`, `registry_last_verified`, `communication_style`, `verbosity`, `formality`, `secondary_languages`, `preferred_tone`, `official_style_required`, `last_health_check`, `watchlist`, `created_at`, `updated_at`.

#### `OrganizationStore`

- `create` / `create_with_observation` — генерация id `org:v1:{nanos}` и вставка. При наличии observation_id связывает через `link_organization_in_transaction`.
- `upsert_review_organization` — вставка или обновление с `org_type='derived'`. Принимает `organization_id`, `display_name`, опциональное `description`.
- `upsert_email_domain_organization` / `upsert_email_domain_organization_with_observation` — создаёт/обновляет организацию по домену почты. id = `org:v1:email-domain:{len}:{domain}`, `org_type='company'`, `website='https://{domain}'`. В случае конфликта обновляет `updated_at`, `last_interaction_at`, инкрементирует `interaction_count`. Возвращает `(Organization, inserted)`.
- `get` — получение по `organization_id`.
- `list` — список с фильтром по `org_type`, лимит [1,100], сортировка по `interaction_count DESC`. Без фильтра возвращает все.

*(Файл `api.rs` обрезан; другие методы не включены в данный чанк.)*

### Ядро (core)

Модуль `core` реэкспортирует следующие подмодули:

#### Aliases (псевдонимы)
- **`OrganizationAlias`**: `id`, `organization_id`, `name`, `alias_type`, `source`, `confidence`, `valid_from`, `valid_to`, `created_at`.
- **`OrgAliasStore`**: методы `list`, `add`, `add_with_observation`, `add_in_transaction`. Нормализация `alias_type`: `former_name` → `former`.

#### Contact links (связи с лицами)
- **`OrgContactLink`**: `id`, `organization_id`, `person_id`, `role`, `department`, `source`, `confidence`, `valid_from`, `valid_to`, `is_primary`, `created_at`, `updated_at`.
- **`OrgContactLinkStore`**:
  - `list_by_org` — сортировка по `is_primary DESC, role`.
  - `link` / `link_with_observation` — вставка/обновление (конфликт `ON CONFLICT (organization_id, person_id, role) DO UPDATE`).
  - `link_email_participant_with_observation` — вставляет/обновляет запись с ролью `email_participant`, источником `email_sync`, `confidence=1.0`. Возвращает `(OrgContactLink, inserted)`.
  - `set_primary` — сбрасывает все `is_primary` и устанавливает флаг для указанного `person_id`.
- Реэкспортируется как порт `OrganizationContactLinkPort`.

#### Departments (отделы)
- **`OrgDepartment`**: `id`, `organization_id`, `name`, `description`, `parent_department_id`, `created_at`.
- **`OrgDepartmentStore`**: `list`, `add`, `add_with_observation`, `add_in_transaction`. При добавлении `parent_department_id` передаётся как `uuid`.

#### Domains (домены)
- **`OrganizationDomain`**: `id`, `organization_id`, `domain`, `domain_type`, `source`, `confidence`, `last_verified_at`, `created_at`.
- **`OrgDomainStore`**:
  - `list`, `add`.
  - `upsert_email_domain` — вставляет домен с типом `email`, если не существует ни одной записи с `domain_type != 'former'`; возвращает `bool` — был ли вставлен.
  - `upsert_email_domain_in_transaction` — транзакционная версия.

#### Errors (ошибки ядра)
- **`OrgCoreError`**: варианты `Sqlx`, `Observation`, `NotFound`. Использует `thiserror`.

#### Evidence (связывание с наблюдениями)
- `link_organization_in_transaction` — связывает организацию с observation через `link_domain_entity_in_transaction` (action: "create" или другое, плюс метаданные).
- `link_entity_in_transaction` — обобщённая привязка произвольной сущности (например, alias, identity) к observation.
- `link_review_transition_in_transaction` — запись перехода статуса (review_transition).
- `link_email_domain_projection_in_transaction` — при получении email-домена создаёт три связи доменной сущности: organization, organization_domain, organization_identity.
- `merge_metadata` — вспомогательная функция для слияния JSON-объектов метаданных.

#### Identity (идентификаторы)
- **`OrganizationIdentity`**: `id`, `organization_id`, `identity_type`, `identity_value`, `source`, `confidence`, `last_verified_at`, `status`, `metadata`, `created_at`, `updated_at`.
- **`OrgIdentityStore`**: `list`, `upsert`, `upsert_with_observation`, `upsert_in_transaction`. Вставка с `ON CONFLICT (identity_type, identity_value) WHERE status='active' DO UPDATE`.

#### Related (связанные организации)
- **`RelatedOrganization`**: `id`, `organization_id`, `related_organization_id`, `relation_type`, `source`, `confidence`, `created_at`.
- **`RelatedOrgStore`**: `list`, `relate`. `relate` использует `ON CONFLICT DO NOTHING`.

### Обогащение (enrichment)

- **`OrgEnrichmentResult`**: `id`, `organization_id`, `source`, `url`, `data` (Value), `confidence`, `status`, `last_checked_at`, `applied_at`, `created_at`.
- **`OrgEnrichmentStore`**:
  - `list` — все результаты для организации.
  - `upsert` — вставка с указанными `data` и `confidence`.
  - `apply` / `apply_with_observation` — устанавливает `status='applied'`, `applied_at=now()`. Версия с observation связывает через `link_review_transition_in_transaction`.
  - `reject` — устанавливает `status='rejected'`.
- Ошибка `OrgEnrichmentError` включает прозрачные варианты `Sqlx`, `Core`, `Observation` и `NotFound`.

### Финансы (finance)

Набор структур и хранилищ для финансовых данных:

- **`OrgFinancialInfo`** (поля: `id`, `organization_id`, `bank_name`, `iban_masked`, `bic`, `payment_terms`, `currency`, `billing_email`, `billing_address`, `created_at`, `updated_at`). Хранилище: `get`, `upsert` (конфликт по `organization_id`).
- **`OrgContract`** (поля: `id`, `organization_id`, `contract_type`, `title`, `signed_at`, `expires_at`, `status`, `document_reference`, `notes`, `created_at`, `updated_at`). Хранилище: `list`, `add`.
- **`OrgCompliance`** (поля: `id`, `organization_id`, `compliance_type`, `status`, `document_reference`, `expires_at`, `notes`, `created_at`, `updated_at`). Хранилище: `list`.
- **`OrgService`** (поля: `id`, `organization_id`, `service_name`, `description`, `status`, `started_at`, `created_at`, `updated_at`). Хранилище: `list`.
- **`OrgProduct`** (поля: `id`, `organization_id`, `product_name`, `description`, `status`, `created_at`, `updated_at`). Хранилище: `list`.
- Ошибка `OrgFinanceError` включает только `Sqlx`.

### Здоровье (health)

- **`OrgHealth`** — сводка: `organization_id`, `display_name`, `health_status`, `last_health_check`, `watchlist`, `interaction_count`, `trust_score`, `open_risks`, `overdue_contracts`.
- **`OrgHealthStore`**:
  - `get` — запрос с агрегацией: подсчёт открытых рисков (`resolved_at IS NULL`) и просроченных контрактов (`expires_at < now()`).
  - `list_unhealthy` — организации с `health_status != 'healthy'`, лимит 50.
  - `toggle_watchlist` / `toggle_watchlist_with_source` / `toggle_watchlist_with_observation` — переключает флаг `watchlist` и записывает предпочтение `ui:watchlist` в таблицу `organization_preferences`.
- **`OrgRisk`** (`id`, `organization_id`, `risk_type`, `description`, `severity`, `source`, `confidence`, `created_at`, `resolved_at`, `resolution`).
  - **`OrgRiskStore`**: `list`, `add`.
- Ошибка `OrgHealthError`: `Sqlx`, `Core`, `Observation`.

### Следователь (investigator)

- **`OrganizationInvestigator`** (использует `PgPool` через `OrganizationStore`):
  - `dossier` — возвращает `OrgDossier` (пока с ограниченной реализацией: заполняет только `organization`, `org_type`, `interaction_count`). Остальные поля — пустые векторы.
  - `brief` — возвращает `OrgBrief` (основные поля: `organization_id`, `display_name`, `org_type`, `last_interaction_days`, `primary_language`).
  - `context_pack` — возвращает `OrgContextPack` (включает `brief`, остальные поля — пустые векторы).
- Ошибка `InvestigatorError` объединяет `Sqlx`, `Core`, `Observation`, `Validation`, `NotFound` и преобразуется из `OrganizationError`.

### Ошибки в целом

Домен организаций использует несколько типов ошибок, каждый из которых оборачивает нижележащие ошибки через `#[from]`:

- `OrganizationError` (в `api`) — `NotFound`, `Validation`, `Sqlx`, `Core`, `Observation`.
- `OrgCoreError` — `Sqlx`, `Observation`, `NotFound`.
- `OrgEnrichmentError` — `Sqlx`, `Core`, `Observation`, `NotFound`.
- `OrgFinanceError` — `Sqlx`.
- `OrgHealthError` — `Sqlx`, `Core`, `Observation`.
- `InvestigatorError` — `Sqlx`, `Core`, `Observation`, `Validation`, `NotFound`.

Все ошибки реализуют `std::error::Error` через `thiserror`.

### Взаимодействие с платформенным слоем

Хранилища доменов `obligations` и `organizations` активно используют:

- `ObservationStore` из `crate::platform::observations` для записи provenance-событий (создание, изменение, связывание).
- Функции `link_domain_entity_in_transaction`, `link_obligation_support_in_transaction`, `link_obligation_review_transition_in_transaction` для фиксации связей сущностей с observation-записями в пределах транзакций.

Многие методы имеют варианты `_with_observation`, позволяющие прикрепить создаваемую/изменяемую сущность к конкретному observation, обеспечивая audit trail.
```

---

## Покрытие источников

| Исходный файл | Факты, отражённые на странице |
|---|---|
| `backend/src/domains/obligations/models/entity_kind.rs` | Перечисление `ObligationEntityKind`, варианты, методы `as_str` и `parse`. |
| `backend/src/domains/obligations/models/evidence.rs` | Структура `NewObligationEvidence`, её поля, конструкторы, метод `validate` с проверками. |
| `backend/src/domains/obligations/models/obligation.rs` | Структура `NewObligation`, её поля, конструкторы, метод `validate`. |
| `backend/src/domains/obligations/models/read_model.rs` | Структура `Obligation` (read-модель) с полями `obligation_id`, `created_at`, `updated_at`. |
| `backend/src/domains/obligations/models/source_kind.rs` | Перечисление `ObligationEvidenceSourceKind`, варианты, метод `as_str`. |
| `backend/src/domains/obligations/models/states.rs` | Перечисления `ObligationStatus`, `ObligationReviewState`, `ObligationRiskState`, их варианты, методы `as_str` и `parse` (для `ReviewState`). |
| `backend/src/domains/obligations/ports.rs` | Реэкспорт `ObligationStore` как `ObligationReviewPort`. |
| `backend/src/domains/obligations/row_mapping.rs` | Функция `row_to_obligation`, вспомогательные парсеры для полей из БД. |
| `backend/src/domains/obligations/service.rs` | Сервис `ObligationCommandService`, метод `review_manual`, создание observation, вызов `set_review_state_with_observation`. |
| `backend/src/domains/obligations/store.rs` | Хранилище `ObligationStore`, методы `upsert_with_evidence`, `list_for_entity`, `list_by_review_state`, `set_review_state`, валидация evidence-observations. |
| `backend/src/domains/obligations/validation.rs` | Функции `validate_obligation_with_evidence`, `validate_non_empty`, `validate_score`, `validate_json_object`. |
| `backend/src/domains/organizations/api.rs` (усечён) | Структура `Organization` (видимые поля), методы `OrganizationStore`: `create`, `upsert_review_organization`, `upsert_email_domain_organization`, `get`, `list`. |
| `backend/src/domains/organizations/core.rs` | Реэкспорт подмодулей ядра (псевдонимы, связи, отделы, домены, ошибки, evidence, идентификаторы, связанные организации). |
| `backend/src/domains/organizations/core/aliases.rs` | `OrganizationAlias`, `OrgAliasStore`, методы `list`, `add`, нормализация `alias_type`. |
| `backend/src/domains/organizations/core/contact_links.rs` | `OrgContactLink`, `OrgContactLinkStore`, методы `list_by_org`, `link`, `link_email_participant_with_observation`, `set_primary`, порт `OrganizationContactLinkPort`. |
| `backend/src/domains/organizations/core/departments.rs` | `OrgDepartment`, `OrgDepartmentStore`, методы `list`, `add`. |
| `backend/src/domains/organizations/core/domains.rs` | `OrganizationDomain`, `OrgDomainStore`, методы `list`, `add`, `upsert_email_domain`. |
| `backend/src/domains/organizations/core/errors.rs` | `OrgCoreError` с вариантами `Sqlx`, `Observation`, `NotFound`. |
| `backend/src/domains/organizations/core/evidence.rs` | Функции `link_organization_in_transaction`, `link_entity_in_transaction`, `link_review_transition_in_transaction`, `link_email_domain_projection_in_transaction`, `merge_metadata`. |
| `backend/src/domains/organizations/core/identity.rs` | `OrganizationIdentity`, `OrgIdentityStore`, методы `list`, `upsert`, `upsert_in_transaction`. |
| `backend/src/domains/organizations/core/related.rs` | `RelatedOrganization`, `RelatedOrgStore`, методы `list`, `relate`. |
| `backend/src/domains/organizations/enrichment.rs` | `OrgEnrichmentResult`, `OrgEnrichmentStore`, методы `list`, `upsert`, `apply`, `reject`, ошибки. |
| `backend/src/domains/organizations/finance.rs` | Структуры `OrgFinancialInfo`, `OrgContract`, `OrgCompliance`, `OrgService`, `OrgProduct` и соответствующие хранилища, ошибка `OrgFinanceError`. |
| `backend/src/domains/organizations/health.rs` | `OrgHealth`, `OrgHealthStore` (get, list_unhealthy, toggle_watchlist), `OrgRisk`, `OrgRiskStore`, ошибки. |
| `backend/src/domains/organizations/investigator.rs` | `OrganizationInvestigator`, структуры `OrgDossier`, `OrgBrief`, `OrgContextPack`, методы `dossier`, `brief`, `context_pack`, ошибки. |

---

## Исходные файлы

- [`backend/src/domains/obligations/models/entity_kind.rs`](../../../../backend/src/domains/obligations/models/entity_kind.rs)
- [`backend/src/domains/obligations/models/evidence.rs`](../../../../backend/src/domains/obligations/models/evidence.rs)
- [`backend/src/domains/obligations/models/obligation.rs`](../../../../backend/src/domains/obligations/models/obligation.rs)
- [`backend/src/domains/obligations/models/read_model.rs`](../../../../backend/src/domains/obligations/models/read_model.rs)
- [`backend/src/domains/obligations/models/source_kind.rs`](../../../../backend/src/domains/obligations/models/source_kind.rs)
- [`backend/src/domains/obligations/models/states.rs`](../../../../backend/src/domains/obligations/models/states.rs)
- [`backend/src/domains/obligations/ports.rs`](../../../../backend/src/domains/obligations/ports.rs)
- [`backend/src/domains/obligations/row_mapping.rs`](../../../../backend/src/domains/obligations/row_mapping.rs)
- [`backend/src/domains/obligations/service.rs`](../../../../backend/src/domains/obligations/service.rs)
- [`backend/src/domains/obligations/store.rs`](../../../../backend/src/domains/obligations/store.rs)
- [`backend/src/domains/obligations/validation.rs`](../../../../backend/src/domains/obligations/validation.rs)
- [`backend/src/domains/organizations/api.rs`](../../../../backend/src/domains/organizations/api.rs)
- [`backend/src/domains/organizations/core.rs`](../../../../backend/src/domains/organizations/core.rs)
- [`backend/src/domains/organizations/core/aliases.rs`](../../../../backend/src/domains/organizations/core/aliases.rs)
- [`backend/src/domains/organizations/core/contact_links.rs`](../../../../backend/src/domains/organizations/core/contact_links.rs)
- [`backend/src/domains/organizations/core/departments.rs`](../../../../backend/src/domains/organizations/core/departments.rs)
- [`backend/src/domains/organizations/core/domains.rs`](../../../../backend/src/domains/organizations/core/domains.rs)
- [`backend/src/domains/organizations/core/errors.rs`](../../../../backend/src/domains/organizations/core/errors.rs)
- [`backend/src/domains/organizations/core/evidence.rs`](../../../../backend/src/domains/organizations/core/evidence.rs)
- [`backend/src/domains/organizations/core/identity.rs`](../../../../backend/src/domains/organizations/core/identity.rs)
- [`backend/src/domains/organizations/core/related.rs`](../../../../backend/src/domains/organizations/core/related.rs)
- [`backend/src/domains/organizations/enrichment.rs`](../../../../backend/src/domains/organizations/enrichment.rs)
- [`backend/src/domains/organizations/finance.rs`](../../../../backend/src/domains/organizations/finance.rs)
- [`backend/src/domains/organizations/health.rs`](../../../../backend/src/domains/organizations/health.rs)
- [`backend/src/domains/organizations/investigator.rs`](../../../../backend/src/domains/organizations/investigator.rs)

## Кандидаты на drift

На основе предоставленного контекста расхождений между кодом и документацией не обнаружено. Однако:

- Файл `backend/src/domains/organizations/api.rs` был обрезан на 12000 символах, поэтому полный набор методов `OrganizationStore` и все поля структуры `Organization` не могут быть подтверждены данным чанком.
- Реализация `OrganizationInvestigator` возвращает структуры, в которых многие поля (например, `identities`, `domains`, `contacts` в `OrgDossier`) являются пустыми векторами — это может указывать на незавершённую функциональность, но не на drift между кодом и документацией.

За пределами этих наблюдений видимых расхождений не выявлено.
