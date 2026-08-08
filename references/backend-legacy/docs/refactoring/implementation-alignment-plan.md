# Implementation Alignment Plan

This document maps the current repository implementation to the canonical
Макошь product model.

It is a planning document only. It does not authorize code changes, route
renames, schema migrations or API redesign without a follow-up implementation
task and ADR where required.

## Target Model

Canonical references:

- [Product Master Spec](../product/master-spec.md)
- [Domain Catalog](../domains/README.md)
- [Engine Catalog](../engines/README.md)
- [Workflow Catalog](../workflows/README.md)
- [ADR-0084 Persona Intelligence System](../archive/adr/ADR-0084-persona-intelligence-system.md)
- [ADR-0085 Communication Spine and Consistency / Contradiction Engine](../archive/adr/ADR-0085-communication-spine-and-contradiction-engine.md)
- [ADR-0086 First-Class Relationship Persistence](../archive/adr/ADR-0086-first-class-relationship-persistence.md)
- [ADR-0087 Contradiction Observation Persistence](../archive/adr/ADR-0087-contradiction-observation-persistence.md)
- [ADR-0088 Obligation Persistence](../archive/adr/ADR-0088-obligation-persistence.md)

Макошь is a local-first Personal Memory System. Communications are the primary
ingestion spine. Domains own source-of-truth entities. Engines produce derived
views, candidates, scores and review items.

## Current Implementation Evidence

The current backend has these relevant surfaces:

- route registration in `backend/src/app/router.rs`;
- domain modules under `backend/src/domains/`;
- search and automation modules under `backend/src/engines/`;
- workflow modules under `backend/src/workflows/`;
- provider integrations under `backend/src/integrations/`;
- migrations `0001` through `0069`;
- frontend pages under `frontend/src/lib/pages/`.

## Documentation Drift Corrected

This alignment pass corrected documentation that conflicted with current
implementation evidence:

- `docs/integrations/mail/modules.md` now maps to actual files under
  `backend/src/domains/communications/`, `backend/src/workflows/` and
  `backend/src/integrations/`.
- `docs/domains/calendar/architecture.md` now maps to actual files under
  `backend/src/domains/calendar/`.
- `docs/domains/tasks/architecture.md` now maps to actual files under
  `backend/src/domains/tasks/`.
- `docs/domains/organizations/architecture.md` now maps to actual files under
  `backend/src/domains/organizations/`.
- `docs/domains/tasks/api.md` now uses the current router base `/api/v1`, not the stale
  `/api/v2` value.
- Channel/status docs now clarify that implementation coverage percentages are
  local surface coverage, not product-wide completion of Memory, Knowledge,
  Obligations, Decisions or Polygraph.
- `docs/architecture/security-model.md` now follows ADR-0056 and current code:
  `MAKOSH_LOCAL_API_SECRET` plus `X-Макошь-Secret` are current; token/actor-id
  headers are historical terms from superseded ADRs.
- `docs/architecture/context-diagram.md` and
  `docs/architecture/container-diagram.md` now show Макошь as the Personal
  Memory System with Communications, Events, Documents, shared Engines and the
  Owner Persona.
- `docs/reviews/backend-architecture-review-2026-06-06.md` is explicitly marked
  as a historical review rather than the current implementation map.
- Root, backend and frontend README files now distinguish the current host vault
  from legacy database-vault compatibility, describe email networking under
  ADR-0055 read/write capability boundaries, and use Persona-compatible identity
  wording instead of target-level Contact terminology.
- Provider runtime/setup/account-control APIs now live under
  `/api/v1/integrations/*`, while product/business Communications read/write
  APIs remain under `/api/v1/communications/*`.
- `docs/integrations/zoom/` and ADR-0102 now document the Zoom provider foundation. The
  current checkout contains `backend/src/integrations/zoom`,
  `frontend/src/integrations/zoom`, migration `0160` and
  `/api/v1/integrations/zoom/*` routes; ADR-0102 is accepted after the full
  backend/frontend validation gate passed.
- Vault ownership is now constrained to host vault lifecycle, encrypted secret
  payload storage, secret references, resolver contracts and provider session
  secret storage. Domain-owned provider account/account-binding stores live
  under their owning domains instead of `backend/src/vault/`.
- Workflow coordination is being tightened around explicit command/query ports
  rather than concrete store imports, and communications-domain generic
  `Mail*` naming is being retired incrementally in favor of neutral
  `Communication*` symbols.
- `backend/migrations/0059_persona_owner_type_constraints.sql`,
  later Persona rename migrations and `backend/src/domains/personas/api.rs`
  now provide `PersonaType` and single Owner Persona semantics on native
  `personas` storage. ADR-0174 and migration `0202` provide Persona-native
  physical identifiers while bounded historical input aliases remain
  compatibility debt.
- `backend/src/app/router/routes/personas.rs` exposes GET/PUT
  `/api/v1/personas/owner` as the Persona-native compatibility bridge for
  reading and assigning the current Owner Persona; legacy `/api/v1/persons/*`
  routes are retired and covered by regression tests.
- `/api/v1/ai/agents` now materializes registry-backed AI agents (`HESTIA`,
  `MAKOSH`, `MNEMOSYNE`, `ATHENA`, `HEPHAESTUS`) as `persona_type = ai_agent`
  Personas and graph nodes. Compatibility email identities use lowercase agent
  IDs at `sh-inc.ru`, such as `hestia@sh-inc.ru`.
- `ai_agent_runs` now stores `agent_persona_id` and `owner_persona_id`
  attribution for service-created AI runs when an Owner Persona exists.
- `backend/migrations/0071_person_identity_trace_types.sql` extends
  compatibility `person_identities` to accept `document_mention` and
  `message_participant` Persona identity traces.
- `backend/migrations/0072_person_identity_disputed_status.sql` extends
  compatibility `person_identities` to accept `disputed` identity trace status.
- `backend/migrations/0073_person_identity_unattached_traces.sql` and
  `PersonaIdentityStore::create_unattached` / `attach_to_persona` provide the
  first backend workflow for identity traces that exist before Persona
  assignment.
- `/api/v1/identity-traces` now exposes guarded compatibility create/list
  routes for unattached identity traces, and
  `/api/v1/identity-traces/{identity_id}/assignment` attaches a trace to a
  Persona.
- `backend/migrations/0060_create_relationships.sql` and
  `backend/src/domains/relationships/mod.rs` now provide the first durable
  Relationship persistence baseline with evidence, trust score, strength score,
  confidence and review state.
- `backend/migrations/0061_relationship_graph_projection.sql` now connects
  active Persona-to-Persona Relationship records to graph traversal through
  generic `entity_relationship` graph edges.
- `backend/src/domains/relationships/api.rs` now exposes guarded backend routes
  for listing Relationships by entity and changing review state while keeping
  active Persona-to-Persona graph projections aligned.
- `backend/migrations/0062_create_contradiction_observations.sql` and
  `backend/src/engines/consistency.rs` now provide the first
  Consistency / Contradiction Engine baseline: structured direct-contradiction
  detection and reviewable `ContradictionObservation` persistence.
- `ContradictionObservationStore::refresh_deterministic_observations` now adds
  the first Communication/Document/Event-to-Polygraph refresh paths by
  comparing active `persona_facts` Memory claims with structured claims
  extracted from projected Communication message subject/body evidence matched
  by Persona email sender, imported Document title/extracted-text evidence that
  references the Persona email, meeting-note content linked through event
  participants and successful call transcript text linked through active
  Telegram identity.
- `backend/src/app/handlers/consistency.rs` now exposes guarded backend routes
  for listing open contradiction observations and changing review state without
  automatically overwriting Memory.
- `backend/migrations/0063_create_obligations.sql` and
  `backend/src/domains/obligations/mod.rs` now provide the first source-backed
  Obligation persistence baseline with evidence, status, review state, risk
  state, confidence and task links.
- `backend/migrations/0064_create_decisions.sql` and
  `backend/src/domains/decisions/mod.rs` now provide the first source-backed
  Decision persistence baseline with evidence, rationale, alternatives, review
  state, confidence and impacted entity links.
- `backend/src/domains/decisions/api.rs` now exposes guarded backend routes for
  listing accepted Decisions by entity and changing accepted Decision review
  state without changing Projects, Tasks or Obligations.
- `backend/src/engines/obligation/` now provides the first Obligation Engine
  candidate detection baseline for explicit commitment and request language.
- `backend/src/domains/tasks/candidates.rs` now uses the Obligation Engine when
  refreshing message task candidates for explicit commitment/request language
  that the legacy task scanner does not match.
- `backend/migrations/0067_task_candidate_kind_metadata.sql` and
  `backend/src/domains/tasks/candidates.rs` now classify obligation-derived
  task candidates and materialize confirmed `obligation_task` candidates into
  source-backed `user_confirmed` Obligations linked to the created Task through
  `fulfillment_task`. Resetting or rejecting an obligation-derived task
  candidate now synchronizes the durable Obligation review state instead of
  leaving stale `user_confirmed` Obligations behind. Task candidate refresh is
  also idempotent across the source/title identity enforced by the database.
- `backend/src/workflows/email_sync_pipeline.rs` now refreshes explicit
  Decision candidates and obligation-derived task candidates for projected
  email Communication messages in the current sync batch. It creates reviewable
  candidates only and does not auto-create Tasks, Projects or accepted
  Obligations.
- `backend/src/domains/tasks/candidates.rs` now also applies the Obligation
  Engine to explicit document commitments when refreshing document task
  candidates. This creates reviewable `obligation_task` candidates with
  document evidence only and does not auto-create Tasks or accepted Obligations.
- `backend/src/integrations/telegram/client.rs` and
  `backend/src/integrations/whatsapp/client.rs` now refresh explicit Decision
  candidates and obligation-derived task candidates for newly projected fixture
  provider Communications. They create reviewable candidates only and do not
  auto-create Tasks, Projects or accepted Obligations.
- `backend/src/domains/obligations/api.rs` now exposes guarded backend routes
  for listing accepted Obligations by entity and changing accepted Obligation
  review state without creating Tasks.
- `backend/src/domains/decisions/mod.rs` now refreshes explicit Communication
  message and imported Document Decision candidates into source-backed
  `suggested` Decisions and preserves reviewed Decision state across repeat
  refreshes.
- `backend/src/domains/calendar/meetings.rs` now adapts meeting outcomes into
  reviewable domain records: `decision` outcomes create source-backed
  `suggested` Decisions impacted by the meeting Event, and `promise`, `task`
  and `follow_up` outcomes create source-backed `suggested` Obligations without
  creating Tasks.
- `backend/src/domains/personas/trust.rs` now adapts compatibility
  `person_promises` records into source-backed `user_confirmed` Obligations
  with `raw_record` evidence and without creating Tasks.
- `backend/src/domains/projects/link_reviews.rs` now adapts explicit project
  link review decisions into source-backed `user_confirmed` Decisions impacted
  by the Project and reviewed Communication or Document.
- `backend/src/domains/projects/link_reviews.rs` now also adapts explicit
  project link reviews into source-backed Relationships from Project to the
  reviewed Communication or Document. Resetting an explicit project link review
  demotes the durable Relationship candidate back to `suggested` instead of
  leaving stale confirmed/rejected semantics behind.
- `backend/src/domains/organizations/core.rs` and
  `backend/src/workflows/email_sync_pipeline.rs` now adapt manual/API and
  email-sync `organization_persona_links` compatibility records into
  source-backed `member_of` Relationships from Persona to Organization.
- `backend/src/domains/personas/core.rs` now adapts manual/API `person_roles`
  compatibility records into source-backed `has_role` Relationships from
  Persona to role Knowledge anchors, and role removal demotes the same
  Relationship to `user_rejected`.
- `backend/src/domains/personas/core.rs` now also adapts manual/API
  `person_personas` compatibility records into source-backed
  `interaction_context:*` Persona Preferences, and removes those derived
  preferences when the compatibility interaction context is deleted.
- `backend/src/domains/personas/enrichment.rs` now adapts enrichment
  `personas.trust_score` compatibility writes into suggested Owner Persona ->
  Persona `trusts` Relationships while keeping the root column as a temporary
  compatibility cache.
- `backend/src/engines/trust/` now owns the first shared Trust Engine
  baseline for converting deprecated Persona compatibility trust scores into
  source-backed Relationship trust signals.
- `backend/src/engines/trust/` now also builds source reliability signals,
  and `backend/src/domains/personas/enrichment.rs` stores those signals in
  Relationship evidence metadata for review.
- `backend/src/domains/personas/enrichment.rs` now adapts manual/API
  `personas.notes` compatibility writes into sourced Persona Memory Cards while
  keeping the root column as a temporary compatibility cache.
- `backend/src/engines/memory.rs` now owns the first shared Memory Engine
  baseline for converting deprecated Persona compatibility notes into
  source-backed memory-card drafts.
- `backend/src/engines/memory.rs` now also builds source-backed accepted
  Persona fact drafts, and `backend/src/domains/personas/memory.rs` uses those
  drafts before writing compatibility `persona_facts`.
- `backend/src/engines/memory.rs` now also assembles bounded source-backed
  entity context packs from memory-card drafts and accepted fact drafts,
  preserving ordered items, deduplicated source citations, aggregate confidence
  and producing process.
- `backend/src/engines/memory.rs` now also detects required fact gaps for an
  entity as deterministic source-backed `suggested` review candidates without
  creating facts automatically.
- `backend/src/engines/memory.rs` now also emits stale-memory review
  candidates for accepted facts whose verification timestamp is missing or
  older than a caller-provided threshold, preserving source citation and
  confidence without automatically decaying or overwriting the fact.
- `backend/src/domains/personas/enrichment.rs` now adapts manual/API
  `personas.is_favorite` compatibility writes into sourced `ui:favorite`
  Persona Preferences while keeping the root column as a temporary
  compatibility cache.
- `backend/src/engines/enrichment/` now owns the first shared Enrichment
  Engine baseline for converting deprecated Persona compatibility favorite
  state into source-backed preference drafts.
- `backend/src/engines/enrichment/` now also builds source-backed pending
  Persona observation candidates, and `backend/src/domains/personas/enrichment_engine.rs`
  uses those drafts when writing compatibility `enrichment_results` with
  `_enrichment` metadata.
- `backend/src/domains/personas/health.rs` now adapts manual/API
  `personas.watchlist` compatibility writes into sourced `ui:watchlist`
  Persona Preferences while keeping the root column as a temporary
  compatibility cache.
- `backend/src/domains/personas/trust.rs` now adapts `person_risks`
  report/resolve writes into the root `personas.health_status` compatibility
  cache derived from unresolved risk observations.
- `backend/src/engines/risk/` now owns the first shared Risk Engine baseline
  for deriving attention status from unresolved risk severities. The Persona
  `health_status` cache uses this engine while remaining a compatibility
  read-model field.
- `backend/src/engines/risk/` now also builds source-backed Persona risk
  observation drafts, and `backend/src/domains/personas/trust.rs` uses those
  drafts before writing compatibility `person_risks` records.
- `backend/src/domains/personas/investigator.rs` now emits target Persona
  Dossier read-model sections for summary, interests, projects,
  organizations, skills, communication patterns, AI observations, source refs
  and `generated_at` while preserving legacy dossier fields for compatibility.
- `backend/src/domains/tasks/core.rs` now adapts manual `task_relations`
  compatibility records into source-backed Relationships from Task to known
  target entity kinds. Migration `0069` also relaxes the old
  `tasks.task_candidate_id` requirement so standalone manual Tasks match the
  current Tasks domain model.
- `backend/src/engines/timeline.rs` now owns the first shared Timeline Engine
  baseline for bounded source-backed entity timeline policy and source-backed
  period summaries. It also emits source-backed recency signals for individual
  entities, detects source-backed gaps between adjacent entity events and diffs
  source-backed entity timeline snapshots by source reference. The engine also
  assembles bounded cross-domain timelines from source-backed events across
  entity kinds and maps canonical `StoredEventEnvelope` replay batches into
  bounded timeline entries while tracking the last replayed position. It now
  wires that replay mapper into the shared projection runner so canonical events
  are read through `EventStore::list_after_position`, validated, converted into
  derived entries and advanced through `ProjectionCursorStore` cursor progress.
  Persona, Organization and Project timeline producers use the shared policy
  while retaining current compatibility tables.

## Alignment Matrix

| Target area | Current evidence | Gap | Required plan |
|---|---|---|---|
| Communications domain | `/api/v1/communications/*`, `backend/src/domains/communications/*`, Gmail/Telegram/WhatsApp integrations, communication migrations | Public API is communication-shaped, implementation module is still email/mail-shaped. | Communications migration plan before any module rename. |
| Email channel | `docs/integrations/mail/*`, email account routes, mail blob migrations | Email is a channel but still has broad module ownership. | Keep channel docs; do not promote Mail to product domain. |
| Persona Intelligence | `backend/src/domains/personas/*`, `/api/v1/personas/*`, ADR-0084, ADR-0090, ADR-0174, Persona migrations, migration `0059` for `is_self` and `person_type` constraints | Target entity is Persona; active module/table/API naming and the physical identifiers scoped by migration `0202` are Persona-native. Historical event/request aliases and explicitly out-of-scope cross-domain identifiers remain compatibility details. Owner Persona storage, Persona-native owner/list/detail/profile/dossier routes, AI workspace Owner Persona display, AI run Owner Persona attribution, PersonaType, role-to-Relationship, interaction-context-to-Preference, trust-to-Relationship, notes-to-memory-card, favorite-to-preference, watchlist-to-preference, risk-to-health-cache, Dossier section adapters, reviewable Dossier snapshots and Personas UI Dossier display have baselines. Downstream engine projections remain incremental. | Keep compatibility aliases explicit; remove remaining legacy naming only when each event/schema migration has evidence and replay safety. |
| Relationships | `backend/src/domains/relationships/mod.rs`, `backend/src/domains/relationships/api.rs`, migrations `0060`, `0061` and `0068`, graph core, Persona roles, Organization-Persona links, task relations, project link reviews, Personas workspace review panel, Review workspace | First-class Relationship persistence, graph projection for all current `RelationshipEntityKind` endpoints, guarded entity/global review routes, Persona role adapters, Organization-Persona link adapters for manual/API and email-sync paths, manual task relation adapters, project link review adapters, Personas workspace global suggested review, cross-domain Review workspace placement and shared Review action dispatch have a baseline, but downstream engine projections are incomplete. | Migrate remaining relationship-shaped read-model semantics behind compatibility boundaries and keep review routing in the cross-domain workflow shell. |
| Memory Engine | `backend/src/engines/memory.rs`, Persona memory, organization memory, project memory docs | A shared Memory Engine baseline now converts deprecated Persona compatibility notes into source-backed memory-card drafts, normalizes source-backed accepted Persona fact drafts before compatibility `persona_facts` writes, assembles bounded source-backed context packs with source citations, detects required fact gaps as `suggested` review candidates and emits stale-memory review candidates for unverified or outdated facts. Broader review workflow and cross-domain context assembly remain incomplete. | Expand shared Memory Engine behavior after domain source boundaries are stable. |
| Timeline Engine | `backend/src/engines/timeline.rs`, calendar events, person timeline, organization timeline, project timelines, frontend timeline page | A shared Timeline Engine baseline now owns bounded entity timeline limits, source-backed event validation for Persona, Organization and Project compatibility timeline producers, source-backed period summaries, source-backed entity recency signals, source-backed entity gap detection, source-backed entity snapshot diffs, bounded cross-domain timeline assembly, canonical event-log replay mapping and cursor-backed projection-runner wiring from `EventStore::list_after_position` through `ProjectionCursorStore`. Durable read-model storage for projected Timeline views remains incomplete. | Define durable Timeline read-model storage only after a follow-up schema/API decision while keeping Calendar as the scheduled event domain. |
| Trust Engine | `backend/src/engines/trust/`, `personas/trust.rs`, `personas/enrichment.rs`, relationship scores in docs | A shared Trust Engine baseline now converts deprecated Persona compatibility trust scores into Owner Persona -> Persona `trusts` Relationship signals and emits source reliability signals into Relationship evidence metadata. Contradiction input handling, trust review recommendations and cross-domain reconciliation remain incomplete. | Continue normalizing trust as source/relationship signal, not generic entity field. |
| Risk Engine | `backend/src/engines/risk/`, `personas/health.rs`, `watchtower`, risk routes in Personas/orgs/calendar/tasks | A shared Risk Engine baseline now builds source-backed Persona risk observation drafts and derives attention status from unresolved risk severities; Persona risks use it before writing compatibility `person_risks` and updating the Persona `health_status` cache. Cross-domain risk observation routing, review workflow and health/watchtower terminology normalization remain incomplete. | Extend Risk Engine observations/review across domains, then migrate health/watchtower compatibility language behind it. |
| Enrichment Engine | `backend/src/engines/enrichment/`, Persona enrichment, organization enrichment | A shared Enrichment Engine baseline now converts deprecated Persona compatibility favorite state into sourced `ui:favorite` preference drafts and builds source-backed pending Persona observation candidates for compatibility `enrichment_results`. Approved-source policy, conflict routing and broader cross-domain candidate enrichment remain incomplete. | Expand shared enrichment semantics with domain-specific source policies and route conflict candidates to the Consistency / Contradiction Engine. |
| Obligation Engine | `backend/src/engines/obligation/`, `backend/src/domains/obligations/mod.rs`, `backend/src/domains/obligations/api.rs`, migrations `0063`, `0066` and `0067`, task candidates, task rules, email sync and Telegram/WhatsApp fixture communication extraction, document candidate extraction, meeting outcomes, person promises, Tasks workspace review panel, Review workspace | Candidate detection, accepted Obligation persistence, accepted Obligation graph projection, guarded accepted-Obligation backend entity/global review routes, global Tasks workspace review, Review workspace aggregation and action dispatch, obligation-derived task-candidate review-state synchronization, email-sync, document and Telegram/WhatsApp fixture candidate refresh, person promise adapters and meeting `promise`/`task`/`follow_up` outcome adapters have baselines. Live-provider ingestion and broader candidate-to-Obligation review workflow coverage are incomplete. | Extend remaining Communication/document ingestion to the engine and feed reviewed candidates to the Obligations domain without auto-creating Tasks outside explicit task-candidate review. |
| Consistency / Contradiction Engine | `backend/src/engines/consistency.rs`, `backend/src/engines/consistency/`, `backend/src/app/handlers/consistency.rs`, `backend/src/application/consistency_review.rs`, migration `0062`, ADR-0085, ADR-0087, Review workspace | Structured direct-contradiction detection, deterministic structured and limited natural-language `location` / `status` claim extraction from Communication/Document/Event evidence text, observation persistence, guarded backend review routes, Knowledge workspace review panel, Review workspace aggregation/action dispatch and projected email/Telegram/WhatsApp message, imported Document, meeting-note and call-transcript refresh against active `persona_facts` have baselines. Broad natural-language extraction and broader provider evidence are incomplete. | Expand ingestion refresh to broader provider evidence, then add reviewed-outcome semantics without automatic overwrite. |
| Decisions domain | `backend/src/domains/decisions/mod.rs`, `backend/src/domains/decisions/api.rs`, `backend/src/domains/decisions/extraction/`, migrations `0064` and `0065`, email sync and Telegram/WhatsApp fixture candidate refresh, meeting outcomes, project link review decisions, communication/document evidence, Tasks workspace review panel, Review workspace | Accepted Decision persistence, deterministic explicit-Decision candidate extraction, explicit message/imported-document candidate persistence as `suggested` Decisions, email-sync and Telegram/WhatsApp fixture candidate refresh for projected Communication messages, accepted Decision graph projection, guarded accepted-Decision backend entity/global review routes, global Tasks workspace review, Review workspace aggregation/action dispatch, meeting `decision` outcome adapters and project link review adapters have baselines. Live-provider ingestion and broader candidate-to-Decision review routing are incomplete. | Connect remaining communication/document candidates to the Decisions domain without auto-changing Projects, Tasks or Obligations. |
| Agents domain | AI runtime/control center, Ollama/OmniRoute, frontend Agents page | Runtime exists, AI registry agents now materialize as `ai_agent` Personas and graph nodes, and AI run records store agent/Owner Persona attribution. Capability policy, UI context assembly and broader agent workflow context remain incomplete. | Agent capability audit and UI/context attribution plan. |
| Notes boundary | frontend Notes page, documents treat notes as artifacts | No backend Notes domain and no ADR promotes one. | Keep Notes as document-like artifacts until ADR changes boundary. |

## Refactoring Slices

### Slice 1: Communications Naming Boundary

Goal: make the implementation shape explicit without breaking working code.

Work items:

- keep `/api/v1/communications/*` as the product-facing route family;
- document `backend/src/domains/communications/*` as the current email-channel
  implementation;
- identify modules that are email-specific versus communication-generic;
- avoid module renames until tests and migration scope are defined;
- add ADR if route or module naming changes.

Validation for future code work:

- communications API tests;
- email provider networking tests;
- Telegram and WhatsApp tests;
- route snapshot or router smoke test if available.

### Slice 2: Persona Compatibility Boundary

Goal: migrate remaining compatibility language toward Persona without corrupting
historical payloads or explicitly out-of-scope cross-domain identifier contracts.

Work items:

- keep ADR-0084 as the target model;
- keep an explicit inventory of remaining legacy event/request aliases,
  out-of-scope `linked_person_id` / `owner_person_id` fields, retired `/persons`
  routes and compatibility DTOs before removing any alias;
- keep development command docs aligned with the current Makefile surface;
- separate compatibility names from product language in docs and UI labels;
- keep the migration `0059` Owner Persona uniqueness and PersonaType validation
  baseline intact;
- design first-class Relationship storage before removing role/contact-shaped
  fields.

Validation for future code work:

- person API tests;
- identity review tests;
- migration replay tests;
- graph projection tests.

### Slice 3: Relationship Model Consolidation

Goal: make Relationship a shared source-of-truth concept instead of scattered
fields.

Work items:

- inventory graph edges, Persona roles, Organization-Persona links, task relations and
  project link reviews;
- define relationship type taxonomy;
- require provenance, confidence, source evidence, validity period and review
  state;
- map `trust_score` and `strength_score` to relationship semantics;
- keep the migration `0060` and `RelationshipStore` source-of-truth baseline
  intact;
- define graph projection and compatibility adapters for existing tables.

Validation for future code work:

- graph core tests;
- projection replay tests;
- relationship query tests;
- identity split/merge regression tests.

### Slice 4: Engine Boundary Extraction

Goal: separate reusable engines from domain-local intelligence modules.

Work items:

- inventory all `health`, `watchtower`, `intelligence`, `enrichment`, `trust`
  and `memory` modules;
- map each output to Memory, Timeline, Trust, Search, Enrichment, Obligation,
  Risk or Consistency / Contradiction Engine;
- keep domain source records in their owning domains;
- convert engine output to reviewable observations or derived projections.

Validation for future code work:

- existing domain tests;
- engine-specific unit tests;
- projection rebuild tests;
- source-citation tests.

### Slice 5: Decisions And Obligations

Goal: add missing target domains only after candidate/review behavior is clear.

Work items:

- define candidate-first flow for decisions and obligations;
- use Communications, Calendar/Events and Documents as initial evidence sources;
- require owner review or explicit narrow policy before accepted state;
- link accepted obligations to tasks instead of converting every obligation into
  a task;
- link accepted decisions to projects, meetings and documents.

Validation for future code work:

- candidate extraction tests;
- review workflow tests;
- event/audit tests;
- graph link tests.

### Slice 6: Polygraph Implementation

Goal: implement contradiction detection as reviewable observations.

Work items:

- start with communications and documents as evidence sources;
- compare new claims to accepted Memory and Knowledge;
- create `ContradictionObservation` records with old and new source references;
- expose review outcomes without automatic memory overwrite;
- feed reviewed outcomes into Memory, Trust, Risk and Relationship semantics.

Validation for future code work:

- contradiction detection unit tests;
- no-auto-overwrite regression tests;
- source citation tests;
- review outcome tests;
- privacy/audit tests.

### Slice 7: UI Vocabulary Migration

Goal: make the desktop UI express the Personal Memory System model.

Work items:

- keep compatibility route names internal;
- prefer Persona, Communication, Context, Memory, Obligation and Decision in UI
  labels;
- treat Timeline as an engine view;
- treat Health/Watchtower as Risk or Attention views;
- treat Notes as capture/document artifacts;
- add Polygraph review surface only after engine implementation exists.

Validation for future code work:

- frontend translation checks;
- page-level rendering tests;
- accessibility checks for review and confirmation states.

## ADR Requirements

Create ADRs before:

- renaming persisted tables or route families;
- introducing first-class Relationships;
- adding Decisions or Obligations persistence;
- implementing Consistency / Contradiction Engine persistence or routes;
- allowing agent write automation beyond current capability policy.

## Non-Goals

- No code changes in this documentation pass.
- No API redesign in this document.
- No schema migration in this document.
- No removal of historical migrations; legacy route retirement requires
  regression coverage and an explicit API migration note.
- No rewrite of historical ADRs except explicit supersession.
