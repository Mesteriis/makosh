# Макошь Product Master Spec

## Status

This is the product-level source of truth for active Макошь documentation.

It describes the target product model and the current implementation baseline at
the same time. When these differ, the target model governs future product
direction, while the implementation baseline tells developers what actually
exists today.

This document does not define API routes, database migrations or runtime
implementation details.

## Canonical Product Definition

Макошь is a local-first Personal Memory System.

Its product experience is a personal operating surface for:

- Communications;
- Knowledge;
- Memory;
- Relationships;
- Projects;
- Documents;
- Decisions;
- Obligations;
- Context.

The primary value is context. CRUD screens, inboxes, calendars, task lists and
document viewers are product surfaces, not the product thesis.

## Product Thesis

Макошь turns communications into durable personal memory and actionable context.

The product has two connected layers:

1. provider-specific operational experiences for working with Mail, Telegram,
   WhatsApp, Zulip and other bundled integrations;
2. provider-neutral evidence, memory and context across those channels.

The first layer preserves real provider capabilities. The second prevents
provider concepts from becoming business-domain semantics. Their boundary is
defined by
[ADR-0204](../adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md).

The current product client is desktop Vue/Tauri. Android is a planned
first-party client over the same Core Gateway and contracts; it is not a
separate mobile backend or business model. Platform-specific UI and capability
availability may differ without changing domain ownership.

The core product cycle is:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Макошь should help the owner answer:

- what happened;
- who and what is involved;
- why something matters;
- what evidence supports it;
- what changed compared with previous memory;
- what obligations, decisions or tasks emerged;
- what context is needed before acting.

## What Макошь Is Not

Макошь is not merely a disconnected collection of:

- email and messenger clients without a shared evidence/context layer;
- a CRM;
- an address book;
- a task tracker;
- a calendar app;
- a note-taking app;
- a generic knowledge base;
- an AI chatbot.

Provider operational surfaces are first-class parts of Макошь, but they are not
independent business domains. Durable context and promoted business truth still
belong to the neutral source-backed memory system.

## Communication As Primary Ingestion Spine

Communication is the primary way real-world signals enter Макошь.

Communication includes:

- email;
- Telegram messages;
- WhatsApp messages;
- calls;
- meetings;
- threads and conversations;
- attachments and linked documents;
- replies, delays, silences and follow-ups where they carry meaning.

A Communication is not just an inbox item. It is evidence that can produce
knowledge, relationships, obligations, decisions, tasks and project context.

Provider-specific production behavior stays under channel capability specs. The
current Telegram channel capability matrix is
[Telegram Channel Capability Spec](../integrations/telegram/README.md) and
[Telegram Gap Analysis](../integrations/telegram/gap-analysis.md), governed by
ADR-0091 and ADR-0097.

Communications are primary, but they are not the only source of evidence.
Documents, calendar events, manual owner input, imported files and provider
records can also create durable memory.

## Source Evidence To Memory Flow

Макошь must preserve evidence before extracting meaning.

```text
Provider or local source
  -> Source Record
  -> Canonical Event
  -> Domain Projection
  -> Knowledge / Memory Candidate
  -> Review or Policy Acceptance
  -> Durable Memory
  -> Derived Views and Agent Context
```

Rules:

- raw provider records and local artifacts are evidence;
- canonical events explain change;
- domain records own accepted truth;
- AI output is derived until accepted under domain rules;
- derived views must be rebuildable where practical;
- answers and actions must cite source evidence.

## Domain Model

Макошь domains are not separate applications. They are ownership boundaries
inside one memory system.

| Domain | Product role | Source-of-truth responsibility |
|---|---|---|
| Communications | Main ingestion spine for messages, calls, meetings, participants and attachments. | Canonical interactions and source communication evidence. |
| Personas | Memory anchors for subjects: owner, people, AI agents, system actors and organization proxies. | Persona identity traces, Persona memory anchors and Persona relationships. |
| Organizations | Collective actor memory. | Organization identity, relationships, portals, procedures, playbooks and organization memory. |
| Projects | Bounded work contexts. | Project state, goals, linked context, decisions and project memory. |
| Documents | Evidence artifacts. | Document versions, extracted content, metadata and document evidence. |
| Knowledge | Evidence-backed understanding. | Reviewed facts, observations and knowledge items with provenance. |
| Decisions | Durable choices. | Rationale, evidence and affected entities for decisions. |
| Obligations | Commitments and duties. | Evidence-backed commitments, expected actions and follow-up state. |
| Tasks | Executable work. | Action lifecycle, task status, task evidence and provider overlays. |
| Events | Things that happened or are scheduled. | Append-only event facts and scheduled event records. |
| Relationships | First-class links between entities. | Typed, source-backed connections with confidence and validity. |

Boundary rule:

```text
Domains own durable truth.
Engines produce derived intelligence.
Agents operate over context.
```

## Engine Model

Engines are shared mechanisms. They do not own domain entities.

| Engine | Purpose | Output type |
|---|---|---|
| Memory Engine | Assemble durable, source-backed memory across domains. | memory views, context summaries, memory gaps |
| Timeline Engine | Build chronological views across entities. | timeline views, diffs, period summaries |
| Trust Engine | Assess relationship and source reliability. | trust signals, confidence adjustments |
| Search Engine | Retrieve source-backed context. | ranked results, snippets, retrieval plans |
| Enrichment Engine | Propose additional knowledge from approved sources. | candidates, observations, conflicts |
| Obligation Engine | Detect commitments, duties and follow-ups. | obligations, task candidates, follow-up candidates |
| Risk Engine | Detect evidence-backed risks and attention signals. | risk observations, attention views |
| Consistency / Contradiction Engine | Detect conflicts between new evidence and accepted memory. | contradiction observations and review items |

### Consistency / Contradiction Engine

The user-facing alias for this engine is Polygraph.

The engine compares new evidence against accepted memory. It detects
contradictions, stale facts, disputed claims, conflicting decisions and
mismatched obligations.

It must not call a person a liar and must not overwrite memory. It creates a
source-backed observation for review.

Example:

```text
New email: "We never approved budget X."
Existing Decision: "Budget X approved on 2026-05-14."
Output: ContradictionObservation linked to Decision, Communication, Project and Personas.
```

Required observation fields:

- old source;
- new source;
- affected entities;
- conflict type;
- confidence;
- review state.

## Current Implementation Inventory

This inventory is based on current repository files.

### Backend Domains And Modules

The backend currently has domain modules for:

- calendar;
- communications;
- decisions;
- documents;
- graph;
- obligations;
- organizations;
- personas;
- projects;
- relationships;
- review;
- signal_hub;
- tasks.

The backend also exports `domains/settings`, but its current module file is
empty. Working application settings logic lives under `platform/settings`.

The backend also has AI, engines, integrations, platform and workflow modules.

Notable integrations:

- Mail;
- Ollama;
- Omniroute;
- Telegram;
- WhatsApp;
- Zoom.

Platform support exists for:

- event log;
- audit log;
- capabilities;
- calls and transcripts;
- observations;
- secrets;
- settings;
- storage;
- host vault.

### Persistence Baseline

Current migrations include storage for:

- event log and projection cursors;
- communication provider accounts, raw records and canonical messages;
- mail blob and attachment metadata;
- documents and document processing jobs;
- graph nodes, edges and evidence;
- first-class relationships and relationship evidence;
- first-class decisions, decision evidence and impacted entity links;
- first-class obligations, obligation evidence and task links;
- projects and project link reviews;
- task candidates and tasks;
- personas storage, persona memory tables, and Persona-native identifier columns;
- organizations and organization memory/workflow tables;
- calendar accounts, events, meetings, deadlines, focus blocks and rules;
- Telegram accounts, chats, messages, policies, calls and transcripts;
- WhatsApp Web sessions and messages;
- application settings, secret references, encrypted vault entries and host vault support;
- AI runtime, semantic embeddings and AI control center tables.

### API Surface Baseline

Routes are currently registered centrally in `backend/src/app/router.rs`.

Implemented route groups include:

- `/api/v1/communications/*`;
- `/api/v1/graph/*`;
- `/api/v1/projects/*`;
- `/api/v1/documents/*` and `/api/v1/document-processing/*`;
- `/api/v1/personas/*`;
- `/api/v1/calendar/*`;
- `/api/v1/organizations/*`;
- `/api/v1/tasks/*` and `/api/v1/task-candidates/*`;
- `/api/v1/settings/*`;
- `/api/v1/ai/*`;
- `/api/v1/integrations/telegram/*`;
- `/api/v1/integrations/whatsapp/*`;
- `/api/v1/policies/*`;
- `/api/v1/calls/*`;
- `/api/v1/integrations/mail/accounts/*`;
- `/api/v1/events/*` and `/api/v1/audit/events`.

This route list is implementation evidence only. It is not the target product
model.

### Frontend Surface Baseline

The frontend currently has page surfaces for:

- Agents;
- Calendar;
- Communications;
- Documents;
- Home;
- Knowledge;
- Notes;
- Organizations;
- Personas;
- Projects;
- Settings;
- Tasks;
- Telegram;
- Timeline;
- WhatsApp.

Some legacy documents still use compatibility names such as `person_id`, Notes,
health or watchtower. Those names must be interpreted through the foundation
glossary and future product roadmap.

## Target Gaps And Refactoring Direction

The current implementation is meaningful but not yet fully aligned with the
target product model.

| Gap | Current evidence | Direction |
|---|---|---|
| Persona-native model incomplete | Persona storage/module naming is now native, and `/api/v1/personas/*` is the active API, while `person_id`, `person_roles`, `person_promises` and legacy event aliases remain compatibility surfaces. Owner Persona, PersonaType, Persona-native read/write compatibility bridge per ADR-0090, role-to-Relationship, interaction-context-to-Preference, enrichment trust-to-Relationship, notes-to-memory-card, favorite-to-preference, watchlist-to-preference, risk-to-health-cache, Dossier section adapters and reviewable Dossier snapshots have baselines. | Keep compatibility explicit and retire it only with API/event/schema migration evidence and replay safety. |
| Owner Persona partially implemented | Migration `0059` adds `is_self` uniqueness and `person_type` constraints on Persona storage, and GET/PUT `/api/v1/personas/owner` exposes the Owner Persona route. Agents and UI still need to consistently route owner-scoped context through that Owner Persona. | Wire agent attribution and context assembly to the Owner Persona before expanding autonomous actions. |
| First-class Relationships partially implemented | Migrations `0060`, `0061` and `0068` plus `backend/src/domains/relationships/` add first-class Relationship persistence with evidence, trust score, strength score, confidence, review state, graph projection for all current Relationship entity kinds, and guarded entity/global review routes. Manual/API Persona roles now materialize source-backed `has_role` Relationships from Persona to role Knowledge anchors and demote those Relationships to `user_rejected` when the role is removed. Manual/API and email-sync Organization-Persona links now materialize source-backed `member_of` Relationships from Persona to Organization. Manual task relations now materialize source-backed Relationships from Task to known target entity kinds. Explicit project link reviews now materialize source-backed Relationships from Project to reviewed Communication or Document and demote the candidate back to `suggested` when explicit review is reset. The Personas workspace and cross-domain Review workspace include suggested Relationship review, and the Review service owns confirm/reject routing for Relationship review items. Downstream engine projections remain incomplete. | Migrate remaining relationship-shaped read-model semantics behind compatibility boundaries and keep review routing in the Review workspace. |
| Polygraph engine partially implemented | ADR-0087, migration `0062`, `backend/src/engines/consistency.rs`, `backend/src/engines/consistency/`, `backend/src/app/handlers/consistency.rs` and `backend/src/application/consistency_review.rs` add structured direct-contradiction detection, deterministic structured and limited natural-language `location` / `status` claim extraction from Communication/Document/Event evidence text, reviewable `ContradictionObservation` persistence and guarded backend review routes. `ContradictionObservationStore::refresh_deterministic_observations` now compares active `persona_facts` Memory claims with claims from projected email message subject/body evidence matched by Persona email sender, projected Telegram/WhatsApp message evidence matched through active channel identities and provider `sender_id`, imported Document title/extracted-text evidence that references the Persona email, meeting-note content linked through event participants and successful call transcript text linked through active Telegram identity. The Knowledge workspace and cross-domain Review workspace include Polygraph review, and the Review service owns confirm/reject routing for contradiction observations. Broad natural-language extraction and broader provider evidence remain incomplete. | Expand ingestion wiring to broader provider evidence, then add reviewed-outcome semantics without automatic memory overwrite. |
| Communications still mail-heavy | Many modules are email-specific under `domains/communications`. | Keep provider-specific modules but document Communications as the product domain and email as one channel. |
| Telegram production capability matrix is not implemented end-to-end | Current Telegram foundation covers account setup, runtime status/start, chat/history sync, manual send, media download facade, policy dry-runs, call metadata and fixture transcripts. ADR-0091 and `docs/integrations/telegram/` define the broader production target for accounts, sessions, proxies, chats, messages, tombstones, history, attachments, calls, offline, export and desktop UX. | Deliver Telegram in gated slices. Do not expose provider-write, destructive, call, export, proxy or session import/export features as available until capability state, storage, audit, UI and validation exist. |
| Engine boundaries are partial | Search, automation, Polygraph and Obligation have baseline engine modules. Memory, Timeline, Trust, Risk and Enrichment remain partly embedded in domain modules. | Continue extracting shared engine behavior only behind dedicated plans and review workflows. |
| Knowledge model incomplete | Knowledge graph exists, but Knowledge as reviewed understanding is not fully documented or implemented as a lifecycle. | Define Knowledge domain spec and review states before implementation work. |
| Decisions and Obligations partially implemented | ADR-0088/ADR-0089 plus migrations `0063`, `0064`, `0065`, `0066` and `0067` add source-backed Obligation and Decision persistence with evidence, review state, links, accepted graph projection and task-candidate classification for obligation-derived candidates. `backend/src/engines/obligation/` adds a deterministic Obligation candidate baseline, `backend/src/domains/decisions/extraction/` adds a deterministic explicit-Decision candidate baseline, message and document task candidate refresh use Obligation detection for explicit commitments/requests, confirmed `obligation_task` candidates materialize source-backed Obligations linked to Tasks, and reset/reject review on those candidates now synchronizes the durable Obligation review state without leaving stale Tasks or links. Email sync and Telegram/WhatsApp fixture ingestion refresh explicit Decision candidates and obligation-derived task candidates for projected Communications without auto-creating Tasks or accepted Obligations. Explicit message/imported-document Decision candidates persist as source-backed `suggested` Decisions, compatibility `person_promises` persist source-backed `user_confirmed` Obligations, meeting `decision` outcomes persist source-backed `suggested` Decisions, project link review decisions persist source-backed `user_confirmed` Decisions, meeting `promise`/`task`/`follow_up` outcomes persist source-backed `suggested` Obligations without creating Tasks, guarded backend routes can list/review accepted Obligations and Decisions by entity or review state, and the Tasks workspace plus cross-domain Review workspace include suggested review panels and shared confirm/reject routing for both. Broader live-provider ingestion and broader candidate-to-domain review workflow coverage remain incomplete. | Connect remaining extraction/review workflows to the domain models without auto-creating Tasks, Projects or Obligations outside explicit review actions. |
| Notes are ambiguous | Frontend has Notes page, while foundation says Notes are document-like artifacts unless a future ADR promotes them. | Treat Notes as document-like capture artifacts until a separate ADR changes scope. |

## Core Workflows

### Incoming Communication To Context

```text
Incoming Communication
  -> preserve source evidence
  -> classify channel, thread and participants
  -> resolve Personas and Organizations
  -> extract claims, facts, preferences, obligations, decisions and risks
  -> check contradictions through the Polygraph engine
  -> link to Projects, Documents, Tasks and prior Memory
  -> update Timeline views and Dossiers
  -> create review items where confidence is insufficient
  -> propose Tasks / Follow-Ups / Decisions
  -> assemble context for owner or agent
```

### Workflow Set

| Workflow | Product output |
|---|---|
| Email to Knowledge | Source-backed knowledge candidates linked to Personas, Organizations, Projects and Documents. |
| Message to Obligation | Obligation candidates and follow-up/task suggestions. |
| Meeting to Decisions | Decisions, obligations, tasks and timeline events from meetings. |
| Document to Context | Document evidence linked to projects, organizations, decisions, risks and tasks. |
| Contradiction Review | Reviewable conflict observations without silent memory overwrite. |
| Dossier Generation | Derived, cited dossiers for Personas, Organizations, Projects or other context anchors. |
| Agent-Assisted Recall | Source-backed answers that distinguish facts, guesses, conflicts and stale memory. |

## Review, Confidence And Provenance

Макошь must distinguish:

- source evidence;
- accepted domain truth;
- inferred candidates;
- AI-generated observations;
- derived read models;
- stale or contradicted memory.

Rules:

- Nothing important becomes durable truth without provenance.
- Nothing uncertain bypasses review.
- Nothing derived silently overwrites memory.
- AI output must cite source evidence.
- Contradictions create review items, not automatic truth replacement.

## Agent Behavior

Agents operate over context. They are not source of truth.

When agents are represented in the world model, they are Personas with
`persona_type = ai_agent`.

Current backend compatibility behavior materializes AI registry agents as
`ai_agent` Personas with stable Persona IDs and `name@sh-inc.ru` compatibility
email identities. The compatibility Persona display name uses the same
email-form identity. Service-created AI run records store both the acting
`agent_persona_id` and the current `owner_persona_id` when an Owner Persona
exists.

Agents must:

- retrieve context from domains and engines;
- distinguish source facts from inference;
- cite evidence;
- respect capability and confirmation policies;
- write auditable actions;
- avoid direct durable mutations without domain rules.

## Documentation Expansion Map

Wave 1 creates the product spine:

- `docs/product/master-spec.md`;
- `docs/product/development-roadmap.md`;
- `docs/README.md`.

Later waves should create or normalize:

- domain specs for Communications, Personas, Relationships, Knowledge,
  Obligations, Tasks, Decisions, Projects, Documents, Organizations and Events;
- engine specs for Memory, Timeline, Trust, Search, Enrichment, Obligation,
  Risk and Consistency / Contradiction;
- workflow specs for communication-to-knowledge, communication-to-obligation,
  meeting-to-decisions, document-to-context, contradiction-review,
  dossier-generation and agent-assisted-recall.
