### Summary / Резюме

Страница `components/backend.md` должна быть обновлена, чтобы описать компоненты схемы базы данных, вводимые или изменяемые миграциями 0051–0075. Поскольку в чанке присутствуют только SQL-файлы миграций, страница играет роль справочника по таблицам, ограничениям, индексам и начальным данным. Основные группы: правила и шаблоны задач, центр управления AI, синхронизация почты с локальной корзиной, отношения и доказательства, противоречия, обязательства, решения, а также многочисленные доработки существующих таблиц (идентичности персон, типы задач, проекции графа, ослабление обязательности полей и др.).

### Proposed pages / Предлагаемые страницы

- `components/backend.md`

```markdown
# Бэкенд

Ниже описаны компоненты схемы базы данных Макошь, добавленные или изменённые миграциями `0051`–`0075`. Для каждой миграции указаны создаваемые таблицы, ограничения, индексы и начальные данные (если есть). Разделы сгруппированы по функциональным областям.

---

## Правила и шаблоны задач (`0051`, `0067`, `0069`)

### `task_rules` (0051)

Хранит правила автоматической обработки задач.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `rule_id` | `TEXT` | `PRIMARY KEY` |
| `name` | `TEXT` | `NOT NULL` |
| `natural_language_description` | `TEXT` |  |
| `compiled_dsl` | `JSONB` | `NOT NULL DEFAULT '{}'` |
| `enabled` | `BOOLEAN` | `NOT NULL DEFAULT true` |
| `approval_mode` | `TEXT` | `NOT NULL DEFAULT 'suggest_only'`, `CHECK (approval_mode IN ('suggest_only','ask_before_execute','auto_execute','dry_run'))` |
| `last_run_at` | `TIMESTAMPTZ` |  |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

### `task_templates` (0051)

Шаблоны для создания задач с предзаполненными полями и чек-листами.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `template_id` | `TEXT` | `PRIMARY KEY` |
| `name` | `TEXT` | `NOT NULL` |
| `description` | `TEXT` |  |
| `default_fields` | `JSONB` | `NOT NULL DEFAULT '{}'` |
| `default_checklist` | `JSONB` | `NOT NULL DEFAULT '[]'` |
| `default_priority` | `TEXT` | `DEFAULT 'medium'` |
| `default_energy_type` | `TEXT` |  |
| `required_documents` | `JSONB` | `NOT NULL DEFAULT '[]'` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

#### Начальные шаблоны

Вставлены 8 шаблонов (`ON CONFLICT (template_id) DO NOTHING`):

- `bug` — Bug Report (priority `high`, checklist: Steps to reproduce, Expected result, Actual result, Environment details)
- `feature` — Feature Request (priority `medium`, checklist: Requirements, Design doc, Implementation plan, Tests)
- `research` — Research Task (priority `medium`, checklist: Define question, Gather sources, Document findings, Make decision)
- `contract_review` — Contract Review (priority `high`, checklist: Check parties, Check amounts, Check deadlines, Check signatures, Check terms, Create summary)
- `aeat_response` — AEAT Response (priority `critical`, checklist: Check documents, Check certificado digital, Download PDFs, Check deadline, Prepare response, Submit)
- `client_followup` — Client Follow-up (priority `medium`, checklist: Send follow-up email, Update project status, Create tasks from decisions, Schedule next check-in)
- `invoice_review` — Invoice Review (priority `high`, checklist: Check amount, Check VAT, Check dates, Check provider details, Approve or flag)
- `code_review` — Code Review (priority `medium`, checklist: Review diff, Check tests, Check docs, Add comments, Approve or request changes)

### `task_snapshots` (0051)

Снимки состояния задачи на определенный момент.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `id` | `UUID` | `PRIMARY KEY DEFAULT gen_random_uuid()` |
| `task_id` | `TEXT` | `NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE` |
| `snapshot_date` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `data` | `JSONB` | `NOT NULL` |
| `source` | `TEXT` | `DEFAULT 'system'` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индекс: `task_snapshots_task_idx` на `(task_id)`.

### `task_candidates` — расширения (0067)

Добавлены колонки `candidate_kind` (тип кандидата) и `candidate_metadata` (JSONB-метаданные).

- `candidate_kind` — `TEXT NOT NULL DEFAULT 'task'`, ограничение `CHECK (candidate_kind IN ('task', 'obligation_task'))`
- `candidate_metadata` — `JSONB NOT NULL DEFAULT '{}'::jsonb`, ограничение `CHECK (jsonb_typeof(candidate_metadata) = 'object')`
- Индекс: `task_candidates_candidate_kind_idx` на `(candidate_kind, review_state, updated_at DESC)`

### `tasks` — ослабление обязательности полей (0069)

- Поле `task_candidate_id` становится `DROP NOT NULL`
- Поле `created_from_event_id` становится `DROP NOT NULL`
- Поле `created_by_actor_id` становится `DROP NOT NULL`
- Обновлена проверка `source_kind`: разрешённые значения включают `'manual', 'message', 'email', 'telegram', 'whatsapp', 'calendar', 'meeting', 'document', 'note', 'jira', 'youtrack', 'github', 'gitlab', 'linear', 'todoist', 'apple_reminders', 'ms_todo', 'ai_rule', 'workflow', 'import'`

---

## Центр управления AI (`0057`, `0070`)

### `ai_provider_accounts` (0057)

Учётные записи AI-провайдеров.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `provider_id` | `TEXT` | `PRIMARY KEY` |
| `provider_kind` | `TEXT` | `NOT NULL`, `CHECK (provider_kind IN ('built_in', 'cli', 'api'))` |
| `provider_key` | `TEXT` | `NOT NULL`, `CHECK (length(trim(provider_key)) > 0)` |
| `display_name` | `TEXT` | `NOT NULL`, `CHECK (length(trim(display_name)) > 0)` |
| `status` | `TEXT` | `NOT NULL DEFAULT 'needs_setup'`, `CHECK (status IN ('ready','disabled','needs_setup','error'))` |
| `consent_state` | `TEXT` | `NOT NULL DEFAULT 'not_required'`, `CHECK (consent_state IN ('not_required','required','granted','revoked'))` |
| `consented_at` | `TIMESTAMPTZ` |  |
| `config` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(config) = 'object')` |
| `capabilities` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(capabilities) = 'array')` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальный индекс: `ai_provider_accounts_kind_key_idx` на `(provider_kind, provider_key)`.

Начальные данные: вставлен провайдер `provider:built_in:ollama` (`built_in`, ключ `ollama`, статус `ready`, consent `not_required`, config `{"base_url":"http://127.0.0.1:11434","manager":"ollama"}`).

### `ai_provider_secret_refs` (0057)

Связывает провайдера с секретом в `secret_references`.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `provider_id` | `TEXT` | `NOT NULL REFERENCES ai_provider_accounts(provider_id) ON DELETE CASCADE` |
| `secret_purpose` | `TEXT` | `NOT NULL`, `CHECK (secret_purpose IN ('api_key'))` |
| `secret_ref` | `TEXT` | `NOT NULL REFERENCES secret_references(secret_ref) ON DELETE RESTRICT` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

`PRIMARY KEY (provider_id, secret_purpose)`.

### `ai_model_catalog` (0057)

Каталог доступных AI-моделей.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `provider_id` | `TEXT` | `NOT NULL REFERENCES ai_provider_accounts(provider_id) ON DELETE CASCADE` |
| `model_key` | `TEXT` | `NOT NULL`, `CHECK (length(trim(model_key)) > 0)` |
| `display_name` | `TEXT` | `NOT NULL`, `CHECK (length(trim(display_name)) > 0)` |
| `category` | `TEXT` | `NOT NULL`, `CHECK (length(trim(category)) > 0)` |
| `privacy` | `TEXT` | `NOT NULL`, `CHECK (privacy IN ('local', 'remote', 'cli'))` |
| `capabilities` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(capabilities) = 'array')` |
| `context_window` | `INTEGER` | `CHECK (context_window IS NULL OR context_window > 0)` |
| `embedding_dimension` | `INTEGER` | `CHECK (embedding_dimension IS NULL OR embedding_dimension > 0)` |
| `is_available` | `BOOLEAN` | `NOT NULL DEFAULT true` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(metadata) = 'object')` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

`PRIMARY KEY (provider_id, model_key)`.

Начальные модели:

- `qwen3:4b` — категория `chat`, privacy `local`, context_window `32768`, capabilities `["chat","reasoning","summarization","extraction"]`, metadata `{"curated":true,"pull_required":true}`
- `qwen3-embedding:4b` — категория `embeddings`, privacy `local`, context_window `8192`, embedding_dimension `2560`, capabilities `["embeddings"]`, metadata `{"curated":true,"pull_required":true}`

### `ai_model_routes` (0057)

Роутинг capability‑слотов на конкретные модели.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `capability_slot` | `TEXT` | `PRIMARY KEY`, `CHECK (capability_slot IN ('default_chat','reasoning','summarization','mail_intelligence','reply_draft','extraction','embeddings','meeting_prep'))` |
| `provider_id` | `TEXT` | `NOT NULL` |
| `model_key` | `TEXT` | `NOT NULL` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

`FOREIGN KEY (provider_id, model_key) REFERENCES ai_model_catalog(provider_id, model_key) ON DELETE RESTRICT`.

Начальные маршруты: все 8 слотов назначены на `provider:built_in:ollama` с моделями `qwen3:4b` и `qwen3-embedding:4b` (для `embeddings`).

### `ai_prompt_templates` и `ai_prompt_template_versions` (0057)

Шаблоны промптов и их версии.

#### `ai_prompt_templates`

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `prompt_id` | `TEXT` | `PRIMARY KEY` |
| `name` | `TEXT` | `NOT NULL`, `CHECK (length(trim(name)) > 0)` |
| `entity_scope` | `TEXT` | `NOT NULL`, `CHECK (entity_scope IN ('global','person','organization','project','document','task','meeting','communication','conversation'))` |
| `capability_slot` | `TEXT` | `NOT NULL`, `CHECK` (те же 8 значений) |
| `description` | `TEXT` |  |
| `is_system` | `BOOLEAN` | `NOT NULL DEFAULT false` |
| `active_version_id` | `TEXT` |  |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(metadata) = 'object')` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

#### `ai_prompt_template_versions`

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `prompt_version_id` | `TEXT` | `PRIMARY KEY` |
| `prompt_id` | `TEXT` | `NOT NULL REFERENCES ai_prompt_templates(prompt_id) ON DELETE CASCADE` |
| `version_label` | `TEXT` | `NOT NULL`, `CHECK (length(trim(version_label)) > 0)` |
| `body_template` | `TEXT` | `NOT NULL`, `CHECK (length(trim(body_template)) > 0)` |
| `variables` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(variables) = 'array')` |
| `status` | `TEXT` | `NOT NULL DEFAULT 'draft'`, `CHECK (status IN ('draft','active','archived'))` |
| `created_by_actor_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(created_by_actor_id)) > 0)` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индекс: `ai_prompt_template_versions_prompt_idx` на `(prompt_id, created_at DESC)`.

Начальные промпты (оба системные, `is_system = true`, `seeded = true`):

- `prompt:system:global:default_chat` — scope `global`, slot `default_chat`, версия `v1`, body: `Answer using only cited Макошь context. Query: {{query}}`, переменная `query`
- `prompt:system:communication:mail_intelligence` — scope `communication`, slot `mail_intelligence`, версия `v1`, body: `Analyze this communication and return concise operational context. Subject: {{subject}}`, переменная `subject`

### `ai_prompt_eval_runs` (0057)

Запуски оценки промптов.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `eval_run_id` | `TEXT` | `PRIMARY KEY` |
| `prompt_id` | `TEXT` | `NOT NULL REFERENCES ai_prompt_templates(prompt_id) ON DELETE CASCADE` |
| `prompt_version_id` | `TEXT` | `NOT NULL REFERENCES ai_prompt_template_versions(prompt_version_id) ON DELETE RESTRICT` |
| `provider_id`, `model_key` | `TEXT` | `NOT NULL`, внешний ключ к `ai_model_catalog` |
| `source_refs` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(source_refs) = 'array')` |
| `variables` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(variables) = 'object')` |
| `output_text` | `TEXT` | `NOT NULL`, `CHECK (length(trim(output_text)) > 0)` |
| `score` | `INTEGER` | `CHECK (score IS NULL OR (score >= 0 AND score <= 100))` |
| `notes` | `TEXT` |  |
| `actor_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(actor_id)) > 0)` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индекс: `ai_prompt_eval_runs_prompt_idx` на `(prompt_id, created_at DESC)`.

### `ai_agent_runs` — атрибуция персон (0070)

Добавлены колонки `agent_persona_id` и `owner_persona_id` (ссылаются на `persons.person_id` с `ON DELETE SET NULL`). Созданы индексы:

- `ai_agent_runs_agent_persona_idx` на `(agent_persona_id, started_at DESC) WHERE agent_persona_id IS NOT NULL`
- `ai_agent_runs_owner_persona_idx` на `(owner_persona_id, started_at DESC) WHERE owner_persona_id IS NOT NULL`

---

## Коммуникации и синхронизация почты (`0055`, `0058`, `0075`)

### `communication_messages` — локальное состояние (0055)

Добавлены поля:

- `local_state` — `TEXT NOT NULL DEFAULT 'active'`, ограничение `CHECK (local_state IN ('active', 'trash'))`
- `local_state_changed_at` — `TIMESTAMPTZ`
- `local_state_reason` — `TEXT`

Индекс: `communication_messages_local_state_idx` на `(local_state, COALESCE(occurred_at, projected_at) DESC)`.

### `communication_account_sync_settings` (0055)

Настройки синхронизации для аккаунта.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `account_id` | `TEXT` | `PRIMARY KEY REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE` |
| `sync_enabled` | `BOOLEAN` | `NOT NULL DEFAULT true` |
| `batch_size` | `INTEGER` | `NOT NULL DEFAULT 5`, `CHECK (batch_size BETWEEN 1 AND 500)` |
| `poll_interval_seconds` | `INTEGER` | `NOT NULL DEFAULT 300`, `CHECK (poll_interval_seconds BETWEEN 60 AND 86400)` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

### `communication_mail_sync_runs` (0055)

Запись каждого запуска синхронизации.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `run_id` | `TEXT` | `PRIMARY KEY` |
| `account_id` | `TEXT` | `NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE` |
| `trigger` | `TEXT` | `NOT NULL`, `CHECK (trigger IN ('scheduled', 'manual'))` |
| `status` | `TEXT` | `NOT NULL`, `CHECK (status IN ('queued','running','completed','failed','skipped','recoverable_full_resync_needed'))` |
| `phase` | `TEXT` | `NOT NULL`, `CHECK (phase IN ('idle','waiting_for_vault','listing','fetching','projecting','persons_graph','completed','failed','skipped'))` |
| `progress_mode` | `TEXT` | `NOT NULL DEFAULT 'indeterminate'`, `CHECK (progress_mode IN ('none','determinate','indeterminate'))` |
| `progress_percent` | `INTEGER` | `CHECK (progress_percent IS NULL OR (progress_percent >= 0 AND progress_percent <= 100))` |
| `processed_messages` | `BIGINT` | `NOT NULL DEFAULT 0` |
| `estimated_total_messages` | `BIGINT` |  |
| `current_batch_size` | `INTEGER` | `NOT NULL DEFAULT 0` |
| `fetched_messages` | `BIGINT` | `NOT NULL DEFAULT 0` |
| `projected_messages` | `BIGINT` | `NOT NULL DEFAULT 0` |
| `upserted_persons` | `BIGINT` | `NOT NULL DEFAULT 0` |
| `upserted_organizations` | `BIGINT` | `NOT NULL DEFAULT 0` |
| `checkpoint_before`, `checkpoint_after` | `JSONB` | `CHECK (… IS NULL OR jsonb_typeof(…) = 'object')` |
| `checkpoint_saved` | `BOOLEAN` | `NOT NULL DEFAULT false` |
| `error_code` | `TEXT` |  |
| `error_message` | `TEXT` |  |
| `started_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `completed_at` | `TIMESTAMPTZ` |  |
| `next_run_at` | `TIMESTAMPTZ` |  |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индексы:
- `communication_mail_sync_runs_account_started_idx` на `(account_id, started_at DESC)`
- уникальный `communication_mail_sync_runs_active_account_idx` на `(account_id) WHERE status IN ('queued','running','recoverable_full_resync_needed')` (запрещает более одного активного запуска на аккаунт)

### `communication_message_participants` (0055)

Участники сообщения (связка сообщения и персоны с ролью).

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `id` | `UUID` | `PRIMARY KEY DEFAULT gen_random_uuid()` |
| `message_id` | `TEXT` | `NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE` |
| `person_id` | `TEXT` | `NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE` |
| `email_address` | `TEXT` | `NOT NULL`, `CHECK (length(trim(email_address)) > 0)` |
| `display_name` | `TEXT` |  |
| `role` | `TEXT` | `NOT NULL`, `CHECK (role IN ('sender', 'recipient', 'cc', 'bcc'))` |
| `source` | `TEXT` | `NOT NULL DEFAULT 'email_sync'` |
| `confidence` | `REAL` | `NOT NULL DEFAULT 1.0`, `CHECK (confidence >= 0 AND confidence <= 1)` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальное ограничение: `(message_id, person_id, role, email_address)`.

Индексы:
- `communication_message_participants_message_idx` на `(message_id)`
- `communication_message_participants_person_idx` на `(person_id)`

### Пустые тела Telegram‑сообщений (0058)

Ограничение `communication_messages_body_not_empty` модифицировано: тело может быть пустым, если `channel_kind IN ('telegram_user', 'telegram_bot')` и `message_metadata` содержит ключ `tdlib_raw` (т.е. есть сырые данные TDLib).

### Пустая тема черновика письма (0075)

Снято ограничение `email_drafts_subject_not_empty` — тема черновика теперь может быть пустой.

---

## Персоны и идентичности (`0053`, `0059`, `0071`, `0072`, `0073`, `0074`)

### `person_identity_candidates` — исправление ограничений (0053)

- Значения `candidate_kind` обновлены: `merge_contacts` → `merge_persons`, `split_contact` → `split_person`
- Старые ограничения `contact_identity_candidate_kind_check`, `contact_identity_merge_has_right_contact` удалены
- Добавлено ограничение `person_identity_candidate_kind_check` с допустимыми значениями `merge_persons`, `attach_email_address`, `split_person`
- Обновлённое ограничение `person_identity_merge_has_right_person`: для `merge_persons` требуется `right_person_id IS NOT NULL`
- Удалены индексы `contact_identity_merge_pair_idx`, создан уникальный индекс `person_identity_merge_pair_idx` на `(candidate_kind, LEAST(left_person_id, COALESCE(right_person_id, left_person_id)), GREATEST(left_person_id, COALESCE(right_person_id, left_person_id))) WHERE candidate_kind = 'merge_persons'`

### `persons` — типы персон и `is_self` (0059)

- Добавлена колонка `is_self BOOLEAN NOT NULL DEFAULT false`
- Обновлён `person_type`: значения `NULL` или некорректные заменены на `'human'`; установлен `DEFAULT 'human'` и `NOT NULL`; ограничение `CHECK (person_type IN ('human', 'ai_agent', 'organization_proxy', 'system'))`
- Индексы: `persons_person_type_idx` на `(person_type)`, уникальный `persons_single_self_idx` на `(is_self) WHERE is_self = true`

### `person_identities` — типы и статусы (0071, 0072, 0073)

- **Типы идентичности (0071):** расширен список `CHECK (identity_type IN (...))` — добавлены `linkedin`, `website`, `mastodon`, `x`, `stackoverflow`, `habr`, `medium`, `orcid`, `google_scholar`, `document_mention`, `message_participant`
- **Статус (0072):** в `CHECK (status IN (...))` добавлено значение `disputed`
- **Неприкреплённые следы (0073):** колонка `person_id` становится `DROP NOT NULL` — идентичность может существовать без привязки к персоне

### `persona_dossier_snapshots` (0074)

Снимки досье персоны.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `dossier_snapshot_id` | `TEXT` | `PRIMARY KEY` |
| `persona_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(persona_id)) > 0)` |
| `dossier` | `JSONB` | `NOT NULL`, `CHECK (jsonb_typeof(dossier) = 'object')` |
| `source_refs` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(source_refs) = 'array')` |
| `review_state` | `TEXT` | `NOT NULL DEFAULT 'suggested'`, `CHECK (review_state IN ('suggested','user_confirmed','user_rejected'))` |
| `reviewed_by` | `TEXT` | `CHECK (reviewed_by IS NULL OR length(trim(reviewed_by)) > 0)` |
| `reviewed_at` | `TIMESTAMPTZ` |  |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(metadata) = 'object')` |
| `generated_at` | `TIMESTAMPTZ` | `NOT NULL` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальный индекс: `persona_dossier_snapshots_persona_latest_unique` на `(persona_id)` — не более одного снимка на персону. Индекс `persona_dossier_snapshots_review_state_idx` на `(review_state, updated_at DESC)`.

---

## Отношения (`0060`, `0061`, `0068`)

### `relationships` (0060)

Универсальная таблица отношений между любыми сущностями системы.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `relationship_id` | `TEXT` | `PRIMARY KEY` |
| `source_entity_kind`, `target_entity_kind` | `TEXT` | `NOT NULL`, `CHECK` (оба входят в набор `'persona','organization','project','communication','document','task','event','decision','obligation','knowledge'`) |
| `source_entity_id`, `target_entity_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `relationship_type` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `trust_score` | `NUMERIC(5,4)` | `NOT NULL DEFAULT 0.5000`, `CHECK (>= 0.0 AND <= 1.0)` |
| `strength_score` | `NUMERIC(5,4)` | `NOT NULL DEFAULT 0.5000`, `CHECK (>= 0.0 AND <= 1.0)` |
| `confidence` | `NUMERIC(5,4)` | `NOT NULL DEFAULT 1.0000`, `CHECK (>= 0.0 AND <= 1.0)` |
| `review_state` | `TEXT` | `NOT NULL DEFAULT 'suggested'`, `CHECK (IN ('suggested','system_accepted','user_confirmed','user_rejected'))` |
| `valid_from`, `valid_to` | `TIMESTAMPTZ` | ограничение `CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(…) = 'object')` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Дополнительные ограничения:

- `CHECK (source_entity_kind != target_entity_kind OR source_entity_id != target_entity_id)` — отношение не может ссылаться само на себя с одинаковым типом

Уникальный индекс: `relationships_active_unique` на `(source_entity_kind, source_entity_id, target_entity_kind, target_entity_id, relationship_type) WHERE valid_to IS NULL` — запрещает дубликаты активных отношений.

Индексы: `relationships_source_idx`, `relationships_target_idx`, `relationships_type_idx`, `relationships_review_state_idx`.

### `relationship_evidence` (0060)

Свидетельства, поддерживающие отношение.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `evidence_id` | `TEXT` | `PRIMARY KEY` |
| `relationship_id` | `TEXT` | `NOT NULL REFERENCES relationships(relationship_id) ON DELETE CASCADE` |
| `source_kind` | `TEXT` | `NOT NULL`, `CHECK (source_kind IN ('communication','document','event','memory','knowledge','decision','obligation','task','project','organization','persona','raw_record'))` |
| `source_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `excerpt` | `TEXT` |  |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(…) = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальное ограничение: `(relationship_id, source_kind, source_id)`. Индексы: `relationship_evidence_relationship_idx`, `relationship_evidence_source_idx`.

### Проекции графа для отношений (0061, 0068)

#### `graph_edges` (0061)

Добавлено значение `'entity_relationship'` в `CHECK (relationship_type IN (...))`.

#### `graph_nodes` (0061, 0068)

Последовательно расширялся список `node_kind`:

- (0061) добавлены `'person','email_address','message','document','project'`
- (0065) добавлен `'decision'`
- (0066) добавлен `'obligation'`
- (0068) окончательный набор: `'person','email_address','message','document','project','organization','task','event','decision','obligation','knowledge'`

#### `graph_evidence` (0061)

`source_kind` расширен значениями `'contact','person','message','document','raw_record','relationship'`; позже (0065, 0066) добавлены `'decision'` и `'obligation'`.

---

## Противоречия (`0062`)

### `contradiction_observations`

Наблюдения о противоречиях между двумя источниками.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `observation_id` | `TEXT` | `PRIMARY KEY` |
| `old_source_kind` | `TEXT` | `NOT NULL`, `CHECK` (из набора `'communication','document','event','memory','knowledge','decision','obligation','task','relationship','raw_record'`) |
| `old_source_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `new_source_kind` | `TEXT` | `NOT NULL`, аналогичный `CHECK` |
| `new_source_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `affected_entities` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(…) IN ('array','object'))` |
| `conflict_type` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `old_claim` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `new_claim` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `confidence` | `NUMERIC(5,4)` | `NOT NULL`, `CHECK (>= 0.0 AND <= 1.0)` |
| `severity` | `TEXT` | `NOT NULL`, `CHECK (IN ('low','medium','high','critical'))` |
| `review_state` | `TEXT` | `NOT NULL DEFAULT 'suggested'`, `CHECK (IN ('suggested','user_confirmed','user_rejected'))` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(…) = 'object')` |
| `reviewed_by` | `TEXT` | `CHECK (reviewed_by IS NULL OR length(trim(reviewed_by)) > 0)` |
| `reviewed_at` | `TIMESTAMPTZ` |  |
| `resolution` | `TEXT` |  |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальный индекс: `contradiction_observations_source_unique` на `(old_source_kind, old_source_id, new_source_kind, new_source_id, conflict_type)`. Индексы: `contradiction_observations_review_state_idx`, `contradiction_observations_new_source_idx`, `contradiction_observations_old_source_idx`.

---

## Обязательства (`0063`, `0066`)

### `obligations`

Обязательство одной сущности перед другой.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `obligation_id` | `TEXT` | `PRIMARY KEY` |
| `obligated_entity_kind` | `TEXT` | `NOT NULL`, `CHECK` (из того же набора типов, что и отношения) |
| `obligated_entity_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `beneficiary_entity_kind` | `TEXT` | может быть `NULL`, тогда и `beneficiary_entity_id NULL`; если не `NULL`, то `CHECK` по типам и `length > 0` |
| `beneficiary_entity_id` | `TEXT` |  |
| `statement` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `status` | `TEXT` | `NOT NULL DEFAULT 'open'`, `CHECK (IN ('open','fulfilled','waived','disputed','canceled'))` |
| `review_state` | `TEXT` | `NOT NULL DEFAULT 'suggested'`, `CHECK (IN ('suggested','user_confirmed','user_rejected'))` |
| `due_at` | `TIMESTAMPTZ` |  |
| `condition` | `TEXT` |  |
| `risk_state` | `TEXT` | `NOT NULL DEFAULT 'none'`, `CHECK (IN ('none','watch','at_risk','breached'))` |
| `confidence` | `NUMERIC(5,4)` | `NOT NULL DEFAULT 1.0000`, `CHECK (>= 0.0 AND <= 1.0)` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(…) = 'object')` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальный индекс: `obligations_active_unique` на `(obligated_entity_kind, obligated_entity_id, COALESCE(beneficiary_entity_kind, ''), COALESCE(beneficiary_entity_id, ''), lower(statement)) WHERE status IN ('open','disputed')`. Индексы по обязанному, бенефициару, статусу, состоянию проверки, уровню риска.

### `obligation_evidence` (0063)

Доказательства обязательства (структура аналогична `relationship_evidence`, с полем `confidence`).

### `obligation_task_links` (0063)

Связь обязательства с задачей.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `obligation_id` | `TEXT` | `NOT NULL REFERENCES obligations(obligation_id) ON DELETE CASCADE` |
| `task_id` | `TEXT` | `NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE` |
| `link_kind` | `TEXT` | `NOT NULL DEFAULT 'related'`, `CHECK (IN ('related','fulfillment_task','follow_up_task','evidence_task'))` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

`PRIMARY KEY (obligation_id, task_id, link_kind)`. Индекс: `obligation_task_links_task_idx` на `(task_id)`.

### Проекция графа обязательств (0066)

Добавлен `'obligation'` в `graph_nodes.node_kind` и `graph_evidence.source_kind`.

---

## Решения (`0064`, `0065`)

### `decisions`

Зафиксированное решение.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `decision_id` | `TEXT` | `PRIMARY KEY` |
| `title` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `status` | `TEXT` | `NOT NULL DEFAULT 'active'`, `CHECK (IN ('active','superseded','reversed','deprecated'))` |
| `rationale` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `alternatives` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK (jsonb_typeof(…) = 'array')` |
| `decided_by_entity_kind`, `decided_by_entity_id` | `TEXT` | либо оба `NULL`, либо оба не `NULL` (тип из набора, `length > 0`) |
| `decided_at` | `TIMESTAMPTZ` |  |
| `review_state` | `TEXT` | `NOT NULL DEFAULT 'suggested'`, `CHECK (IN ('suggested','user_confirmed','user_rejected'))` |
| `confidence` | `NUMERIC(5,4)` | `NOT NULL DEFAULT 1.0000`, `CHECK (>= 0.0 AND <= 1.0)` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(…) = 'object')` |
| `created_at`, `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индексы: `decisions_status_idx`, `decisions_review_state_idx`, `decisions_decider_idx`, `decisions_decided_at_idx`.

### `decision_evidence` (0064)

Доказательства решения (структура аналогична другим таблицам evidence).

### `decision_impacted_entities` (0064)

Сущности, на которые влияет решение.

| Колонка | Тип | Ограничения / Примечания |
|---------|-----|--------------------------|
| `decision_id` | `TEXT` | `NOT NULL REFERENCES decisions(decision_id) ON DELETE CASCADE` |
| `entity_kind` | `TEXT` | `NOT NULL`, `CHECK` (стандартный набор) |
| `entity_id` | `TEXT` | `NOT NULL`, `CHECK (length(trim(…)) > 0)` |
| `impact_type` | `TEXT` | `NOT NULL DEFAULT 'related'`, `CHECK (length(trim(…)) > 0)` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK (jsonb_typeof(…) = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

`PRIMARY KEY (decision_id, entity_kind, entity_id)`. Индекс: `decision_impacted_entities_entity_idx`.

### Проекция графа решений (0065)

Добавлены `'decision'` в `graph_nodes.node_kind` и `graph_evidence.source_kind`.

---

## Прочие изменения

### Удаление настройки `frontend.actor_id` (0052)

```sql
DELETE FROM application_settings WHERE setting_key = 'frontend.actor_id';
```

### Хранилище секретов `host_vault` (0054)

В ограничение `secret_references_store_kind` добавлено значение `'host_vault'`, полный список:

- `os_keychain`
- `encrypted_vault`
- `database_encrypted_vault`
- `host_vault`
- `external_vault`
- `test_double`

### `email_invoices.linked_person_id` (0056)

Добавлена колонка `linked_person_id TEXT` и индекс `email_invoices_linked_person_idx`.
```

### Source coverage / Покрытие источников

| Source file | Covered facts |
|-------------|---------------|
| `0051_tasks_rules_templates.sql` | Таблицы `task_rules`, `task_templates`, `task_snapshots`; их колонки, ограничения, индексы; вставка 8 шаблонов с полями и чек-листами. |
| `0052_remove_frontend_actor_setting.sql` | Удаление настройки `frontend.actor_id` из `application_settings`. |
| `0053_fix_person_identity_candidate_kind_constraints.sql` | Переименование значений `candidate_kind`, обновление CHECK-ограничений, удаление старых индексов и создание нового уникального индекса. |
| `0054_add_host_vault_secret_store_kind.sql` | Добавление `host_vault` в CHECK-ограничение `secret_references.store_kind`. |
| `0055_mail_sync_local_trash.sql` | Колонки локального состояния в `communication_messages`; таблицы `communication_account_sync_settings`, `communication_mail_sync_runs`, `communication_message_participants`; все ограничения и индексы. |
| `0056_email_invoices_linked_person.sql` | Колонка `linked_person_id` и индекс в `email_invoices`. |
| `0057_ai_control_center.sql` | Все AI-таблицы: провайдеры, секреты, модели, маршруты, промпты и их версии, оценочные запуски; начальные данные (провайдер ollama, модели, маршруты, системные промпты). |
| `0058_allow_empty_telegram_tdlib_message_bodies.sql` | Модификация ограничения `communication_messages_body_not_empty` для Telegram-сообщений с `tdlib_raw`. |
| `0059_persona_owner_type_constraints.sql` | Колонка `is_self`, приведение `person_type`, новое CHECK-ограничение на типы персон, индексы. |
| `0060_create_relationships.sql` | Таблицы `relationships` и `relationship_evidence`; все колонки, ограничения, индексы. |
| `0061_relationship_graph_projection.sql` | Расширение CHECK-ограничений `graph_edges.relationship_type` и `graph_evidence.source_kind`. |
| `0062_create_contradiction_observations.sql` | Таблица `contradiction_observations`; все колонки, ограничения, индексы. |
| `0063_create_obligations.sql` | Таблицы `obligations`, `obligation_evidence`, `obligation_task_links`; все детали. |
| `0064_create_decisions.sql` | Таблицы `decisions`, `decision_evidence`, `decision_impacted_entities`; все детали. |
| `0065_decision_graph_projection.sql` | Добавление `decision` в `graph_nodes` и `graph_evidence`. |
| `0066_obligation_graph_projection.sql` | Добавление `obligation` в `graph_nodes` и `graph_evidence`. |
| `0067_task_candidate_kind_metadata.sql` | Колонки `candidate_kind`, `candidate_metadata` с ограничениями и индексом в `task_candidates`. |
| `0068_expand_relationship_graph_node_kinds.sql` | Финальный набор `node_kind` для `graph_nodes`. |
| `0069_relax_task_candidate_requirement.sql` | Снятие `NOT NULL` с полей `tasks`; обновлённый `source_kind` CHECK. |
| `0070_ai_run_persona_attribution.sql` | Колонки `agent_persona_id`, `owner_persona_id` и индексы в `ai_agent_runs`. |
| `0071_person_identity_trace_types.sql` | Расширенный список типов идентичности в CHECK-ограничении. |
| `0072_person_identity_disputed_status.sql` | Добавление `disputed` в допустимые статусы `person_identities`. |
| `0073_person_identity_unattached_traces.sql` | Разрешение `NULL` для `person_id` в `person_identities`. |
| `0074_persona_dossier_snapshots.sql` | Таблица `persona_dossier_snapshots`; все колонки, ограничения, индексы. |
| `0075_allow_empty_email_draft_subject.sql` | Удаление ограничения `email_drafts_subject_not_empty`. |

### Drift candidates / Кандидаты на drift

- В миграции `0055` есть комментарий `-- ADR-0080: per-account mail sync progress and local-only trash`. Так как содержание ADR-0080 в данном чанке отсутствует, невозможно подтвердить, что реализация в миграции полностью соответствует архитектурному решению.
- Миграции `0061`, `0065`, `0066`, `0068` многократно изменяют CHECK-ограничения таблиц `graph_nodes`, `graph_edges`, `graph_evidence`. Без кода приложения, использующего эти таблицы, нельзя оценить, поддерживает ли приложение все перечисленные значения на каждом этапе, или возможен разрыв между схемой и логикой.
- Для всех таблиц, создаваемых в этих миграциях (AI, отношения, обязательства, решения, противоречия), отсутствует исходный код сервисного слоя; потенциальный дрейф между схемой и бизнес-логикой не может быть проверен.
- В остальном из предоставленного контекста расхождений не видно.
