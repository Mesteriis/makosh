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
    )
ON CONFLICT (setting_key) DO NOTHING;
