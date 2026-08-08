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

- Chunk ID / ID чанка: `112-doc-docs-part-003`
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

### `docs/domains/graph/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/graph/README.md`
- Size bytes / Размер в байтах: `2066`
- Included characters / Включено символов: `2066`
- Truncated / Обрезано: `no`

```markdown
# Knowledge Graph Design

Status: documentation package aligned to the current repository structure.

## Purpose

The knowledge graph represents durable relationships between Макошь world-model
entities. It is the primary substrate for relationship-aware memory and context.

The graph is not a generic visualization feature. It stores relationship records
with provenance, confidence and review state.

## Core Entities

- Persona.
- Organization.
- Project.
- Document.
- Communication.
- Event.
- Task.
- Decision.
- Obligation.
- Location.
- ChannelAccount.
- Attachment.

## Relationship Objects

Relationships are first-class records, not anonymous edges. A relationship must
store:

- source entity;
- target entity;
- relationship type;
- directionality where relevant;
- confidence;
- provenance;
- valid time range where relevant;
- created-by source: owner, ingestion, agent, rule or import;
- review state where inferred.

## Relationship Examples

- `persona_member_of_organization`
- `persona_participated_in_event`
- `communication_mentions_project`
- `document_related_to_project`
- `task_created_from_communication`
- `decision_made_in_event`
- `organization_related_to_project`
- `obligation_derived_from_communication`

## Identity Resolution

The graph must support uncertain identity:

- multiple digital traces per Persona;
- provider-specific usernames;
- phone numbers;
- aliases;
- merged and split identities;
- confidence-scored candidates.

Automatic merges are risky. High-confidence suggestions may be staged, but user
review must exist for ambiguous identities.

## Provenance

Every inferred entity or relationship must link back to evidence:

- source record ID;
- communication ID;
- document version;
- event ID;
- extraction run;
- agent run;
- manual owner action.

Graph answers without provenance are incomplete.

## Engine Boundary

The graph is a domain/projection boundary for relationships. Search, Timeline,
Trust, Risk and Memory engines may use graph relationships, but they do not own
the relationship source of truth.
```

### `docs/domains/notes/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/notes/README.md`
- Size bytes / Размер в байтах: `1292`
- Included characters / Включено символов: `1292`
- Truncated / Обрезано: `no`

```markdown
# Notes Boundary

Status: documentation package aligned to the current repository structure.

Notes are lightweight capture artifacts in the current Макошь model.

They are not a first-class domain unless a future ADR promotes them.

## Current Definition

A Note is a document-like artifact or memory input that may contain:

- owner-written text;
- meeting notes;
- quick observations;
- pasted evidence;
- draft thinking;
- temporary capture.

Notes can become evidence for Knowledge, Tasks, Decisions, Obligations,
Projects, Personas or Organizations after review or linking.

## Boundary

Notes do not own:

- durable truth;
- global knowledge;
- task lifecycle;
- document versioning beyond the Documents domain rules;
- memory state without review.

## Current Implementation Evidence

The frontend contains a Notes surface, but the backend domain list does not
include a dedicated notes module. Existing document documentation treats
lightweight notes as document-like artifacts.

## Migration Plan

1. Continue treating Notes as document-like artifacts.
2. Do not introduce a Notes domain in documentation without an ADR.
3. If Notes become first-class later, define how they differ from Documents,
   Knowledge and Memory.
4. Keep note-derived facts reviewable and evidence-backed.
```

### `docs/domains/obligations/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/obligations/README.md`
- Size bytes / Размер в байтах: `5417`
- Included characters / Включено символов: `5417`
- Truncated / Обрезано: `no`

````markdown
# Obligations Domain

Status: documentation package aligned to the current repository structure.

Obligations are commitments, duties or promised responsibilities backed by
evidence.

A Task is an actionable unit. An Obligation is the reason something may need to
be done.

## Responsibilities

The Obligations domain owns:

- obligation records;
- obligated party;
- beneficiary or counterparty;
- source evidence;
- due date or condition when known;
- fulfillment state;
- related tasks and reminders;
- risk and contradiction observations.

The Obligations domain does not own:

- every task;
- every follow-up;
- task status lifecycle;
- communication source records;
- calendar event identity.

## Obligation Sources

Obligations can be extracted from:

- communications;
- meetings;
- calls;
- contracts and documents;
- calendar events;
- manual owner input;
- agent suggestions with review.

The Obligation Engine creates candidates. The domain stores reviewed obligations
or policy-approved low-risk captures.

## Obligation Model

```yaml
Obligation:
  id:
  obligated_entity:
  beneficiary_entity:
  statement:
  status:
  due_at:
  condition:
  evidence:
  linked_tasks:
  linked_events:
  risk_state:
  review_state:
```

## Current Implementation Evidence

Current backend baseline:

- `backend/migrations/0063_create_obligations.sql`;
- `backend/migrations/0066_obligation_graph_projection.sql`;
- `backend/migrations/0067_task_candidate_kind_metadata.sql`;
- `backend/src/domains/obligations/mod.rs`;
- `backend/src/domains/obligations/api.rs`;
- `backend/src/domains/persons/trust.rs`;
- `backend/src/domains/tasks/candidates.rs`;
- `backend/tests/obligations.rs`;
- `backend/tests/obligations_api.rs`;
- `backend/tests/task_candidates.rs`;
- `backend/tests/calendar.rs`;
- ADR-0088.

This baseline provides source-backed Obligation persistence with evidence,
status, review state, risk state, confidence, due date or condition and optional
Task links. It also projects accepted Obligations into the graph for supported
obligated and beneficiary entity kinds, using `obligation` graph nodes and
source-backed `entity_relationship` edges. It explicitly does not auto-create
Tasks.

Task candidate review has a backend baseline for obligation-derived candidates:
message and document candidates produced by the Obligation Engine are stored as
`candidate_kind = obligation_task`, preserve the source `ObligationCandidate` in
metadata and, when user-confirmed, create or update a `user_confirmed`
Obligation with source evidence and a `fulfillment_task` link to the created
Task. Resetting or rejecting that candidate synchronizes the durable Obligation
review state and removes the concrete Task link. Generic task candidates remain
task-only.
Email sync and Telegram/WhatsApp fixture ingestion call the same targeted
message refresh path after Communication projection. This creates suggested
obligation-derived task candidates only; it does not auto-create Tasks or
accepted Obligations.

Meeting outcomes with `outcome_type = promise`, `task` or `follow_up` now adapt
into source-backed `suggested` Obligations without creating Tasks. If the
meeting outcome has an `owner_person_id`, the Obligation is owed by that
Persona; otherwise the meeting Event remains the obligated compatibility anchor.
The meeting outcome keeps the created Obligation id in `linked_entity_id`.

Compatibility `person_promises` created through `PersonPromiseStore::create`
now adapt into source-backed `user_confirmed` Obligations with `raw_record`
evidence. This preserves the old `persons` compatibility surface while making
Obligation the durable commitment record. It does not create Tasks.

Backend routes currently expose:

- `GET /api/v1/obligations?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/obligations?review_state=&limit=`;
- `PUT /api/v1/obligations/{obligation_id}/review`.

These routes are guarded by the local API secret and support accepted
Obligation review state changes. They do not create Tasks or convert task
candidates into accepted Obligations.

The Tasks workspace includes the first desktop review panel for global
suggested Obligations and Decisions, with optional entity-scoped filtering. It
lists Obligations through the guarded Obligation route and submits explicit
owner confirm/reject review state without creating Tasks or converting
candidates into accepted Obligations.

Related behavior still exists through:

- `backend/src/domains/tasks/candidates.rs`;
- `backend/src/domains/tasks/rules.rs`;
- `backend/src/domains/tasks/intelligence.rs`;
- `backend/src/domains/persons/trust.rs`;
- meeting outcomes;
- communication extraction and workflow state;
- task candidate migrations.

## Migration Plan

1. Keep Obligations distinct from Tasks in all documentation.
2. Keep the ADR-0088 persistence boundary intact.
3. Expand Obligation Engine extraction beyond explicit message/document task
   candidates and the current meeting outcome adapter.
4. Expand candidate-to-Obligation review routing beyond the current
   obligation-derived task-candidate path and add reviewed Obligation links to
   documents, events and compatibility sources without converting every
   obligation into a task.
5. Project reviewed Obligations into timeline and dossier views.
6. Use the Consistency / Contradiction Engine when new evidence conflicts with
   obligation status or remembered commitments.
````

### `docs/domains/organizations/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/organizations/README.md`
- Size bytes / Размер в байтах: `1112`
- Included characters / Включено символов: `1112`
- Truncated / Обрезано: `no`

```markdown
# Макошь Organizations

Status: documentation package aligned to the current repository structure.

Organizations are first-class memory anchors for companies, institutions,
agencies, communities and similar collective actors.

An Organization is not:

- a field on a Persona;
- a Project;
- a CRM account object.

## Domain Boundary

Organizations own:

- legal and display identity;
- domains and identifiers;
- relationships to Personas and Projects;
- portals, procedures and playbooks;
- organization-specific memory records;
- organization evidence and source provenance.

Organizations do not own:

- Persona identity;
- Project lifecycle;
- Communication source records;
- global Timeline/Memory/Search engines.

## Engine Use

- Memory Engine for organization memory views.
- Timeline Engine for organization history.
- Trust Engine for relationship/source reliability.
- Enrichment Engine for approved candidate data.
- Risk Engine for organization risk observations.
- Search Engine for recall.

## Navigation

- [API Reference](api.md)
- [Data Model](data-model.md)
- [Architecture](architecture.md)
```

### `docs/domains/organizations/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/organizations/api.md`
- Size bytes / Размер в байтах: `2331`
- Included characters / Включено символов: `2179`
- Truncated / Обрезано: `no`

```markdown
# Organizations — API Reference

This file documents current compatibility routes. Canonical Organization,
Relationship and Persona terminology is defined in `../../foundation/glossary.md`.

Base: `/api/v1/`

## Core

| Метод | Путь | Описание |
|---|---|---|
| GET | `/organizations` | Список (?org_type, ?limit) |
| POST | `/organizations` | Создать |
| GET | `/organizations/{id}` | Профиль |
| PUT | `/organizations/{id}` | Обновить |
| GET | `/organizations/search?q=` | Поиск |
| POST | `/organizations/{id}/archive` | Архивировать |

## Identities & Aliases

| Метод | Путь |
|---|---|
| GET, POST | `/organizations/{id}/identities` |
| GET, POST | `/organizations/{id}/aliases` |
| GET | `/organizations/{id}/domains` |

## Departments & Persona Links

| Метод | Путь | Описание |
|---|---|---|
| GET, POST | `/organizations/{id}/departments` | |
| GET, POST | `/organizations/{id}/contacts` | Compatibility route for Organization-Persona links |
| GET | `/organizations/{id}/related` | |

## Timeline & Templates

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/timeline` |
| GET | `/organizations/{id}/templates` |

## Portals, Procedures, Playbooks

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/portals` |
| GET | `/organizations/{id}/procedures` |
| GET | `/organizations/{id}/playbooks` |

## Finance

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/financial` |
| GET | `/organizations/{id}/contracts` |
| GET | `/organizations/{id}/compliance` |
| GET | `/organizations/{id}/services` |
| GET | `/organizations/{id}/products` |

## Enrichment

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/enrichment` |
| POST | `/organizations/{id}/enrichment/{rid}/apply` |

## Risk & Attention

| Метод | Путь | Описание |
|---|---|---|
| GET | `/organizations/{id}/risks` | |
| GET | `/organizations/{id}/health` | Compatibility route for an attention/risk read model |
| POST | `/organizations/{id}/watchlist` | Compatibility route for attention/read-model state |

## Dossier & Context

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/dossier` |
| GET | `/organizations/{id}/brief` |
| GET | `/organizations/{id}/context-pack` |
```

### `docs/domains/organizations/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/organizations/architecture.md`
- Size bytes / Размер в байтах: `1820`
- Included characters / Включено символов: `1820`
- Truncated / Обрезано: `no`

````markdown
# Organizations Architecture

## Position

The Organizations domain owns Organization entities and organization-specific
identity, relationships and operational memory. It uses shared engines for
timeline, memory, trust, enrichment, risk and search.

## Modules

Paths below refer to `backend/src/domains/organizations/`.

| Module | Responsibility |
|---|---|
| `core.rs` | Organization core, store, identities, aliases, domains, departments, Persona links and related organizations |
| `memory.rs` | facts, memory cards, preferences, required documents and memory decay inputs |
| `workflows.rs` | portals, procedures, playbooks and workflow records |
| `finance.rs` | financial information, contracts, compliance, services and products |
| `enrichment.rs` | Enrichment Engine candidates and review state |
| `health.rs` | Risk Engine/attention read models |
| `investigator.rs` | Dossier/context assembly read models |
| `api.rs` | current route handlers and DTO-facing compatibility surface |

## Data Flows

### Organization Creation From Communication

```text
incoming Communication
  -> source/domain evidence
  -> Organization candidate or upsert
  -> Organization identity/domain record
  -> Relationship to Persona when evidence supports it
```

### Identity Resolution

```text
organizations with similar names/domains/VAT
  -> candidate
  -> owner confirm/reject
```

### Enrichment

```text
approved sources
  -> Enrichment Engine
  -> organization enrichment candidate
  -> owner/policy review
  -> organization facts, identities or relationships
```

## ADR

| ADR | Topic |
|---|---|
| 0061 | Organization as first-class entity |
| 0062 | Identity and resolution |
| 0063 | Passive OSINT boundary |
| 0064 | Memory and provenance |
| 0065 | Portals, procedures, playbooks |
| 0066 | Graph integration |
````

### `docs/domains/organizations/data-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/organizations/data-model.md`
- Size bytes / Размер в байтах: `1490`
- Included characters / Включено символов: `1490`
- Truncated / Обрезано: `no`

```markdown
# Organizations Data Model

## `organizations`

| Column | Type | Description |
|---|---|---|
| `organization_id` | TEXT PK | `org:v1:{nanos}` |
| `display_name` | TEXT NOT NULL | display label |
| `legal_name` | TEXT | legal name |
| `org_type` | TEXT | organization type |
| `status` | TEXT | lifecycle/status value |
| `country`, `city`, `address` | TEXT | location metadata |
| `website`, `industry`, `description` | TEXT | descriptive metadata |
| `primary_language`, `timezone` | TEXT | communication/context metadata |
| `tags` | JSONB | user/system tags |
| `org_metadata` | JSONB | structured metadata |
| `registration_number`, `vat`, `cif`, `nif`, `tax_id` | TEXT | legal identifiers |
| `communication_style`, `verbosity`, `formality` | TEXT | communication pattern hints |
| `secondary_languages` | JSONB | additional languages |
| `last_interaction_at`, `interaction_count` | | derived interaction hints |

## Relationship And Trust Boundary

Trust, risk, watchlist and health-like values are attention or engine outputs.
They must not be treated as Organization identity. Organization relationships to
Personas, Projects and other Organizations should be modeled as relationships
with provenance.

## Other Tables

The current schema includes identity, alias, domain, department, relationship,
memory, required document, timeline/workflow, portal, procedure, playbook,
finance, enrichment and risk/alert tables.

Full implementation schema lives in migrations `0038`-`0043`.
```

### `docs/domains/organizations/spec.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/organizations/spec.md`
- Size bytes / Размер в байтах: `2564`
- Included characters / Включено символов: `2564`
- Truncated / Обрезано: `no`

```markdown
# Organizations Domain

Organizations are first-class memory anchors for collective actors such as
companies, institutions, agencies, communities and product teams.

An Organization is not a Persona, Project or CRM account object.

## Responsibilities

The Organizations domain owns:

- organization identity;
- legal and display names;
- domains, websites and external identifiers;
- departments and sub-units;
- relationships to Personas;
- relationships to Projects;
- portals, procedures and playbooks;
- organization-specific memory;
- risk, finance and enrichment observations with provenance.

The Organizations domain does not own:

- Persona identity resolution;
- Project lifecycle;
- Communication source records;
- global Memory, Timeline, Search or Risk engines.

## Relationship Boundary

Organizations connect to Personas and Projects through first-class
relationships, not embedded fields.

Examples:

- Persona works for Organization;
- Persona represents Organization;
- Organization sponsors Project;
- Organization provides service to Owner Persona;
- Organization owns Portal or Procedure.

Relationship records require provenance, confidence and validity period when
time-bounded.

## Memory Boundary

Organization memory answers questions such as:

- what this organization does;
- how to interact with it;
- what procedures or portals matter;
- which Personas are associated with it;
- what risks or obligations exist;
- which Projects, Documents, Communications and Decisions reference it.

Organization memory is evidence-backed. It can use Memory Engine views, but the
Organization domain owns only organization-specific source records and accepted
facts.

## Current Implementation Evidence

Current backend implementation includes:

- `backend/src/domains/organizations/*`;
- organization identity, department, memory, timeline, workflow, finance,
  enrichment, risk and alert migrations `0038` through `0043`;
- Organizations frontend page.

This is closer to the target model than several other domains, but relationship
and engine boundaries still need to be kept explicit in future plans.

## Migration Plan

1. Keep Organization as a separate domain, not a subtype of Persona.
2. Use `organization_proxy` Persona only when an organization-like actor must
   participate in Persona-to-Persona memory.
3. Move relationship semantics toward the shared Relationship model.
4. Keep enrichment and risk outputs as engine-derived observations with
   citations.
5. Avoid reintroducing CRM account language in organization documentation.
```

### `docs/domains/persons/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/README.md`
- Size bytes / Размер в байтах: `4604`
- Included characters / Включено символов: `4590`
- Truncated / Обрезано: `no`

````markdown
# Макошь — Persona Intelligence

Status: documentation package aligned to the current repository structure.

`persons` is the domain that lets Макошь understand people, remember relationships
and build context.

Макошь no longer treats people as contacts. A Persona is not an address-book
entry, CRM lead or contact card. A Persona is a durable memory anchor for a
subject in the local knowledge graph.

```text
Understand people.
Remember relationships.
Build context.
```

## Domain Vision

Макошь is a Personal Memory System. The persons domain provides the Persona
Intelligence layer for that system:

- **Identity**: digital traces that can point to the same subject.
- **Relationships**: explicit edges between Personas with provenance, trust and
  strength.
- **Communication**: observed channels, patterns and interaction context.
- **Memory**: facts, preferences, knowledge and durable notes worth remembering.
- **Timeline view**: Timeline Engine output that explains how relationships
  evolved.
- **Context**: projects, organizations, documents, messages and tasks connected
  through the graph.
- **Dossier**: a generated read model built from trusted memory and
  cited evidence.

The domain is not:

- CRM
- Address book
- Contact manager
- Sales pipeline
- Social network profile store

## Core Model

```yaml
Persona:
  id:
  is_self:
  persona_type:

  identity:
  communication:
  memory:
  timeline_view:
  relationships:
  dossier_read_model:
```

`Persona.id` is the logical identity of the subject inside Макошь. Current
backend tables may still use `person_id` and `/persons` compatibility APIs until
an implementation migration is designed, but new domain language must use
Persona.

## Persona Types

```yaml
PersonaType:
  human
  ai_agent
  organization_proxy
  system
```

`human` represents a person. `ai_agent` allows HESTIA and future agents to exist
inside the graph. `organization_proxy` represents an organization-like subject
when it participates as an actor in relationship memory; it does not replace the
organizations domain. `system` represents local system actors that need explicit
provenance in the graph.

## Self Persona

There is exactly one owner Persona:

```yaml
Persona:
  is_self: true
```

Макошь does not need a separate `UserProfile` or Self domain. Agents, UI actions,
capability checks and generated observations must be attributable to the Owner
Persona when they act for the system owner.

## Relationship First

Relationships are first-class records. They must not be hidden as fields on a
Persona.

```yaml
Relationship:
  source_persona:
  target_persona:
  type:
  trust_score:
  strength_score:
```

A relationship can connect the Owner Persona to another human, an AI agent to the
Owner Persona, two humans, a human to an organization proxy, or any future
Persona type that is allowed by policy.

## Memory First

Each Persona must have structured memory:

```yaml
PersonaMemory:
  facts:
  knowledge:
  preferences:
  memory_cards:
  conflicts:
```

Memory is evidence-backed. AI output may suggest facts or observations, but it is
not source of truth unless it is stored as a reviewed, cited memory record.

## Dossier

Each Persona can have an automatically assembled dossier:

```yaml
Dossier:
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
```

The dossier is a read model. It is generated from identity, relationships,
communication history, memory, Timeline Engine output and graph context. It must
cite the records it uses.

## Persona Intelligence

The previous vocabulary of `fingerprint`, `communication profile`, `trust`,
`analytics` and `investigator` is consolidated under Persona Intelligence.

Persona Intelligence includes:

- communication pattern extraction;
- relationship strength and trust signals;
- memory gap detection;
- identity resolution over digital traces;
- dossier assembly;
- meeting and conversation preparation;
- AI observations with provenance and confidence.

## Navigation

- [Architecture](architecture.md) — target Persona Intelligence architecture.
- [Data Model](data-model.md) — logical model and current persistence
  compatibility notes.
- [API Reference](api.md) — target API shape and legacy endpoint caveats.
- [Status](status.md) — documentation and implementation migration status.
- [Blockers](blockers.md) — architecture blockers for the Persona model.
- [Implementation Alignment Plan](../../refactoring/implementation-alignment-plan.md)
  — cross-domain gap tracking for canonical architecture alignment.
````

### `docs/domains/persons/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/api.md`
- Size bytes / Размер в байтах: `1433`
- Included characters / Включено символов: `1433`
- Truncated / Обрезано: `no`

```markdown
# Persons API Compatibility Notes

This document intentionally does not design a new Persona API.

The current backend may still expose `/api/v1/persons/*` routes and `person_id`
payload fields. Those names are compatibility details from the existing
implementation. The canonical domain language is defined in:

- [Foundation Glossary](../../foundation/glossary.md)
- [World Model](../../foundation/world-model.md)
- [Persona Architecture](architecture.md)

## Interpretation Rules

| Existing API concept | Canonical interpretation |
|---|---|
| `person` | Persona compatibility representation |
| `person_id` | Persona identifier compatibility field |
| `identity` | Persona digital trace or identity-resolution state |
| `roles` | Relationship candidates or compatibility projection |
| `personas` nested under person | Deprecated interaction context concept |
| `fingerprint` | communication pattern output |
| `health` / `watchlist` | relationship attention or Risk Engine read model |
| `investigate` | Dossier/context assembly workflow |
| `analytics` | Persona Intelligence read model |

## Documentation Boundary

API reference files document existing implementation surfaces. They are not the
canonical domain model.

Do not infer new routes, payloads or migrations from this document. Any future
API migration from `/persons` compatibility naming to Persona-native naming
requires a separate ADR and implementation plan.
```

### `docs/domains/persons/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/architecture.md`
- Size bytes / Размер в байтах: `8744`
- Included characters / Включено символов: `8742`
- Truncated / Обрезано: `no`

````markdown
# Persons — Persona Intelligence Architecture

This document describes the target architecture for the `persons` domain after
the Persona refactoring. It is a domain architecture document, not a statement
that every backend table or route has already been migrated.

## Architectural Position

Макошь is a local-first Personal Memory System. The persons domain owns Persona
Intelligence: the structures that allow Макошь to understand subjects, remember
relationships and build context over time.

The domain sits between raw evidence and user-facing memory:

```text
provider records / documents / messages / calendar events
  -> canonical events and append-only source records
  -> identity resolution
  -> Personas and Relationships
  -> Memory records, Timeline Engine views and Dossier read models
  -> UI, agents, search and graph queries
```

## Core Boundaries

| Boundary | Owns | Does not own |
|---|---|---|
| Persona | Subject identity, type and lifecycle | Raw provider payloads |
| Identity Resolution | Digital trace matching, merge/split candidates | Silent ambiguous merges |
| Relationships | Persona-to-Persona edges, trust, strength and provenance | Relationship fields embedded on Persona |
| Memory | Facts, knowledge, preferences, memory cards and conflicts | Uncited AI claims |
| Timeline Engine use | Time-ordered views connected to Personas and Relationships | A separate Persona-owned timeline engine |
| Dossier read model | Generated context and preparation brief | New source-of-truth facts |
| Persona Intelligence | Pattern extraction, observations and scoring | CRM pipeline logic |

## Persona

`Persona` is the root aggregate for a subject in Макошь.

```yaml
Persona:
  id:
  is_self:
  persona_type:
    - human
    - ai_agent
    - organization_proxy
    - system
  identity:
  communication:
  memory:
  timeline_view:
  relationships:
  dossier_read_model:
```

The current implementation still uses `persons` tables and `/persons` routes.
Those names are compatibility details. New documentation and future schema work
must use Persona as the domain concept.

## Self Persona

There is one and only one `Persona` with `is_self: true`. It represents the owner
of the local Макошь instance.

Consequences:

- no separate `UserProfile` domain;
- no separate Self domain;
- local agents act through the Owner Persona;
- audit and provenance records can attribute user-owned actions to the Owner
  Persona when appropriate;
- generated memory about the owner is stored as Persona memory, not as app
  settings.

## Identity Resolution

Identity Resolution merges digital traces into a Persona candidate, not into a
contact record.

Supported traces include:

- email addresses;
- phone numbers;
- Telegram identities;
- WhatsApp identities;
- GitHub accounts;
- LinkedIn profiles;
- document mentions;
- message participants;
- future provider-specific handles.

Ambiguous matches create reviewable candidates. They must not be collapsed
silently. This preserves the ADR-0019 safety property while replacing the old
Contact framing.

## Relationship First

Relationships are primary records:

```yaml
Relationship:
  id:
  source_persona:
  target_persona:
  type:
  trust_score:
  strength_score:
  provenance:
  confidence:
  valid_from:
  valid_to:
```

Relationship examples:

- Owner Persona collaborates with a human Persona.
- Human Persona works with an organization proxy Persona.
- AI agent Persona assists the Owner Persona.
- System Persona produced an automated observation.

Do not model relationships as `primary_role`, `organization_reference` or other
Persona-root fields. `watchlist` and `health_status` may exist only as
temporary UI/risk compatibility projections until the schema/API migration
retires those names.

## Memory First

Persona memory contains structured, cited records:

```yaml
PersonaMemory:
  facts:
  knowledge:
  preferences:
  memory_cards:
  conflicts:
```

Memory records must carry source, confidence and verification metadata. AI can
propose memory, detect conflicts or produce observations, but AI output remains
derived state unless reviewed and stored as evidence-backed memory.

## Timeline Engine Use

The Timeline Engine explains how a Persona or Relationship changed over time.
The persons domain may contribute dated records, but it does not own a separate
Timeline subsystem.

```yaml
TimelineEvent:
  id:
  persona_id:
  relationship_id:
  event_type:
  occurred_at:
  summary:
  source_refs:
  confidence:
```

The existing `relationship_events` table is a transitional projection. The target
model separates Relationship records from dated events and uses the shared
Timeline Engine to present them.

## Dossier

The dossier is generated, not manually maintained.

```yaml
Dossier:
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
  source_refs:
  generated_at:
```

The dossier may be cached as a read model, but the source of truth remains
Persona identity, relationships, memory, timeline, graph evidence and provider
records.

## Persona Intelligence

Persona Intelligence replaces the fragmented legacy terms:

| Old term | New concept |
|---|---|
| communication fingerprint | communication patterns |
| communication profile | Persona communication intelligence |
| trust analytics | relationship intelligence |
| health status | relationship attention or Risk Engine signal |
| watchlist | UI attention preference |
| investigator | dossier and context assembly |
| analytics | Persona Intelligence read models |

The Persona Intelligence layer should be implemented through domain services and
shared engines:

- `IdentityResolutionService`
- `RelationshipIntelligenceService`
- `PersonaMemoryService`
- `DossierAssembler`
- `CommunicationPatternService`
- `PersonaObservationService`

Names above are architectural roles, not a mandate to create these exact Rust
modules in one migration.

## Compatibility With Current Backend

The current Rust backend already contains useful implementation pieces, but they
carry legacy names and CRM-shaped fields.

| Current artifact | Target interpretation |
|---|---|
| `persons` table | Transitional Persona projection table |
| `person_identities` | Persona digital traces |
| `person_identity_candidates` | Identity resolution review candidates |
| `person_roles` | Deprecated; replace with Relationships |
| `person_personas` | Deprecated; Persona is the root entity, not a nested context |
| `relationship_events` | Transitional dated event projection consumed by Timeline Engine |
| `person_facts` | Persona facts |
| `person_memory_cards` | Persona memory cards |
| `person_preferences` | Persona preferences |
| `person_knowledge_conflicts` | Persona memory conflicts |
| `person_expertise` | Persona skills and knowledge signals |
| `person_promises` | Commitment facts or timeline events |
| `person_risks` | Persona/relationship observations requiring evidence |
| `health_status` | Deprecated Risk/attention cache; risk writes materialize this projection |
| `watchlist` | Deprecated UI/read-model cache; writes materialize Persona Preferences |
| `/api/v1/persons` | Legacy compatibility API until a Persona API migration |

Any implementation migration must preserve event sourcing, graph provenance and
reviewed identity resolution. It must not drop current compatibility contracts
without a separate schema/API migration plan.

## Source of Truth

Source-of-truth order inside the domain:

1. Canonical events and append-only provider records.
2. Persona, Identity and Relationship records with provenance.
3. Memory records and dated events derived from reviewed evidence.
4. Dossier and Persona Intelligence read models.
5. Search indexes, embeddings and UI projections.

AI output, embeddings, dossiers and analytics are derived state. They are useful
for context, but they cannot become the source of truth for private memory.

## ADR

Relevant ADR:

- [ADR-0084 Persona Intelligence System](../../adr/ADR-0084-persona-intelligence-system.md)
- [ADR-0001 Event Sourcing as System Spine](../../adr/ADR-0001-event-sourcing-as-system-spine.md)
- [ADR-0008 Knowledge Graph First](../../adr/ADR-0008-knowledge-graph-first.md)
- [ADR-0022 No Fine Tuning on Private Data](../../adr/ADR-0022-no-fine-tuning-on-private-data.md)
- [ADR-0057 Person Memory and Provenance](../../adr/ADR-0057-person-memory-and-provenance.md)
- [ADR-0058 Person Enrichment Engine](../../adr/ADR-0058-person-enrichment-engine.md)
- [ADR-0060 Person Timeline and Graph Integration](../../adr/ADR-0060-person-timeline-and-graph-integration.md)
- [ADR-0074 Person Multi-Channel Identity Model](../../adr/ADR-0074-person-multi-channel-identity-model.md)
````

### `docs/domains/persons/blockers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/blockers.md`
- Size bytes / Размер в байтах: `3028`
- Included characters / Включено символов: `3026`
- Truncated / Обрезано: `no`

```markdown
# Persons — Persona Architecture Blockers

## Current Blockers

The target Persona Intelligence architecture is not blocked at the documentation
level, but implementation is blocked by unresolved migration work.

| Blocker | Why it matters | Required decision |
|---|---|---|
| Legacy `persons` naming | Domain language now requires Persona, while backend/API still expose person/contact history. | Decide route/schema migration strategy. |
| Owner Persona integration incomplete | Storage and compatibility API now support a single `is_self: true` Persona, but agents, UI context assembly and owner-scoped workflows do not consistently use it yet. | Route owner-scoped actions and context assembly through the Owner Persona. |
| Missing Relationship records | Current model stores relationship-like state as fields and timeline events. | Add first-class Relationship storage and API. |
| `person_personas` conflict | Nested personas contradict Persona as the root entity. | Compatibility writes now migrate interaction-context values into Persona Preferences; route/schema deprecation remains a future migration decision. |
| Email-derived `person_id` compatibility | ADR-0074 keeps text IDs for current implementation, but target Persona should not be email-rooted. | Future opaque ID migration ADR if/when implementation changes. |
| Root compatibility caches | Legacy Persona columns still exist for API/schema compatibility. | Root `trust_score`, `watchlist` and `health_status` now have target-aligned write adapters, but route/schema deprecation remains a future migration decision. |
| Dossier workflow not formalized | Backend investigator now emits target Dossier sections with source refs, but cache/workflow/UI semantics remain incomplete. | Define Dossier cache, review and workflow placement. |
| PersonaType adoption incomplete | Compatibility storage and projection support `human`, `ai_agent`, `organization_proxy`, `system`, and current AI registry agents materialize as `ai_agent` Personas; broader UI/agent workflows do not use those types consistently yet. | Route AI agents, organization proxies and system actors through PersonaType-aware graph semantics. |

## Not Blockers

- Keeping current `persons` tables temporarily for compatibility.
- Keeping `/api/v1/persons/*` temporarily as legacy routes.
- Reusing current memory/fact/preference/timeline tables as migration inputs.
- Reusing current identity candidate review workflow if terminology and trace
  semantics are updated.

## Deferred Work

- Backend schema migration from person/contact naming to Persona naming.
- Target `/personas` API implementation.
- UI redesign around Persona Intelligence.
- Dossier cache/workflow implementation beyond the backend read-model baseline.
- Relationship graph UI and traversal views.
- Broader Agent Persona attribution for future agents beyond the current AI registry baseline.

Any implementation work in these areas must be covered by a dedicated plan,
relevant ADR review and repository validation.
```

### `docs/domains/persons/data-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/data-model.md`
- Size bytes / Размер в байтах: `10982`
- Included characters / Включено символов: `10980`
- Truncated / Обрезано: `no`

````markdown
# Persons — Persona Data Model

This document defines the target logical model for the persons domain. It does
not claim that the current PostgreSQL schema has already been migrated. Current
tables named `persons` remain compatibility storage until a dedicated migration
ADR exists.

## Model Principles

- Persona is the root entity.
- The Owner Persona is represented by `is_self: true`; there is no separate
  `UserProfile`.
- Identity is a collection of digital traces, not a single email column.
- Relationships are first-class records, not fields on a Persona.
- Memory records are evidence-backed.
- Timeline is a shared engine view over dated records.
- Dossier is a generated read model.
- AI observations are derived, confidence-scored and cited.

## Persona

```yaml
Persona:
  persona_id: string
  is_self: boolean
  persona_type: human | ai_agent | organization_proxy | system
  display_name: string
  lifecycle_status: active | archived | merged

  identity:
    primary_label:
    traces:

  communication:
    preferred_channels:
    patterns:

  memory:
    facts:
    knowledge:
    preferences:
    memory_cards:
    conflicts:

  timeline_view:
    events:

  relationships:
    outgoing:
    incoming:

  dossier_read_model:
    current:

  created_at:
  updated_at:
```

Rules:

- Exactly one Persona may have `is_self = true`.
- `persona_type` is required.
- Email, phone and provider usernames are identities, not root columns.
- Organization membership is a Relationship, not a free-text field.
- Favorites, watchlists and relationship health are UI/read-model concerns, not
  Persona identity.

## PersonaType

| Value | Meaning |
|---|---|
| `human` | A real person represented in memory. |
| `ai_agent` | HESTIA or another local/future AI agent represented in the graph. Registry-backed AI agent Personas use stable Persona IDs and compatibility email/display identities such as `hestia@sh-inc.ru`. |
| `organization_proxy` | An organization-like actor when it must participate as a Persona in relationships. |
| `system` | Local system actor used for provenance and automation attribution. |

## PersonaIdentity

Identity Resolution works over traces:

```yaml
PersonaIdentity:
  identity_id: string
  persona_id: string | null
  trace_type:
    - email
    - phone
    - telegram
    - whatsapp
    - github
    - linkedin
    - document_mention
    - message_participant
    - provider_handle
  value: string
  normalized_value: string
  provider: string
  source_ref: string
  confidence: number
  status: active | outdated | unreachable | blocked | disputed
  first_seen_at:
  last_verified_at:
  metadata:
```

Rules:

- Active exact traces should be unique per trace type and normalized value.
- Ambiguous traces create identity resolution candidates.
- Provider-specific identity must be preserved for replay and audit.
- A trace may exist before it is attached to a Persona; compatibility storage
  now supports unattached traces and explicit later assignment.

## IdentityResolutionCandidate

```yaml
IdentityResolutionCandidate:
  candidate_id: string
  candidate_kind:
    - merge_personas
    - attach_trace
    - split_persona
  left_persona_id:
  right_persona_id:
  identity_id:
  evidence_summary:
  evidence_refs:
  confidence:
  review_state: suggested | user_confirmed | user_rejected
  actor_persona_id:
  generated_at:
  reviewed_at:
```

Rules:

- Ambiguous merge/split decisions require review.
- Confirming a merge must preserve enough evidence to support a later split.
- AI may rank candidates, but it must not silently merge ambiguous Personas.

## Relationship

Relationships are primary domain records:

```yaml
Relationship:
  relationship_id: string
  source_persona_id: string
  target_persona_id: string
  relationship_type: string
  trust_score: number
  strength_score: number
  confidence: number
  source_refs:
  valid_from:
  valid_to:
  status: active | inactive | disputed
  metadata:
  created_at:
  updated_at:
```

Rules:

- `source_persona_id` and `target_persona_id` are required.
- `trust_score` and `strength_score` are relationship attributes, not Persona
  root attributes.
- Relationship types must be explicit and queryable.
- Relationship evidence must point to events, messages, documents or reviewed
  user input.

Example relationship types:

- `knows`
- `collaborates_with`
- `works_with`
- `reports_to`
- `represents`
- `assists`
- `owns`
- `member_of`
- `introduced`

The list above is illustrative; a future implementation should control valid
values through a typed domain registry or migration.

## PersonaMemory

Memory is split into durable, cited record types.

### PersonaFact

```yaml
PersonaFact:
  fact_id: string
  persona_id: string
  fact_type: string
  value: string
  source_refs:
  confidence: number
  last_verified_at:
  valid_from:
  valid_to:
  status: active | superseded | rejected
```

### PersonaKnowledgeItem

```yaml
PersonaKnowledgeItem:
  knowledge_id: string
  persona_id: string
  topic: string
  summary: string
  source_refs:
  confidence: number
  updated_at:
```

### PersonaPreference

```yaml
PersonaPreference:
  preference_id: string
  persona_id: string
  preference_type: string
  value: string
  source_refs:
  confidence: number
  last_verified_at:
```

### PersonaMemoryCard

```yaml
PersonaMemoryCard:
  memory_card_id: string
  persona_id: string
  title: string
  body: string
  importance: 1..10
  source_refs:
  confidence: number
  created_at:
  last_verified_at:
```

### PersonaKnowledgeConflict

```yaml
PersonaKnowledgeConflict:
  conflict_id: string
  persona_id: string
  field: string
  value_a: string
  value_b: string
  source_ref_a: string
  source_ref_b: string
  detected_at:
  resolved_at:
  resolution:
```

## Persona Dated Events

```yaml
PersonaDatedEvent:
  event_id: string
  persona_id: string
  relationship_id:
  event_type: string
  title: string
  description:
  occurred_at:
  source_refs:
  related_entity_refs:
  confidence: number
  metadata:
  created_at:
```

Dated events can describe first interaction, a project collaboration, an
obligation, an introduction, a conflict, a meeting, a document mention or a
system observation. They are not a substitute for Relationship records. The
Timeline Engine turns dated events into timeline views.

## PersonaCommunication

```yaml
PersonaCommunicationPattern:
  pattern_id: string
  persona_id: string
  channel:
  language:
  tone:
  verbosity:
  technical_depth:
  response_pattern:
  active_hours:
  active_days:
  source_refs:
  confidence:
  computed_at:
```

This replaces the old `CommunicationFingerprint` vocabulary. Patterns are
derived observations and may be recomputed from messages.

## PersonaDossier

```yaml
PersonaDossier:
  dossier_id: string
  persona_id: string
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
  source_refs:
  generated_at:
```

Rules:

- Dossier is a read model.
- Dossier fields must cite source memory, relationships, messages, documents or
  graph records.
- AI observations must be labeled as observations, not facts.

## Compatibility Mapping

The current schema contains useful pieces but does not yet match the target
model.

| Current table/field | Target model | Migration note |
|---|---|---|
| `persons` | `Persona` projection | Keep as compatibility until a migration ADR. |
| `persons.email_address` | `PersonaIdentity(trace_type=email)` | Root email is compatibility only. |
| `persons.person_type` | `Persona.persona_type` | Value set must become `human`, `ai_agent`, `organization_proxy`, `system`. |
| `persons.trust_score` | `Relationship.trust_score` | Compatibility cache only. Enrichment writes now materialize suggested Owner Persona -> Persona trust Relationships. |
| `persons.primary_role` | `Relationship.relationship_type` or memory fact | Do not model as Persona field. |
| `persons.organization_reference` | Relationship to organization proxy or organizations domain | Keep only as cached compatibility. |
| `persons.is_favorite` | `PersonaPreference(ui:favorite)` compatibility cache | Not domain identity. Writes now materialize a sourced UI preference. |
| `persons.notes` | `PersonaMemoryCard` | Compatibility cache only. Writes now materialize a sourced memory card. |
| `persons.health_status` | Risk/attention compatibility cache | Not source of truth. `PersonRisk` writes now derive it from unresolved risks. |
| `persons.watchlist` | `PersonaPreference(ui:watchlist)` compatibility cache | Not domain identity. Writes now materialize a sourced UI preference. |
| `person_identities` | `PersonaIdentity` | Compatibility schema supports account handles, `document_mention`, `message_participant`, `disputed` status and unattached trace assignment; Persona-native naming and review UI/API remain future work. |
| `person_identity_candidates` | `IdentityResolutionCandidate` | Rename semantics from person/contact to Persona. |
| `person_roles` | `Relationship` | Deprecated in target model. |
| `person_personas` | `PersonaPreference` interaction context compatibility | Deprecated as a nested Persona concept. Compatibility writes now materialize `interaction_context:*` preferences with source references. |
| `person_facts` | `PersonaFact` | Keep concept; rename when schema migrates. |
| `person_memory_cards` | `PersonaMemoryCard` | Keep concept; ensure evidence-backed semantics. |
| `person_preferences` | `PersonaPreference` | Keep concept. |
| `person_snapshots` | Persona read-model snapshots | Keep only if used for diff/replay. |
| `person_knowledge_conflicts` | `PersonaKnowledgeConflict` | Keep concept. |
| `relationship_events` | dated events consumed by Timeline Engine | Split from first-class Relationship records. |
| `enrichment_results` | Persona Intelligence observation candidates | Must be reviewed/cited. |
| `person_expertise` | Persona skills/knowledge signals | Keep concept. |
| `person_promises` | Obligation, commitment event or fact | Do not treat as CRM task tracking. |
| `person_risks` | Evidence-backed AI/user observations | Avoid uncited risk labels. |

## Required Additions

Future implementation work needs explicit storage for:

- Owner Persona uniqueness (`is_self = true`).
- Persona type enum values.
- First-class Relationship records with `source_persona_id`,
  `target_persona_id`, `trust_score` and `strength_score`.
- Persona Dossier cache/read model with source references.
- Persona Intelligence observations with observation type, confidence and
  evidence.
- Digital traces from documents and messages, not only account handles.

## Required Removals From Domain Semantics

These concepts must not appear as target domain primitives:

- Contact.
- Address book.
- CRM profile.
- Contact role.
- Nested contact/person personas.
- Favorite/watchlist as identity.
- Relationship stored only as Persona fields.
- Trust score stored only on Persona.
- Email as required root identity.
````

### `docs/domains/persons/spec.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/spec.md`
- Size bytes / Размер в байтах: `1277`
- Included characters / Включено символов: `1277`
- Truncated / Обрезано: `no`

```markdown
# Persons Domain

## Responsibilities

The persons domain owns Persona Intelligence: digital representations of
subjects, their identities, relationships, memory anchors and dossier views.

Макошь does not treat people as contacts. Personas are durable memory anchors in
the knowledge graph.

## Persona View

A Persona view should show:

- canonical display label;
- known digital traces;
- identity resolution state;
- relationship graph neighborhood;
- relationship trust and strength signals;
- communication patterns;
- memory facts;
- knowledge items;
- preferences;
- related organizations;
- related projects;
- related documents;
- related tasks;
- timeline view from the Timeline Engine;
- generated dossier with source citations.

## Relationship Types

Relationships are first-class records between Personas. Examples include:

- knows;
- collaborates with;
- works with;
- reports to;
- represents;
- assists;
- owns;
- member of;
- introduced.

Relationship records carry provenance, confidence, trust score and strength
score.

## Merge and Split

Identity resolution must support:

- digital trace candidate detection;
- manual merge;
- manual split;
- provider-specific identity preservation;
- audit trail.

Ambiguous Personas must not be silently collapsed.
```

### `docs/domains/persons/status.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/persons/status.md`
- Size bytes / Размер в байтах: `6659`
- Included characters / Включено символов: `6657`
- Truncated / Обрезано: `no`

```markdown
# Persons — Persona Refactoring Status

This status document tracks the documentation and implementation migration from
the legacy Contact/Person model to the target Persona Intelligence model.

It intentionally does not preserve the old "implemented sections" scorecard,
because that scorecard measured a CRM-shaped specification that is no longer the
domain target.

## Documentation Status

| Area | Status | Notes |
|---|---|---|
| Domain vision | Updated | Persona Intelligence replaces Contact/CRM framing. |
| Architecture | Updated | Persona, Relationship, Memory, Dossier, Self Persona and Timeline Engine use are defined. |
| Data model | Updated | Target logical model documented with compatibility mapping. |
| API | Updated | Target `/personas` API shape documented; `/persons` marked legacy compatibility. |
| Gap analysis | Updated | Cross-domain gaps now tracked in `docs/refactoring/implementation-alignment-plan.md` and the root `canonical-evidence-final-report.md`. |
| ADR | Added | ADR-0084 records the domain decision. |

## Current Implementation Compatibility

The backend currently contains implementation pieces that can be reused, but they
do not yet fully implement the target model.

| Current artifact | Status against target |
|---|---|
| `persons` table | Transitional Persona projection, still rooted in email/contact history. |
| `person_identities` | Useful identity trace table; compatibility schema, API and UI now support account handles, `document_mention`, `message_participant`, `disputed` status and unattached trace create/list/attach workflow. |
| `person_identity_candidates` | Compatible review workflow; contact/person language must become Persona language. |
| `person_roles` | Deprecated as standalone role storage; compatibility writes materialize first-class Relationships. |
| `person_personas` | Deprecated as nested Personas; compatibility writes materialize `interaction_context:*` Persona Preferences. |
| `relationship_events` | Useful timeline projection; not a first-class Relationship model. |
| `person_facts`, `person_memory_cards`, `person_preferences` | Compatible with Persona Memory after naming/provenance alignment. |
| `person_expertise` | Compatible as Persona skills/knowledge signals. |
| `person_promises`, `person_risks` | Must be reframed as cited facts, timeline events or observations. |
| `trust_score` | Compatibility cache; enrichment now materializes suggested Owner Persona trust Relationships. |
| `notes` | Compatibility cache; writes now materialize sourced Persona Memory Cards. |
| `is_favorite` | Compatibility cache; writes now materialize sourced `ui:favorite` Persona Preferences. |
| `watchlist` | Compatibility cache; writes now materialize sourced `ui:watchlist` Persona Preferences. |
| `health_status` | Compatibility cache; `PersonRisk` report/resolve now derives it from unresolved risk observations. |
| `/api/v1/persons/owner` | Compatibility API for reading and assigning the single Owner Persona. |
| `/api/v1/persons/*` | Legacy compatibility API until a Persona-native route strategy is accepted. |
| `/api/v1/personas/*` | Persona-native compatibility API bridge over the existing transitional projection; list/detail plus narrow update for display name and Owner Persona assignment. |

## Target Migration Slices

| Slice | Status | Required outcome |
|---|---|---|
| ADR and docs | Complete in this refactoring | New source of truth for domain language. |
| Self Persona | Backend/UI baseline | Compatibility storage enforces one `is_self = true` Owner Persona, exposes GET/PUT `/api/v1/persons/owner`, AI run records store Owner Persona attribution, and the AI workspace loads Owner Persona context for display. Broader cross-domain UI usage remains incremental. |
| PersonaType | Backend/UI baseline | Compatibility storage and projection support `human`, `ai_agent`, `organization_proxy`, `system`; `/api/v1/ai/agents` materializes registry agents as `ai_agent` Personas with `name@sh-inc.ru` compatibility email/display identities and AI run records store agent Persona attribution. |
| Relationship model | Backend/UI baseline | First-class Relationship storage, review state and graph projection exist; Personas workspace and the cross-domain Review shell expose suggested Relationship review. |
| Identity traces | Backend/UI baseline | Compatibility identities now accept handle/email, document mention and message participant traces plus `disputed` status and unattached trace assignment; guarded compatibility API and UI review workflow exist for create/list/attach. |
| Memory model | Partially implemented | Preserve facts, knowledge, preferences, memory cards and conflicts with evidence. |
| Timeline Engine use | Partially implemented | Split dated events from first-class Relationship records. |
| Dossier read/cache model | Backend/API/UI baseline | `PersonInvestigator` now emits generated dossier sections for summary, interests, projects, organizations, skills, communication patterns, AI observations, source refs and `generated_at`; `/api/v1/persons/{person_id}/dossier` persists a reviewable snapshot, `/dossier/review` updates review state, and Persons UI reads/displays the generated dossier. |
| Persona Intelligence | Partially implemented | Consolidate fingerprint/profile/trust/analytics/investigator into one concept. |
| API migration | Backend/frontend bridge baseline | `/api/v1/personas` list/detail routes, narrow `PUT /api/v1/personas/{persona_id}` update route, frontend client types and ADR-0090 exist over the transitional projection; physical schema migration still requires a dedicated ADR and validation plan. |
| Schema migration | Not implemented | Rename/restructure tables only under a dedicated migration ADR and validation plan. |

## Removed Scorecard

The previous status document claimed completion for features such as Contact
Merge, Contact Roles, Contact Personas, Health & Monitoring, Investigator and
Analytics. Those labels are no longer accepted as target-domain milestones.

Replacement milestones:

- Identity Resolution over Persona traces.
- Relationship-first graph model.
- Persona Memory with provenance.
- Persona timeline views through the Timeline Engine.
- Persona Dossier generation.
- Persona Intelligence observations and communication patterns.
- Owner Persona integration for agents, UI context assembly and user-owned actions.

## Validation Expectation

For documentation-only refactoring, validation is scoped to repository file
inspection, Markdown presence checks and scoped diff checks. Backend validation
is required only when implementation or migration code changes.
```

### `docs/domains/projects/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/projects/README.md`
- Size bytes / Размер в байтах: `2635`
- Included characters / Включено символов: `2635`
- Truncated / Обрезано: `no`

```markdown
# Projects Domain

Status: documentation package aligned to the current repository structure.

Projects are bounded work contexts that connect communications, documents,
tasks, decisions, obligations, Personas, Organizations and events.

Макошь is not a project management tool. A Project is a context boundary inside
the Personal Memory System.

## Responsibilities

The Projects domain owns:

- project identity and lifecycle state;
- project goals and scope;
- project context pack;
- links to related entities;
- project-specific decisions;
- project evidence;
- review state for candidate links;
- project timeline view through the Timeline Engine.

The Projects domain does not own:

- Organization identity;
- Task lifecycle;
- Communication source records;
- document versioning;
- global graph traversal;
- global memory.

## Project Versus Organization

An Organization is a durable collective actor. A Project is a bounded work
context.

An organization can sponsor or participate in many projects. A project can
involve many organizations. Neither entity should be modeled as a field of the
other.

## Project Versus Task

A Task is a concrete actionable unit with lifecycle. A Project is a larger
context that may contain many tasks, documents, communications, decisions and
obligations.

Tasks can be linked to Projects, but project state must not be derived only from
task status.

## Project Context Pack

A project context pack should include:

- summary;
- goals;
- current state;
- open obligations;
- active tasks;
- key decisions;
- important documents;
- recent communications;
- involved Personas and Organizations;
- risk and blocker observations;
- source citations.

The context pack is a derived read model, not the source of truth.

## Current Implementation Evidence

Current backend implementation includes:

- `backend/src/domains/projects/core.rs`;
- `backend/src/domains/projects/link_reviews.rs`;
- migrations `0013_create_projects_and_extend_graph.sql` and
  `0014_create_project_link_reviews.sql`;
- `/api/v1/projects/*` route registration;
- Projects frontend page.

The implementation exists, but domain documentation was incomplete compared to
Personas, Organizations and Tasks.

## Migration Plan

1. Make this document the canonical Projects domain description.
2. Expand future architecture/data-model docs if implementation changes require
   deeper detail.
3. Keep candidate links reviewable and provenance-backed.
4. Define Decision and Obligation links before introducing automated project
   health conclusions.
5. Avoid treating Projects as external issue trackers or CRM opportunities.
```

### `docs/domains/relationships/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/relationships/README.md`
- Size bytes / Размер в байтах: `4213`
- Included characters / Включено символов: `4213`
- Truncated / Обрезано: `no`

````markdown
# Relationships Domain

Status: documentation package aligned to the current repository structure.

Relationships are first-class source-of-truth records connecting Макошь world
entities.

Макошь is relationship-first: Personas, Organizations, Projects,
Communications, Documents, Tasks, Events, Decisions, Obligations and Knowledge
items gain meaning from source-backed relationships.

## Responsibilities

The Relationships domain owns:

- durable relationship records;
- source and target entity references;
- relationship type;
- trust score;
- strength score;
- confidence;
- provenance evidence;
- review state;
- validity period;
- relationship metadata.

The Relationships domain does not own:

- graph traversal indexes;
- timeline rendering;
- trust computation;
- risk computation;
- dossier generation;
- automatic contradiction resolution.

Those are engine or projection responsibilities.

## Persona Relationships

Persona-to-Persona relationships are the first implementation path:

```yaml
Relationship:
  source_entity_kind: persona
  source_entity_id:
  target_entity_kind: persona
  target_entity_id:
  relationship_type:
  trust_score:
  strength_score:
  confidence:
  review_state:
```

Examples:

- knows;
- collaborates_with;
- works_with;
- reports_to;
- represents;
- assists;
- owns;
- member_of;
- introduced.

## Evidence

Every durable Relationship must have source evidence:

```yaml
RelationshipEvidence:
  relationship_id:
  source_kind:
  source_id:
  excerpt:
  metadata:
```

Evidence can come from Communications, Documents, Events, Memory, Knowledge,
Decisions, Obligations, Tasks, Projects, Organizations, Personas or raw source
records.

AI may propose a Relationship, but it must remain source-backed and reviewable.

## Relationship Versus Graph Edge

A Relationship is a durable domain record.

A graph edge is a traversal/projection record.

Graph edges may be derived from Relationships, but they are not the only source
of truth for relationship semantics.

## Current Implementation Evidence

Current backend baseline:

- `backend/migrations/0060_create_relationships.sql`;
- `backend/migrations/0061_relationship_graph_projection.sql`;
- `backend/migrations/0068_expand_relationship_graph_node_kinds.sql`;
- `backend/src/domains/relationships/mod.rs`;
- `backend/src/domains/relationships/api.rs`;
- `backend/tests/relationships.rs`;
- `backend/tests/relationships_api.rs`;
- ADR-0086.

This baseline provides first-class Relationship persistence, validation and
graph projection for the current `RelationshipEntityKind` set: Persona,
Organization, Project, Communication, Document, Task, Event, Decision,
Obligation and Knowledge. Guarded backend routes can list Relationships by
entity or by review state and change review state while keeping the graph
projection aligned. The Personas workspace includes a desktop review panel for
global suggested Relationships, while still formatting selected-Persona
relationships compactly when a Persona is selected. It does not yet provide
cross-domain workflow placement or timeline projection. Manual/API
`person_roles` now materialize source-backed `has_role` Relationships from
Persona to role Knowledge anchors, and deletion demotes the same Relationship
to `user_rejected`. Manual/API and email-sync `organization_contact_links` now
have a compatibility adapter that materializes source-backed `member_of`
Relationships from Persona to Organization. Manual `task_relations` now have a
compatibility adapter that materializes source-backed Relationships from Task to
known target entity kinds. Explicit project link reviews now materialize
source-backed Relationships from Project to the reviewed Communication or
Document and demote the relationship candidate back to `suggested` when the
explicit review is reset.

## Migration Direction

1. Keep `relationships` as the durable source-of-truth table.
2. Reclassify remaining relationship-shaped compatibility/read-model surfaces
   behind Relationship records.
3. Feed Relationship records into Trust, Risk, Timeline, Memory and Dossier
   projections.
4. Move or duplicate Relationship review into a broader cross-domain workflow
   inbox after that shell exists.
````

### `docs/domains/review/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/review/README.md`
- Size bytes / Размер в байтах: `1872`
- Included characters / Включено символов: `1872`
- Truncated / Обрезано: `no`

```markdown
# Review Domain

Status: code-aligned documentation package created from ADR-0096 and current
backend modules.

Review is the durable inbox for material that needs owner triage, approval,
dismissal or promotion before it becomes accepted domain truth.

ADR source of truth:

- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)

## Responsibilities

The Review domain owns:

- review inbox items;
- item lifecycle state;
- evidence links from review items to observations;
- target references for promotion results;
- review transition metadata.

It does not own:

- accepted Persona, Organization, Project, Task, Document, Decision,
  Obligation, Relationship or Knowledge truth;
- provider runtime state;
- Radar vocabulary;
- the concrete workflow that materializes promoted entities.

## Current Implementation Evidence

Current backend files:

- `backend/src/domains/review/mod.rs`;
- `backend/src/domains/review/models.rs`;
- `backend/src/domains/review/store.rs`;
- `backend/src/domains/review/service.rs`;
- `backend/src/workflows/review_promotion/mod.rs`.

The domain exports `ReviewInboxStore`, `ReviewInboxService`,
`ReviewItemKind`, `ReviewItemStatus`, `ReviewPromotionTarget` and evidence
records. Current item kinds include identity, project-link, contradiction,
task, obligation, decision, relationship, project and knowledge candidates.

The promotion materialization logic is currently implemented in
`backend/src/workflows/review_promotion`, not inside the Review domain. Review
keeps the inbox and transition state; the target domain owns the accepted
entity after promotion.

## Boundary Rule

Review may record that a candidate was promoted to a target domain/entity. It
must not silently mutate target-domain truth without the target domain command
or workflow boundary.
```

### `docs/domains/signal-hub/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/signal-hub/README.md`
- Size bytes / Размер в байтах: `18090`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Макошь Signal Hub

Status: `IMPLEMENTATION STARTED`, 2026-06-23.

Signal Hub is the system control plane for external and synthetic signal
sources in Макошь. It is not a messenger UI, not an email client and not a
provider-specific integration folder. Signal Hub owns the durable registry of
sources, connections, capabilities, runtime state, health, profiles, mute/pause
policies and recovery fixtures.

Макошь receives signals from the world and turns them into Communications,
Radar, Review, domain objects, Memory and Knowledge. Signal Hub controls the
first boundary of that chain.

```text
External World
  -> Signal Hub
  -> Event Backbone
  -> Communications / Calendar / Documents / Tasks / Knowledge
  -> Radar
  -> Review
  -> Domain Objects
  -> Memory / Knowledge / Projections
```

## Position

Signal Hub is a system domain. It manages all signal sources, not only
communication providers.

Examples:

- Email / Mail;
- Telegram;
- WhatsApp;
- GitHub;
- Browser capture;
- RSS;
- Calendar providers;
- Filesystem watchers;
- Home Assistant;
- voice input;
- deterministic fixture sources.
- system/internal runtime sources;
- local AI runtime sources.

Signal Hub does not own provider protocol code. Provider protocol/runtime code
continues to live under `backend/src/integrations/*`. Signal Hub owns the
source registry and control state used to decide whether a source can publish,
be muted, be paused, replayed, restored or used in tests.

## Core Invariants

- A provider is not a domain.
- A source is not automatically a Communication.
- A signal is evidence from the external or synthetic world.
- Signal Hub controls sources and signal flow.
- No source class is hardcoded as unpausable or unmutable: system, internal and
  AI signals must go through the same policy model as provider signals.
- Event Backbone transports versioned events.
- Communications owns messages, conversations, participants and attachments.
- Radar owns attention and candidate incubation.
- Review owns promotion decisions.
- Domain objects are created by their owning domains.
- AI can suggest candidates, but cannot become source of truth.

## Current Repository Context

The repository already contains:

- append-only `event_log` migration;
- canonical `EventEnvelope` model;
- an in-process broadcast `EventBus`;
- durable event consumers, retry state and DLQ tables;
- Communications as the provider-neutral message domain;
- Telegram and Mail documentation that states channels are integrations, not
  domains.

Signal Hub now has the first backend and Settings UI foundation:

- `backend/src/domains/signal_hub`;
- Signal Hub source registry tables;
- schema-agnostic system source recovery fixture;
- raw signal policy processing for accept, reject, mute and pause;
- runtime state control for core subscriber and scheduler loops through
  `signal_runtime_states`;
- connection metadata/status lifecycle control through `signal_connections`;
- PostgreSQL event outbox foundation;
- NATS JetStream adapter, bootstrap dispatcher with in-memory realtime fan-out
  for published events, and local development service;
- generated ConnectRPC server/client slice for source listing, connection
  get/enable/disable, generic scoped disable/enable, scoped
  mute/unmute/pause/resume, connection listing/create/update/remove, runtime
  state listing/updates, health listing, policy list/create, replay request
  create/list, fixture catalog listing, fixture emission and fixture restore;
- protected local REST compatibility endpoints still exist for compatibility
  and for the remaining not-yet-migrated surfaces;
- root Protobuf contract files plus generated Rust/TypeScript code for the
  implemented service slice;
- Settings UI section for source registry, connections, runtime state, health,
  replay request create/list with pattern and position/time selectors,
  policies and recovery state.

Remaining implementation work includes broader provider producer migration,
accepted-signal Communications consumers beyond the current slices, broader
replay semantics and broader UI/control coverage for the not-yet-migrated
surfaces.

Current migration note: Telegram provider-observation events now enter Макошь as
`signal.raw.telegram.*.observed` and the Communications projection consumes the
accepted Signal Hub family for that slice. These provider-observation raw
events now also use the durable outbox-dispatch path instead of only appending
to `event_log`. Central Mail sync and email fixture
workflows now emit `signal.raw.mail.message.observed` and Communications
projects mail messages only from accepted Signal Hub events through the shared
accepted-signal projection entry point. Mail delivery-status and read-receipt
callbacks now also publish canonical raw Signal Hub events
(`signal.raw.mail.delivery_status.observed` and
`signal.raw.mail.read_receipt.observed`) before the accepted-signal consumer
updates Communications outbox/read-receipt state. WhatsApp fixture ingestion now emits
`signal.raw.whatsapp.message.observed`, and Communications project WhatsApp
messages only from `signal.accepted.whatsapp.message`. The current repository
slice for WhatsApp still exposes fixture ingest plus session/message read
surfaces, but does not yet implement a separate live send/reply/forward runtime
write path. The legacy
`integration.telegram.*` fallback has been removed from the Communications
projection path. Telegram fixture ingestion now also emits
`signal.raw.telegram.message.observed`, and its message projection runs only
from `signal.accepted.telegram.message` before the legacy compatibility UI event
is emitted. The Communications accepted-signal owner now also consumes the base
`signal.accepted.telegram.message` path directly, and workflow/app callers use
the same owner helper instead of calling projection primitives themselves.
Telegram manual send/reply/forward response projection now also
stores the raw record first and re-enters Communications only through accepted
Signal Hub events. Provider breadth outside the implemented
Telegram/Mail/WhatsApp fixture-or-central slice remains part of the outstanding
migration work. Targeted backend regression coverage for Telegram and WhatsApp
message seeding now also follows the same raw -> Signal Hub -> accepted ->
Communications path instead of calling the old direct projection helper. The
old production-facing application shim for provider direct projection has now
been removed, and the legacy direct projection helper has been deleted
entirely. TDLib runtime-created messages, TDLib history/search ingestion and
background Telegram command reconciliation now also persist provider raw
records through a neutral platform raw-record port and publish
`signal.raw.telegram.message.observed` without importing `domains::signal_hub`
from the integration runtime layer.

Current runtime note: the bootstrap-managed loops now register and honor
runtime state for core subscribers/schedulers. Users can pause, mute, stop or
resume these loops through the Settings Signal Hub runtime section, including
the `event_outbox_dispatcher` that controls PostgreSQL-to-JetStream fan-out.
`EnableSource` and `DisableSource` now also orchestrate the durable
`signal_runtime_states` rows for existing source-owned loops, so source-level
runtime control no longer lives only in the separate runtime panel.
Published outbox events now also re-enter the in-memory realtime bus from the
same dispatcher, so websocket clients observe the same persisted `signal.*`
event families that already exist in `event_log` and JetStream.
The synchronous Telegram/Mail/WhatsApp raw-signal helper paths now consult the
same durable runtime state for `signal_hub_raw_signal_dispatcher` before
attempting immediate acceptance, so pausing or stopping that system dispatcher
also blocks direct helper-driven accepted-signal emission and leaves only the
durable raw fact queued for later processing.
The live TDLib-backed Telegram runtime event bridge now also owns a durable
`telegram_runtime_event_bridge` runtime row under the `telegram` source, so
source-level runtime control and the runtime panel can pause live Telegram
subscription event publication before raw Signal Hub append/broadcast.
The synchronous accepted-signal projection helpers used by fixture ingest,
mail sync/fixture flows and Telegram manual-send projection now also consult
the durable runtime state of `communication_provider_observation_projection`
before materializing `communication_messages`, so pausing that consumer no
longer leaves hidden sync app/workflow bypasses around the accepted-signal
owner path.
Lazy runtime rows now also inherit source-level disabled state: if a source is
disabled before a given runtime kind appears for the first time, the first gate
check creates that runtime row in `stopped` instead of silently defaulting it
back to `running`.
That inheritance now follows the same source-control priority everywhere:
`disabled > paused > muted > running`. Source-level pause/mute/unpause/resume
reconcile existing runtime rows to the same state that future lazy runtime rows
will get on first use.
Dedicated
runtime-change SSE events are not complete yet; current UI refresh for these
controls relies on mutation invalidation plus the existing Signal Hub reads.

Current replay note: Signal Hub can now create replay requests through REST
compatibility and ConnectRPC, and the background replay dispatcher executes
queued requests against the paused-event buffer, against raw-signal slices
selected from `event_log` by position/time range, and against one selected
consumer by rewinding that consumer cursor over a matching signal slice.
Replayed raw events re-enter the accepted-signal path with replay provenance.
Connection-scoped replay now also works for raw events that carry `account_id`
and match a Signal Hub connection bound through non-secret
`settings.account_id`. Consumer-targeted replay intentionally preserves
consumer idempotency markers, so it reopens missed or dead-lettered work for a
single consumer without acting as a full projection rebuild. The current
Settings UI now exposes pattern replay, position/time selectors and an
optional target consumer. Replay requests can now also target the first
projection rebuild paths: `timeline_event_log` rewinds the projection cursor,
replays the selected event-log slice through the shared Timeline Engine mapper
and emits `timeline.projection.updated`, while `communication_messages`
clears processed markers for the accepted-signal Communications consumer,
rewinds that consumer cursor over the selected signal slice and emits
`communications.projection.updated`. Signal Hub now also supports first-class
projection-targeted rebuilds for `person_derived_evidence` and
`project_link_review_effects`, each rewinding the matching consumer cursor,
clearing processed markers for the selected event-log slice and emitting
`persons.derived_evidence.updated` or `projects.link_review_effects.updated`.
Broader projection rebuild coverage is still separate work for future targets.

Current fixture note: Signal Hub now has an initial deterministic fixture
catalog and can emit fixture raw signals through REST compatibility and
ConnectRPC. These raw fixture events then flow through the normal
`signal_hub_raw_signal_dispatcher` path into accepted/rejected/muted/paused
Signal Hub outcomes. The current catalog is still intentionally narrow and does
not yet replace the broader provider-specific fixture coverage already present
elsewhere in the repository. The current Settings Signal Hub UI can list the
catalog through ConnectRPC and trigger fixture emission from the `fixture`
source inspector without typing fixture ids manually.

Current connection note: the Settings Signal Hub connection section can now
create, update and remove Signal Hub connection records and switch their
status/profile metadata. Raw-signal policy and replay can now bind a connection
scope through non-secr
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/domains/signal-hub/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/signal-hub/api.md`
- Size bytes / Размер в байтах: `12587`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Signal Hub API

Status: target ConnectRPC API.

Signal Hub APIs are command/query APIs for the local owner and UI. They do not
expose raw provider protocols. They must be implemented as contract-first
Protobuf + ConnectRPC services, with Axum hosting the HTTP transport.

REST endpoints may exist only as temporary compatibility shims during migration.
They must not become the canonical API contract.

## Service

```proto
service SignalHubService {
  rpc ListSources(ListSourcesRequest) returns (ListSourcesResponse);
  rpc GetSource(GetSourceRequest) returns (GetSourceResponse);
  rpc ListCapabilities(ListCapabilitiesRequest) returns (ListCapabilitiesResponse);
  rpc ListFixtureSources(ListFixtureSourcesRequest) returns (ListFixtureSourcesResponse);
  rpc ListConnections(ListConnectionsRequest) returns (ListConnectionsResponse);
  rpc CreateConnection(CreateConnectionRequest) returns (CreateConnectionResponse);
  rpc UpdateConnection(UpdateConnectionRequest) returns (UpdateConnectionResponse);
  rpc RemoveConnection(RemoveConnectionRequest) returns (RemoveConnectionResponse);

  rpc EnableSource(EnableSourceRequest) returns (EnableSourceResponse);
  rpc DisableSource(DisableSourceRequest) returns (DisableSourceResponse);
  rpc DisableSignals(DisableSignalsRequest) returns (DisableSignalsResponse);
  rpc EnableSignals(EnableSignalsRequest) returns (EnableSignalsResponse);
  rpc MuteSignals(MuteSignalsRequest) returns (MuteSignalsResponse);
  rpc UnmuteSignals(UnmuteSignalsRequest) returns (UnmuteSignalsResponse);
  rpc PauseSignals(PauseSignalsRequest) returns (PauseSignalsResponse);
  rpc ResumeSignals(ResumeSignalsRequest) returns (ResumeSignalsResponse);

  rpc ListHealth(ListHealthRequest) returns (ListHealthResponse);
  rpc RunHealthCheck(RunHealthCheckRequest) returns (RunHealthCheckResponse);
  rpc ListRuntimeStates(ListRuntimeStatesRequest) returns (ListRuntimeStatesResponse);
  rpc UpdateRuntimeState(UpdateRuntimeStateRequest) returns (UpdateRuntimeStateResponse);
  rpc ListPolicies(ListPoliciesRequest) returns (ListPoliciesResponse);
  rpc CreatePolicy(CreatePolicyRequest) returns (CreatePolicyResponse);

  rpc ListProfiles(ListProfilesRequest) returns (ListProfilesResponse);
  rpc CreateProfile(CreateProfileRequest) returns (CreateProfileResponse);
  rpc UpdateProfile(UpdateProfileRequest) returns (UpdateProfileResponse);
  rpc RemoveProfile(RemoveProfileRequest) returns (RemoveProfileResponse);
  rpc ApplyProfile(ApplyProfileRequest) returns (ApplyProfileResponse);

  rpc RequestReplay(RequestReplayRequest) returns (RequestReplayResponse);
  rpc ListReplayRequests(ListReplayRequestsRequest) returns (ListReplayRequestsResponse);
  rpc EmitFixtureSignal(EmitFixtureSignalRequest) returns (EmitFixtureSignalResponse);
  rpc RestoreSystemFixture(RestoreSystemFixtureRequest) returns (RestoreSystemFixtureResponse);
}
```

Current implemented contract note: the generated ConnectRPC slice now carries
non-secret `settings_json` for connections, `evidence_json` for health rows,
runtime timestamps/error diagnostics and durable capability rows in addition to
the base control/query surface; profile responses now also carry their policy
definitions so custom profile authoring/editing can stay inside Settings
without dropping back to ad hoc JSON or REST-only shims.

The same root contract set now also has a live provider-neutral
`makosh.communications.v1.CommunicationsService` backend slice for:

- `ListMessages`
- `GetMessage`
- `TransitionMessageWorkflowState`
- `TrashMessage`
- `RestoreMessage`
- `MarkMessageRead`
- `DeleteMessageFromProvider`
- `BulkMessageAction`
- `ToggleMessagePin`
- `ToggleMessageImportant`
- `ToggleMessageMute`
- `SnoozeMessage`
- `AddMessageLabel`
- `RemoveMessageLabel`
- `ListMessageWorkflowStateCounts`
- `RunWorkflowAction`
- `ListSubscriptions`
- `GetMailboxHealth`
- `ListTopSenders`
- `ListCommunicationBlockers`
- `ListCommunicationPersonas`
- `ListRichTemplates`
- `UpsertRichTemplate`
- `DeleteRichTemplate`
- `RenderRichTemplate`
- `PreviewRichTemplateMailMerge`
- `SearchMessages`
- `AnalyzeMessage`
- `GetMessageExplain`
- `GetMessageSmartCc`
- `GetMessageExport`
- `GetMessageAuth`
- `GetMessageSignature`
- `GenerateAiReply`
- `GenerateAiReplyVariants`
- `DetectMessageLanguage`
- `TranslateMessage`
- `ExtractMessageTasks`
- `ExtractMessageNotes`
- `SearchAttachments`
- `GetAttachmentPreview`
- `GetAttachmentArchiveInspection`
- `TranslateAttachment`
- `ListThreads`
- `ListThreadMessages`
- `TranslateThread`
- `ListSavedSearches`
- `CreateSavedSearch`
- `UpdateSavedSearch`
- `DeleteSavedSearch`
- `ListFolders`
- `CreateFolder`
- `UpdateFolder`
- `DeleteFolder`
- `ListFolderMessages`
- `CopyMessageToFolder`
- `MoveMessageToFolder`
- `ListDrafts`
- `CreateDraft`
- `DeleteDraft`
- `ListOutbox`
- `UndoOutboxItem`
- `SendMessage`
- `RedirectMessage`

That communications slice currently reuses existing Communications stores and
confirmed send path under the same router-level `X-Макошь-Secret` boundary; it
does not replace all legacy REST endpoints yet. The frontend now also exposes a
dedicated typed wrapper around this `communications/v1` service for targeted
query/command usage and regression coverage, and the current provider-neutral
frontend query entrypoints for messages, message detail, saved searches,
folders, folder messages, drafts, outbox, threads, thread messages,
attachment search, attachment preview, attachment archive inspection,
attachment translation, message analysis, message explain, smart-cc,
message export, SPF/DKIM auth review, signature detection,
AI reply drafting and reply variants, detect-language, single-message
translation, task extraction, note extraction, workflow-state transition,
workflow-state counts, workflow actions, local trash/restore, mark-read, provider-delete
alias, bulk message action, pin/important/mute, snooze, message labels,
message search, subscriptions, mailbox health, top senders, blockers,
communication personas,
`sendEmail`, `redirectMessage`,
`translateThread`, saved-search CRUD, folder CRUD/message actions, draft
save/delete, rich-template CRUD/render/preview and outbox undo already use this ConnectRPC layer.
The remaining
legacy REST surface is now concentrated in still-unmigrated provider-specific
operations elsewhere in the repository rather than the main Communications UI path.

## Realtime

Realtime is not ConnectRPC streaming in the first browser surface. Browser
updates use Axum SSE by default:

```text
GET /api/v1/events/stream
```

WebSocket delivery is also available through the shared event realtime bus:

```text
GET /api/events/realtime/ws
```

Realtime event families:

```text
signal.source.updated
signal.connection.updated
signal.health.updated
signal.policy.updated
signal.replay.updated
projection.signal_hub.updated
```

The frontend patches generated-client query caches from these event families.
SSE replays directly from durable `event_log`; websocket delivery now also
follows the persisted outbox-dispatch path for published `signal.*` events. The
realtime layer does not replace durable event processing.

## Command Semantics

### EnableSource

Enables source runtime and publication policy.

Current implementation also resumes existing durable `signal_runtime_states`
rows for the same `source_code` back to `running`.

Must emit:

```text
signal.source.enabled
```

### DisableSource

Stops source runtime and prevents capture/publication.

Current implementation also moves existing durable `signal_runtime_states` rows
for the same `source_code` to `stopped`.

Must emit:

```text
signal.source.disabled
```

### DisableSignals

Applies a `disabled` policy to `global`, `source`, `connection` or
`event_pattern` scope without forcing callers through the source-only endpoint.

Current implementation uses the same policy evaluator priority as all other
runtime and publication controls: `disabled > paused > muted > running`.

Must emit:

```text
signal.signals.disabled
```

### EnableSignals

Clears matching scoped `disabled` policies for `global`, `source`,
`connection` or `event_pattern`.

Must emit:

```text
signal.signals.enabled
```

### MuteSignals

Suppresses publication according to scope and pattern. Runtime may stay active.

Current implemented scopes:

```text
global
source
connection
event_pattern
```

`profile` remains a higher-level composition path through `ApplyProfile`, not a
direct command scope in the current control API implementation.

Must emit:

```text
signal.source.muted
signal.policy.changed
```

### PauseSignals

Captures/buffers eligible signals but does not publish them to downstream
consumers until resumed.

Must emit:

```text
signal.source.paused
signal.policy.changed
```

### ResumeSignals

Clears matching scoped `paused` policies and lets eligible runtimes/processors
continue immediately on the next tick or synchronous runtime gate check.

Must emit:

```text
signal.source.resumed
signal.policy.changed
```

### UnmuteSignals

Clears matching scoped `muted` policies and restores publication for the
selected scope.

Must emit:

```text
signal.source.unmuted
signal.policy.changed
```

### UpdateRuntimeState

Applies a durable runtime state override for a concrete `source_code` plus
`runtime_kind` row.

Current implementation supports at least:

```text
running
paused
muted
stopped
```

These runtime rows are consulted live by subscriber loops and synchronous
Signal Hub helper gates; no process restart is required for the new state to
take effect.

### CreatePolicy

Creates a durable policy row directly.

Current implementation supports:

```text
scope: global | source | connection | event_pattern
mode: disabled | paused | muted
```

The Settings UI mostly uses higher-level control commands for toggles, but the
policy create/list path remains part of the canonical contract and is covered
by the generated client.

### RequestReplay

Creates a replay request. Replay consumes from event store / NATS JetStream / fixture
catalog depending on request scope.

Current implementation supports:

```text
paused-buffer replay into accepted-signal flow
event-log replay by pattern / position / time selectors
consumer-targeted replay by rewinding one consumer cursor over the selected signal slice
projection-targeted replay for `timeline_event_log` by rewinding the projection cursor and emitting `timeline.projection.updated`
projection-targeted replay for `communication_messages` by clearing processed markers for the accepted-signal Communications consumer, rewinding its cursor over the selected signal slice and emitting `communications.projection.updated`
```

Must emit:

```text
signal.replay.requested
signal.replay.completed
signal.replay.failed
```

### RestoreSystemFixture

Restores missing system Signal Hub source definitions from the schema-agnostic
fixture. It must never overwrite user-owned connection secrets or provider
runtime sessions.

Must emit:

```text
signal.fixture.restore_requested
signal.fixture.restored
signal.fixture.restore_failed
```

## Query Semantics

Queries read projections where possible:

```text
signal_hub_dashboard_projection
signal_hub_source_projection
signal_hub_health_projection
```

They must not perform expensive cross-domain joins. If a query needs
Communications, Radar or provider runtime details, it should use read-model
composition at app/BFF level, not mutate ownership boundaries.

## Authorization

Initial local API auth remains the repository's local protected API pattern.
Signal Hub commands are owner-local admin commands and should be guarded as
sensitive local operations.

Command classes:

| Command | Action class |
|---|---|
| EnableSource | admin |
| DisableSource | admin |
| DisableSignals | admin |
| EnableSignals | admin |
| MuteSignals | admin |
| UnmuteSignals | admin |
| PauseSignals | admin |
| ResumeSignals | admin |
| UpdateRuntimeState | admin |
| CreatePolicy | admin |
| RequestReplay | admin/export-sensitive when payloa
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/domains/signal-hub/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/signal-hub/architecture.md`
- Size bytes / Размер в байтах: `8660`
- Included characters / Включено символов: `8660`
- Truncated / Обрезано: `no`

````markdown
# Signal Hub Architecture

Status: target architecture, not full implementation.

## Purpose

Signal Hub is the control plane for source activation, source runtime state,
source capabilities, source health, mute/pause policies, replay and deterministic
fixtures.

It exists because Макошь must handle many signal sources without turning every
provider into a separate product domain. Mail, Telegram, WhatsApp, GitHub,
Browser capture, RSS, Calendar, Filesystem and Home Assistant should all enter
Макошь through one governed source boundary.

## High-Level Flow

```text
External Provider / Fixture
  -> Integration Adapter / Fixture Source
  -> observation.captured.v1
  -> signal.raw.<source>.<thing>.observed
  -> signal.accepted.<source>.<thing>
  -> Owning Domain Consumers
  -> Projections
  -> SSE / UI
```

Communication example:

```text
Telegram runtime update
  -> observation.captured.v1
  -> signal.raw.telegram.message.observed
  -> signal.accepted.telegram.message
  -> communication.message.recorded / communication.message.updated
  -> radar.signal.detected
  -> review.item.promoted
  -> task.created / persona.identity_trace.recorded / document.import.requested
```

Calendar example:

```text
Calendar provider update
  -> signal.calendar.event.observed
  -> calendar.event.recorded
  -> timeline.projection.updated
  -> meeting.preparation.requested
```

Filesystem example:

```text
Filesystem watcher event
  -> signal.filesystem.file.observed
  -> document.import.requested
  -> document.processed
  -> knowledge.candidate.detected
```

## Layer Ownership

| Layer | Owns | Must not own |
|---|---|---|
| `domains/signal_hub` | source registry, connections, capabilities, runtime state, health, profiles, mute/pause/replay policy, fixture catalog metadata | provider protocol code, communication messages, tasks, personas, documents |
| `backend/src/integrations/*` | provider protocol, transport, auth/session runtime, provider command execution, raw provider observation capture | Signal Hub policy, Communications state, Radar state, business domain state |
| `platform/events` | event envelope, event store, trace context, trace reconstruction, EventBus abstraction, NATS JetStream transport, consumer cursors, DLQ, replay primitives | business meaning, provider sessions, UI state |
| `domains/communications` | messages, conversations, participants, attachments, drafts, outbox, provider-neutral command state | provider runtime sessions, Signal Hub source policy |
| `domains/radar` | attention signals and review candidates | provider runtime or message storage |
| `workflows/*` | event-driven cross-domain orchestration | direct store mutation outside owner domain |
| `frontend/src/platform` | generated ConnectRPC client setup, SSE bootstrap, shared query plumbing | domain ownership |
| `frontend/src/domains/settings/*` | current user-facing Signal Hub Settings state, queries and composition | provider protocol UI internals or standalone Signal Hub route ownership |
| `frontend/src/integrations/*` | provider setup/runtime panels when needed | business communication workspace |

## Event Backbone

Signal Hub is designed around a versioned event backbone.

Immediate target technologies:

- PostgreSQL append-only `event_log` as audit/recovery source of truth;
- NATS JetStream as production delivery and fan-out transport;
- in-memory EventBus implementation for unit tests;
- Axum SSE for browser realtime updates;
- ConnectRPC + Protobuf for typed command/query API contracts;
- Vue 3 frontend with generated clients and TanStack Query for server state.

The code must depend on traits and contracts, not on transport-specific details:

```text
SignalSource
  -> SignalHubService
  -> EventPublisher trait
  -> EventStore + EventTransport
```

## Event Store And Transport Split

Макошь keeps source-of-truth event history in PostgreSQL and uses NATS JetStream
as the delivery transport.

```text
Domain command / source observation
  -> append EventEnvelope to PostgreSQL event_log
  -> publish same envelope to NATS JetStream subject
  -> durable consumers process through EventConsumerRunner / NATS durable consumer
  -> failures go to DLQ / review state
```

Rationale:

- PostgreSQL is already the primary local store and supports audit/recovery.
- NATS JetStream gives durable live fan-out, replayable delivery and subject
  filtering.
- Consumers must remain idempotent because delivery is at-least-once.
- If NATS publish fails, the event remains in `event_log`; a dispatcher can
  republish from stored positions.

## Canonical Subject Families

Target subject families:

```text
signal.*
communication.*
radar.*
review.*
task.*
persona.*
organization.*
document.*
calendar.*
knowledge.*
projection.*
system.*
```

Provider-specific compatibility families can exist during migration:

```text
integration.telegram.*
integration.mail.*
integration.whatsapp.*
```

New source work should prefer `signal.<source>.*` for source observations and
reserve `integration.<provider>.*` for provider runtime/internal status during
compatibility windows.

## Event Envelope

All cross-boundary events use the canonical envelope:

```text
event_id
event_type
schema_version
occurred_at
recorded_at
source
actor
subject
payload
provenance
causation_id
correlation_id
```

Signal Hub extensions live inside `source`, `subject`, `payload` and
`provenance`; the envelope shape remains stable.

## Trace Contract

Signal Hub participates in the canonical trace graph but does not own product
domain state.

```text
observation.captured.v1
  -> signal.raw.<source>.<thing>.observed
  -> signal.accepted.<source>.<thing>
  -> owning domain event
```

Raw source signals inherit `correlation_id` from the root observation and set
`causation_id` to the observation captured event id. Accepted, rejected, muted
and paused Signal Hub events set `causation_id = raw_event.event_id` and inherit
the raw event correlation id.

## Signal Control Plane

Signal Hub must support these controls from the first implementation slice:

- enable source;
- disable source;
- mute all source events;
- selectively mute by event family/type;
- pause source event publication;
- resume paused source publication;
- replay source events;
- health check;
- fixture mode;
- apply profile.

Control semantics:

| Control | Provider runtime active? | Event captured? | Event published? | Intended use |
|---|---:|---:|---:|---|
| Enabled | yes | yes | yes | normal operation |
| Disabled | no | no | no | source unavailable/off |
| Muted | yes | optional | no | test/debug/maintenance suppression |
| Paused | yes | yes | buffered | maintenance, projection rebuild, deterministic test boundary |
| Replayed | no provider call required | from event store/fixture | yes | recovery and projection rebuild |

## Runtime Model

Signal Hub should stay in the modular monolith initially.

No sidecars are introduced for Mail, Telegram, WhatsApp, Redis or provider
runtimes in the first implementation. Provider runtimes are modules in the same
backend process until a measured reason appears:

- memory isolation;
- crash isolation;
- incompatible native dependencies;
- separate scaling requirements;
- long-running provider runtime that blocks the main process.

The boundary must still make future extraction possible:

```text
InProcessSignalSource
RemoteSignalSource
```

Both implement the same `SignalSource` contract.

## Realtime UI

SSE is the browser realtime delivery path.

```text
Event persisted / projection updated
  -> projection.* event
  -> SSE Gateway
  -> browser EventSource
  -> Vue/TanStack Query cache patch
```

WebSocket hubs are not part of the target Signal Hub architecture.

## ConnectRPC API Boundary

Signal Hub command/query APIs are contract-first.

Canonical API transport:

```text
Protobuf schema
  -> ConnectRPC service
  -> generated Rust / TypeScript clients
```

Axum remains the HTTP host/router. REST compatibility endpoints may exist only
as migration shims and must not define new product contracts.

## Security Boundary

Signal Hub stores only non-secret source and connection metadata.

Allowed:

- provider kind;
- display name;
- capability snapshot;
- health state;
- runtime state;
- settings without secrets;
- `secret_ref`.

Forbidden:

- access tokens;
- refresh tokens;
- cookies;
- TDLib database keys;
- WhatsApp session blobs;
- IMAP passwords;
- OAuth client secrets;
- raw provider payloads containing private message bodies.

Secrets live in the vault/secret resolver boundary. Raw message bodies belong to
provider raw record / Communications evidence storage, not Signal Hub policy
records.
````

### `docs/domains/signal-hub/blockers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/signal-hub/blockers.md`
- Size bytes / Размер в байтах: `2126`
- Included characters / Включено символов: `2126`
- Truncated / Обрезано: `no`

```markdown
# Signal Hub Blockers

Status: active planning blockers.

## Technical Blockers

| Blocker | Required decision |
|---|---|
| NATS dev/test lifecycle | define local/testcontainer NATS JetStream setup without sidecar provider runtimes |
| ConnectRPC Rust wiring | choose exact crate versions and codegen layout for Axum-hosted ConnectRPC |
| Protobuf package layout | decide whether contracts live at repository root or under backend first |
| Signal Hub migration order | create tables before source fixtures; loader must run after schema exists |
| Event transport split | keep PostgreSQL event log as source of truth while NATS handles live delivery |
| Fixture loader timing | decide whether bootstrap runs at backend startup, migration hook or explicit recovery command |
| Projection ownership | decide initial Signal Hub dashboard/read models |

## Architectural Blockers

| Blocker | Risk |
|---|---|
| Provider-specific code still leaking into Communications | Signal Hub controls could inherit provider naming and break neutrality |
| Event family naming | `integration.*` compatibility vs `signal.*` canonical family must be explicit |
| Direct imports | integrations/domains/workflows must not bypass event contracts |
| Sidecar temptation | premature process extraction would increase local dev/test complexity |
| Redis temptation | second event substrate would fragment replay/audit semantics |

## Product Blockers

| Blocker | Required UX decision |
|---|---|
| UI naming | final surface label: `Signal Hub`, likely under Settings or Hub workspace |
| Dangerous controls | disable/mute/pause/replay need confirmation and clear visual state |
| Profile switching | profiles must be obvious because testing profile can suppress real signals |
| Fixture mode | UI must show fixture mode loudly to avoid confusing test data with real data |

## Not Blockers

These are intentionally not blockers for the first implementation:

- extracting provider runtimes into sidecars;
- Redis;
- Kafka;
- WebSocket hub;
- full WhatsApp implementation;
- multi-user or multi-tenant permission model;
- external SaaS deployment.
```

### `docs/domains/signal-hub/data-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/signal-hub/data-model.md`
- Size bytes / Размер в байтах: `5725`
- Included characters / Включено символов: `5725`
- Truncated / Обрезано: `no`

````markdown
# Signal Hub Data Model

Status: target data model.

Signal Hub data is split into durable domain state, technical event state and
schema-agnostic fixture definitions.

## Core Entities

### SignalSource

Canonical source type known to Макошь.

Fields:

```text
id
code
display_name
category
source_kind
default_enabled
supports_connections
supports_runtime
supports_replay
supports_pause
supports_mute
capability_schema_version
created_at
updated_at
```

`settings` is non-secret control metadata. It may carry binding keys such as
`account_id` that let Signal Hub map raw provider signals onto a specific
connection scope for policy and replay decisions. Secrets still belong outside
Signal Hub.

`code` is canonical and stable, for example:

```text
mail
telegram
whatsapp
github
browser
rss
calendar
filesystem
home_assistant
voice
fixture_mail
fixture_telegram
fixture_github
```

`code` is the only value a system recovery fixture may depend on.

### SignalConnection

User-created or system-created connection to a source.

Fields:

```text
id
source_code
display_name
status
profile
settings
secret_ref
connected_at
last_seen_at
last_signal_at
last_sync_at
created_at
updated_at
```

Connection status:

```text
not_configured
connecting
awaiting_user_action
connected
degraded
disconnected
paused
muted
disabled
error
removed
```

### SignalCapability

Capability published by a source or runtime.

Fields:

```text
id
source_code
connection_id optional
capability
state
reason
requires_confirmation
action_class
updated_at
```

Capability examples:

```text
messages.read
messages.send
attachments.read
attachments.write
contacts.read
calendar.events.read
calendar.events.write
files.observe
browser.capture
voice.transcribe
runtime.health_check
runtime.replay
runtime.pause
runtime.mute
```

Capability state:

```text
available
degraded
blocked
unsupported
unknown
```

Action classes:

```text
read
write
destructive
admin
recording
export
secret-bearing
```

### SignalRuntime

Current runtime state for a source/connection.

Fields:

```text
id
source_code
connection_id optional
runtime_kind
state
last_started_at
last_stopped_at
last_heartbeat_at
last_error_at
last_error_code
last_error_message_redacted
metadata
updated_at
```

Runtime state:

```text
stopped
starting
running
reconnecting
paused
muted
stopping
error
```

### SignalHealth

Health projection for UI and diagnostics.

Fields:

```text
id
source_code
connection_id optional
level
summary
last_ok_at
last_failure_at
failure_count
consecutive_failure_count
next_retry_at
evidence
updated_at
```

Health levels:

```text
healthy
degraded
failing
disabled
unknown
```

### SignalPolicy

Control-plane policy for mute/pause/filter/replay behavior.

Fields:

```text
id
scope
source_code optional
connection_id optional
event_pattern
mode
reason
created_by
created_at
expires_at optional
metadata
```

Scopes:

```text
global
source
connection
event_pattern
profile
```

Modes:

```text
enabled
disabled
muted
paused
replay_only
fixture_only
```

Examples:

```text
global muted during test
telegram muted
telegram.message.* muted
mail paused
fixture_* enabled for testing profile
```

### SignalProfile

Named policy bundle.

Fields:

```text
id
code
display_name
description
source_policies
is_system
created_at
updated_at
```

Profiles:

```text
production
development
testing
maintenance
```

### SignalReplayRequest

Replay command and audit record.

Fields:

```text
id
source_code optional
connection_id optional
event_type_pattern optional
from_position optional
to_position optional
from_time optional
to_time optional
target_consumer optional
state
requested_by
requested_at
started_at
finished_at
error_message_redacted
```

States:

```text
requested
running
completed
failed
cancelled
```

## Event Model

Signal Hub events:

```text
signal.source.registered
signal.source.enabled
signal.source.disabled
signal.source.muted
signal.source.unmuted
signal.source.paused
signal.source.resumed
signal.source.health_changed
signal.connection.created
signal.connection.updated
signal.connection.removed
signal.capability.changed
signal.profile.applied
signal.replay.requested
signal.replay.completed
signal.replay.failed
```

Source observation events:

```text
signal.mail.message.observed
signal.telegram.message.observed
signal.whatsapp.message.observed
signal.github.issue.observed
signal.browser.page.observed
signal.filesystem.file.observed
signal.calendar.event.observed
```

## Storage Principles

- Signal Hub source-of-truth state is relational PostgreSQL domain state.
- Cross-boundary facts are appended to `event_log`.
- Delivery uses NATS JetStream subjects.
- UI reads projections, not raw domain tables.
- Secrets are referenced by `secret_ref` only.
- Raw provider payloads are not stored in Signal Hub tables.
- System recovery fixtures never contain row IDs, UUIDs, FK values or direct
  table references.

## Suggested Tables

```text
signal_sources
signal_connections
signal_capabilities
signal_runtimes
signal_health
signal_policies
signal_profiles
signal_replay_requests
signal_fixture_sources
```

Projection tables can be separate:

```text
signal_hub_dashboard_projection
signal_hub_source_projection
signal_hub_health_projection
```

## Idempotency

All source observations must carry idempotency material in the event `source`
object:

```json
{
  "kind": "signal_source",
  "source": "telegram",
  "connection_code": "personal_telegram",
  "source_id": "provider-native-event-id"
}
```

If a provider cannot provide a stable source ID, the adapter must derive one
from source code, provider account, timestamp bucket and stable payload hash.
The derived hash must not include secrets or full raw private bodies.
````

### `docs/domains/signal-hub/fixtures.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/domains/signal-hub/fixtures.md`
- Size bytes / Размер в байтах: `6169`
- Included characters / Включено символов: `6139`
- Truncated / Обрезано: `no`

````markdown
# Signal Hub Fixtures And Recovery

Status: target fixture and recovery contract with initial implementation.

Signal Hub has two kinds of fixtures:

1. system recovery fixtures;
2. test signal fixtures.

They solve different problems and must not be mixed.

Current implementation note:

- the system recovery fixture is implemented and loaded idempotently from
  `backend/fixtures/signal_hub/system_sources.toml`;
- the initial test signal fixture catalog is implemented at
  `backend/fixtures/signal_hub/test_signals.toml`;
- fixture raw signals can now be emitted through Signal Hub REST/ConnectRPC and
  then flow through the normal `signal_hub_raw_signal_dispatcher` consumer
  path;
- the current test fixture catalog is intentionally narrow and does not yet
  replace the broader provider-specific fixture coverage already present in the
  repository.

## System Recovery Fixture

The system recovery fixture defines the canonical built-in Signal Hub sources
that must exist for Макошь to operate.

It is used for:

- first boot bootstrap;
- migration repair;
- recovery after accidental user deletion of system records;
- consistency checks after schema evolution.

## Hard Rules

The recovery fixture must contain no database references.

Forbidden:

```text
UUID
FK
row id
secret_ref
provider account id
graph id
communication id
task id
document id
```

Allowed:

```text
canonical source code
canonical capability code
canonical profile code
category strings
default booleans
display names
non-secret default settings
```

Reason: the fixture may be loaded during migrations or repair flows when the DB
schema has evolved. Fixed IDs and FK references become lies with confidence,
which is the most annoying kind of lie.

## Suggested Location

```text
backend/src/domains/signal_hub/fixtures/system.toml
```

or, if shared with tooling:

```text
backend/fixtures/signal_hub/system.toml
```

## Example Recovery Fixture

```toml
schema_version = 1

[[sources]]
code = "mail"
display_name = "Mail"
category = "communication"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "messages.read",
  "messages.send",
  "attachments.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "telegram"
display_name = "Telegram"
category = "communication"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "messages.read",
  "messages.send",
  "attachments.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "whatsapp"
display_name = "WhatsApp"
category = "communication"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "messages.read",
  "messages.send",
  "attachments.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "github"
display_name = "GitHub"
category = "development"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "issues.read",
  "pull_requests.read",
  "repositories.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "browser"
display_name = "Browser"
category = "capture"
default_enabled = false
supports_connections = false
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "browser.capture",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "filesystem"
display_name = "Filesystem"
category = "documents"
default_enabled = false
supports_connections = false
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "files.observe",
  "documents.import",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "fixture"
display_name = "Fixture Sources"
category = "test"
default_enabled = true
supports_connections = false
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "fixture.emit",
  "runtime.health_check",
  "runtime.replay",
]

[[profiles]]
code = "production"
display_name = "Production"

[[profiles]]
code = "development"
display_name = "Development"

[[profiles]]
code = "testing"
display_name = "Testing"

[[profiles]]
code = "maintenance"
display_name = "Maintenance"
```

## Recovery Loader Semantics

The loader must be idempotent.

```text
for each source in fixture:
  if signal_sources.code exists:
    patch missing non-user-owned metadata only
  else:
    create source from current schema mapping

for each capability in source.capabilities:
  if capability exists for source:
    skip
  else:
    create capability definition using current schema mapping
```

Loader must not:

- overwrite user-created connections;
- overwrite secret references;
- overwrite provider runtime sessions;
- delete sources not present in the fixture;
- assume numeric IDs;
- assume current migration shape beyond the loader's own code.

## Test Signal Fixtures

Test fixtures generate deterministic source observations.

Suggested location:

```text
tests/fixtures/signal_hub/
├── telegram_basic.toml
├── mail_basic.toml
├── whatsapp_basic.toml
├── github_issue.toml
└── browser_capture.toml
```

Example:

```toml
schema_version = 1
fixture_id = "telegram_basic_message"
source = "telegram"
event_type = "signal.telegram.message.observed"
source_id = "fixture-telegram-message-001"
occurred_at = "2026-01-01T00:00:00Z"

[payload]
conversation_key = "fixture-chat-1"
message_key = "fixture-message-1"
sender_display_name = "Fixture Sender"
text = "Test message"
```

Test fixture payloads may contain representative text, but must not contain real
private data.

## Fixture Mode

Testing profile should default to:

```text
real sources disabled or muted
fixture source enabled
```

Макошь downstream domains should process fixture events exactly like normal
source events.
````
