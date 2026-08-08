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

- Chunk ID / ID чанка: `098-other-backend-part-005`
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

### `backend/migrations/0101_link_graph_evidence_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0101_link_graph_evidence_to_observations.sql`
- Size bytes / Размер в байтах: `2271`
- Included characters / Включено символов: `2271`
- Truncated / Обрезано: `no`

```text
ALTER TABLE graph_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE graph_evidence evidence
SET observation_id = message.observation_id
FROM communication_messages message
WHERE evidence.source_kind = 'message'
  AND evidence.source_id = message.message_id
  AND evidence.observation_id IS NULL;

UPDATE graph_evidence evidence
SET observation_id = raw.observation_id
FROM communication_raw_records raw
WHERE evidence.source_kind = 'raw_record'
  AND evidence.source_id = raw.raw_record_id
  AND evidence.observation_id IS NULL;

UPDATE graph_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.source_kind = 'observation'
  AND evidence.source_id = observation.observation_id
  AND evidence.observation_id IS NULL;

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;

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
            'obligation',
            'observation'
        )
    );

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_observation_fk;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_message_observation_required;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_message_observation_required CHECK (
        source_kind != 'message'
        OR observation_id IS NOT NULL
    );

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_observation_source_check;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

CREATE INDEX IF NOT EXISTS graph_evidence_observation_idx
    ON graph_evidence (observation_id)
    WHERE observation_id IS NOT NULL;
```

### `backend/migrations/0102_link_semantic_embeddings_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0102_link_semantic_embeddings_to_observations.sql`
- Size bytes / Размер в байтах: `1037`
- Included characters / Включено символов: `1037`
- Truncated / Обрезано: `no`

```text
ALTER TABLE semantic_embeddings
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE semantic_embeddings embedding
SET observation_id = message.observation_id
FROM communication_messages message
WHERE embedding.source_kind = 'message'
  AND embedding.source_id = message.message_id
  AND embedding.observation_id IS NULL;

ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_observation_fk;

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_message_observation_required;

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_message_observation_required CHECK (
        source_kind != 'message'
        OR observation_id IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS semantic_embeddings_observation_idx
    ON semantic_embeddings (observation_id)
    WHERE observation_id IS NOT NULL;
```

### `backend/migrations/0103_link_documents_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0103_link_documents_to_observations.sql`
- Size bytes / Размер в байтах: `2765`
- Included characters / Включено символов: `2765`
- Truncated / Обрезано: `no`

```text
ALTER TABLE documents
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
    'observation:v1:legacy-document:' || md5(
        COALESCE(document_id, '') || '|' || imported_at::text || '|' || source_fingerprint || '|' || COALESCE(extracted_text, '')
    ),
    kind.kind_definition_id,
    'file_import',
    NULL,
    imported_at,
    imported_at,
    jsonb_build_object(
        'legacy_document_id', document_id,
        'document_kind', document_kind,
        'title', title,
        'source_fingerprint', source_fingerprint,
        'extracted_text', extracted_text,
        'legacy_backfill', true
    ),
    1.0,
    'sha256:' || md5(
        COALESCE(document_id, '') || '|' || COALESCE(document_kind, '') || '|' || COALESCE(title, '') || '|' ||
        COALESCE(source_fingerprint, '') || '|' || COALESCE(extracted_text, '')
    ),
    'document://' || document_id,
    jsonb_build_object('legacy_backfill', true)
FROM documents
LEFT JOIN observation_kind_definitions kind
  ON kind.code = 'DOCUMENT'
 AND kind.version = 1
WHERE documents.observation_id IS NULL
  AND kind.kind_definition_id IS NOT NULL
ON CONFLICT (observation_id) DO NOTHING;

UPDATE documents
SET observation_id = 'observation:v1:legacy-document:' || md5(
    COALESCE(documents.document_id, '') || '|' || documents.imported_at::text || '|' || documents.source_fingerprint || '|' || COALESCE(documents.extracted_text, '')
)
FROM observation_kind_definitions kind
WHERE documents.observation_id IS NULL
  AND kind.code = 'DOCUMENT'
  AND kind.version = 1
  AND EXISTS (
        SELECT 1
        FROM observations ob
        WHERE ob.observation_id = 'observation:v1:legacy-document:' || md5(
            COALESCE(documents.document_id, '') || '|' || documents.imported_at::text || '|' || documents.source_fingerprint || '|' || COALESCE(documents.extracted_text, '')
        )
        AND ob.source_ref = 'document://' || documents.document_id
    );

ALTER TABLE documents
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE documents
    DROP CONSTRAINT IF EXISTS documents_observation_fk;

ALTER TABLE documents
    ADD CONSTRAINT documents_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE documents
    DROP CONSTRAINT IF EXISTS documents_source_kind_observation_check;

ALTER TABLE documents
    ADD CONSTRAINT documents_source_kind_observation_check CHECK (
        observation_id IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS documents_observation_idx
    ON documents (observation_id);
```

### `backend/migrations/0104_add_task_provenance_reference_guard.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0104_add_task_provenance_reference_guard.sql`
- Size bytes / Размер в байтах: `1727`
- Included characters / Включено символов: `1727`
- Truncated / Обрезано: `no`

```text
-- Enforce that task provenance references an existing record.
CREATE OR REPLACE FUNCTION enforce_task_provenance_target()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.provenance_kind = 'observation' THEN
        IF NOT EXISTS (SELECT 1 FROM observations WHERE observation_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing observation'
                USING ERRCODE = '23503';
        END IF;
    ELSIF NEW.provenance_kind = 'review_item' THEN
        IF NOT EXISTS (SELECT 1 FROM review_items WHERE review_item_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing review item'
                USING ERRCODE = '23503';
        END IF;
    ELSIF NEW.provenance_kind = 'decision' THEN
        IF NOT EXISTS (SELECT 1 FROM decisions WHERE decision_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing decision'
                USING ERRCODE = '23503';
        END IF;
    ELSIF NEW.provenance_kind = 'obligation' THEN
        IF NOT EXISTS (SELECT 1 FROM obligations WHERE obligation_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing obligation'
                USING ERRCODE = '23503';
        END IF;
    ELSE
        RAISE EXCEPTION 'unsupported tasks.provenance_kind value'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS tasks_provenance_target_guard ON tasks;
CREATE TRIGGER tasks_provenance_target_guard
    BEFORE INSERT OR UPDATE OF provenance_kind, provenance_id ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION enforce_task_provenance_target();
```

### `backend/migrations/0105_translate_task_candidates_to_observation_identity.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0105_translate_task_candidates_to_observation_identity.sql`
- Size bytes / Размер в байтах: `1240`
- Included characters / Включено символов: `1240`
- Truncated / Обрезано: `no`

```text
UPDATE task_candidates candidate
SET observation_id = message.observation_id
FROM communication_messages message
WHERE candidate.source_kind = 'message'
  AND candidate.source_id = message.message_id
  AND candidate.observation_id IS NULL;

UPDATE task_candidates candidate
SET observation_id = document.observation_id
FROM documents document
WHERE candidate.source_kind = 'document'
  AND candidate.source_id = document.document_id
  AND candidate.observation_id IS NULL;

UPDATE task_candidates
SET
    source_kind = 'observation',
    source_id = observation_id
WHERE observation_id IS NOT NULL
  AND source_kind IN ('message', 'document');

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_source_kind_check;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_source_kind_check
    CHECK (source_kind IN ('observation'));

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_message_observation_required;

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_observation_required;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_observation_required CHECK (
        observation_id IS NOT NULL
        AND source_id = observation_id
    );
```

### `backend/migrations/0106_link_calendar_events_to_observations.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0106_link_calendar_events_to_observations.sql`
- Size bytes / Размер в байтах: `4054`
- Included characters / Включено символов: `4054`
- Truncated / Обрезано: `no`

```text
ALTER TABLE calendar_events
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
    'observation:v1:legacy-calendar-event:' || md5(
        COALESCE(calendar_events.event_id, '') || '|' ||
        calendar_events.start_at::text || '|' ||
        calendar_events.end_at::text || '|' ||
        COALESCE(calendar_events.title, '') || '|' ||
        COALESCE(calendar_events.description, '') || '|' ||
        COALESCE(calendar_events.location, '')
    ),
    kind.kind_definition_id,
    'local_runtime',
    NULL,
    calendar_events.start_at,
    calendar_events.created_at,
    jsonb_build_object(
        'legacy_event_id', calendar_events.event_id,
        'source_event_id', calendar_events.source_event_id,
        'account_id', calendar_events.account_id,
        'source_id', calendar_events.source_id,
        'title', calendar_events.title,
        'description', calendar_events.description,
        'location', calendar_events.location,
        'start_at', calendar_events.start_at,
        'end_at', calendar_events.end_at,
        'timezone', calendar_events.timezone,
        'all_day', calendar_events.all_day,
        'recurrence_rule', calendar_events.recurrence_rule,
        'status', calendar_events.status,
        'visibility', calendar_events.visibility,
        'event_type', calendar_events.event_type,
        'conference_url', calendar_events.conference_url,
        'conference_provider', calendar_events.conference_provider,
        'preparation_reminder_minutes', calendar_events.preparation_reminder_minutes,
        'travel_buffer_minutes', calendar_events.travel_buffer_minutes,
        'legacy_backfill', true
    ),
    1.0,
    'sha256:' || md5(
        COALESCE(calendar_events.event_id, '') || '|' ||
        calendar_events.start_at::text || '|' ||
        calendar_events.end_at::text || '|' ||
        COALESCE(calendar_events.title, '') || '|' ||
        COALESCE(calendar_events.description, '') || '|' ||
        COALESCE(calendar_events.location, '')
    ),
    'calendar_event://' || calendar_events.event_id,
    jsonb_build_object(
        'legacy_backfill', true,
        'ingested_by', 'calendar_events_domain'
    )
FROM calendar_events
LEFT JOIN observation_kind_definitions kind
  ON kind.code = 'CALENDAR_EVENT'
 AND kind.version = 1
WHERE calendar_events.observation_id IS NULL
  AND kind.kind_definition_id IS NOT NULL
ON CONFLICT (observation_id) DO NOTHING;

UPDATE calendar_events
SET observation_id = 'observation:v1:legacy-calendar-event:' || md5(
    COALESCE(calendar_events.event_id, '') || '|' || calendar_events.start_at::text || '|' ||
    calendar_events.end_at::text || '|' || COALESCE(calendar_events.title, '') || '|' ||
    COALESCE(calendar_events.description, '') || '|' || COALESCE(calendar_events.location, '')
)
WHERE calendar_events.observation_id IS NULL
  AND EXISTS (
        SELECT 1
        FROM observations observation
        WHERE observation.observation_id = 'observation:v1:legacy-calendar-event:' || md5(
            COALESCE(calendar_events.event_id, '') || '|' || calendar_events.start_at::text || '|' ||
            calendar_events.end_at::text || '|' || COALESCE(calendar_events.title, '') || '|' ||
            COALESCE(calendar_events.description, '') || '|' || COALESCE(calendar_events.location, '')
        )
          AND observation.source_ref = 'calendar_event://' || calendar_events.event_id
    );

ALTER TABLE calendar_events
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE calendar_events
    DROP CONSTRAINT IF EXISTS calendar_events_observation_fk;

ALTER TABLE calendar_events
    ADD CONSTRAINT calendar_events_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

CREATE INDEX IF NOT EXISTS calendar_events_observation_idx
    ON calendar_events (observation_id);
```

### `backend/migrations/0107_add_event_participant_source.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0107_add_event_participant_source.sql`
- Size bytes / Размер в байтах: `192`
- Included characters / Включено символов: `192`
- Truncated / Обрезано: `no`

```text
ALTER TABLE event_participants
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';

UPDATE event_participants
SET source = 'manual'
WHERE source IS NULL OR btrim(source) = '';
```

### `backend/migrations/0108_add_calendar_reminder_source.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0108_add_calendar_reminder_source.sql`
- Size bytes / Размер в байтах: `192`
- Included characters / Включено символов: `192`
- Truncated / Обрезано: `no`

```text
ALTER TABLE calendar_reminders
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';

UPDATE calendar_reminders
SET source = 'manual'
WHERE source IS NULL OR btrim(source) = '';
```

### `backend/migrations/0109_add_task_subtask_source.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0109_add_task_subtask_source.sql`
- Size bytes / Размер в байтах: `183`
- Included characters / Включено символов: `183`
- Truncated / Обрезано: `no`

```text
ALTER TABLE task_subtasks
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';

UPDATE task_subtasks
SET source = 'manual'
WHERE source IS NULL OR btrim(source) = '';
```

### `backend/migrations/0110_add_communication_draft_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0110_add_communication_draft_observation_kind.sql`
- Size bytes / Размер в байтах: `575`
- Included characters / Включено символов: `575`
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
    'observation_kind:v1:communication_draft',
    'COMMUNICATION_DRAFT',
    'Communication draft',
    1,
    'communication',
    'Manual or provider-backed communication draft captured as evidence.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0111_add_communication_folder_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0111_add_communication_folder_observation_kind.sql`
- Size bytes / Размер в байтах: `579`
- Included characters / Включено символов: `579`
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
    'observation_kind:v1:communication_folder',
    'COMMUNICATION_FOLDER',
    'Communication folder',
    1,
    'communication',
    'Manual or provider-backed communication folder captured as evidence.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0112_add_communication_saved_search_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0112_add_communication_saved_search_observation_kind.sql`
- Size bytes / Размер в байтах: `604`
- Included characters / Включено символов: `604`
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
    'observation_kind:v1:communication_saved_search',
    'COMMUNICATION_SAVED_SEARCH',
    'Communication saved search',
    1,
    'communication',
    'Saved search or smart folder definition captured as communication evidence.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0113_add_communication_outbox_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0113_add_communication_outbox_observation_kind.sql`
- Size bytes / Размер в байтах: `591`
- Included characters / Включено символов: `591`
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
    'observation_kind:v1:communication_outbox',
    'COMMUNICATION_OUTBOX',
    'Communication outbox item',
    1,
    'communication',
    'Queued, scheduled, or canceled outbound communication captured as evidence.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0114_add_communication_delivery_status_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0114_add_communication_delivery_status_observation_kind.sql`
- Size bytes / Размер в байтах: `612`
- Included characters / Включено символов: `612`
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
    'observation_kind:v1:communication_delivery_status',
    'COMMUNICATION_DELIVERY_STATUS',
    'Communication delivery status',
    1,
    'communication',
    'Observed provider or parser delivery status for an outbound communication.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0115_add_communication_read_receipt_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0115_add_communication_read_receipt_observation_kind.sql`
- Size bytes / Размер в байтах: `581`
- Included characters / Включено символов: `581`
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
    'observation_kind:v1:communication_read_receipt',
    'COMMUNICATION_READ_RECEIPT',
    'Communication read receipt',
    1,
    'communication',
    'Observed read receipt for an outbound communication.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0116_add_contradiction_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0116_add_contradiction_observation_kind.sql`
- Size bytes / Размер в байтах: `581`
- Included characters / Включено символов: `581`
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
    'observation_kind:v1:contradiction_observation',
    'CONTRADICTION_OBSERVATION',
    'Contradiction Observation',
    1,
    'review',
    'Consistency engine contradiction evidence captured for review.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0117_add_ai_control_center_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0117_add_ai_control_center_observation_kinds.sql`
- Size bytes / Размер в байтах: `1055`
- Included characters / Включено символов: `1055`
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
VALUES
    (
        'observation_kind:v1:ai_provider_account',
        'AI_PROVIDER_ACCOUNT',
        'AI provider account',
        1,
        'ai',
        'AI control center provider account configuration captured as evidence.'
    ),
    (
        'observation_kind:v1:ai_provider_secret_binding',
        'AI_PROVIDER_SECRET_BINDING',
        'AI provider secret binding',
        1,
        'ai',
        'AI control center provider secret binding captured as evidence.'
    ),
    (
        'observation_kind:v1:ai_model_route',
        'AI_MODEL_ROUTE',
        'AI model route',
        1,
        'ai',
        'AI control center capability-slot routing captured as evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0118_add_telegram_command_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0118_add_telegram_command_observation_kinds.sql`
- Size bytes / Размер в байтах: `949`
- Included characters / Включено символов: `949`
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
VALUES
    (
        'observation_kind:v1:telegram_provider_write_command',
        'TELEGRAM_PROVIDER_WRITE_COMMAND',
        'Telegram provider write command',
        1,
        'telegram',
        'Telegram provider write command queued as durable action evidence.'
    ),
    (
        'observation_kind:v1:telegram_provider_write_command_status',
        'TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS',
        'Telegram provider write command status',
        1,
        'telegram',
        'Telegram provider write command lifecycle or reconciliation state captured as evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0119_add_ai_agent_run_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0119_add_ai_agent_run_observation_kinds.sql`
- Size bytes / Размер в байтах: `798`
- Included characters / Включено символов: `798`
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
VALUES
    (
        'observation_kind:v1:ai_agent_run',
        'AI_AGENT_RUN',
        'AI agent run',
        1,
        'ai',
        'AI agent run request captured as durable execution evidence.'
    ),
    (
        'observation_kind:v1:ai_agent_run_status',
        'AI_AGENT_RUN_STATUS',
        'AI agent run status',
        1,
        'ai',
        'AI agent run lifecycle state captured as durable execution evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0120_add_document_processing_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0120_add_document_processing_observation_kinds.sql`
- Size bytes / Размер в байтах: `888`
- Included characters / Включено символов: `888`
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
VALUES
    (
        'observation_kind:v1:document_processing_job',
        'DOCUMENT_PROCESSING_JOB',
        'Document processing job',
        1,
        'document',
        'Document processing job queued as durable execution evidence.'
    ),
    (
        'observation_kind:v1:document_processing_job_status',
        'DOCUMENT_PROCESSING_JOB_STATUS',
        'Document processing job status',
        1,
        'document',
        'Document processing job lifecycle state captured as durable execution evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0121_add_mail_sync_run_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0121_add_mail_sync_run_observation_kinds.sql`
- Size bytes / Размер в байтах: `696`
- Included characters / Включено символов: `696`
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
VALUES
    (
        'okd_communication_mail_sync_run_v1',
        'COMMUNICATION_MAIL_SYNC_RUN',
        'Communication Mail Sync Run',
        1,
        'communications',
        'Canonical evidence for mail background sync run creation.'
    ),
    (
        'okd_communication_mail_sync_run_status_v1',
        'COMMUNICATION_MAIL_SYNC_RUN_STATUS',
        'Communication Mail Sync Run Status',
        1,
        'communications',
        'Canonical evidence for mail background sync run lifecycle transitions.'
    )
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0122_add_ai_prompt_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0122_add_ai_prompt_observation_kinds.sql`
- Size bytes / Размер в байтах: `833`
- Included characters / Включено символов: `833`
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
VALUES
    (
        'okd_ai_prompt_template_v1',
        'AI_PROMPT_TEMPLATE',
        'AI Prompt Template',
        1,
        'ai',
        'Canonical evidence for AI prompt template lifecycle mutations.'
    ),
    (
        'okd_ai_prompt_template_version_v1',
        'AI_PROMPT_TEMPLATE_VERSION',
        'AI Prompt Template Version',
        1,
        'ai',
        'Canonical evidence for AI prompt template version lifecycle mutations.'
    ),
    (
        'okd_ai_prompt_eval_run_v1',
        'AI_PROMPT_EVAL_RUN',
        'AI Prompt Eval Run',
        1,
        'ai',
        'Canonical evidence for AI prompt preview and evaluation runs.'
    )
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0123_add_ai_model_catalog_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0123_add_ai_model_catalog_observation_kind.sql`
- Size bytes / Размер в байтах: `365`
- Included characters / Включено символов: `365`
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
    'okd_ai_model_catalog_item_v1',
    'AI_MODEL_CATALOG_ITEM',
    'AI Model Catalog Item',
    1,
    'ai',
    'Canonical evidence for AI curated model catalog materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0124_add_ai_semantic_embedding_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0124_add_ai_semantic_embedding_observation_kind.sql`
- Size bytes / Размер в байтах: `367`
- Included characters / Включено символов: `367`
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
    'okd_ai_semantic_embedding_v1',
    'AI_SEMANTIC_EMBEDDING',
    'AI Semantic Embedding',
    1,
    'ai',
    'Canonical evidence for derived semantic embedding materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0125_add_whatsapp_session_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0125_add_whatsapp_session_observation_kind.sql`
- Size bytes / Размер в байтах: `380`
- Included characters / Включено символов: `380`
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
    'okd_whatsapp_web_session_v1',
    'WHATSAPP_WEB_SESSION',
    'WhatsApp Web Session',
    1,
    'communications',
    'Canonical evidence for WhatsApp Web session lifecycle materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```
