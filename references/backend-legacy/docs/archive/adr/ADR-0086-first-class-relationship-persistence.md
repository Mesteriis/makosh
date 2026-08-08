# ADR-0086 First-Class Relationship Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0045 Graph Core Projection
- ADR-0084 Persona Intelligence System
- ADR-0085 Communication Spine and Consistency / Contradiction Engine

## Context

Макошь is relationship-first. Current implementation stores relationship-shaped
data in several places:

- `graph_edges` as graph projection records;
- `relationship_events` as Persona timeline records;
- `person_roles` and `person_personas` as historical contact-era structures;
- organization, project and task link tables;
- trust and health fields on Persona and Organization read models.

This fragments the source of truth. It also conflicts with ADR-0084, which
requires Relationship records with source Persona, target Persona,
relationship type, trust score and strength score.

The graph remains essential, but graph edges are optimized for traversal and
projection. They should not be the only durable model for reviewed
relationships.

## Decision

Introduce first-class Relationship persistence.

The initial implementation creates a compatibility-safe `relationships` table
and a backend `relationships` domain store. The table stores a relationship as:

```yaml
Relationship:
  relationship_id:
  source_entity_kind:
  source_entity_id:
  target_entity_kind:
  target_entity_id:
  relationship_type:
  trust_score:
  strength_score:
  confidence:
  review_state:
  valid_from:
  valid_to:
  metadata:
```

Persona-to-Persona relationships are the first supported source path:

```yaml
source_entity_kind: persona
target_entity_kind: persona
```

This preserves the ADR-0084 model while leaving room for later relationships
between Organizations, Projects, Communications, Documents, Tasks, Decisions
and Obligations.

Each relationship must have evidence:

```yaml
RelationshipEvidence:
  relationship_id:
  source_kind:
  source_id:
  excerpt:
  metadata:
```

AI output may propose relationships, but accepted durable relationships remain
source-backed. Suggested relationships are stored with review state and
provenance; they are not silent truth.

`graph_edges` remain a derived graph traversal surface. The implementation
projects Relationship records between graph-supported entities as generic
`entity_relationship` graph edges, while preserving the Relationship record as
source of truth. The current supported projection endpoints match the current
`RelationshipEntityKind` set: Persona, Organization, Project, Communication,
Document, Task, Event, Decision, Obligation and Knowledge.

## Consequences

Positive:

- Relationship becomes a durable domain concept instead of a scattered field.
- Trust and strength scores have a clear owner.
- Persona Intelligence can depend on relationship records without treating
  Personas as CRM contacts.
- The graph can remain rebuildable from source relationships and evidence.
- Future Polygraph, Trust and Risk outputs can point to relationship records.

Negative:

- Existing relationship-like tables remain as compatibility or read-model
  surfaces until migration plans retire them.
- There is temporary duplication between `relationships`, graph edges and
  timeline events.
- The first desktop review UI is still surfaced inside the Personas workspace;
  broader cross-domain workflow placement and remaining compatibility adapters
  still need explicit follow-up work.

## Non-Goals

- Reintroducing `/persons` routes.
- Removing `graph_edges`.
- Removing `relationship_events`, `person_roles` or organization/project link
  tables.
- Automatically deriving trust from contradictions.

## Implementation Status

The backend now includes guarded routes for listing Relationship records by
entity and changing review state:

- `GET /api/v1/relationships?entity_kind=&entity_id=&limit=`
- `GET /api/v1/relationships?review_state=&limit=`
- `PUT /api/v1/relationships/{relationship_id}/review`

Review updates re-project active graph-supported Relationship edges so the
graph projection follows the Relationship source of truth.

The desktop frontend now includes a Personas workspace review panel for global
suggested Relationships. It uses the guarded global review list route, keeps
entity-scoped formatting when a Persona is selected and sends explicit owner
`user_confirmed` / `user_rejected` review state.

Manual/API `person_roles` now materialize source-backed `has_role`
Relationships from Persona to role Knowledge anchors. Removing a role demotes
the same Relationship to `user_rejected`.

Manual/API and email-sync `organization_persona_links`, manual `task_relations`
and explicit project link reviews now also materialize source-backed
Relationship records behind their compatibility surfaces.

## Required Follow-Up

- Move or duplicate Relationship review into a broader cross-domain review
  inbox when the workflow shell is defined.
- Reclassify remaining relationship-shaped compatibility and read-model
  surfaces into Relationship records.
- Feed reviewed Relationship records into Trust, Risk, Timeline and Dossier
  projections.
- Update implementation alignment docs as each compatibility surface is
  retired.
