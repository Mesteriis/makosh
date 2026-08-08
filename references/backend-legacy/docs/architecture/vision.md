# Canonical Architecture Vision

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: product and architecture direction only. This document does not authorize
code refactoring, API changes, database migrations or provider adapter work.

## Purpose

This document states what Макошь is today at the architecture level.

Макошь is a local-first Personal Operating System built on durable personal
memory and context. Its product surface may include communication workbenches,
project views, tasks, documents, timelines, review queues and agents, but those
surfaces exist to serve one goal:

```text
help the owner remember, understand context and make decisions
```

The shorter product thesis is:

```text
Context + Memory
```

## Non-Identity

Макошь is not merely a disconnected collection of:

- email and messenger clients without shared evidence and context;
- a CRM;
- an address book;
- a task tracker;
- a calendar app;
- a note-taking app;
- a generic knowledge base;
- an AI chatbot.

Provider-specific operational surfaces are first-class product experiences,
while provider-neutral evidence and context remain the shared system layer.
Operational surfaces must not become independent business-domain owners.

## Responsibilities

The architecture is responsible for:

- preserving source evidence before extracting meaning;
- maintaining explicit domain ownership for durable truth;
- keeping Memory and Context as cross-domain outcomes;
- making Relationships, Decisions, Obligations and provenance first-class;
- keeping provider channels behind adapter and source-record boundaries;
- making AI and agents permissioned, cited and reviewable;
- keeping derived state rebuildable where practical;
- enabling desktop and planned Android operating surfaces over the same local-
  first memory system and client-neutral contracts.

## Boundaries

Макошь must separate these state categories:

| Category | Role | Source of truth |
|---|---|---|
| Source evidence | Raw imported or local artifacts. | Provider/source boundary and local artifact storage. |
| Events | Explain meaningful change. | Append-only event log. |
| Domain records | Durable accepted state and lifecycle. | Owning domain. |
| Relationships | Source-backed semantic links. | Relationships domain, projected into graph views. |
| Memory and knowledge | Reviewed, source-backed understanding. | Domain records plus Memory/Knowledge policy. |
| Derived views | Search, timeline, dossiers, context packs, scores. | Rebuildable projections and engines. |
| Agent outputs | Proposals, summaries, tool actions. | Agent run records until accepted by a domain. |

Provider state is not Макошь truth. AI output is not Макошь truth. UI state is
not Макошь truth. A durable mutation belongs to the owning domain and must cite
source evidence or an explicit owner action.

## System Shape

```text
External and local sources
  -> Provider and import boundaries
  -> Source evidence
  -> Canonical events
  -> Domain records
  -> Relationships and graph projections
  -> Shared engines
  -> Memory and context
  -> Review, UI and agents
  -> Owner decision or action
```

## Domain Connections

Communications are the main intake spine, but not the only source of evidence.
Documents, calendar events, local owner input, imported files and provider
records can also create source-backed memory.

Domains own durable entities. Engines compute reusable intelligence. Agents act
through capabilities and audit. The UI is the Personal Operating System surface
over those boundaries.

## Reasons For Existence

Макошь exists because personal context is fragmented across providers, files,
projects, relationships and time. A provider client can show messages. A task
tracker can show actions. A notes app can show text. Макошь should explain:

- what happened;
- who and what is involved;
- why it matters;
- what changed;
- what evidence supports it;
- what conflicts with prior memory;
- what decision, obligation, task or project context follows.

If a feature does not improve memory, context, evidence or decision quality, it
does not belong in the core architecture.
