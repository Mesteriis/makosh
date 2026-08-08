CREATE TABLE IF NOT EXISTS application_settings (
    setting_key TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    value_kind TEXT NOT NULL,
    value JSONB NOT NULL,
    label TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_editable BOOLEAN NOT NULL DEFAULT true,
    updated_by_actor_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT application_settings_key_not_empty CHECK (length(trim(setting_key)) > 0),
    CONSTRAINT application_settings_key_format CHECK (setting_key ~ '^[a-z0-9][a-z0-9_.-]*[a-z0-9]$'),
    CONSTRAINT application_settings_key_not_secret_like CHECK (
        setting_key !~* '(secret|password|token|credential|private_key)'
    ),
    CONSTRAINT application_settings_category_not_empty CHECK (length(trim(category)) > 0),
    CONSTRAINT application_settings_label_not_empty CHECK (length(trim(label)) > 0),
    CONSTRAINT application_settings_value_kind CHECK (
        value_kind IN ('boolean', 'integer', 'string', 'json')
    ),
    CONSTRAINT application_settings_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS application_settings_category_idx
    ON application_settings (category, setting_key);

INSERT INTO application_settings (
    setting_key,
    category,
    value_kind,
    value,
    label,
    description,
    metadata
)
VALUES
    (
        'server.http_addr',
        'server',
        'string',
        '"127.0.0.1:8080"'::jsonb,
        'Backend HTTP bind',
        'Backend HTTP address used when the local server starts. Changes require a backend restart.',
        '{"ui_control":"text","placeholder":"127.0.0.1:8080","restart_required":true,"bootstrap":true,"env_var":"MAKOSH_HTTP_ADDR"}'::jsonb
    ),
    (
        'frontend.api_base_url',
        'frontend',
        'string',
        '"http://127.0.0.1:8080"'::jsonb,
        'Frontend API base URL',
        'Backend URL used by the desktop shell after it has loaded local settings.',
        '{"ui_control":"text","placeholder":"http://127.0.0.1:8080","bootstrap":true,"env_var":"VITE_MAKOSH_API_BASE_URL"}'::jsonb
    ),
    (
        'frontend.actor_id',
        'frontend',
        'string',
        '"desktop-shell"'::jsonb,
        'Frontend actor ID',
        'Non-secret local actor identifier sent with protected API requests for audit records.',
        '{"ui_control":"text","placeholder":"desktop-shell","env_var":"VITE_MAKOSH_ACTOR_ID"}'::jsonb
    ),
    (
        'ai.ollama_base_url',
        'ai',
        'string',
        '"http://127.0.0.1:11434"'::jsonb,
        'Ollama base URL',
        'Local Ollama HTTP endpoint used by AI runtime requests.',
        '{"ui_control":"text","placeholder":"http://127.0.0.1:11434"}'::jsonb
    ),
    (
        'ai.chat_model',
        'ai',
        'string',
        '"qwen3:4b"'::jsonb,
        'Chat model',
        'Ollama model used for chat and source-backed answers.',
        '{"ui_control":"text","placeholder":"qwen3:4b"}'::jsonb
    ),
    (
        'ai.embedding_model',
        'ai',
        'string',
        '"qwen3-embedding:4b"'::jsonb,
        'Embedding model',
        'Ollama model used for semantic embeddings.',
        '{"ui_control":"text","placeholder":"qwen3-embedding:4b"}'::jsonb
    ),
    (
        'ai.timeout_seconds',
        'ai',
        'integer',
        '120'::jsonb,
        'AI request timeout',
        'Timeout in seconds for Ollama HTTP requests.',
        '{"ui_control":"number","min":1,"max":600,"step":1}'::jsonb
    ),
    (
        'ui.theme',
        'ui',
        'string',
        '"system"'::jsonb,
        'Theme',
        'Desktop shell color theme preference.',
        '{"ui_control":"select","allowed_values":["system","dark","light"]}'::jsonb
    ),
    (
        'ui.density',
        'ui',
        'string',
        '"comfortable"'::jsonb,
        'UI density',
        'Desktop shell spacing density preference.',
        '{"ui_control":"select","allowed_values":["comfortable","compact"]}'::jsonb
    )
ON CONFLICT (setting_key) DO NOTHING;
