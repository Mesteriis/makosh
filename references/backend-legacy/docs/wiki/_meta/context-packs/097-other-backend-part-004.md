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

- Chunk ID / ID чанка: `097-other-backend-part-004`
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

### `backend/migrations/0076_create_email_outbox_tracking.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0076_create_email_outbox_tracking.sql`
- Size bytes / Размер в байтах: `1732`
- Included characters / Включено символов: `1732`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS email_outbox_tracking (
    outbox_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    draft_id TEXT REFERENCES email_drafts(draft_id) ON DELETE SET NULL,
    to_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    cc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    bcc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    subject TEXT NOT NULL DEFAULT '',
    body_text TEXT NOT NULL DEFAULT '',
    body_html TEXT,
    status TEXT NOT NULL,
    scheduled_send_at TIMESTAMPTZ,
    undo_deadline_at TIMESTAMPTZ,
    send_attempts INTEGER NOT NULL DEFAULT 0,
    claimed_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    provider_message_id TEXT,
    last_error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_outbox_tracking_status CHECK (
        status IN ('queued', 'scheduled', 'sending', 'sent', 'failed', 'canceled')
    ),
    CONSTRAINT email_outbox_tracking_to_is_array CHECK (jsonb_typeof(to_recipients) = 'array'),
    CONSTRAINT email_outbox_tracking_cc_is_array CHECK (jsonb_typeof(cc_recipients) = 'array'),
    CONSTRAINT email_outbox_tracking_bcc_is_array CHECK (jsonb_typeof(bcc_recipients) = 'array'),
    CONSTRAINT email_outbox_tracking_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS email_outbox_tracking_account_status_idx
    ON email_outbox_tracking (account_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS email_outbox_tracking_due_idx
    ON email_outbox_tracking (status, scheduled_send_at, undo_deadline_at, created_at);
```

### `backend/migrations/0077_create_mail_saved_searches.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0077_create_mail_saved_searches.sql`
- Size bytes / Размер в байтах: `1848`
- Included characters / Включено символов: `1848`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS mail_saved_searches (
    saved_search_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    account_id TEXT REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    query_text TEXT NOT NULL DEFAULT '',
    workflow_state TEXT,
    local_state TEXT NOT NULL DEFAULT 'active',
    channel_kind TEXT,
    is_smart_folder BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_saved_searches_id_not_empty CHECK (length(trim(saved_search_id)) > 0),
    CONSTRAINT mail_saved_searches_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT mail_saved_searches_query_not_blank_or_filters CHECK (
        length(trim(query_text)) > 0
        OR workflow_state IS NOT NULL
        OR channel_kind IS NOT NULL
        OR account_id IS NOT NULL
        OR local_state <> 'active'
    ),
    CONSTRAINT mail_saved_searches_workflow_state CHECK (
        workflow_state IS NULL
        OR workflow_state IN ('new', 'reviewed', 'needs_action', 'waiting', 'done', 'archived', 'muted', 'spam')
    ),
    CONSTRAINT mail_saved_searches_local_state CHECK (local_state IN ('active', 'trash', 'all')),
    CONSTRAINT mail_saved_searches_description_not_blank CHECK (
        description IS NULL OR length(trim(description)) > 0
    ),
    CONSTRAINT mail_saved_searches_channel_kind_not_blank CHECK (
        channel_kind IS NULL OR length(trim(channel_kind)) > 0
    )
);

CREATE INDEX IF NOT EXISTS mail_saved_searches_account_smart_idx
    ON mail_saved_searches (account_id, is_smart_folder, sort_order, lower(name));

CREATE INDEX IF NOT EXISTS mail_saved_searches_smart_idx
    ON mail_saved_searches (is_smart_folder, sort_order, lower(name));
```

### `backend/migrations/0078_add_attachment_search_indexes.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0078_add_attachment_search_indexes.sql`
- Size bytes / Размер в байтах: `413`
- Included characters / Включено символов: `413`
- Truncated / Обрезано: `no`

```text
CREATE INDEX IF NOT EXISTS communication_attachments_search_order_idx
    ON communication_attachments (created_at DESC, attachment_id ASC);

CREATE INDEX IF NOT EXISTS communication_attachments_scan_status_idx
    ON communication_attachments (scan_status, created_at DESC);

CREATE INDEX IF NOT EXISTS communication_attachments_content_type_idx
    ON communication_attachments (content_type, created_at DESC);
```

### `backend/migrations/0079_create_mail_custom_folders.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0079_create_mail_custom_folders.sql`
- Size bytes / Размер в байтах: `1738`
- Included characters / Включено символов: `1738`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS mail_folders (
    folder_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_folders_id_not_empty CHECK (length(trim(folder_id)) > 0),
    CONSTRAINT mail_folders_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT mail_folders_description_not_empty CHECK (
        description IS NULL OR length(trim(description)) > 0
    ),
    CONSTRAINT mail_folders_color_not_empty CHECK (
        color IS NULL OR length(trim(color)) > 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS mail_folders_account_name_unique_idx
    ON mail_folders (COALESCE(account_id, ''), lower(name));

CREATE INDEX IF NOT EXISTS mail_folders_account_order_idx
    ON mail_folders (account_id, sort_order, lower(name), folder_id);

CREATE TABLE IF NOT EXISTS mail_folder_messages (
    folder_id TEXT NOT NULL REFERENCES mail_folders(folder_id) ON DELETE CASCADE,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_operation TEXT NOT NULL DEFAULT 'copy',

    PRIMARY KEY (folder_id, message_id),
    CONSTRAINT mail_folder_messages_operation CHECK (last_operation IN ('copy', 'move'))
);

CREATE INDEX IF NOT EXISTS mail_folder_messages_message_idx
    ON mail_folder_messages (message_id, added_at DESC);

CREATE INDEX IF NOT EXISTS mail_folder_messages_folder_order_idx
    ON mail_folder_messages (folder_id, added_at DESC, message_id);
```

### `backend/migrations/0080_create_mail_ai_states.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0080_create_mail_ai_states.sql`
- Size bytes / Размер в байтах: `1056`
- Included characters / Включено символов: `1056`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS mail_ai_states (
    message_id TEXT PRIMARY KEY REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    ai_state TEXT NOT NULL DEFAULT 'NEW',
    review_reason TEXT,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_ai_states_state CHECK (
        ai_state IN ('NEW', 'PROCESSING', 'PROCESSED', 'REVIEW_REQUIRED', 'FAILED', 'ARCHIVED')
    ),
    CONSTRAINT mail_ai_states_review_reason_not_blank CHECK (
        review_reason IS NULL OR length(trim(review_reason)) > 0
    ),
    CONSTRAINT mail_ai_states_last_error_not_blank CHECK (
        last_error IS NULL OR length(trim(last_error)) > 0
    )
);

CREATE INDEX IF NOT EXISTS mail_ai_states_state_updated_idx
    ON mail_ai_states (ai_state, updated_at DESC, message_id);

INSERT INTO mail_ai_states (message_id, ai_state, created_at, updated_at)
SELECT message_id, 'NEW', projected_at, projected_at
FROM communication_messages
ON CONFLICT (message_id) DO NOTHING;
```

### `backend/migrations/0081_create_mail_read_receipts.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0081_create_mail_read_receipts.sql`
- Size bytes / Размер в байтах: `1856`
- Included characters / Включено символов: `1856`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS mail_read_receipts (
    receipt_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    outbox_id TEXT REFERENCES email_outbox_tracking(outbox_id) ON DELETE SET NULL,
    provider_message_id TEXT NOT NULL,
    recipient TEXT NOT NULL,
    receipt_kind TEXT NOT NULL DEFAULT 'read',
    read_at TIMESTAMPTZ NOT NULL,
    source_kind TEXT NOT NULL DEFAULT 'mdn',
    provider_record_id TEXT,
    raw_record_id TEXT REFERENCES communication_raw_records(raw_record_id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_read_receipts_id_not_empty CHECK (length(trim(receipt_id)) > 0),
    CONSTRAINT mail_read_receipts_provider_message_not_empty CHECK (
        length(trim(provider_message_id)) > 0
    ),
    CONSTRAINT mail_read_receipts_recipient_not_empty CHECK (length(trim(recipient)) > 0),
    CONSTRAINT mail_read_receipts_kind CHECK (receipt_kind IN ('read')),
    CONSTRAINT mail_read_receipts_source_kind_not_empty CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT mail_read_receipts_provider_record_not_empty CHECK (
        provider_record_id IS NULL OR length(trim(provider_record_id)) > 0
    ),
    CONSTRAINT mail_read_receipts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS mail_read_receipts_provider_record_unique_idx
    ON mail_read_receipts (account_id, provider_record_id)
    WHERE provider_record_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS mail_read_receipts_outbox_read_at_idx
    ON mail_read_receipts (outbox_id, read_at DESC, receipt_id);

CREATE INDEX IF NOT EXISTS mail_read_receipts_provider_message_idx
    ON mail_read_receipts (account_id, provider_message_id, read_at DESC);
```

### `backend/migrations/0082_create_telegram_message_lifecycle_schema.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0082_create_telegram_message_lifecycle_schema.sql`
- Size bytes / Размер в байтах: `9016`
- Included characters / Включено символов: `9010`
- Truncated / Обрезано: `no`

```text
-- Migration 0082: Telegram message lifecycle schema
-- ADR-0091: version history, tombstones and provider-write command model

-- ---------------------------------------------------------------------------
-- telegram_message_versions — append-only observed edit version history
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_versions (
    version_id          TEXT PRIMARY KEY,
    message_id          TEXT NOT NULL,
    account_id          TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_message_id TEXT NOT NULL,
    provider_chat_id    TEXT NOT NULL,
    version_number      INTEGER NOT NULL,
    body_text           TEXT,
    edit_timestamp      TIMESTAMPTZ NOT NULL,
    source_event        TEXT,
    raw_diff_payload    JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_versions_message_id_not_empty
        CHECK (length(trim(message_id)) > 0),
    CONSTRAINT telegram_message_versions_provider_message_id_not_empty
        CHECK (length(trim(provider_message_id)) > 0),
    CONSTRAINT telegram_message_versions_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_message_versions_version_positive
        CHECK (version_number > 0),
    CONSTRAINT telegram_message_versions_raw_diff_is_object
        CHECK (jsonb_typeof(raw_diff_payload) = 'object'),
    CONSTRAINT telegram_message_versions_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_versions_message_version_unique
        UNIQUE (message_id, version_number)
);

CREATE INDEX IF NOT EXISTS telegram_message_versions_message_idx
    ON telegram_message_versions (message_id, version_number DESC);

CREATE INDEX IF NOT EXISTS telegram_message_versions_account_provider_idx
    ON telegram_message_versions (account_id, provider_chat_id, provider_message_id);

-- ---------------------------------------------------------------------------
-- telegram_message_tombstones — local visibility and delete evidence
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_tombstones (
    tombstone_id        TEXT PRIMARY KEY,
    message_id          TEXT NOT NULL,
    account_id          TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_message_id TEXT NOT NULL,
    provider_chat_id    TEXT NOT NULL,
    reason_class        TEXT NOT NULL,
    actor_class         TEXT NOT NULL,
    observed_at         TIMESTAMPTZ NOT NULL,
    source_event        TEXT,
    is_provider_delete  BOOLEAN NOT NULL DEFAULT false,
    is_local_visible    BOOLEAN NOT NULL DEFAULT true,
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_tombstones_message_id_not_empty
        CHECK (length(trim(message_id)) > 0),
    CONSTRAINT telegram_message_tombstones_provider_message_id_not_empty
        CHECK (length(trim(provider_message_id)) > 0),
    CONSTRAINT telegram_message_tombstones_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_message_tombstones_reason_class
        CHECK (reason_class IN (
            'deleted_by_owner',
            'deleted_by_counterparty',
            'deleted_by_provider',
            'moderation_removed',
            'account_removed',
            'retention_policy',
            'unknown'
        )),
    CONSTRAINT telegram_message_tombstones_actor_class
        CHECK (actor_class IN (
            'owner',
            'provider',
            'automation',
            'system',
            'unknown'
        )),
    CONSTRAINT telegram_message_tombstones_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_tombstones_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS telegram_message_tombstones_message_idx
    ON telegram_message_tombstones (message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_tombstones_account_idx
    ON telegram_message_tombstones (account_id, provider_chat_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- telegram_provider_write_commands — durable provider-write command model
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_provider_write_commands (
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
    completed_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_provider_write_commands_command_id_not_empty
        CHECK (length(trim(command_id)) > 0),
    CONSTRAINT telegram_provider_write_commands_idempotency_key_not_empty
        CHECK (length(trim(idempotency_key)) > 0),
    CONSTRAINT telegram_provider_write_commands_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_provider_write_commands_actor_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'admin_action'
        )),
    CONSTRAINT telegram_provider_write_commands_capability_state
        CHECK (capability_state IN ('available', 'blocked', 'degraded', 'unsupported')),
    CONSTRAINT telegram_provider_write_commands_action_class
        CHECK (action_class IN ('read', 'local_write', 'provider_write', 'destructive', 'export', 'secret_access', 'automation')),
    CONSTRAINT telegram_provider_write_commands_confirmation_decision
        CHECK (confirmation_decision IN ('pending', 'confirmed', 'rejected', 'not_required')),
    CONSTRAINT telegram_provider_write_commands_status
        CHECK (status IN ('queued', 'executing', 'completed', 'failed', 'retrying', 'cancelled')),
    CONSTRAINT telegram_provider_write_commands_retry_count_non_negative
        CHECK (retry_count >= 0),
    CONSTRAINT telegram_provider_write_commands_max_retries_positive
        CHECK (max_retries > 0),
    CONSTRAINT telegram_provider_write_commands_target_ref_is_object
        CHECK (jsonb_typeof(target_ref) = 'object'),
    CONSTRAINT telegram_provider_write_commands_payload_is_object
        CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT telegram_provider_write_commands_result_payload_is_object
        CHECK (jsonb_typeof(result_payload) = 'object'),
    CONSTRAINT telegram_provider_write_commands_audit_metadata_is_object
        CHECK (jsonb_typeof(audit_metadata) = 'object'),
    CONSTRAINT telegram_provider_write_commands_idempotency_unique
        UNIQUE (account_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_account_idx
    ON telegram_provider_write_commands (account_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_chat_idx
    ON telegram_provider_write_commands (account_id, provider_chat_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_idempotency_idx
    ON telegram_provider_write_commands (idempotency_key);
```

### `backend/migrations/0083_create_telegram_reactions.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0083_create_telegram_reactions.sql`
- Size bytes / Размер в байтах: `2236`
- Included characters / Включено символов: `2236`
- Truncated / Обрезано: `no`

```text
-- Migration 0083: Telegram message reactions
-- ADR-0091: reaction add/remove/sync with source-backed projection

CREATE TABLE IF NOT EXISTS telegram_message_reactions (
    reaction_id             TEXT PRIMARY KEY,
    message_id              TEXT NOT NULL,
    account_id              TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_message_id     TEXT NOT NULL,
    provider_chat_id        TEXT NOT NULL,
    sender_id               TEXT NOT NULL,
    sender_display_name     TEXT,
    reaction_emoji          TEXT NOT NULL,
    is_active               BOOLEAN NOT NULL DEFAULT true,
    observed_at             TIMESTAMPTZ NOT NULL,
    source_event            TEXT,
    provider_actor_id       TEXT,
    metadata                JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance              JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_reactions_message_id_not_empty
        CHECK (length(trim(message_id)) > 0),
    CONSTRAINT telegram_message_reactions_provider_message_id_not_empty
        CHECK (length(trim(provider_message_id)) > 0),
    CONSTRAINT telegram_message_reactions_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_message_reactions_sender_id_not_empty
        CHECK (length(trim(sender_id)) > 0),
    CONSTRAINT telegram_message_reactions_emoji_not_empty
        CHECK (length(trim(reaction_emoji)) > 0),
    CONSTRAINT telegram_message_reactions_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_reactions_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_reactions_unique_active
        UNIQUE (message_id, sender_id, reaction_emoji)
);

CREATE INDEX IF NOT EXISTS telegram_message_reactions_message_idx
    ON telegram_message_reactions (message_id, is_active, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_reactions_account_idx
    ON telegram_message_reactions (account_id, provider_chat_id, provider_message_id);
```

### `backend/migrations/0084_create_telegram_reply_forward_refs.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0084_create_telegram_reply_forward_refs.sql`
- Size bytes / Размер в байтах: `4007`
- Included characters / Включено символов: `4003`
- Truncated / Обрезано: `no`

```text
-- Migration 0084: Telegram reply and forward reference tracking
-- ADR-0091: reply targets, reply chains, forward attribution, forward chains

-- ---------------------------------------------------------------------------
-- telegram_message_reply_refs — reply target and reply chain
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_reply_refs (
    reply_ref_id        TEXT PRIMARY KEY,
    source_message_id   TEXT NOT NULL,
    target_message_id   TEXT NOT NULL,
    account_id          TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id    TEXT NOT NULL,
    source_provider_id  TEXT NOT NULL,
    target_provider_id  TEXT NOT NULL,
    reply_depth         INTEGER NOT NULL DEFAULT 1,
    is_topic_reply      BOOLEAN NOT NULL DEFAULT false,
    topic_id            TEXT,
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_reply_refs_source_not_empty
        CHECK (length(trim(source_message_id)) > 0),
    CONSTRAINT telegram_message_reply_refs_target_not_empty
        CHECK (length(trim(target_message_id)) > 0),
    CONSTRAINT telegram_message_reply_refs_reply_depth_positive
        CHECK (reply_depth > 0),
    CONSTRAINT telegram_message_reply_refs_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_reply_refs_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_reply_refs_unique
        UNIQUE (source_message_id, target_message_id)
);

CREATE INDEX IF NOT EXISTS telegram_message_reply_refs_target_idx
    ON telegram_message_reply_refs (target_message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_reply_refs_source_idx
    ON telegram_message_reply_refs (source_message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_reply_refs_chat_idx
    ON telegram_message_reply_refs (account_id, provider_chat_id);

-- ---------------------------------------------------------------------------
-- telegram_message_forward_refs — forward attribution and chains
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_forward_refs (
    forward_ref_id          TEXT PRIMARY KEY,
    source_message_id       TEXT NOT NULL,
    account_id              TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id        TEXT NOT NULL,
    source_provider_id      TEXT NOT NULL,
    forward_origin_chat_id  TEXT,
    forward_origin_message_id TEXT,
    forward_origin_sender_id TEXT,
    forward_origin_sender_name TEXT,
    forward_date            TIMESTAMPTZ,
    forward_depth           INTEGER NOT NULL DEFAULT 1,
    metadata                JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance              JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_forward_refs_source_not_empty
        CHECK (length(trim(source_message_id)) > 0),
    CONSTRAINT telegram_message_forward_refs_forward_depth_positive
        CHECK (forward_depth > 0),
    CONSTRAINT telegram_message_forward_refs_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_forward_refs_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_forward_refs_unique
        UNIQUE (source_message_id, account_id)
);

CREATE INDEX IF NOT EXISTS telegram_message_forward_refs_source_idx
    ON telegram_message_forward_refs (source_message_id);

CREATE INDEX IF NOT EXISTS telegram_message_forward_refs_chat_idx
    ON telegram_message_forward_refs (account_id, provider_chat_id);
```

### `backend/migrations/0085_allow_mark_unread_telegram_command.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0085_allow_mark_unread_telegram_command.sql`
- Size bytes / Размер в байтах: `807`
- Included characters / Включено символов: `807`
- Truncated / Обрезано: `no`

```text
-- Migration 0085: allow local dialog mark_unread command rows

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_command_kind;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'mark_unread',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'admin_action'
        ));
```

### `backend/migrations/0086_create_telegram_topics.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0086_create_telegram_topics.sql`
- Size bytes / Размер в байтах: `1516`
- Included characters / Включено символов: `1516`
- Truncated / Обрезано: `no`

```text
-- Forum topics for supergroup/channel chats with forum mode enabled.
-- Each topic maps to a TDLib forumTopic with a stable provider_topic_id (BIGINT).
-- Messages belong to a topic via metadata->>'forum_topic_id'; see ADR-0091.

CREATE TABLE IF NOT EXISTS telegram_topics (
    topic_id            TEXT PRIMARY KEY,
    telegram_chat_id    TEXT NOT NULL REFERENCES telegram_chats(telegram_chat_id) ON DELETE CASCADE,
    account_id          TEXT NOT NULL,
    provider_topic_id   BIGINT NOT NULL,
    provider_chat_id    TEXT NOT NULL,
    title               TEXT NOT NULL,
    icon_emoji          TEXT,
    is_pinned           BOOLEAN NOT NULL DEFAULT FALSE,
    is_closed           BOOLEAN NOT NULL DEFAULT FALSE,
    unread_count        INT NOT NULL DEFAULT 0,
    last_message_at     TIMESTAMPTZ,
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (telegram_chat_id, provider_topic_id)
);

CREATE INDEX IF NOT EXISTS idx_telegram_topics_chat
    ON telegram_topics (telegram_chat_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_telegram_topics_account
    ON telegram_topics (account_id, updated_at DESC);

-- Index for message-per-topic queries via message_metadata JSONB
CREATE INDEX IF NOT EXISTS idx_comm_messages_forum_topic_id
    ON communication_messages ((message_metadata->>'forum_topic_id'))
    WHERE message_metadata->>'forum_topic_id' IS NOT NULL;
```

### `backend/migrations/0087_extend_telegram_provider_write_outbox.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0087_extend_telegram_provider_write_outbox.sql`
- Size bytes / Размер в байтах: `2645`
- Included characters / Включено символов: `2645`
- Truncated / Обрезано: `no`

```text
-- Migration 0087: Telegram provider-write outbox runtime and reconciliation state
-- ADR-0091: provider writes must be durable, retryable and completed only after
-- provider-observed state is recorded.

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_status;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_status
        CHECK (status IN (
            'queued',
            'executing',
            'completed',
            'failed',
            'retrying',
            'cancelled',
            'dead_letter'
        ));

ALTER TABLE telegram_provider_write_commands
    ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_by TEXT,
    ADD COLUMN IF NOT EXISTS provider_observed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS reconciliation_status TEXT NOT NULL DEFAULT 'not_observed',
    ADD COLUMN IF NOT EXISTS reconciled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dead_lettered_at TIMESTAMPTZ;

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_provider_state_is_object;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_provider_state_is_object
        CHECK (jsonb_typeof(provider_state) = 'object');

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_reconciliation_status;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_reconciliation_status
        CHECK (reconciliation_status IN (
            'not_observed',
            'awaiting_provider',
            'observed',
            'mismatch',
            'not_required'
        ));

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_locked_by_not_empty;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_locked_by_not_empty
        CHECK (locked_by IS NULL OR length(trim(locked_by)) > 0);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_due_idx
    ON telegram_provider_write_commands (account_id, status, next_attempt_at, created_at);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_reconciliation_idx
    ON telegram_provider_write_commands (account_id, reconciliation_status, updated_at DESC);
```

### `backend/migrations/0088_create_communication_attachment_imports.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0088_create_communication_attachment_imports.sql`
- Size bytes / Размер в байтах: `2813`
- Included characters / Включено символов: `2813`
- Truncated / Обрезано: `no`

```text
-- Migration 0088: Provider-neutral local attachment imports for composer/media upload.
-- Imported rows reference local Communication blobs before a provider message exists.

CREATE TABLE IF NOT EXISTS communication_attachment_imports (
    attachment_id TEXT PRIMARY KEY,
    account_id TEXT,
    channel_kind TEXT,
    blob_id TEXT NOT NULL REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    filename TEXT,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 TEXT NOT NULL,
    source_kind TEXT NOT NULL DEFAULT 'local_import',
    imported_by TEXT NOT NULL,
    scan_status TEXT NOT NULL DEFAULT 'not_scanned',
    scan_engine TEXT,
    scan_checked_at TIMESTAMPTZ,
    scan_summary TEXT,
    scan_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT communication_attachment_imports_attachment_id_not_empty
        CHECK (length(trim(attachment_id)) > 0),
    CONSTRAINT communication_attachment_imports_account_id_not_empty
        CHECK (account_id IS NULL OR length(trim(account_id)) > 0),
    CONSTRAINT communication_attachment_imports_channel_kind_not_empty
        CHECK (channel_kind IS NULL OR length(trim(channel_kind)) > 0),
    CONSTRAINT communication_attachment_imports_filename_not_empty
        CHECK (filename IS NULL OR length(trim(filename)) > 0),
    CONSTRAINT communication_attachment_imports_content_type_not_empty
        CHECK (length(trim(content_type)) > 0),
    CONSTRAINT communication_attachment_imports_size_positive
        CHECK (size_bytes > 0),
    CONSTRAINT communication_attachment_imports_sha256_format
        CHECK (sha256 ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT communication_attachment_imports_source_kind_not_empty
        CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT communication_attachment_imports_imported_by_not_empty
        CHECK (length(trim(imported_by)) > 0),
    CONSTRAINT communication_attachment_imports_scan_status
        CHECK (scan_status IN ('not_scanned', 'clean', 'suspicious', 'malicious', 'failed')),
    CONSTRAINT communication_attachment_imports_scan_metadata_object
        CHECK (jsonb_typeof(scan_metadata) = 'object'),
    CONSTRAINT communication_attachment_imports_metadata_object
        CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_attachment_imports_account_idx
    ON communication_attachment_imports (account_id, created_at DESC);

CREATE INDEX IF NOT EXISTS communication_attachment_imports_blob_idx
    ON communication_attachment_imports (blob_id);

CREATE INDEX IF NOT EXISTS communication_attachment_imports_sha256_idx
    ON communication_attachment_imports (sha256);
```

### `backend/migrations/0089_create_telegram_chat_participants.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0089_create_telegram_chat_participants.sql`
- Size bytes / Размер в байтах: `2807`
- Included characters / Включено символов: `2807`
- Truncated / Обрезано: `no`

```text
-- Migration 0089: Telegram provider participant projection
-- Telegram remains a Communication Channel: participant rows are provider
-- communication projection state, not Persona/Organization lifecycle records.

CREATE TABLE IF NOT EXISTS telegram_chat_participants (
    participant_id       TEXT PRIMARY KEY,
    telegram_chat_id     TEXT NOT NULL
        REFERENCES telegram_chats(telegram_chat_id) ON DELETE CASCADE,
    account_id           TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id     TEXT NOT NULL,
    provider_member_id   TEXT NOT NULL,
    display_name         TEXT,
    username             TEXT,
    role                 TEXT NOT NULL,
    status               TEXT NOT NULL,
    is_admin             BOOLEAN NOT NULL DEFAULT false,
    is_owner             BOOLEAN NOT NULL DEFAULT false,
    permissions          JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw_payload          JSONB NOT NULL DEFAULT '{}'::jsonb,
    source               TEXT NOT NULL DEFAULT 'tdlib',
    observed_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_chat_participants_participant_id_not_empty
        CHECK (length(trim(participant_id)) > 0),
    CONSTRAINT telegram_chat_participants_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_chat_participants_provider_member_id_not_empty
        CHECK (length(trim(provider_member_id)) > 0),
    CONSTRAINT telegram_chat_participants_role_not_empty
        CHECK (length(trim(role)) > 0),
    CONSTRAINT telegram_chat_participants_status_not_empty
        CHECK (length(trim(status)) > 0),
    CONSTRAINT telegram_chat_participants_permissions_is_object
        CHECK (jsonb_typeof(permissions) = 'object'),
    CONSTRAINT telegram_chat_participants_raw_payload_is_object
        CHECK (jsonb_typeof(raw_payload) = 'object'),
    CONSTRAINT telegram_chat_participants_source
        CHECK (source IN ('tdlib', 'bot_api')),
    CONSTRAINT telegram_chat_participants_unique_provider_member
        UNIQUE (telegram_chat_id, provider_member_id)
);

CREATE INDEX IF NOT EXISTS telegram_chat_participants_chat_idx
    ON telegram_chat_participants (telegram_chat_id, role, updated_at DESC);

CREATE INDEX IF NOT EXISTS telegram_chat_participants_account_chat_idx
    ON telegram_chat_participants (account_id, provider_chat_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS telegram_chat_participants_search_idx
    ON telegram_chat_participants (
        telegram_chat_id,
        lower(coalesce(display_name, '')),
        lower(coalesce(username, '')),
        lower(provider_member_id)
    );
```

### `backend/migrations/0090_restore_topic_telegram_command_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0090_restore_topic_telegram_command_kinds.sql`
- Size bytes / Размер в байтах: `923`
- Included characters / Включено символов: `923`
- Truncated / Обрезано: `no`

```text
-- Migration 0090: restore topic provider-write command kinds after 0085 narrowed the allowlist

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_command_kind;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'mark_unread',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'topic_create',
            'topic_close',
            'topic_reopen',
            'admin_action'
        ));
```

### `backend/migrations/0091_add_telegram_folder_add_command_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0091_add_telegram_folder_add_command_kind.sql`
- Size bytes / Размер в байтах: `926`
- Included characters / Включено символов: `926`
- Truncated / Обрезано: `no`

```text
-- Migration 0091: allow Telegram folder-add provider-write command kind

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_command_kind;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'mark_unread',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'folder_add',
            'topic_create',
            'topic_close',
            'topic_reopen',
            'admin_action'
        ));
```

### `backend/migrations/0092_add_telegram_folder_remove_command_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0092_add_telegram_folder_remove_command_kind.sql`
- Size bytes / Размер в байтах: `958`
- Included characters / Включено символов: `958`
- Truncated / Обрезано: `no`

```text
-- Migration 0092: allow Telegram folder-remove provider-write command kind

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_command_kind;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'mark_unread',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'folder_add',
            'folder_remove',
            'topic_create',
            'topic_close',
            'topic_reopen',
            'admin_action'
        ));
```

### `backend/migrations/0093_create_event_consumers_dlq.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0093_create_event_consumers_dlq.sql`
- Size bytes / Размер в байтах: `4474`
- Included characters / Включено символов: `4474`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS event_consumers (
    consumer_name TEXT PRIMARY KEY,
    last_processed_position BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    locked_by TEXT,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_consumers_name_not_empty CHECK (length(trim(consumer_name)) > 0),
    CONSTRAINT event_consumers_position_non_negative CHECK (last_processed_position >= 0),
    CONSTRAINT event_consumers_status CHECK (status IN ('active', 'paused', 'disabled')),
    CONSTRAINT event_consumers_locked_by_not_empty CHECK (
        locked_by IS NULL OR length(trim(locked_by)) > 0
    )
);

CREATE INDEX IF NOT EXISTS event_consumers_updated_at_idx
    ON event_consumers (updated_at);

CREATE TABLE IF NOT EXISTS event_consumer_failures (
    consumer_name TEXT NOT NULL REFERENCES event_consumers(consumer_name) ON DELETE CASCADE,
    event_position BIGINT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 1,
    next_attempt_at TIMESTAMPTZ NOT NULL,
    last_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (consumer_name, event_position),

    CONSTRAINT event_consumer_failures_position_positive CHECK (event_position > 0),
    CONSTRAINT event_consumer_failures_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_consumer_failures_event_type_not_empty CHECK (length(trim(event_type)) > 0),
    CONSTRAINT event_consumer_failures_attempt_count_positive CHECK (attempt_count > 0),
    CONSTRAINT event_consumer_failures_last_error_not_empty CHECK (length(trim(last_error)) > 0)
);

CREATE INDEX IF NOT EXISTS event_consumer_failures_due_idx
    ON event_consumer_failures (consumer_name, next_attempt_at, event_position);

CREATE TABLE IF NOT EXISTS event_consumer_processed_events (
    consumer_name TEXT NOT NULL REFERENCES event_consumers(consumer_name) ON DELETE CASCADE,
    event_position BIGINT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (consumer_name, event_position),

    CONSTRAINT event_consumer_processed_events_position_positive CHECK (event_position > 0),
    CONSTRAINT event_consumer_processed_events_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_consumer_processed_events_event_type_not_empty CHECK (length(trim(event_type)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS event_consumer_processed_events_event_id_idx
    ON event_consumer_processed_events (consumer_name, event_id);

CREATE TABLE IF NOT EXISTS event_dead_letters (
    dead_letter_id TEXT PRIMARY KEY,
    consumer_name TEXT NOT NULL REFERENCES event_consumers(consumer_name) ON DELETE CASCADE,
    event_position BIGINT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    attempts INTEGER NOT NULL,
    last_error TEXT NOT NULL,
    event_payload JSONB NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'open',
    replay_requested_at TIMESTAMPTZ,
    replayed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_dead_letters_consumer_event_unique UNIQUE (consumer_name, event_position),
    CONSTRAINT event_dead_letters_id_not_empty CHECK (length(trim(dead_letter_id)) > 0),
    CONSTRAINT event_dead_letters_position_positive CHECK (event_position > 0),
    CONSTRAINT event_dead_letters_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_dead_letters_event_type_not_empty CHECK (length(trim(event_type)) > 0),
    CONSTRAINT event_dead_letters_attempts_positive CHECK (attempts > 0),
    CONSTRAINT event_dead_letters_last_error_not_empty CHECK (length(trim(last_error)) > 0),
    CONSTRAINT event_dead_letters_payload_is_object CHECK (jsonb_typeof(event_payload) = 'object'),
    CONSTRAINT event_dead_letters_review_state CHECK (
        review_state IN ('open', 'replay_requested', 'replayed', 'dismissed')
    )
);

CREATE INDEX IF NOT EXISTS event_dead_letters_review_idx
    ON event_dead_letters (review_state, created_at DESC);

CREATE INDEX IF NOT EXISTS event_dead_letters_consumer_idx
    ON event_dead_letters (consumer_name, event_position);
```

### `backend/migrations/0094_create_canonical_evidence_review_context.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0094_create_canonical_evidence_review_context.sql`
- Size bytes / Размер в байтах: `14026`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
CREATE TABLE IF NOT EXISTS observation_kind_definitions (
    kind_definition_id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    category TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT observation_kind_definitions_id_not_empty CHECK (length(trim(kind_definition_id)) > 0),
    CONSTRAINT observation_kind_definitions_code_not_empty CHECK (length(trim(code)) > 0),
    CONSTRAINT observation_kind_definitions_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT observation_kind_definitions_version_positive CHECK (version > 0),
    CONSTRAINT observation_kind_definitions_category_not_empty CHECK (length(trim(category)) > 0),
    CONSTRAINT observation_kind_definitions_code_upper CHECK (code = upper(code)),
    UNIQUE (code, version)
);

INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES
    (
        'observation_kind:v1:communication_message',
        'COMMUNICATION_MESSAGE',
        'Communication message',
        1,
        'communication',
        'Provider or manual communication message captured as evidence.'
    ),
    (
        'observation_kind:v1:communication_message_deleted',
        'COMMUNICATION_MESSAGE_DELETED',
        'Communication message deleted',
        1,
        'communication',
        'Observed provider-side deletion or disappearance of a communication message.'
    ),
    (
        'observation_kind:v1:communication_attachment',
        'COMMUNICATION_ATTACHMENT',
        'Communication attachment',
        1,
        'communication',
        'Attachment captured from a communication source.'
    ),
    (
        'observation_kind:v1:meeting',
        'MEETING',
        'Meeting',
        1,
        'meeting',
        'Meeting occurrence captured as evidence.'
    ),
    (
        'observation_kind:v1:meeting_recording',
        'MEETING_RECORDING',
        'Meeting recording',
        1,
        'meeting',
        'Meeting audio or video recording reference captured as evidence.'
    ),
    (
        'observation_kind:v1:meeting_transcript',
        'MEETING_TRANSCRIPT',
        'Meeting transcript',
        1,
        'meeting',
        'Meeting transcript captured as evidence.'
    ),
    (
        'observation_kind:v1:document',
        'DOCUMENT',
        'Document',
        1,
        'document',
        'Imported or manually created document evidence.'
    ),
    (
        'observation_kind:v1:voice_recording',
        'VOICE_RECORDING',
        'Voice recording',
        1,
        'voice',
        'Voice memo or recording captured as evidence.'
    ),
    (
        'observation_kind:v1:browser_capture',
        'BROWSER_CAPTURE',
        'Browser capture',
        1,
        'browser',
        'User-driven browser capture evidence.'
    ),
    (
        'observation_kind:v1:contact_record',
        'CONTACT_RECORD',
        'Contact record',
        1,
        'identity',
        'Provider contact or address book record captured as evidence.'
    ),
    (
        'observation_kind:v1:calendar_event',
        'CALENDAR_EVENT',
        'Calendar event',
        1,
        'calendar',
        'Calendar event captured as evidence.'
    ),
    (
        'observation_kind:v1:calendar_event_deleted',
        'CALENDAR_EVENT_DELETED',
        'Calendar event deleted',
        1,
        'calendar',
        'Observed deletion or archival removal of a calendar event.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();

CREATE TABLE IF NOT EXISTS observations (
    observation_id TEXT PRIMARY KEY,
    kind_definition_id TEXT NOT NULL REFERENCES observation_kind_definitions(kind_definition_id),
    origin_kind TEXT NOT NULL,
    vault_source_id TEXT,
    observed_at TIMESTAMPTZ NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    content_hash TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT observations_id_not_empty CHECK (length(trim(observation_id)) > 0),
    CONSTRAINT observations_origin_kind CHECK (
        origin_kind IN (
            'vault_source',
            'manual',
            'browser_capture',
            'voice_memo',
            'file_import',
            'local_runtime',
            'test_fixture'
        )
    ),
    CONSTRAINT observations_vault_source_id_not_empty CHECK (
        vault_source_id IS NULL OR length(trim(vault_source_id)) > 0
    ),
    CONSTRAINT observations_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT observations_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT observations_content_hash_not_empty CHECK (length(trim(content_hash)) > 0),
    CONSTRAINT observations_source_ref_not_empty CHECK (length(trim(source_ref)) > 0),
    CONSTRAINT observations_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS observations_kind_captured_idx
    ON observations (kind_definition_id, captured_at DESC);

CREATE INDEX IF NOT EXISTS observations_vault_source_idx
    ON observations (vault_source_id)
    WHERE vault_source_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS observations_source_ref_idx
    ON observations (source_ref, captured_at DESC);

CREATE INDEX IF NOT EXISTS observations_content_hash_idx
    ON observations (content_hash);

CREATE OR REPLACE FUNCTION prevent_observations_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'observations are append-only; create a new observation instead'
        USING ERRCODE = '55000';
END;
$$;

DROP TRIGGER IF EXISTS observations_append_only_update ON observations;
CREATE TRIGGER observations_append_only_update
    BEFORE UPDATE ON observations
    FOR EACH ROW
    EXECUTE FUNCTION prevent_observations_mutation();

DROP TRIGGER IF EXISTS observations_append_only_delete ON observations;
CREATE TRIGGER observations_append_only_delete
    BEFORE DELETE ON observations
    FOR EACH ROW
    EXECUTE FUNCTION prevent_observations_mutation();

CREATE TABLE IF NOT EXISTS observation_links (
    observation_id TEXT NOT NULL REFERENCES observations(observation_id),
    domain TEXT NOT NULL,
    entity_kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    relationship_kind TEXT NOT NULL DEFAULT 'evidence_for',
    confidence DOUBLE PRECISION NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (observation_id, domain, entity_kind, entity_id, relationship_kind),

    CONSTRAINT observation_links_domain_not_empty CHECK (length(trim(domain)) > 0),
    CONSTRAINT observation_links_entity_kind_not_empty CHECK (length(trim(entity_kind)) > 0),
    CONSTRAINT observation_links_entity_id_not_empty CHECK (length(trim(entity_id)) > 0),
    CONSTRAINT observation_links_relationship_kind_not_empty CHECK (length(trim(relationship_kind)) > 0),
    CONSTRAINT observation_links_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT observation_links_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS observation_links_entity_idx
    ON observation_links (domain, entity_kind, entity_id);

CREATE TABLE IF NOT EXISTS observation_ingestion_runs (
    ingestion_run_id TEXT PRIMARY KEY,
    observation_id TEXT NOT NULL REFERENCES observations(observation_id),
    pipeline TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    output JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,

    CONSTRAINT observation_ingestion_runs_id_not_empty CHECK (length(trim(ingestion_run_id)) > 0),
    CONSTRAINT observation_ingestion_runs_pipeline_not_empty CHECK (length(trim(pipeline)) > 0),
    CONSTRAINT observation_ingestion_runs_status CHECK (
        status IN ('running', 'succeeded', 'failed', 'skipped')
    ),
    CONSTRAINT observation_ingestion_runs_output_is_object CHECK (jsonb_typeof(output) = 'object'),
    CONSTRAINT observation_ingestion_runs_error_not_empty CHECK (
        error_message IS NULL OR length(trim(error_message)) > 0
    )
);

CREATE INDEX IF NOT EXISTS observation_ingestion_runs_observation_idx
    ON observation_ingestion_runs (observation_id, started_at DESC);

CREATE TABLE IF NOT EXISTS review_items (
    review_item_id TEXT PRIMARY KEY,
    item_kind TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new',
    target_domain TEXT,
    target_entity_kind TEXT,
    target_entity_id TEXT,
    confidence DOUBLE PRECISION NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT review_items_id_not_empty CHECK (length(trim(review_item_id)) > 0),
    CONSTRAINT review_items_item_kind CHECK (
        item_kind IN (
            'new_person',
            'new_organization',
            'potential_task',
            'potential_obligation',
            'potential_decision',
            'potential_relationship',
            'potential_project',
            'knowledge_candidate'
        )
    ),
    CONSTRAINT review_items_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT review_items_summary_not_empty CHECK (length(trim(summary)) > 0),
    CONSTRAINT review_items_status CHECK (
        status IN ('new', 'in_review', 'approved', 'promoted', 'dismissed', 'archived')
    ),
    CONSTRAINT review_items_target_domain_not_empty CHECK (
        target_domain IS NULL OR length(trim(target_domain)) > 0
    ),
    CONSTRAINT review_items_target_entity_kind_not_empty CHECK (
        target_entity_kind IS NULL OR length(trim(target_entity_kind)) > 0
    ),
    CONSTRAINT review_items_target_entity_id_not_empty CHECK (
        target_entity_id IS NULL OR length(trim(target_entity_id)) > 0
    ),
    CONSTRAINT review_items_target_complete CHECK (
        (target_domain IS NULL AND target_entity_kind IS NULL AND target_entity_id IS NULL)
        OR
        (target_domain IS NOT NULL AND target_entity_kind IS NOT NULL AND target_entity_id IS NOT NULL)
    ),
    CONSTRAINT review_items_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT review_items_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS review_items_status_idx
    ON review_items (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS review_items_kind_status_idx
    ON review_items (item_kind, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS review_items_target_idx
    ON review_items (target_domain, target_entity_kind, target_entity_id)
    WHERE target_domain IS NOT NULL;

CREATE TABLE IF NOT EXISTS review_item_evidence (
    review_item_id TEXT NOT NULL REFERENCES review_items(review_item_id) ON DELETE CASCADE,
    observation_id TEXT NOT NULL REFERENCES observations(observation_id),
    evidence_role TEXT NOT NULL DEFAULT 'primary',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (review_item_id, observation_id, evidence_role),

    CONSTRAINT review_item_evidence_role_not_empty CHECK (length(trim(evidence_role)) > 0),
    CONSTRAINT review_item_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS review_item_evidence_observation_idx
    ON review_item_evidence (observation_id);

CREATE TABLE IF NOT EXISTS context_packs (
    context_pack_id TEXT PRIMARY KEY,
    kind
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/migrations/0095_add_task_provenance.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0095_add_task_provenance.sql`
- Size bytes / Размер в байтах: `1218`
- Included characters / Включено символов: `1218`
- Truncated / Обрезано: `no`

```text
ALTER TABLE tasks
    ADD COLUMN IF NOT EXISTS provenance_kind TEXT,
    ADD COLUMN IF NOT EXISTS provenance_id TEXT;

UPDATE tasks
SET
    provenance_kind = COALESCE(
        provenance_kind,
        CASE
            WHEN source_kind IN ('manual', 'message', 'email', 'telegram', 'whatsapp', 'calendar', 'meeting', 'document', 'note', 'jira', 'youtrack', 'github', 'gitlab', 'linear', 'todoist', 'apple_reminders', 'ms_todo', 'ai_rule', 'workflow', 'import')
                THEN 'observation'
            ELSE 'review_item'
        END
    ),
    provenance_id = COALESCE(provenance_id, source_id)
WHERE provenance_kind IS NULL
   OR provenance_id IS NULL;

ALTER TABLE tasks
    ALTER COLUMN provenance_kind SET NOT NULL,
    ALTER COLUMN provenance_id SET NOT NULL;

ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_provenance_kind_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_provenance_kind_check CHECK (
        provenance_kind IN ('observation', 'review_item', 'decision', 'obligation')
    );

ALTER TABLE tasks
    ADD CONSTRAINT tasks_provenance_id_not_empty CHECK (length(trim(provenance_id)) > 0);

CREATE INDEX IF NOT EXISTS tasks_provenance_idx
    ON tasks (provenance_kind, provenance_id);
```

### `backend/migrations/0096_expand_task_source_type_for_observation_spine.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0096_expand_task_source_type_for_observation_spine.sql`
- Size bytes / Размер в байтах: `651`
- Included characters / Включено символов: `651`
- Truncated / Обрезано: `no`

```text
ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_type_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_type_check CHECK (
        source_type IN (
            'manual',
            'communication',
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

### `backend/migrations/0097_link_communication_raw_records_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0097_link_communication_raw_records_to_observations.sql`
- Size bytes / Размер в байтах: `1938`
- Included characters / Включено символов: `1938`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_raw_records
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

INSERT INTO observations (
    observation_id,
    kind_definition_id,
    origin_kind,
    vault_source_id,
    observed_at,
    captured_at,
    payload,
    confidence,
    content_hash,
    source_ref,
    provenance
)
SELECT
    'observation:v1:communication-raw-record:' || raw.raw_record_id,
    kind.kind_definition_id,
    'vault_source',
    NULL,
    COALESCE(raw.occurred_at, raw.captured_at),
    raw.captured_at,
    raw.payload,
    1.0,
    raw.source_fingerprint,
    'communication://' || raw.account_id || '/' || raw.record_kind || '/' || raw.provider_record_id,
    raw.provenance || jsonb_build_object(
        'legacy_backfill', true,
        'raw_record_id', raw.raw_record_id,
        'account_id', raw.account_id,
        'record_kind', raw.record_kind,
        'provider_record_id', raw.provider_record_id,
        'import_batch_id', raw.import_batch_id
    )
FROM communication_raw_records raw
JOIN observation_kind_definitions kind
  ON kind.code = CASE
      WHEN raw.record_kind LIKE '%attachment%' THEN 'COMMUNICATION_ATTACHMENT'
      ELSE 'COMMUNICATION_MESSAGE'
  END
 AND kind.version = 1
WHERE raw.observation_id IS NULL
ON CONFLICT (observation_id) DO NOTHING;

UPDATE communication_raw_records raw
SET observation_id = 'observation:v1:communication-raw-record:' || raw.raw_record_id
WHERE raw.observation_id IS NULL;

ALTER TABLE communication_raw_records
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE communication_raw_records
    DROP CONSTRAINT IF EXISTS communication_raw_records_observation_fk;

ALTER TABLE communication_raw_records
    ADD CONSTRAINT communication_raw_records_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

CREATE INDEX IF NOT EXISTS communication_raw_records_observation_idx
    ON communication_raw_records (observation_id);
```

### `backend/migrations/0098_link_communication_messages_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0098_link_communication_messages_to_observations.sql`
- Size bytes / Размер в байтах: `758`
- Included characters / Включено символов: `758`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE communication_messages message
SET observation_id = raw.observation_id
FROM communication_raw_records raw
WHERE message.raw_record_id = raw.raw_record_id
  AND message.observation_id IS NULL;

ALTER TABLE communication_messages
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_observation_fk;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

CREATE INDEX IF NOT EXISTS communication_messages_observation_idx
    ON communication_messages (observation_id);
```

### `backend/migrations/0099_link_domain_evidence_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0099_link_domain_evidence_to_observations.sql`
- Size bytes / Размер в байтах: `5028`
- Included characters / Включено символов: `5028`
- Truncated / Обрезано: `no`

```text
ALTER TABLE decision_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

ALTER TABLE obligation_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

ALTER TABLE relationship_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE decision_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.observation_id IS NULL
  AND evidence.source_id = observation.observation_id;

UPDATE obligation_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.observation_id IS NULL
  AND evidence.source_id = observation.observation_id;

UPDATE relationship_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.observation_id IS NULL
  AND evidence.source_id = observation.observation_id;

ALTER TABLE decision_evidence
    DROP CONSTRAINT IF EXISTS decision_evidence_source_kind_check;

ALTER TABLE decision_evidence
    ADD CONSTRAINT decision_evidence_source_kind_check CHECK (
        source_kind IN (
            'observation',
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
    );

ALTER TABLE obligation_evidence
    DROP CONSTRAINT IF EXISTS obligation_evidence_source_kind_check;

ALTER TABLE obligation_evidence
    ADD CONSTRAINT obligation_evidence_source_kind_check CHECK (
        source_kind IN (
            'observation',
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
    );

ALTER TABLE relationship_evidence
    DROP CONSTRAINT IF EXISTS relationship_evidence_source_kind_check;

ALTER TABLE relationship_evidence
    ADD CONSTRAINT relationship_evidence_source_kind_check CHECK (
        source_kind IN (
            'observation',
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
    );

ALTER TABLE decision_evidence
    DROP CONSTRAINT IF EXISTS decision_evidence_observation_fk;

ALTER TABLE decision_evidence
    ADD CONSTRAINT decision_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE obligation_evidence
    DROP CONSTRAINT IF EXISTS obligation_evidence_observation_fk;

ALTER TABLE obligation_evidence
    ADD CONSTRAINT obligation_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE relationship_evidence
    DROP CONSTRAINT IF EXISTS relationship_evidence_observation_fk;

ALTER TABLE relationship_evidence
    ADD CONSTRAINT relationship_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE decision_evidence
    DROP CONSTRAINT IF EXISTS decision_evidence_observation_source_check;

ALTER TABLE decision_evidence
    ADD CONSTRAINT decision_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

ALTER TABLE obligation_evidence
    DROP CONSTRAINT IF EXISTS obligation_evidence_observation_source_check;

ALTER TABLE obligation_evidence
    ADD CONSTRAINT obligation_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

ALTER TABLE relationship_evidence
    DROP CONSTRAINT IF EXISTS relationship_evidence_observation_source_check;

ALTER TABLE relationship_evidence
    ADD CONSTRAINT relationship_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

CREATE INDEX IF NOT EXISTS decision_evidence_observation_idx
    ON decision_evidence (observation_id)
    WHERE observation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS obligation_evidence_observation_idx
    ON obligation_evidence (observation_id)
    WHERE observation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS relationship_evidence_observation_idx
    ON relationship_evidence (observation_id)
    WHERE observation_id IS NOT NULL;
```

### `backend/migrations/0100_link_task_candidates_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0100_link_task_candidates_to_observations.sql`
- Size bytes / Размер в байтах: `989`
- Included characters / Включено символов: `989`
- Truncated / Обрезано: `no`

```text
ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE task_candidates candidate
SET observation_id = message.observation_id
FROM communication_messages message
WHERE candidate.source_kind = 'message'
  AND candidate.source_id = message.message_id
  AND candidate.observation_id IS NULL;

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_observation_fk;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_message_observation_required;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_message_observation_required CHECK (
        source_kind != 'message'
        OR observation_id IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS task_candidates_observation_idx
    ON task_candidates (observation_id)
    WHERE observation_id IS NOT NULL;
```
