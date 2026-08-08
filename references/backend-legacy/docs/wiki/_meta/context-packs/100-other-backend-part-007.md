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

- Chunk ID / ID чанка: `100-other-backend-part-007`
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

### `backend/migrations/0151_create_communication_read_receipts.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0151_create_communication_read_receipts.sql`
- Size bytes / Размер в байтах: `2663`
- Included characters / Включено символов: `2663`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS communication_read_receipts (
    receipt_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    outbox_id TEXT REFERENCES communication_outbox(outbox_id) ON DELETE SET NULL,
    provider_message_id TEXT NOT NULL,
    recipient TEXT NOT NULL,
    receipt_kind TEXT NOT NULL DEFAULT 'read',
    read_at TIMESTAMPTZ NOT NULL,
    source_kind TEXT NOT NULL DEFAULT 'mdn',
    provider_record_id TEXT,
    raw_record_id TEXT REFERENCES communication_raw_records(raw_record_id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_read_receipts_id_not_empty CHECK (length(trim(receipt_id)) > 0),
    CONSTRAINT communication_read_receipts_provider_message_not_empty CHECK (
        length(trim(provider_message_id)) > 0
    ),
    CONSTRAINT communication_read_receipts_recipient_not_empty CHECK (length(trim(recipient)) > 0),
    CONSTRAINT communication_read_receipts_kind CHECK (receipt_kind IN ('read')),
    CONSTRAINT communication_read_receipts_source_kind_not_empty CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT communication_read_receipts_provider_record_not_empty CHECK (
        provider_record_id IS NULL OR length(trim(provider_record_id)) > 0
    ),
    CONSTRAINT communication_read_receipts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS communication_read_receipts_provider_record_unique_idx
    ON communication_read_receipts (account_id, provider_record_id)
    WHERE provider_record_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS communication_read_receipts_outbox_read_at_idx
    ON communication_read_receipts (outbox_id, read_at DESC, receipt_id);

CREATE INDEX IF NOT EXISTS communication_read_receipts_provider_message_idx
    ON communication_read_receipts (account_id, provider_message_id, read_at DESC);

INSERT INTO communication_read_receipts (
    receipt_id,
    account_id,
    outbox_id,
    provider_message_id,
    recipient,
    receipt_kind,
    read_at,
    source_kind,
    provider_record_id,
    raw_record_id,
    metadata,
    created_at
)
SELECT
    receipt.receipt_id,
    receipt.account_id,
    receipt.outbox_id,
    receipt.provider_message_id,
    receipt.recipient,
    receipt.receipt_kind,
    receipt.read_at,
    receipt.source_kind,
    receipt.provider_record_id,
    receipt.raw_record_id,
    receipt.metadata || jsonb_build_object('source_table', 'mail_read_receipts'),
    receipt.created_at
FROM mail_read_receipts receipt
ON CONFLICT (receipt_id) DO NOTHING;
```

### `backend/migrations/0152_create_canonical_communication_aux_tables.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0152_create_canonical_communication_aux_tables.sql`
- Size bytes / Размер в байтах: `10104`
- Included characters / Включено символов: `10104`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS communication_rules (
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

    CONSTRAINT communication_rules_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_rules_mode CHECK (
        mode IN ('suggest', 'ask_before_execute', 'auto_execute', 'dry_run')
    ),
    CONSTRAINT communication_rules_conditions_is_array CHECK (jsonb_typeof(conditions_json) = 'array'),
    CONSTRAINT communication_rules_actions_is_array CHECK (jsonb_typeof(actions_json) = 'array')
);

CREATE TABLE IF NOT EXISTS communication_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL DEFAULT '',
    variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    language TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_templates_subject_not_empty CHECK (length(trim(subject_template)) > 0),
    CONSTRAINT communication_templates_variables_is_array CHECK (jsonb_typeof(variables) = 'array')
);

CREATE TABLE IF NOT EXISTS communication_personas (
    persona_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    signature TEXT NOT NULL DEFAULT '',
    default_language TEXT,
    default_tone TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_personas_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_personas_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS communication_personas_one_default_per_account
    ON communication_personas (account_id)
    WHERE is_default = true;

CREATE TABLE IF NOT EXISTS communication_invoices (
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
    linked_person_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_invoices_status CHECK (
        status IN ('received', 'recognized', 'needs_review', 'approved', 'paid', 'closed', 'rejected')
    ),
    CONSTRAINT communication_invoices_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_invoices_status_idx
    ON communication_invoices (status, due_date);

CREATE INDEX IF NOT EXISTS communication_invoices_linked_person_idx
    ON communication_invoices (linked_person_id);

CREATE TABLE IF NOT EXISTS communication_legal_documents (
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

    CONSTRAINT communication_legal_docs_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT communication_legal_docs_type CHECK (
        document_type IN (
            'contract', 'nda', 'msa', 'dpa', 'agreement', 'legal_notice',
            'claim', 'court_document', 'tax_notice', 'government_doc', 'other'
        )
    ),
    CONSTRAINT communication_legal_docs_status CHECK (
        status IN ('active', 'expired', 'pending_review', 'signed', 'terminated', 'draft')
    ),
    CONSTRAINT communication_legal_docs_parties_is_array CHECK (jsonb_typeof(parties) = 'array'),
    CONSTRAINT communication_legal_docs_risks_is_array CHECK (jsonb_typeof(risks) = 'array'),
    CONSTRAINT communication_legal_docs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_certificates (
    cert_id TEXT PRIMARY KEY,
    owner_name TEXT NOT NULL,
    issuer TEXT NOT NULL DEFAULT '',
    serial_number TEXT,
    fingerprint_sha256 TEXT,
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,
    cert_type TEXT NOT NULL DEFAULT 'unknown',
    provider TEXT NOT NULL DEFAULT 'other',
    storage_kind TEXT NOT NULL DEFAULT 'encrypted_vault',
    storage_ref TEXT,
    trust_status TEXT NOT NULL DEFAULT 'untrusted',
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    usage JSONB NOT NULL DEFAULT '[]'::jsonb,
    linked_message_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_certs_type CHECK (
        cert_type IN ('smime', 'pgp', 'pdf_sign', 'cades', 'xades', 'gost_sign', 'unknown')
    ),
    CONSTRAINT communication_certs_provider CHECK (
        provider IN ('fnmt', 'dnie', 'cryptopro', 'gost', 'apple_keychain', 'pkcs12', 'yubikey', 'usb_token', 'other')
    ),
    CONSTRAINT communication_certs_storage CHECK (
        storage_kind IN ('os_keychain', 'encrypted_vault', 'pkcs12_file', 'pfx_file', 'smart_card', 'usb_token', 'external_vault')
    ),
    CONSTRAINT communication_certs_trust CHECK (
        trust_status IN ('trusted', 'untrusted', 'expired', 'revoked', 'pending_verification', 'self_signed')
    ),
    CONSTRAINT communication_certs_usage_is_array CHECK (jsonb_typeof(usage) = 'array'),
    CONSTRAINT communication_certs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_certs_expiry_idx
    ON communication_certificates (valid_until)
    WHERE valid_until IS NOT NULL AND is_revoked = false;

INSERT INTO communication_rules (
    rule_id, name, description_nl, conditions_json, actions_json, mode, enabled,
    match_count, last_matched_at, created_at, updated_at
)
SELECT
    rule_id, name, description_nl, conditions_json, actions_json, mode, enabled,
    match_count, last_matched_at, created_at, updated_at
FROM email_rules
ON CONFLICT (rule_id) DO NOTHING;

INSERT INTO communication_templates (
    template_id, name, subject_template, body_template, variables, language, created_at, updated_at
)
SELECT
    template_id, name, subject_template, body_template, variables, language, created_at, updated_at
FROM email_templates
ON CONFLICT (template_id) DO NOTHING;

INSERT INTO communication_accounts (
    account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
)
SELECT
    account_id,
    provider_kind,
    display_name,
    external_account_id,
    config,
    '{}'::jsonb,
    created_at,
    updated_at
FROM communication_provider_accounts
WHERE EXISTS (
    SELECT 1
    FROM email_personas persona
    WHERE persona.account_id = communication_provider_accounts.account_id
)
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO communication_personas (
    persona_id, name, account_id, display_name, signature, default_language, default_tone,
    is_default, metadata, created_at, updated_at
)
SELECT
    persona_id, name, account_id, display_name, signature, default_language, default_tone,
    is_default, metadata || jsonb_build_object('source_table', 'email_personas'), created_at, updated_at
FROM email_personas
ON CONFLICT (persona_id) DO NOTHING;

INSERT INTO communication_invoices (
    invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date,
    counterparty, tax_id, status, linked_project_id, linked_person_id, metadata, created_at, updated_at
)
SELECT
    invoice_id,
    message_id,
    amount,
    currency,
    invoice_number,
    issue_date,
    due_date,
    counterparty,
    tax_id,
    status,
    linked_project_id,
    COALESCE(linked_person_id, linked_contact_id),
    metadata || jsonb_build_object('source_table', 'email_invoices'),
    created_at,
    updated_at
FROM email_invoices
ON CONFLICT (invoice_id) DO NOTHING;

INSERT INTO communication_legal_documents (
    document_id, message_id, document_type, title, parties, effective_date, expiry_date,
    amount, currency, status, linked_project_id, risks, metadata, created_at, updated_at
)
SELECT
    document_id, message_id, document_type, title, parties, effective_date, expiry_date,
    amount, currency, status, linked_project_id, risks,
    metadata || jsonb_build_object('source_table', 'email_legal_documents'),
    created_at, updated_at
FROM email_legal_documents
ON CONFLICT (document_id) DO NOTHING;

INSERT INTO communication_certificates (
    cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until,
    cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage,
    linked_message_id, metadata, created_at, updated_at
)
SELECT
    cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until,
    cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage,
    linked_message_id,
    metadata || jsonb_build_object('source_table', 'email_certificates'),
    created_at, updated_at
FROM email_certificates
ON CONFLICT (cert_id) DO NOTHING;
```

### `backend/migrations/0153_allow_person_semantic_sources.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0153_allow_person_semantic_sources.sql`
- Size bytes / Размер в байтах: `283`
- Included characters / Включено символов: `283`
- Truncated / Обрезано: `no`

```text
ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_source_kind_check;

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_source_kind_check
    CHECK (source_kind IN ('message', 'document', 'project', 'task', 'contact', 'person'));
```

### `backend/migrations/0154_create_signal_hub.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0154_create_signal_hub.sql`
- Size bytes / Размер в байтах: `8758`
- Included characters / Включено символов: `8758`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS signal_sources (
    id UUID PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    category TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    default_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    supports_connections BOOLEAN NOT NULL DEFAULT FALSE,
    supports_runtime BOOLEAN NOT NULL DEFAULT FALSE,
    supports_replay BOOLEAN NOT NULL DEFAULT FALSE,
    supports_pause BOOLEAN NOT NULL DEFAULT FALSE,
    supports_mute BOOLEAN NOT NULL DEFAULT FALSE,
    capability_schema_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_sources_code_not_empty CHECK (length(trim(code)) > 0),
    CONSTRAINT signal_sources_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT signal_sources_category_not_empty CHECK (length(trim(category)) > 0),
    CONSTRAINT signal_sources_kind_not_empty CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT signal_sources_capability_schema_version_positive CHECK (capability_schema_version > 0)
);

CREATE INDEX IF NOT EXISTS signal_sources_category_idx
    ON signal_sources (category, code);

CREATE TABLE IF NOT EXISTS signal_connections (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    profile TEXT,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    secret_ref TEXT,
    connected_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    last_signal_at TIMESTAMPTZ,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_connections_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT signal_connections_status_not_empty CHECK (length(trim(status)) > 0),
    CONSTRAINT signal_connections_settings_is_object CHECK (jsonb_typeof(settings) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_connections_source_status_idx
    ON signal_connections (source_code, status);

CREATE TABLE IF NOT EXISTS signal_capabilities (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    capability TEXT NOT NULL,
    state TEXT NOT NULL,
    reason TEXT,
    requires_confirmation BOOLEAN NOT NULL DEFAULT FALSE,
    action_class TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_capabilities_capability_not_empty CHECK (length(trim(capability)) > 0),
    CONSTRAINT signal_capabilities_state_not_empty CHECK (length(trim(state)) > 0),
    CONSTRAINT signal_capabilities_action_class_not_empty CHECK (length(trim(action_class)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS signal_capabilities_identity_idx
    ON signal_capabilities (source_code, COALESCE(connection_id, '00000000-0000-0000-0000-000000000000'::uuid), capability);

CREATE TABLE IF NOT EXISTS signal_runtime_states (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    runtime_kind TEXT NOT NULL,
    state TEXT NOT NULL,
    last_started_at TIMESTAMPTZ,
    last_stopped_at TIMESTAMPTZ,
    last_heartbeat_at TIMESTAMPTZ,
    last_error_at TIMESTAMPTZ,
    last_error_code TEXT,
    last_error_message_redacted TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_runtime_states_runtime_kind_not_empty CHECK (length(trim(runtime_kind)) > 0),
    CONSTRAINT signal_runtime_states_state_not_empty CHECK (length(trim(state)) > 0),
    CONSTRAINT signal_runtime_states_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_runtime_states_source_state_idx
    ON signal_runtime_states (source_code, state);

CREATE TABLE IF NOT EXISTS signal_health (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    level TEXT NOT NULL,
    summary TEXT NOT NULL,
    last_ok_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    failure_count INTEGER NOT NULL DEFAULT 0,
    consecutive_failure_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_health_level_not_empty CHECK (length(trim(level)) > 0),
    CONSTRAINT signal_health_summary_not_empty CHECK (length(trim(summary)) > 0),
    CONSTRAINT signal_health_failure_count_non_negative CHECK (failure_count >= 0),
    CONSTRAINT signal_health_consecutive_failure_count_non_negative CHECK (consecutive_failure_count >= 0),
    CONSTRAINT signal_health_evidence_is_object CHECK (jsonb_typeof(evidence) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_health_source_level_idx
    ON signal_health (source_code, level);

CREATE TABLE IF NOT EXISTS signal_policies (
    id UUID PRIMARY KEY,
    scope TEXT NOT NULL,
    source_code TEXT REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    event_pattern TEXT,
    mode TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT signal_policies_scope_not_empty CHECK (length(trim(scope)) > 0),
    CONSTRAINT signal_policies_mode_not_empty CHECK (length(trim(mode)) > 0),
    CONSTRAINT signal_policies_reason_not_empty CHECK (length(trim(reason)) > 0),
    CONSTRAINT signal_policies_created_by_not_empty CHECK (length(trim(created_by)) > 0),
    CONSTRAINT signal_policies_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_policies_active_idx
    ON signal_policies (scope, source_code, connection_id, event_pattern, mode, expires_at);

CREATE TABLE IF NOT EXISTS signal_profiles (
    id UUID PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL,
    source_policies JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_profiles_code_not_empty CHECK (length(trim(code)) > 0),
    CONSTRAINT signal_profiles_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT signal_profiles_description_not_empty CHECK (length(trim(description)) > 0),
    CONSTRAINT signal_profiles_source_policies_is_array CHECK (jsonb_typeof(source_policies) = 'array')
);

CREATE INDEX IF NOT EXISTS signal_profiles_system_idx
    ON signal_profiles (is_system, code);

CREATE TABLE IF NOT EXISTS signal_paused_events (
    id UUID PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    raw_event_type TEXT NOT NULL,
    event_envelope JSONB NOT NULL,
    reason TEXT NOT NULL,
    paused_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    released_at TIMESTAMPTZ,

    CONSTRAINT signal_paused_events_raw_event_type_not_empty CHECK (length(trim(raw_event_type)) > 0),
    CONSTRAINT signal_paused_events_event_envelope_is_object CHECK (jsonb_typeof(event_envelope) = 'object'),
    CONSTRAINT signal_paused_events_reason_not_empty CHECK (length(trim(reason)) > 0)
);

CREATE INDEX IF NOT EXISTS signal_paused_events_source_paused_idx
    ON signal_paused_events (source_code, paused_at)
    WHERE released_at IS NULL;

CREATE TABLE IF NOT EXISTS signal_replay_requests (
    id UUID PRIMARY KEY,
    source_code TEXT REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    event_pattern TEXT,
    status TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    last_error_redacted TEXT,
    replayed_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT signal_replay_requests_status_not_empty CHECK (length(trim(status)) > 0),
    CONSTRAINT signal_replay_requests_requested_by_not_empty CHECK (length(trim(requested_by)) > 0),
    CONSTRAINT signal_replay_requests_replayed_count_non_negative CHECK (replayed_count >= 0),
    CONSTRAINT signal_replay_requests_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_replay_requests_status_idx
    ON signal_replay_requests (status, requested_at);
```

### `backend/migrations/0155_create_event_outbox.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0155_create_event_outbox.sql`
- Size bytes / Размер в байтах: `1338`
- Included characters / Включено символов: `1338`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS event_outbox (
    event_id TEXT PRIMARY KEY REFERENCES event_log(event_id) ON DELETE RESTRICT,
    subject TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error_redacted TEXT,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_outbox_subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT event_outbox_status_not_empty CHECK (length(trim(status)) > 0),
    CONSTRAINT event_outbox_attempts_non_negative CHECK (attempts >= 0)
);

CREATE INDEX IF NOT EXISTS event_outbox_pending_idx
    ON event_outbox (next_attempt_at, created_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS event_log_source_code_idx
    ON event_log ((source ->> 'source_code'), occurred_at, position)
    WHERE source ? 'source_code';

CREATE INDEX IF NOT EXISTS event_log_subject_identity_idx
    ON event_log ((subject ->> 'kind'), (subject ->> 'entity_id'), occurred_at, position)
    WHERE subject ? 'kind';

CREATE INDEX IF NOT EXISTS event_log_source_gin_idx
    ON event_log USING GIN (source);

CREATE INDEX IF NOT EXISTS event_log_subject_gin_idx
    ON event_log USING GIN (subject);
```

### `backend/migrations/0156_add_event_trace_indexes.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0156_add_event_trace_indexes.sql`
- Size bytes / Размер в байтах: `280`
- Included characters / Включено символов: `280`
- Truncated / Обрезано: `no`

```text
CREATE INDEX IF NOT EXISTS event_log_trace_position_idx
    ON event_log (correlation_id, position)
    WHERE correlation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS event_log_causation_id_position_idx
    ON event_log (causation_id, position)
    WHERE causation_id IS NOT NULL;
```

### `backend/migrations/0157_create_whatsapp_provider_write_commands.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0157_create_whatsapp_provider_write_commands.sql`
- Size bytes / Размер в байтах: `7259`
- Included characters / Включено символов: `7259`
- Truncated / Обрезано: `no`

```text
-- Migration 0157: WhatsApp provider-write command outbox foundation
-- ADR-0101: WhatsApp provider writes must be durable, capability-gated and
-- completed only after provider-observed reconciliation.

CREATE TABLE IF NOT EXISTS whatsapp_provider_write_commands (
    command_id              TEXT PRIMARY KEY,
    account_id              TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    command_kind            TEXT NOT NULL,
    idempotency_key         TEXT NOT NULL,
    provider_chat_id        TEXT NOT NULL,
    provider_message_id     TEXT,
    target_ref              JSONB NOT NULL DEFAULT '{}'::jsonb,
    payload                 JSONB NOT NULL DEFAULT '{}'::jsonb,
    capability_state        TEXT NOT NULL,
    action_class            TEXT NOT NULL,
    confirmation_decision   TEXT NOT NULL DEFAULT 'pending',
    status                  TEXT NOT NULL DEFAULT 'queued',
    retry_count             INTEGER NOT NULL DEFAULT 0,
    max_retries             INTEGER NOT NULL DEFAULT 3,
    last_error              TEXT,
    result_payload          JSONB NOT NULL DEFAULT '{}'::jsonb,
    audit_metadata          JSONB NOT NULL DEFAULT '{}'::jsonb,
    actor_id                TEXT NOT NULL,
    happened_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    next_attempt_at         TIMESTAMPTZ,
    last_attempt_at         TIMESTAMPTZ,
    locked_at               TIMESTAMPTZ,
    locked_by               TEXT,
    provider_observed_at    TIMESTAMPTZ,
    provider_state          JSONB NOT NULL DEFAULT '{}'::jsonb,
    reconciliation_status   TEXT NOT NULL DEFAULT 'not_observed',
    reconciled_at           TIMESTAMPTZ,
    dead_lettered_at        TIMESTAMPTZ,
    completed_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT whatsapp_provider_write_commands_command_id_not_empty
        CHECK (length(trim(command_id)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_idempotency_key_not_empty
        CHECK (length(trim(idempotency_key)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_actor_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'download_media',
            'send_text',
            'send_media',
            'send_voice_note',
            'reply',
            'forward',
            'edit',
            'delete',
            'react',
            'unreact',
            'mark_read',
            'mark_unread',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'pin',
            'unpin',
            'join_group',
            'leave_group',
            'publish_status'
        )),
    CONSTRAINT whatsapp_provider_write_commands_capability_state
        CHECK (capability_state IN ('available', 'blocked', 'degraded', 'unsupported')),
    CONSTRAINT whatsapp_provider_write_commands_action_class
        CHECK (action_class IN ('read', 'local_write', 'provider_write', 'destructive', 'export', 'secret_access', 'automation')),
    CONSTRAINT whatsapp_provider_write_commands_confirmation_decision
        CHECK (confirmation_decision IN ('pending', 'confirmed', 'rejected', 'not_required')),
    CONSTRAINT whatsapp_provider_write_commands_status
        CHECK (status IN (
            'queued',
            'confirmed',
            'executing',
            'retrying',
            'completed',
            'failed',
            'dead_letter',
            'cancelled'
        )),
    CONSTRAINT whatsapp_provider_write_commands_retry_count_non_negative
        CHECK (retry_count >= 0),
    CONSTRAINT whatsapp_provider_write_commands_max_retries_positive
        CHECK (max_retries > 0),
    CONSTRAINT whatsapp_provider_write_commands_target_ref_is_object
        CHECK (jsonb_typeof(target_ref) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_payload_is_object
        CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_result_payload_is_object
        CHECK (jsonb_typeof(result_payload) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_audit_metadata_is_object
        CHECK (jsonb_typeof(audit_metadata) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_provider_state_is_object
        CHECK (jsonb_typeof(provider_state) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_reconciliation_status
        CHECK (reconciliation_status IN (
            'not_observed',
            'awaiting_provider',
            'observed',
            'mismatch',
            'not_required'
        )),
    CONSTRAINT whatsapp_provider_write_commands_locked_by_not_empty
        CHECK (locked_by IS NULL OR length(trim(locked_by)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_idempotency_unique
        UNIQUE (account_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_account_idx
    ON whatsapp_provider_write_commands (account_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_chat_idx
    ON whatsapp_provider_write_commands (account_id, provider_chat_id, created_at DESC);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_idempotency_idx
    ON whatsapp_provider_write_commands (idempotency_key);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_due_idx
    ON whatsapp_provider_write_commands (account_id, status, next_attempt_at, created_at);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_reconciliation_idx
    ON whatsapp_provider_write_commands (account_id, reconciliation_status, updated_at DESC);

INSERT INTO communication_accounts (
    account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
)
SELECT
    account_id,
    provider_kind,
    display_name,
    external_account_id,
    config,
    jsonb_build_object('source_table', 'communication_provider_accounts'),
    created_at,
    updated_at
FROM communication_provider_accounts
WHERE provider_kind = 'whatsapp_web'
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO communication_provider_commands (
    command_id, account_id, channel_kind, command_kind, idempotency_key,
    provider_conversation_id, provider_message_id, target_ref, payload, capability_state,
    action_class, confirmation_decision, status, retry_count, max_retries, last_error,
    result_payload, audit_metadata, actor_id, happened_at, completed_at, created_at, updated_at
)
SELECT
    command_id,
    account_id,
    'whatsapp',
    command_kind,
    idempotency_key,
    provider_chat_id,
    provider_message_id,
    target_ref,
    payload,
    capability_state,
    action_class,
    confirmation_decision,
    status,
    retry_count,
    max_retries,
    last_error,
    result_payload,
    audit_metadata,
    actor_id,
    happened_at,
    completed_at,
    created_at,
    updated_at
FROM whatsapp_provider_write_commands
ON CONFLICT (command_id) DO NOTHING;
```

### `backend/migrations/0158_extend_whatsapp_session_link_state_for_pair_code.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0158_extend_whatsapp_session_link_state_for_pair_code.sql`
- Size bytes / Размер в байтах: `407`
- Included characters / Включено символов: `407`
- Truncated / Обрезано: `no`

```text
ALTER TABLE whatsapp_web_sessions
    DROP CONSTRAINT IF EXISTS whatsapp_web_sessions_link_state;

ALTER TABLE whatsapp_web_sessions
    ADD CONSTRAINT whatsapp_web_sessions_link_state CHECK (
        link_state IN (
            'fixture',
            'qr_pending',
            'pair_code_pending',
            'linked',
            'degraded',
            'revoked',
            'blocked'
        )
    );
```

### `backend/migrations/0159_add_whatsapp_business_cloud_provider_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0159_add_whatsapp_business_cloud_provider_kind.sql`
- Size bytes / Размер в байтах: `1319`
- Included characters / Включено символов: `1319`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_provider_accounts
    DROP CONSTRAINT IF EXISTS communication_provider_account_kind;

ALTER TABLE communication_provider_accounts
    ADD CONSTRAINT communication_provider_account_kind CHECK (
        provider_kind IN (
            'gmail',
            'icloud',
            'imap',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud'
        )
    );

ALTER TABLE whatsapp_web_sessions
    DROP CONSTRAINT IF EXISTS whatsapp_web_sessions_runtime;

ALTER TABLE whatsapp_web_sessions
    ADD CONSTRAINT whatsapp_web_sessions_runtime CHECK (
        companion_runtime IN ('fixture', 'manual_webview', 'blocked', 'api_credentials')
    );

ALTER TABLE communication_provider_account_secret_refs
    DROP CONSTRAINT IF EXISTS communication_provider_account_secret_purpose;

ALTER TABLE communication_provider_account_secret_refs
    ADD CONSTRAINT communication_provider_account_secret_purpose CHECK (
        secret_purpose IN (
            'oauth_token',
            'imap_password',
            'smtp_password',
            'telegram_api_hash',
            'telegram_session_key',
            'telegram_bot_token',
            'whatsapp_web_session_key',
            'whatsapp_business_cloud_access_token'
        )
    );
```

### `backend/migrations/0160_add_zoom_provider_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0160_add_zoom_provider_kind.sql`
- Size bytes / Размер в байтах: `1481`
- Included characters / Включено символов: `1481`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_provider_accounts
    DROP CONSTRAINT IF EXISTS communication_provider_account_kind;

ALTER TABLE communication_provider_accounts
    ADD CONSTRAINT communication_provider_account_kind CHECK (
        provider_kind IN (
            'gmail',
            'icloud',
            'imap',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud',
            'zoom_user',
            'zoom_server_to_server'
        )
    );

ALTER TABLE communication_provider_account_secret_refs
    DROP CONSTRAINT IF EXISTS communication_provider_account_secret_purpose;

ALTER TABLE communication_provider_account_secret_refs
    ADD CONSTRAINT communication_provider_account_secret_purpose CHECK (
        secret_purpose IN (
            'oauth_token',
            'imap_password',
            'smtp_password',
            'telegram_api_hash',
            'telegram_session_key',
            'telegram_bot_token',
            'whatsapp_web_session_key',
            'whatsapp_business_cloud_access_token',
            'whatsapp_business_cloud_app_secret',
            'whatsapp_business_cloud_webhook_verify_token',
            'zoom_oauth_token',
            'zoom_client_secret',
            'zoom_webhook_secret'
        )
    );

CREATE INDEX IF NOT EXISTS telegram_calls_zoom_provider_idx
    ON telegram_calls (account_id, provider_call_id, created_at DESC)
    WHERE metadata->>'provider' = 'zoom';
```

### `backend/migrations/0161_expand_communication_delivery_state.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0161_expand_communication_delivery_state.sql`
- Size bytes / Размер в байтах: `416`
- Included characters / Включено символов: `416`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_delivery_state;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_delivery_state CHECK (
        delivery_state IN (
            'received',
            'sent',
            'delivered',
            'read',
            'played',
            'send_dry_run',
            'send_blocked'
        )
    );
```

### `backend/migrations/0162_extend_canonical_provider_commands_runtime_state.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0162_extend_canonical_provider_commands_runtime_state.sql`
- Size bytes / Размер в байтах: `825`
- Included characters / Включено символов: `825`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_provider_commands
    ADD COLUMN IF NOT EXISTS provider_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS reconciliation_status TEXT NOT NULL DEFAULT 'not_observed',
    ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_observed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS reconciled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dead_lettered_at TIMESTAMPTZ;

ALTER TABLE communication_provider_commands
    ADD CONSTRAINT communication_provider_commands_provider_state_is_object
        CHECK (jsonb_typeof(provider_state) = 'object'),
    ADD CONSTRAINT communication_provider_commands_reconciliation_status_not_empty
        CHECK (length(trim(reconciliation_status)) > 0);
```

### `backend/migrations/0163_expand_communication_message_channel_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0163_expand_communication_message_channel_kind.sql`
- Size bytes / Размер в байтах: `388`
- Included characters / Включено символов: `388`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_channel_kind;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN (
            'email',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud'
        )
    );
```

### `backend/migrations/0164_expand_calendar_event_relation_entity_type.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0164_expand_calendar_event_relation_entity_type.sql`
- Size bytes / Размер в байтах: `413`
- Included characters / Включено символов: `413`
- Truncated / Обрезано: `no`

```text
ALTER TABLE event_relations
DROP CONSTRAINT IF EXISTS event_relations_entity_type_check;

ALTER TABLE event_relations
ADD CONSTRAINT event_relations_entity_type_check CHECK (
    entity_type IN (
        'person',
        'organization',
        'project',
        'document',
        'task',
        'email',
        'note',
        'decision',
        'obligation',
        'recording',
        'call'
    )
);
```

### `backend/migrations/0165_add_yandex_telemost_provider_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0165_add_yandex_telemost_provider_kind.sql`
- Size bytes / Размер в байтах: `1387`
- Included characters / Включено символов: `1387`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_provider_accounts
    DROP CONSTRAINT IF EXISTS communication_provider_account_kind;

ALTER TABLE communication_provider_accounts
    ADD CONSTRAINT communication_provider_account_kind CHECK (
        provider_kind IN (
            'gmail',
            'icloud',
            'imap',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud',
            'zoom_user',
            'zoom_server_to_server',
            'yandex_telemost_user'
        )
    );

ALTER TABLE communication_provider_account_secret_refs
    DROP CONSTRAINT IF EXISTS communication_provider_account_secret_purpose;

ALTER TABLE communication_provider_account_secret_refs
    ADD CONSTRAINT communication_provider_account_secret_purpose CHECK (
        secret_purpose IN (
            'oauth_token',
            'imap_password',
            'smtp_password',
            'telegram_api_hash',
            'telegram_session_key',
            'telegram_bot_token',
            'whatsapp_web_session_key',
            'whatsapp_business_cloud_access_token',
            'whatsapp_business_cloud_app_secret',
            'whatsapp_business_cloud_webhook_verify_token',
            'zoom_oauth_token',
            'zoom_client_secret',
            'zoom_webhook_secret',
            'yandex_telemost_oauth_token'
        )
    );
```

### `backend/migrations/0166_add_realtime_conversation_radar_signal_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0166_add_realtime_conversation_radar_signal_observation_kind.sql`
- Size bytes / Размер в байтах: `678`
- Included characters / Включено символов: `678`
- Truncated / Обрезано: `no`

```text
INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:realtime_conversation_radar_signal',
    'REALTIME_CONVERSATION_RADAR_SIGNAL',
    'Realtime conversation radar signal',
    1,
    'meeting',
    'Provider-neutral realtime conversation radar candidate captured from a local or provider runtime before owner review and promotion.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```
