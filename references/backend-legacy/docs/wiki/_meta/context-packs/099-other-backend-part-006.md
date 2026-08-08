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

- Chunk ID / ID чанка: `099-other-backend-part-006`
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

### `backend/migrations/0126_add_telegram_chat_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0126_add_telegram_chat_observation_kind.sql`
- Size bytes / Размер в байтах: `342`
- Included characters / Включено символов: `342`
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
    'okd_telegram_chat_v1',
    'TELEGRAM_CHAT',
    'Telegram Chat',
    1,
    'telegram',
    'Canonical evidence for Telegram chat state materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0127_add_telegram_chat_participant_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0127_add_telegram_chat_participant_observation_kind.sql`
- Size bytes / Размер в байтах: `391`
- Included characters / Включено символов: `391`
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
    'okd_telegram_chat_participant_v1',
    'TELEGRAM_CHAT_PARTICIPANT',
    'Telegram Chat Participant',
    1,
    'telegram',
    'Canonical evidence for Telegram chat participant roster materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0128_add_telegram_topic_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0128_add_telegram_topic_observation_kind.sql`
- Size bytes / Размер в байтах: `346`
- Included characters / Включено символов: `346`
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
    'okd_telegram_topic_v1',
    'TELEGRAM_TOPIC',
    'Telegram Topic',
    1,
    'telegram',
    'Canonical evidence for Telegram forum topic materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0129_add_telegram_message_reaction_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0129_add_telegram_message_reaction_observation_kind.sql`
- Size bytes / Размер в байтах: `390`
- Included characters / Включено символов: `390`
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
    'okd_telegram_message_reaction_v1',
    'TELEGRAM_MESSAGE_REACTION',
    'Telegram Message Reaction',
    1,
    'telegram',
    'Canonical evidence for Telegram message reaction state materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0130_add_telegram_message_lifecycle_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0130_add_telegram_message_lifecycle_observation_kinds.sql`
- Size bytes / Размер в байтах: `614`
- Included characters / Включено символов: `614`
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
    'okd_telegram_message_version_v1',
    'TELEGRAM_MESSAGE_VERSION',
    'Telegram Message Version',
    1,
    'telegram',
    'Canonical evidence for append-only Telegram message edit versions.'
),
(
    'okd_telegram_message_tombstone_v1',
    'TELEGRAM_MESSAGE_TOMBSTONE',
    'Telegram Message Tombstone',
    1,
    'telegram',
    'Canonical evidence for append-only Telegram message tombstones and visibility deletions.'
)
ON CONFLICT (code, version) DO NOTHING;
```

### `backend/migrations/0131_add_automation_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0131_add_automation_observation_kinds.sql`
- Size bytes / Размер в байтах: `1117`
- Included characters / Включено символов: `1117`
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
        'observation_kind:v1:automation_template',
        'AUTOMATION_TEMPLATE',
        'Automation template',
        1,
        'automation',
        'Automation template configuration captured as canonical evidence.'
    ),
    (
        'observation_kind:v1:automation_policy',
        'AUTOMATION_POLICY',
        'Automation policy',
        1,
        'automation',
        'Automation policy configuration captured as canonical evidence.'
    ),
    (
        'observation_kind:v1:telegram_outbound_message',
        'TELEGRAM_OUTBOUND_MESSAGE',
        'Telegram outbound message',
        1,
        'automation',
        'Automation dry-run or live outbound Telegram message materialization captured as canonical evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0132_add_review_transition_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0132_add_review_transition_observation_kind.sql`
- Size bytes / Размер в байтах: `630`
- Included characters / Включено символов: `630`
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
    'observation_kind:v1:review_transition',
    'REVIEW_TRANSITION',
    'Review transition',
    1,
    'review',
    'Manual review transition, approval, rejection, promotion, or similar user-driven review workflow change captured as canonical evidence.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0133_add_person_identity_candidate_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0133_add_person_identity_candidate_observation_kind.sql`
- Size bytes / Размер в байтах: `638`
- Included characters / Включено символов: `638`
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
    'observation_kind:v1:person_identity_candidate',
    'PERSON_IDENTITY_CANDIDATE',
    'Person identity candidate',
    1,
    'identity',
    'Synthetic but canonical evidence describing a person identity candidate generated for review and promotion workflows.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0134_add_project_link_review_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0134_add_project_link_review_observation_kind.sql`
- Size bytes / Размер в байтах: `607`
- Included characters / Включено символов: `607`
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
    'observation_kind:v1:project_link_review',
    'PROJECT_LINK_REVIEW',
    'Project link review',
    1,
    'review',
    'Canonical evidence describing a project link review event and its downstream review-state materialization.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0135_add_task_mutation_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0135_add_task_mutation_observation_kind.sql`
- Size bytes / Размер в байтах: `617`
- Included characters / Включено символов: `617`
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
    'observation_kind:v1:task_mutation',
    'TASK_MUTATION',
    'Task mutation',
    1,
    'tasks',
    'Canonical evidence describing a manual or local-runtime task mutation, task-local record change, or compatibility task materialization.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0136_add_person_mutation_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0136_add_person_mutation_observation_kind.sql`
- Size bytes / Размер в байтах: `662`
- Included characters / Включено символов: `662`
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
    'observation_kind:v1:person_mutation',
    'PERSON_MUTATION',
    'Person mutation',
    1,
    'persons',
    'Canonical evidence describing a manual mutation of a persona or person-centric profile state such as owner assignment, persona update, favorite toggle, or watchlist toggle.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0137_add_organization_mutation_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0137_add_organization_mutation_observation_kind.sql`
- Size bytes / Размер в байтах: `626`
- Included characters / Включено символов: `626`
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
    'observation_kind:v1:organization_mutation',
    'ORGANIZATION_MUTATION',
    'Organization mutation',
    1,
    'organizations',
    'Canonical evidence describing a manual mutation of an organization aggregate such as create, update, or archive.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0138_add_organization_record_mutation_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0138_add_organization_record_mutation_observation_kind.sql`
- Size bytes / Размер в байтах: `691`
- Included characters / Включено символов: `691`
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
    'observation_kind:v1:organization_record_mutation',
    'ORGANIZATION_RECORD_MUTATION',
    'Organization record mutation',
    1,
    'organizations',
    'Canonical evidence describing a manual mutation of subordinate organization records such as identities, aliases, departments, or organization contact links.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0139_add_person_record_mutation_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0139_add_person_record_mutation_observation_kind.sql`
- Size bytes / Размер в байтах: `723`
- Included characters / Включено символов: `723`
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
    'observation_kind:v1:person_record_mutation',
    'PERSON_RECORD_MUTATION',
    'Person record mutation',
    1,
    'persons',
    'Canonical evidence describing a manual mutation of subordinate person records such as identity traces, identities, compatibility roles, compatibility personas, facts, preferences, or relationship timeline events.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0140_add_vault_owner_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0140_add_vault_owner_observation_kinds.sql`
- Size bytes / Размер в байтах: `1374`
- Included characters / Включено символов: `1374`
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
    'observation_kind:v1:calendar_account_link',
    'CALENDAR_ACCOUNT_LINK',
    'Calendar account link',
    1,
    'vault',
    'Canonical evidence describing a linked provider calendar account materialized through the vault owner boundary.'
),
(
    'observation_kind:v1:task_provider_account',
    'TASK_PROVIDER_ACCOUNT',
    'Task provider account',
    1,
    'vault',
    'Canonical evidence describing creation of a vault-owned task provider account.'
),
(
    'observation_kind:v1:communication_provider_account',
    'COMMUNICATION_PROVIDER_ACCOUNT',
    'Communication provider account',
    1,
    'vault',
    'Canonical evidence describing an upsert of a vault-owned communication provider account.'
),
(
    'observation_kind:v1:communication_provider_secret_binding',
    'COMMUNICATION_PROVIDER_SECRET_BINDING',
    'Communication provider secret binding',
    1,
    'vault',
    'Canonical evidence describing a vault-owned communication provider account secret binding mutation.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0141_add_calendar_account_mutation_observation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0141_add_calendar_account_mutation_observation_kind.sql`
- Size bytes / Размер в байтах: `649`
- Included characters / Включено символов: `649`
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
    'observation_kind:v1:calendar_account_mutation',
    'CALENDAR_ACCOUNT_MUTATION',
    'Calendar account mutation',
    1,
    'calendar',
    'Canonical evidence describing a manual mutation of a calendar account aggregate such as create, update, delete, or sync trigger.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0142_add_vault_removal_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0142_add_vault_removal_observation_kinds.sql`
- Size bytes / Размер в байтах: `979`
- Included characters / Включено символов: `979`
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
    'observation_kind:v1:communication_provider_account_deleted',
    'COMMUNICATION_PROVIDER_ACCOUNT_DELETED',
    'Communication provider account deleted',
    1,
    'vault',
    'Canonical evidence describing deletion of vault-owned communication provider account metadata.'
),
(
    'observation_kind:v1:communication_provider_secret_binding_removed',
    'COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED',
    'Communication provider secret binding removed',
    1,
    'vault',
    'Canonical evidence describing removal of a vault-owned communication provider secret binding during metadata cleanup.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0143_add_communication_provider_account_config_mutation_kind.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0143_add_communication_provider_account_config_mutation_kind.sql`
- Size bytes / Размер в байтах: `683`
- Included characters / Включено символов: `683`
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
    'observation_kind:v1:communication_provider_account_config_mutation',
    'COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION',
    'Communication provider account config mutation',
    1,
    'vault',
    'Canonical evidence describing an update of vault-owned communication provider account config metadata.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0144_add_person_calendar_document_replacement_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0144_add_person_calendar_document_replacement_kinds.sql`
- Size bytes / Размер в байтах: `1403`
- Included characters / Включено символов: `1403`
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
    'observation_kind:v1:person_memory_card',
    'PERSON_MEMORY_CARD',
    'Person memory card',
    1,
    'persons',
    'Canonical evidence describing a manual person memory note or memory card captured into persona memory.'
),
(
    'observation_kind:v1:event_agenda',
    'EVENT_AGENDA',
    'Event agenda',
    1,
    'calendar',
    'Canonical evidence describing a manual agenda captured for a calendar event.'
),
(
    'observation_kind:v1:event_checklist',
    'EVENT_CHECKLIST',
    'Event checklist',
    1,
    'calendar',
    'Canonical evidence describing a manual checklist captured for a calendar event.'
),
(
    'observation_kind:v1:meeting_note',
    'MEETING_NOTE',
    'Meeting note',
    1,
    'calendar',
    'Canonical evidence describing a manual meeting note captured for a calendar event.'
),
(
    'observation_kind:v1:calendar_rule',
    'CALENDAR_RULE',
    'Calendar rule',
    1,
    'calendar',
    'Canonical evidence describing a manual create, update, or delete mutation of a calendar rule.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0145_add_person_derived_evidence_observation_kinds.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0145_add_person_derived_evidence_observation_kinds.sql`
- Size bytes / Размер в байтах: `1066`
- Included characters / Включено символов: `1066`
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
    'observation_kind:v1:person_role',
    'PERSON_ROLE',
    'Person role',
    1,
    'persons',
    'Canonical evidence describing a person role assignment or removal materialized as compatibility knowledge and relationship evidence.'
),
(
    'observation_kind:v1:person_trust_signal',
    'PERSON_TRUST_SIGNAL',
    'Person trust signal',
    1,
    'persons',
    'Canonical evidence describing a derived trust signal for a persona relationship materialized from person enrichment.'
),
(
    'observation_kind:v1:person_promise',
    'PERSON_PROMISE',
    'Person promise',
    1,
    'persons',
    'Canonical evidence describing a persona promise that is projected into an obligation.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
```

### `backend/migrations/0146_expand_review_item_kind_constraint.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0146_expand_review_item_kind_constraint.sql`
- Size bytes / Размер в байтах: `569`
- Included characters / Включено символов: `569`
- Truncated / Обрезано: `no`

```text
ALTER TABLE review_items
    DROP CONSTRAINT IF EXISTS review_items_item_kind;

ALTER TABLE review_items
    ADD CONSTRAINT review_items_item_kind CHECK (
        item_kind IN (
            'new_person',
            'new_organization',
            'identity_candidate',
            'project_link_candidate',
            'contradiction_candidate',
            'potential_task',
            'potential_obligation',
            'potential_decision',
            'potential_relationship',
            'potential_project',
            'knowledge_candidate'
        )
    );
```

### `backend/migrations/0147_allow_observation_task_sources.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0147_allow_observation_task_sources.sql`
- Size bytes / Размер в байтах: `672`
- Included characters / Включено символов: `672`
- Truncated / Обрезано: `no`

```text
ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_kind_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_kind_check CHECK (
        source_kind IN (
            'manual',
            'observation',
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

### `backend/migrations/0148_allow_observation_task_source_type.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0148_allow_observation_task_source_type.sql`
- Size bytes / Размер в байтах: `678`
- Included characters / Включено символов: `678`
- Truncated / Обрезано: `no`

```text
ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_type_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_type_check CHECK (
        source_type IN (
            'manual',
            'observation',
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

### `backend/migrations/0149_create_canonical_communication_tables.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0149_create_canonical_communication_tables.sql`
- Size bytes / Размер в байтах: `33331`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
-- Canonical Communications tables.
--
-- Historical provider-prefixed tables remain for upgrade compatibility, but
-- communication-owned durable state is mirrored into provider-neutral tables.

CREATE TABLE IF NOT EXISTS communication_accounts (
    account_id TEXT PRIMARY KEY,
    provider_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    external_account_id TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_accounts_id_not_empty CHECK (length(trim(account_id)) > 0),
    CONSTRAINT communication_accounts_provider_kind_not_empty CHECK (length(trim(provider_kind)) > 0),
    CONSTRAINT communication_accounts_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_accounts_external_id_not_empty CHECK (length(trim(external_account_id)) > 0),
    CONSTRAINT communication_accounts_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT communication_accounts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_accounts_provider_idx
    ON communication_accounts (provider_kind, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_channels (
    channel_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL,
    provider_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    runtime_state TEXT NOT NULL DEFAULT 'metadata_only',
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_channels_id_not_empty CHECK (length(trim(channel_id)) > 0),
    CONSTRAINT communication_channels_kind_not_empty CHECK (length(trim(channel_kind)) > 0),
    CONSTRAINT communication_channels_provider_not_empty CHECK (length(trim(provider_kind)) > 0),
    CONSTRAINT communication_channels_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_channels_runtime_state_not_empty CHECK (length(trim(runtime_state)) > 0),
    CONSTRAINT communication_channels_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT communication_channels_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_channels_account_idx
    ON communication_channels (account_id, channel_kind);

CREATE TABLE IF NOT EXISTS communication_identities (
    identity_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_id TEXT REFERENCES communication_channels(channel_id) ON DELETE CASCADE,
    identity_kind TEXT NOT NULL,
    provider_identity_id TEXT NOT NULL,
    display_name TEXT,
    address TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_identities_id_not_empty CHECK (length(trim(identity_id)) > 0),
    CONSTRAINT communication_identities_kind_not_empty CHECK (length(trim(identity_kind)) > 0),
    CONSTRAINT communication_identities_provider_id_not_empty CHECK (length(trim(provider_identity_id)) > 0),
    CONSTRAINT communication_identities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_identities_provider_unique UNIQUE (account_id, identity_kind, provider_identity_id)
);

CREATE TABLE IF NOT EXISTS communication_conversations (
    conversation_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_id TEXT REFERENCES communication_channels(channel_id) ON DELETE SET NULL,
    channel_kind TEXT NOT NULL,
    provider_conversation_id TEXT,
    title TEXT,
    last_message_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_conversations_id_not_empty CHECK (length(trim(conversation_id)) > 0),
    CONSTRAINT communication_conversations_channel_kind_not_empty CHECK (length(trim(channel_kind)) > 0),
    CONSTRAINT communication_conversations_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_conversations_account_idx
    ON communication_conversations (account_id, last_message_at DESC NULLS LAST);

CREATE TABLE IF NOT EXISTS communication_conversation_participants (
    participant_id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES communication_conversations(conversation_id) ON DELETE CASCADE,
    identity_id TEXT REFERENCES communication_identities(identity_id) ON DELETE SET NULL,
    person_id TEXT REFERENCES persons(person_id) ON DELETE SET NULL,
    role TEXT NOT NULL,
    display_name TEXT,
    address TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_conversation_participants_id_not_empty CHECK (length(trim(participant_id)) > 0),
    CONSTRAINT communication_conversation_participants_role_not_empty CHECK (length(trim(role)) > 0),
    CONSTRAINT communication_conversation_participants_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_conversation_participants_conversation_idx
    ON communication_conversation_participants (conversation_id, role);

CREATE TABLE IF NOT EXISTS communication_message_versions (
    version_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    version_number INTEGER NOT NULL,
    body_text TEXT,
    edited_at TIMESTAMPTZ NOT NULL,
    source_event TEXT,
    diff_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_versions_version_positive CHECK (version_number > 0),
    CONSTRAINT communication_message_versions_diff_is_object CHECK (jsonb_typeof(diff_payload) = 'object'),
    CONSTRAINT communication_message_versions_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT communication_message_versions_unique UNIQUE (message_id, version_number)
);

CREATE TABLE IF NOT EXISTS communication_message_tombstones (
    tombstone_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    reason_class TEXT NOT NULL,
    actor_class TEXT NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    source_event TEXT,
    is_provider_delete BOOLEAN NOT NULL DEFAULT false,
    is_local_visible BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_tombstones_reason_not_empty CHECK (length(trim(reason_class)) > 0),
    CONSTRAINT communication_message_tombstones_actor_not_empty CHECK (length(trim(actor_class)) > 0),
    CONSTRAINT communication_message_tombstones_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_message_tombstones_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_message_reactions (
    reaction_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    sender_identity_id TEXT,
    sender_display_name TEXT,
    reaction TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    observed_at TIMESTAMPTZ NOT NULL,
    source_event TEXT,
    provider_actor_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_reactions_reaction_not_empty CHECK (length(trim(reaction)) > 0),
    CONSTRAINT communication_message_reactions_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_message_reactions_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_message_reactions_message_idx
    ON communication_message_reactions (message_id, is_active, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_message_refs (
    message_ref_id TEXT PRIMARY KEY,
    ref_kind TEXT NOT NULL,
    source_message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    target_message_id TEXT REFERENCES communication_messages(message_id) ON DELETE SET NULL,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_conversation_id TEXT,
    source_provider_id TEXT,
    target_provider_id TEXT,
    depth INTEGER NOT NULL DEFAULT 1,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_refs_kind CHECK (ref_kind IN ('reply', 'forward')),
    CONSTRAINT communication_message_refs_depth_positive CHECK (depth > 0),
    CONSTRAINT communication_message_refs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_message_refs_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_message_refs_source_idx
    ON communication_message_refs (source_message_id, ref_kind, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_folders (
    folder_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_folders_id_not_empty CHECK (length(trim(folder_id)) > 0),
    CONSTRAINT communication_folders_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_folders_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_folder_messages (
    folder_id TEXT NOT NULL REFERENCES communication_folders(folder_id) ON DELETE CASCADE,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_operation TEXT NOT NULL DEFAULT 'copy',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (folder_id, message_id)
);

CREATE TABLE IF NOT EXISTS communication_saved_searches (
    saved_search_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT,
    name TEXT NOT NULL,
    description TEXT,
    query_text TEXT NOT NULL DEFAULT '',
    workflow_state TEXT,
    local_state TEXT NOT NULL DEFAULT 'active',
    is_smart_folder BOOLEA
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/migrations/0150_create_communication_ai_states.sql`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/migrations/0150_create_communication_ai_states.sql`
- Size bytes / Размер в байтах: `1188`
- Included characters / Включено символов: `1188`
- Truncated / Обрезано: `no`

```text
CREATE TABLE IF NOT EXISTS communication_ai_states (
    message_id TEXT PRIMARY KEY REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    ai_state TEXT NOT NULL DEFAULT 'NEW',
    review_reason TEXT,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_ai_states_state CHECK (
        ai_state IN ('NEW', 'PROCESSING', 'PROCESSED', 'REVIEW_REQUIRED', 'FAILED', 'ARCHIVED')
    ),
    CONSTRAINT communication_ai_states_review_reason_not_blank CHECK (
        review_reason IS NULL OR length(trim(review_reason)) > 0
    ),
    CONSTRAINT communication_ai_states_last_error_not_blank CHECK (
        last_error IS NULL OR length(trim(last_error)) > 0
    )
);

CREATE INDEX IF NOT EXISTS communication_ai_states_state_updated_idx
    ON communication_ai_states (ai_state, updated_at DESC, message_id);

INSERT INTO communication_ai_states (message_id, ai_state, review_reason, last_error, created_at, updated_at)
SELECT
    message_id,
    ai_state,
    review_reason,
    last_error,
    created_at,
    updated_at
FROM mail_ai_states
ON CONFLICT (message_id) DO NOTHING;
```
