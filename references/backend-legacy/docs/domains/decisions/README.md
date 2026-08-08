# Decisions Domain

Status: documentation package aligned to the current repository structure.

Decisions are durable choices with rationale, evidence and consequences.

Макошь needs Decisions because the Personal Memory System must remember not only
what happened, but why a direction was chosen.

## Responsibilities

The Decisions domain owns:

- decision records;
- decision status;
- rationale;
- alternatives considered;
- evidence links;
- impacted entities;
- review and supersession history.

The Decisions domain does not own:

- generic notes;
- project lifecycle;
- task status;
- AI summaries;
- every preference or fact.

## Decision Sources

Decisions can come from:

- explicit owner entry;
- communication evidence;
- meetings and calls;
- documents;
- project reviews;
- agent suggestions that are confirmed by the owner.

AI can propose a decision candidate. A durable decision requires review or an
explicit policy that allows automatic capture for a narrow source.

## Decision Model

```yaml
Decision:
  id:
  title:
  status:
  rationale:
  alternatives:
  decided_by:
  decided_at:
  evidence:
  impacted_entities:
  supersedes:
  review_state:
```

## Current Implementation Evidence

Current backend baseline:

- `backend/migrations/0064_create_decisions.sql`;
- `backend/src/domains/decisions/mod.rs`;
- `backend/src/domains/decisions/api.rs`;
- `backend/src/domains/decisions/extraction/`;
- `backend/migrations/0065_decision_graph_projection.sql`;
- `backend/tests/decisions.rs`;
- `backend/tests/decisions_api.rs`;
- `backend/tests/decision_engine.rs`;
- `backend/tests/calendar.rs`;
- `backend/tests/project_link_reviews.rs`;
- ADR-0089.

This baseline provides source-backed Decision persistence with evidence,
rationale, alternatives, review state, confidence and impacted entities. It
also includes a deterministic Decision candidate detector for explicit
Communication and Document evidence such as `Decision: ... because ...`. The
candidate detector can produce a `NewDecision` draft, source evidence and
impacted entity links. `DecisionStore::refresh_deterministic_candidates`
provides the first backend persistence path for explicit Communication message
and imported Document candidates: it stores them as source-backed `suggested`
Decisions impacted by the source Communication or Document and preserves
reviewed state across repeat refreshes.
Email sync and Telegram/WhatsApp fixture ingestion call the same targeted
message refresh path for newly projected Communications. This creates
reviewable suggested Decisions only; it does not auto-create Tasks, Projects or
Obligations.
The existing review route can then confirm or reject the suggested Decision. The
store projects accepted Decisions into the graph for supported impacted entity
kinds, using `decision` graph nodes and source-backed `entity_relationship`
edges. It explicitly does not auto-create Tasks, Projects or Obligations.
Meeting outcomes with `outcome_type = decision` now adapt into source-backed
`suggested` Decisions impacted by the meeting Event. The meeting outcome keeps
the created Decision id in `linked_entity_id` so the calendar compatibility
surface points at the durable Decision record.

Project link review commands with explicit `user_confirmed` or `user_rejected`
state now adapt into source-backed `user_confirmed` Decisions impacted by the
Project and the reviewed Communication or Document. This keeps the existing
project review workflow but records the owner decision in the Decisions domain.

Backend routes currently expose:

- `GET /api/v1/decisions?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/decisions?review_state=&limit=`;
- `PUT /api/v1/decisions/{decision_id}/review`.

These routes are guarded by the local API secret and support accepted Decision
review state changes. They do not create Tasks, Projects or Obligations and do
not convert meeting outcomes or project review decisions into accepted
Decisions.

The Tasks workspace includes the first desktop review panel for global
suggested Decisions and Obligations, with optional entity-scoped filtering. It
lists Decisions through the guarded Decision route and submits explicit owner
confirm/reject review state without creating Tasks, Projects or Obligations.

Decisions still also appear indirectly through project context, documents and
communications. Those are source or compatibility surfaces until remaining
candidate routing is complete.

## Migration Plan

1. Keep ADR-0089 as the persistence boundary.
2. Keep decision capture candidate-first.
3. Expand Communication and Meeting ingestion beyond the initial explicit
   message/imported-document refresh path and meeting outcome adapter.
4. Add candidate-to-Decision review before any automatic decision capture.
5. Require evidence citations and review state.
6. Expand graph projection beyond the current supported impacted entity kinds.
7. Project reviewed Decisions into timeline and dossier views.
