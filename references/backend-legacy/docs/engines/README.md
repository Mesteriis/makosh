# Макошь Engine Catalog

Status: documentation package aligned to the current repository structure.

Engines are reusable system mechanisms. They are used by domains but do not own
domain source-of-truth entities.

Canonical foundation map: [Foundation Engines](../foundation/engines.md).

## Engine Boundary

An engine may:

- read domain entities and event history;
- build derived views;
- emit candidates, observations, scores or review items;
- maintain rebuildable projections or indexes.

An engine must not:

- become the owner of Persona, Organization, Project, Task, Document,
  Communication, Decision or Obligation truth;
- silently overwrite accepted memory;
- hide uncertainty;
- emit conclusions without provenance.

## Engine Specs

| Engine | Spec |
|---|---|
| Memory Engine | [Memory Engine](memory/README.md) |
| Timeline Engine | [Timeline Engine](timeline/README.md) |
| Trust Engine | [Trust Engine](trust/README.md) |
| Search Engine | [Search Engine](search/README.md), [architecture](search/architecture.md) |
| Enrichment Engine | [Enrichment Engine](enrichment/README.md) |
| Obligation Engine | [Obligation Engine](obligation/README.md) |
| Risk Engine | [Risk Engine](risk/README.md) |
| Consistency / Contradiction Engine | [Consistency / Contradiction Engine](consistency/README.md) |
| Automation Engine | [Automation Engine](automation/README.md) |
| Attention Engine | [Attention Engine](attention/README.md) |
| Context Packs Engine | [Context Packs Engine](context-packs/README.md) |
| Identity Resolution Engine | [Identity Resolution Engine](identity-resolution/README.md) |
| Relationship Candidate Engine | [Relationship Candidate Engine](relationships/README.md) |
| Call Intelligence Engine | [Call Intelligence Engine](call-intelligence/README.md) |
| Speaker Identity Engine | [Speaker Identity Engine](speaker-identity/README.md) |

## Package Shape

Engine documentation mirrors `backend/src/engines/<engine>/` where possible.
Use `README.md` for engine semantics and add `architecture.md`, `api.md`,
`status.md`, `gap-analysis.md` or `modules.md` only when the current
implementation needs that detail.

## Current Implementation Evidence

The current repository contains dedicated baseline modules for:

- `backend/src/engines/automation.rs` and `backend/src/engines/automation/`;
- `backend/src/engines/attention/`;
- `backend/src/engines/call_intelligence/`;
- `backend/src/engines/search.rs`;
- `backend/src/engines/consistency.rs`;
- `backend/src/engines/context_packs/`;
- `backend/src/engines/enrichment/`;
- `backend/src/engines/identity_resolution/`;
- `backend/src/engines/memory.rs` and `backend/src/engines/memory/`;
- `backend/src/engines/obligation/`;
- `backend/src/engines/relationships/`;
- `backend/src/engines/risk/`;
- `backend/src/engines/speaker_identity/`;
- `backend/src/engines/timeline.rs` and `backend/src/engines/timeline/`;
- `backend/src/engines/trust/`.

Many other engine-like behaviors are still domain-local:

- `backend/src/domains/personas/memory.rs`;
- `backend/src/domains/personas/trust.rs`;
- `backend/src/domains/personas/enrichment_engine.rs`;
- `backend/src/domains/organizations/memory.rs`;
- `backend/src/domains/*/health.rs`;
- `backend/src/domains/*/intelligence.rs`;
- `backend/src/workflows/email_intelligence.rs`.

That implementation shape is a migration fact, not a domain rule. Future
refactors should move shared reusable behavior toward engine boundaries only
after an implementation plan and ADR where persistence or public contracts
change.
