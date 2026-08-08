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

- Chunk ID / ID чанка: `120-adr-docs-part-001`
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

### `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0001-event-sourcing-as-system-spine.md`
- Size bytes / Размер в байтах: `626`
- Included characters / Включено символов: `626`
- Truncated / Обрезано: `no`

```markdown
# ADR-0001 Event Sourcing as System Spine

Status: Proposed

## Context

Макошь must preserve years of communication, document, task and relationship history. Current state alone cannot explain why a conclusion exists or when a commitment emerged.

## Decision

Represent meaningful changes as canonical events and use those events to build projections, graph links, indexes and timelines.

## Consequences

- Historical reconstruction becomes possible.
- Projection bugs can be fixed by replay.
- Schema evolution requires versioned event payloads.
- Implementation must handle idempotency and replay from the beginning.
```

### `docs/adr/ADR-0002-rust-backend.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0002-rust-backend.md`
- Size bytes / Размер в байтах: `607`
- Included characters / Включено символов: `607`
- Truncated / Обрезано: `no`

```markdown
# ADR-0002 Rust Backend

Status: Proposed

## Context

The backend will coordinate ingestion, indexing, local storage, provider adapters, agent tools and desktop integration. It needs strong correctness properties and predictable performance.

## Decision

Use Rust for the backend.

## Consequences

- Strong typing and explicit error handling support long-term maintainability.
- Rust integrates naturally with Tauri and Tantivy.
- Development speed may be lower than Python for exploratory features.
- Integration with AI and document tooling may require careful library selection or sidecar boundaries.
```

### `docs/adr/ADR-0003-sveltekit-frontend.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0003-sveltekit-frontend.md`
- Size bytes / Размер в байтах: `558`
- Included characters / Включено символов: `558`
- Truncated / Обрезано: `no`

```markdown
# ADR-0003 SvelteKit Frontend

Status: Proposed

## Context

The UI must support dense desktop workflows, reactive state, command palette interactions and future web portability.

## Decision

Use SvelteKit for the frontend.

## Consequences

- The UI can remain highly interactive with relatively low framework overhead.
- SvelteKit keeps routing and frontend composition structured.
- SSR features are secondary in the desktop shell and must not complicate local operation.
- Frontend state must remain subordinate to backend commands for durable changes.
```

### `docs/adr/ADR-0004-tauri-desktop-shell.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0004-tauri-desktop-shell.md`
- Size bytes / Размер в байтах: `515`
- Included characters / Включено символов: `515`
- Truncated / Обрезано: `no`

```markdown
# ADR-0004 Tauri Desktop Shell

Status: Proposed

## Context

Макошь is local-first and needs desktop integration for files, local services, secret storage, notifications and possible provider bridge workflows.

## Decision

Use Tauri as the desktop shell.

## Consequences

- Rust backend and desktop bridge can share technology.
- The app can remain lighter than Electron-based alternatives.
- Tauri command boundaries must be narrow and validated.
- OS-specific behavior must be isolated behind clear ports.
```

### `docs/adr/ADR-0005-postgresql-primary-store.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0005-postgresql-primary-store.md`
- Size bytes / Размер в байтах: `624`
- Included characters / Включено символов: `624`
- Truncated / Обрезано: `no`

```markdown
# ADR-0005 PostgreSQL Primary Store

Status: Proposed

## Context

Макошь needs durable relational state, event storage, graph-like relationships, JSON payloads, migrations and local deployment.

## Decision

Use PostgreSQL as the primary local store.

## Consequences

- Events, entities, relationships, metadata and projection offsets can live in one transactional system.
- JSONB can support versioned payloads while typed tables support queryable state.
- Local PostgreSQL installation and lifecycle must be handled cleanly.
- PostgreSQL is not the only storage component; search and object storage remain separate.
```

### `docs/adr/ADR-0006-tantivy-full-text-search.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0006-tantivy-full-text-search.md`
- Size bytes / Размер в байтах: `492`
- Included characters / Включено символов: `492`
- Truncated / Обрезано: `no`

```markdown
# ADR-0006 Tantivy Full Text Search

Status: Proposed

## Context

Макошь requires fast local full text search over messages, documents, tasks, contacts and projects. The backend target language is Rust.

## Decision

Use Tantivy for full text search.

## Consequences

- Search can run locally without cloud dependencies.
- Rust integration is strong.
- Indexes must be treated as derived and rebuildable.
- Query planning must combine Tantivy results with graph and semantic retrieval.
```

### `docs/adr/ADR-0007-replaceable-vector-search.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0007-replaceable-vector-search.md`
- Size bytes / Размер в байтах: `585`
- Included characters / Включено символов: `585`
- Truncated / Обрезано: `no`

```markdown
# ADR-0007 Replaceable Vector Search

Status: Proposed

## Context

Semantic recall is required, but vector database choices and embedding models will evolve. The system must not bind durable memory to a single vector store.

## Decision

Define vector search behind a replaceable interface and treat embeddings/indexes as derived artifacts.

## Consequences

- The product can switch vector backends later.
- Embeddings can be regenerated after model changes.
- Search quality depends on evaluation and metadata discipline.
- Canonical state must not live only inside vector indexes.
```

### `docs/adr/ADR-0008-knowledge-graph-first.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0008-knowledge-graph-first.md`
- Size bytes / Размер в байтах: `619`
- Included characters / Включено символов: `619`
- Truncated / Обрезано: `no`

```markdown
# ADR-0008 Knowledge Graph First

Status: Proposed

## Context

The product's core value is long-term relationships between people, organizations, projects, documents, messages, tasks and decisions.

## Decision

Make the knowledge graph a first-class architectural component, with relationships represented as durable records carrying provenance and confidence.

## Consequences

- Memory queries can use relationship context, not only text similarity.
- Inferred links can be reviewed and corrected.
- Graph schema design becomes central early work.
- UI must expose graph value without overwhelming daily workflows.
```

### `docs/adr/ADR-0009-local-ai-through-ollama.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0009-local-ai-through-ollama.md`
- Size bytes / Размер в байтах: `472`
- Included characters / Включено символов: `472`
- Truncated / Обрезано: `no`

```markdown
# ADR-0009 Local AI Through Ollama

Status: Proposed

## Context

The product is local-first and handles private communications and documents. AI should work without mandatory cloud model calls.

## Decision

Use Ollama as the initial local AI runtime boundary.

## Consequences

- Local inference is available by default.
- Model replacement remains feasible.
- Performance depends on user hardware.
- Remote models, if added later, must be opt-in and policy controlled.
```

### `docs/adr/ADR-0010-specialized-agent-system.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0010-specialized-agent-system.md`
- Size bytes / Размер в байтах: `484`
- Included characters / Включено символов: `484`
- Truncated / Обрезано: `no`

```markdown
# ADR-0010 Specialized Agent System

Status: Proposed

## Context

A single generic assistant would blur responsibility and make permissions difficult to reason about.

## Decision

Use specialized agents: HESTIA, MAKOSH, MNEMOSYNE, ATHENA and HEPHAESTUS.

## Consequences

- Agent responsibilities are easier to explain and constrain.
- Tool permissions can be scoped by role.
- HESTIA must coordinate without becoming a hidden god object.
- Agent interactions require audit events.
```

### `docs/adr/ADR-0011-plugin-architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0011-plugin-architecture.md`
- Size bytes / Размер в байтах: `523`
- Included characters / Включено символов: `523`
- Truncated / Обрезано: `no`

```markdown
# ADR-0011 Plugin Architecture

Status: Proposed

## Context

Макошь will need new providers, processors, tools and UI extensions over time. Hardcoding all integrations into core will not scale.

## Decision

Introduce a plugin architecture with manifests, explicit capabilities and bounded runtime access.

## Consequences

- Integrations can evolve outside the core.
- Permissions become visible and enforceable.
- Plugin sandboxing is a security-critical design area.
- The core must expose stable extension points.
```

### `docs/adr/ADR-0012-opentelemetry-observability.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0012-opentelemetry-observability.md`
- Size bytes / Размер в байтах: `541`
- Included characters / Включено символов: `541`
- Truncated / Обрезано: `no`

```markdown
# ADR-0012 OpenTelemetry Observability

Status: Proposed

## Context

Ingestion, projections, indexing and agent workflows will fail in ways that need diagnosis without leaking private content.

## Decision

Use OpenTelemetry for traces, metrics and structured observability.

## Consequences

- Long-running local workflows can be inspected.
- Projection and ingestion latency can be measured.
- Telemetry must avoid message bodies, secrets and private document content.
- A local collector should be supported before any remote telemetry.
```

### `docs/adr/ADR-0013-local-first-data-ownership.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0013-local-first-data-ownership.md`
- Size bytes / Размер в байтах: `541`
- Included characters / Включено символов: `541`
- Truncated / Обрезано: `no`

```markdown
# ADR-0013 Local First Data Ownership

Status: Proposed

## Context

The user must own communication history, graph memory and document-derived knowledge.

## Decision

Design local storage and local operation as the default. Cloud services are optional integrations, not required infrastructure.

## Consequences

- The app remains useful offline for already-ingested data.
- Backup and restore become product-critical.
- Multi-device sync is deferred but must not be made impossible.
- Local machine lifecycle and storage capacity matter.
```

### `docs/adr/ADR-0014-canonical-event-envelope.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0014-canonical-event-envelope.md`
- Size bytes / Размер в байтах: `567`
- Included characters / Включено символов: `567`
- Truncated / Обрезано: `no`

```markdown
# ADR-0014 Canonical Event Envelope

Status: Proposed

## Context

Events from providers, user actions, document processing and agents need consistent metadata for replay, audit and provenance.

## Decision

Define a canonical event envelope with event ID, type, schema version, timestamps, source, actor, subject, payload, provenance, causation and correlation IDs.

## Consequences

- Cross-domain events can be processed uniformly.
- Replay and audit become practical.
- All producers must populate required metadata.
- Payload schemas require version discipline.
```

### `docs/adr/ADR-0015-command-query-separation.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0015-command-query-separation.md`
- Size bytes / Размер в байтах: `532`
- Included characters / Включено символов: `532`
- Truncated / Обрезано: `no`

```markdown
# ADR-0015 Command Query Separation

Status: Proposed

## Context

Макошь has durable state transitions and complex read models. Mixing writes, reads and AI side effects would make behavior hard to test.

## Decision

Separate commands from queries at the application boundary.

## Consequences

- Durable mutations pass through explicit validation.
- Query models can be optimized for UI and AI retrieval.
- Agents can be restricted to read-only or side-effecting tools.
- More boilerplate is required in application services.
```

### `docs/adr/ADR-0016-secrets-and-encryption-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0016-secrets-and-encryption-boundary.md`
- Size bytes / Размер в байтах: `550`
- Included characters / Включено символов: `550`
- Truncated / Обрезано: `no`

```markdown
# ADR-0016 Secrets and Encryption Boundary

Status: Proposed

## Context

Provider tokens, app passwords, private keys and backup credentials are high-value secrets.

## Decision

Keep secrets outside ordinary application tables and access them through an OS-backed secret store or encrypted vault abstraction.

## Consequences

- Database compromise does not automatically expose provider credentials.
- Backups need explicit treatment for encrypted secret export.
- Cross-platform behavior must be validated.
- Tests need secret-store substitutes.
```

### `docs/adr/ADR-0017-document-processing-pipeline.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0017-document-processing-pipeline.md`
- Size bytes / Размер в байтах: `554`
- Included characters / Включено символов: `554`
- Truncated / Обрезано: `no`

```markdown
# ADR-0017 Document Processing Pipeline

Status: Proposed

## Context

Documents require OCR, extraction, summary, entity linking, versioning and indexing. Doing this inline with upload would create latency and failure coupling.

## Decision

Use an asynchronous document processing pipeline driven by events.

## Consequences

- Upload can complete before expensive processing finishes.
- Failed processing steps can be retried.
- Users need visible processing states.
- Document versions and extraction outputs must be immutable enough for provenance.
```

### `docs/adr/ADR-0018-provider-adapter-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0018-provider-adapter-boundary.md`
- Size bytes / Размер в байтах: `548`
- Included characters / Включено символов: `548`
- Truncated / Обрезано: `no`

```markdown
# ADR-0018 Provider Adapter Boundary

Status: Proposed

## Context

Email, Telegram, WhatsApp and future sources have different APIs, IDs, pagination and delivery semantics.

## Decision

Use provider adapters that preserve raw source records and emit normalized commands or events through application boundaries.

## Consequences

- Provider quirks stay isolated.
- Raw evidence remains available for replay and debugging.
- Adapter contracts must handle idempotency and rate limits.
- Outbound provider writes require separate capability checks.
```

### `docs/adr/ADR-0019-contact-identity-resolution.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0019-contact-identity-resolution.md`
- Size bytes / Размер в байтах: `550`
- Included characters / Включено символов: `550`
- Truncated / Обрезано: `no`

```markdown
# ADR-0019 Contact Identity Resolution

Status: Proposed

## Context

People appear through emails, phone numbers, usernames, aliases and organizations. Incorrect automatic merges can damage memory integrity.

## Decision

Model identity resolution as confidence-scored candidates with explicit merge and split workflows.

## Consequences

- Ambiguity remains visible.
- User correction can improve future linking.
- Contact profiles require provenance for channels and aliases.
- Fully automatic identity collapse is disallowed for ambiguous cases.
```

### `docs/adr/ADR-0020-task-candidate-lifecycle.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0020-task-candidate-lifecycle.md`
- Size bytes / Размер в байтах: `493`
- Included characters / Включено символов: `493`
- Truncated / Обрезано: `no`

```markdown
# ADR-0020 Task Candidate Lifecycle

Status: Proposed

## Context

AI can extract tasks from messages and documents, but false positives can create operational noise or false obligations.

## Decision

AI extraction creates task candidates. Activation requires user confirmation or a narrowly scoped policy.

## Consequences

- The user remains in control of commitments.
- Task provenance remains clear.
- UI must support efficient review.
- Automation policies require careful design later.
```

### `docs/adr/ADR-0021-calendar-as-event-source.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0021-calendar-as-event-source.md`
- Size bytes / Размер в байтах: `515`
- Included characters / Включено символов: `515`
- Truncated / Обрезано: `no`

```markdown
# ADR-0021 Calendar as Event Source

Status: Proposed

## Context

Meetings and calendar changes are important context for projects, tasks and communications.

## Decision

Treat calendars as event sources that produce meeting, schedule and attendance events, not merely UI widgets.

## Consequences

- Meeting context can enrich graph and search.
- Calendar provider changes need idempotent sync.
- Meeting completion can trigger summaries and task extraction.
- Outbound calendar edits require capability checks.
```

### `docs/adr/ADR-0022-no-fine-tuning-on-private-data.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0022-no-fine-tuning-on-private-data.md`
- Size bytes / Размер в байтах: `539`
- Included characters / Включено символов: `539`
- Truncated / Обрезано: `no`

```markdown
# ADR-0022 No Fine Tuning on Private Data

Status: Proposed

## Context

Private communications and documents must remain portable and explainable. Fine-tuning would bury user memory inside model weights.

## Decision

Do not fine-tune models on private user data. Use graph, RAG, vector search and structured memory.

## Consequences

- Model replacement is feasible.
- Generated answers can cite sources.
- Retrieval quality becomes critical.
- Some personalized behavior must be represented structurally rather than learned implicitly.
```

### `docs/adr/ADR-0023-rebuildable-projections.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0023-rebuildable-projections.md`
- Size bytes / Размер в байтах: `469`
- Included characters / Включено символов: `469`
- Truncated / Обрезано: `no`

```markdown
# ADR-0023 Rebuildable Projections

Status: Proposed

## Context

Search indexes, graph views, timelines and summaries will change as schemas and extraction improve.

## Decision

Treat projections and indexes as rebuildable from canonical events, raw records and document artifacts.

## Consequences

- Projection bugs can be repaired.
- Rebuild tooling is required.
- Derived state must record source versions.
- Canonical storage must be complete enough to rebuild.
```

### `docs/adr/ADR-0024-idempotent-imports.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0024-idempotent-imports.md`
- Size bytes / Размер в байтах: `521`
- Included characters / Включено символов: `521`
- Truncated / Обрезано: `no`

```markdown
# ADR-0024 Idempotent Imports

Status: Proposed

## Context

Provider sync jobs will be interrupted, retried and re-run. Duplicate messages or documents would corrupt timelines and graph links.

## Decision

All imports must be idempotent using provider IDs, content fingerprints, import batch IDs and source-specific checkpoints.

## Consequences

- Retry safety improves.
- Provider-specific identity logic is required.
- Some sources without stable IDs need fingerprint strategy.
- Import audit data must be retained.
```

### `docs/adr/ADR-0025-keyboard-first-command-palette.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0025-keyboard-first-command-palette.md`
- Size bytes / Размер в байтах: `512`
- Included characters / Включено символов: `512`
- Truncated / Обрезано: `no`

```markdown
# ADR-0025 Keyboard First Command Palette

Status: Proposed

## Context

The target product is a daily-use personal operating environment for a technical user. Frequent workflows must be fast.

## Decision

Make keyboard-first navigation and command palette a primary UI pattern.

## Consequences

- Power workflows become efficient.
- Commands need consistent naming and discoverability.
- Accessibility and focus management are mandatory.
- Mouse/touch interactions remain supported but are not the only path.
```
