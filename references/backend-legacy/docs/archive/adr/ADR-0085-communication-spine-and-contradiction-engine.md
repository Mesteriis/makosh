# ADR-0085 Communication Spine and Consistency / Contradiction Engine

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0041 Email Provider Ingestion Foundation
- ADR-0055 Full Email Provider Networking
- ADR-0084 Persona Intelligence System

## Context

Макошь is a local-first Personal Memory System. The product model treats
Communications as the primary ingestion spine: messages, meetings, calls and
provider events enter Макошь as source evidence and become knowledge, memory,
relationships, obligations, tasks, decisions and project context.

The repository still contains email-heavy implementation boundaries because
email was implemented first. Telegram, WhatsApp, calls and meetings already
exist as adjacent surfaces. Documentation needs one canonical model that
explains how all interaction evidence enters the system.

The user also approved a Polygraph concept: when a new message, document or
event contradicts remembered knowledge, Макошь should detect the conflict and
surface it for review.

## Decision

Treat Communications as the primary ingestion spine for the Personal Memory
System.

The canonical flow is:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
```

Email, Telegram, WhatsApp, calls, meetings and future providers are channels
feeding the Communications model. Provider-specific behavior remains at adapter
and source-record boundaries.

Add the Consistency / Contradiction Engine as a shared engine. Its user-facing
alias is Polygraph.

The engine compares new evidence with accepted memory and knowledge. It creates
source-backed contradiction observations and review items. It must not:

- decide that a Persona is lying;
- silently overwrite accepted memory;
- mutate domain state without review or explicit policy;
- hide source references.

Required contradiction output includes:

```yaml
ContradictionObservation:
  old_source:
  new_source:
  affected_entities:
  conflict_type:
  old_claim:
  new_claim:
  confidence:
  severity:
  review_state:
```

## Consequences

Positive:

- Communications become the common entry point for memory, knowledge and action.
- Email-specific functionality can be documented as a channel, not as the whole
  product.
- Contradictions become explicit reviewable observations instead of silent
  memory drift.
- The engine boundary prevents every domain from inventing its own local
  polygraph logic.

Negative:

- Existing `mail` module naming remains a compatibility detail until a future
  implementation migration is planned.
- No dedicated backend module, table or review workflow exists yet for the
  Consistency / Contradiction Engine.
- Existing domain-local intelligence and health modules must be audited before
  shared engine extraction.

## Non-Goals

- Immediate code rename from Mail to Communications.
- Immediate schema migration.
- Immediate public API design.
- Automatic contradiction resolution.
- Using contradiction detection as a punitive trust judgment.

## Required Follow-Up

- Keep detailed engine behavior in `docs/engines/consistency/README.md`.
- Keep communication ingestion behavior in `docs/domains/communications/README.md`.
- Add implementation ADRs before introducing persistence, route groups or
  automated contradiction resolution.
- Start future implementation with reviewable contradiction observations.
