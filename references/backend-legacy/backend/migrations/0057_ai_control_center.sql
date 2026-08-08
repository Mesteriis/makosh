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
