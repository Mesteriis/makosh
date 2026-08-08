# ADR-0174: Persona-Native Physical Identifier Columns

Status: Proposed
Date: 2026-07-10

Clarifies:

- ADR-0074 Persona Multi-Channel Identity Compatibility
- ADR-0084 Persona Intelligence System
- ADR-0090 Persona-Native Compatibility API Bridge

## Context

Макошь already uses Persona-native table, API and domain names, while several
durable columns and internal models still use `person_id`. ADR-0074, ADR-0084
and ADR-0090 intentionally retained those names until a dedicated physical
migration decision was accepted.

Maintaining two names for the same identifier now adds ambiguity to SQL,
projections, event consumers and compatibility aliases.

## Decision

Use `persona_id` as the canonical physical column and internal code name for
identifiers that reference a Persona within the scope of migration `0202`.

Migration `0202` performs in-place renames for:

- `personas.person_id` to `personas.persona_id`;
- the Persona-owned and resolved-reference `person_id` columns enumerated by the
  migration to `persona_id`;
- identity candidate sides to `left_persona_id` and `right_persona_id`;
- expertise endorsement to `endorsed_by_persona_id`;
- `persona_interaction_contexts.persona_id` to `interaction_context_id` and
  `person_id` to `source_persona_id`;
- related index names where they encode renamed columns.

Stored values, SQL types, nullability, primary keys, foreign keys and uniqueness
semantics do not change. PostgreSQL updates dependent definitions during the
column rename; the migration does not add duplicate compatibility columns or
writable compatibility views.

New SQL, models, API responses and newly emitted payloads use Persona-native
names. Historical durable payloads and established request contracts remain
readable through bounded legacy aliases such as `person_id`, while serialization
and new writes use canonical names.

## Compatibility Constraints

- Existing text identifier values, including `person:v1:email:*`, remain
  unchanged.
- Existing event, observation and review history is not rewritten.
- `/api/v1/personas/*` remains the public API; `/api/v1/persons/*` is not
  restored.
- Schema and backend code are upgraded together. A pre-0202 binary must not run
  against a post-0202 schema.

## Rollback

Migration files remain append-only after release. Rollback requires either
restoring the pre-0202 database backup together with the previous binary or
shipping a new forward migration and matching code. Editing or deleting a
released migration `0202` is not a supported rollback.

## Non-Goals

- Changing Persona identifier values or their generation strategy.
- Migrating to opaque UUID identifiers.
- Rewriting historical event or evidence payloads.
- Renaming `linked_person_id`, `owner_person_id` or other identifiers outside
  the explicitly reviewed `0202` scope.
- Reintroducing legacy public routes or legacy output fields.

## Consequences

Positive:

- The migrated storage, SQL and active contracts use Persona-native vocabulary.
- Existing durable data and identifier references retain identity continuity.
- Replay consumers retain bounded legacy-input compatibility.

Negative:

- The migration is a lock-step schema/application upgrade.
- A rollback requires backup restoration or a new forward migration.
- Identifiers outside the explicit `0202` scope retain their current names.

## Validation

- Apply migrations through `0202` to an isolated PostgreSQL database.
- Assert canonical columns and indexes exist and scoped legacy names are absent.
- Exercise Persona creation against the migrated schema.
- Replay a payload containing the legacy `person_id` key.
- Run `make architecture-check`, `make backend-validate` and `make validate`.
