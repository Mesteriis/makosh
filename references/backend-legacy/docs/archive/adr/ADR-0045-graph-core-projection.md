# ADR-0045 Graph Core Projection

Status: Proposed

## Context

Version 2 starts by turning the Knowledge Graph into a real backend projection. Макошь already has local PostgreSQL storage for Personas, communication messages and documents. ADR-0008 requires relationships to be durable records with provenance and confidence. ADR-0023 requires derived state to be rebuildable. ADR-0019 forbids ambiguous automatic identity merges. ADR-0031 keeps the UI desktop/laptop only.

## Decision

Use PostgreSQL relational graph tables for the first V2 graph core:

- `graph_nodes`
- `graph_edges`
- `graph_evidence`

The graph tables are a rebuildable projection, not source of truth. Source records remain in `personas`, `communication_messages` and `documents`.

Initial node kinds:

- `person` persisted storage string for Persona nodes;
- `email_address`
- `message`
- `document`

Initial relationship types:

- `person_has_email_address`
- `person_sent_message`
- `person_received_message`
- `email_address_sent_message`
- `email_address_received_message`

System-created edges require evidence. The first projection only uses exact email matching to connect messages to Personas. When no exact Persona exists, the graph uses an `email_address` node instead of inventing a Persona.

Read APIs are local-only, read-only and protected by the existing bearer token plus `X-Макошь-Actor-Id`.

## Non-Goals

- Separate graph database.
- GraphQL.
- Fuzzy person merge.
- Graph editing.
- OCR and entity extraction.
- Task candidate extraction.
- Mobile graph UI.

## Consequences

Positive:

- Graph data stays inspectable and rebuildable in PostgreSQL.
- Provenance is queryable without unpacking arbitrary edge JSON.
- The first V2 slice avoids false Persona merges.
- Existing Docker, SQLx and live PostgreSQL smoke tests remain enough for validation.

Negative:

- Graph traversal depth is intentionally limited in the first slice.
- Richer identity resolution requires a later reviewed merge/split workflow.
- Document-Persona and document-project edges wait for a later extraction engine.
