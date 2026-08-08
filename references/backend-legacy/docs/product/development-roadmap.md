# Макошь Product Development Roadmap

## Status

This roadmap is derived from the Product Master Spec. It is forward-looking and
records target-vs-current gaps discovered from the current repository.

It does not replace historical version checklists under `docs/roadmap/`. Those
files describe past implementation milestones and may contain compatibility
terminology.

## Roadmap Principle

Development should follow the product spine:

```text
Communication
  -> Evidence
  -> Knowledge / Memory
  -> Relationships / Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Each slice should improve Макошь as one Personal Memory System, not as a set of
separate apps.

## Current Implementation Baseline

The repository already contains meaningful implementation slices:

| Area | Current implementation evidence |
|---|---|
| Communications and email | `domains/communications`, communication ingestion/messages migrations, `/api/v1/communications/*`, mail sync, drafts, send/reply/forward, workflow state, analytics, invoices, legal docs, certificates and attachment metadata. |
| Telegram | Telegram integration modules, runtime manager, migrations for chats/messages/policies/calls, `/api/v1/integrations/telegram/*` routes, Telegram frontend page and production capability target in `docs/integrations/telegram/` / ADR-0091. |
| WhatsApp | WhatsApp integration modules, WhatsApp Web sessions/messages migrations, `/api/v1/integrations/whatsapp/*` routes and WhatsApp frontend page. |
| Graph | Graph tables, graph projection module and `/api/v1/graph/*` routes. |
| Documents | Document tables, processing jobs/artifacts, document processing APIs and Documents frontend page. |
| Projects | Project tables, project link review workflow, project APIs and Projects frontend page. |
| Personas | `personas` / `persona_*` storage, identity candidates, memory cards, preferences, timeline events, expertise, risks, promises and `/api/v1/personas/*` routes. Legacy `/api/v1/persons/*` routes are retired. |
| Organizations | Organization tables, identities, departments, linked Personas, memory, risks, finance/enrichment routes and Organizations frontend page. |
| Calendar and events | Calendar account/source/event tables, meetings, outcomes, deadlines, focus blocks, rules and Calendar frontend page. |
| Tasks | Task candidates, tasks, task evidence/context/relations/rules/templates and Tasks frontend page. |
| AI and agents | AI runtime, semantic embeddings, AI control center, AI APIs and Agents frontend page. |
| Settings and security | Application settings, secret references, encrypted vault entries, host vault, audit log and capability decision code. |
| UI operating surface | Frontend pages for home, communications, knowledge, timeline, notes, settings and domain surfaces. |

## Target-State Gaps

| Gap | Why it matters | Refactoring or delivery direction |
|---|---|---|
| Communication is not yet documented as the single ingestion spine across all channels. | Mail, Telegram, WhatsApp, calls and meetings can drift into separate apps. | Create Communications domain spec and normalize channel docs around source evidence and canonical Communication. |
| Telegram production capability model is broader than current implementation. | The requested production surface includes account/session lifecycle, proxies, chat management, message commands, soft delete, message history, attachments, voice/video messages, calls, channels, groups, forums, search, drafts, notifications, address book data, media gallery, offline, export and desktop UX. | Use ADR-0091 and `docs/integrations/telegram/` as the delivery contract; expose every new feature through backend capability states before enabling UI controls. |
| Persona target model is not implemented end-to-end. | Persona storage/module naming is now native, including `persona_metadata`; `/api/v1/personas/*` is the active API and read payloads use Persona-native identifiers. Internal `person_id` storage columns, legacy event aliases and some request compatibility DTOs still carry contact/person history. Owner Persona storage, Persona-native owner routes, ADR-0090 Persona-native read/write compatibility bridge, PersonaType, persona role Relationship adapters, interaction-context Preference adapters, enrichment trust Relationship adapters and notes-to-memory-card adapters have baselines. | Retire remaining compatibility aliases only with explicit API/event/schema migration evidence and replay safety. |
| Relationships are not a complete first-class model. | Roles, organization links, graph edges and relationship events are spread across domains. Durable Relationship records, graph projection for all current Relationship entity kinds, guarded entity/global review routes, manual/API Persona role adapters, manual/API and email-sync Organization-Persona link adapters, manual task relation adapters, project link review adapters, Personas workspace review and cross-domain Review workspace review/action routing have a baseline. | Migrate remaining relationship-shaped compatibility surfaces behind Relationship records and keep review routing in the Review workspace. |
| Polygraph engine is partially implemented. | Structured direct contradictions can be stored as reviewable observations, deterministic structured and limited natural-language `location` / `status` claims can be extracted from Communication/Document/Event evidence text, projected email/Telegram/WhatsApp message refresh, imported Document refresh, meeting-note refresh and call-transcript refresh can compare active `persona_facts` Memory claims against evidence by Persona email identity, active Telegram/WhatsApp identity, event participant link or active Telegram call identity, guarded backend routes can list/review observations without overwriting Memory, and the Knowledge workspace plus cross-domain Review workspace have Polygraph review panels and shared review action routing. Broad natural-language extraction and broader provider evidence remain incomplete. | Expand ingestion refresh to broader provider evidence, then add reviewed-outcome semantics. |
| Decisions and Obligations are partial top-level domains. | Both have source-backed persistence, deterministic candidate detectors where explicit evidence exists, accepted graph projection, guarded backend entity/global list/review routes, a global Tasks workspace review panel and a cross-domain Review workspace panel with shared action routing. Message and document task candidate refresh use Obligation detection for explicit commitments/requests, confirmed `obligation_task` candidates now materialize accepted Obligations linked to Tasks, reset/reject review on those candidates synchronizes the durable Obligation state, email sync and Telegram/WhatsApp fixture ingestion refresh reviewable Decision and obligation-derived task candidates for projected Communications, compatibility `person_promises` now materialize accepted Obligations, explicit message/imported-document Decision candidates now persist as source-backed `suggested` Decisions, project link reviews now materialize accepted Decisions, and meeting outcomes now create reviewable Decisions or Obligations for `decision`, `promise`, `task` and `follow_up` outcomes. Broader live-provider ingestion, candidate routing and follow-ups can still blur together. | Wire remaining candidate extraction and review workflows to accepted Decisions and Obligations, then add adapters from compatibility surfaces. |
| Engine boundaries are not fully separated. | Memory, Timeline, Trust, Risk, Enrichment and Obligation behavior appears inside domain modules. | Write engine specs before extraction or renaming. |
| Notes remain ambiguous. | Frontend has Notes page, but foundation treats Notes as document-like artifacts. | Keep Notes as capture/document artifacts until a future ADR promotes them. |
| Rust-native embedded AI inference is not implemented. | AI Hub can route to Ollama and remote/API providers, and Rust owns local deterministic guards, but small translation/classification/extraction models do not yet run inside the backend process. | Add a future AI Hub ADR for embedded inference through a Rust runtime such as Candle, mistral.rs or llama.cpp bindings, with model cache, download lifecycle, evaluation fixtures and route integration. |
| Documentation tree is incomplete. | Developers cannot yet derive all domain behavior from one product model. | Complete Wave 1 first, then create domain, engine and workflow specs in order. |

## Slice 1: Communication Memory Spine

Goal: make Communications the clear ingestion backbone.

Documentation outcomes:

- `docs/domains/communications/README.md`;
- channel mapping for email, Telegram, WhatsApp, calls and meetings;
- source evidence rules;
- canonical Communication lifecycle;
- current implementation compatibility notes for `domains/communications` and route names.

Implementation plan topics:

- keep provider-specific adapters;
- avoid renaming code until route/schema migration is explicitly planned;
- ensure every communication source can produce evidence, events and graph links.

## Slice 2: Persona And Relationship Memory

Goal: finish the Persona-native baseline and move the remaining physical
`person_id` compatibility debt toward first-class
Relationship memory.

Documentation outcomes:

- `docs/domains/personas/spec.md`;
- `docs/domains/relationships/README.md`;
- Owner Persona behavior;
- PersonaType behavior;
- Persona identity trace lifecycle;
- Relationship trust, strength, provenance and validity rules.

Refactoring plan topics:

- introduce Owner Persona semantics;
- keep legacy `/api/v1/persons/*` routes retired and guarded;
- retire remaining Persona physical/runtime compatibility aliases only through a
  schema/API migration plan;
- preserve existing identity candidate review behavior while changing language.

## Slice 3: Knowledge And Polygraph

Goal: make Макошь able to detect conflicts between new evidence and accepted
memory.

Documentation outcomes:

- `docs/domains/graph/README.md`;
- `docs/engines/consistency/README.md`;
- contradiction review workflow;
- memory conflict taxonomy.

Refactoring plan topics:

- define `ContradictionObservation` as a target concept before schema work;
- connect contradictions to Communications, Documents, Decisions, Obligations,
  Personas, Organizations and Projects;
- ensure contradictions create review items rather than automatic truth changes.

## Slice 4: Obligations, Tasks And Decisions

Goal: separate commitments, executable work and durable choices.

Documentation outcomes:

- `docs/domains/obligations/README.md`;
- `docs/domains/tasks/spec.md`;
- `docs/domains/decisions/README.md`;
- workflow docs for communication-to-obligation and meeting-to-decisions.

Refactoring plan topics:

- separate Obligation from Task and Follow-Up in code and docs;
- map remaining compatibility follow-up cases and task candidates into target
  concepts;
- define how accepted obligations create tasks, follow-ups or risks.

## Slice 5: Projects And Documents Context

Goal: make Projects and Documents the durable context anchors for knowledge
work.

Documentation outcomes:

- `docs/domains/projects/README.md`;
- `docs/domains/documents/README.md`;
- workflow docs for document-to-context;
- project context and document evidence rules.

Refactoring plan topics:

- ensure project memory uses source-backed graph links;
- preserve immutable document evidence before summaries;
- keep Notes as document-like capture artifacts unless an ADR changes scope.

## Slice 6: Agents Over Context

Goal: make agents operate through source-backed context, not private guesses.

Documentation outcomes:

- agent context rules in `docs/ai/agents/`;
- workflow doc for agent-assisted recall;
- alignment with AI control center and capability policy.

Refactoring plan topics:

- represent AI agents as Personas when they enter the graph;
- make agent writes auditable;
- keep AI output derived until accepted by domain rules.
- keep Ollama/API providers as runtime adapters, and plan Rust-native embedded
  models as a separate AI Hub backend for small offline translation,
  categorization and extraction tasks.

## Slice 7: Operating Surface

Goal: present Макошь as a dense desktop personal operating environment.

Documentation outcomes:

- UI information architecture tied to master-spec domains;
- navigation model for Communications, Context, Memory, Projects and Actions;
- compatibility strategy for retired legacy `/api/v1/persons/*` APIs plus current Notes,
  Timeline and Knowledge pages.

Refactoring plan topics:

- align UI labels with foundation vocabulary;
- avoid app-like silos where the product model says shared context;
- use current frontend pages as surfaces over one memory system.

## Documentation Workstream

Recommended order:

1. Product spine: master spec, development roadmap, docs index.
2. Core domain specs: Communications, Personas, Relationships, Knowledge,
   Obligations, Tasks, Decisions.
3. Context domain specs: Projects, Documents, Organizations, Events.
4. Engine specs: Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk,
   Consistency / Contradiction.
5. Workflow specs: communication-to-knowledge, communication-to-obligation,
   meeting-to-decisions, document-to-context, contradiction-review,
   dossier-generation, agent-assisted-recall.

## Refactoring Plan Summary

Create dedicated refactoring plans for:

| Plan | Scope |
|---|---|
| Persona migration plan | Owner Persona, PersonaType, retired `/api/v1/persons/*` compatibility guardrails, target Persona naming and relationship extraction. |
| Relationship model plan | First-class relationship records, graph integration, trust/strength and provenance. |
| Communications normalization plan | Channel-agnostic Communication model over mail, Telegram, WhatsApp, calls and meetings. |
| Telegram production plan | Capability-gated Telegram delivery slices for accounts, sessions, proxies, chats, messages, tombstones, media, calls, offline, export and desktop UX. |
| Polygraph engine plan | Consistency / Contradiction Engine, observations, review workflow and source citations. |
| Obligations and Decisions plan | Durable obligations and decisions separated from tasks, promises, follow-ups and meeting outcomes. |
| Rust-native embedded AI plan | Small local models running inside the Rust backend through AI Hub routes for translation, categorization, extraction and guard refinement; separate from the current Ollama provider lifecycle. |
| Engine boundary plan | Documentation and later implementation extraction for Memory, Timeline, Trust, Risk, Enrichment and Obligation behavior. |
| UI vocabulary plan | Rename or reinterpret current UI surfaces that still expose compatibility language. |

Each plan must inspect the current implementation before proposing changes and
must state whether it is documentation-only, migration-only or implementation
work.
