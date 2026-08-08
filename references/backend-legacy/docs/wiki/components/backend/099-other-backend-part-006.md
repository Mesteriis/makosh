---
chunk_id: 099-other-backend-part-006
batch_id: batch-20260628T214902
group: backend
role: other
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 099-other-backend-part-006 — backend/other

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Чанк содержит миграции 0126–0150, которые добавляют 34 новых канонических типа evidence (observation_kind_definitions) для Telegram, автоматизаций, ревью, identity, проектов, задач, персон, организаций, vault’а, календаря и производных evidence. Кроме того, миграции расширяют допустимые значения для ограничений `review_items.item_kind` (добавлен `knowledge_candidate`) и `tasks.source_kind` / `tasks.source_type` (добавлен `observation`), создают набор канонических таблиц коммуникаций (`communication_accounts`, `communication_channels`, …) и таблицу `communication_ai_states` с переносом данных из `mail_ai_states`. Страница `components/backend.md` должна быть обновлена, чтобы отразить все перечисленные изменения бэкенда.

## Предложенные страницы

### `components/backend.md`

```markdown
# Бэкенд

*Страница описывает компонент бэкенда Макошь. Обновлена с учётом миграций схемы 0126–0150.*

## Канонические типы evidence (`observation_kind_definitions`)

В таблице `observation_kind_definitions` зарегистрированы новые канонические evidence‑типы:

### Telegram

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `TELEGRAM_CHAT` | Telegram Chat | 1 | `telegram` | Canonical evidence for Telegram chat state materialization. |
| `TELEGRAM_CHAT_PARTICIPANT` | Telegram Chat Participant | 1 | `telegram` | Canonical evidence for Telegram chat participant roster materialization. |
| `TELEGRAM_TOPIC` | Telegram Topic | 1 | `telegram` | Canonical evidence for Telegram forum topic materialization. |
| `TELEGRAM_MESSAGE_REACTION` | Telegram Message Reaction | 1 | `telegram` | Canonical evidence for Telegram message reaction state materialization. |
| `TELEGRAM_MESSAGE_VERSION` | Telegram Message Version | 1 | `telegram` | Canonical evidence for append-only Telegram message edit versions. |
| `TELEGRAM_MESSAGE_TOMBSTONE` | Telegram Message Tombstone | 1 | `telegram` | Canonical evidence for append-only Telegram message tombstones and visibility deletions. |

Идентификаторы определений (kind_definition_id):
- `okd_telegram_chat_v1`
- `okd_telegram_chat_participant_v1`
- `okd_telegram_topic_v1`
- `okd_telegram_message_reaction_v1`
- `okd_telegram_message_version_v1`
- `okd_telegram_message_tombstone_v1`

### Автоматизации

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `AUTOMATION_TEMPLATE` | Automation template | 1 | `automation` | Automation template configuration captured as canonical evidence. |
| `AUTOMATION_POLICY` | Automation policy | 1 | `automation` | Automation policy configuration captured as canonical evidence. |
| `TELEGRAM_OUTBOUND_MESSAGE` | Telegram outbound message | 1 | `automation` | Automation dry-run or live outbound Telegram message materialization captured as canonical evidence. |

Идентификаторы определений: `observation_kind:v1:automation_template`, `observation_kind:v1:automation_policy`, `observation_kind:v1:telegram_outbound_message`.

### Ревью

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `REVIEW_TRANSITION` | Review transition | 1 | `review` | Manual review transition, approval, rejection, promotion, or similar user-driven review workflow change captured as canonical evidence. |
| `PROJECT_LINK_REVIEW` | Project link review | 1 | `review` | Canonical evidence describing a project link review event and its downstream review-state materialization. |

Идентификаторы: `observation_kind:v1:review_transition`, `observation_kind:v1:project_link_review`.

### Identity и персоны

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `PERSON_IDENTITY_CANDIDATE` | Person identity candidate | 1 | `identity` | Synthetic but canonical evidence describing a person identity candidate generated for review and promotion workflows. |
| `PERSON_MUTATION` | Person mutation | 1 | `persons` | Canonical evidence describing a manual mutation of a persona or person-centric profile state such as owner assignment, persona update, favorite toggle, or watchlist toggle. |
| `PERSON_RECORD_MUTATION` | Person record mutation | 1 | `persons` | Canonical evidence describing a manual mutation of subordinate person records such as identity traces, identities, compatibility roles, compatibility personas, facts, preferences, or relationship timeline events. |
| `PERSON_MEMORY_CARD` | Person memory card | 1 | `persons` | Canonical evidence describing a manual person memory note or memory card captured into persona memory. |
| `PERSON_ROLE` | Person role | 1 | `persons` | Canonical evidence describing a person role assignment or removal materialized as compatibility knowledge and relationship evidence. |
| `PERSON_TRUST_SIGNAL` | Person trust signal | 1 | `persons` | Canonical evidence describing a derived trust signal for a persona relationship materialized from person enrichment. |
| `PERSON_PROMISE` | Person promise | 1 | `persons` | Canonical evidence describing a persona promise that is projected into an obligation. |

Идентификаторы: `observation_kind:v1:person_identity_candidate`, `observation_kind:v1:person_mutation`, `observation_kind:v1:person_record_mutation`, `observation_kind:v1:person_memory_card`, `observation_kind:v1:person_role`, `observation_kind:v1:person_trust_signal`, `observation_kind:v1:person_promise`.

### Задачи

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `TASK_MUTATION` | Task mutation | 1 | `tasks` | Canonical evidence describing a manual or local-runtime task mutation, task-local record change, or compatibility task materialization. |

Идентификатор: `observation_kind:v1:task_mutation`.

### Организации

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `ORGANIZATION_MUTATION` | Organization mutation | 1 | `organizations` | Canonical evidence describing a manual mutation of an organization aggregate such as create, update, or archive. |
| `ORGANIZATION_RECORD_MUTATION` | Organization record mutation | 1 | `organizations` | Canonical evidence describing a manual mutation of subordinate organization records such as identities, aliases, departments, or organization contact links. |

Идентификаторы: `observation_kind:v1:organization_mutation`, `observation_kind:v1:organization_record_mutation`.

### Vault

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `CALENDAR_ACCOUNT_LINK` | Calendar account link | 1 | `vault` | Canonical evidence describing a linked provider calendar account materialized through the vault owner boundary. |
| `TASK_PROVIDER_ACCOUNT` | Task provider account | 1 | `vault` | Canonical evidence describing creation of a vault-owned task provider account. |
| `COMMUNICATION_PROVIDER_ACCOUNT` | Communication provider account | 1 | `vault` | Canonical evidence describing an upsert of a vault-owned communication provider account. |
| `COMMUNICATION_PROVIDER_SECRET_BINDING` | Communication provider secret binding | 1 | `vault` | Canonical evidence describing a vault-owned communication provider account secret binding mutation. |
| `COMMUNICATION_PROVIDER_ACCOUNT_DELETED` | Communication provider account deleted | 1 | `vault` | Canonical evidence describing deletion of vault-owned communication provider account metadata. |
| `COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED` | Communication provider secret binding removed | 1 | `vault` | Canonical evidence describing removal of a vault-owned communication provider secret binding during metadata cleanup. |
| `COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION` | Communication provider account config mutation | 1 | `vault` | Canonical evidence describing an update of vault-owned communication provider account config metadata. |

Идентификаторы: `observation_kind:v1:calendar_account_link`, `observation_kind:v1:task_provider_account`, `observation_kind:v1:communication_provider_account`, `observation_kind:v1:communication_provider_secret_binding`, `observation_kind:v1:communication_provider_account_deleted`, `observation_kind:v1:communication_provider_secret_binding_removed`, `observation_kind:v1:communication_provider_account_config_mutation`.

### Календарь

| Код | Название | Версия | Категория | Описание |
|-----|----------|--------|-----------|----------|
| `CALENDAR_ACCOUNT_MUTATION` | Calendar account mutation | 1 | `calendar` | Canonical evidence describing a manual mutation of a calendar account aggregate such as create, update, delete, or sync trigger. |
| `EVENT_AGENDA` | Event agenda | 1 | `calendar` | Canonical evidence describing a manual agenda captured for a calendar event. |
| `EVENT_CHECKLIST` | Event checklist | 1 | `calendar` | Canonical evidence describing a manual checklist captured for a calendar event. |
| `MEETING_NOTE` | Meeting note | 1 | `calendar` | Canonical evidence describing a manual meeting note captured for a calendar event. |
| `CALENDAR_RULE` | Calendar rule | 1 | `calendar` | Canonical evidence describing a manual create, update, or delete mutation of a calendar rule. |

Идентификаторы: `observation_kind:v1:calendar_account_mutation`, `observation_kind:v1:event_agenda`, `observation_kind:v1:event_checklist`, `observation_kind:v1:meeting_note`, `observation_kind:v1:calendar_rule`.

## Расширение ограничений схемы

### `review_items.item_kind`

Миграция 0146 заменяет CHECK‑ограничение поля `item_kind` таблицы `review_items`. Допустимые значения теперь включают `'knowledge_candidate'` дополнительно к ранее существовавшим:

```
'new_person', 'new_organization', 'identity_candidate',
'project_link_candidate', 'contradiction_candidate',
'potential_task', 'potential_obligation', 'potential_decision',
'potential_relationship', 'potential_project', 'knowledge_candidate'
```

### `tasks.source_kind`

Миграция 0147 обновляет CHECK‑ограничение `tasks.source_kind`. В перечень добавлено значение `'observation'`. Полный перечень:

```
'manual', 'observation', 'message', 'email', 'telegram', 'whatsapp',
'calendar', 'meeting', 'document', 'note', 'jira', 'youtrack', 'github',
'gitlab', 'linear', 'todoist', 'apple_reminders', 'ms_todo', 'ai_rule',
'workflow', 'import'
```

### `tasks.source_type`

Миграция 0148 обновляет CHECK‑ограничение `tasks.source_type`. Добавлено значение `'observation'`. Полный перечень:

```
'manual', 'observation', 'communication', 'email', 'telegram', 'whatsapp',
'calendar', 'meeting', 'document', 'note', 'jira', 'youtrack', 'github',
'gitlab', 'linear', 'todoist', 'apple_reminders', 'ms_todo', 'ai_rule',
'workflow', 'import'
```

## Канонические таблицы коммуникаций

Миграция 0149 создаёт провайдер‑нейтральные таблицы для дублирующегося состояния коммуникаций (ранее хранившегося в таблицах с префиксом провайдера). Из‑за ограничения размера контекста приведены только первые 12 000 символов файла миграции; перечисленные ниже таблицы гарантированно содержатся в ней:

- `communication_accounts` – учётные записи провайдеров.
- `communication_channels` – каналы (чаты, группы и т.п.), связанные с учётной записью.
- `communication_identities` – идентичности участников в рамках аккаунта/канала.
- `communication_conversations` – диалоги/беседы.
- `communication_conversation_participants` – участники диалогов.
- `communication_message_versions` – версии сообщений (для append‑only редактирования).
- `communication_message_tombstones` – записи об удалении/скрытии сообщений.
- `communication_message_reactions` – реакции на сообщения.
- `communication_message_refs` – связи между сообщениями (reply, forward).
- `communication_folders` – организационные папки.
- `communication_folder_messages` – связь папок и сообщений.
- `communication_saved_searches` – сохранённые поиски/умные папки (таблица начинается в обрезанной части, определение неполностью видно).

Каждая таблица содержит `created_at` / `updated_at`, JSONB‑поля `config`/`metadata`/`provenance` (где указано), внешние ключи и индексы для производительности.

**Примечание:** Миграция 0149 обрезана контекстом, поэтому описание может не охватывать все создаваемые в ней таблицы.

## Состояния AI‑обработки сообщений (`communication_ai_states`)

Миграция 0150 создаёт таблицу `communication_ai_states`, выносящую состояние AI‑обработки сообщений на уровень провайдер‑нейтрального слоя:

- `message_id` – первичный ключ, ссылается на `communication_messages(message_id) ON DELETE CASCADE`.
- `ai_state` – `TEXT NOT NULL DEFAULT 'NEW'`, допустимые значения: `'NEW'`, `'PROCESSING'`, `'PROCESSED'`, `'REVIEW_REQUIRED'`, `'FAILED'`, `'ARCHIVED'`.
- `review_reason` и `last_error` – опциональные текстовые поля с проверкой на непустую строку при наличии.
- Стандартные `created_at`, `updated_at`.

При создании таблицы выполняется перенос существующих записей из `mail_ai_states`:

```sql
INSERT INTO communication_ai_states (message_id, ai_state, review_reason, last_error, created_at, updated_at)
SELECT message_id, ai_state, review_reason, last_error, created_at, updated_at
FROM mail_ai_states
ON CONFLICT (message_id) DO NOTHING;
```

Создан индекс `communication_ai_states_state_updated_idx` по `(ai_state, updated_at DESC, message_id)`.
```

## Покрытие источников

| Исходный файл | Покрываемые факты |
|---|---|
| `backend/migrations/0126_add_telegram_chat_observation_kind.sql` | Добавление observation kind `TELEGRAM_CHAT` (`okd_telegram_chat_v1`). |
| `backend/migrations/0127_add_telegram_chat_participant_observation_kind.sql` | Добавление `TELEGRAM_CHAT_PARTICIPANT` (`okd_telegram_chat_participant_v1`). |
| `backend/migrations/0128_add_telegram_topic_observation_kind.sql` | Добавление `TELEGRAM_TOPIC` (`okd_telegram_topic_v1`). |
| `backend/migrations/0129_add_telegram_message_reaction_observation_kind.sql` | Добавление `TELEGRAM_MESSAGE_REACTION` (`okd_telegram_message_reaction_v1`). |
| `backend/migrations/0130_add_telegram_message_lifecycle_observation_kinds.sql` | Добавление `TELEGRAM_MESSAGE_VERSION` и `TELEGRAM_MESSAGE_TOMBSTONE` (`okd_telegram_message_version_v1`, `okd_telegram_message_tombstone_v1`). |
| `backend/migrations/0131_add_automation_observation_kinds.sql` | Добавление `AUTOMATION_TEMPLATE`, `AUTOMATION_POLICY`, `TELEGRAM_OUTBOUND_MESSAGE` (identifiers with `observation_kind:v1:` prefix, upsert on conflict). |
| `backend/migrations/0132_add_review_transition_observation_kind.sql` | Добавление `REVIEW_TRANSITION` (upsert). |
| `backend/migrations/0133_add_person_identity_candidate_observation_kind.sql` | Добавление `PERSON_IDENTITY_CANDIDATE` (upsert). |
| `backend/migrations/0134_add_project_link_review_observation_kind.sql` | Добавление `PROJECT_LINK_REVIEW` (upsert). |
| `backend/migrations/0135_add_task_mutation_observation_kind.sql` | Добавление `TASK_MUTATION` (upsert). |
| `backend/migrations/0136_add_person_mutation_observation_kind.sql` | Добавление `PERSON_MUTATION` (upsert). |
| `backend/migrations/0137_add_organization_mutation_observation_kind.sql` | Добавление `ORGANIZATION_MUTATION` (upsert). |
| `backend/migrations/0138_add_organization_record_mutation_observation_kind.sql` | Добавление `ORGANIZATION_RECORD_MUTATION` (upsert). |
| `backend/migrations/0139_add_person_record_mutation_observation_kind.sql` | Добавление `PERSON_RECORD_MUTATION` (upsert). |
| `backend/migrations/0140_add_vault_owner_observation_kinds.sql` | Добавление `CALENDAR_ACCOUNT_LINK`, `TASK_PROVIDER_ACCOUNT`, `COMMUNICATION_PROVIDER_ACCOUNT`, `COMMUNICATION_PROVIDER_SECRET_BINDING` (upsert). |
| `backend/migrations/0141_add_calendar_account_mutation_observation_kind.sql` | Добавление `CALENDAR_ACCOUNT_MUTATION` (upsert). |
| `backend/migrations/0142_add_vault_removal_observation_kinds.sql` | Добавление `COMMUNICATION_PROVIDER_ACCOUNT_DELETED`, `COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED` (upsert). |
| `backend/migrations/0143_add_communication_provider_account_config_mutation_kind.sql` | Добавление `COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION` (upsert). |
| `backend/migrations/0144_add_person_calendar_document_replacement_kinds.sql` | Добавление `PERSON_MEMORY_CARD`, `EVENT_AGENDA`, `EVENT_CHECKLIST`, `MEETING_NOTE`, `CALENDAR_RULE` (upsert). |
| `backend/migrations/0145_add_person_derived_evidence_observation_kinds.sql` | Добавление `PERSON_ROLE`, `PERSON_TRUST_SIGNAL`, `PERSON_PROMISE` (upsert). |
| `backend/migrations/0146_expand_review_item_kind_constraint.sql` | Замена ограничения `review_items.item_kind`: новый список включает `knowledge_candidate`. |
| `backend/migrations/0147_allow_observation_task_sources.sql` | Замена ограничения `tasks.source_kind`: добавлен `observation`. |
| `backend/migrations/0148_allow_observation_task_source_type.sql` | Замена ограничения `tasks.source_type`: добавлен `observation`. |
| `backend/migrations/0149_create_canonical_communication_tables.sql` (truncated) | Создание таблиц `communication_accounts`, `communication_channels`, `communication_identities`, `communication_conversations`, `communication_conversation_participants`, `communication_message_versions`, `communication_message_tombstones`, `communication_message_reactions`, `communication_message_refs`, `communication_folders`, `communication_folder_messages`, начало `communication_saved_searches`. |
| `backend/migrations/0150_create_communication_ai_states.sql` | Создание `communication_ai_states`, допустимые значения `ai_state`, вставка из `mail_ai_states`. |

## Исходные файлы

- [`backend/migrations/0126_add_telegram_chat_observation_kind.sql`](../../../../backend/migrations/0126_add_telegram_chat_observation_kind.sql)
- [`backend/migrations/0127_add_telegram_chat_participant_observation_kind.sql`](../../../../backend/migrations/0127_add_telegram_chat_participant_observation_kind.sql)
- [`backend/migrations/0128_add_telegram_topic_observation_kind.sql`](../../../../backend/migrations/0128_add_telegram_topic_observation_kind.sql)
- [`backend/migrations/0129_add_telegram_message_reaction_observation_kind.sql`](../../../../backend/migrations/0129_add_telegram_message_reaction_observation_kind.sql)
- [`backend/migrations/0130_add_telegram_message_lifecycle_observation_kinds.sql`](../../../../backend/migrations/0130_add_telegram_message_lifecycle_observation_kinds.sql)
- [`backend/migrations/0131_add_automation_observation_kinds.sql`](../../../../backend/migrations/0131_add_automation_observation_kinds.sql)
- [`backend/migrations/0132_add_review_transition_observation_kind.sql`](../../../../backend/migrations/0132_add_review_transition_observation_kind.sql)
- [`backend/migrations/0133_add_person_identity_candidate_observation_kind.sql`](../../../../backend/migrations/0133_add_person_identity_candidate_observation_kind.sql)
- [`backend/migrations/0134_add_project_link_review_observation_kind.sql`](../../../../backend/migrations/0134_add_project_link_review_observation_kind.sql)
- [`backend/migrations/0135_add_task_mutation_observation_kind.sql`](../../../../backend/migrations/0135_add_task_mutation_observation_kind.sql)
- [`backend/migrations/0136_add_person_mutation_observation_kind.sql`](../../../../backend/migrations/0136_add_person_mutation_observation_kind.sql)
- [`backend/migrations/0137_add_organization_mutation_observation_kind.sql`](../../../../backend/migrations/0137_add_organization_mutation_observation_kind.sql)
- [`backend/migrations/0138_add_organization_record_mutation_observation_kind.sql`](../../../../backend/migrations/0138_add_organization_record_mutation_observation_kind.sql)
- [`backend/migrations/0139_add_person_record_mutation_observation_kind.sql`](../../../../backend/migrations/0139_add_person_record_mutation_observation_kind.sql)
- [`backend/migrations/0140_add_vault_owner_observation_kinds.sql`](../../../../backend/migrations/0140_add_vault_owner_observation_kinds.sql)
- [`backend/migrations/0141_add_calendar_account_mutation_observation_kind.sql`](../../../../backend/migrations/0141_add_calendar_account_mutation_observation_kind.sql)
- [`backend/migrations/0142_add_vault_removal_observation_kinds.sql`](../../../../backend/migrations/0142_add_vault_removal_observation_kinds.sql)
- [`backend/migrations/0143_add_communication_provider_account_config_mutation_kind.sql`](../../../../backend/migrations/0143_add_communication_provider_account_config_mutation_kind.sql)
- [`backend/migrations/0144_add_person_calendar_document_replacement_kinds.sql`](../../../../backend/migrations/0144_add_person_calendar_document_replacement_kinds.sql)
- [`backend/migrations/0145_add_person_derived_evidence_observation_kinds.sql`](../../../../backend/migrations/0145_add_person_derived_evidence_observation_kinds.sql)
- [`backend/migrations/0146_expand_review_item_kind_constraint.sql`](../../../../backend/migrations/0146_expand_review_item_kind_constraint.sql)
- [`backend/migrations/0147_allow_observation_task_sources.sql`](../../../../backend/migrations/0147_allow_observation_task_sources.sql)
- [`backend/migrations/0148_allow_observation_task_source_type.sql`](../../../../backend/migrations/0148_allow_observation_task_source_type.sql)
- [`backend/migrations/0149_create_canonical_communication_tables.sql`](../../../../backend/migrations/0149_create_canonical_communication_tables.sql)
- [`backend/migrations/0150_create_communication_ai_states.sql`](../../../../backend/migrations/0150_create_communication_ai_states.sql)

## Кандидаты на drift

1. **Неоднородный формат `kind_definition_id`.** Миграции 0126–0130 используют компактные идентификаторы вида `okd_telegram_chat_v1`, тогда как начиная с 0131 применяется префикс `observation_kind:v1:…`. Это может быть осознанным переходом на новый стиль именования, но в рамках данного чанка выглядит как расхождение (drift) в соглашениях об идентификаторах.

2. **Миграция 0149 обрезана.** Полный объём файла составляет 33 331 символ, в контекст попало только 12 000. Часть создаваемых таблиц могла остаться недокументированной; это потенциальный drift между документацией (ограниченной контекстом) и фактическим содержимым миграции.
