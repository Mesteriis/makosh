# ADR-0089 Decision Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0015 Command Query Separation
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0087 Contradiction Observation Persistence
- ADR-0088 Obligation Persistence

## Context

Макошь is a Personal Memory System. It must remember not only what happened,
but why a direction was chosen.

Current implementation has decision-shaped data in several places:

- meeting outcomes with `outcome_type = 'decision'`;
- project link review decisions;
- capability policy decisions;
- communication and document evidence that can imply decisions.

These are useful source or workflow surfaces, but none is the durable Decisions
domain described by the product model.

## Decision

Introduce first-class Decision persistence.

The initial implementation creates durable, source-backed Decision records:

```yaml
Decision:
  decision_id:
  title:
  status:
  rationale:
  alternatives:
  decided_by_entity_kind:
  decided_by_entity_id:
  decided_at:
  review_state:
  confidence:
  metadata:
```

Every durable Decision must have evidence:

```yaml
DecisionEvidence:
  decision_id:
  source_kind:
  source_id:
  quote:
  confidence:
  metadata:
```

Decisions also link to impacted entities:

```yaml
DecisionImpactedEntity:
  decision_id:
  entity_kind:
  entity_id:
  impact_type:
  metadata:
```

Initial statuses:

```yaml
DecisionStatus:
  active
  superseded
  reversed
  deprecated
```

Initial review states:

```yaml
DecisionReviewState:
  suggested
  user_confirmed
  user_rejected
```

A meeting outcome, project review or AI extraction may propose a Decision, but
it is not the Decision source of truth until stored as a source-backed Decision
record. Decision persistence does not automatically create Tasks, Projects or
Obligations.

## Consequences

Positive:

- Макошь can answer why a project, communication thread or workflow moved in a
  particular direction.
- Decisions become evidence-backed and reviewable instead of being hidden in
  meeting text, task notes or project state.
- Polygraph can point to conflicting decisions as reviewable contradictions.
- Projects, Documents, Communications, Events, Personas, Organizations, Tasks
  and Obligations can link to Decisions without owning decision truth.

Negative:

- Existing meeting outcomes and review decision tables remain compatibility or
  source surfaces. Initial adapters exist for meeting outcomes and project link
  review decisions, while broader routing remains follow-up work.
- The first desktop UI is scoped to the Tasks workspace; meeting/provider-wide
  candidate-to-Decision review flows are still follow-up work.
- Provider-wide Decision extraction from Communications and Meetings remains
  outside the first persistence slice.

## Non-Goals

- Cross-domain workflow placement outside the Tasks workspace.
- Automatic decision extraction.
- Automatic project status changes.
- Automatic task or obligation creation.
- Removing meeting outcomes or project link review decisions.

## Required Follow-Up

- Add candidate-to-Decision review routing.
- Connect meeting and provider-wide communication extraction to Decision
  candidates.
- Expand adapters beyond the initial meeting outcome and project link review
  baselines.
- Expand accepted Decision graph projection and project reviewed Decisions into
  timeline and dossier views.
- Feed conflicting Decisions into the Consistency / Contradiction Engine.

## Implementation Status

The backend now has guarded accepted-Decision list/review routes:

- `GET /api/v1/decisions?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/decisions?review_state=&limit=`;
- `PUT /api/v1/decisions/{decision_id}/review`.

These routes update accepted Decision review state only. They do not create
Tasks, Projects, Obligations or accepted Decisions from candidates.

The desktop frontend now includes a Tasks workspace review panel for global
suggested Decisions and Obligations, with optional entity-scoped filtering. It
uses the guarded list/review routes and sends only explicit owner
`user_confirmed` / `user_rejected` review state. It does not create Tasks,
Projects or Obligations.

`backend/src/domains/decisions/extraction/` adds a deterministic candidate
detector for explicit Communication and Document evidence, for example
`Decision: Use local-first storage because private context must work offline`.
The detector produces reviewable Decision drafts and evidence references; it
does not persist accepted Decisions or mutate Projects, Tasks or Obligations.

Migration `0065` and `DecisionStore` project accepted Decisions into graph for
supported impacted entity kinds. The projection creates `decision` graph nodes,
source-backed `entity_relationship` edges and `decision` graph evidence while
preserving the Decision domain as the source of truth.

`DecisionStore::refresh_deterministic_candidates` now provides the first backend
candidate-to-Decision persistence path for explicit Communication messages and
imported Documents. It stores detected candidates as source-backed `suggested`
Decisions impacted by the source Communication or Document, preserves
`user_confirmed` and `user_rejected` review state across repeat refreshes, and
relies on the existing guarded Decision review route for confirmation. It does
not create Tasks, Projects or Obligations.

`MeetingOutcomeStore::add` now materializes meeting `decision` outcomes into
source-backed `suggested` Decisions impacted by the meeting Event and stores the
created Decision id in `meeting_outcomes.linked_entity_id`. It does not create
Tasks, Projects or Obligations.

`ProjectLinkReviewStore::set_review_state` and projection replay now
materialize explicit `user_confirmed` / `user_rejected` project link review
events into source-backed `user_confirmed` Decisions impacted by the Project and
the reviewed Communication or Document. This records the owner decision behind
the compatibility project-link surface without changing Project, Task or
Obligation state.
