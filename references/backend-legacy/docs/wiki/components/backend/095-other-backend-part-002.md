---
chunk_id: 095-other-backend-part-002
batch_id: batch-20260628T214902
group: backend
role: other
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 095-other-backend-part-002 — backend/other

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/backend.md` должна быть обновлена документацией по структуре базы данных, определяемой миграциями `0026`–`0050`. В текущем виде страница, вероятно, либо отсутствует, либо неполна. Предлагается создать (или заменить) её сводным описанием всех таблиц, их колонок, ограничений и индексов, извлечённым непосредственно из приложенных SQL-файлов. Описание даётся на русском языке без домысливания семантики, не подтверждённой исходниками.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend-компоненты (база данных)

Документ описывает таблицы, создаваемые миграциями `0026`–`0050` проекта makosh‑hub.
Информация основывается исключительно на DDL; поведенческие детали,
не подтверждённые исходным SQL, не приводятся.

## Email‑подсистема

### `email_rules` (миграция 0026)

Правила обработки входящей почты.

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `rule_id` | `TEXT` | `PRIMARY KEY` |
| `name` | `TEXT` | `NOT NULL`, `CHECK(length(trim(name)) > 0)` |
| `description_nl` | `TEXT` | `NOT NULL DEFAULT ''` |
| `conditions_json` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK(jsonb_typeof = 'array')` |
| `actions_json` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK(jsonb_typeof = 'array')` |
| `mode` | `TEXT` | `NOT NULL DEFAULT 'suggest'`, `CHECK(IN ('suggest','ask_before_execute','auto_execute','dry_run'))` |
| `enabled` | `BOOLEAN` | `NOT NULL DEFAULT true` |
| `match_count` | `BIGINT` | `NOT NULL DEFAULT 0` |
| `last_matched_at` | `TIMESTAMPTZ` | |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

### `email_templates` (миграция 0027)

Шаблоны писем.

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `template_id` | `TEXT` | `PRIMARY KEY` |
| `name` | `TEXT` | `NOT NULL`, `CHECK(length(trim(name)) > 0)` |
| `subject_template` | `TEXT` | `NOT NULL`, `CHECK(length(trim(subject_template)) > 0)` |
| `body_template` | `TEXT` | `NOT NULL DEFAULT ''` |
| `variables` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK(jsonb_typeof = 'array')` |
| `language` | `TEXT` | |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

### `email_personas` (миграция 0028)

Персоны отправителей (привязаны к учётным записям почтовых провайдеров).

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `persona_id` | `TEXT` | `PRIMARY KEY` |
| `name` | `TEXT` | `NOT NULL`, `CHECK(length(trim(name)) > 0)` |
| `account_id` | `TEXT` | `NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE` |
| `display_name` | `TEXT` | `NOT NULL` |
| `signature` | `TEXT` | `NOT NULL DEFAULT ''` |
| `default_language` | `TEXT` | |
| `default_tone` | `TEXT` | |
| `is_default` | `BOOLEAN` | `NOT NULL DEFAULT false` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK(jsonb_typeof = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальный частичный индекс: `email_personas_one_default_per_account` на `(account_id) WHERE is_default = true`.

### `email_drafts` (миграция 0029)

Черновики писем.

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `draft_id` | `TEXT` | `PRIMARY KEY` |
| `account_id` | `TEXT` | `NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE` |
| `persona_id` | `TEXT` | |
| `to_recipients` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK(jsonb_typeof = 'array')` |
| `cc_recipients` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb` |
| `bcc_recipients` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb` |
| `subject` | `TEXT` | `NOT NULL`, `CHECK(length(trim(subject)) > 0)` |
| `body_text` | `TEXT` | `NOT NULL DEFAULT ''` |
| `body_html` | `TEXT` | |
| `in_reply_to` | `TEXT` | |
| `message_references` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb` |
| `status` | `TEXT` | `NOT NULL DEFAULT 'draft'`, `CHECK(IN ('draft','scheduled','sending','sent','failed'))` |
| `scheduled_send_at` | `TIMESTAMPTZ` | |
| `send_attempts` | `INTEGER` | `NOT NULL DEFAULT 0` |
| `last_error` | `TEXT` | |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK(jsonb_typeof = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индекс: `email_drafts_account_status_idx` на `(account_id, status, updated_at DESC)`.

### `email_invoices` (миграция 0030)

Счета, полученные по email.

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `invoice_id` | `TEXT` | `PRIMARY KEY` |
| `message_id` | `TEXT` | |
| `amount` | `DOUBLE PRECISION` | |
| `currency` | `TEXT` | |
| `invoice_number` | `TEXT` | |
| `issue_date` | `TIMESTAMPTZ` | |
| `due_date` | `TIMESTAMPTZ` | |
| `counterparty` | `TEXT` | |
| `tax_id` | `TEXT` | |
| `status` | `TEXT` | `NOT NULL DEFAULT 'received'`, `CHECK(IN ('received','recognized','needs_review','approved','paid','closed','rejected'))` |
| `linked_project_id` | `TEXT` | |
| `linked_contact_id` | `TEXT` | |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK(jsonb_typeof = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индекс: `email_invoices_status_idx` на `(status, due_date)`.

### `email_legal_documents` (миграция 0031)

Юридические документы, связанные с email.

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `document_id` | `TEXT` | `PRIMARY KEY` |
| `message_id` | `TEXT` | |
| `document_type` | `TEXT` | `NOT NULL DEFAULT 'other'`, `CHECK(IN ('contract','nda','msa','dpa','agreement','legal_notice','claim','court_document','tax_notice','government_doc','other'))` |
| `title` | `TEXT` | `NOT NULL`, `CHECK(length(trim(title)) > 0)` |
| `parties` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb` |
| `effective_date` | `TIMESTAMPTZ` | |
| `expiry_date` | `TIMESTAMPTZ` | |
| `amount` | `DOUBLE PRECISION` | |
| `currency` | `TEXT` | |
| `status` | `TEXT` | `NOT NULL DEFAULT 'draft'`, `CHECK(IN ('active','expired','pending_review','signed','terminated','draft'))` |
| `linked_project_id` | `TEXT` | |
| `risks` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK(jsonb_typeof = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

### `email_certificates` (миграция 0032)

Сертификаты (S/MIME, PGP, ГОСТ и др.).

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `cert_id` | `TEXT` | `PRIMARY KEY` |
| `owner_name` | `TEXT` | `NOT NULL` |
| `issuer` | `TEXT` | `NOT NULL DEFAULT ''` |
| `serial_number` | `TEXT` | |
| `fingerprint_sha256` | `TEXT` | |
| `valid_from` | `TIMESTAMPTZ` | |
| `valid_until` | `TIMESTAMPTZ` | |
| `cert_type` | `TEXT` | `NOT NULL DEFAULT 'unknown'`, `CHECK(IN ('smime','pgp','pdf_sign','cades','xades','gost_sign','unknown'))` |
| `provider` | `TEXT` | `NOT NULL DEFAULT 'other'`, `CHECK(IN ('fnmt','dnie','cryptopro','gost','apple_keychain','pkcs12','yubikey','usb_token','other'))` |
| `storage_kind` | `TEXT` | `NOT NULL DEFAULT 'encrypted_vault'`, `CHECK(IN ('os_keychain','encrypted_vault','pkcs12_file','pfx_file','smart_card','usb_token','external_vault'))` |
| `storage_ref` | `TEXT` | |
| `trust_status` | `TEXT` | `NOT NULL DEFAULT 'untrusted'`, `CHECK(IN ('trusted','untrusted','expired','revoked','pending_verification','self_signed'))` |
| `is_revoked` | `BOOLEAN` | `NOT NULL DEFAULT false` |
| `usage` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb` |
| `linked_message_id` | `TEXT` | |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индекс: `email_certs_expiry_idx` на `(valid_until) WHERE valid_until IS NOT NULL AND is_revoked = false`.

## Контакты / Персоны

### Расширение `contacts` (миграция 0033)

К таблице `contacts` добавляются колонки:

- `language TEXT`
- `tone TEXT`
- `trust_score SMALLINT` (с `CHECK(IS NULL OR (0..100))`)
- `avg_response_hours DOUBLE PRECISION`
- `preferred_channel TEXT`
- `last_interaction_at TIMESTAMPTZ`
- `interaction_count INTEGER NOT NULL DEFAULT 0`
- `frequent_topics JSONB NOT NULL DEFAULT '[]'::jsonb`
- `writing_style TEXT`
- `contact_metadata JSONB NOT NULL DEFAULT '{}'::jsonb` (`CHECK(jsonb_typeof = 'object')`)
- `is_favorite BOOLEAN NOT NULL DEFAULT false`
- `notes TEXT`

Индексы: `contacts_trust_score_idx`, `contacts_last_interaction_idx`, `contacts_favorite_idx`.

### Переименование `contacts` → `persons` (миграция 0034)

Таблица `contacts` переименована в `persons`, `contact_identity_candidates` в `person_identity_candidates`.
Соответствующие колонки (`contact_id`→`person_id`, `contact_metadata`→`person_metadata`) также переименованы.
Значения идентификаторов вида `contact:v1:` заменены на `person:v1:`.
Обновлены ссылки в `event_log`, `graph_nodes`, переименованы ограничения и индексы.

### Мультиканальная модель персон (миграция 0035)

К `persons` добавлены колонки:

- `person_type TEXT`
- `primary_role TEXT`
- `organization_reference TEXT`
- `timezone TEXT`

Создана таблица `person_identities` (идентификаторы на разных каналах):

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `id` | `UUID` | `PRIMARY KEY DEFAULT gen_random_uuid()` |
| `person_id` | `TEXT` | `NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE` |
| `identity_type` | `TEXT` | `NOT NULL`, `CHECK(IN ('email','telegram','whatsapp','phone','github','linkedin','website','mastodon','x','stackoverflow','habr','medium','orcid','google_scholar'))` |
| `identity_value` | `TEXT` | `NOT NULL` |
| `source` | `TEXT` | `NOT NULL DEFAULT 'manual'` |
| `confidence` | `REAL` | `NOT NULL DEFAULT 1.0` |
| `last_verified_at` | `TIMESTAMPTZ` | |
| `status` | `TEXT` | `NOT NULL DEFAULT 'active'`, `CHECK(IN ('active','outdated','unreachable','blocked'))` |
| `metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK(jsonb_typeof = 'object')` |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Уникальный индекс: `person_identities_type_value_idx` на `(identity_type, identity_value) WHERE status = 'active'`.

Таблица `person_roles` (назначение ролей):

- `id UUID PK`, `person_id TEXT NOT NULL REFERENCES persons…`, `role TEXT NOT NULL`, `assigned_by TEXT`, `assigned_at TIMESTAMPTZ`.
- `UNIQUE(person_id, role)`.

Таблица `person_personas` (именованные контексты взаимодействия):

- `persona_id TEXT PK`, `person_id TEXT NOT NULL REFERENCES persons…`, `name TEXT NOT NULL`, `context TEXT`, `default_tone TEXT`, `default_language TEXT`, `preferred_channel TEXT`, `metadata JSONB`, временные метки.
- `CHECK(length(trim(name)) > 0)`, `CHECK(jsonb_typeof(metadata) = 'object')`.

Выполнен backfill: для каждой персоны с непустым `email_address` создана запись в `person_identities` с типом `email`.

### Память и хронология персон (миграция 0036)

Таблица `person_facts` (извлечённые факты):

- `id UUID PK`, `person_id TEXT`, `fact_type TEXT`, `value TEXT`, `source TEXT`, `confidence REAL` (0..1), `last_verified_at`, `valid_from`, `valid_to`, `is_active BOOLEAN DEFAULT true`, временные метки.

Таблица `person_memory_cards`:

- `id UUID PK`, `person_id TEXT`, `title TEXT NOT NULL`, `description TEXT NOT NULL`, `source TEXT NOT NULL`, `confidence REAL` (0..1), `importance SMALLINT` (1..10, default 5), `created_at`, `last_verified_at`.

Таблица `person_preferences`:

- `id UUID PK`, `person_id TEXT`, `preference_type TEXT NOT NULL`, `value TEXT NOT NULL`, `source TEXT`, `confidence REAL` (0..1), `last_verified_at`, временные метки.
- `UNIQUE(person_id, preference_type)`.

Таблица `person_snapshots` (срезы состояния):

- `id UUID PK`, `person_id TEXT`, `snapshot_date TIMESTAMPTZ`, `data JSONB NOT NULL` (`CHECK(jsonb_typeof = 'object')`), `source TEXT DEFAULT 'manual'`.

Таблица `person_knowledge_conflicts`:

- `id UUID PK`, `person_id TEXT`, `field TEXT`, `value_a TEXT`, `value_b TEXT`, `source_a`, `source_b`, `detected_at`, `resolved_at`, `resolution`.

Таблица `relationship_events` (события отношений):

- `id UUID PK`, `person_id TEXT`, `event_type TEXT`, `title TEXT NOT NULL`, `description`, `occurred_at TIMESTAMPTZ`, `source TEXT`, `related_entity_id`, `related_entity_kind`, `confidence REAL` (0..1), `metadata JSONB`, `created_at`.

К таблице `persons` добавлены колонки «Коммуникационное ДНК»:

- `communication_style TEXT`, `verbosity TEXT`, `technical_depth TEXT`, `question_frequency TEXT`, `call_preference TEXT`, `response_pattern TEXT`, `active_hours JSONB`, `active_days JSONB`.

### Обогащение, экспертиза, доверие, риски (миграция 0037)

Таблица `enrichment_results` (результаты обогащения из внешних источников):

- `id UUID PK`, `person_id TEXT`, `source TEXT NOT NULL`, `url TEXT`, `data JSONB`, `confidence REAL` (0..1), `status TEXT CHECK(IN ('pending','applied','rejected','conflict'))`, `last_checked_at`, `applied_at`, `created_at`.

Таблица `person_expertise` (навыки и домены):

- `id UUID PK`, `person_id TEXT`, `skill TEXT NOT NULL`, `domain TEXT`, `evidence TEXT`, `source TEXT NOT NULL`, `confidence REAL` (0..1), `last_verified_at`, `endorsed_by_person_id TEXT`, временные метки.

Таблица `person_promises` (отслеживаемые обещания):

- `id UUID PK`, `person_id TEXT`, `description TEXT NOT NULL`, `source_message_id`, `promised_at`, `due_at`, `fulfilled_at`, `status TEXT CHECK(IN ('pending','fulfilled','broken','forgiven'))`.

Таблица `person_risks`:

- `id UUID PK`, `person_id TEXT`, `risk_type TEXT NOT NULL`, `description TEXT NOT NULL`, `severity TEXT CHECK(IN ('low','medium','high','critical'))`, `source TEXT`, `confidence REAL` (0..1), `created_at`, `resolved_at`, `resolution`.

Добавлены колонки здоровья в `persons`:

- `health_status TEXT DEFAULT 'healthy' CHECK(IN ('healthy','needs_attention','at_risk','dormant'))`
- `last_health_check TIMESTAMPTZ`
- `communication_gap_days INT DEFAULT 0`
- `watchlist BOOLEAN NOT NULL DEFAULT false`

Индекс: `persons_watchlist_idx` на `(person_id) WHERE watchlist = true`.

## Организации

### Ядро `organizations` (миграция 0038)

| Столбец | Тип | Ограничения |
|---------|-----|-------------|
| `organization_id` | `TEXT` | `PRIMARY KEY` |
| `display_name` | `TEXT` | `NOT NULL`, `CHECK(length(trim(…)) > 0)` |
| `legal_name` | `TEXT` | |
| `org_type` | `TEXT` | |
| `status` | `TEXT` | `NOT NULL DEFAULT 'active'`, `CHECK(IN ('active','inactive','archived','watchlist','blocked','unknown'))` |
| `country` | `TEXT` | |
| `city` | `TEXT` | |
| `address` | `TEXT` | |
| `website` | `TEXT` | |
| `industry` | `TEXT` | |
| `description` | `TEXT` | |
| `primary_language` | `TEXT` | |
| `timezone` | `TEXT` | |
| `trust_score` | `SMALLINT` | `CHECK(IS NULL OR (0..100))` |
| `health_status` | `TEXT` | `DEFAULT 'healthy'` |
| `priority` | `TEXT` | `DEFAULT 'medium'`, `CHECK(IN ('low','medium','high','critical'))` |
| `notes` | `TEXT` | |
| `tags` | `JSONB` | `NOT NULL DEFAULT '[]'::jsonb`, `CHECK(jsonb_typeof = 'array')` |
| `org_metadata` | `JSONB` | `NOT NULL DEFAULT '{}'::jsonb`, `CHECK(jsonb_typeof = 'object')` |
| `last_interaction_at` | `TIMESTAMPTZ` | |
| `interaction_count` | `INT` | `NOT NULL DEFAULT 0` |
| *Юридические*: `registration_number`, `country_of_registration`, `vat`, `cif`, `nif`, `tax_id`, `legal_address`, `registry_source`, `registry_last_verified` | |
| *ДНК*: `communication_style`, `verbosity`, `formality`, `secondary_languages JSONB`, `preferred_tone`, `official_style_required BOOL DEFAULT false` | |
| *Здоровье*: `last_health_check`, `watchlist BOOL NOT NULL DEFAULT false` | |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` |

Индексы: по `org_type`, `status`, `vat`, `website`, `watchlist`, `trust_score`.

### Идентификация, департаменты, связи (миграция 0039)

Таблица `organization_identities` (мультиканальные идентификаторы):

- Типы: `domain`, `website`, `email_domain`, `support_email`, `billing_email`, `legal_email`, `phone`, `vat`, `cif`, `nif`, `registry_number`, `github_org`, `linkedin_page`, `twitter`, `mastodon`, `support_portal`, `customer_portal`, `tax_portal`, `app_portal`.
- Статусы: `active`, `outdated`, `unreachable`, `blocked`.

Таблица `organization_aliases`:

- `name TEXT NOT NULL`, `alias_type TEXT CHECK(IN ('legal','trading','brand','former'))`, `source`, `confidence` (0..1), `valid_from`, `valid_to`.

Таблица `organization_domains`:

- `domain TEXT NOT NULL`, `domain_type TEXT CHECK(IN ('primary','additional','email','portal','former'))`.
- Уникальный индекс: `org_domains_unique_active` на `(organization_id, domain) WHERE domain_type != 'former'`.

Таблица `organization_departments`:

- `name TEXT NOT NULL`, `description`, `parent_department_id UUID REFERENCES organization_departments(id)`.

Таблица `organization_contact_links` (связь персон с организациями):

- `organization_id TEXT NOT NULL REFERENCES organizations…`, `person_id TEXT NOT NULL REFERENCES persons…`, `role TEXT`, `department TEXT`, `source TEXT DEFAULT 'manual'`, `confidence REAL`, `valid_from`/`valid_to`, `is_primary BOOL DEFAULT false`.
- `UNIQUE(organization_id, person_id, role)`.

Таблица `related_organizations`:

- `organization_id`, `related_organization_id`, `relation_type TEXT CHECK(IN ('parent','subsidiary','division','partner','supplier','customer'))`, `source`, `confidence`.

### Память организаций (миграция 0040)

- `organization_facts` – аналог `person_facts`.
- `organization_memory_cards` – аналог `person_memory_cards`.
- `organization_preferences` – аналог `person_preferences` (с уникальностью `(organization_id, preference_type)`).
- `organization_required_documents` – типы требуемых документов (`document_type TEXT NOT NULL`, `description`, `source`, `confidence`).
- `organization_snapshots` – срезы состояния.
- `organization_knowledge_conflicts` – противоречия в данных.

### Хронология и автоматизация (миграция 0041)

- `organization_timeline_events` – события в хронологии организации.
- `organization_templates` – шаблоны (`template_type CHECK(IN ('email','document'))`).
- `organization_portals` – порталы (`portal_type CHECK(IN ('tax','customer','banking','support','billing','admin','app'))`), `url`, `login_hint`, `secret_reference`.
- `organization_procedures` – процедуры, `steps JSONB CHECK(jsonb_typeof = 'array')`.
- `organization_playbooks` – сценарии (`approval_mode CHECK(IN ('auto','confirm','disabled'))`), `steps JSONB`.
- `organization_quick_actions` – быстрые действия (`label`, `action_type`, `action_params JSONB`, `sort_order`).

### Финансы, контракты, услуги, обогащение (миграция 0042)

- `organization_financial_info` – банковские реквизиты, условия оплаты.
- `organization_contracts` – договоры, статусы: `draft`, `active`, `expired`, `terminated`, `renewed`.
- `organization_compliance` – комплаенс, статусы: `compliant`, `pending`, `expired`, `not_applicable`.
- `organization_services` – оказываемые услуги.
- `organization_products` – продукты.
- `organization_enrichment_results` – результаты обогащения (аналог `enrichment_results` для персон).

### Риски и предупреждения (миграция 0043)

- `organization_risks` – риски (`severity CHECK(IN ('low','medium','high','critical'))`, `confidence` 0..1).
- `organization_alerts` – алерты (`severity CHECK(IN ('low','medium','high','critical'))`).

## Календарь

### Ядро календаря (миграция 0044)

**`calendar_accounts`** – учётные записи календарных провайдеров.

- `account_id TEXT PK`, `provider TEXT CHECK(IN ('google','microsoft','exchange','apple','caldav','ics','local'))`, `account_name TEXT NOT NULL`, `email TEXT`, `credentials_reference TEXT`, `sync_status TEXT CHECK(IN ('idle','syncing','synced','error','disabled'))`, `capabilities JSONB`.

**`calendar_sources`** – календари внутри учётной записи.

- `source_id TEXT PK`, `account_id TEXT REFERENCES calendar_accounts ON DELETE CASCADE`, `provider_calendar_id TEXT`, `name TEXT NOT NULL`, `color TEXT`, `timezone TEXT`, `visibility TEXT CHECK(IN ('private','public','confidential'))`, `read_only BOOLEAN DEFAULT false`, `sync_enabled BOOLEAN DEFAULT true`, `capabilities JSONB`.

**`calendar_events`** – события.

- `event_id TEXT PK`, `source_event_id TEXT`, `account_id TEXT`, `source_id TEXT`, `title TEXT NOT NULL`, `description TEXT`, `location TEXT`, `start_at`/`end_at TIMESTAMPTZ NOT NULL`, `timezone TEXT`, `all_day BOOLEAN DEFAULT false`, `recurrence_rule TEXT`, `status TEXT CHECK(IN ('scheduled','prepared','in_progress','completed','cancelled','rescheduled','no_show','needs_follow_up','archived'))`, `visibility TEXT CHECK(IN ('private','public','confidential','hidden_details','local_only'))`, `event_type TEXT`, `importance_score REAL` (0..1), `readiness_score REAL` (0..1), `sync_status TEXT CHECK(IN ('local','syncing','synced','conflict','error'))`.
- Индексы: по `account_id`, `source_id`, `start_at`, `end_at`, `status`, `event_type`, диапазону `(start_at, end_at)`.

### Участники, связи, контекст, повестки, чеклисты, заметки (миграция 0045)

- **`event_participants`** – участники события: `event_id`, `person_id`, `email`, `display_name`, `role CHECK(IN ('organizer','required','optional','attendee','speaker'))`, `response_status CHECK(IN ('needs_action','accepted','declined','tentative','no_response'))`, `organization_id`, `timezone`, `confidence`.
- **`event_relations`** – связи события с другими сущностями: `entity_type CHECK(IN ('person','organization','project','document','task','email','note','decision','obligation','recording'))`, `entity_id`, `relation_type`.
- **`event_context_packs`** – контекстные пакеты: `summary`, `participants_summary`, `documents JSONB`, `tasks JSONB`, `open_questions JSONB`, `risks JSONB`, `suggested_agenda JSONB`, `suggested_actions JSONB`, `generated_at`, `model`.
- **`event_agendas`** – повестки: `items JSONB`.
- **`event_checklists`** – чеклисты: `items JSONB`.
- **`meeting_notes`** – заметки встречи: `content TEXT NOT NULL`, `format TEXT DEFAULT 'markdown'`, `source`, `linked_note_id`.
- **`meeting_outcomes`** – результаты встречи: `outcome_type CHECK(IN ('decision','task','promise','risk','question','document_request','follow_up','agreement','blocker'))`, `title`, `description`, `owner_person_id`, `due_date`, `linked_entity_id`.
- **`event_recordings`** – записи: `file_path`, `duration_seconds`, `transcript_id`, `processing_status CHECK(IN ('pending','transcribing','transcribed','failed'))`.
- **`event_transcripts`** – транскрипты: `text TEXT NOT NULL`, `language DEFAULT 'en'`, `summary`, `model`.

### Правила, дедлайны, фокус-блоки (миграция 0046)

- **`deadline_events`** – дедлайны: `source_entity_type`, `source_entity_id`, `title`, `due_at`, `severity CHECK(IN ('low','medium','high','critical'))`, `status CHECK(IN ('active','completed','overdue','cancelled'))`, `linked_calendar_event_id`.
- **`focus_blocks`** – блоки концентрации: `title`, `start_at`/`end_at`, `purpose`, `linked_project_id`, `protection_level CHECK(IN ('low','medium','high','locked'))`, `status CHECK(IN ('scheduled','in_progress','completed','interrupted','cancelled'))`.
- **`calendar_rules`** – правила календаря: `rule_id TEXT PK`, `name TEXT NOT NULL`, `natural_language_description`, `compiled_dsl JSONB`, `enabled BOOLEAN DEFAULT true`, `approval_mode CHECK(IN ('suggest_only','ask_before_execute','auto_execute','dry_run'))`, `last_run_at`.

### Расширения календаря (миграция 0047)

К `calendar_events` добавлены колонки:

- `conference_url TEXT`
- `conference_provider TEXT`
- `preparation_reminder_minutes INTEGER`
- `travel_buffer_minutes INTEGER`

Созданы таблицы:

- **`calendar_reminders`** – напоминания: `reminder_type CHECK(IN ('time_based','context_based','preparation_based','location_based','deadline_based','document_based'))`, `minutes_before`, `condition_json JSONB`, `message`, `is_active`, `last_triggered_at`.
- **`event_locations`** – история локаций: `raw_location`, `parsed_name`, `parsed_address`, `is_online`, `latitude`, `longitude`, `frequency_count`.

## Задачи

### Расширение `tasks` (миграция 0048)

К существующей таблице `tasks` добавлены:

- `description TEXT`
- `priority_score REAL`
- `risk_score REAL`
- `readiness_score REAL`
- `source_type TEXT DEFAULT 'manual' CHECK(IN ('manual','email','telegram','whatsapp','calendar','meeting','document','note','jira','youtrack','github','gitlab','linear','todoist','apple_reminders','ms_todo','ai_rule','workflow','import'))`
- `area TEXT`
- `why TEXT`
- `outcome TEXT`
- `due_at TIMESTAMPTZ`
- `completed_at TIMESTAMPTZ`
- `archived_at TIMESTAMPTZ`
- `makosh_status TEXT DEFAULT 'new' CHECK(IN ('new','triaged','ready','in_progress','waiting','blocked','review','done','cancelled','archived'))`
- `waiting_reason TEXT`
- `energy_type TEXT`
- `confidentiality TEXT DEFAULT 'private_local' CHECK(IN ('public_to_provider','private_local','sensitive','confidential'))`
- `tags JSONB DEFAULT '[]'`
- `task_metadata JSONB DEFAULT '{}'`
- `linked_person_id TEXT`
- `linked_organization_id TEXT`

Индексы: по `makosh_status`, `due_at`, `priority_score`, `linked_person_id`, `linked_organization_id`.

### Провайдеры задач (миграция 0049)

- **`task_provider_accounts`** – учётные записи внешних систем задач: `provider CHECK(IN ('jira','youtrack','github','gitlab','linear','todoist','apple_reminders','ms_todo','trello','local'))`, `sync_mode CHECK(IN ('manual','read_only','two_way'))`, `capabilities JSONB`.
- **`external_task_identities`** – связь задач Макошь с внешними задачами: `task_id`, `provider`, `account_id`, `external_project_id`, `external_task_id`, `external_url`, `external_status`, `sync_status CHECK(IN ('local_only','syncing','synced','conflict','error'))`.
- **`provider_status_mappings`** – маппинг статусов провайдеров на статусы Макошь: `UNIQUE(provider, external_status)`.

### Контекст, доказательства, связи, чеклисты (миграция 0050)

- **`task_context_packs`** – контекстные пакеты: `summary`, `source_summary`, `open_questions JSONB`, `blockers JSONB`, `risks JSONB`, `suggested_next_action`, `generated_at`, `model`.
- **`task_evidence`** – доказательства/источники задачи: `source_type TEXT NOT NULL`, `source_id TEXT NOT NULL`, `quote TEXT`, `confidence REAL` (0..1).
- **`task_relations`** – связи задач: `relation_type CHECK(IN ('blocks','blocked_by','depends_on','relates_to','duplicates','caused_by','derived_from','follow_up_for','parent','subtask'))`, `entity_type`, `entity_id`.
- **`task_checklists`** – чеклисты: `items JSONB`.
- **`task_subtasks`** – подзадачи: `UNIQUE(parent_task_id, child_task_id)`, `sort_order INT`.
```

## Покрытие источников

| Исходный файл | Покрытые факты |
|---------------|----------------|
| `0026_create_email_rules.sql` | Структура таблицы `email_rules`, все колонки, ограничения, значения по умолчанию. |
| `0027_create_email_templates.sql` | Структура `email_templates`, колонки, ограничения, значения по умолчанию. |
| `0028_create_email_personas.sql` | Структура `email_personas`, колонки, ограничения, уникальный частичный индекс. |
| `0029_create_email_drafts.sql` | Структура `email_drafts`, колонки, ограничения, индекс `email_drafts_account_status_idx`. |
| `0030_create_email_invoices.sql` | Структура `email_invoices`, колонки, ограничения, индекс `email_invoices_status_idx`. |
| `0031_create_email_legal_documents.sql` | Структура `email_legal_documents`, колонки, ограничения. |
| `0032_create_email_certificates.sql` | Структура `email_certificates`, колонки, ограничения, индекс `email_certs_expiry_idx`. |
| `0033_extend_contacts.sql` | Добавление колонок в `contacts`, новые ограничения, индексы. |
| `0034_rename_contacts_to_persons.sql` | Переименование таблиц и колонок, замена префиксов ID, обновление `event_log` и `graph_nodes`, переименование ограничений и индексов. |
| `0035_person_identities_roles_personas.sql` | Расширение `persons` (новые колонки), создание `person_identities`, `person_roles`, `person_personas` с ограничениями и индексами, backfill email‑идентичностей. |
| `0036_person_memory_timeline.sql` | Создание `person_facts`, `person_memory_cards`, `person_preferences`, `person_snapshots`, `person_knowledge_conflicts`, `relationship_events`; добавление Communication DNA колонок в `persons`. |
| `0037_enrichment_expertise_trust.sql` | Создание `enrichment_results`, `person_expertise`, `person_promises`, `person_risks`; добавление health‑колонок в `persons`, индексы. |
| `0038_create_organizations.sql` | Полная структура `organizations`, все колонки, ограничения, индексы. |
| `0039_organization_identities_departments.sql` | Создание `organization_identities`, `organization_aliases`, `organization_domains`, `organization_departments`, `organization_contact_links`, `related_organizations` с ограничениями и индексами. |
| `0040_organization_memory.sql` | Создание `organization_facts`, `organization_memory_cards`, `organization_preferences`, `organization_required_documents`, `organization_snapshots`, `organization_knowledge_conflicts` и индексов. |
| `0041_organization_timeline_workflows.sql` | Создание `organization_timeline_events`, `organization_templates`, `organization_portals`, `organization_procedures`, `organization_playbooks`, `organization_quick_actions` с ограничениями и индексами. |
| `0042_organization_finance_enrichment.sql` | Создание `organization_financial_info`, `organization_contracts`, `organization_compliance`, `organization_services`, `organization_products`, `organization_enrichment_results` с ограничениями и индексами. |
| `0043_organization_risks_alerts.sql` | Создание `organization_risks`, `organization_alerts` с ограничениями и индексами. |
| `0044_calendar_core.sql` | Создание `calendar_accounts`, `calendar_sources`, `calendar_events` с полным набором колонок, ограничений, индексов. |
| `0045_calendar_core_tables.sql` | Создание `event_participants`, `event_relations`, `event_context_packs`, `event_agendas`, `event_checklists`, `meeting_notes`, `meeting_outcomes`, `event_recordings`, `event_transcripts` с ограничениями и индексами. |
| `0046_calendar_scheduling_rules.sql` | Создание `deadline_events`, `focus_blocks`, `calendar_rules` с ограничениями и индексами. |
| `0047_calendar_extensions.sql` | Расширение `calendar_events` колонками конференций/напоминаний; создание `calendar_reminders`, `event_locations` с ограничениями и индексами. |
| `0048_tasks_core.sql` | Добавление колонок в `tasks`, новые ограничения, индексы. |
| `0049_tasks_providers.sql` | Создание `task_provider_accounts`, `external_task_identities`, `provider_status_mappings` с ограничениями и индексами. |
| `0050_tasks_context.sql` | Создание `task_context_packs`, `task_evidence`, `task_relations`, `task_checklists`, `task_subtasks` с ограничениями и индексами. |

## Исходные файлы

- [`backend/migrations/0026_create_email_rules.sql`](../../../../backend/migrations/0026_create_email_rules.sql)
- [`backend/migrations/0027_create_email_templates.sql`](../../../../backend/migrations/0027_create_email_templates.sql)
- [`backend/migrations/0028_create_email_personas.sql`](../../../../backend/migrations/0028_create_email_personas.sql)
- [`backend/migrations/0029_create_email_drafts.sql`](../../../../backend/migrations/0029_create_email_drafts.sql)
- [`backend/migrations/0030_create_email_invoices.sql`](../../../../backend/migrations/0030_create_email_invoices.sql)
- [`backend/migrations/0031_create_email_legal_documents.sql`](../../../../backend/migrations/0031_create_email_legal_documents.sql)
- [`backend/migrations/0032_create_email_certificates.sql`](../../../../backend/migrations/0032_create_email_certificates.sql)
- [`backend/migrations/0033_extend_contacts.sql`](../../../../backend/migrations/0033_extend_contacts.sql)
- [`backend/migrations/0034_rename_contacts_to_persons.sql`](../../../../backend/migrations/0034_rename_contacts_to_persons.sql)
- [`backend/migrations/0035_person_identities_roles_personas.sql`](../../../../backend/migrations/0035_person_identities_roles_personas.sql)
- [`backend/migrations/0036_person_memory_timeline.sql`](../../../../backend/migrations/0036_person_memory_timeline.sql)
- [`backend/migrations/0037_enrichment_expertise_trust.sql`](../../../../backend/migrations/0037_enrichment_expertise_trust.sql)
- [`backend/migrations/0038_create_organizations.sql`](../../../../backend/migrations/0038_create_organizations.sql)
- [`backend/migrations/0039_organization_identities_departments.sql`](../../../../backend/migrations/0039_organization_identities_departments.sql)
- [`backend/migrations/0040_organization_memory.sql`](../../../../backend/migrations/0040_organization_memory.sql)
- [`backend/migrations/0041_organization_timeline_workflows.sql`](../../../../backend/migrations/0041_organization_timeline_workflows.sql)
- [`backend/migrations/0042_organization_finance_enrichment.sql`](../../../../backend/migrations/0042_organization_finance_enrichment.sql)
- [`backend/migrations/0043_organization_risks_alerts.sql`](../../../../backend/migrations/0043_organization_risks_alerts.sql)
- [`backend/migrations/0044_calendar_core.sql`](../../../../backend/migrations/0044_calendar_core.sql)
- [`backend/migrations/0045_calendar_core_tables.sql`](../../../../backend/migrations/0045_calendar_core_tables.sql)
- [`backend/migrations/0046_calendar_scheduling_rules.sql`](../../../../backend/migrations/0046_calendar_scheduling_rules.sql)
- [`backend/migrations/0047_calendar_extensions.sql`](../../../../backend/migrations/0047_calendar_extensions.sql)
- [`backend/migrations/0048_tasks_core.sql`](../../../../backend/migrations/0048_tasks_core.sql)
- [`backend/migrations/0049_tasks_providers.sql`](../../../../backend/migrations/0049_tasks_providers.sql)
- [`backend/migrations/0050_tasks_context.sql`](../../../../backend/migrations/0050_tasks_context.sql)

## Кандидаты на drift

В рамках предоставленного контекста (только SQL‑миграции) не выявлено явных противоречий между кодом и документацией. Однако следует отметить:

- Миграция `0033` расширяет таблицу `contacts`, а следующая `0034` немедленно переименовывает её в `persons`. При последовательном применении drift не возникает, но любые внешние ссылки (конфигурации, UI, wiki‑страницы) всё ещё могут использовать старое имя `contacts`. В текущем чанке нет файлов, позволяющих подтвердить или опровергнуть такое расхождение.
- Таблицы `communication_provider_accounts`, `contacts` (до переименования), `tasks` (до расширения), `event_log`, `graph_nodes` упоминаются в виде внешних ссылок (REFERENCES, UPDATE), но их определения не включены в чанк. Потенциальный drift в этих таблицах не может быть оценён.
- В остальных файлах структура и соглашения об именовании выдержаны единообразно, видимых расхождений нет.
