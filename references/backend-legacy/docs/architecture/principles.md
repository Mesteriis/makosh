# Canonical Architecture Principles

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: architecture principles only. This document does not replace ADR
traceability; conflicting implementation work still requires ADR evolution
before code changes.

## Purpose

This document defines the active principles that guide Макошь architecture.

## Principles

### 1. Personal First

Макошь serves the local owner first. Provider integrations, agents, plugins and
automation exist only inside owner-controlled boundaries.

Responsibilities:

- keep private data local by default;
- make external calls explicit and capability-gated;
- treat the Owner Persona as the center of local context;
- preserve recovery and backup as product concerns.

Boundaries:

- cloud providers are sources or optional integrations;
- remote AI is opt-in and policy controlled;
- multi-user or SaaS assumptions are out of scope until new ADRs say otherwise.

### 2. Memory First

Макошь preserves evidence and memory before optimizing workflow surfaces.

Responsibilities:

- retain source evidence;
- preserve provenance on extracted facts;
- make reviewed memory durable;
- distinguish facts, observations, candidates and derived summaries.

Boundaries:

- AI summaries are not memory until accepted under domain rules;
- search indexes and embeddings are derived;
- provider projections do not replace canonical observations.

### 3. Context First

The product value is context assembly, not CRUD.

Responsibilities:

- link Communications, Personas, Organizations, Projects, Documents, Tasks,
  Decisions, Obligations and Events;
- make Relationships first-class;
- expose timeline, dossier, search and review views as source-backed context;
- explain why a record matters.

Boundaries:

- UI screens are operating surfaces, not separate products;
- standalone provider-client behavior is only justified when it feeds context;
- context views must cite their sources.

### 4. Domain First

Durable truth belongs to bounded contexts with explicit ownership.

Responsibilities:

- name each owning domain for a durable entity;
- keep lifecycle rules inside that owner;
- use events, relationships and application services for cross-domain flows;
- prevent provider or UI modules from owning core domain truth.

Boundaries:

- engines do not own domain entities;
- integrations do not own domain lifecycle;
- frontend domains may mirror product surfaces but do not define backend truth.

### 5. No MVP

Макошь is a long-term local-first Personal Operating System. Thin slices are
allowed; fake product semantics are not.

Responsibilities:

- prefer small correct increments over broad placeholders;
- mark blocked, planned and unsupported capabilities honestly;
- keep unfinished behavior behind explicit status;
- avoid scaffolds that imply durable ownership before the architecture is clear.

Boundaries:

- no fake domains;
- no silent source-of-truth shortcuts;
- no implementation claims without validation.

## Cross-Cutting Rules

| Rule | Meaning |
|---|---|
| Evidence before inference | Imported data and owner actions outrank generated conclusions. |
| Events explain change | Meaningful state changes should be traceable through events or source evidence. |
| AI is derived | AI proposes, summarizes and detects; domains accept or reject. |
| Engines are reusable | Memory, Timeline, Search, Trust, Risk, Enrichment, Obligation and Consistency are shared mechanisms. |
| Providers are channels | Email, Telegram, WhatsApp and calendars are adapters/source boundaries, not product identities. |
| Capabilities gate side effects | Provider writes, destructive actions, exports, recording and secret access require backend authority and audit. |
| Derived state is rebuildable | Indexes, embeddings, graph projections, dossiers, context packs and scores must not be the only copy of memory. |

## Connections

These principles connect the rest of the canonical architecture:

- [Vision](vision.md) defines the product thesis.
- [Domains](domains.md) defines ownership.
- [Communications](communications.md) defines the intake spine.
- [Memory](memory.md) defines evidence-to-context flow.
- [Radar](radar.md) defines the candidate intake layer position.
- [Agents](agents.md) defines permissioned action.
- [UI](ui.md) defines the operating surface.
