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

- Chunk ID / ID чанка: `096-other-backend-part-003`
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

### `backend/migrations/0051_tasks_rules_templates.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0051_tasks_rules_templates.sql`
- Size bytes / Размер в байтах: `3251`
- Included characters / Включено символов: `3251`
- Truncated / Обрезано: `no`

```text
-- Phase 4: Rules and templates

CREATE TABLE IF NOT EXISTS task_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    natural_language_description TEXT,
    compiled_dsl JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    approval_mode TEXT NOT NULL DEFAULT 'suggest_only',
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_rules_approval_check CHECK (approval_mode IN ('suggest_only','ask_before_execute','auto_execute','dry_run'))
);

CREATE TABLE IF NOT EXISTS task_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    default_fields JSONB NOT NULL DEFAULT '{}',
    default_checklist JSONB NOT NULL DEFAULT '[]',
    default_priority TEXT DEFAULT 'medium',
    default_energy_type TEXT,
    required_documents JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO task_templates (template_id, name, description, default_fields, default_checklist, default_priority) VALUES
    ('bug','Bug Report','Standard bug report template','{"source_type":"manual","area":"engineering"}','["Steps to reproduce","Expected result","Actual result","Environment details"]','high'),
    ('feature','Feature Request','New feature specification','{"source_type":"manual","area":"engineering"}','["Requirements","Design doc","Implementation plan","Tests"]','medium'),
    ('research','Research Task','Investigation template','{"source_type":"manual","area":"research"}','["Define question","Gather sources","Document findings","Make decision"]','medium'),
    ('contract_review','Contract Review','Legal document review','{"source_type":"manual","area":"legal"}','["Check parties","Check amounts","Check deadlines","Check signatures","Check terms","Create summary"]','high'),
    ('aeat_response','AEAT Response','Spanish tax agency response','{"source_type":"manual","area":"tax"}','["Check documents","Check certificado digital","Download PDFs","Check deadline","Prepare response","Submit"]','critical'),
    ('client_followup','Client Follow-up','Post-meeting client follow-up','{"source_type":"meeting","area":"client"}','["Send follow-up email","Update project status","Create tasks from decisions","Schedule next check-in"]','medium'),
    ('invoice_review','Invoice Review','Invoice verification','{"source_type":"manual","area":"finance"}','["Check amount","Check VAT","Check dates","Check provider details","Approve or flag"]','high'),
    ('code_review','Code Review','Code review task','{"source_type":"manual","area":"engineering"}','["Review diff","Check tests","Check docs","Add comments","Approve or request changes"]','medium')
ON CONFLICT (template_id) DO NOTHING;

CREATE TABLE IF NOT EXISTS task_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    snapshot_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL,
    source TEXT DEFAULT 'system',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS task_snapshots_task_idx ON task_snapshots (task_id);
```

### `backend/migrations/0052_remove_frontend_actor_setting.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0052_remove_frontend_actor_setting.sql`
- Size bytes / Размер в байтах: `74`
- Included characters / Включено символов: `74`
- Truncated / Обрезано: `no`

```text
DELETE FROM application_settings
WHERE setting_key = 'frontend.actor_id';
```

### `backend/migrations/0053_fix_person_identity_candidate_kind_constraints.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0053_fix_person_identity_candidate_kind_constraints.sql`
- Size bytes / Размер в байтах: `1484`
- Included characters / Включено символов: `1484`
- Truncated / Обрезано: `no`

```text
UPDATE person_identity_candidates
SET candidate_kind = CASE candidate_kind
    WHEN 'merge_contacts' THEN 'merge_persons'
    WHEN 'split_contact' THEN 'split_person'
    ELSE candidate_kind
END
WHERE candidate_kind IN ('merge_contacts', 'split_contact');

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS contact_identity_candidate_kind_check;

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS person_identity_candidate_kind_check;

ALTER TABLE person_identity_candidates
    ADD CONSTRAINT person_identity_candidate_kind_check
        CHECK (candidate_kind IN ('merge_persons', 'attach_email_address', 'split_person'));

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS contact_identity_merge_has_right_contact;

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS person_identity_merge_has_right_person;

ALTER TABLE person_identity_candidates
    ADD CONSTRAINT person_identity_merge_has_right_person
        CHECK (candidate_kind <> 'merge_persons' OR right_person_id IS NOT NULL);

DROP INDEX IF EXISTS contact_identity_merge_pair_idx;
DROP INDEX IF EXISTS person_identity_merge_pair_idx;

CREATE UNIQUE INDEX person_identity_merge_pair_idx
    ON person_identity_candidates (
        candidate_kind,
        LEAST(left_person_id, COALESCE(right_person_id, left_person_id)),
        GREATEST(left_person_id, COALESCE(right_person_id, left_person_id))
    )
    WHERE candidate_kind = 'merge_persons';
```

### `backend/migrations/0054_add_host_vault_secret_store_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0054_add_host_vault_secret_store_kind.sql`
- Size bytes / Размер в байтах: `388`
- Included characters / Включено символов: `388`
- Truncated / Обрезано: `no`

```text
ALTER TABLE secret_references
    DROP CONSTRAINT secret_references_store_kind;

ALTER TABLE secret_references
    ADD CONSTRAINT secret_references_store_kind CHECK (
        store_kind IN (
            'os_keychain',
            'encrypted_vault',
            'database_encrypted_vault',
            'host_vault',
            'external_vault',
            'test_double'
        )
    );
```

### `backend/migrations/0055_mail_sync_local_trash.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0055_mail_sync_local_trash.sql`
- Size bytes / Размер в байтах: `5094`
- Included characters / Включено символов: `5094`
- Truncated / Обрезано: `no`

```text
-- ADR-0080: per-account mail sync progress and local-only trash

ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS local_state TEXT NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS local_state_changed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS local_state_reason TEXT;

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_local_state_check;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_local_state_check CHECK (
        local_state IN ('active', 'trash')
    );

CREATE INDEX IF NOT EXISTS communication_messages_local_state_idx
    ON communication_messages (local_state, COALESCE(occurred_at, projected_at) DESC);

CREATE TABLE IF NOT EXISTS communication_account_sync_settings (
    account_id TEXT PRIMARY KEY REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    batch_size INTEGER NOT NULL DEFAULT 5,
    poll_interval_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_account_sync_settings_batch_size_check CHECK (batch_size BETWEEN 1 AND 500),
    CONSTRAINT communication_account_sync_settings_poll_interval_check CHECK (poll_interval_seconds BETWEEN 60 AND 86400)
);

CREATE TABLE IF NOT EXISTS communication_mail_sync_runs (
    run_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    trigger TEXT NOT NULL,
    status TEXT NOT NULL,
    phase TEXT NOT NULL,
    progress_mode TEXT NOT NULL DEFAULT 'indeterminate',
    progress_percent INTEGER,
    processed_messages BIGINT NOT NULL DEFAULT 0,
    estimated_total_messages BIGINT,
    current_batch_size INTEGER NOT NULL DEFAULT 0,
    fetched_messages BIGINT NOT NULL DEFAULT 0,
    projected_messages BIGINT NOT NULL DEFAULT 0,
    upserted_persons BIGINT NOT NULL DEFAULT 0,
    upserted_organizations BIGINT NOT NULL DEFAULT 0,
    checkpoint_before JSONB,
    checkpoint_after JSONB,
    checkpoint_saved BOOLEAN NOT NULL DEFAULT false,
    error_code TEXT,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_mail_sync_runs_trigger_check CHECK (trigger IN ('scheduled', 'manual')),
    CONSTRAINT communication_mail_sync_runs_status_check CHECK (status IN ('queued', 'running', 'completed', 'failed', 'skipped', 'recoverable_full_resync_needed')),
    CONSTRAINT communication_mail_sync_runs_phase_check CHECK (phase IN ('idle', 'waiting_for_vault', 'listing', 'fetching', 'projecting', 'persons_graph', 'completed', 'failed', 'skipped')),
    CONSTRAINT communication_mail_sync_runs_progress_mode_check CHECK (progress_mode IN ('none', 'determinate', 'indeterminate')),
    CONSTRAINT communication_mail_sync_runs_progress_percent_check CHECK (progress_percent IS NULL OR (progress_percent >= 0 AND progress_percent <= 100)),
    CONSTRAINT communication_mail_sync_runs_checkpoint_before_is_object CHECK (checkpoint_before IS NULL OR jsonb_typeof(checkpoint_before) = 'object'),
    CONSTRAINT communication_mail_sync_runs_checkpoint_after_is_object CHECK (checkpoint_after IS NULL OR jsonb_typeof(checkpoint_after) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_mail_sync_runs_account_started_idx
    ON communication_mail_sync_runs (account_id, started_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS communication_mail_sync_runs_active_account_idx
    ON communication_mail_sync_runs (account_id)
    WHERE status IN ('queued', 'running', 'recoverable_full_resync_needed');

CREATE TABLE IF NOT EXISTS communication_message_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    email_address TEXT NOT NULL,
    display_name TEXT,
    role TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'email_sync',
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_participants_role_check CHECK (role IN ('sender', 'recipient', 'cc', 'bcc')),
    CONSTRAINT communication_message_participants_email_not_empty CHECK (length(trim(email_address)) > 0),
    CONSTRAINT communication_message_participants_confidence_check CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT communication_message_participants_unique UNIQUE (message_id, person_id, role, email_address)
);

CREATE INDEX IF NOT EXISTS communication_message_participants_message_idx
    ON communication_message_participants (message_id);

CREATE INDEX IF NOT EXISTS communication_message_participants_person_idx
    ON communication_message_participants (person_id);
```

### `backend/migrations/0056_email_invoices_linked_person.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0056_email_invoices_linked_person.sql`
- Size bytes / Размер в байтах: `253`
- Included characters / Включено символов: `253`
- Truncated / Обрезано: `no`

```text
-- Keep email finance schema aligned with the mail finance API model.

ALTER TABLE email_invoices
    ADD COLUMN IF NOT EXISTS linked_person_id TEXT;

CREATE INDEX IF NOT EXISTS email_invoices_linked_person_idx
    ON email_invoices (linked_person_id);
```

### `backend/migrations/0057_ai_control_center.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0057_ai_control_center.sql`
- Size bytes / Размер в байтах: `11554`
- Included characters / Включено символов: `11554`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS ai_provider_accounts (
    provider_id TEXT PRIMARY KEY,
    provider_kind TEXT NOT NULL,
    provider_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'needs_setup',
    consent_state TEXT NOT NULL DEFAULT 'not_required',
    consented_at TIMESTAMPTZ,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    capabilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_provider_accounts_kind_check
        CHECK (provider_kind IN ('built_in', 'cli', 'api')),
    CONSTRAINT ai_provider_accounts_status_check
        CHECK (status IN ('ready', 'disabled', 'needs_setup', 'error')),
    CONSTRAINT ai_provider_accounts_consent_check
        CHECK (consent_state IN ('not_required', 'required', 'granted', 'revoked')),
    CONSTRAINT ai_provider_accounts_key_not_empty CHECK (length(trim(provider_key)) > 0),
    CONSTRAINT ai_provider_accounts_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT ai_provider_accounts_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT ai_provider_accounts_capabilities_is_array CHECK (jsonb_typeof(capabilities) = 'array')
);

CREATE UNIQUE INDEX IF NOT EXISTS ai_provider_accounts_kind_key_idx
    ON ai_provider_accounts (provider_kind, provider_key);

CREATE TABLE IF NOT EXISTS ai_provider_secret_refs (
    provider_id TEXT NOT NULL REFERENCES ai_provider_accounts(provider_id) ON DELETE CASCADE,
    secret_purpose TEXT NOT NULL,
    secret_ref TEXT NOT NULL REFERENCES secret_references(secret_ref) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (provider_id, secret_purpose),
    CONSTRAINT ai_provider_secret_refs_purpose_check
        CHECK (secret_purpose IN ('api_key'))
);

CREATE TABLE IF NOT EXISTS ai_model_catalog (
    provider_id TEXT NOT NULL REFERENCES ai_provider_accounts(provider_id) ON DELETE CASCADE,
    model_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    category TEXT NOT NULL,
    privacy TEXT NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    context_window INTEGER,
    embedding_dimension INTEGER,
    is_available BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (provider_id, model_key),
    CONSTRAINT ai_model_catalog_key_not_empty CHECK (length(trim(model_key)) > 0),
    CONSTRAINT ai_model_catalog_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT ai_model_catalog_category_not_empty CHECK (length(trim(category)) > 0),
    CONSTRAINT ai_model_catalog_privacy_check CHECK (privacy IN ('local', 'remote', 'cli')),
    CONSTRAINT ai_model_catalog_capabilities_is_array CHECK (jsonb_typeof(capabilities) = 'array'),
    CONSTRAINT ai_model_catalog_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT ai_model_catalog_context_positive CHECK (context_window IS NULL OR context_window > 0),
    CONSTRAINT ai_model_catalog_embedding_positive CHECK (embedding_dimension IS NULL OR embedding_dimension > 0)
);

CREATE INDEX IF NOT EXISTS ai_model_catalog_category_idx
    ON ai_model_catalog (category, display_name);

CREATE TABLE IF NOT EXISTS ai_model_routes (
    capability_slot TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    model_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_model_routes_slot_check CHECK (
        capability_slot IN (
            'default_chat',
            'reasoning',
            'summarization',
            'mail_intelligence',
            'reply_draft',
            'extraction',
            'embeddings',
            'meeting_prep'
        )
    ),
    CONSTRAINT ai_model_routes_model_fk
        FOREIGN KEY (provider_id, model_key)
        REFERENCES ai_model_catalog(provider_id, model_key)
        ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS ai_prompt_templates (
    prompt_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    entity_scope TEXT NOT NULL,
    capability_slot TEXT NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    active_version_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_prompt_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT ai_prompt_templates_entity_scope_check CHECK (
        entity_scope IN (
            'global',
            'person',
            'organization',
            'project',
            'document',
            'task',
            'meeting',
            'communication',
            'conversation'
        )
    ),
    CONSTRAINT ai_prompt_templates_slot_check CHECK (
        capability_slot IN (
            'default_chat',
            'reasoning',
            'summarization',
            'mail_intelligence',
            'reply_draft',
            'extraction',
            'embeddings',
            'meeting_prep'
        )
    ),
    CONSTRAINT ai_prompt_templates_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS ai_prompt_templates_scope_slot_idx
    ON ai_prompt_templates (entity_scope, capability_slot);

CREATE TABLE IF NOT EXISTS ai_prompt_template_versions (
    prompt_version_id TEXT PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES ai_prompt_templates(prompt_id) ON DELETE CASCADE,
    version_label TEXT NOT NULL,
    body_template TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft',
    created_by_actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_prompt_template_versions_label_not_empty CHECK (length(trim(version_label)) > 0),
    CONSTRAINT ai_prompt_template_versions_body_not_empty CHECK (length(trim(body_template)) > 0),
    CONSTRAINT ai_prompt_template_versions_variables_is_array CHECK (jsonb_typeof(variables) = 'array'),
    CONSTRAINT ai_prompt_template_versions_status_check CHECK (status IN ('draft', 'active', 'archived')),
    CONSTRAINT ai_prompt_template_versions_actor_not_empty CHECK (length(trim(created_by_actor_id)) > 0)
);

CREATE INDEX IF NOT EXISTS ai_prompt_template_versions_prompt_idx
    ON ai_prompt_template_versions (prompt_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ai_prompt_eval_runs (
    eval_run_id TEXT PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES ai_prompt_templates(prompt_id) ON DELETE CASCADE,
    prompt_version_id TEXT NOT NULL REFERENCES ai_prompt_template_versions(prompt_version_id) ON DELETE RESTRICT,
    provider_id TEXT NOT NULL,
    model_key TEXT NOT NULL,
    source_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    output_text TEXT NOT NULL,
    score INTEGER,
    notes TEXT,
    actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_prompt_eval_runs_source_refs_is_array CHECK (jsonb_typeof(source_refs) = 'array'),
    CONSTRAINT ai_prompt_eval_runs_variables_is_object CHECK (jsonb_typeof(variables) = 'object'),
    CONSTRAINT ai_prompt_eval_runs_output_not_empty CHECK (length(trim(output_text)) > 0),
    CONSTRAINT ai_prompt_eval_runs_score_range CHECK (score IS NULL OR (score >= 0 AND score <= 100)),
    CONSTRAINT ai_prompt_eval_runs_actor_not_empty CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT ai_prompt_eval_runs_model_fk
        FOREIGN KEY (provider_id, model_key)
        REFERENCES ai_model_catalog(provider_id, model_key)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS ai_prompt_eval_runs_prompt_idx
    ON ai_prompt_eval_runs (prompt_id, created_at DESC);

INSERT INTO ai_provider_accounts (
    provider_id,
    provider_kind,
    provider_key,
    display_name,
    status,
    consent_state,
    config,
    capabilities
)
VALUES (
    'provider:built_in:ollama',
    'built_in',
    'ollama',
    'Built-in Ollama',
    'ready',
    'not_required',
    '{"base_url":"http://127.0.0.1:11434","manager":"ollama"}'::jsonb,
    '["chat","embeddings","local_runtime"]'::jsonb
)
ON CONFLICT (provider_id) DO NOTHING;

INSERT INTO ai_model_catalog (
    provider_id,
    model_key,
    display_name,
    category,
    privacy,
    capabilities,
    context_window,
    embedding_dimension,
    metadata
)
VALUES
    (
        'provider:built_in:ollama',
        'qwen3:4b',
        'Qwen3 4B',
        'chat',
        'local',
        '["chat","reasoning","summarization","extraction"]'::jsonb,
        32768,
        NULL,
        '{"curated":true,"pull_required":true}'::jsonb
    ),
    (
        'provider:built_in:ollama',
        'qwen3-embedding:4b',
        'Qwen3 Embedding 4B',
        'embeddings',
        'local',
        '["embeddings"]'::jsonb,
        8192,
        2560,
        '{"curated":true,"pull_required":true}'::jsonb
    )
ON CONFLICT (provider_id, model_key) DO NOTHING;

INSERT INTO ai_model_routes (capability_slot, provider_id, model_key)
VALUES
    ('default_chat', 'provider:built_in:ollama', 'qwen3:4b'),
    ('reasoning', 'provider:built_in:ollama', 'qwen3:4b'),
    ('summarization', 'provider:built_in:ollama', 'qwen3:4b'),
    ('mail_intelligence', 'provider:built_in:ollama', 'qwen3:4b'),
    ('reply_draft', 'provider:built_in:ollama', 'qwen3:4b'),
    ('extraction', 'provider:built_in:ollama', 'qwen3:4b'),
    ('embeddings', 'provider:built_in:ollama', 'qwen3-embedding:4b'),
    ('meeting_prep', 'provider:built_in:ollama', 'qwen3:4b')
ON CONFLICT (capability_slot) DO NOTHING;

INSERT INTO ai_prompt_templates (
    prompt_id,
    name,
    entity_scope,
    capability_slot,
    description,
    is_system,
    active_version_id,
    metadata
)
VALUES
    (
        'prompt:system:global:default_chat',
        'Default source-backed answer',
        'global',
        'default_chat',
        'System seed prompt for source-backed answers.',
        true,
        'prompt-version:system:global:default_chat:v1',
        '{"seeded":true}'::jsonb
    ),
    (
        'prompt:system:communication:mail_intelligence',
        'Mail intelligence',
        'communication',
        'mail_intelligence',
        'System seed prompt for communication intelligence.',
        true,
        'prompt-version:system:communication:mail_intelligence:v1',
        '{"seeded":true}'::jsonb
    )
ON CONFLICT (prompt_id) DO NOTHING;

INSERT INTO ai_prompt_template_versions (
    prompt_version_id,
    prompt_id,
    version_label,
    body_template,
    variables,
    status,
    created_by_actor_id
)
VALUES
    (
        'prompt-version:system:global:default_chat:v1',
        'prompt:system:global:default_chat',
        'v1',
        'Answer using only cited Макошь context. Query: {{query}}',
        '["query"]'::jsonb,
        'active',
        'system:ai-control-center'
    ),
    (
        'prompt-version:system:communication:mail_intelligence:v1',
        'prompt:system:communication:mail_intelligence',
        'v1',
        'Analyze this communication and return concise operational context. Subject: {{subject}}',
        '["subject"]'::jsonb,
        'active',
        'system:ai-control-center'
    )
ON CONFLICT (prompt_version_id) DO NOTHING;
```

### `backend/migrations/0058_allow_empty_telegram_tdlib_message_bodies.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0058_allow_empty_telegram_tdlib_message_bodies.sql`
- Size bytes / Размер в байтах: `438`
- Included characters / Включено символов: `438`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_body_not_empty;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_body_not_empty CHECK (
        length(trim(body_text)) > 0
        OR (
            channel_kind IN ('telegram_user', 'telegram_bot')
            AND jsonb_typeof(message_metadata) = 'object'
            AND message_metadata ? 'tdlib_raw'
        )
    );
```

### `backend/migrations/0059_persona_owner_type_constraints.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0059_persona_owner_type_constraints.sql`
- Size bytes / Размер в байтах: `770`
- Included characters / Включено символов: `770`
- Truncated / Обрезано: `no`

```text
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS is_self BOOLEAN NOT NULL DEFAULT false;

UPDATE persons
SET person_type = 'human'
WHERE person_type IS NULL
   OR person_type NOT IN ('human', 'ai_agent', 'organization_proxy', 'system');

ALTER TABLE persons
    ALTER COLUMN person_type SET DEFAULT 'human',
    ALTER COLUMN person_type SET NOT NULL;

DO $$
BEGIN
    ALTER TABLE persons
        ADD CONSTRAINT persons_person_type_check
        CHECK (person_type IN ('human', 'ai_agent', 'organization_proxy', 'system'));
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE INDEX IF NOT EXISTS persons_person_type_idx
    ON persons (person_type);

CREATE UNIQUE INDEX IF NOT EXISTS persons_single_self_idx
    ON persons (is_self)
    WHERE is_self = true;
```

### `backend/migrations/0060_create_relationships.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0060_create_relationships.sql`
- Size bytes / Размер в байтах: `4371`
- Included characters / Включено символов: `4371`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS relationships (
    relationship_id TEXT PRIMARY KEY,
    source_entity_kind TEXT NOT NULL,
    source_entity_id TEXT NOT NULL,
    target_entity_kind TEXT NOT NULL,
    target_entity_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    trust_score NUMERIC(5,4) NOT NULL DEFAULT 0.5000,
    strength_score NUMERIC(5,4) NOT NULL DEFAULT 0.5000,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT relationships_entity_kind_check CHECK (
        source_entity_kind IN (
            'persona',
            'organization',
            'project',
            'communication',
            'document',
            'task',
            'event',
            'decision',
            'obligation',
            'knowledge'
        )
        AND target_entity_kind IN (
            'persona',
            'organization',
            'project',
            'communication',
            'document',
            'task',
            'event',
            'decision',
            'obligation',
            'knowledge'
        )
    ),
    CONSTRAINT relationships_source_entity_id_not_empty CHECK (length(trim(source_entity_id)) > 0),
    CONSTRAINT relationships_target_entity_id_not_empty CHECK (length(trim(target_entity_id)) > 0),
    CONSTRAINT relationships_type_not_empty CHECK (length(trim(relationship_type)) > 0),
    CONSTRAINT relationships_distinct_endpoints CHECK (
        source_entity_kind != target_entity_kind
        OR source_entity_id != target_entity_id
    ),
    CONSTRAINT relationships_trust_score_range CHECK (trust_score >= 0.0 AND trust_score <= 1.0),
    CONSTRAINT relationships_strength_score_range CHECK (strength_score >= 0.0 AND strength_score <= 1.0),
    CONSTRAINT relationships_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT relationships_review_state_check CHECK (
        review_state IN ('suggested', 'system_accepted', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT relationships_temporal_range_check CHECK (
        valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from
    ),
    CONSTRAINT relationships_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS relationships_active_unique
    ON relationships (
        source_entity_kind,
        source_entity_id,
        target_entity_kind,
        target_entity_id,
        relationship_type
    )
    WHERE valid_to IS NULL;

CREATE INDEX IF NOT EXISTS relationships_source_idx
    ON relationships (source_entity_kind, source_entity_id);
CREATE INDEX IF NOT EXISTS relationships_target_idx
    ON relationships (target_entity_kind, target_entity_id);
CREATE INDEX IF NOT EXISTS relationships_type_idx
    ON relationships (relationship_type);
CREATE INDEX IF NOT EXISTS relationships_review_state_idx
    ON relationships (review_state, updated_at);

CREATE TABLE IF NOT EXISTS relationship_evidence (
    evidence_id TEXT PRIMARY KEY,
    relationship_id TEXT NOT NULL REFERENCES relationships(relationship_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    excerpt TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT relationship_evidence_source_kind_check CHECK (
        source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    ),
    CONSTRAINT relationship_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT relationship_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (relationship_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS relationship_evidence_relationship_idx
    ON relationship_evidence (relationship_id);
CREATE INDEX IF NOT EXISTS relationship_evidence_source_idx
    ON relationship_evidence (source_kind, source_id);
```

### `backend/migrations/0061_relationship_graph_projection.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0061_relationship_graph_projection.sql`
- Size bytes / Размер в байтах: `802`
- Included characters / Включено символов: `802`
- Truncated / Обрезано: `no`

```text
ALTER TABLE graph_edges DROP CONSTRAINT IF EXISTS graph_edges_relationship_type;
ALTER TABLE graph_edges
ADD CONSTRAINT graph_edges_relationship_type CHECK (
    relationship_type IN (
        'person_has_email_address',
        'person_sent_message',
        'person_received_message',
        'email_address_sent_message',
        'email_address_received_message',
        'project_has_message',
        'project_has_document',
        'project_involves_person',
        'project_involves_email_address',
        'entity_relationship'
    )
);

ALTER TABLE graph_evidence DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;
ALTER TABLE graph_evidence
ADD CONSTRAINT graph_evidence_source_kind CHECK (
    source_kind IN ('contact', 'person', 'message', 'document', 'raw_record', 'relationship')
);
```

### `backend/migrations/0062_create_contradiction_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0062_create_contradiction_observations.sql`
- Size bytes / Размер в байтах: `3318`
- Included characters / Включено символов: `3318`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS contradiction_observations (
    observation_id TEXT PRIMARY KEY,
    old_source_kind TEXT NOT NULL,
    old_source_id TEXT NOT NULL,
    new_source_kind TEXT NOT NULL,
    new_source_id TEXT NOT NULL,
    affected_entities JSONB NOT NULL DEFAULT '[]'::jsonb,
    conflict_type TEXT NOT NULL,
    old_claim TEXT NOT NULL,
    new_claim TEXT NOT NULL,
    confidence NUMERIC(5,4) NOT NULL,
    severity TEXT NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    resolution TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT contradiction_observations_source_kind_check CHECK (
        old_source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'raw_record'
        )
        AND new_source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'raw_record'
        )
    ),
    CONSTRAINT contradiction_observations_old_source_id_not_empty CHECK (length(trim(old_source_id)) > 0),
    CONSTRAINT contradiction_observations_new_source_id_not_empty CHECK (length(trim(new_source_id)) > 0),
    CONSTRAINT contradiction_observations_conflict_type_not_empty CHECK (length(trim(conflict_type)) > 0),
    CONSTRAINT contradiction_observations_old_claim_not_empty CHECK (length(trim(old_claim)) > 0),
    CONSTRAINT contradiction_observations_new_claim_not_empty CHECK (length(trim(new_claim)) > 0),
    CONSTRAINT contradiction_observations_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT contradiction_observations_severity_check CHECK (
        severity IN ('low', 'medium', 'high', 'critical')
    ),
    CONSTRAINT contradiction_observations_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT contradiction_observations_affected_entities_json_check CHECK (
        jsonb_typeof(affected_entities) IN ('array', 'object')
    ),
    CONSTRAINT contradiction_observations_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT contradiction_observations_reviewed_by_not_empty CHECK (
        reviewed_by IS NULL OR length(trim(reviewed_by)) > 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS contradiction_observations_source_unique
    ON contradiction_observations (
        old_source_kind,
        old_source_id,
        new_source_kind,
        new_source_id,
        conflict_type
    );

CREATE INDEX IF NOT EXISTS contradiction_observations_review_state_idx
    ON contradiction_observations (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS contradiction_observations_new_source_idx
    ON contradiction_observations (new_source_kind, new_source_id);

CREATE INDEX IF NOT EXISTS contradiction_observations_old_source_idx
    ON contradiction_observations (old_source_kind, old_source_id);
```

### `backend/migrations/0063_create_obligations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0063_create_obligations.sql`
- Size bytes / Размер в байтах: `5485`
- Included characters / Включено символов: `5485`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS obligations (
    obligation_id TEXT PRIMARY KEY,
    obligated_entity_kind TEXT NOT NULL,
    obligated_entity_id TEXT NOT NULL,
    beneficiary_entity_kind TEXT,
    beneficiary_entity_id TEXT,
    statement TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    review_state TEXT NOT NULL DEFAULT 'suggested',
    due_at TIMESTAMPTZ,
    condition TEXT,
    risk_state TEXT NOT NULL DEFAULT 'none',
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT obligations_entity_kind_check CHECK (
        obligated_entity_kind IN (
            'persona',
            'organization',
            'project',
            'communication',
            'document',
            'task',
            'event',
            'decision',
            'obligation',
            'knowledge'
        )
        AND (
            beneficiary_entity_kind IS NULL
            OR beneficiary_entity_kind IN (
                'persona',
                'organization',
                'project',
                'communication',
                'document',
                'task',
                'event',
                'decision',
                'obligation',
                'knowledge'
            )
        )
    ),
    CONSTRAINT obligations_obligated_entity_id_not_empty CHECK (length(trim(obligated_entity_id)) > 0),
    CONSTRAINT obligations_beneficiary_pair_check CHECK (
        (beneficiary_entity_kind IS NULL AND beneficiary_entity_id IS NULL)
        OR (
            beneficiary_entity_kind IS NOT NULL
            AND beneficiary_entity_id IS NOT NULL
            AND length(trim(beneficiary_entity_id)) > 0
        )
    ),
    CONSTRAINT obligations_statement_not_empty CHECK (length(trim(statement)) > 0),
    CONSTRAINT obligations_status_check CHECK (
        status IN ('open', 'fulfilled', 'waived', 'disputed', 'canceled')
    ),
    CONSTRAINT obligations_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT obligations_risk_state_check CHECK (
        risk_state IN ('none', 'watch', 'at_risk', 'breached')
    ),
    CONSTRAINT obligations_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT obligations_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS obligations_active_unique
    ON obligations (
        obligated_entity_kind,
        obligated_entity_id,
        COALESCE(beneficiary_entity_kind, ''),
        COALESCE(beneficiary_entity_id, ''),
        lower(statement)
    )
    WHERE status IN ('open', 'disputed');

CREATE INDEX IF NOT EXISTS obligations_obligated_entity_idx
    ON obligations (obligated_entity_kind, obligated_entity_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS obligations_beneficiary_entity_idx
    ON obligations (beneficiary_entity_kind, beneficiary_entity_id, updated_at DESC)
    WHERE beneficiary_entity_kind IS NOT NULL;
CREATE INDEX IF NOT EXISTS obligations_status_idx
    ON obligations (status, updated_at DESC);
CREATE INDEX IF NOT EXISTS obligations_review_state_idx
    ON obligations (review_state, updated_at DESC);
CREATE INDEX IF NOT EXISTS obligations_risk_state_idx
    ON obligations (risk_state, updated_at DESC);

CREATE TABLE IF NOT EXISTS obligation_evidence (
    evidence_id TEXT PRIMARY KEY,
    obligation_id TEXT NOT NULL REFERENCES obligations(obligation_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    quote TEXT,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT obligation_evidence_source_kind_check CHECK (
        source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    ),
    CONSTRAINT obligation_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT obligation_evidence_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT obligation_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (obligation_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS obligation_evidence_obligation_idx
    ON obligation_evidence (obligation_id);
CREATE INDEX IF NOT EXISTS obligation_evidence_source_idx
    ON obligation_evidence (source_kind, source_id);

CREATE TABLE IF NOT EXISTS obligation_task_links (
    obligation_id TEXT NOT NULL REFERENCES obligations(obligation_id) ON DELETE CASCADE,
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    link_kind TEXT NOT NULL DEFAULT 'related',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT obligation_task_links_kind_check CHECK (
        link_kind IN ('related', 'fulfillment_task', 'follow_up_task', 'evidence_task')
    ),
    CONSTRAINT obligation_task_links_task_id_not_empty CHECK (length(trim(task_id)) > 0),
    PRIMARY KEY (obligation_id, task_id, link_kind)
);

CREATE INDEX IF NOT EXISTS obligation_task_links_task_idx
    ON obligation_task_links (task_id);
```

### `backend/migrations/0064_create_decisions.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0064_create_decisions.sql`
- Size bytes / Размер в байтах: `4943`
- Included characters / Включено символов: `4943`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS decisions (
    decision_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    rationale TEXT NOT NULL,
    alternatives JSONB NOT NULL DEFAULT '[]'::jsonb,
    decided_by_entity_kind TEXT,
    decided_by_entity_id TEXT,
    decided_at TIMESTAMPTZ,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT decisions_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT decisions_rationale_not_empty CHECK (length(trim(rationale)) > 0),
    CONSTRAINT decisions_status_check CHECK (
        status IN ('active', 'superseded', 'reversed', 'deprecated')
    ),
    CONSTRAINT decisions_decider_pair_check CHECK (
        (decided_by_entity_kind IS NULL AND decided_by_entity_id IS NULL)
        OR (
            decided_by_entity_kind IS NOT NULL
            AND decided_by_entity_id IS NOT NULL
            AND decided_by_entity_kind IN (
                'persona',
                'organization',
                'project',
                'communication',
                'document',
                'task',
                'event',
                'decision',
                'obligation',
                'knowledge'
            )
            AND length(trim(decided_by_entity_id)) > 0
        )
    ),
    CONSTRAINT decisions_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT decisions_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT decisions_alternatives_is_array CHECK (jsonb_typeof(alternatives) = 'array'),
    CONSTRAINT decisions_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS decisions_status_idx
    ON decisions (status, updated_at DESC);
CREATE INDEX IF NOT EXISTS decisions_review_state_idx
    ON decisions (review_state, updated_at DESC);
CREATE INDEX IF NOT EXISTS decisions_decider_idx
    ON decisions (decided_by_entity_kind, decided_by_entity_id, decided_at DESC)
    WHERE decided_by_entity_kind IS NOT NULL;
CREATE INDEX IF NOT EXISTS decisions_decided_at_idx
    ON decisions (decided_at DESC)
    WHERE decided_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS decision_evidence (
    evidence_id TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL REFERENCES decisions(decision_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    quote TEXT,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT decision_evidence_source_kind_check CHECK (
        source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    ),
    CONSTRAINT decision_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT decision_evidence_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT decision_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (decision_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS decision_evidence_decision_idx
    ON decision_evidence (decision_id);
CREATE INDEX IF NOT EXISTS decision_evidence_source_idx
    ON decision_evidence (source_kind, source_id);

CREATE TABLE IF NOT EXISTS decision_impacted_entities (
    decision_id TEXT NOT NULL REFERENCES decisions(decision_id) ON DELETE CASCADE,
    entity_kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    impact_type TEXT NOT NULL DEFAULT 'related',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT decision_impacted_entities_entity_kind_check CHECK (
        entity_kind IN (
            'persona',
            'organization',
            'project',
            'communication',
            'document',
            'task',
            'event',
            'decision',
            'obligation',
            'knowledge'
        )
    ),
    CONSTRAINT decision_impacted_entities_entity_id_not_empty CHECK (length(trim(entity_id)) > 0),
    CONSTRAINT decision_impacted_entities_impact_type_not_empty CHECK (length(trim(impact_type)) > 0),
    CONSTRAINT decision_impacted_entities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    PRIMARY KEY (decision_id, entity_kind, entity_id)
);

CREATE INDEX IF NOT EXISTS decision_impacted_entities_entity_idx
    ON decision_impacted_entities (entity_kind, entity_id);
```

### `backend/migrations/0065_decision_graph_projection.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0065_decision_graph_projection.sql`
- Size bytes / Размер в байтах: `610`
- Included characters / Включено символов: `610`
- Truncated / Обрезано: `no`

```text
ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN (
        'person',
        'email_address',
        'message',
        'document',
        'project',
        'decision'
    )
);

ALTER TABLE graph_evidence DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;
ALTER TABLE graph_evidence
ADD CONSTRAINT graph_evidence_source_kind CHECK (
    source_kind IN (
        'contact',
        'person',
        'message',
        'document',
        'raw_record',
        'relationship',
        'decision'
    )
);
```

### `backend/migrations/0066_obligation_graph_projection.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0066_obligation_graph_projection.sql`
- Size bytes / Размер в байтах: `654`
- Included characters / Включено символов: `654`
- Truncated / Обрезано: `no`

```text
ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN (
        'person',
        'email_address',
        'message',
        'document',
        'project',
        'decision',
        'obligation'
    )
);

ALTER TABLE graph_evidence DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;
ALTER TABLE graph_evidence
ADD CONSTRAINT graph_evidence_source_kind CHECK (
    source_kind IN (
        'contact',
        'person',
        'message',
        'document',
        'raw_record',
        'relationship',
        'decision',
        'obligation'
    )
);
```

### `backend/migrations/0067_task_candidate_kind_metadata.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0067_task_candidate_kind_metadata.sql`
- Size bytes / Размер в байтах: `1020`
- Included characters / Включено символов: `1020`
- Truncated / Обрезано: `no`

```text
ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS candidate_kind TEXT NOT NULL DEFAULT 'task';

ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS candidate_metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'task_candidates_candidate_kind_check'
    ) THEN
        ALTER TABLE task_candidates
            ADD CONSTRAINT task_candidates_candidate_kind_check
            CHECK (candidate_kind IN ('task', 'obligation_task'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'task_candidates_candidate_metadata_is_object'
    ) THEN
        ALTER TABLE task_candidates
            ADD CONSTRAINT task_candidates_candidate_metadata_is_object
            CHECK (jsonb_typeof(candidate_metadata) = 'object');
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS task_candidates_candidate_kind_idx
    ON task_candidates (candidate_kind, review_state, updated_at DESC);
```

### `backend/migrations/0068_expand_relationship_graph_node_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0068_expand_relationship_graph_node_kinds.sql`
- Size bytes / Размер в байтах: `380`
- Included characters / Включено символов: `380`
- Truncated / Обрезано: `no`

```text
ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN (
        'person',
        'email_address',
        'message',
        'document',
        'project',
        'organization',
        'task',
        'event',
        'decision',
        'obligation',
        'knowledge'
    )
);
```

### `backend/migrations/0069_relax_task_candidate_requirement.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0069_relax_task_candidate_requirement.sql`
- Size bytes / Размер в байтах: `858`
- Included characters / Включено символов: `858`
- Truncated / Обрезано: `no`

```text
ALTER TABLE tasks
    ALTER COLUMN task_candidate_id DROP NOT NULL;

ALTER TABLE tasks
    ALTER COLUMN created_from_event_id DROP NOT NULL;

ALTER TABLE tasks
    ALTER COLUMN created_by_actor_id DROP NOT NULL;

ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_kind_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_kind_check CHECK (
        source_kind IN (
            'manual',
            'message',
            'email',
            'telegram',
            'whatsapp',
            'calendar',
            'meeting',
            'document',
            'note',
            'jira',
            'youtrack',
            'github',
            'gitlab',
            'linear',
            'todoist',
            'apple_reminders',
            'ms_todo',
            'ai_rule',
            'workflow',
            'import'
        )
    );
```

### `backend/migrations/0070_ai_run_persona_attribution.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0070_ai_run_persona_attribution.sql`
- Size bytes / Размер в байтах: `542`
- Included characters / Включено символов: `542`
- Truncated / Обрезано: `no`

```text
ALTER TABLE ai_agent_runs
    ADD COLUMN IF NOT EXISTS agent_persona_id TEXT REFERENCES persons(person_id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS owner_persona_id TEXT REFERENCES persons(person_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS ai_agent_runs_agent_persona_idx
    ON ai_agent_runs (agent_persona_id, started_at DESC)
    WHERE agent_persona_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS ai_agent_runs_owner_persona_idx
    ON ai_agent_runs (owner_persona_id, started_at DESC)
    WHERE owner_persona_id IS NOT NULL;
```

### `backend/migrations/0071_person_identity_trace_types.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0071_person_identity_trace_types.sql`
- Size bytes / Размер в байтах: `439`
- Included characters / Включено символов: `439`
- Truncated / Обрезано: `no`

```text
ALTER TABLE person_identities
    DROP CONSTRAINT IF EXISTS person_identities_type_check;

ALTER TABLE person_identities
    ADD CONSTRAINT person_identities_type_check CHECK (identity_type IN (
        'email', 'telegram', 'whatsapp', 'phone',
        'github', 'linkedin', 'website',
        'mastodon', 'x', 'stackoverflow', 'habr',
        'medium', 'orcid', 'google_scholar',
        'document_mention', 'message_participant'
    ));
```

### `backend/migrations/0072_person_identity_disputed_status.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0072_person_identity_disputed_status.sql`
- Size bytes / Размер в байтах: `267`
- Included characters / Включено символов: `267`
- Truncated / Обрезано: `no`

```text
ALTER TABLE person_identities
    DROP CONSTRAINT IF EXISTS person_identities_status_check;

ALTER TABLE person_identities
    ADD CONSTRAINT person_identities_status_check CHECK (status IN (
        'active', 'outdated', 'unreachable', 'blocked', 'disputed'
    ));
```

### `backend/migrations/0073_person_identity_unattached_traces.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0073_person_identity_unattached_traces.sql`
- Size bytes / Размер в байтах: `72`
- Included characters / Включено символов: `72`
- Truncated / Обрезано: `no`

```text
ALTER TABLE person_identities
    ALTER COLUMN person_id DROP NOT NULL;
```

### `backend/migrations/0074_persona_dossier_snapshots.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0074_persona_dossier_snapshots.sql`
- Size bytes / Размер в байтах: `1468`
- Included characters / Включено символов: `1468`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS persona_dossier_snapshots (
    dossier_snapshot_id TEXT PRIMARY KEY,
    persona_id TEXT NOT NULL,
    dossier JSONB NOT NULL,
    source_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    generated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT persona_dossier_snapshots_persona_id_not_empty CHECK (length(trim(persona_id)) > 0),
    CONSTRAINT persona_dossier_snapshots_dossier_is_object CHECK (jsonb_typeof(dossier) = 'object'),
    CONSTRAINT persona_dossier_snapshots_source_refs_is_array CHECK (jsonb_typeof(source_refs) = 'array'),
    CONSTRAINT persona_dossier_snapshots_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT persona_dossier_snapshots_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT persona_dossier_snapshots_reviewed_by_not_empty CHECK (
        reviewed_by IS NULL OR length(trim(reviewed_by)) > 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS persona_dossier_snapshots_persona_latest_unique
    ON persona_dossier_snapshots (persona_id);

CREATE INDEX IF NOT EXISTS persona_dossier_snapshots_review_state_idx
    ON persona_dossier_snapshots (review_state, updated_at DESC);
```

### `backend/migrations/0075_allow_empty_email_draft_subject.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0075_allow_empty_email_draft_subject.sql`
- Size bytes / Размер в байтах: `87`
- Included characters / Включено символов: `87`
- Truncated / Обрезано: `no`

```text
ALTER TABLE email_drafts
    DROP CONSTRAINT IF EXISTS email_drafts_subject_not_empty;
```
