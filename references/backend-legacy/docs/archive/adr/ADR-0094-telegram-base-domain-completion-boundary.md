# ADR-0094 Telegram Base Domain Completion Boundary

Status: Superseded by ADR-0097
Date: 2026-06-18

Superseded note: ADR-0097 replaces the "Telegram Channel" operating-surface
framing. Telegram remains a Communication Channel capability set and integration
adapter; it is not a product/backend/frontend domain.

Clarifies:

- ADR-0052 Capability Runtime and Action Confirmation Policy
- ADR-0083 Telegram Live User Client Runtime
- ADR-0091 Telegram Production Client Capability Model
- ADR-0093 Frontend Platform Migration to Vue 3

## Context

Telegram has reached the base Communication Channel scope for Макошь
Communications. It provides source evidence, provider commands, communication
projections, realtime events, identity traces, timeline evidence and media
evidence for other Макошь systems.

Telegram must not become a Memory Engine, Knowledge Engine, Persona Engine,
Organization Engine, Project Engine, Obligation Engine or Decision Engine.
Those systems consume Telegram evidence through existing Макошь boundaries.

Several requested Telegram-adjacent features remain valuable, but they require
separate runtime, permissions, security, media-device or AI review work. Keeping
them inside the base Telegram completion scope would hide unfinished
architecture behind a broad domain label.

## Decision

The base Telegram channel capability set is completed and moves to maintenance
once the implementation, tests and documentation agree that P0
provider-command, lifecycle, reply/forward, topic, dialog, search and media
parity are closed for the supported scope.

Capability states now include:

- `available`: implementation, storage, policy, audit and validation exist.
- `blocked`: architecturally allowed but blocked by a missing runtime,
  dependency, permission, credential or validation gate in the current account.
- `degraded`: implemented but currently running with reduced provider/runtime
  confidence.
- `planned`: intentionally deferred to a named initiative and not part of base
  Telegram completion.
- `unsupported`: intentionally outside Макошь policy or incompatible with the
  current Telegram account/runtime.

The following capabilities are `planned`, not base-domain gaps:

- Bot Runtime;
- Voice Recording;
- Voice Send;
- Video Recording;
- Live Calls;
- Session Export;
- Session Import;
- MTProxy;
- SOCKS5;
- AI Summary;
- Translation;
- Bilingual Reply;
- AI Review Flows.

Rules:

- `planned` capabilities must be visible in the backend capability contract and
  frontend capability matrix.
- `planned` does not authorize implementation inside a Telegram product domain.
- Future work for Bot Runtime, Voice, Calls and AI Layer must start as separate
  initiatives with their own ADR or ADR update before implementation.
- Provider writes continue to use the durable outbox.
- Destructive actions continue to require audit.
- ACK from TDLib or another provider adapter is not success. Provider-write
  commands complete only after provider-observed state or an explicit provider
  result snapshot that carries the durable evidence needed by the projection.
- Telegram UI must use projected/sanitized evidence, not raw TDLib payloads
  directly.
- TanStack Query owns Telegram server state in the frontend; component-level
  fetch remains forbidden.

## Consequences

Positive:

- Base Telegram can be treated as a maintained Communication Channel instead of
  a permanently open feature bucket.
- Deferred features remain discoverable to users and tests through capability
  status without being mislabeled as broken or unsupported.
- Cross-domain systems keep consuming Telegram evidence without Telegram owning
  Memory, Knowledge, Persona, Organization, Project, Obligation or Decision
  behavior.

Negative:

- Some feature requests need a new initiative even if they are Telegram-branded
  in the UI.
- Capability consumers must handle five states instead of four.

## Validation

Completion requires:

- no confirmed `BROKEN` or `REGRESSION` Telegram capability;
- no base-domain P0 gap in Telegram gap analysis;
- no Telegram implementation, test or frontend source file over 700 lines;
- provider writes through outbox;
- destructive actions through audit;
- realtime events through the shared event bus;
- no runtime polling where a realtime provider path exists;
- no component-level Telegram fetch;
- Telegram documentation aligned with the implemented code.
