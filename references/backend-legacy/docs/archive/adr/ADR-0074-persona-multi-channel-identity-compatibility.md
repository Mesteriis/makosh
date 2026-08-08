# ADR-0074 Persona Multi-Channel Identity Compatibility

Status: Accepted

## Context

ADR-0019 established identity resolution as confidence-scored merge/split
candidates. The current Persona projection was originally derived from a single
email address and is already referenced by graph projections, project links and
task context. Макошь Personas require multi-channel identities (email, Telegram,
WhatsApp, phone, GitHub, LinkedIn, and others) linked to a single Persona.

## Decision

Keep the current storage primary key as a stable text identifier:
`person:v1:email:{len}:{normalized_email}` for email-created Personas. The
`personas.email_address` unique constraint remains in place as the primary-email
compatibility contract for existing projections.

Add a separate `persona_identities` table for all channel-specific identifiers.
Each identity carries `source`, `confidence`, `status`, verification metadata,
and a unique active `(identity_type, identity_value)` identity constraint.
Auto-creation from incoming email creates or matches the primary email Persona
row and backfills the matching `persona_identities` record.

Opaque UUID Persona IDs are not part of this implementation slice. Moving from
text `person_id` compatibility values to opaque IDs requires a separate ADR and
migration plan covering graph nodes, tasks, projects, communication projections,
API payloads and frontend state.

## Consequences

- One Persona can have many identities across channels.
- Identity resolution (merge/split) operates on current text `person_id`
  compatibility values.
- Historical backfill migrates existing `contact:v1:email:*` IDs to
  `person:v1:email:*` format and creates email identity rows.
- The identity candidate table is `persona_identity_candidates`.
- A future opaque-ID migration remains possible, but it must be explicit and cannot be inferred from this ADR.
