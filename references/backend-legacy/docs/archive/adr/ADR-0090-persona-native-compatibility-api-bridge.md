# ADR-0090 Persona-Native Compatibility API Bridge

Status: Accepted

Clarifies:

- ADR-0084 Persona Intelligence System

## Context

ADR-0084 defines Persona as the target domain entity. Durable storage now uses
`personas` / `persona_*` table names and the active API surface is
`/api/v1/personas/*`; internal `person_id` storage columns remain as explicit
physical compatibility.

Макошь needs Persona-native read and write surfaces so new UI and agent flows
can speak the target language. A physical identifier rename away from
`person_id` columns is still a separate migration decision because existing
projections, graph rows, tasks, communications and historical event payloads
depend on compatibility names.

## Decision

Expose the active Persona API under `/api/v1/personas/*`.

The bridge may read and write the current compatibility projection, but its
public contract uses Persona terminology and target-model shapes.

Allowed in this bridge:

- read Persona list/detail models from Persona storage with physical
  `person_id` compatibility columns;
- update owner-editable Persona identity fields such as display name;
- set the single Owner Persona through Persona-native request fields;
- return the same Persona read model after writes;
- keep storage compatibility details explicit in docs and migration plans.

Not allowed in this bridge:

- infer an opaque Persona identifier migration from the API rename;
- reintroduce `/api/v1/persons/*` compatibility routes;
- change Persona identity or `persona_type` without explicit validation rules;
- create separate Self/UserProfile storage;
- auto-merge identity traces without review.

The initial write bridge is intentionally narrow. It updates:

```yaml
PersonaUpdate:
  identity:
    display_name:
  is_self:
```

`is_self: true` sets the requested Persona as the only Owner Persona. `is_self:
false` is not a supported way to remove the Owner Persona; ownership must move
to another Persona instead.

## Consequences

Positive:

- New code can use Persona terminology without waiting for physical schema
  migration.
- Compatibility state remains stable for existing workflows.
- Owner Persona semantics are available through the target API language.
- The future schema migration has a clearer API contract to preserve.

Negative:

- The backend still contains physical `person_id` compatibility columns and
  historical aliases internally.
- Legacy `/api/v1/persons/*` routes are retired and guarded by regression tests.
- Some target-model fields remain read-only until their source-of-truth
  boundaries are defined.

## Follow-Up

- Physical column renaming is completed under ADR-0174; changing email-derived
  `person:v1:email:*` values remains a separate future decision.
- Expand write support only for fields with clear source-of-truth ownership and
  review semantics.
- Keep compatibility gaps visible in `docs/refactoring/implementation-alignment-plan.md`.

## Implementation Update: 2026-07-10

ADR-0174 completes the scoped physical rename from `person_id` columns to
Persona-native column names. This ADR remains the active API bridge decision;
references above to physical compatibility columns describe its pre-0202
baseline. Historical payloads and established request aliases remain readable,
while responses and newly emitted payloads use Persona-native names only.
