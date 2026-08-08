# ADR-0068 Calendar Event as Knowledge Graph Node

Status: Proposed

## Context

Calendar events in Макошь must not be isolated time-block rectangles. Each event is a system node connected to Personas, organizations, projects, documents, tasks, emails, and notes. ADR-0045 established the graph core projection. Events need explicit, queryable relationships.

## Decision

Events participate in the knowledge graph through `event_relations` with `entity_type`/`entity_id`/`relation_type`. Supported entity types: `persona`, `organization`, `project`, `document`, `task`, `email`, `note`, `decision`, `obligation`, `recording`. Event-participant links are stored in `event_participants` with resolved Persona references, email, role, and response status.

Event context packs aggregate related data (documents, tasks, open questions, risks, suggested agenda, suggested actions) into a materialized JSONB snapshot for fast retrieval.

## Consequences

- Events are traversable from any related entity through the graph.
- Context packs provide instant context without cross-domain joins at read time.
- Participants are first-class with Persona resolution, enabling participant intelligence.
- Graph integration (`graph_nodes`/`graph_edges`) is read-through and deferred until the graph projection is updated for event nodes.
