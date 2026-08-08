# ADR-0048 Project Link Review Workflow

Status: Proposed

## Context

ADR-0047 introduced project nodes and keyword-derived project relationships. Those relationships are suggested because deterministic keyword containment can create false positives and false negatives.

ADR-0001 requires meaningful changes to be represented as canonical events. ADR-0023 and ADR-0045 make graph tables rebuildable projections, so user review decisions cannot live only on graph edges.

## Decision

Add event-backed project link review for direct project-to-message and project-to-document links.

User review commands append `project.link_review_state_changed` events. A durable `project_link_reviews` read model stores only explicit decisions:

- `user_confirmed`
- `user_rejected`

Resetting a link to `suggested` appends an event and removes the explicit decision row. Unreviewed suggested links remain derived from project keyword rules.

Project graph edges remain rebuildable projection state. During graph projection:

- keyword-only active links use `review_state = suggested`;
- confirmed links use `review_state = user_confirmed`;
- rejected links are omitted;
- confirmed links remain active even when current keyword rules do not match.

Persona and email-address project links remain derived from active project-message links. Direct Persona review is out of scope for this slice.

Protected local review APIs must require the temporary local bearer token and `X-Макошь-Actor-Id`.

## Non-Goals

- Project create/edit UI.
- Keyword management UI.
- Manual Persona merge.
- Direct review of project-person edges.
- AI project inference.
- OCR or entity extraction.
- Mobile UI.

## Consequences

Positive:

- False project links can be rejected without editing source messages or documents.
- Important links can be confirmed even if keyword rules later change.
- Review state survives graph rebuild.
- Project detail and graph projection can share the same active-link rules.

Negative:

- Review commands require event/table transaction discipline.
- The first workflow only handles direct message and document links.
- A later keyword editor still needs separate ADR-backed work.
