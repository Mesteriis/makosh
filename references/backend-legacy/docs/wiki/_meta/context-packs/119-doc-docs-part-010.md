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

- Chunk ID / ID чанка: `119-doc-docs-part-010`
- Group / Группа: `docs`
- Role / Роль: `doc`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/documentation-map.md`

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

### `docs/roadmap/v2-closure-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/v2-closure-checklist.md`
- Size bytes / Размер в байтах: `2649`
- Included characters / Включено символов: `2649`
- Truncated / Обрезано: `no`

```markdown
# V2 Closure Checklist

## Release Goal

Version 2.0 is complete when Макошь makes graph-backed memory central:
messages, Persona-compatible identity records, documents and projects are
connected through rebuildable graph projections, reviewable workflow candidates,
visible document processing state and desktop-only backend-backed UI surfaces.

## In Scope

- graph core projection from Persona-compatible identity records, messages and documents.
- Project memory spine with project timeline views and keyword-derived evidence-backed links.
- Project link review commands backed by canonical events.
- Source-backed task candidates from messages and documents with explicit review before active local tasks exist.
- Persona identity merge/split review without ambiguous automatic identity collapse.
- Document processing jobs and artifacts for Markdown/text extraction and OCR state.
- Protected read/write APIs using the local shared secret header for protected requests.
- Desktop/laptop SvelteKit surfaces for graph, projects, task candidates, Persona identity and document processing.
- Full local validation through `make validate`.

## Out Of Scope For V2

- Version 3 agent runtime.
- Ollama or AI-backed extraction.
- Embedding provider and retrieval planner.
- Remote OCR service.
- Provider task/calendar writes.
- Graph editing.
- Mobile UI design, implementation or validation.

## Acceptance Gate Status

- [x] graph core projection is implemented and covered by live PostgreSQL smoke validation.
- [x] Knowledge Graph explorer reads summary, search, picker and neighborhood APIs.
- [x] Project memory spine is implemented with project records, timelines and graph links.
- [x] Project link review commands append canonical events and survive graph rebuild.
- [x] Task candidate review creates active local tasks only after explicit confirmation.
- [x] Persona identity review creates conservative merge candidates without mutating source identity records.
- [x] Document processing jobs/artifacts exist and Markdown extraction is implemented.
- [x] Persona identity supports explicit split review for confirmed merge links.
- [x] Document processing failed jobs can be retried through a protected event-backed command.
- [x] `make validate` includes live PostgreSQL smoke coverage for workflow APIs.
- [x] Backend README documents all workflow APIs and dev commands.
- [x] Frontend README documents V2 desktop surfaces and validation commands.
- [x] Full `make validate` passes from a clean checkout with Docker available.
- [x] Desktop browser smoke validates graph, projects, tasks, Personas and document-processing surfaces.
```

### `docs/roadmap/v2-graph-core-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/v2-graph-core-checklist.md`
- Size bytes / Размер в байтах: `1884`
- Included characters / Включено символов: `1884`
- Truncated / Обрезано: `no`

```markdown
# V2 Graph Core Checklist

## Release Goal

The first Version 2 slice is complete when Макошь builds a deterministic,
read-only Knowledge Graph projection from existing Persona-compatible identity
records, communication messages and documents, exposes protected read APIs, and
renders graph-backed desktop dashboard data.

## In Scope

- PostgreSQL graph projection tables.
- Graph node, edge and evidence store.
- Deterministic graph IDs.
- Idempotent projection from Persona-compatible `persons` records, `communication_messages` and `documents`.
- Exact-email identity linking only.
- Read-only graph summary, neighborhood and search APIs.
- Desktop dashboard graph summary and read-only explorer entry point.
- Live PostgreSQL graph smoke validation.

## Out of Scope

- Fuzzy identity merge.
- Persona identity merge/split UI.
- OCR.
- Entity extraction from document text.
- Task candidate extraction.
- AI summaries.
- Graph editing.
- Mobile graph UI.

## Acceptance Gate Status

- [x] `backend/migrations/0010_create_graph_core.sql` creates graph tables and constraints.
- [x] Graph node upserts are idempotent.
- [x] Graph edge upserts are idempotent.
- [x] System-created graph edges require evidence.
- [x] V1 graph projection from Persona-compatible identity records, messages and documents is idempotent.
- [x] Exact email rules do not create fuzzy Persona merges.
- [x] `GET /api/v1/graph/summary` has auth and response coverage.
- [x] `GET /api/v1/graph/neighborhood` has auth, not-found, unsupported-depth and happy-path coverage.
- [x] `GET /api/v1/graph/search` has auth, empty-query and happy-path coverage.
- [x] `make backend-graph-smoke-dev` passes against live PostgreSQL.
- [x] `make validate` includes the graph smoke target.
- [x] `pnpm --dir frontend check` passes after graph UI wiring.
- [x] `pnpm --dir frontend build` passes after graph UI wiring.
```

### `docs/roadmap/v3-closure-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/v3-closure-checklist.md`
- Size bytes / Размер в байтах: `3033`
- Included characters / Включено символов: `3033`
- Truncated / Обрезано: `no`

```markdown
# V3 Closure Checklist

## Release Goal

Version 3.0 is complete when Макошь exposes local, source-backed AI workflows over the existing memory spine: Ollama runtime health, pgvector semantic retrieval, cited answers, suggested task candidates, meeting prep packets, persisted agent run history and desktop AI surfaces.

## In Scope

- Local Ollama chat and embedding provider boundary.
- pgvector `halfvec(2560)` semantic embedding store for messages, documents, projects, tasks and Personas.
- Semantic indexing from existing canonical projections.
- Retrieval planner combining semantic nearest neighbors and local text match signals.
- Prompt builder that treats retrieved source text as untrusted context.
- Registered agents: `HESTIA`, `MAKOSH`, `MNEMOSYNE`, `ATHENA`.
- Persisted AI run history with model config, prompt template version, citations, answer, timings, constant frontend actor and correlation IDs.
- Canonical events for AI run requested/completed/failed and task extraction completion.
- Protected AI APIs with local shared secret header.
- AI task extraction that creates only `suggested` candidates linked to `agent_run_id`.
- Meeting prep packets backed by local sources, without calendar/provider writes.
- Desktop-only AI Agents and scoped ask/brief/task refresh surfaces.
- Live validation against Ollama at the configured local endpoint.

## Out Of Scope For V3

- Cloud models.
- Fine-tuning private data.
- Autonomous activation.
- External email, calendar, provider or task writes.
- Calendar ingestion as a prerequisite for meeting prep.
- Provider adapter implementation.
- Mobile UI design, implementation or validation.

## Acceptance Gate Status

- [x] ADR-0049 documents the V3 AI runtime, retrieval and provenance policy.
- [x] Docker Compose uses pinned `pgvector/pgvector:0.8.2-pg16`.
- [x] Backend migration enables pgvector and creates `halfvec(2560)` semantic embeddings.
- [x] Backend migration creates persisted AI agent run history.
- [x] `task_candidates` supports `agent_run_id` for AI-suggested candidates.
- [x] Ollama client covers `/api/version`, `/api/tags`, `/api/chat` and `/api/embed`.
- [x] AI APIs expose status, agents, run history, answers, task refresh and meeting prep.
- [x] AI APIs require `X-Макошь-Secret`.
- [x] AI answers return citations and persist completed run history.
- [x] AI task extraction creates suggested candidates only.
- [x] Meeting prep returns a source-backed briefing without calendar writes.
- [x] `make backend-ai-smoke-dev` validates pgvector integration and live Ollama model behavior.
- [x] Desktop AI Agents tab reads live backend AI status, agents, run history, answer form and citations.
- [x] Scoped desktop Ask AI / Prepare brief controls are available on source-backed surfaces.
- [x] AI task extraction action reuses the existing task candidate review queue.
- [x] `make validate`, `make frontend-check` and `make frontend-build` pass.
- [x] Desktop browser smoke validates AI Agents, cited answer, task refresh and meeting prep.
```

### `docs/roadmap/v4-closure-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/v4-closure-checklist.md`
- Size bytes / Размер в байтах: `2685`
- Included characters / Включено символов: `2685`
- Truncated / Обрезано: `no`

```markdown
# V4 Closure Checklist

## Release Goal

Version 4.0 is complete when Макошь provides a desktop-configurable Telegram client foundation with multiple Telegram user and bot accounts, policy-approved automated sending, 1:1 audio call state, local call transcription artifacts, plugin/capability policy visibility and backup-aware V4 data handling.

## In Scope

- ADR-0050 governs Telegram runtime, policy-backed sending and call intelligence.
- Multiple `telegram_user` and `telegram_bot` accounts.
- Telegram fixture runtime for CI and smoke validation.
- Account-scoped Telegram raw records, checkpoints, chats and projected messages.
- UI-configured templates and automation policies.
- Automated-send dry-run and audit trail.
- 1:1 audio call metadata and transcript artifact storage.
- Local speech-to-text provider boundary with fixture provider by default.
- Desktop-only Telegram surfaces for Telegram, policies and call transcripts.
- Protected Telegram capability contract for available, blocked and unsupported capabilities.

## Out Of Scope For V4

- Video calls.
- Group calls.
- Screen sharing.
- Hidden recording.
- Cloud transcription by default.
- Mobile UI.
- Training or fine-tuning on Telegram data.
- Third-party plugin code execution.

## Acceptance Gate Status

- [x] ADR-0050 documents Telegram, policy automation and call intelligence constraints.
- [x] V4 roadmap closure checklist exists.
- [x] Provider account model accepts `telegram_user` and `telegram_bot` without breaking email providers.
- [x] Telegram secret purposes are account-scoped and compatible only with non-plaintext secret references.
- [x] Backend migration creates Telegram chat, outbound policy and call transcript tables.
- [x] Backend exposes protected `/api/v1/integrations/telegram/*`, `/api/v1/policies/*` and `/api/v1/calls/*` foundation endpoints.
- [x] Automated-send dry-run rejects sends outside enabled policies.
- [x] Automated-send dry-run records auditable preview metadata without storing secret values.
- [x] Call transcript storage preserves account, call and source provenance.
- [x] Protected `/api/v1/integrations/telegram/capabilities` exposes fixture-ready, live-blocked and Telegram unsupported capabilities.
- [x] Desktop Telegram account, policy, call transcript and runtime guardrail surfaces call protected backend APIs.
- [x] `make backend-telegram-smoke-dev` covers Telegram fixture runtime, policy and transcript storage.
- [x] `make validate`, `make frontend-check` and `make frontend-build` pass after Telegram UI integration.
- [x] Desktop browser smoke validates Telegram, policy and call transcript Telegram surfaces render without layout breakage.
```

### `docs/roadmap/v5-closure-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/v5-closure-checklist.md`
- Size bytes / Размер в байтах: `2456`
- Included characters / Включено символов: `2456`
- Truncated / Обрезано: `no`

```markdown
# V5 Closure Checklist

## Release Goal

Version 5.0 is complete when Макошь can use WhatsApp Web as a local-first, user-visible companion source while preserving source provenance, privacy, auditability, graph-backed recall and long-horizon personal knowledge workflows.

## In Scope

- ADR-0051 governs WhatsApp Web companion boundaries.
- `whatsapp_web` provider account metadata and account-scoped secret references.
- Fixture/manual WhatsApp Web session state for CI and local development.
- Append-only WhatsApp Web raw records and canonical message projections.
- Desktop-only WhatsApp Web account, session and sync status surfaces.
- Explicit capability reporting for fixture/manual runtime, blocked live runtime and unsupported automation.
- Long-horizon analytics and decision/relationship/project memory improvements from V5 roadmap.

## Out Of Scope For V5 Foundation

- Hidden WhatsApp Web scraping.
- Reverse-engineered protocol runtime as a production dependency.
- Bulk messaging, auto-messaging or auto-dialing.
- Live outbound WhatsApp sends.
- Mobile UI.
- Training or fine-tuning on WhatsApp data.
- Treating WhatsApp Business Platform Cloud API as personal WhatsApp Web.

## Acceptance Gate Status

- [x] ADR-0051 documents WhatsApp Web companion constraints.
- [x] V5 roadmap closure checklist exists.
- [x] Provider account model accepts `whatsapp_web` without breaking email or Telegram providers.
- [x] WhatsApp Web session secret purpose is account-scoped and compatible only with non-plaintext session protection secrets.
- [x] Backend migration extends provider/message constraints and creates WhatsApp Web session metadata storage.
- [x] Backend exposes protected `/api/v1/integrations/whatsapp/*` fixture/manual foundation endpoints.
- [x] WhatsApp Web fixture ingestion projects raw messages into canonical communication messages.
- [x] Protected `/api/v1/integrations/whatsapp/capabilities` exposes fixture-ready, live-blocked and unsupported WhatsApp capabilities.
- [x] Desktop WhatsApp account/session/status surfaces call protected backend APIs.
- [x] `make backend-whatsapp-smoke-dev` covers WhatsApp Web fixture/manual foundation.
- [x] `make validate`, frontend checks and desktop browser smoke pass after WhatsApp UI integration.

## Remaining V5 Risk

- Live WhatsApp Web runtime and live outbound sends remain blocked until an explicit runtime ADR, capability model and user-visible consent flow are approved.
```

### `docs/site/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/site/README.md`
- Size bytes / Размер в байтах: `350`
- Included characters / Включено символов: `350`
- Truncated / Обрезано: `no`

```markdown
# Documentation Site

Status: documentation package aligned to the current repository structure.

This package contains the static GitHub Pages documentation portal and assets.
Canonical content remains in the Markdown documentation packages.

## Navigation

- [Site Entrypoint](./index.html)
- [Site Styles](./makosh-docs.css)
- [Assets](./assets/)
```

### `docs/ui/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/ui/README.md`
- Size bytes / Размер в байтах: `271`
- Included characters / Включено символов: `271`
- Truncated / Обрезано: `no`

```markdown
# UI

Status: documentation package aligned to the current repository structure.

UI docs describe product interface direction and design-system constraints.
Implementation files live under `frontend/`.

## Navigation

- [Design System Vision](./design-system-vision.md)
```

### `docs/ui/design-system-vision.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/ui/design-system-vision.md`
- Size bytes / Размер в байтах: `1665`
- Included characters / Включено символов: `1665`
- Truncated / Обрезано: `no`

```markdown
# Design System Vision

## Product Feel

Макошь should feel like a serious personal operating environment: fast, calm, dense, modern and explainable. The design language can use glassmorphism selectively, but readability and workflow speed take priority over decoration.

## Inspirations

- Arc: spatial navigation and modern surface treatment
- Raycast: command palette and keyboard-first utility
- Linear: dense operational workflows and state clarity
- Notion: flexible knowledge surfaces
- Obsidian: graph thinking and local-first knowledge

## Principles

- UI surfaces expose relationships, not only lists.
- Important provenance is visible without overwhelming the main flow.
- AI output is visually distinct from source evidence.
- High-risk actions are visibly confirmed.
- Motion supports orientation and does not hide latency.
- Dark and light themes must both be first-class.

## Core Components

- app shell
- command palette
- timeline item
- message thread
- persona card
- graph node panel
- document viewer
- task row
- source citation chip
- confidence indicator
- permission prompt
- agent activity item

## Visual Direction

- restrained palette with high contrast text
- subtle translucency for overlays and command surfaces
- compact spacing for information-heavy views
- clear focus states
- accessible typography
- icon-led actions with tooltips

## Interaction Direction

- command palette for most global operations
- inline contextual actions for object-specific operations
- keyboard navigation across lists, panes and search results
- optimistic UI only when rollback is clear
- visible processing states for AI and ingestion tasks
```

### `docs/vault/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/vault/README.md`
- Size bytes / Размер в байтах: `966`
- Included characters / Включено символов: `966`
- Truncated / Обрезано: `no`

```markdown
# Макошь Vault

Status: documentation package aligned to the current repository structure.

Vault documentation mirrors `backend/src/vault`.

The vault layer handles local host-vault lifecycle, secret payload storage
boundaries, key material handling and recovery support. PostgreSQL stores
secret metadata and references only; new provider credential payloads must not
be stored in database tables.

## Current Code Areas

- `backend/src/vault/secrets.rs` - vault-backed secret payload operations.
- `backend/src/vault/lifecycle.rs` - vault initialization and lifecycle.
- `backend/src/vault/storage.rs` - local storage implementation.
- `backend/src/vault/manifest.rs` - vault manifest metadata.
- `backend/src/vault/recovery.rs` - recovery support.

## Documentation Rule

Vault docs must not include secrets, tokens, passwords, private keys or local
`.env` values. Provider account metadata belongs to domains/integrations;
secret payload handling belongs here.
```

### `docs/vision/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/vision/README.md`
- Size bytes / Размер в байтах: `288`
- Included characters / Включено символов: `288`
- Truncated / Обрезано: `no`

```markdown
# Vision

Status: documentation package aligned to the current repository structure.

Vision documents preserve the long-term product direction. Current canonical
product detail lives under `docs/product/` and `docs/foundation/`.

## Navigation

- [Vision Document](./vision-document.md)
```

### `docs/vision/vision-document.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/vision/vision-document.md`
- Size bytes / Размер в байтах: `2678`
- Included characters / Включено символов: `2678`
- Truncated / Обрезано: `no`

```markdown
# Vision Document

## Mission

Макошь is a local-first Personal Memory System with a Personal Operating
System style interface for communications, knowledge, memory, relationships,
projects, documents, decisions, obligations and context.

Макошь does not compete with email clients, messengers, CRM systems, task
trackers, calendar apps or note-taking tools. It absorbs evidence from those
surfaces and turns it into durable, source-backed context.

Canonical vocabulary lives in:

- [Foundation Vision](../foundation/vision.md)
- [Glossary](../foundation/glossary.md)
- [World Model](../foundation/world-model.md)

## North Star

After years of use, Макошь should reliably answer:

- what changed in the owner's world over a period;
- what obligations exist and where they came from;
- which projects are active and why;
- what was discussed around a project;
- why a decision was made;
- which documents support a claim;
- what the full history of a Persona or Organization is;
- what context matters before acting.

Answers must be explainable. Макошь must show source records, events, documents,
communications and relationships behind each conclusion.

## Non-Product Boundaries

Макошь is not:

- an email client with an AI filter;
- a messenger with a unified inbox;
- a CRM;
- a task tracker;
- a calendar app;
- a note-taking app;
- a cloud SaaS;
- a fine-tuned personal LLM;
- a system where chat is the only control surface.

## Long-Term Value

The owner's memory must survive:

- replacement of an LLM;
- replacement of the UI;
- replacement of a search engine;
- replacement of a messaging provider;
- migration between machines;
- temporary unavailability of cloud services.

The foundation is source records, canonical events, domain entities,
relationships, document versions, search indexes and reproducible ingestion
pipelines.

## Product Principles

- Context is more valuable than CRUD.
- Memory is more valuable than an interface.
- Evidence is more valuable than generated output.
- Local ownership is more valuable than cloud dependency.
- Relationships are more valuable than isolated lists.
- Automation must expose uncertainty.
- The owner must be able to delete, export and restore memory.

## Success Criteria

- The owner can find any meaningful interaction after years of use.
- Макошь connects communications, documents, Personas, Organizations, Projects,
  Tasks, Events, Decisions and Obligations without requiring manual
  classification of everything.
- AI answers include verifiable source references.
- Loss of a model, provider or derived index does not destroy memory.
- Daily workflows remain keyboard-first and context-driven.
```

### `docs/workflows/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/README.md`
- Size bytes / Размер в байтах: `1359`
- Included characters / Включено символов: `1359`
- Truncated / Обрезано: `no`

````markdown
# Макошь Workflow Catalog

Workflows describe how evidence moves through Макошь.

They are not APIs and not implementation modules. They define product behavior
and architectural boundaries that future implementation plans must respect.

## Core Principle

Communication is the primary ingestion spine:

```text
Communication
  -> Evidence
  -> Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
```

Documents, meetings, calls and notes can also produce evidence, but
Communications are the most common entry point.

## Workflow Specs

| Workflow | Spec |
|---|---|
| Communication to Knowledge | [Communication To Knowledge](communication-to-knowledge.md) |
| Communication to Obligation | [Communication To Obligation](communication-to-obligation.md) |
| Meeting to Decisions | [Meeting To Decisions](meeting-to-decisions.md) |
| Document to Context | [Document To Context](document-to-context.md) |
| Contradiction Review | [Contradiction Review](contradiction-review.md) |
| Dossier Generation | [Dossier Generation](dossier-generation.md) |
| Agent Assisted Recall | [Agent Assisted Recall](agent-assisted-recall.md) |

## Boundary Rule

Workflows coordinate domains and engines. They do not own durable entities.
Durable state must be written by the owning domain or as a reviewed engine
observation.
````

### `docs/workflows/agent-assisted-recall.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/agent-assisted-recall.md`
- Size bytes / Размер в байтах: `1634`
- Included characters / Включено символов: `1634`
- Truncated / Обрезано: `no`

````markdown
# Agent Assisted Recall

This workflow explains how an agent retrieves and uses Макошь context.

Agents help the Owner Persona operate the Personal Memory System. They do not
own source truth.

## Trigger

The workflow starts when the owner asks an agent for help, or when an approved
policy allows a scoped agent action.

## Flow

```text
owner or policy request
  -> identify acting agent Persona
  -> check capabilities
  -> retrieve context through domains and engines
  -> cite sources
  -> propose answer, observation or action
  -> review or policy gate
  -> write audited event if action is accepted
```

## Required Outputs

- acting agent identity;
- capability decision;
- retrieved context with citations;
- proposed answer or action;
- review or policy result;
- audit event for accepted side effects.

## Domain And Engine Boundaries

- Agents own run and permission records.
- Domains own source-of-truth updates.
- Search and Memory Engines provide context.
- Risk, Trust and Contradiction engines provide signals.
- Owner Persona remains the system owner.

## Current Implementation Evidence

Current implementation includes AI runtime/control center, Ollama integration,
settings and capability infrastructure. Product-level agent behavior still needs
more explicit permission and Persona graph documentation before broader
automation.

## Migration Plan

1. Require capabilities for side effects.
2. Keep retrieved context cited.
3. Treat agent conclusions as proposed observations unless reviewed.
4. Audit accepted actions.
5. Represent durable agents as `PersonaType: ai_agent` when graph identity is
   needed.
````

### `docs/workflows/communication-to-knowledge.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/communication-to-knowledge.md`
- Size bytes / Размер в байтах: `1713`
- Included characters / Включено символов: `1713`
- Truncated / Обрезано: `no`

````markdown
# Communication To Knowledge

This workflow explains how a communication becomes evidence-backed knowledge.

## Trigger

The workflow starts when Макошь imports or receives:

- email;
- Telegram message;
- WhatsApp message;
- call record;
- meeting-linked communication;
- future provider message.

## Flow

```text
source communication
  -> preserve source evidence
  -> normalize communication
  -> identify participants
  -> link candidate Personas and Organizations
  -> extract claims and entities
  -> compare against accepted memory
  -> create knowledge candidates
  -> review or policy gate
  -> store accepted facts or observations
```

## Required Outputs

- immutable source evidence;
- Communication record;
- participant records;
- candidate entity links;
- extracted claim candidates;
- contradiction observations when conflicts exist;
- reviewed Knowledge or Memory updates.

## Domain And Engine Boundaries

- Communications owns the source communication.
- Personas and Organizations own accepted identity links.
- Knowledge Graph owns accepted relationships.
- Memory Engine assembles memory views.
- Enrichment Engine proposes candidates.
- Consistency / Contradiction Engine detects conflicts.

## Current Implementation Evidence

Current implementation includes mail ingestion, communication messages,
Telegram, WhatsApp, extraction and email intelligence surfaces. The full
workflow is not yet implemented as one explicit pipeline.

## Migration Plan

1. Keep this workflow as the canonical behavior target.
2. Use current mail/Telegram/WhatsApp ingestion as evidence sources.
3. Add reviewable candidates before automatic knowledge updates.
4. Require source citations for accepted facts.
````

### `docs/workflows/communication-to-obligation.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/communication-to-obligation.md`
- Size bytes / Размер в байтах: `2247`
- Included characters / Включено символов: `2247`
- Truncated / Обрезано: `no`

````markdown
# Communication To Obligation

This workflow explains how a communication can create an obligation, follow-up
or task candidate.

## Trigger

The workflow starts when a communication contains a commitment, request, due
date, expectation or responsibility.

Examples:

- someone asks the owner to send a document;
- the owner promises to call back;
- a provider states a deadline;
- a meeting follow-up appears in an email thread.

## Flow

```text
source communication
  -> preserve source evidence
  -> extract commitment language
  -> identify obligated party
  -> identify beneficiary or counterparty
  -> detect due date or condition
  -> create obligation candidate
  -> create optional task candidate
  -> review or policy gate
  -> store accepted obligation and linked task if needed
```

## Required Outputs

- source evidence reference;
- obligation candidate;
- optional task candidate;
- linked Personas, Organizations, Projects or Events;
- confidence and review state;
- risk signal when obligation is urgent or overdue.

## Domain And Engine Boundaries

- Communications owns the source evidence.
- Obligation Engine creates candidates.
- Obligations domain owns accepted obligations.
- Tasks domain owns task lifecycle.
- Risk Engine detects overdue, blocked or high-impact obligations.

## Current Implementation Evidence

Current related implementation exists through task candidates, task rules, task
intelligence and communication workflow state. The accepted Obligation
persistence baseline exists in `backend/src/domains/obligations/mod.rs`.
Message task candidate refresh now uses `backend/src/engines/obligation/`
for explicit commitment/request language when the legacy task scanner does not
match.

This is still not the full communication-to-obligation workflow. Accepted
Obligation backend list/review routes exist, but candidate-to-Obligation
creation, provider-wide extraction, meeting/document adapters and desktop
review UI routing remain incomplete.

## Migration Plan

1. Keep obligations distinct from tasks.
2. Require review before converting candidates into accepted obligations.
3. Link accepted obligations to tasks only when action is needed.
4. Add extraction and review workflow before automated capture.
````

### `docs/workflows/contradiction-review.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/contradiction-review.md`
- Size bytes / Размер в байтах: `1754`
- Included characters / Включено символов: `1754`
- Truncated / Обрезано: `no`

````markdown
# Contradiction Review

This workflow explains how Макошь handles conflicts between new evidence and
accepted memory.

User-facing alias: Polygraph review.

## Trigger

The workflow starts when the Consistency / Contradiction Engine finds a conflict
between new evidence and accepted memory, knowledge, obligation state or
decision state.

## Flow

```text
new evidence
  -> compare with accepted memory
  -> create contradiction observation
  -> collect old and new source references
  -> classify conflict type
  -> present review item
  -> owner or policy resolves
  -> update memory, mark disputed, create task or keep existing state
```

## Review Outcomes

- accept new claim;
- keep existing memory;
- mark both claims disputed;
- split entities;
- update relationship confidence;
- create verification task;
- defer until more evidence exists.

## Required Outputs

- contradiction observation;
- old source reference;
- new source reference;
- affected entities;
- confidence and severity;
- review outcome.

## Domain And Engine Boundaries

- Consistency / Contradiction Engine creates observations.
- Domains own accepted state updates.
- Memory Engine updates memory views after accepted changes.
- Trust Engine can use reviewed contradictions as signals.
- Risk Engine can create attention items for unresolved conflicts.

## Current Implementation Evidence

No dedicated implementation exists yet. This workflow is target documentation
approved during product refactoring.

## Migration Plan

1. Start with review items, not automatic overwrites.
2. Require source citations before creating a contradiction observation.
3. Add ADR before persistence or route implementation.
4. Use communications and documents as initial evidence sources.
````

### `docs/workflows/document-to-context.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/document-to-context.md`
- Size bytes / Размер в байтах: `1644`
- Included characters / Включено символов: `1644`
- Truncated / Обрезано: `no`

````markdown
# Document To Context

This workflow explains how a document becomes useful context.

## Trigger

The workflow starts when Макошь imports, creates or updates:

- PDF;
- Office document;
- image;
- Markdown document;
- lightweight note;
- attachment promoted to document evidence.

## Flow

```text
document version
  -> preserve artifact and metadata
  -> extract text and structure
  -> classify document type
  -> extract entities, claims and dates
  -> link candidate Personas, Organizations, Projects and Tasks
  -> check contradictions
  -> create context candidates
  -> review or policy gate
  -> update accepted memory or graph links
```

## Required Outputs

- immutable document version;
- extraction artifacts;
- entity and relationship candidates;
- knowledge candidates;
- contradiction observations when conflicts exist;
- context links to owning domains.

## Domain And Engine Boundaries

- Documents owns artifacts, versions and extraction outputs.
- Knowledge Graph owns accepted relationship records.
- Memory Engine assembles context packs.
- Search Engine indexes derived text.
- Consistency / Contradiction Engine detects conflicts with accepted memory.

## Current Implementation Evidence

Documents and document processing exist. Notes are currently document-like
artifacts. Attachment intelligence exists under the Documents domain.

## Migration Plan

1. Keep extracted output derived until reviewed.
2. Preserve document version immutability.
3. Link documents to Projects, Personas, Organizations, Tasks, Decisions and
   Obligations through graph relationships.
4. Avoid treating document summaries as source truth.
````

### `docs/workflows/dossier-generation.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/dossier-generation.md`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1620`
- Truncated / Обрезано: `no`

````markdown
# Dossier Generation

This workflow explains how Макошь assembles a dossier for a Persona,
Organization, Project or other entity.

## Trigger

The workflow starts when a user or agent requests an entity context view, or when
a background process refreshes a derived read model.

## Flow

```text
entity request
  -> collect identity and relationships
  -> collect accepted memory
  -> collect recent timeline
  -> collect documents, communications, tasks and decisions
  -> collect risk, trust and contradiction observations
  -> assemble cited dossier
  -> expose read model
```

## Persona Dossier Fields

- summary;
- interests;
- projects;
- organizations;
- skills;
- communication patterns;
- ai_observations;
- open obligations;
- recent timeline;
- unresolved contradictions.

## Required Outputs

- dossier summary;
- cited sections;
- freshness metadata;
- confidence or review markers;
- unresolved gaps and contradictions.

## Domain And Engine Boundaries

- Domains own source records.
- Memory Engine assembles durable memory.
- Timeline Engine provides chronological context.
- Trust and Risk Engines provide signals.
- Consistency / Contradiction Engine provides unresolved conflict observations.

## Current Implementation Evidence

Persona and Organization memory/dossier-like concepts exist. A single
cross-domain dossier workflow is not yet implemented.

## Migration Plan

1. Keep dossiers as derived read models.
2. Require citations in every dossier section.
3. Avoid using AI observations as accepted truth without review.
4. Add entity-specific dossier specs only when they reuse this workflow.
````

### `docs/workflows/meeting-to-decisions.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/workflows/meeting-to-decisions.md`
- Size bytes / Размер в байтах: `1875`
- Included characters / Включено символов: `1875`
- Truncated / Обрезано: `no`

````markdown
# Meeting To Decisions

This workflow explains how meetings and calls become decision memory.

## Trigger

The workflow starts when Макошь has evidence from:

- calendar meeting;
- call;
- meeting notes;
- communication thread around a meeting;
- attached agenda or minutes.

## Flow

```text
event evidence
  -> identify attendees
  -> collect linked communications and documents
  -> extract decision candidates
  -> extract alternatives and rationale
  -> link affected Projects, Personas and Organizations
  -> review or policy gate
  -> store accepted Decisions
  -> generate related obligations or tasks when needed
```

## Required Outputs

- event source reference;
- candidate decisions;
- rationale and alternatives where available;
- affected entities;
- accepted Decisions after review;
- linked obligations, tasks or follow-ups.

## Domain And Engine Boundaries

- Calendar/Events owns event records.
- Communications owns message evidence.
- Documents owns meeting notes or attachments.
- Decisions owns accepted decision records.
- Timeline Engine builds the chronological view.
- Memory Engine assembles meeting memory.

## Current Implementation Evidence

Calendar, calls, documents and communications exist. The accepted Decision
persistence baseline exists in `backend/src/domains/decisions/mod.rs`, and
guarded backend routes can list accepted Decisions and update accepted Decision
review state.

This is still not the full meeting-to-decision workflow. Meeting-to-decision
extraction, candidate-to-Decision review, desktop UI and adapters from
`meeting_outcomes` are not implemented yet.

## Migration Plan

1. Keep decision capture as candidate-first.
2. Require evidence citations for every accepted decision.
3. Feed reviewed candidates into the ADR-0089 Decisions domain model.
4. Link decisions to Projects before deriving project state from them.
````
