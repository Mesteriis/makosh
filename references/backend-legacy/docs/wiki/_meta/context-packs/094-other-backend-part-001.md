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

- Chunk ID / ID чанка: `094-other-backend-part-001`
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

### `backend/migrations/0001_create_event_log.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0001_create_event_log.sql`
- Size bytes / Размер в байтах: `2474`
- Included characters / Включено символов: `2474`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS event_log (
    position BIGINT GENERATED ALWAYS AS IDENTITY UNIQUE,
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    source JSONB NOT NULL,
    actor JSONB,
    subject JSONB NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    causation_id TEXT,
    correlation_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_log_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_log_event_type_not_empty CHECK (length(trim(event_type)) > 0),
    CONSTRAINT event_log_schema_version_positive CHECK (schema_version > 0),
    CONSTRAINT event_log_source_is_object CHECK (jsonb_typeof(source) = 'object'),
    CONSTRAINT event_log_actor_is_object CHECK (actor IS NULL OR jsonb_typeof(actor) = 'object'),
    CONSTRAINT event_log_subject_is_object CHECK (jsonb_typeof(subject) = 'object'),
    CONSTRAINT event_log_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT event_log_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS event_log_recorded_at_idx
    ON event_log (recorded_at, position);

CREATE INDEX IF NOT EXISTS event_log_occurred_at_idx
    ON event_log (occurred_at, position);

CREATE INDEX IF NOT EXISTS event_log_event_type_idx
    ON event_log (event_type, recorded_at);

CREATE INDEX IF NOT EXISTS event_log_correlation_id_idx
    ON event_log (correlation_id)
    WHERE correlation_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS event_log_source_idempotency_idx
    ON event_log (
        event_type,
        (source ->> 'kind'),
        COALESCE(source ->> 'provider', ''),
        (source ->> 'source_id')
    )
    WHERE source ? 'source_id';

CREATE OR REPLACE FUNCTION prevent_event_log_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'event_log is append-only';
END;
$$;

DROP TRIGGER IF EXISTS event_log_prevent_update ON event_log;
CREATE TRIGGER event_log_prevent_update
    BEFORE UPDATE ON event_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_event_log_mutation();

DROP TRIGGER IF EXISTS event_log_prevent_delete ON event_log;
CREATE TRIGGER event_log_prevent_delete
    BEFORE DELETE ON event_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_event_log_mutation();
```

### `backend/migrations/0002_create_projection_cursors.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0002_create_projection_cursors.sql`
- Size bytes / Размер в байтах: `484`
- Included characters / Включено символов: `484`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS projection_cursors (
    projection_name TEXT PRIMARY KEY,
    last_processed_position BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT projection_cursors_name_not_empty CHECK (length(trim(projection_name)) > 0),
    CONSTRAINT projection_cursors_position_non_negative CHECK (last_processed_position >= 0)
);

CREATE INDEX IF NOT EXISTS projection_cursors_updated_at_idx
    ON projection_cursors (updated_at);
```

### `backend/migrations/0003_create_api_audit_log.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0003_create_api_audit_log.sql`
- Size bytes / Размер в байтах: `1870`
- Included characters / Включено символов: `1870`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS api_audit_log (
    audit_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_kind TEXT NOT NULL,
    operation TEXT NOT NULL,
    method TEXT NOT NULL,
    path_template TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT api_audit_log_actor_kind_not_empty CHECK (length(trim(actor_kind)) > 0),
    CONSTRAINT api_audit_log_operation_not_empty CHECK (length(trim(operation)) > 0),
    CONSTRAINT api_audit_log_method_not_empty CHECK (length(trim(method)) > 0),
    CONSTRAINT api_audit_log_path_template_not_empty CHECK (length(trim(path_template)) > 0),
    CONSTRAINT api_audit_log_target_kind_not_empty CHECK (length(trim(target_kind)) > 0),
    CONSTRAINT api_audit_log_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS api_audit_log_recorded_at_idx
    ON api_audit_log (recorded_at, audit_id);

CREATE INDEX IF NOT EXISTS api_audit_log_operation_idx
    ON api_audit_log (operation, recorded_at);

CREATE INDEX IF NOT EXISTS api_audit_log_target_idx
    ON api_audit_log (target_kind, target_id, recorded_at)
    WHERE target_id IS NOT NULL;

CREATE OR REPLACE FUNCTION prevent_api_audit_log_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'api_audit_log is append-only';
END;
$$;

DROP TRIGGER IF EXISTS api_audit_log_prevent_update ON api_audit_log;
CREATE TRIGGER api_audit_log_prevent_update
    BEFORE UPDATE ON api_audit_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_api_audit_log_mutation();

DROP TRIGGER IF EXISTS api_audit_log_prevent_delete ON api_audit_log;
CREATE TRIGGER api_audit_log_prevent_delete
    BEFORE DELETE ON api_audit_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_api_audit_log_mutation();
```

### `backend/migrations/0004_add_api_audit_actor_id.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0004_add_api_audit_actor_id.sql`
- Size bytes / Размер в байтах: `322`
- Included characters / Включено символов: `322`
- Truncated / Обрезано: `no`

```text
ALTER TABLE api_audit_log
    ADD COLUMN actor_id TEXT;

ALTER TABLE api_audit_log
    ADD CONSTRAINT api_audit_log_actor_id_not_empty
    CHECK (actor_id IS NULL OR length(trim(actor_id)) > 0);

CREATE INDEX api_audit_log_actor_idx
    ON api_audit_log (actor_kind, actor_id, recorded_at)
    WHERE actor_id IS NOT NULL;
```

### `backend/migrations/0005_create_communication_ingestion.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0005_create_communication_ingestion.sql`
- Size bytes / Размер в байтах: `4097`
- Included characters / Включено символов: `4097`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS communication_provider_accounts (
    account_id TEXT PRIMARY KEY,
    provider_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    external_account_id TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_provider_account_kind CHECK (provider_kind IN ('gmail', 'icloud', 'imap')),
    CONSTRAINT communication_provider_account_id_not_empty CHECK (length(trim(account_id)) > 0),
    CONSTRAINT communication_provider_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_provider_external_id_not_empty CHECK (length(trim(external_account_id)) > 0),
    CONSTRAINT communication_provider_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT communication_provider_external_identity_unique UNIQUE (provider_kind, external_account_id)
);

CREATE INDEX IF NOT EXISTS communication_provider_accounts_kind_idx
    ON communication_provider_accounts (provider_kind, created_at);

CREATE TABLE IF NOT EXISTS communication_raw_records (
    raw_record_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    record_kind TEXT NOT NULL,
    provider_record_id TEXT NOT NULL,
    source_fingerprint TEXT NOT NULL,
    import_batch_id TEXT NOT NULL,
    occurred_at TIMESTAMPTZ,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT communication_raw_record_id_not_empty CHECK (length(trim(raw_record_id)) > 0),
    CONSTRAINT communication_raw_record_kind_not_empty CHECK (length(trim(record_kind)) > 0),
    CONSTRAINT communication_raw_provider_record_id_not_empty CHECK (length(trim(provider_record_id)) > 0),
    CONSTRAINT communication_raw_source_fingerprint_not_empty CHECK (length(trim(source_fingerprint)) > 0),
    CONSTRAINT communication_raw_import_batch_id_not_empty CHECK (length(trim(import_batch_id)) > 0),
    CONSTRAINT communication_raw_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT communication_raw_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT communication_raw_provider_identity_unique UNIQUE (account_id, record_kind, provider_record_id)
);

CREATE INDEX IF NOT EXISTS communication_raw_records_account_idx
    ON communication_raw_records (account_id, captured_at, raw_record_id);

CREATE INDEX IF NOT EXISTS communication_raw_records_import_batch_idx
    ON communication_raw_records (import_batch_id, captured_at);

CREATE TABLE IF NOT EXISTS communication_ingestion_checkpoints (
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    stream_id TEXT NOT NULL,
    checkpoint JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_checkpoint_stream_id_not_empty CHECK (length(trim(stream_id)) > 0),
    CONSTRAINT communication_checkpoint_is_object CHECK (jsonb_typeof(checkpoint) = 'object'),
    PRIMARY KEY (account_id, stream_id)
);

CREATE INDEX IF NOT EXISTS communication_ingestion_checkpoints_updated_at_idx
    ON communication_ingestion_checkpoints (updated_at);

CREATE OR REPLACE FUNCTION prevent_communication_raw_records_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'communication_raw_records is append-only';
END;
$$;

DROP TRIGGER IF EXISTS communication_raw_records_prevent_update ON communication_raw_records;
CREATE TRIGGER communication_raw_records_prevent_update
    BEFORE UPDATE ON communication_raw_records
    FOR EACH ROW
    EXECUTE FUNCTION prevent_communication_raw_records_mutation();

DROP TRIGGER IF EXISTS communication_raw_records_prevent_delete ON communication_raw_records;
CREATE TRIGGER communication_raw_records_prevent_delete
    BEFORE DELETE ON communication_raw_records
    FOR EACH ROW
    EXECUTE FUNCTION prevent_communication_raw_records_mutation();
```

### `backend/migrations/0006_create_secret_references.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0006_create_secret_references.sql`
- Size bytes / Размер в байтах: `1862`
- Included characters / Включено символов: `1862`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS secret_references (
    secret_ref TEXT PRIMARY KEY,
    secret_kind TEXT NOT NULL,
    store_kind TEXT NOT NULL,
    label TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT secret_references_kind CHECK (
        secret_kind IN ('oauth_token', 'app_password', 'password', 'api_token', 'private_key', 'other')
    ),
    CONSTRAINT secret_references_store_kind CHECK (
        store_kind IN ('os_keychain', 'encrypted_vault', 'external_vault', 'test_double')
    ),
    CONSTRAINT secret_references_ref_not_empty CHECK (length(trim(secret_ref)) > 0),
    CONSTRAINT secret_references_label_not_empty CHECK (length(trim(label)) > 0),
    CONSTRAINT secret_references_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS secret_references_kind_idx
    ON secret_references (secret_kind, created_at);

CREATE TABLE IF NOT EXISTS communication_provider_account_secret_refs (
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    secret_purpose TEXT NOT NULL,
    secret_ref TEXT NOT NULL REFERENCES secret_references(secret_ref) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_provider_account_secret_purpose CHECK (
        secret_purpose IN ('oauth_token', 'imap_password', 'smtp_password')
    ),
    CONSTRAINT communication_provider_account_secret_ref_not_empty CHECK (length(trim(secret_ref)) > 0),
    PRIMARY KEY (account_id, secret_purpose)
);

CREATE INDEX IF NOT EXISTS communication_provider_account_secret_refs_secret_idx
    ON communication_provider_account_secret_refs (secret_ref, updated_at);
```

### `backend/migrations/0007_create_communication_messages.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0007_create_communication_messages.sql`
- Size bytes / Размер в байтах: `946`
- Included characters / Включено символов: `946`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS communication_messages (
    message_id TEXT PRIMARY KEY,
    raw_record_id TEXT NOT NULL REFERENCES communication_raw_records(raw_record_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_record_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    sender TEXT NOT NULL,
    recipients JSONB NOT NULL,
    body_text TEXT NOT NULL,
    occurred_at TIMESTAMPTZ,
    projected_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_messages_subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT communication_messages_sender_not_empty CHECK (length(trim(sender)) > 0),
    CONSTRAINT communication_messages_recipients_is_array CHECK (jsonb_typeof(recipients) = 'array'),
    CONSTRAINT communication_messages_body_not_empty CHECK (length(trim(body_text)) > 0),
    UNIQUE (account_id, provider_record_id)
);
```

### `backend/migrations/0008_create_contacts.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0008_create_contacts.sql`
- Size bytes / Размер в байтах: `416`
- Included characters / Включено символов: `416`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS contacts (
    contact_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    email_address TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT contacts_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT contacts_email_not_empty CHECK (length(trim(email_address)) > 0)
);
```

### `backend/migrations/0009_create_documents.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0009_create_documents.sql`
- Size bytes / Размер в байтах: `501`
- Included characters / Включено символов: `501`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS documents (
    document_id TEXT PRIMARY KEY,
    document_kind TEXT NOT NULL,
    title TEXT NOT NULL,
    source_fingerprint TEXT NOT NULL,
    extracted_text TEXT NOT NULL,
    imported_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT documents_kind CHECK (document_kind IN ('markdown', 'pdf')),
    CONSTRAINT documents_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT documents_fingerprint_not_empty CHECK (length(trim(source_fingerprint)) > 0)
);
```

### `backend/migrations/0010_create_graph_core.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0010_create_graph_core.sql`
- Size bytes / Размер в байтах: `3048`
- Included characters / Включено символов: `3048`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS graph_nodes (
    node_id TEXT PRIMARY KEY,
    node_kind TEXT NOT NULL,
    stable_key TEXT NOT NULL,
    label TEXT NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_nodes_kind CHECK (node_kind IN ('person', 'email_address', 'message', 'document')),
    CONSTRAINT graph_nodes_stable_key_not_empty CHECK (length(trim(stable_key)) > 0),
    CONSTRAINT graph_nodes_label_not_empty CHECK (length(trim(label)) > 0),
    CONSTRAINT graph_nodes_properties_is_object CHECK (jsonb_typeof(properties) = 'object'),
    UNIQUE (node_kind, stable_key)
);

CREATE TABLE IF NOT EXISTS graph_edges (
    edge_id TEXT PRIMARY KEY,
    source_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    target_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    confidence NUMERIC(5,4) NOT NULL,
    review_state TEXT NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_edges_relationship_type CHECK (
        relationship_type IN (
            'person_has_email_address',
            'person_sent_message',
            'person_received_message',
            'email_address_sent_message',
            'email_address_received_message'
        )
    ),
    CONSTRAINT graph_edges_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT graph_edges_review_state CHECK (
        review_state IN ('system_accepted', 'suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT graph_edges_properties_is_object CHECK (jsonb_typeof(properties) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS graph_edges_active_unique
ON graph_edges (source_node_id, target_node_id, relationship_type)
WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS graph_evidence (
    evidence_id TEXT PRIMARY KEY,
    edge_id TEXT NOT NULL REFERENCES graph_edges(edge_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    excerpt TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_evidence_source_kind CHECK (source_kind IN ('contact', 'message', 'document', 'raw_record')),
    CONSTRAINT graph_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT graph_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (edge_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS graph_nodes_label_idx ON graph_nodes (label);
CREATE INDEX IF NOT EXISTS graph_edges_source_idx ON graph_edges (source_node_id);
CREATE INDEX IF NOT EXISTS graph_edges_target_idx ON graph_edges (target_node_id);
CREATE INDEX IF NOT EXISTS graph_evidence_edge_idx ON graph_evidence (edge_id);
```

### `backend/migrations/0011_create_mail_blob_storage.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0011_create_mail_blob_storage.sql`
- Size bytes / Размер в байтах: `2894`
- Included characters / Включено символов: `2894`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS communication_mail_blobs (
    blob_id TEXT PRIMARY KEY,
    storage_kind TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    content_type TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_mail_blob_storage_kind CHECK (storage_kind IN ('local_fs')),
    CONSTRAINT communication_mail_blob_id_not_empty CHECK (length(trim(blob_id)) > 0),
    CONSTRAINT communication_mail_blob_storage_path_not_empty CHECK (length(trim(storage_path)) > 0),
    CONSTRAINT communication_mail_blob_sha256_not_empty CHECK (length(trim(sha256)) > 0),
    CONSTRAINT communication_mail_blob_size_non_negative CHECK (size_bytes >= 0),
    CONSTRAINT communication_mail_blob_content_type_not_empty CHECK (
        content_type IS NULL OR length(trim(content_type)) > 0
    ),
    CONSTRAINT communication_mail_blob_storage_path_unique UNIQUE (storage_kind, storage_path),
    CONSTRAINT communication_mail_blob_sha256_unique UNIQUE (sha256)
);

CREATE TABLE IF NOT EXISTS communication_attachments (
    attachment_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE RESTRICT,
    raw_record_id TEXT NOT NULL REFERENCES communication_raw_records(raw_record_id) ON DELETE RESTRICT,
    blob_id TEXT NOT NULL REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    provider_attachment_id TEXT NOT NULL,
    filename TEXT,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 TEXT NOT NULL,
    disposition TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_attachment_id_not_empty CHECK (length(trim(attachment_id)) > 0),
    CONSTRAINT communication_attachment_provider_id_not_empty CHECK (length(trim(provider_attachment_id)) > 0),
    CONSTRAINT communication_attachment_filename_not_empty CHECK (
        filename IS NULL OR length(trim(filename)) > 0
    ),
    CONSTRAINT communication_attachment_content_type_not_empty CHECK (length(trim(content_type)) > 0),
    CONSTRAINT communication_attachment_size_non_negative CHECK (size_bytes >= 0),
    CONSTRAINT communication_attachment_sha256_not_empty CHECK (length(trim(sha256)) > 0),
    CONSTRAINT communication_attachment_disposition CHECK (disposition IN ('attachment', 'inline', 'unknown')),
    CONSTRAINT communication_attachment_provider_identity_unique UNIQUE (message_id, provider_attachment_id)
);

CREATE INDEX IF NOT EXISTS communication_attachments_message_idx
    ON communication_attachments (message_id, created_at);

CREATE INDEX IF NOT EXISTS communication_attachments_raw_record_idx
    ON communication_attachments (raw_record_id);

CREATE INDEX IF NOT EXISTS communication_attachments_blob_idx
    ON communication_attachments (blob_id);
```

### `backend/migrations/0012_add_attachment_scan_metadata.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0012_add_attachment_scan_metadata.sql`
- Size bytes / Размер в байтах: `848`
- Included characters / Включено символов: `848`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_attachments
    ADD COLUMN scan_status TEXT NOT NULL DEFAULT 'not_scanned',
    ADD COLUMN scan_engine TEXT,
    ADD COLUMN scan_checked_at TIMESTAMPTZ,
    ADD COLUMN scan_summary TEXT,
    ADD COLUMN scan_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD CONSTRAINT communication_attachment_scan_status CHECK (
        scan_status IN ('not_scanned', 'clean', 'suspicious', 'malicious', 'failed')
    ),
    ADD CONSTRAINT communication_attachment_scan_engine_not_empty CHECK (
        scan_engine IS NULL OR length(trim(scan_engine)) > 0
    ),
    ADD CONSTRAINT communication_attachment_scan_summary_not_empty CHECK (
        scan_summary IS NULL OR length(trim(scan_summary)) > 0
    ),
    ADD CONSTRAINT communication_attachment_scan_metadata_object CHECK (
        jsonb_typeof(scan_metadata) = 'object'
    );
```

### `backend/migrations/0013_create_projects_and_extend_graph.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0013_create_projects_and_extend_graph.sql`
- Size bytes / Размер в байтах: `3036`
- Included characters / Включено символов: `3036`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS projects (
    project_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    description TEXT NOT NULL,
    owner_display_name TEXT NOT NULL,
    progress_percent INTEGER NOT NULL DEFAULT 0,
    start_date DATE,
    target_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT projects_id_not_empty CHECK (length(trim(project_id)) > 0),
    CONSTRAINT projects_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT projects_kind_not_empty CHECK (length(trim(kind)) > 0),
    CONSTRAINT projects_status CHECK (status IN ('planning', 'active', 'on_hold', 'completed', 'archived')),
    CONSTRAINT projects_description_not_empty CHECK (length(trim(description)) > 0),
    CONSTRAINT projects_owner_not_empty CHECK (length(trim(owner_display_name)) > 0),
    CONSTRAINT projects_progress_range CHECK (progress_percent >= 0 AND progress_percent <= 100)
);

CREATE TABLE IF NOT EXISTS project_keywords (
    project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    keyword TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT project_keywords_keyword_not_empty CHECK (length(trim(keyword)) > 0),
    PRIMARY KEY (project_id, keyword)
);

ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN ('person', 'email_address', 'message', 'document', 'project')
);

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
        'project_involves_email_address'
    )
);

CREATE INDEX IF NOT EXISTS projects_status_idx ON projects (status);
CREATE INDEX IF NOT EXISTS project_keywords_project_idx ON project_keywords (project_id);
CREATE INDEX IF NOT EXISTS project_keywords_keyword_idx ON project_keywords (keyword);

INSERT INTO projects (
    project_id,
    name,
    kind,
    status,
    description,
    owner_display_name,
    progress_percent,
    start_date,
    target_date
)
VALUES (
    'project:v1:makosh',
    'Макошь',
    'Product Development',
    'active',
    'Personal knowledge system for local-first communications, documents, graph memory and workflows.',
    'Alex Morgan',
    75,
    DATE '2024-01-15',
    DATE '2024-12-20'
)
ON CONFLICT (project_id) DO NOTHING;

INSERT INTO project_keywords (project_id, keyword)
VALUES
    ('project:v1:makosh', 'Макошь'),
    ('project:v1:makosh', 'Макошь Project'),
    ('project:v1:makosh', 'makosh')
ON CONFLICT (project_id, keyword) DO NOTHING;
```

### `backend/migrations/0014_create_project_link_reviews.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0014_create_project_link_reviews.sql`
- Size bytes / Размер в байтах: `1302`
- Included characters / Включено символов: `1302`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS project_link_reviews (
    project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL,
    review_state TEXT NOT NULL,
    event_id TEXT NOT NULL REFERENCES event_log(event_id),
    actor_id TEXT NOT NULL,
    reviewed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT project_link_reviews_pk PRIMARY KEY (project_id, target_kind, target_id),
    CONSTRAINT project_link_reviews_target_kind_check
        CHECK (target_kind IN ('message', 'document')),
    CONSTRAINT project_link_reviews_review_state_check
        CHECK (review_state IN ('user_confirmed', 'user_rejected')),
    CONSTRAINT project_link_reviews_actor_id_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT project_link_reviews_project_id_not_empty
        CHECK (length(trim(project_id)) > 0),
    CONSTRAINT project_link_reviews_target_id_not_empty
        CHECK (length(trim(target_id)) > 0)
);

CREATE INDEX IF NOT EXISTS project_link_reviews_event_id_idx
    ON project_link_reviews (event_id);

CREATE INDEX IF NOT EXISTS project_link_reviews_review_state_idx
    ON project_link_reviews (review_state, updated_at);
```

### `backend/migrations/0015_create_task_candidates.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0015_create_task_candidates.sql`
- Size bytes / Размер в байтах: `2734`
- Included characters / Включено символов: `2734`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS task_candidates (
    task_candidate_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    due_text TEXT,
    assignee_label TEXT,
    confidence DOUBLE PRECISION NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    evidence_excerpt TEXT NOT NULL,
    event_id TEXT,
    actor_id TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_candidates_source_kind_check
        CHECK (source_kind IN ('message', 'document')),
    CONSTRAINT task_candidates_review_state_check
        CHECK (review_state IN ('suggested', 'user_confirmed', 'user_rejected')),
    CONSTRAINT task_candidates_confidence_check
        CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT task_candidates_id_not_empty
        CHECK (length(trim(task_candidate_id)) > 0),
    CONSTRAINT task_candidates_source_id_not_empty
        CHECK (length(trim(source_id)) > 0),
    CONSTRAINT task_candidates_title_not_empty
        CHECK (length(trim(title)) > 0),
    CONSTRAINT task_candidates_evidence_excerpt_not_empty
        CHECK (length(trim(evidence_excerpt)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS task_candidates_source_title_idx
    ON task_candidates (source_kind, source_id, lower(title));

CREATE INDEX IF NOT EXISTS task_candidates_review_state_idx
    ON task_candidates (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS task_candidates_project_idx
    ON task_candidates (project_id);

CREATE TABLE IF NOT EXISTS tasks (
    task_id TEXT PRIMARY KEY,
    task_candidate_id TEXT NOT NULL UNIQUE
        REFERENCES task_candidates(task_candidate_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_from_event_id TEXT NOT NULL,
    created_by_actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT tasks_source_kind_check
        CHECK (source_kind IN ('message', 'document')),
    CONSTRAINT tasks_status_check
        CHECK (status IN ('active')),
    CONSTRAINT tasks_id_not_empty CHECK (length(trim(task_id)) > 0),
    CONSTRAINT tasks_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS tasks_project_idx ON tasks (project_id);
CREATE INDEX IF NOT EXISTS tasks_source_idx ON tasks (source_kind, source_id);
```

### `backend/migrations/0016_create_contact_identity_reviews.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0016_create_contact_identity_reviews.sql`
- Size bytes / Размер в байтах: `2222`
- Included characters / Включено символов: `2222`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS contact_identity_candidates (
    identity_candidate_id TEXT PRIMARY KEY,
    candidate_kind TEXT NOT NULL,
    left_contact_id TEXT NOT NULL REFERENCES contacts(contact_id) ON DELETE CASCADE,
    right_contact_id TEXT REFERENCES contacts(contact_id) ON DELETE CASCADE,
    email_address TEXT,
    evidence_summary TEXT NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    event_id TEXT,
    actor_id TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT contact_identity_candidate_kind_check
        CHECK (candidate_kind IN ('merge_contacts', 'attach_email_address', 'split_contact')),
    CONSTRAINT contact_identity_review_state_check
        CHECK (review_state IN ('suggested', 'user_confirmed', 'user_rejected')),
    CONSTRAINT contact_identity_confidence_check
        CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT contact_identity_candidate_id_not_empty
        CHECK (length(trim(identity_candidate_id)) > 0),
    CONSTRAINT contact_identity_left_contact_not_empty
        CHECK (length(trim(left_contact_id)) > 0),
    CONSTRAINT contact_identity_evidence_not_empty
        CHECK (length(trim(evidence_summary)) > 0),
    CONSTRAINT contact_identity_merge_has_right_contact
        CHECK (candidate_kind <> 'merge_contacts' OR right_contact_id IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS contact_identity_merge_pair_idx
    ON contact_identity_candidates (
        candidate_kind,
        LEAST(left_contact_id, COALESCE(right_contact_id, left_contact_id)),
        GREATEST(left_contact_id, COALESCE(right_contact_id, left_contact_id))
    )
    WHERE candidate_kind = 'merge_contacts';

CREATE INDEX IF NOT EXISTS contact_identity_review_state_idx
    ON contact_identity_candidates (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS contact_identity_left_contact_idx
    ON contact_identity_candidates (left_contact_id);

CREATE INDEX IF NOT EXISTS contact_identity_right_contact_idx
    ON contact_identity_candidates (right_contact_id);
```

### `backend/migrations/0017_create_document_processing.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0017_create_document_processing.sql`
- Size bytes / Размер в байтах: `2539`
- Included characters / Включено символов: `2539`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS document_processing_jobs (
    job_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    step TEXT NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error_summary TEXT,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT document_processing_step_check
        CHECK (step IN ('extract_text', 'ocr')),
    CONSTRAINT document_processing_status_check
        CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'skipped')),
    CONSTRAINT document_processing_attempts_check
        CHECK (attempts >= 0 AND max_attempts >= 1 AND attempts <= max_attempts),
    CONSTRAINT document_processing_job_id_not_empty
        CHECK (length(trim(job_id)) > 0),
    CONSTRAINT document_processing_document_id_not_empty
        CHECK (length(trim(document_id)) > 0),
    CONSTRAINT document_processing_document_step_unique
        UNIQUE (document_id, step)
);

CREATE INDEX IF NOT EXISTS document_processing_status_idx
    ON document_processing_jobs (status, queued_at);

CREATE INDEX IF NOT EXISTS document_processing_document_idx
    ON document_processing_jobs (document_id);

CREATE TABLE IF NOT EXISTS document_artifacts (
    artifact_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES document_processing_jobs(job_id) ON DELETE CASCADE,
    artifact_kind TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    text_content TEXT,
    storage_kind TEXT,
    storage_path TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT document_artifact_kind_check
        CHECK (artifact_kind IN ('extracted_text', 'ocr_text')),
    CONSTRAINT document_artifact_id_not_empty
        CHECK (length(trim(artifact_id)) > 0),
    CONSTRAINT document_artifact_sha_not_empty
        CHECK (length(trim(content_sha256)) > 0),
    CONSTRAINT document_artifact_text_or_storage
        CHECK (text_content IS NOT NULL OR storage_path IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS document_artifacts_document_kind_idx
    ON document_artifacts (document_id, artifact_kind);

CREATE INDEX IF NOT EXISTS document_artifacts_job_idx
    ON document_artifacts (job_id);
```

### `backend/migrations/0018_create_ai_runtime.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0018_create_ai_runtime.sql`
- Size bytes / Размер в байтах: `3758`
- Included characters / Включено символов: `3758`
- Truncated / Обрезано: `no`

```text
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS ai_agent_runs (
    run_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    status TEXT NOT NULL,
    chat_model TEXT NOT NULL,
    embedding_model TEXT NOT NULL,
    prompt_template_version TEXT NOT NULL,
    model_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    query TEXT NOT NULL,
    answer TEXT,
    citations JSONB NOT NULL DEFAULT '[]'::jsonb,
    error_summary TEXT,
    actor_id TEXT NOT NULL,
    causation_id TEXT,
    correlation_id TEXT,
    requested_event_id TEXT,
    completed_event_id TEXT,
    failed_event_id TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_agent_runs_status_check
        CHECK (status IN ('requested', 'completed', 'failed')),
    CONSTRAINT ai_agent_runs_agent_id_not_empty CHECK (length(trim(agent_id)) > 0),
    CONSTRAINT ai_agent_runs_chat_model_not_empty CHECK (length(trim(chat_model)) > 0),
    CONSTRAINT ai_agent_runs_embedding_model_not_empty CHECK (length(trim(embedding_model)) > 0),
    CONSTRAINT ai_agent_runs_prompt_template_version_not_empty
        CHECK (length(trim(prompt_template_version)) > 0),
    CONSTRAINT ai_agent_runs_query_not_empty CHECK (length(trim(query)) > 0),
    CONSTRAINT ai_agent_runs_actor_id_not_empty CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT ai_agent_runs_model_config_is_object CHECK (jsonb_typeof(model_config) = 'object'),
    CONSTRAINT ai_agent_runs_citations_is_array CHECK (jsonb_typeof(citations) = 'array')
);

CREATE INDEX IF NOT EXISTS ai_agent_runs_started_at_idx
    ON ai_agent_runs (started_at DESC, run_id);

CREATE INDEX IF NOT EXISTS ai_agent_runs_agent_status_idx
    ON ai_agent_runs (agent_id, status, started_at DESC);

CREATE TABLE IF NOT EXISTS semantic_embeddings (
    semantic_embedding_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    title TEXT NOT NULL,
    source_text TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    embedding_model TEXT NOT NULL,
    embedding_dimension INTEGER NOT NULL,
    embedding halfvec(2560) NOT NULL,
    graph_node_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT semantic_embeddings_source_kind_check
        CHECK (source_kind IN ('message', 'document', 'project', 'task', 'contact')),
    CONSTRAINT semantic_embeddings_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT semantic_embeddings_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT semantic_embeddings_source_text_not_empty CHECK (length(trim(source_text)) > 0),
    CONSTRAINT semantic_embeddings_content_hash_not_empty CHECK (length(trim(content_hash)) > 0),
    CONSTRAINT semantic_embeddings_model_not_empty CHECK (length(trim(embedding_model)) > 0),
    CONSTRAINT semantic_embeddings_dimension_check CHECK (embedding_dimension = 2560),
    UNIQUE (source_kind, source_id, embedding_model)
);

CREATE INDEX IF NOT EXISTS semantic_embeddings_source_idx
    ON semantic_embeddings (source_kind, source_id);

CREATE INDEX IF NOT EXISTS semantic_embeddings_model_idx
    ON semantic_embeddings (embedding_model, updated_at DESC);

CREATE INDEX IF NOT EXISTS semantic_embeddings_embedding_hnsw_idx
    ON semantic_embeddings
    USING hnsw (embedding halfvec_cosine_ops);

ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS agent_run_id TEXT
        REFERENCES ai_agent_runs(run_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS task_candidates_agent_run_idx
    ON task_candidates (agent_run_id)
    WHERE agent_run_id IS NOT NULL;
```

### `backend/migrations/0019_rebuild_graph_projection_after_v3.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0019_rebuild_graph_projection_after_v3.sql`
- Size bytes / Размер в байтах: `296`
- Included characters / Включено символов: `296`
- Truncated / Обрезано: `no`

```text
-- Graph tables are derived projection state per ADR-0045. Rebuilding them during
-- the V3 pgvector upgrade avoids carrying forward corrupted local projection rows
-- from earlier dev smoke runs while preserving canonical source records.
TRUNCATE TABLE graph_evidence, graph_edges, graph_nodes;
```

### `backend/migrations/0020_create_v4_telegram_policy_calls.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0020_create_v4_telegram_policy_calls.sql`
- Size bytes / Размер в байтах: `8949`
- Included characters / Включено символов: `8949`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_provider_accounts
    DROP CONSTRAINT IF EXISTS communication_provider_account_kind;

ALTER TABLE communication_provider_accounts
    ADD CONSTRAINT communication_provider_account_kind CHECK (
        provider_kind IN ('gmail', 'icloud', 'imap', 'telegram_user', 'telegram_bot')
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
            'telegram_bot_token'
        )
    );

ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS channel_kind TEXT NOT NULL DEFAULT 'email',
    ADD COLUMN IF NOT EXISTS conversation_id TEXT,
    ADD COLUMN IF NOT EXISTS sender_display_name TEXT,
    ADD COLUMN IF NOT EXISTS delivery_state TEXT NOT NULL DEFAULT 'received',
    ADD COLUMN IF NOT EXISTS message_metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN ('email', 'telegram_user', 'telegram_bot')
    ),
    ADD CONSTRAINT communication_messages_delivery_state CHECK (
        delivery_state IN ('received', 'sent', 'send_dry_run', 'send_blocked')
    ),
    ADD CONSTRAINT communication_messages_metadata_is_object CHECK (
        jsonb_typeof(message_metadata) = 'object'
    );

CREATE TABLE IF NOT EXISTS telegram_chats (
    telegram_chat_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id TEXT NOT NULL,
    chat_kind TEXT NOT NULL,
    title TEXT NOT NULL,
    username TEXT,
    sync_state TEXT NOT NULL DEFAULT 'fixture',
    last_message_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_chats_provider_chat_id_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_chats_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT telegram_chats_kind CHECK (chat_kind IN ('private', 'group', 'channel', 'bot')),
    CONSTRAINT telegram_chats_sync_state CHECK (sync_state IN ('fixture', 'syncing', 'synced', 'degraded', 'error')),
    CONSTRAINT telegram_chats_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_chats_account_provider_unique UNIQUE (account_id, provider_chat_id)
);

CREATE INDEX IF NOT EXISTS telegram_chats_account_idx
    ON telegram_chats (account_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS automation_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    body_template TEXT NOT NULL,
    required_variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT automation_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT automation_templates_body_not_empty CHECK (length(trim(body_template)) > 0),
    CONSTRAINT automation_templates_required_variables_is_array CHECK (jsonb_typeof(required_variables) = 'array')
);

CREATE TABLE IF NOT EXISTS automation_policies (
    policy_id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES automation_templates(template_id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    allowed_chat_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    trigger_kind TEXT NOT NULL,
    max_sends_per_hour INTEGER NOT NULL,
    quiet_hours JSONB NOT NULL DEFAULT '{}'::jsonb,
    expires_at TIMESTAMPTZ,
    conditions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT automation_policies_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT automation_policies_trigger_kind_not_empty CHECK (length(trim(trigger_kind)) > 0),
    CONSTRAINT automation_policies_allowed_chat_ids_is_array CHECK (jsonb_typeof(allowed_chat_ids) = 'array'),
    CONSTRAINT automation_policies_quiet_hours_is_object CHECK (jsonb_typeof(quiet_hours) = 'object'),
    CONSTRAINT automation_policies_conditions_is_object CHECK (jsonb_typeof(conditions) = 'object'),
    CONSTRAINT automation_policies_max_sends_positive CHECK (max_sends_per_hour > 0)
);

CREATE INDEX IF NOT EXISTS automation_policies_account_idx
    ON automation_policies (account_id, enabled, updated_at DESC);

CREATE TABLE IF NOT EXISTS telegram_outbound_messages (
    outbound_message_id TEXT PRIMARY KEY,
    policy_id TEXT REFERENCES automation_policies(policy_id) ON DELETE RESTRICT,
    template_id TEXT REFERENCES automation_templates(template_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id TEXT NOT NULL,
    send_mode TEXT NOT NULL,
    status TEXT NOT NULL,
    rendered_preview_hash TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_context JSONB NOT NULL DEFAULT '{}'::jsonb,
    actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_outbound_provider_chat_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_outbound_send_mode CHECK (send_mode IN ('dry_run', 'live')),
    CONSTRAINT telegram_outbound_status CHECK (status IN ('allowed', 'blocked', 'sent', 'failed')),
    CONSTRAINT telegram_outbound_preview_hash_not_empty CHECK (length(trim(rendered_preview_hash)) > 0),
    CONSTRAINT telegram_outbound_variables_is_object CHECK (jsonb_typeof(variables) = 'object'),
    CONSTRAINT telegram_outbound_source_context_is_object CHECK (jsonb_typeof(source_context) = 'object'),
    CONSTRAINT telegram_outbound_actor_not_empty CHECK (length(trim(actor_id)) > 0)
);

CREATE INDEX IF NOT EXISTS telegram_outbound_policy_idx
    ON telegram_outbound_messages (policy_id, created_at DESC);

CREATE TABLE IF NOT EXISTS telegram_calls (
    call_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_call_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    call_state TEXT NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    transcription_policy_id TEXT REFERENCES automation_policies(policy_id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_calls_provider_call_id_not_empty CHECK (length(trim(provider_call_id)) > 0),
    CONSTRAINT telegram_calls_provider_chat_id_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_calls_direction CHECK (direction IN ('incoming', 'outgoing')),
    CONSTRAINT telegram_calls_state CHECK (call_state IN ('ringing', 'active', 'ended', 'missed', 'declined', 'failed')),
    CONSTRAINT telegram_calls_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_calls_account_provider_unique UNIQUE (account_id, provider_call_id)
);

CREATE INDEX IF NOT EXISTS telegram_calls_account_idx
    ON telegram_calls (account_id, created_at DESC);

CREATE TABLE IF NOT EXISTS call_transcripts (
    transcript_id TEXT PRIMARY KEY,
    call_id TEXT NOT NULL REFERENCES telegram_calls(call_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id TEXT NOT NULL,
    transcript_status TEXT NOT NULL,
    stt_provider TEXT NOT NULL,
    source_audio_ref TEXT,
    language_code TEXT,
    transcript_text TEXT NOT NULL,
    segments JSONB NOT NULL DEFAULT '[]'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT call_transcripts_provider_chat_id_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT call_transcripts_status CHECK (transcript_status IN ('queued', 'running', 'succeeded', 'failed')),
    CONSTRAINT call_transcripts_stt_provider_not_empty CHECK (length(trim(stt_provider)) > 0),
    CONSTRAINT call_transcripts_segments_is_array CHECK (jsonb_typeof(segments) = 'array'),
    CONSTRAINT call_transcripts_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS call_transcripts_call_idx
    ON call_transcripts (call_id, created_at DESC);
```

### `backend/migrations/0021_create_v5_whatsapp_web_foundation.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0021_create_v5_whatsapp_web_foundation.sql`
- Size bytes / Размер в байтах: `2562`
- Included characters / Включено символов: `2562`
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
            'whatsapp_web'
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
            'whatsapp_web_session_key'
        )
    );

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_channel_kind;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN ('email', 'telegram_user', 'telegram_bot', 'whatsapp_web')
    );

CREATE TABLE IF NOT EXISTS whatsapp_web_sessions (
    session_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    device_name TEXT NOT NULL,
    companion_runtime TEXT NOT NULL DEFAULT 'fixture',
    link_state TEXT NOT NULL DEFAULT 'fixture',
    local_state_path TEXT NOT NULL,
    last_sync_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT whatsapp_web_sessions_account_unique UNIQUE (account_id),
    CONSTRAINT whatsapp_web_sessions_device_name_not_empty CHECK (length(trim(device_name)) > 0),
    CONSTRAINT whatsapp_web_sessions_runtime CHECK (
        companion_runtime IN ('fixture', 'manual_webview', 'blocked')
    ),
    CONSTRAINT whatsapp_web_sessions_link_state CHECK (
        link_state IN ('fixture', 'qr_pending', 'linked', 'degraded', 'revoked', 'blocked')
    ),
    CONSTRAINT whatsapp_web_sessions_local_state_path_not_empty CHECK (length(trim(local_state_path)) > 0),
    CONSTRAINT whatsapp_web_sessions_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS whatsapp_web_sessions_state_idx
    ON whatsapp_web_sessions (link_state, updated_at DESC);
```

### `backend/migrations/0022_create_database_encrypted_secret_vault.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0022_create_database_encrypted_secret_vault.sql`
- Size bytes / Размер в байтах: `1319`
- Included characters / Включено символов: `1319`
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
            'external_vault',
            'test_double'
        )
    );

CREATE TABLE IF NOT EXISTS encrypted_secret_vault_entries (
    secret_ref TEXT PRIMARY KEY REFERENCES secret_references(secret_ref) ON DELETE RESTRICT,
    kdf TEXT NOT NULL,
    salt TEXT NOT NULL,
    nonce TEXT NOT NULL,
    ciphertext TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT encrypted_secret_vault_entries_ref_not_empty CHECK (length(trim(secret_ref)) > 0),
    CONSTRAINT encrypted_secret_vault_entries_kdf CHECK (kdf IN ('argon2id:v1')),
    CONSTRAINT encrypted_secret_vault_entries_salt_not_empty CHECK (length(trim(salt)) > 0),
    CONSTRAINT encrypted_secret_vault_entries_nonce_not_empty CHECK (length(trim(nonce)) > 0),
    CONSTRAINT encrypted_secret_vault_entries_ciphertext_not_empty CHECK (length(trim(ciphertext)) > 0)
);

CREATE INDEX IF NOT EXISTS encrypted_secret_vault_entries_updated_idx
    ON encrypted_secret_vault_entries (updated_at);
```

### `backend/migrations/0023_create_application_settings.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0023_create_application_settings.sql`
- Size bytes / Размер в байтах: `4158`
- Included characters / Включено символов: `4158`
- Truncated / Обрезано: `no`

```text
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
```

### `backend/migrations/0024_seed_runtime_application_settings.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0024_seed_runtime_application_settings.sql`
- Size bytes / Размер в байтах: `1280`
- Included characters / Включено символов: `1280`
- Truncated / Обрезано: `no`

```text
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
```

### `backend/migrations/0025_add_message_workflow_state.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0025_add_message_workflow_state.sql`
- Size bytes / Размер в байтах: `1182`
- Included characters / Включено символов: `1182`
- Truncated / Обрезано: `no`

```text
ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS workflow_state TEXT NOT NULL DEFAULT 'new',
    ADD COLUMN IF NOT EXISTS importance_score SMALLINT,
    ADD COLUMN IF NOT EXISTS ai_category TEXT,
    ADD COLUMN IF NOT EXISTS ai_summary TEXT,
    ADD COLUMN IF NOT EXISTS ai_summary_generated_at TIMESTAMPTZ;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_workflow_state CHECK (
        workflow_state IN (
            'new',
            'reviewed',
            'needs_action',
            'waiting',
            'done',
            'archived',
            'muted',
            'spam'
        )
    );

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_importance_score_range CHECK (
        importance_score IS NULL OR (importance_score >= 0 AND importance_score <= 100)
    );

CREATE INDEX IF NOT EXISTS communication_messages_workflow_state_idx
    ON communication_messages (workflow_state, COALESCE(occurred_at, projected_at) DESC);

CREATE INDEX IF NOT EXISTS communication_messages_importance_idx
    ON communication_messages (importance_score DESC NULLS LAST)
    WHERE importance_score IS NOT NULL;
```
