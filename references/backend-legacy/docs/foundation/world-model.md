# Макошь World Model

## What Exists In Макошь

Макошь models a personal world of evidence, entities, relationships and context.

The core entity types are:

- Persona;
- Organization;
- Communication;
- Project;
- Document;
- Task;
- Event;
- Decision;
- Obligation;
- Relationship;
- Knowledge item;
- Observation;
- Review item.

Provider-specific objects such as Gmail messages, Telegram chats, WhatsApp
threads and calendar provider records are captured as Observations or
channel-specific representations. They are not separate product domains.

Observation is evidence, not truth. If a provider message is deleted, Макошь
captures a deletion observation and keeps the original observation.

## Primary Entities

Primary entities are source-of-truth records with domain ownership:

| Entity | Owner | Source-of-truth role |
|---|---|---|
| Observation | Observation Platform | Canonical append-only evidence. |
| Review item | Review domain | Inbox item for triage, approval, dismissal and promotion. |
| Event | Event log | Append-only fact that something happened. |
| Persona | Personas domain | Subject memory anchor. |
| Organization | Organizations domain | Collective actor memory anchor. |
| Communication | Communications domain | Canonical interaction. |
| Project | Projects domain | Bounded work context. |
| Document | Documents domain | Versioned artifact and evidence. |
| Task | Tasks domain | Actionable unit with lifecycle. |
| Decision | Decisions model | Durable choice with evidence. |
| Obligation | Obligations model | Commitment or duty with evidence. |
| Relationship | Knowledge graph/domain workflow | First-class connection with provenance. |
| Knowledge item | Memory/knowledge model | Reviewed understanding with sources. |

## Derived Objects

Derived objects are rebuildable or generated from primary records:

- Timeline views;
- Dossiers;
- Context packs;
- Search results;
- Search indexes;
- Embeddings;
- AI summaries;
- AI observations;
- scores and rankings;
- graph views;
- attention and risk views.

Derived objects must cite observations or primary entities when they influence
decisions or user-facing explanations.

## Relationship Model

Макошь is relationship-first.

Relationships connect entities:

```text
Persona -> Organization
Persona -> Project
Communication -> Persona
Communication -> Project
Document -> Decision
Decision -> Project
Obligation -> Task
Event -> Relationship
```

Relationships must carry:

- source entity;
- target entity;
- relationship type;
- confidence;
- provenance;
- valid time range where relevant;
- review state where inferred.

## What Is Primary

The primary sources of truth are:

1. Append-only observations.
2. Canonical events.
3. Domain entities and relationships with provenance.
4. Reviewed memory, decisions, obligations and knowledge.

Search indexes, AI outputs, dossiers and timeline views are derived.

## Domain Versus Engine

A domain owns entities and invariants.

An engine provides reusable mechanisms:

- Memory Engine assembles durable memory.
- Timeline Engine builds chronological views.
- Search Engine retrieves source-backed context.
- Trust Engine computes trust signals.
- Context Packs Engine builds rebuildable Persona, Meeting, Task, Calendar and
  Project context packs from explicit sources.
- Identity Resolution Engine proposes same-subject candidates.
- Relationship Engine proposes links between entities.
- Enrichment Engine proposes additional knowledge.
- Obligation Engine detects and tracks commitments.
- Risk Engine detects evidence-backed risks.
- Consistency / Contradiction Engine detects conflicts between new evidence and
  accepted memory.

For example, there is no separate Persona Timeline, Project Timeline and
Document Timeline as independent source-of-truth systems. There is one Timeline
Engine used by Personas, Projects, Documents, Organizations and other domains.

## Owner Model

The owner of the local Макошь instance is represented by the Owner Persona:

```yaml
Persona:
  is_self: true
```

There is exactly one Owner Persona. Agents operate with context from the Owner
Persona and must preserve provenance and permissions.

## Knowledge Model

Knowledge is evidence-backed understanding over the world model. It can be:

- extracted from observations;
- manually created;
- inferred by AI and reviewed;
- linked through relationships;
- invalidated or superseded by later evidence.

Knowledge without provenance is incomplete.
