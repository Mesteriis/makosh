# ADR-0096 Canonical Evidence, Review Inbox and Context Packs

Status: Accepted

## Context

Макошь already has events, source-backed communications, decisions,
obligations, relationships and reviewable candidates. The missing boundary was
the layer between provider/runtime captures and domain truth.

Provider records are not domain objects. An email message, browser capture,
voice memo, PDF import or meeting transcript should first become evidence. Only
later may ingestion or review promote that evidence into Personas,
Organizations, Meetings, Tasks, Decisions, Obligations, Relationships,
Documents, Projects or Knowledge.

ADR-0001 keeps the event log as the system spine. ADR-0095 keeps cross-domain
communication event-driven. This ADR defines the durable evidence and review
ownership model that sits between integrations and domains.

## Decision

Макошь uses this target flow:

```text
External Systems
  -> Integrations
  -> Vault
     (accounts / capabilities / sources / sessions)
  -> Observation Platform
     (canonical evidence)
  -> Ingestion
  -> Domains
  -> Knowledge
  -> Review
     (inbox / promotion / approval / dismissal)
  -> Actions
```

The Observation Platform is the Canonical Evidence Store. It is a platform
layer, not part of Vault and not a business domain.

Core invariants:

- Observation is evidence, not truth.
- Observations are append-only.
- External deletion or mutation creates another observation. It does not mutate
  or delete the previous observation.
- Vault owns provider accounts, capabilities, sources and sessions. Vault does
  not own observations.
- Review is a domain and the main Макошь inbox for reviewable material.
- Radar remains attention vocabulary and read-model language. It is not a
  durable domain.
- Context Packs are engine output under `engines/context_packs/`. They are
  derived and rebuildable from observations, domains, knowledge, relationships
  and prior decisions.
- Do not create `domains/signals`, `domains/events`, `domains/attention` or
  `domains/evidence`.

Canonical observation kinds are registry-backed. Initial kinds include:

```text
COMMUNICATION_MESSAGE
COMMUNICATION_MESSAGE_DELETED
COMMUNICATION_ATTACHMENT
MEETING
MEETING_RECORDING
MEETING_TRANSCRIPT
DOCUMENT
VOICE_RECORDING
BROWSER_CAPTURE
PERSONA_RECORD
CALENDAR_EVENT
```

Review item kinds include:

```text
new_persona
new_organization
potential_task
potential_obligation
potential_decision
potential_relationship
potential_project
knowledge_candidate
```

`new_person` remains a legacy read alias for older review rows; new writes use
`new_persona`.

Review lifecycle states are:

```text
new
in_review
approved
promoted
dismissed
archived
```

Event flow:

```text
observation.captured.v1
persona.detected.v1
organization.detected.v1
task.candidate.detected.v1
decision.candidate.detected.v1
obligation.candidate.detected.v1
relationship.candidate.detected.v1
knowledge.candidate.detected.v1
review.item.available.v1
review.item.approved.v1
review.item.promoted.v1
review.item.dismissed.v1
```

Identity resolution and relationship detection are separate engines:

- `engines/identity_resolution` decides whether two subjects represent the same
  entity.
- `engines/relationships` decides whether two entities are linked and how.

## Consequences

Positive:

- Provider records stop being promoted directly into domain truth.
- Manual notes, browser captures, voice recordings and imported documents can
  create observations without Vault.
- Provider deletion is represented as evidence, not destructive data loss.
- Review becomes one concrete inbox instead of scattering promotion state across
  Radar, Tasks, Knowledge and candidates.
- Context packs have a real home without becoming source-of-truth records.
- Architecture guard can reject forbidden evidence/attention/signal domains and
  Vault-owned observations.

Negative:

- Existing communication and provider ingestion paths need gradual migration to
  create observations before domain candidates.
- Domains that already store source references need compatibility bridges until
  evidence links are backfilled.
- Review promotion is eventually consistent when downstream domains consume
  events instead of being synchronously called.

## Implementation Notes

The initial implementation adds:

- `observation_kind_definitions`;
- append-only `observations`;
- `observation_links`;
- `observation_ingestion_runs`;
- `review_items`;
- `review_item_evidence`;
- `context_packs`;
- `context_pack_sources`;
- backend modules for `platform::observations`, `domains::review` and
  `engines::context_packs`;
- lightweight engine contracts for identity resolution and relationship
  candidates;
- architecture guard checks for forbidden domain directories and Vault-owned
  observations.

The first implementation does not rename existing provider/source tables or
force all existing task creation paths through observations. That migration
requires a separate compatibility plan because current domains still expose
legacy source/evidence fields.
