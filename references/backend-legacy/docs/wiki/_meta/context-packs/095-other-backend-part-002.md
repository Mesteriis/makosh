# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `095-other-backend-part-002`
- Group / Группа: `backend`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/migrations/0026_create_email_rules.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0026_create_email_rules.sql`
- Size bytes / Размер в байтах: `885`
- Included characters / Включено символов: `885`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description_nl TEXT NOT NULL DEFAULT '',
    conditions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    actions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    mode TEXT NOT NULL DEFAULT 'suggest',
    enabled BOOLEAN NOT NULL DEFAULT true,
    match_count BIGINT NOT NULL DEFAULT 0,
    last_matched_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_rules_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_rules_mode CHECK (mode IN ('suggest', 'ask_before_execute', 'auto_execute', 'dry_run')),
    CONSTRAINT email_rules_conditions_is_array CHECK (jsonb_typeof(conditions_json) = 'array'),
    CONSTRAINT email_rules_actions_is_array CHECK (jsonb_typeof(actions_json) = 'array')
);
```

### `backend/migrations/0027_create_email_templates.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0027_create_email_templates.sql`
- Size bytes / Размер в байтах: `621`
- Included characters / Включено символов: `621`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL DEFAULT '',
    variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    language TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_templates_subject_not_empty CHECK (length(trim(subject_template)) > 0),
    CONSTRAINT email_templates_variables_is_array CHECK (jsonb_typeof(variables) = 'array')
);
```

### `backend/migrations/0028_create_email_personas.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0028_create_email_personas.sql`
- Size bytes / Размер в байтах: `829`
- Included characters / Включено символов: `829`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_personas (
    persona_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    signature TEXT NOT NULL DEFAULT '',
    default_language TEXT,
    default_tone TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_personas_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_personas_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
CREATE UNIQUE INDEX IF NOT EXISTS email_personas_one_default_per_account
    ON email_personas (account_id) WHERE is_default = true;
```

### `backend/migrations/0029_create_email_drafts.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0029_create_email_drafts.sql`
- Size bytes / Размер в байтах: `1304`
- Included characters / Включено символов: `1304`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_drafts (
    draft_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    persona_id TEXT,
    to_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    cc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    bcc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    subject TEXT NOT NULL,
    body_text TEXT NOT NULL DEFAULT '',
    body_html TEXT,
    in_reply_to TEXT,
    message_references JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft',
    scheduled_send_at TIMESTAMPTZ,
    send_attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_drafts_subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT email_drafts_status CHECK (status IN ('draft', 'scheduled', 'sending', 'sent', 'failed')),
    CONSTRAINT email_drafts_to_is_array CHECK (jsonb_typeof(to_recipients) = 'array'),
    CONSTRAINT email_drafts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
CREATE INDEX IF NOT EXISTS email_drafts_account_status_idx ON email_drafts (account_id, status, updated_at DESC);
```

### `backend/migrations/0030_create_email_invoices.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0030_create_email_invoices.sql`
- Size bytes / Размер в байтах: `838`
- Included characters / Включено символов: `838`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_invoices (
    invoice_id TEXT PRIMARY KEY,
    message_id TEXT,
    amount DOUBLE PRECISION,
    currency TEXT,
    invoice_number TEXT,
    issue_date TIMESTAMPTZ,
    due_date TIMESTAMPTZ,
    counterparty TEXT,
    tax_id TEXT,
    status TEXT NOT NULL DEFAULT 'received',
    linked_project_id TEXT,
    linked_contact_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_invoices_status CHECK (status IN ('received','recognized','needs_review','approved','paid','closed','rejected')),
    CONSTRAINT email_invoices_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
CREATE INDEX IF NOT EXISTS email_invoices_status_idx ON email_invoices (status, due_date);
```

### `backend/migrations/0031_create_email_legal_documents.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0031_create_email_legal_documents.sql`
- Size bytes / Размер в байтах: `1092`
- Included characters / Включено символов: `1092`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_legal_documents (
    document_id TEXT PRIMARY KEY,
    message_id TEXT,
    document_type TEXT NOT NULL DEFAULT 'other',
    title TEXT NOT NULL,
    parties JSONB NOT NULL DEFAULT '[]'::jsonb,
    effective_date TIMESTAMPTZ,
    expiry_date TIMESTAMPTZ,
    amount DOUBLE PRECISION,
    currency TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    linked_project_id TEXT,
    risks JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_legal_docs_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT email_legal_docs_type CHECK (document_type IN ('contract','nda','msa','dpa','agreement','legal_notice','claim','court_document','tax_notice','government_doc','other')),
    CONSTRAINT email_legal_docs_status CHECK (status IN ('active','expired','pending_review','signed','terminated','draft')),
    CONSTRAINT email_legal_docs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
```

### `backend/migrations/0032_create_email_certificates.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0032_create_email_certificates.sql`
- Size bytes / Размер в байтах: `1439`
- Included characters / Включено символов: `1439`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_certificates (
    cert_id TEXT PRIMARY KEY, owner_name TEXT NOT NULL, issuer TEXT NOT NULL DEFAULT '',
    serial_number TEXT, fingerprint_sha256 TEXT,
    valid_from TIMESTAMPTZ, valid_until TIMESTAMPTZ,
    cert_type TEXT NOT NULL DEFAULT 'unknown',
    provider TEXT NOT NULL DEFAULT 'other',
    storage_kind TEXT NOT NULL DEFAULT 'encrypted_vault',
    storage_ref TEXT,
    trust_status TEXT NOT NULL DEFAULT 'untrusted',
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    usage JSONB NOT NULL DEFAULT '[]'::jsonb,
    linked_message_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT email_certs_type CHECK (cert_type IN ('smime','pgp','pdf_sign','cades','xades','gost_sign','unknown')),
    CONSTRAINT email_certs_provider CHECK (provider IN ('fnmt','dnie','cryptopro','gost','apple_keychain','pkcs12','yubikey','usb_token','other')),
    CONSTRAINT email_certs_storage CHECK (storage_kind IN ('os_keychain','encrypted_vault','pkcs12_file','pfx_file','smart_card','usb_token','external_vault')),
    CONSTRAINT email_certs_trust CHECK (trust_status IN ('trusted','untrusted','expired','revoked','pending_verification','self_signed'))
);
CREATE INDEX IF NOT EXISTS email_certs_expiry_idx ON email_certificates (valid_until) WHERE valid_until IS NOT NULL AND is_revoked = false;
```

### `backend/migrations/0033_extend_contacts.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0033_extend_contacts.sql`
- Size bytes / Размер в байтах: `1358`
- Included characters / Включено символов: `1358`
- Truncated / Обрезано: `no`

```text
ALTER TABLE contacts
    ADD COLUMN IF NOT EXISTS language TEXT,
    ADD COLUMN IF NOT EXISTS tone TEXT,
    ADD COLUMN IF NOT EXISTS trust_score SMALLINT,
    ADD COLUMN IF NOT EXISTS avg_response_hours DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS preferred_channel TEXT,
    ADD COLUMN IF NOT EXISTS last_interaction_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS interaction_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS frequent_topics JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS writing_style TEXT,
    ADD COLUMN IF NOT EXISTS contact_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS is_favorite BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS notes TEXT;

ALTER TABLE contacts
    ADD CONSTRAINT contacts_trust_score_range CHECK (trust_score IS NULL OR (trust_score >= 0 AND trust_score <= 100)),
    ADD CONSTRAINT contacts_contact_metadata_is_object CHECK (jsonb_typeof(contact_metadata) = 'object');

CREATE INDEX IF NOT EXISTS contacts_trust_score_idx ON contacts (trust_score DESC NULLS LAST) WHERE trust_score IS NOT NULL;
CREATE INDEX IF NOT EXISTS contacts_last_interaction_idx ON contacts (last_interaction_at DESC NULLS LAST) WHERE last_interaction_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS contacts_favorite_idx ON contacts (contact_id) WHERE is_favorite = true;
```

### `backend/migrations/0034_rename_contacts_to_persons.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0034_rename_contacts_to_persons.sql`
- Size bytes / Размер в байтах: `2197`
- Included characters / Включено символов: `2197`
- Truncated / Обрезано: `no`

```text
-- Phase 0: Rename contacts -> persons
-- Renames tables and updates ID values from contact:v1: to person:v1:

ALTER TABLE contacts RENAME TO persons;
ALTER TABLE contact_identity_candidates RENAME TO person_identity_candidates;

-- Rename columns in the renamed tables
ALTER TABLE persons RENAME COLUMN contact_id TO person_id;
ALTER TABLE persons RENAME COLUMN contact_metadata TO person_metadata;

ALTER TABLE person_identity_candidates RENAME COLUMN left_contact_id TO left_person_id;
ALTER TABLE person_identity_candidates RENAME COLUMN right_contact_id TO right_person_id;

-- Update ID values: contact:v1: -> person:v1:
UPDATE persons SET person_id = replace(person_id, 'contact:v1:', 'person:v1:');
UPDATE person_identity_candidates SET left_person_id = replace(left_person_id, 'contact:v1:', 'person:v1:');
UPDATE person_identity_candidates SET right_person_id = replace(right_person_id, 'contact:v1:', 'person:v1:');

-- Update event_log payloads
UPDATE event_log SET event_id = replace(event_id, 'contact_identity_review:', 'person_identity_review:') WHERE event_id LIKE 'contact_identity_review:%';
UPDATE event_log SET event_type = replace(event_type, 'contact_identity.', 'person_identity.') WHERE event_type LIKE 'contact_identity.%';

-- Update graph nodes
UPDATE graph_nodes SET node_id = replace(node_id, 'contact:v1:', 'person:v1:') WHERE node_id LIKE 'contact:v1:%';

-- Rename constraints
ALTER TABLE persons RENAME CONSTRAINT contacts_display_name_not_empty TO persons_display_name_not_empty;
ALTER TABLE persons RENAME CONSTRAINT contacts_email_not_empty TO persons_email_not_empty;
ALTER TABLE persons RENAME CONSTRAINT contacts_pkey TO persons_pkey;
ALTER TABLE persons RENAME CONSTRAINT contacts_trust_score_range TO persons_trust_score_range;
ALTER TABLE persons RENAME CONSTRAINT contacts_contact_metadata_is_object TO persons_person_metadata_is_object;

-- Rename indexes
ALTER INDEX contacts_email_address_key RENAME TO persons_email_address_key;
ALTER INDEX contacts_trust_score_idx RENAME TO persons_trust_score_idx;
ALTER INDEX contacts_last_interaction_idx RENAME TO persons_last_interaction_idx;
ALTER INDEX contacts_favorite_idx RENAME TO persons_favorite_idx;
```

### `backend/migrations/0035_person_identities_roles_personas.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0035_person_identities_roles_personas.sql`
- Size bytes / Размер в байтах: `3288`
- Included characters / Включено символов: `3288`
- Truncated / Обрезано: `no`

```text
-- Phase 1: Multi-channel person identity model

-- Extend persons table with type, role, org, timezone
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS person_type TEXT,
    ADD COLUMN IF NOT EXISTS primary_role TEXT,
    ADD COLUMN IF NOT EXISTS organization_reference TEXT,
    ADD COLUMN IF NOT EXISTS timezone TEXT;

-- Person identities: multi-channel identifiers
CREATE TABLE IF NOT EXISTS person_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    identity_type TEXT NOT NULL,
    identity_value TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_identities_type_check CHECK (identity_type IN (
        'email', 'telegram', 'whatsapp', 'phone',
        'github', 'linkedin', 'website',
        'mastodon', 'x', 'stackoverflow', 'habr',
        'medium', 'orcid', 'google_scholar'
    )),
    CONSTRAINT person_identities_status_check CHECK (status IN (
        'active', 'outdated', 'unreachable', 'blocked'
    )),
    CONSTRAINT person_identities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS person_identities_type_value_idx
    ON person_identities (identity_type, identity_value) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS person_identities_person_id_idx ON person_identities (person_id);

-- Person roles: many-to-many role assignments
CREATE TABLE IF NOT EXISTS person_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    assigned_by TEXT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_roles_unique UNIQUE (person_id, role)
);

CREATE INDEX IF NOT EXISTS person_roles_person_id_idx ON person_roles (person_id);
CREATE INDEX IF NOT EXISTS person_roles_role_idx ON person_roles (role);

-- Person personas: named interaction contexts
CREATE TABLE IF NOT EXISTS person_personas (
    persona_id TEXT PRIMARY KEY,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    context TEXT,
    default_tone TEXT,
    default_language TEXT,
    preferred_channel TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_personas_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT person_personas_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS person_personas_person_id_idx ON person_personas (person_id);

-- Backfill: create an email identity for each existing person
INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
SELECT person_id, 'email', email_address, 'import', 1.0, 'active'
FROM persons
WHERE email_address IS NOT NULL
ON CONFLICT (identity_type, identity_value) WHERE status = 'active' DO NOTHING;
```

### `backend/migrations/0036_person_memory_timeline.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0036_person_memory_timeline.sql`
- Size bytes / Размер в байтах: `5324`
- Included characters / Включено символов: `5324`
- Truncated / Обрезано: `no`

```text
-- Phases 2-3: Person memory and relationship timeline

-- Person facts: extracted facts with source and confidence
CREATE TABLE IF NOT EXISTS person_facts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    fact_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_facts_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS person_facts_person_id_idx ON person_facts (person_id);
CREATE INDEX IF NOT EXISTS person_facts_type_idx ON person_facts (fact_type);

-- Person memory cards: important things to remember
CREATE TABLE IF NOT EXISTS person_memory_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    importance SMALLINT NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_at TIMESTAMPTZ,

    CONSTRAINT person_memory_cards_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT person_memory_cards_importance_range CHECK (importance >= 1 AND importance <= 10),
    CONSTRAINT person_memory_cards_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS person_memory_cards_person_id_idx ON person_memory_cards (person_id);

-- Person preferences: communication preferences
CREATE TABLE IF NOT EXISTS person_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    preference_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_preferences_unique UNIQUE (person_id, preference_type),
    CONSTRAINT person_preferences_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS person_preferences_person_id_idx ON person_preferences (person_id);

-- Person snapshots: state at a point in time
CREATE TABLE IF NOT EXISTS person_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    snapshot_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_snapshots_data_is_object CHECK (jsonb_typeof(data) = 'object')
);

CREATE INDEX IF NOT EXISTS person_snapshots_person_id_idx ON person_snapshots (person_id);

-- Person knowledge conflicts: detected contradictions
CREATE TABLE IF NOT EXISTS person_knowledge_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    value_a TEXT NOT NULL,
    value_b TEXT NOT NULL,
    source_a TEXT NOT NULL,
    source_b TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);

CREATE INDEX IF NOT EXISTS person_knowledge_conflicts_person_id_idx
    ON person_knowledge_conflicts (person_id);

-- Relationship events: timeline of relationship events
CREATE TABLE IF NOT EXISTS relationship_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    occurred_at TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    related_entity_id TEXT,
    related_entity_kind TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT relationship_events_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT relationship_events_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT relationship_events_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS relationship_events_person_id_idx ON relationship_events (person_id);
CREATE INDEX IF NOT EXISTS relationship_events_type_idx ON relationship_events (event_type);
CREATE INDEX IF NOT EXISTS relationship_events_occurred_at_idx ON relationship_events (occurred_at);

-- Phase 4: Communication DNA columns on persons table
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS communication_style TEXT,
    ADD COLUMN IF NOT EXISTS verbosity TEXT,
    ADD COLUMN IF NOT EXISTS technical_depth TEXT,
    ADD COLUMN IF NOT EXISTS question_frequency TEXT,
    ADD COLUMN IF NOT EXISTS call_preference TEXT,
    ADD COLUMN IF NOT EXISTS response_pattern TEXT,
    ADD COLUMN IF NOT EXISTS active_hours JSONB,
    ADD COLUMN IF NOT EXISTS active_days JSONB;
```

### `backend/migrations/0037_enrichment_expertise_trust.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0037_enrichment_expertise_trust.sql`
- Size bytes / Размер в байтах: `4203`
- Included characters / Включено символов: `4203`
- Truncated / Обрезано: `no`

```text
-- Phases 5-7: Enrichment engine, expertise, trust and risk

-- Enrichment results: tracking enrichment attempts from external sources
CREATE TABLE IF NOT EXISTS enrichment_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    url TEXT,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence REAL NOT NULL DEFAULT 0.5,
    status TEXT NOT NULL DEFAULT 'pending',
    last_checked_at TIMESTAMPTZ,
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT enrichment_results_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT enrichment_results_status_check CHECK (status IN ('pending', 'applied', 'rejected', 'conflict')),
    CONSTRAINT enrichment_results_data_is_object CHECK (jsonb_typeof(data) = 'object')
);

CREATE INDEX IF NOT EXISTS enrichment_results_person_id_idx ON enrichment_results (person_id);
CREATE INDEX IF NOT EXISTS enrichment_results_status_idx ON enrichment_results (person_id, status);

-- Person expertise: skills and domains
CREATE TABLE IF NOT EXISTS person_expertise (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    skill TEXT NOT NULL,
    domain TEXT,
    evidence TEXT,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    last_verified_at TIMESTAMPTZ,
    endorsed_by_person_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_expertise_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT person_expertise_skill_not_empty CHECK (length(trim(skill)) > 0)
);

CREATE INDEX IF NOT EXISTS person_expertise_person_id_idx ON person_expertise (person_id);
CREATE INDEX IF NOT EXISTS person_expertise_skill_idx ON person_expertise (skill);

-- Person promises: tracked promises and commitments
CREATE TABLE IF NOT EXISTS person_promises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    source_message_id TEXT,
    promised_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    due_at TIMESTAMPTZ,
    fulfilled_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_promises_status_check CHECK (status IN ('pending', 'fulfilled', 'broken', 'forgiven')),
    CONSTRAINT person_promises_desc_not_empty CHECK (length(trim(description)) > 0)
);

CREATE INDEX IF NOT EXISTS person_promises_person_id_idx ON person_promises (person_id);
CREATE INDEX IF NOT EXISTS person_promises_status_idx ON person_promises (person_id, status);

-- Person risks: risk tracking
CREATE TABLE IF NOT EXISTS person_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    risk_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT,

    CONSTRAINT person_risks_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT person_risks_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS person_risks_person_id_idx ON person_risks (person_id);

-- Phase 8 prep: health columns on persons
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS health_status TEXT DEFAULT 'healthy',
    ADD COLUMN IF NOT EXISTS last_health_check TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS communication_gap_days INT DEFAULT 0,
    ADD COLUMN IF NOT EXISTS watchlist BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE persons
    ADD CONSTRAINT persons_health_status_check CHECK (health_status IN ('healthy', 'needs_attention', 'at_risk', 'dormant'));

CREATE INDEX IF NOT EXISTS persons_watchlist_idx ON persons (person_id) WHERE watchlist = true;
```

### `backend/migrations/0038_create_organizations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0038_create_organizations.sql`
- Size bytes / Размер в байтах: `2602`
- Included characters / Включено символов: `2602`
- Truncated / Обрезано: `no`

```text
-- Phase 0: Organizations core table

CREATE TABLE IF NOT EXISTS organizations (
    organization_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    legal_name TEXT,
    org_type TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    country TEXT,
    city TEXT,
    address TEXT,
    website TEXT,
    industry TEXT,
    description TEXT,
    primary_language TEXT,
    timezone TEXT,
    trust_score SMALLINT,
    health_status TEXT DEFAULT 'healthy',
    priority TEXT DEFAULT 'medium',
    notes TEXT,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    org_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_interaction_at TIMESTAMPTZ,
    interaction_count INT NOT NULL DEFAULT 0,
    -- Legal identity columns
    registration_number TEXT,
    country_of_registration TEXT,
    vat TEXT,
    cif TEXT,
    nif TEXT,
    tax_id TEXT,
    legal_address TEXT,
    registry_source TEXT,
    registry_last_verified TIMESTAMPTZ,
    -- DNA columns (populated in Phase 3)
    communication_style TEXT,
    verbosity TEXT,
    formality TEXT,
    secondary_languages JSONB,
    preferred_tone TEXT,
    official_style_required BOOL DEFAULT false,
    -- Health columns (populated in Phase 7)
    last_health_check TIMESTAMPTZ,
    watchlist BOOL NOT NULL DEFAULT false,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT organizations_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT organizations_status_check CHECK (status IN ('active', 'inactive', 'archived', 'watchlist', 'blocked', 'unknown')),
    CONSTRAINT organizations_priority_check CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT organizations_trust_score_range CHECK (trust_score IS NULL OR (trust_score >= 0 AND trust_score <= 100)),
    CONSTRAINT organizations_tags_is_array CHECK (jsonb_typeof(tags) = 'array'),
    CONSTRAINT organizations_metadata_is_object CHECK (jsonb_typeof(org_metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS organizations_type_idx ON organizations (org_type);
CREATE INDEX IF NOT EXISTS organizations_status_idx ON organizations (status);
CREATE INDEX IF NOT EXISTS organizations_vat_idx ON organizations (vat) WHERE vat IS NOT NULL;
CREATE INDEX IF NOT EXISTS organizations_domain_idx ON organizations (website);
CREATE INDEX IF NOT EXISTS organizations_watchlist_idx ON organizations (organization_id) WHERE watchlist = true;
CREATE INDEX IF NOT EXISTS organizations_trust_score_idx ON organizations (trust_score DESC NULLS LAST) WHERE trust_score IS NOT NULL;
```

### `backend/migrations/0039_organization_identities_departments.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0039_organization_identities_departments.sql`
- Size bytes / Размер в байтах: `5529`
- Included characters / Включено символов: `5529`
- Truncated / Обрезано: `no`

```text
-- Phase 1: Organization identities, aliases, departments, contacts, domains

CREATE TABLE IF NOT EXISTS organization_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    identity_type TEXT NOT NULL,
    identity_value TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_identities_type_check CHECK (identity_type IN (
        'domain', 'website', 'email_domain', 'support_email', 'billing_email', 'legal_email',
        'phone', 'vat', 'cif', 'nif', 'registry_number',
        'github_org', 'linkedin_page', 'twitter', 'mastodon',
        'support_portal', 'customer_portal', 'tax_portal', 'app_portal'
    )),
    CONSTRAINT org_identities_status_check CHECK (status IN ('active', 'outdated', 'unreachable', 'blocked')),
    CONSTRAINT org_identities_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_identities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS org_identities_type_value_idx ON organization_identities (identity_type, identity_value) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS org_identities_org_id_idx ON organization_identities (organization_id);

CREATE TABLE IF NOT EXISTS organization_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    alias_type TEXT DEFAULT 'trading',
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_aliases_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_aliases_type_check CHECK (alias_type IN ('legal', 'trading', 'brand', 'former'))
);

CREATE INDEX IF NOT EXISTS org_aliases_org_id_idx ON organization_aliases (organization_id);

CREATE TABLE IF NOT EXISTS organization_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    domain_type TEXT DEFAULT 'primary',
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_domains_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_domains_type_check CHECK (domain_type IN ('primary', 'additional', 'email', 'portal', 'former'))
);

CREATE INDEX IF NOT EXISTS org_domains_org_id_idx ON organization_domains (organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS org_domains_unique_active ON organization_domains (organization_id, domain) WHERE domain_type != 'former';

CREATE TABLE IF NOT EXISTS organization_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    parent_department_id UUID REFERENCES organization_departments(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_departments_name_not_empty CHECK (length(trim(name)) > 0)
);

CREATE INDEX IF NOT EXISTS org_departments_org_id_idx ON organization_departments (organization_id);

CREATE TABLE IF NOT EXISTS organization_contact_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    role TEXT,
    department TEXT,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    valid_from TIMESTAMPTZ DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_primary BOOL NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_contact_links_unique UNIQUE (organization_id, person_id, role),
    CONSTRAINT org_contact_links_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS org_contact_links_org_id_idx ON organization_contact_links (organization_id);
CREATE INDEX IF NOT EXISTS org_contact_links_person_id_idx ON organization_contact_links (person_id);

CREATE TABLE IF NOT EXISTS related_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    related_organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT related_orgs_type_check CHECK (relation_type IN ('parent', 'subsidiary', 'division', 'partner', 'supplier', 'customer')),
    CONSTRAINT related_orgs_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS related_orgs_org_id_idx ON related_organizations (organization_id);
```

### `backend/migrations/0040_organization_memory.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0040_organization_memory.sql`
- Size bytes / Размер в байтах: `3833`
- Included characters / Включено символов: `3833`
- Truncated / Обрезано: `no`

```text
-- Phase 2: Organization memory

CREATE TABLE IF NOT EXISTS organization_facts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    fact_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    is_active BOOL NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_facts_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_memory_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    importance SMALLINT NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_at TIMESTAMPTZ,
    CONSTRAINT org_memory_cards_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_memory_cards_importance_range CHECK (importance >= 1 AND importance <= 10)
);

CREATE TABLE IF NOT EXISTS organization_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    preference_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_preferences_unique UNIQUE (organization_id, preference_type),
    CONSTRAINT org_preferences_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_required_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    document_type TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_reqdocs_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    snapshot_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_snapshots_data_is_object CHECK (jsonb_typeof(data) = 'object')
);

CREATE TABLE IF NOT EXISTS organization_knowledge_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    value_a TEXT NOT NULL,
    value_b TEXT NOT NULL,
    source_a TEXT NOT NULL,
    source_b TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);

CREATE INDEX IF NOT EXISTS org_facts_org_id_idx ON organization_facts (organization_id);
CREATE INDEX IF NOT EXISTS org_memory_cards_org_id_idx ON organization_memory_cards (organization_id);
CREATE INDEX IF NOT EXISTS org_preferences_org_id_idx ON organization_preferences (organization_id);
CREATE INDEX IF NOT EXISTS org_snapshots_org_id_idx ON organization_snapshots (organization_id);
CREATE INDEX IF NOT EXISTS org_conflicts_org_id_idx ON organization_knowledge_conflicts (organization_id);
```

### `backend/migrations/0041_organization_timeline_workflows.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0041_organization_timeline_workflows.sql`
- Size bytes / Размер в байтах: `4032`
- Included characters / Включено символов: `4032`
- Truncated / Обрезано: `no`

```text
-- Phase 3-4: Timeline, templates, portals, procedures, playbooks

CREATE TABLE IF NOT EXISTS organization_timeline_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    occurred_at TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    related_entity_id TEXT,
    related_entity_kind TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_timeline_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_timeline_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE TABLE IF NOT EXISTS organization_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    template_type TEXT NOT NULL DEFAULT 'email',
    subject TEXT,
    body TEXT,
    language TEXT,
    tone TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_templates_type_check CHECK (template_type IN ('email', 'document'))
);

CREATE TABLE IF NOT EXISTS organization_portals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    portal_type TEXT NOT NULL DEFAULT 'customer',
    login_hint TEXT,
    secret_reference TEXT,
    last_used_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_portals_type_check CHECK (portal_type IN ('tax', 'customer', 'banking', 'support', 'billing', 'admin', 'app'))
);

CREATE TABLE IF NOT EXISTS organization_procedures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_procedures_steps_is_array CHECK (jsonb_typeof(steps) = 'array')
);

CREATE TABLE IF NOT EXISTS organization_playbooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_condition TEXT,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    approval_mode TEXT NOT NULL DEFAULT 'confirm',
    enabled BOOL NOT NULL DEFAULT false,
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_playbooks_approval_check CHECK (approval_mode IN ('auto', 'confirm', 'disabled')),
    CONSTRAINT org_playbooks_steps_is_array CHECK (jsonb_typeof(steps) = 'array')
);

CREATE TABLE IF NOT EXISTS organization_quick_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    action_type TEXT NOT NULL,
    action_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS org_timeline_org_id_idx ON organization_timeline_events (organization_id);
CREATE INDEX IF NOT EXISTS org_portals_org_id_idx ON organization_portals (organization_id);
CREATE INDEX IF NOT EXISTS org_procedures_org_id_idx ON organization_procedures (organization_id);
CREATE INDEX IF NOT EXISTS org_playbooks_org_id_idx ON organization_playbooks (organization_id);
```

### `backend/migrations/0042_organization_finance_enrichment.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0042_organization_finance_enrichment.sql`
- Size bytes / Размер в байтах: `3572`
- Included characters / Включено символов: `3572`
- Truncated / Обрезано: `no`

```text
-- Phase 5-6: Finance, contracts, compliance, services, products, enrichment

CREATE TABLE IF NOT EXISTS organization_financial_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE UNIQUE,
    bank_name TEXT,
    iban_masked TEXT,
    bic TEXT,
    payment_terms TEXT,
    currency TEXT DEFAULT 'EUR',
    billing_email TEXT,
    billing_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organization_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    contract_type TEXT NOT NULL,
    title TEXT NOT NULL,
    signed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    document_reference TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_contracts_status_check CHECK (status IN ('draft', 'active', 'expired', 'terminated', 'renewed'))
);

CREATE TABLE IF NOT EXISTS organization_compliance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    compliance_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    document_reference TEXT,
    expires_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_compliance_status_check CHECK (status IN ('compliant', 'pending', 'expired', 'not_applicable'))
);

CREATE TABLE IF NOT EXISTS organization_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    service_name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organization_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    product_name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organization_enrichment_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    url TEXT,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence REAL NOT NULL DEFAULT 0.5,
    status TEXT NOT NULL DEFAULT 'pending',
    last_checked_at TIMESTAMPTZ,
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_enrichment_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_enrichment_status_check CHECK (status IN ('pending', 'applied', 'rejected', 'conflict'))
);

CREATE INDEX IF NOT EXISTS org_contracts_org_id_idx ON organization_contracts (organization_id);
CREATE INDEX IF NOT EXISTS org_compliance_org_id_idx ON organization_compliance (organization_id);
CREATE INDEX IF NOT EXISTS org_enrichment_org_id_idx ON organization_enrichment_results (organization_id);
```

### `backend/migrations/0043_organization_risks_alerts.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0043_organization_risks_alerts.sql`
- Size bytes / Размер в байтах: `1371`
- Included characters / Включено символов: `1371`
- Truncated / Обрезано: `no`

```text
-- Phase 7: Risks and alerts

CREATE TABLE IF NOT EXISTS organization_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    risk_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT,
    CONSTRAINT org_risks_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT org_risks_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    severity TEXT NOT NULL DEFAULT 'medium',
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_alerts_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX IF NOT EXISTS org_risks_org_id_idx ON organization_risks (organization_id);
CREATE INDEX IF NOT EXISTS org_alerts_org_id_idx ON organization_alerts (organization_id);
```

### `backend/migrations/0044_calendar_core.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0044_calendar_core.sql`
- Size bytes / Размер в байтах: `3933`
- Included characters / Включено символов: `3933`
- Truncated / Обрезано: `no`

```text
-- Phase 0: Calendar core tables

CREATE TABLE IF NOT EXISTS calendar_accounts (
    account_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    account_name TEXT NOT NULL,
    email TEXT,
    credentials_reference TEXT,
    sync_status TEXT NOT NULL DEFAULT 'idle',
    capabilities JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_accounts_provider_check CHECK (provider IN ('google', 'microsoft', 'exchange', 'apple', 'caldav', 'ics', 'local')),
    CONSTRAINT calendar_accounts_sync_status_check CHECK (sync_status IN ('idle', 'syncing', 'synced', 'error', 'disabled')),
    CONSTRAINT calendar_accounts_caps_is_object CHECK (jsonb_typeof(capabilities) = 'object')
);

CREATE INDEX IF NOT EXISTS calendar_accounts_provider_idx ON calendar_accounts (provider);

CREATE TABLE IF NOT EXISTS calendar_sources (
    source_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES calendar_accounts(account_id) ON DELETE CASCADE,
    provider_calendar_id TEXT,
    name TEXT NOT NULL,
    color TEXT,
    timezone TEXT,
    visibility TEXT NOT NULL DEFAULT 'private',
    read_only BOOLEAN NOT NULL DEFAULT false,
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    capabilities JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_sources_visibility_check CHECK (visibility IN ('private', 'public', 'confidential')),
    CONSTRAINT calendar_sources_caps_is_object CHECK (jsonb_typeof(capabilities) = 'object')
);

CREATE INDEX IF NOT EXISTS calendar_sources_account_idx ON calendar_sources (account_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    event_id TEXT PRIMARY KEY,
    source_event_id TEXT,
    account_id TEXT REFERENCES calendar_accounts(account_id) ON DELETE SET NULL,
    source_id TEXT REFERENCES calendar_sources(source_id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT,
    location TEXT,
    start_at TIMESTAMPTZ NOT NULL,
    end_at TIMESTAMPTZ NOT NULL,
    timezone TEXT,
    all_day BOOLEAN NOT NULL DEFAULT false,
    recurrence_rule TEXT,
    status TEXT NOT NULL DEFAULT 'scheduled',
    visibility TEXT NOT NULL DEFAULT 'private',
    event_type TEXT,
    importance_score REAL,
    readiness_score REAL,
    sync_status TEXT NOT NULL DEFAULT 'local',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_events_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT calendar_events_status_check CHECK (status IN ('scheduled', 'prepared', 'in_progress', 'completed', 'cancelled', 'rescheduled', 'no_show', 'needs_follow_up', 'archived')),
    CONSTRAINT calendar_events_visibility_check CHECK (visibility IN ('private', 'public', 'confidential', 'hidden_details', 'local_only')),
    CONSTRAINT calendar_events_sync_status_check CHECK (sync_status IN ('local', 'syncing', 'synced', 'conflict', 'error')),
    CONSTRAINT calendar_events_importance_range CHECK (importance_score IS NULL OR (importance_score >= 0 AND importance_score <= 1)),
    CONSTRAINT calendar_events_readiness_range CHECK (readiness_score IS NULL OR (readiness_score >= 0 AND readiness_score <= 1))
);

CREATE INDEX IF NOT EXISTS calendar_events_account_idx ON calendar_events (account_id);
CREATE INDEX IF NOT EXISTS calendar_events_source_idx ON calendar_events (source_id);
CREATE INDEX IF NOT EXISTS calendar_events_start_at_idx ON calendar_events (start_at);
CREATE INDEX IF NOT EXISTS calendar_events_end_at_idx ON calendar_events (end_at);
CREATE INDEX IF NOT EXISTS calendar_events_status_idx ON calendar_events (status);
CREATE INDEX IF NOT EXISTS calendar_events_type_idx ON calendar_events (event_type);
CREATE INDEX IF NOT EXISTS calendar_events_time_range_idx ON calendar_events (start_at, end_at);
```

### `backend/migrations/0045_calendar_core_tables.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0045_calendar_core_tables.sql`
- Size bytes / Размер в байтах: `6725`
- Included characters / Включено символов: `6725`
- Truncated / Обрезано: `no`

```text
-- Phase 1: Event participants, relations, context, agendas, checklists
-- Phase 3: Meeting notes, outcomes, recordings, transcripts in 0046 below

CREATE TABLE IF NOT EXISTS event_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    person_id TEXT,
    email TEXT NOT NULL,
    display_name TEXT,
    role TEXT DEFAULT 'attendee',
    response_status TEXT DEFAULT 'needs_action',
    organization_id TEXT,
    timezone TEXT,
    confidence REAL DEFAULT 0.7,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_participants_role_check CHECK (role IN ('organizer', 'required', 'optional', 'attendee', 'speaker')),
    CONSTRAINT event_participants_response_check CHECK (response_status IN ('needs_action', 'accepted', 'declined', 'tentative', 'no_response'))
);

CREATE INDEX IF NOT EXISTS event_participants_event_idx ON event_participants (event_id);
CREATE INDEX IF NOT EXISTS event_participants_person_idx ON event_participants (person_id);

CREATE TABLE IF NOT EXISTS event_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    source TEXT DEFAULT 'manual',
    confidence REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_relations_entity_type_check CHECK (entity_type IN ('person', 'organization', 'project', 'document', 'task', 'email', 'note', 'decision', 'obligation', 'recording'))
);

CREATE INDEX IF NOT EXISTS event_relations_event_idx ON event_relations (event_id);
CREATE INDEX IF NOT EXISTS event_relations_entity_idx ON event_relations (entity_type, entity_id);

CREATE TABLE IF NOT EXISTS event_context_packs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    summary TEXT,
    participants_summary TEXT,
    documents JSONB NOT NULL DEFAULT '[]',
    tasks JSONB NOT NULL DEFAULT '[]',
    open_questions JSONB NOT NULL DEFAULT '[]',
    risks JSONB NOT NULL DEFAULT '[]',
    suggested_agenda JSONB NOT NULL DEFAULT '[]',
    suggested_actions JSONB NOT NULL DEFAULT '[]',
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_context_packs_docs_is_array CHECK (jsonb_typeof(documents) = 'array'),
    CONSTRAINT event_context_packs_tasks_is_array CHECK (jsonb_typeof(tasks) = 'array'),
    CONSTRAINT event_context_packs_questions_is_array CHECK (jsonb_typeof(open_questions) = 'array'),
    CONSTRAINT event_context_packs_risks_is_array CHECK (jsonb_typeof(risks) = 'array'),
    CONSTRAINT event_context_packs_agenda_is_array CHECK (jsonb_typeof(suggested_agenda) = 'array'),
    CONSTRAINT event_context_packs_actions_is_array CHECK (jsonb_typeof(suggested_actions) = 'array')
);

CREATE INDEX IF NOT EXISTS event_context_packs_event_idx ON event_context_packs (event_id);

CREATE TABLE IF NOT EXISTS event_agendas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    items JSONB NOT NULL DEFAULT '[]',
    source TEXT DEFAULT 'manual',
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_agendas_items_is_array CHECK (jsonb_typeof(items) = 'array')
);

CREATE INDEX IF NOT EXISTS event_agendas_event_idx ON event_agendas (event_id);

CREATE TABLE IF NOT EXISTS event_checklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    items JSONB NOT NULL DEFAULT '[]',
    source TEXT DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_checklists_items_is_array CHECK (jsonb_typeof(items) = 'array')
);

CREATE INDEX IF NOT EXISTS event_checklists_event_idx ON event_checklists (event_id);

-- Phase 3 tables (in same migration for simplicity)

CREATE TABLE IF NOT EXISTS meeting_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    format TEXT DEFAULT 'markdown',
    source TEXT DEFAULT 'manual',
    linked_note_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS meeting_notes_event_idx ON meeting_notes (event_id);

CREATE TABLE IF NOT EXISTS meeting_outcomes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    outcome_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    owner_person_id TEXT,
    due_date TIMESTAMPTZ,
    source TEXT DEFAULT 'manual',
    confidence REAL DEFAULT 1.0,
    linked_entity_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT meeting_outcomes_type_check CHECK (outcome_type IN ('decision', 'task', 'promise', 'risk', 'question', 'document_request', 'follow_up', 'agreement', 'blocker'))
);

CREATE INDEX IF NOT EXISTS meeting_outcomes_event_idx ON meeting_outcomes (event_id);
CREATE INDEX IF NOT EXISTS meeting_outcomes_owner_idx ON meeting_outcomes (owner_person_id);

CREATE TABLE IF NOT EXISTS event_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    file_path TEXT,
    source TEXT DEFAULT 'manual',
    duration_seconds INTEGER,
    transcript_id UUID,
    processing_status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_recordings_status_check CHECK (processing_status IN ('pending', 'transcribing', 'transcribed', 'failed'))
);

CREATE INDEX IF NOT EXISTS event_recordings_event_idx ON event_recordings (event_id);

CREATE TABLE IF NOT EXISTS event_transcripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    language TEXT DEFAULT 'en',
    summary TEXT,
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS event_transcripts_event_idx ON event_transcripts (event_id);
```

### `backend/migrations/0046_calendar_scheduling_rules.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0046_calendar_scheduling_rules.sql`
- Size bytes / Размер в байтах: `2362`
- Included characters / Включено символов: `2362`
- Truncated / Обрезано: `no`

```text
-- Phase 4: Deadlines and focus blocks
-- Phase 7: Calendar rules

CREATE TABLE IF NOT EXISTS deadline_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_entity_type TEXT,
    source_entity_id TEXT,
    title TEXT NOT NULL,
    due_at TIMESTAMPTZ NOT NULL,
    severity TEXT DEFAULT 'medium',
    status TEXT DEFAULT 'active',
    linked_calendar_event_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT deadline_events_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT deadline_events_status_check CHECK (status IN ('active', 'completed', 'overdue', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS deadline_events_due_idx ON deadline_events (due_at);
CREATE INDEX IF NOT EXISTS deadline_events_status_idx ON deadline_events (status);

CREATE TABLE IF NOT EXISTS focus_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    start_at TIMESTAMPTZ NOT NULL,
    end_at TIMESTAMPTZ NOT NULL,
    purpose TEXT,
    linked_project_id TEXT,
    protection_level TEXT DEFAULT 'medium',
    status TEXT DEFAULT 'scheduled',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT focus_blocks_protection_check CHECK (protection_level IN ('low', 'medium', 'high', 'locked')),
    CONSTRAINT focus_blocks_status_check CHECK (status IN ('scheduled', 'in_progress', 'completed', 'interrupted', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS focus_blocks_time_idx ON focus_blocks (start_at, end_at);

CREATE TABLE IF NOT EXISTS calendar_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    natural_language_description TEXT,
    compiled_dsl JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    approval_mode TEXT NOT NULL DEFAULT 'suggest_only',
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_rules_approval_check CHECK (approval_mode IN ('suggest_only', 'ask_before_execute', 'auto_execute', 'dry_run')),
    CONSTRAINT calendar_rules_dsl_is_object CHECK (jsonb_typeof(compiled_dsl) = 'object')
);

CREATE INDEX IF NOT EXISTS calendar_rules_enabled_idx ON calendar_rules (rule_id) WHERE enabled = true;
```

### `backend/migrations/0047_calendar_extensions.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0047_calendar_extensions.sql`
- Size bytes / Размер в байтах: `1967`
- Included characters / Включено символов: `1967`
- Truncated / Обрезано: `no`

```text
-- Phase e1: Conference links, reminders, event metadata extensions

ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS conference_url TEXT;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS conference_provider TEXT;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS preparation_reminder_minutes INTEGER;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS travel_buffer_minutes INTEGER;

-- Smart reminders
CREATE TABLE IF NOT EXISTS calendar_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    reminder_type TEXT NOT NULL DEFAULT 'time_based',
    minutes_before INTEGER,
    condition_json JSONB,
    message TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_reminders_type_check CHECK (reminder_type IN ('time_based', 'context_based', 'preparation_based', 'location_based', 'deadline_based', 'document_based'))
);

CREATE INDEX IF NOT EXISTS calendar_reminders_event_idx ON calendar_reminders (event_id);
CREATE INDEX IF NOT EXISTS calendar_reminders_active_idx ON calendar_reminders (event_id) WHERE is_active = true;

-- Event location history for location intelligence
CREATE TABLE IF NOT EXISTS event_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    raw_location TEXT NOT NULL,
    parsed_name TEXT,
    parsed_address TEXT,
    is_online BOOLEAN DEFAULT false,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    frequency_count INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS event_locations_event_idx ON event_locations (event_id);
CREATE INDEX IF NOT EXISTS event_locations_name_idx ON event_locations (parsed_name);
```

### `backend/migrations/0048_tasks_core.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0048_tasks_core.sql`
- Size bytes / Размер в байтах: `2410`
- Included characters / Включено символов: `2410`
- Truncated / Обрезано: `no`

```text
-- Phase 0: Extend tasks table with full domain model

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS priority_score REAL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS risk_score REAL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS readiness_score REAL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS source_type TEXT DEFAULT 'manual';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS area TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS why TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS outcome TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS due_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS makosh_status TEXT DEFAULT 'new';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS waiting_reason TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS energy_type TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS confidentiality TEXT DEFAULT 'private_local';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS tags JSONB DEFAULT '[]';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS task_metadata JSONB DEFAULT '{}';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS linked_person_id TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS linked_organization_id TEXT;

-- Drop old constraint, add new
ALTER TABLE tasks DROP CONSTRAINT IF EXISTS tasks_status_check;
ALTER TABLE tasks ADD CONSTRAINT tasks_makosh_status_check CHECK (makosh_status IN ('new','triaged','ready','in_progress','waiting','blocked','review','done','cancelled','archived'));
ALTER TABLE tasks ADD CONSTRAINT tasks_source_type_check CHECK (source_type IN ('manual','email','telegram','whatsapp','calendar','meeting','document','note','jira','youtrack','github','gitlab','linear','todoist','apple_reminders','ms_todo','ai_rule','workflow','import'));
ALTER TABLE tasks ADD CONSTRAINT tasks_confidentiality_check CHECK (confidentiality IN ('public_to_provider','private_local','sensitive','confidential'));

CREATE INDEX IF NOT EXISTS tasks_makosh_status_idx ON tasks (makosh_status);
CREATE INDEX IF NOT EXISTS tasks_due_at_idx ON tasks (due_at);
CREATE INDEX IF NOT EXISTS tasks_priority_idx ON tasks (priority_score DESC NULLS LAST);
CREATE INDEX IF NOT EXISTS tasks_person_idx ON tasks (linked_person_id);
CREATE INDEX IF NOT EXISTS tasks_org_idx ON tasks (linked_organization_id);
```

### `backend/migrations/0049_tasks_providers.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0049_tasks_providers.sql`
- Size bytes / Размер в байтах: `1883`
- Included characters / Включено символов: `1883`
- Truncated / Обрезано: `no`

```text
-- Phase 1: Provider accounts, external identities, status mappings

CREATE TABLE IF NOT EXISTS task_provider_accounts (
    account_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    account_name TEXT NOT NULL,
    credentials_reference TEXT,
    sync_mode TEXT DEFAULT 'manual',
    capabilities JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_providers_provider_check CHECK (provider IN ('jira','youtrack','github','gitlab','linear','todoist','apple_reminders','ms_todo','trello','local')),
    CONSTRAINT task_providers_sync_check CHECK (sync_mode IN ('manual','read_only','two_way'))
);

CREATE TABLE IF NOT EXISTS external_task_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    account_id TEXT,
    external_project_id TEXT,
    external_task_id TEXT,
    external_url TEXT,
    external_status TEXT,
    sync_status TEXT DEFAULT 'synced',
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ext_task_sync_status_check CHECK (sync_status IN ('local_only','syncing','synced','conflict','error'))
);

CREATE INDEX IF NOT EXISTS ext_task_identities_task_idx ON external_task_identities (task_id);
CREATE UNIQUE INDEX IF NOT EXISTS ext_task_identities_unique_idx ON external_task_identities (provider, account_id, external_task_id) WHERE external_task_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS provider_status_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,
    external_status TEXT NOT NULL,
    makosh_status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(provider, external_status)
);
```

### `backend/migrations/0050_tasks_context.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0050_tasks_context.sql`
- Size bytes / Размер в байтах: `2748`
- Included characters / Включено символов: `2748`
- Truncated / Обрезано: `no`

```text
-- Phase 2: Context packs, evidence, relations, checklists, subtasks

CREATE TABLE IF NOT EXISTS task_context_packs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    summary TEXT,
    source_summary TEXT,
    open_questions JSONB NOT NULL DEFAULT '[]',
    blockers JSONB NOT NULL DEFAULT '[]',
    risks JSONB NOT NULL DEFAULT '[]',
    suggested_next_action TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS task_context_packs_task_idx ON task_context_packs (task_id);

CREATE TABLE IF NOT EXISTS task_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    quote TEXT,
    confidence REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_evidence_confidence_check CHECK (confidence >= 0.0 AND confidence <= 1.0)
);
CREATE INDEX IF NOT EXISTS task_evidence_task_idx ON task_evidence (task_id);

CREATE TABLE IF NOT EXISTS task_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    source TEXT DEFAULT 'manual',
    confidence REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_relations_type_check CHECK (relation_type IN ('blocks','blocked_by','depends_on','relates_to','duplicates','caused_by','derived_from','follow_up_for','parent','subtask'))
);
CREATE INDEX IF NOT EXISTS task_relations_task_idx ON task_relations (task_id);

CREATE TABLE IF NOT EXISTS task_checklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    items JSONB NOT NULL DEFAULT '[]',
    source TEXT DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS task_checklists_task_idx ON task_checklists (task_id);

CREATE TABLE IF NOT EXISTS task_subtasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    child_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    sort_order INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(parent_task_id, child_task_id)
);
CREATE INDEX IF NOT EXISTS task_subtasks_parent_idx ON task_subtasks (parent_task_id);
```
