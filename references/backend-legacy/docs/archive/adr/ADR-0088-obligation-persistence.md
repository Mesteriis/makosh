# ADR-0088 Obligation Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0015 Command Query Separation
- ADR-0020 Task Candidate Lifecycle
- ADR-0070 Tasks First-Class Domain
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0087 Contradiction Observation Persistence

## Context

Макошь is a Personal Memory System. Communications, meetings, calls and
documents often contain commitments, duties and promises. The documentation
distinguishes three concepts:

- an Obligation is a commitment or duty backed by evidence;
- a Task is an actionable unit with status lifecycle;
- a Follow-Up is a prompt to revisit something.

Current implementation represents adjacent behavior through task candidates,
meeting outcomes, person promises and follow-up status. That is not enough for
the target model because it collapses the reason something matters into the
action that may be created from it.

## Decision

Introduce first-class Obligation persistence.

The initial implementation creates durable, source-backed obligation records:

```yaml
Obligation:
  obligation_id:
  obligated_entity_kind:
  obligated_entity_id:
  beneficiary_entity_kind:
  beneficiary_entity_id:
  statement:
  status:
  review_state:
  due_at:
  condition:
  risk_state:
  confidence:
  metadata:
```

Every durable Obligation must have evidence:

```yaml
ObligationEvidence:
  obligation_id:
  source_kind:
  source_id:
  quote:
  confidence:
  metadata:
```

Initial statuses:

```yaml
ObligationStatus:
  open
  fulfilled
  waived
  disputed
  canceled
```

Initial review states:

```yaml
ObligationReviewState:
  suggested
  user_confirmed
  user_rejected
```

Initial risk states:

```yaml
ObligationRiskState:
  none
  watch
  at_risk
  breached
```

Obligations may link to Tasks, but a confirmed Obligation must not
automatically create a Task. Task creation requires a separate user action,
policy or candidate review flow.

## Consequences

Positive:

- Макошь can remember commitments without forcing them into task lifecycle.
- Tasks can cite Obligations as reasons instead of becoming the source of truth
  for commitments.
- Consistency / Contradiction Engine can point at obligation status conflicts.
- Risk and Timeline engines can consume obligations later.

Negative:

- Existing person promises, meeting outcomes and task candidates remain
  compatibility or source surfaces. Initial adapters exist for person promises,
  selected meeting outcomes and obligation-derived task candidates, while
  broader routing remains follow-up work.
- The first desktop UI is scoped to the Tasks workspace; non-task-candidate
  Obligation review routing beyond accepted Obligations remains follow-up work.
- Obligation extraction remains limited and candidate-first. Explicit message
  and document task-candidate refresh paths now exist, while full automatic
  extraction across every provider stream remains follow-up work.

## Non-Goals

- Public `/obligations` API routes.
- Cross-domain workflow placement outside the Tasks workspace.
- Automatic task creation.
- Automatic obligation extraction from every message.
- Removing task candidates, meeting outcomes or person promises.

## Required Follow-Up

- Add candidate-to-Obligation review routing.
- Connect broader Communication, meeting and document extraction to obligation
  candidates beyond the explicit task-candidate paths.
- Expand adapters beyond the initial person promise and meeting outcome
  compatibility baselines.
- Expand reviewed Obligation links to additional compatibility sources.
- Feed obligation conflicts into the Consistency / Contradiction Engine.

## Implementation Status

The backend now has guarded accepted-Obligation list/review routes:

- `GET /api/v1/obligations?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/obligations?review_state=&limit=`;
- `PUT /api/v1/obligations/{obligation_id}/review`.

These routes update accepted Obligation review state only. They do not create
Tasks or create accepted Obligations from candidates.

The desktop frontend now includes a Tasks workspace review panel for global
suggested Obligations and Decisions, with optional entity-scoped filtering. It
uses the guarded list/review routes and sends only explicit owner
`user_confirmed` / `user_rejected` review state. It does not create Tasks or
convert candidates into accepted Obligations.

Migration `0066` and `ObligationStore` project accepted Obligations into graph
for supported obligated and beneficiary entity kinds. The projection creates
`obligation` graph nodes, source-backed `entity_relationship` edges and
`obligation` graph evidence while preserving the Obligation domain as the
source of truth.

Migration `0067` adds explicit task-candidate classification metadata.
`TaskCandidateStore` now creates `obligation_task` candidates from the
Obligation Engine for explicit message and document commitments. When the
candidate is user-confirmed, it creates or updates a source-backed
`user_confirmed` Obligation, preserves source evidence and links the created
Task through `obligation_task_links.link_kind = fulfillment_task`. Generic task
candidates remain task-only.

`PersonaPromiseStore::create` now materializes compatibility `person_promises`
records into source-backed `user_confirmed` Obligations with `raw_record`
evidence. It does not create Tasks.

`MeetingOutcomeStore::add` now materializes meeting `promise`, `task` and
`follow_up` outcomes into source-backed `suggested` Obligations and stores the
created Obligation id in `meeting_outcomes.linked_entity_id`. It does not create
Tasks.
