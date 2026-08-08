# ADR-0047 Project Memory Spine

Status: Proposed

## Context

Version 2 needs graph-backed memory that connects messages, Personas, projects and documents. The current V2 graph core projects Personas, email addresses, messages and documents, but projects still exist only as frontend presentation data. This blocks project timelines and prevents the graph from using projects as durable memory anchors.

ADR-0045 intentionally limited the first graph core to four node kinds and five relationship types. Project nodes and project relationships therefore require an explicit ADR and schema evolution before implementation.

## Decision

Add a local `projects` read model as the first project memory spine.

Projects are canonical local records with deterministic `project_id` values, human-readable metadata and explicit `project_keywords`. Keywords are user/system configured matching rules, not AI inference. The first implementation may seed a local `Макошь` project record so a development database has a real project anchor, but all project relationships must still be derived from stored messages and documents.

Extend the PostgreSQL graph projection with:

- node kind `project`;
- relationship type `project_has_message`;
- relationship type `project_has_document`;
- semantic relationship `project_involves_persona` (`project_involves_person`
  remains the persisted graph string until a graph replay migration is planned);
- relationship type `project_involves_email_address`.

Project graph edges are rebuildable projection state. They must carry evidence from the matched message or document and preserve confidence/review state. The first project matching rule is deterministic keyword containment over message subject/body and document title/extracted text. These links are `suggested` unless a later review workflow confirms them.

Expose read-only protected local APIs:

- `GET /api/v2/projects`;
- `GET /api/v2/projects/{project_id}`.

The project detail API returns project metadata, derived stats, recent communications, related documents, key Personas and timeline items. It must not expose message body text.

## Non-Goals

- Project write UI.
- AI project inference.
- Fuzzy project merge or rename workflow.
- OCR or rich entity extraction.
- Task candidate extraction.
- Outbound provider writes.
- Mobile project UI.

## Consequences

Positive:

- Projects become first-class graph nodes.
- Project pages can use real local backend data instead of frontend mocks.
- Project timelines can be built from source-backed messages and documents.
- Later OCR/entity extraction can improve links without changing the frontend contract.

Negative:

- Keyword matching can create false project suggestions.
- The first project model is intentionally narrow.
- Project management UI and review/confirmation workflows remain future work.
