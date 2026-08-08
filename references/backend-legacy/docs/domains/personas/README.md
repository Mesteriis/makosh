# Макошь — Persona Intelligence

Status: documentation package aligned to the current repository structure.

`personas` is the domain that lets Макошь understand people, remember relationships
and build context.

Макошь no longer treats people as legacy address-book rows. A Persona is not an
address-book entry, CRM lead or imported card. A Persona is a durable memory
anchor for a subject in the local knowledge graph.

```text
Understand people.
Remember relationships.
Build context.
```

## Domain Vision

Макошь is a Personal Memory System. The Personas domain provides the Persona
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
- Address-book manager
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

`Persona.id` is the logical identity of the subject inside Макошь. Active API
routes and read payloads use `/personas` and Persona-native identifier names.
Physical storage uses `personas` / `persona_*` tables and Persona-native
`persona_id` identifier columns. New domain language must use Persona.

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
