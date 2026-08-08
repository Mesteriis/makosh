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

- Chunk ID / ID чанка: `121-adr-docs-part-002`
- Group / Группа: `docs`
- Role / Роль: `adr`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `decisions/adr-index.md`

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

### `docs/adr/ADR-0026-desktop-first-responsive-ui.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0026-desktop-first-responsive-ui.md`
- Size bytes / Размер в байтах: `556`
- Included characters / Включено символов: `556`
- Truncated / Обрезано: `no`

```markdown
# ADR-0026 Desktop First Responsive UI

Status: Proposed

## Context

The main product surface is a desktop app, but responsive behavior matters for window sizes and possible future web use.

## Decision

Design desktop-first, with responsive layouts that preserve usability across narrow and wide windows.

## Consequences

- Dense split-pane workflows can be first-class.
- Mobile-like simplification should not drive the core UI.
- Layout primitives must adapt without text overlap.
- Future web/mobile surfaces may need separate interaction decisions.
```

### `docs/adr/ADR-0027-capability-based-permission-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0027-capability-based-permission-model.md`
- Size bytes / Размер в байтах: `541`
- Included characters / Включено символов: `541`
- Truncated / Обрезано: `no`

```markdown
# ADR-0027 Capability Based Permission Model

Status: Proposed

## Context

Agents and plugins may read private data or perform side effects such as sending messages, exporting data or deleting records.

## Decision

Use a capability-based permission model for agents, plugins and external actions.

## Consequences

- Permissions can be scoped and audited.
- High-risk actions can require explicit confirmation.
- Capability manifests become part of plugin and tool contracts.
- Policy evaluation must be centralized enough to be reliable.
```

### `docs/adr/ADR-0028-backup-and-restore-as-core-feature.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0028-backup-and-restore-as-core-feature.md`
- Size bytes / Размер в байтах: `510`
- Included characters / Включено символов: `510`
- Truncated / Обрезано: `no`

```markdown
# ADR-0028 Backup and Restore as Core Feature

Status: Proposed

## Context

Local-first data ownership is only credible if the user can recover from machine loss, corruption or migration.

## Decision

Treat backup and restore as core product architecture, not operational afterthought.

## Consequences

- Storage layout must be backup-aware.
- Restore verification is required.
- Secret export requires explicit secure handling.
- Indexes may be rebuilt, but canonical data and artifacts must be preserved.
```

### `docs/adr/ADR-0029-explicit-schema-evolution.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0029-explicit-schema-evolution.md`
- Size bytes / Размер в байтах: `441`
- Included characters / Включено символов: `441`
- Truncated / Обрезано: `no`

```markdown
# ADR-0029 Explicit Schema Evolution

Status: Proposed

## Context

Events, relational tables, graph relationships and extracted artifacts will evolve over years.

## Decision

Use explicit schema versions, migrations and compatibility checks for durable data.

## Consequences

- Long-term upgrades become safer.
- Importers and projectors must handle older schemas.
- Migration testing is required.
- Breaking storage decisions need ADRs.
```

### `docs/adr/ADR-0030-documentation-first-monorepo.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0030-documentation-first-monorepo.md`
- Size bytes / Размер в байтах: `704`
- Included characters / Включено символов: `704`
- Truncated / Обрезано: `no`

```markdown
# ADR-0030 Documentation First Monorepo

Status: Proposed

## Context

The project is at foundation stage and explicitly should not start with implementation code. Future backend, frontend, infrastructure, tools and examples need shared architecture context.

## Decision

Use a documentation-first monorepo skeleton with dedicated directories for docs, backend, frontend, infrastructure, tools and examples.

## Consequences

- Implementation can start from shared architectural constraints.
- ADRs remain close to code as it appears.
- Empty implementation directories need ownership notes to avoid fake placeholders.
- Future package and build tooling should be added only when implementation begins.
```

### `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`
- Size bytes / Размер в байтах: `1234`
- Included characters / Включено символов: `1234`
- Truncated / Обрезано: `no`

```markdown
# ADR-0031 Temporary Desktop Only UI Scope

Status: Temporary

## Context

Макошь is currently defined as a desktop-first personal productivity system for PC and laptop use. Mobile UI introduces a separate product surface with different constraints: small-screen navigation, touch-first interactions, mobile OS permissions, background sync, notification behavior and mobile QA breakpoints.

ADR-0026 keeps responsive behavior for desktop window resizing and future web optionality, but it does not require mobile product design.

## Decision

Until this ADR is superseded, Макошь will not design, implement or validate a mobile UI.

Product, UX and frontend architecture work target PC and laptop layouts only. Responsive behavior means usable desktop resizing, not phone or tablet workflows.

## Consequences

- Mobile viewports may be incomplete or unusable; this is accepted temporary scope.
- No mobile navigation model, touch-first workflow, mobile breakpoint matrix or mobile packaging is required.
- UI documentation and future implementation must not claim mobile support.
- Future mobile support requires a new ADR defining goals, target devices, interaction model, data sync assumptions and validation requirements.
```

### `docs/adr/ADR-0032-docker-compose-development-environment.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0032-docker-compose-development-environment.md`
- Size bytes / Размер в байтах: `1002`
- Included characters / Включено символов: `1002`
- Truncated / Обрезано: `no`

```markdown
# ADR-0032 Docker Compose Development Environment

Status: Proposed

## Context

Макошь targets a local-first desktop product with PostgreSQL, Rust, SvelteKit and Tauri. Development needs a repeatable local environment without introducing production deployment semantics or scattering Docker files across the repository.

## Decision

Use Docker Compose for local development infrastructure. Keep Docker-specific files under `docker/`, including `docker/docker-compose.yml`, `docker/Dockerfile`, environment examples and bind-mounted development data under `docker/data/`.

Expose developer commands through the root `Makefile`.

## Consequences

- Local development can start from a consistent Compose entry point.
- Persistent dev data stays under `docker/data/` and is ignored by Git.
- Docker files do not leak into backend or frontend implementation directories.
- This setup is not a production deployment model.
- Any future production/self-hosted deployment design requires a separate ADR.
```

### `docs/adr/ADR-0033-backend-managed-local-schema-migrations.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0033-backend-managed-local-schema-migrations.md`
- Size bytes / Размер в байтах: `1233`
- Included characters / Включено символов: `1233`
- Truncated / Обрезано: `no`

```markdown
# ADR-0033 Backend Managed Local Schema Migrations

Status: Proposed

## Context

Макошь is a local-first desktop product with PostgreSQL as primary store. The user must be able to start the local app without manually applying schema changes, while schema evolution still needs explicit, reviewable migration files.

## Decision

The Rust backend owns local PostgreSQL migrations and applies embedded migrations at startup when `DATABASE_URL` is configured.

Migration files live in `backend/migrations/` and must be append-only once released. Durable schema changes require tests and, when architecturally significant, an ADR update.

`GET /readyz` must verify that the required backend-managed schema is present, not only that PostgreSQL accepts `SELECT 1`.

## Consequences

- Local development and desktop startup can keep schema and backend version aligned.
- Schema changes remain explicit in versioned SQL files.
- Readiness can catch schema drift or missing migrations before API handlers operate on missing tables.
- A bad migration can block startup, so migrations require smoke validation against development PostgreSQL.
- Future production or self-hosted deployment may require a separate migration execution policy.
```

### `docs/adr/ADR-0034-event-replay-and-projection-cursors.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0034-event-replay-and-projection-cursors.md`
- Size bytes / Размер в байтах: `940`
- Included characters / Включено символов: `940`
- Truncated / Обрезано: `no`

```markdown
# ADR-0034 Event Replay and Projection Cursors

Status: Proposed

## Context

ADR-0001 makes the event log the system spine. ADR-0023 requires projections and indexes to be rebuildable from canonical events. Rebuildable projections need a durable replay position and a safe way to resume after interruption.

## Decision

Use the `event_log.position` identity column as the durable replay order. Projection workers read events using `list_after_position(after_position, limit)` and persist progress in `projection_cursors`.

Projection cursor updates are monotonic: saving a lower position must not move a projection backward.

## Consequences

- Projection workers can resume after interruption.
- Replay order is independent from wall-clock timestamps.
- Rebuild workflows can reset or create projection-specific cursors intentionally.
- Future concurrent projection workers may need lease/lock semantics, which are not part of this ADR.
```

### `docs/adr/ADR-0035-local-event-api-command-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0035-local-event-api-command-boundary.md`
- Size bytes / Размер в байтах: `1086`
- Included characters / Включено символов: `1086`
- Truncated / Обрезано: `no`

```markdown
# ADR-0035 Local Event API Command Boundary

Status: Proposed

## Context

ADR-0015 requires durable mutations to pass through command boundaries. The backend already has an `EventStore`, but without an HTTP/API command boundary the UI and future local clients cannot append or query canonical events through the application layer.

## Decision

Expose a local backend API for canonical event operations:

- `POST /api/events` appends a validated canonical event envelope.
- `GET /api/events/{event_id}` loads a canonical event by ID.

The API uses the same event envelope validation and PostgreSQL idempotency constraints as the storage layer.

## Consequences

- UI and future local clients can use the same command/query boundary.
- Invalid envelopes fail before persistence.
- Duplicate event/source identity conflicts are explicit.
- The API is currently local-development/local-desktop scoped.
- Event API calls are guarded by the temporary local API token from ADR-0038.
- Remote exposure and the full capability runtime require future ADR-backed work before network deployment.
```

### `docs/adr/ADR-0036-projection-runner-checkpoint-semantics.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0036-projection-runner-checkpoint-semantics.md`
- Size bytes / Размер в байтах: `847`
- Included characters / Включено символов: `847`
- Truncated / Обрезано: `no`

```markdown
# ADR-0036 Projection Runner Checkpoint Semantics

Status: Proposed

## Context

Макошь projections must be rebuildable and resumable. `event_log.position` and `projection_cursors` define durable replay state, but workers also need consistent checkpoint semantics to avoid skipping failed events.

## Decision

Projection runners process events in ascending `event_log.position` order and save the projection cursor only after the event handler succeeds.

If a handler fails, the batch fails and the cursor remains at the last successfully processed position. The failed event remains eligible for retry.

## Consequences

- Projection workers are at-least-once by default.
- Projection handlers must be idempotent or tolerate retries.
- Failed events are not skipped accidentally.
- Future worker leasing/concurrency requires a separate ADR.
```

### `docs/adr/ADR-0037-local-write-capability-token.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0037-local-write-capability-token.md`
- Size bytes / Размер в байтах: `1704`
- Included characters / Включено символов: `1704`
- Truncated / Обрезано: `no`

```markdown
# ADR-0037 Local Write Capability Token

Status: Superseded by ADR-0038

## Context

ADR-0027 requires capability-based permissions. ADR-0035 introduced a local event API command boundary, but accepting unauthenticated writes to the canonical event log is unsafe even in development because future UI, agent and plugin code may accidentally depend on an open mutation path.

The project does not yet have the full user/session/capability runtime.

## Decision

Use a temporary local write capability token for mutating backend HTTP endpoints.

This decision was superseded by ADR-0038 because read endpoints also expose private event payloads and need the same local API capability guard.

Rules:

- `MAKOSH_LOCAL_WRITE_TOKEN` configures the local write token.
- Empty `MAKOSH_LOCAL_WRITE_TOKEN` is invalid configuration.
- `POST /api/events` requires `Authorization: Bearer <token>`.
- If `MAKOSH_LOCAL_WRITE_TOKEN` is not configured, write commands fail closed with `503 write_token_not_configured`.
- If the bearer token is missing or invalid, write commands return `401 invalid_write_token`.
- `GET /healthz` and `GET /readyz` remain unauthenticated operational probes.

This token is a local-development and local-desktop command guard, not a substitute for the long-term capability runtime.

## Consequences

- Accidental unauthenticated writes to the event log are blocked.
- Local smoke and development commands must provide `MAKOSH_LOCAL_WRITE_TOKEN`.
- `docker/.env.example` may contain only a non-secret placeholder token.
- Before network exposure, multi-user access, plugins or agents can perform writes, this temporary token must be replaced or wrapped by the full capability policy model.
```

### `docs/adr/ADR-0038-local-event-api-capability-token.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0038-local-event-api-capability-token.md`
- Size bytes / Размер в байтах: `1768`
- Included characters / Включено символов: `1768`
- Truncated / Обрезано: `no`

```markdown
# ADR-0038 Local Event API Capability Token

Status: Temporary

## Context

ADR-0027 requires capability-based permissions. ADR-0035 introduced local event API command/query endpoints. ADR-0037 protected writes, but `GET /api/events/{event_id}` can expose private event payloads and provenance, so protecting only mutating endpoints leaves a local data disclosure path.

The project does not yet have the full user/session/capability runtime.

## Decision

Use a temporary local API capability token for local event API endpoints.

Rules:

- `MAKOSH_LOCAL_API_TOKEN` configures the local API token.
- Empty `MAKOSH_LOCAL_API_TOKEN` is invalid configuration.
- `MAKOSH_LOCAL_WRITE_TOKEN` remains a legacy fallback during the transition from ADR-0037.
- `POST /api/events` requires `Authorization: Bearer <token>`.
- `GET /api/events/{event_id}` requires `Authorization: Bearer <token>`.
- If no local API token is configured, API calls fail closed with `503 api_token_not_configured`.
- If the bearer token is missing or invalid, API calls return `401 invalid_api_token`.
- `GET /healthz` and `GET /readyz` remain unauthenticated operational probes.

This token is a local-development and local-desktop API guard, not a substitute for the long-term capability runtime.

## Consequences

- Event reads and writes both require local API authorization.
- Local smoke and development commands must provide `MAKOSH_LOCAL_API_TOKEN`.
- Existing `MAKOSH_LOCAL_WRITE_TOKEN` development setups continue to work as a fallback.
- `docker/.env.example` may contain only a non-secret placeholder token.
- Before network exposure, multi-user access, plugins or agents can perform reads or writes, this temporary token must be replaced or wrapped by the full capability policy model.
```

### `docs/adr/ADR-0039-local-event-api-access-audit-log.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0039-local-event-api-access-audit-log.md`
- Size bytes / Размер в байтах: `1988`
- Included characters / Включено символов: `1988`
- Truncated / Обрезано: `no`

```markdown
# ADR-0039 Local Event API Access Audit Log

Status: Proposed

## Context

ADR-0038 protects local event API reads and writes with a temporary API capability token. Protection without durable auditability still leaves an operational gap: authorized event reads and writes cannot be reviewed later.

The canonical `event_log` is the domain event spine. Recording API reads as canonical events would make query operations mutate replay state and projection cursors, which would mix operational access audit with domain history.

## Decision

Create a separate append-only `api_audit_log` table for local event API access attempts.

Rules:

- Authorized `POST /api/events` records an `event.append` audit attempt before appending the canonical event.
- Authorized `GET /api/events/{event_id}` records an `event.get` audit attempt before loading the event.
- Audit records store operation, method, path template, target kind, target ID, actor kind and metadata.
- API tokens and secrets are never stored in audit records.
- Audit logging is fail-closed: if the audit insert fails, the API operation is not performed.
- `GET /api/audit/events` exposes protected read-only audit inspection with optional `target_id`, `after_audit_id` and `limit` query parameters.
- Audit inspection uses monotonic `audit_id` cursor pagination; `after_audit_id` returns records with greater audit IDs.
- Reading audit records is not itself recorded in `api_audit_log` to avoid unbounded self-audit noise.
- `api_audit_log` is append-only and must reject updates and deletes.

## Consequences

- Local event API reads and writes are reviewable.
- Audit review requires the same temporary local API capability token as event API access.
- Query operations no longer need to pollute canonical event replay streams to become auditable.
- Audit records are operational security data, not domain events.
- Future capability runtime can replace `actor_kind = local_api_token` with richer actor/capability identifiers.
```

### `docs/adr/ADR-0040-local-api-actor-identity.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0040-local-api-actor-identity.md`
- Size bytes / Размер в байтах: `1784`
- Included characters / Включено символов: `1784`
- Truncated / Обрезано: `no`

```markdown
# ADR-0040 Local API Actor Identity

Status: Superseded by ADR-0056

## Context

ADR-0038 protects local event API access with a temporary API capability token, and ADR-0039 records authorized event API access in `api_audit_log`.

The token identifies a shared local capability, not the client or tool using that capability. Without a caller identity, audit records can prove that the local API token was used but cannot distinguish the desktop UI, CLI smoke tests, future local agents or other local development tools.

## Decision

Require a temporary local API actor identity header for protected local event API endpoints.

Rules:

- Protected local event API requests must include `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>`.
- After the bearer token is valid, protected local event API requests must include `X-Макошь-Actor-Id`.
- `X-Макошь-Actor-Id` is a stable local client identifier, not a secret.
- The accepted actor ID character set is ASCII letters, digits, `.`, `_`, `-`, `:`, `@` and `/`.
- Actor IDs must be non-empty after trimming and at most 128 bytes.
- Missing or invalid actor IDs return `400 invalid_actor_id`.
- `api_audit_log` stores `actor_kind = local_api_token` and the supplied `actor_id`.
- Audit inspection supports filtering by `actor_id`.
- API tokens and secrets must never be stored in audit records.

## Consequences

- Local event API audit records can distinguish authorized local clients.
- Existing clients must send `X-Макошь-Actor-Id` in addition to the bearer token.
- This remains self-asserted identity while the temporary token model exists.
- The full capability runtime must replace this with authenticated capability and actor identifiers before multi-user access, plugins or agents are allowed to perform broad reads or writes.
```

### `docs/adr/ADR-0041-email-provider-ingestion-foundation.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0041-email-provider-ingestion-foundation.md`
- Size bytes / Размер в байтах: `2370`
- Included characters / Включено символов: `2370`
- Truncated / Обрезано: `no`

```markdown
# ADR-0041 Email Provider Ingestion Foundation

Status: Proposed

## Context

Version 1.0 requires the first communication source and an event-backed ingestion pipeline. Email must support more than one provider shape from the start:

- Gmail through the Gmail API and OAuth.
- iCloud Mail through IMAP with app-specific credentials.
- Generic IMAP for self-hosted or provider-neutral mailboxes.

Implementing a concrete adapter before defining raw source preservation, idempotency and checkpoints would push provider quirks into application code and make retries unsafe.

## Decision

Create a provider-neutral email ingestion storage boundary before implementing concrete provider adapters.

Rules:

- Supported initial email provider kinds are `gmail`, `icloud` and `imap`.
- Provider account records store non-secret account metadata and non-secret adapter configuration only.
- OAuth tokens, app passwords and mailbox passwords must stay behind the secret boundary from ADR-0016 and must not be stored in provider account config.
- Provider account credentials are represented through secret references from ADR-0042.
- Raw provider records are stored append-only in `communication_raw_records`.
- Raw provider record identity is idempotent by `(account_id, record_kind, provider_record_id)`.
- Raw records keep `source_fingerprint`, `import_batch_id`, provider payload and provenance for replay/debugging.
- Ingestion checkpoints are stored per `(account_id, stream_id)` with provider-specific JSON payloads.
- Gmail adapters should checkpoint Gmail history streams, for example `stream_id = gmail:history`.
- iCloud and generic IMAP adapters should checkpoint mailbox streams, for example `stream_id = imap:INBOX`, with UID validity and last seen UID data.
- This decision establishes the storage boundary before provider networking. ADR-0043 adds read-only Gmail API and IMAP networking against this boundary.

## Consequences

- Gmail, iCloud and generic IMAP can share one ingestion persistence contract.
- Provider adapters can be retried without duplicating raw source records.
- Raw records remain available for replay, parser fixes and future projection rebuilds.
- Provider account metadata is separated from secrets.
- The next implementation slice can build a read-only adapter against this boundary instead of inventing persistence inside the adapter.
```

### `docs/adr/ADR-0042-secret-references-for-provider-credentials.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0042-secret-references-for-provider-credentials.md`
- Size bytes / Размер в байтах: `3228`
- Included characters / Включено символов: `3228`
- Truncated / Обрезано: `no`

```markdown
# ADR-0042 Secret References for Provider Credentials

Status: Superseded by ADR-0053

This decision was superseded by ADR-0053. The account-scoped secret reference and resolver boundary remains, but encrypted provider credential payloads now live in a dedicated PostgreSQL vault table instead of outside PostgreSQL.

## Context

ADR-0016 requires provider credentials to stay outside ordinary application tables. ADR-0041 adds provider account metadata for Gmail, iCloud Mail and generic IMAP, but real adapters will need credentials:

- Gmail needs OAuth credential material.
- iCloud Mail needs an app-specific password for IMAP.
- Generic IMAP usually needs a mailbox password and may later need SMTP credentials.

Storing credential values in provider account config would make database backups and debugging workflows unsafe.

## Decision

Store only secret references in PostgreSQL, never secret values.

Rules:

- `secret_references` stores non-secret metadata: `secret_ref`, `secret_kind`, `store_kind`, label and JSON metadata.
- Supported initial secret kinds are `oauth_token`, `app_password`, `password`, `api_token`, `private_key` and `other`.
- Supported initial secret store kinds are `os_keychain`, `encrypted_vault`, `external_vault` and `test_double`.
- Communication provider accounts bind to secrets through `communication_provider_account_secret_refs`.
- Supported initial communication secret purposes are `oauth_token`, `imap_password` and `smtp_password`.
- Gmail provider accounts should bind `oauth_token`.
- iCloud and generic IMAP provider accounts should bind `imap_password`.
- `oauth_token` bindings require `secret_kind = oauth_token`; `imap_password` and `smtp_password` bindings require `secret_kind = app_password` or `password`.
- Multiple accounts for the same provider kind are supported. Credential lookup must use the provider `account_id` and secret purpose, not provider kind alone.
- Provider adapters should use the account-scoped `ProviderCredentialReader` path instead of reimplementing credential joins.
- Secret values must be written to and read from the configured secret store through a `SecretResolver` boundary.
- The in-memory resolver is valid only for `test_double` references in tests and local adapter tests. It must not resolve `os_keychain`, `encrypted_vault` or `external_vault` references.
- Provider account config and secret reference metadata must not contain OAuth tokens, app passwords, mailbox passwords, private keys or API tokens.

## Consequences

- PostgreSQL can express which credentials an adapter needs without storing credential values.
- Provider adapters can resolve credentials explicitly at runtime.
- Missing credential bindings, incompatible secret kinds and resolver failures are reported explicitly before provider network calls begin.
- Provider adapters can support multiple Gmail, iCloud or IMAP accounts without shared global credentials.
- Database backups still need secret reference metadata but do not automatically leak provider credentials.
- ADR-0044 adds encrypted vault storage and account setup for Gmail OAuth and IMAP credentials. A native OS keychain resolver can still be added later as another secret store implementation.
```

### `docs/adr/ADR-0043-read-only-email-provider-networking.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0043-read-only-email-provider-networking.md`
- Size bytes / Размер в байтах: `535`
- Included characters / Включено символов: `533`
- Truncated / Обрезано: `no`

```markdown
# ADR-0043 Read-Only Email Provider Networking

Status: Superseded by ADR-0055

Superseded because: the read-only constraint was a temporary safety measure for the initial implementation phase. Макошь is a personal local-first system and needs full email functionality including sending, replying, flag mutations, and server-side state changes. The read-only restriction now applies ONLY to automated integration tests — production code must support both read and write provider operations.

See ADR-0055 for the current policy.
```

### `docs/adr/ADR-0044-account-setup-and-encrypted-secret-vault.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0044-account-setup-and-encrypted-secret-vault.md`
- Size bytes / Размер в байтах: `2281`
- Included characters / Включено символов: `2281`
- Truncated / Обрезано: `no`

```markdown
# ADR-0044 Account Setup and Encrypted Secret Vault

Status: Superseded by ADR-0076

This decision was superseded by ADR-0053 and then ADR-0076. ADR-0076 keeps encrypted local account setup but moves new secret payloads to a dedicated macOS host vault under `~/.makosh/vault`.

## Context

ADR-0042 keeps provider credential values outside PostgreSQL, and ADR-0043 adds read-only Gmail API and IMAP networking. The remaining gap was account setup: users needed a way to obtain Gmail OAuth tokens, refresh them, and store iCloud/raw IMAP passwords without writing secrets into provider account config or secret reference metadata.

## Decision

Add a local account setup boundary backed by an encrypted secret vault.

Rules:

- `MAKOSH_SECRET_VAULT_PATH` points to the local encrypted vault file.
- `MAKOSH_SECRET_VAULT_KEY` is the local vault master key and must not be logged, persisted in PostgreSQL or committed.
- The encrypted vault uses per-entry AES-256-GCM encryption with an Argon2id-derived key and authenticated `secret_ref` associated data.
- Gmail account setup uses OAuth authorization code with PKCE and `gmail.readonly` scope.
- Gmail token bundles are stored only in the encrypted vault and include access token, refresh token, token endpoint and OAuth client material required for refresh.
- Gmail access token refresh reads the encrypted token bundle, exchanges the refresh token and updates the encrypted vault.
- iCloud and raw IMAP setup store app-password/password values only in the encrypted vault.
- PostgreSQL stores only provider account metadata, secret reference metadata and account-to-secret bindings.
- The desktop account wizard calls local API endpoints protected by the local API token and actor header.

## Consequences

- Local development and desktop account setup can create usable Gmail, iCloud and raw IMAP provider accounts without plaintext secrets in PostgreSQL.
- Provider networking can obtain runtime access tokens through refresh instead of requiring manual token injection.
- The encrypted vault becomes part of local operational setup and must be backed up with its master key handled separately.
- A native OS keychain resolver can still be added later as another `SecretStoreKind`, but account setup is no longer blocked on it.
```

### `docs/adr/ADR-0045-graph-core-projection.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0045-graph-core-projection.md`
- Size bytes / Размер в байтах: `2043`
- Included characters / Включено символов: `2043`
- Truncated / Обрезано: `no`

```markdown
# ADR-0045 Graph Core Projection

Status: Proposed

## Context

Version 2 starts by turning the Knowledge Graph into a real backend projection. Макошь already has local PostgreSQL storage for contacts, communication messages and documents. ADR-0008 requires relationships to be durable records with provenance and confidence. ADR-0023 requires derived state to be rebuildable. ADR-0019 forbids ambiguous automatic identity merges. ADR-0031 keeps the UI desktop/laptop only.

## Decision

Use PostgreSQL relational graph tables for the first V2 graph core:

- `graph_nodes`
- `graph_edges`
- `graph_evidence`

The graph tables are a rebuildable projection, not source of truth. Source records remain in `contacts`, `communication_messages` and `documents`.

Initial node kinds:

- `person`
- `email_address`
- `message`
- `document`

Initial relationship types:

- `person_has_email_address`
- `person_sent_message`
- `person_received_message`
- `email_address_sent_message`
- `email_address_received_message`

System-created edges require evidence. The first projection only uses exact email matching to connect messages to contacts. When no exact contact exists, the graph uses an `email_address` node instead of inventing a person.

Read APIs are local-only, read-only and protected by the existing bearer token plus `X-Макошь-Actor-Id`.

## Non-Goals

- Separate graph database.
- GraphQL.
- Fuzzy person merge.
- Graph editing.
- OCR and entity extraction.
- Task candidate extraction.
- Mobile graph UI.

## Consequences

Positive:

- Graph data stays inspectable and rebuildable in PostgreSQL.
- Provenance is queryable without unpacking arbitrary edge JSON.
- The first V2 slice avoids false person merges.
- Existing Docker, SQLx and live PostgreSQL smoke tests remain enough for validation.

Negative:

- Graph traversal depth is intentionally limited in the first slice.
- Richer identity resolution requires a later reviewed merge/split workflow.
- Document-person and document-project edges wait for a later extraction engine.
```

### `docs/adr/ADR-0046-persistent-dev-mail-cache-and-blob-storage.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0046-persistent-dev-mail-cache-and-blob-storage.md`
- Size bytes / Размер в байтах: `4075`
- Included characters / Включено символов: `4075`
- Truncated / Обрезано: `no`

```markdown
# ADR-0046 Persistent Dev Mail Cache and Blob Storage

Status: Proposed

## Context

ADR-0041 defines provider-neutral email ingestion, ADR-0043 requires read-only provider networking, ADR-0044 keeps credentials behind the secret boundary, and ADR-0032 keeps persistent development data under `docker/data/`.

The desktop UI now needs realistic local mail data for development: after starting the dev stack, previously downloaded messages should be visible without reconnecting to the provider or re-entering credentials. At the same time, email messages can contain large raw MIME payloads and attachments that do not belong in PostgreSQL as ordinary row data.

## Decision

Use a persistent local mail cache split by responsibility:

- PostgreSQL stores provider accounts, mailbox checkpoints, message metadata, searchable extracted content, attachment metadata, projections, graph references and search index references.
- Local blob storage stores heavy or opaque mail bytes: raw `.eml` payloads, attachment bytes, previews and future extracted attachment artifacts.
- Development blob storage lives under `docker/data/mail/` and is ignored by Git.
- Database rows reference local blobs by stable metadata: storage kind, relative storage path, SHA-256 digest, byte size, content type and optional filename.
- Attachments are represented as first-class metadata records linked to canonical messages and source raw records.
- Extracted attachment metadata must pass through an attachment safety scanning boundary before it is stored or exposed to application workflows.
- The initial scanner is an explicit no-op scanner and records `not_scanned`; it must not mark attachments as `clean` until a real scanner backend is implemented.
- Attachment scan metadata is stored with the attachment record: scan status, scanner engine, scan timestamp, human-readable summary and structured metadata.
- The initial MIME extractor is intentionally basic: it supports recursive multipart traversal, `text/plain` body projection, attachment-like parts with `attachment` disposition, inline parts with filenames, `filename`, single-section `filename*`, ordered RFC2231 continuation filename segments, and `base64` or `quoted-printable` transfer decoding.
- The initial MIME extractor is not a complete RFC MIME engine. Encrypted/signed containers, malformed boundary recovery, charset transcoding beyond lossy UTF-8 handling, preview generation and deep attachment inspection remain future slices.
- The system must not store mailbox credentials, OAuth tokens or app passwords in blob paths, blob metadata, database payloads, logs or fixture files.
- Read-only constraint applies to automated tests only per ADR-0055. Production provider sync uses read-write networking.
- `make dev` should be allowed to reuse already downloaded local cache data and should not require provider connectivity for the UI to display previously downloaded messages.
- `make reset-data` remains the explicit destructive command for local development cache removal.

Initial implementation may keep fixture import redacted and attachment-free. Full provider sync should evolve toward storing raw MIME and attachments through the blob store while projecting only normalized metadata, scan state and extracted text into PostgreSQL.

## Consequences

- The UI can be built against persistent realistic local data instead of synthetic mocks.
- PostgreSQL remains optimized for queries, search and relationships instead of becoming a large binary object store.
- Local development data is durable across `make dev` restarts but remains outside Git.
- Backup and restore must eventually include both PostgreSQL state and `docker/data/mail/` blob state.
- Until a real scanner is implemented, extracted attachment rows are intentionally not trusted and must remain distinguishable from scanned-clean attachments.
- Blob garbage collection, attachment extraction quality, previews and remote/self-hosted object storage require later ADR-backed implementation details if they change this local-first storage contract.
```

### `docs/adr/ADR-0047-project-memory-spine.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0047-project-memory-spine.md`
- Size bytes / Размер в байтах: `2740`
- Included characters / Включено символов: `2740`
- Truncated / Обрезано: `no`

```markdown
# ADR-0047 Project Memory Spine

Status: Proposed

## Context

Version 2 needs graph-backed memory that connects messages, people, projects and documents. The current V2 graph core projects contacts, email addresses, messages and documents, but projects still exist only as frontend presentation data. This blocks project timelines and prevents the graph from using projects as durable memory anchors.

ADR-0045 intentionally limited the first graph core to four node kinds and five relationship types. Project nodes and project relationships therefore require an explicit ADR and schema evolution before implementation.

## Decision

Add a local `projects` read model as the first project memory spine.

Projects are canonical local records with deterministic `project_id` values, human-readable metadata and explicit `project_keywords`. Keywords are user/system configured matching rules, not AI inference. The first implementation may seed a local `Макошь` project record so a development database has a real project anchor, but all project relationships must still be derived from stored messages and documents.

Extend the PostgreSQL graph projection with:

- node kind `project`;
- relationship type `project_has_message`;
- relationship type `project_has_document`;
- relationship type `project_involves_person`;
- relationship type `project_involves_email_address`.

Project graph edges are rebuildable projection state. They must carry evidence from the matched message or document and preserve confidence/review state. The first project matching rule is deterministic keyword containment over message subject/body and document title/extracted text. These links are `suggested` unless a later review workflow confirms them.

Expose read-only protected local APIs:

- `GET /api/v2/projects`;
- `GET /api/v2/projects/{project_id}`.

The project detail API returns project metadata, derived stats, recent communications, related documents, key people and timeline items. It must not expose message body text.

## Non-Goals

- Project write UI.
- AI project inference.
- Fuzzy project merge or rename workflow.
- OCR or rich entity extraction.
- Task candidate extraction.
- Outbound provider writes.
- Mobile project UI.

## Consequences

Positive:

- Projects become first-class graph nodes.
- Project pages can use real local backend data instead of frontend mocks.
- Project timelines can be built from source-backed messages and documents.
- Later OCR/entity extraction can improve links without changing the frontend contract.

Negative:

- Keyword matching can create false project suggestions.
- The first project model is intentionally narrow.
- Project management UI and review/confirmation workflows remain future work.
```

### `docs/adr/ADR-0048-project-link-review-workflow.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0048-project-link-review-workflow.md`
- Size bytes / Размер в байтах: `2188`
- Included characters / Включено символов: `2188`
- Truncated / Обрезано: `no`

```markdown
# ADR-0048 Project Link Review Workflow

Status: Proposed

## Context

ADR-0047 introduced project nodes and keyword-derived project relationships. Those relationships are suggested because deterministic keyword containment can create false positives and false negatives.

ADR-0001 requires meaningful changes to be represented as canonical events. ADR-0023 and ADR-0045 make graph tables rebuildable projections, so user review decisions cannot live only on graph edges.

## Decision

Add event-backed project link review for direct project-to-message and project-to-document links.

User review commands append `project.link_review_state_changed` events. A durable `project_link_reviews` read model stores only explicit decisions:

- `user_confirmed`
- `user_rejected`

Resetting a link to `suggested` appends an event and removes the explicit decision row. Unreviewed suggested links remain derived from project keyword rules.

Project graph edges remain rebuildable projection state. During graph projection:

- keyword-only active links use `review_state = suggested`;
- confirmed links use `review_state = user_confirmed`;
- rejected links are omitted;
- confirmed links remain active even when current keyword rules do not match.

People and email-address project links remain derived from active project-message links. Direct people review is out of scope for this slice.

Protected local review APIs must require the temporary local bearer token and `X-Макошь-Actor-Id`.

## Non-Goals

- Project create/edit UI.
- Keyword management UI.
- Manual people/contact merge.
- Direct review of project-person edges.
- AI project inference.
- OCR or entity extraction.
- Mobile UI.

## Consequences

Positive:

- False project links can be rejected without editing source messages or documents.
- Important links can be confirmed even if keyword rules later change.
- Review state survives graph rebuild.
- Project detail and graph projection can share the same active-link rules.

Negative:

- Review commands require event/table transaction discipline.
- The first workflow only handles direct message and document links.
- A later keyword editor still needs separate ADR-backed work.
```

### `docs/adr/ADR-0049-v3-local-ai-runtime-and-retrieval.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0049-v3-local-ai-runtime-and-retrieval.md`
- Size bytes / Размер в байтах: `2605`
- Included characters / Включено символов: `2605`
- Truncated / Обрезано: `no`

```markdown
# ADR-0049 V3 Local AI Runtime and Retrieval

Status: Proposed

## Context

This ADR is amended by ADR-0081 for opt-in OmniRoute runtime support.

ADR-0009 selects Ollama as the first local AI runtime. ADR-0007 keeps vector search replaceable. ADR-0022 forbids fine-tuning private data. V3 needs source-backed AI workflows over the V1/V2 memory spine without turning model output into source of truth.

Qwen3 embedding output is 2560 dimensions. pgvector `vector` is not suitable for this dimension in the current stack, while `halfvec(2560)` supports the required shape and approximate cosine search.

## Decision

Implement V3 AI as a thin local runtime over existing canonical projections:

- Ollama is the only V3 model provider.
- Default chat model is `qwen3:4b`.
- Default embedding model is `qwen3-embedding:4b`.
- Embeddings use pgvector `halfvec(2560)` with HNSW cosine indexing.
- Semantic embeddings are derived state for messages, documents, projects, tasks and contacts.
- Source-backed answer generation must retrieve local citations before prompting.
- Retrieved text is treated as untrusted context in prompts.
- AI answers, task extraction runs and meeting prep runs persist `ai_agent_runs` records with status, model config, prompt template version, answer, citations, timings, actor ID and correlation IDs.
- AI run requested/completed/failed and AI task extraction lifecycle events are represented as canonical events.
- V3 task extraction may create only `suggested` task candidates linked to `agent_run_id`; existing review APIs remain the only path to active tasks.
- V3 meeting prep returns a local briefing packet and does not require calendar ingestion.
- V3 protected APIs require `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>` and `X-Макошь-Actor-Id`.

## Non-Goals

- Cloud model providers.
- Fine-tuning or training on private data.
- Autonomous activation.
- External email, calendar, message or task writes.
- Provider adapter implementation.
- Mobile UI.

## Consequences

Positive:

- AI behavior is auditable through persisted runs and canonical events.
- Citations keep local source provenance visible to UI and tests.
- Embeddings remain rebuildable derived state.
- Model/provider replacement remains possible behind the Ollama boundary and semantic store.

Negative:

- Local model latency becomes part of V3 workflow UX.
- `make validate` depends on a reachable live Ollama runtime for AI smoke validation.
- pgvector is now required in the local development PostgreSQL image.
- Prompt quality and retrieval ranking need ongoing evaluation with real local data.
```

### `docs/adr/ADR-0050-v4-telegram-client-policy-and-call-intelligence.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0050-v4-telegram-client-policy-and-call-intelligence.md`
- Size bytes / Размер в байтах: `4140`
- Included characters / Включено символов: `4140`
- Truncated / Обрезано: `no`

```markdown
# ADR-0050 V4 Telegram Client, Policy Automation and Call Intelligence

Status: Proposed

## Context

Version 4 expands Макошь from read-only memory and source-backed AI into a Telegram-capable local client with controlled outbound automation and call-derived task context. Telegram user accounts and bot accounts have different API surfaces, credentials, and limits. Telegram user accounts require a full client runtime, while bot accounts are constrained by Bot API visibility.

AI and automation are now allowed to send Telegram messages without per-message confirmation, but only when an explicit user-configured policy and approved template authorize the send. Call transcription is required for enabled Telegram accounts/chats and must remain local.

## Decision

Implement V4 around explicit boundaries:

- Telegram supports multiple `telegram_user` and `telegram_bot` provider accounts.
- Telegram user accounts use a TDLib-first runtime boundary. TDLib local state must be account-scoped and stored under ignored local data paths, encrypted where supported.
- Telegram bot accounts use a Bot API-compatible runtime boundary.
- PostgreSQL stores account metadata, raw source records, checkpoints, canonical projections, policy state, call metadata, transcript metadata and audit records. It does not store Telegram API hashes, bot tokens, session encryption keys or other secret values.
- Telegram credentials are resolved by `account_id + secret_purpose`; provider kind alone must never select credentials.
- V4 accepts a fixture Telegram runtime for tests and local smoke validation. Live Telegram validation is opt-in.
- AI and automation may send Telegram messages only through enabled policies configured in the UI. Policies bind templates, accounts, chats, triggers, limits, quiet hours and expiry.
- AI may fill only declared template variables. It cannot choose destinations, templates, policy authority or send scope from retrieved content.
- Every automated send writes canonical event/audit metadata with policy ID, template ID, account ID, chat ID, preview hash and actor context.
- V4 call scope is 1:1 audio call MVP. Video calls, group calls and screen sharing are V4.x or later.
- Call transcription is local, policy/account/chat scoped, visible in UI, and stored with source provenance.
- Telegram data may be used for local workflows, retrieval and task extraction, but not for fine-tuning or training models.
- V4 exposes a protected capability contract that reports available fixture capabilities, blocked live-runtime capabilities and unsupported V4.x features to both UI and tests.

## Consequences

Positive:

- Telegram becomes a first-class local communication channel without making provider data the source of truth.
- Automated sends are possible while preserving user-configured authority and auditability.
- Call transcripts can feed existing task candidate and project memory workflows.
- Fixture runtime keeps CI independent from live Telegram credentials and audio devices.

Negative:

- TDLib/native media integration is a larger operational dependency than prior HTTP-only providers.
- Call capture and transcription introduce privacy, storage and platform permission complexity.
- The policy evaluator becomes security-critical and must be covered by regression tests before live sends.

Risk handling:

- Live TDLib sessions, live Bot API sends, desktop audio capture and `whisper-rs` transcription are not silent runtime gaps. They must report `blocked` through the V4 capability contract until their adapters, permissions, secret resolution and smoke validation exist.
- Fixture Telegram runtime, automated-send dry-run, call metadata storage and fixture speech-to-text are V4 closure capabilities and must report `available`.
- Video calls, group calls, screen sharing, hidden recording, Telegram-data fine-tuning and third-party plugin execution are unsupported V4 features and must remain outside V4 closure gates.

## Non-Goals

- Video calls.
- Group calls.
- Screen sharing.
- Training or fine-tuning models on Telegram data.
- Hidden recording.
- Third-party plugin code execution.
```
