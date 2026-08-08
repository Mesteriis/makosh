# ADR-0240: Canonical Communications owner clean-room migration

Status: Accepted
Date: 2026-07-22
Implementation state: Fully implemented. The owner now has hash-scoped evidence,
accounts, conversations, messages, observed participants, attachment anchors,
reply/forward references, transactional inbox/outbox, a generated metadata-query
port, typed Blob-backed body admission receipts/failures, and a Kernel-inherited
managed domain process root. The generic private owner-control launch contract
and Core Gateway routing to the owner-owned public query contract are present.
`managed_communications_domain_starts_with_owner_local_storage_and_events`
runs against the disposable authenticated Storage/NATS/Vault/Blob contour through
`test-authenticated-storage.mjs`; it proves the generic managed-domain launch
with owner-local Storage and Event Hub credentials, typed `makosh-mail-core`
and `makosh-telegram-core` ingress observations and their resulting canonical
owner projections through the
managed owner query route, and the exact canonical outbox envelope on its
catalog NATS subject. The same live contour redelivers the exact ingress bytes
and proves inbox idempotency by observing no second canonical event. External
authenticated Core Gateway application routing is also covered: a logical
owner Browser WebAuthn session reaches the descriptor-declared ConnectRPC
Communications query route and the managed runtime response. Listener/TLS
transport coverage remains Gateway platform evidence. A typed admitted opaque
Blob receipt also reaches canonical body state while the public query response
does not expose its Blob locator. A typed attachment descriptor reaches its
canonical attachment anchor while the public anchor response does not expose
the provider-local media locator. A typed participant, reply and forward scope
reaches canonical relationship projections without exposing provider-local
identifiers through the public query contract. The Blob child starts as a
separately signed managed platform process; the same managed conformance now
proves integration-owned physical body admission, custody transfer, owner-local
derived indexing and a canonical-ID-only search hit. The frontend generation
pipeline now includes the same versioned owner query descriptor, and its
platform-level Connect client is distinct from the legacy provider operational
client. The first clean-room client use case is an owner-local canonical search
adapter: it accepts transient search text and returns only generated canonical
search hits. The same generated metadata-query contract now provides canonical
evidence lookup without a provider cursor, content, Blob locator or custody
proof. Canonical evidence now persists ingress causation/correlation and exact
recorded timestamp in its owner-local audit row; the metadata lookup exposes
only these opaque audit identifiers and timestamps.
The canonical evidence event retains the ingress correlation ID and names the
direct ingress message as its causation; it never creates a replacement
cross-owner correlation.
Attachment anchor, Blob-admission, safety-verdict and lifecycle schemas now
have one domain-owned source in `makosh-communications-attachment-contract`;
the former ingress/API schema locations were removed without a facade or
duplicate re-export.
All backend owner criteria below are implemented and covered by the exact
`attachment_security_engine_v1` inventory plus the canonical backend gates. The
repository cutover is also complete:
`legacyCommunicationsRestInventory.boundary.test.ts` proves that production
frontend source contains no `/api/v1/communications/*` caller outside the
generated canonical owner client. The last orphaned attachment-import and
WhatsApp business REST chains were removed. The Telegram media-upload workflow
and its file action were removed with them because no admitted client Blob
upload boundary exists; they were not replaced by a facade, fallback or direct
integration-to-domain call. The generated Telegram operational Connect client
remains the only admitted future provider-operation seam, and a future media
upload capability requires its own client Blob admission phase gate.

Depends on:

- ADR-0204: bundled integrations and provider-neutral context boundary;
- ADR-0205: Core Gateway and client transport;
- ADR-0220: canonical durable envelope;
- ADR-0223: encrypted Vault and scoped credential leases;
- ADR-0236: integration owners, protocol adapters, and configuration instances;
- ADR-0239: Mail/IMAP read-only first-owner slice.
- ADR-0257: event-backed Blob custody transfer for canonical evidence.

## Decision

`communications` is one clean-room canonical evidence owner. It owns durable
communication evidence and its provider-neutral read model. Provider
integrations own operational state, provider cursors, authentication, provider
commands, and provider-specific screens. No integration imports a
Communications implementation, and Communications never imports a provider SDK
or switches behaviour by provider identity.

The owner is composed only from these packages:

```text
makosh-communications-ingress
makosh-communications-attachment-contract
makosh-communications-api
makosh-communications-domain
makosh-communications-persistence
makosh-communications-runtime
```

Their responsibilities are exact:

| Package | Responsibility |
|---|---|
| ingress | Versioned typed neutral observation accepted from integrations. |
| attachment contract | Versioned typed attachment lifecycle events and observations shared only through exact public contracts. |
| api | Public typed query and command contracts for Gateway/client adapters. |
| domain | Validation, deterministic identity, idempotency decisions and canonical state transitions. |
| persistence | Owner-local PostgreSQL schema, inbox/outbox and implementations of domain ports. |
| runtime | Composition root and only consumer of the private persistence implementation. |

### Canonical model

A provider observation becomes one `CommunicationEvidenceV1`. It contains only:

- stable observation and source identities;
- provider-neutral channel and record kinds;
- source provenance suitable for audit, never a provider cursor or secret;
- observed and recorded timestamps;
- causation and correlation identifiers when supplied;
- body/media references owned by Communications or Blob, never raw provider
  payload;
- bounded display metadata represented by typed fields, not a generic map.

The owner derives a canonical conversation, message and participant projection
from accepted evidence. A message edit, deletion, reaction, reply or forward is
an explicit typed transition; it is not an untyped JSON patch. A record may be
represented as metadata-only evidence when its body is not admitted.

`CommunicationEvidenceV1` is not a Task, Persona, Organization, Document or
any other business entity. Promotion requires a separate Review/workflow
contract with an explicit evidence reference.

### Inbound and outbound boundaries

```text
provider runtime
  -> typed Communications ingress observation
  -> Communications inbox/idempotency decision
  -> canonical evidence and owner outbox
  -> durable envelope/event spine
  -> Communications API query
  -> generated Gateway client
```

Inbound integrations must not call owner-local SQL, create canonical IDs, or
write Communications blobs. Communications does not call provider operational
contracts. Provider identity is preserved as provenance for traceability but is
not a domain behaviour selector.

Communications has no generic outbound provider command. A provider-neutral
intent is performed only by an explicit workflow that records its evidence and
then calls one provider-specific operational contract.

### Clean-room and migration rules

The legacy Communications workspace is historical behaviour evidence only. Its
generic JSON metadata/payload fields, cross-owner persistence assumptions,
provider-specific projection types, compatibility exports, schema reuse and
runtime wiring are prohibited from the clean-room graph. A legacy behaviour is
adopted only after it has a typed clean-room contract, an owner-local
implementation and regression coverage.

No compatibility facade, dual-write, fallback query path, legacy import or
runtime routing to `references/` is allowed. Existing temporary skeleton
objects are removed as each canonical contract replaces them; they are not
preserved as aliases.

### Completion criteria

The domain is considered migrated only when all of the following are true:

1. Every public Communications input/output is versioned, typed and
   provider-neutral; no generic metadata map or provider DTO crosses the owner
   boundary.
2. Evidence, conversations, messages, participants, body/media anchors and
   typed message transitions have canonical owner contracts and owner-local
   persistence.
3. Ingress idempotency, durable inbox/outbox, replay, causation, correlation,
   source provenance and body-admission failure outcomes are implemented.
4. Mail and Telegram publish only through ingress and read canonical state only
   through public owner APIs; their operational contracts remain independent.
5. Gateway/client integration uses generated Communications contracts; Vue
   contains presentation/lifecycle state only.
6. There is no production legacy reference, facade, fallback, cross-owner SQL
   or direct runtime/store edge.
7. Architecture policy, compile isolation, owner tests, backend CI, frontend
   validation and integration evidence cover the complete path.

### Current completion evidence

| Criterion | State | Evidence |
|---|---|---|
| 1-4: typed owner model, durable owner state and event-only integration ingress | Complete | Exact six-package Communications inventory, typed lifecycle contracts, owner-local PostgreSQL inbox/outbox and live Mail/Telegram replay evidence. |
| 5: generated Gateway/client owner contract | Complete | Generated Communications Connect client and canonical search/evidence adapters are present. The exact legacy REST inventory now proves zero production callers, and unsupported provider operations were removed rather than routed through a compatibility client. |
| 6: no legacy facade or cross-owner backend edge | Complete | Architecture and Cargo guards reject backend facades, cross-owner SQL and direct owner edges. Frontend boundary tests reject any renewed `/api/v1/communications/*` production caller and keep the deleted Telegram workflow inventory explicit. |
| 7: complete validation | Complete for the Communications cutover | The exact managed-owner acceptance remains independently replayable through `managed_communications_domain_starts_with_owner_local_storage_and_events`. Frontend lint, typecheck, 707 unit tests, build, browser-bootstrap and clean-room Tauri bundle gates pass. Targeted Messenger visual execution reaches the affected story; the repository-wide visual baseline still reports pre-existing local-icon drift unrelated to this cutover, so no unrelated baseline was rewritten as migration evidence. |

## Consequences

The current read-only Mail path remains a useful provider slice, but it is no
longer evidence that the Communications owner is complete. Future channel
features land by extending the typed canonical contract and owner-local
projections, not by adding another provider-specific branch to a shared client
or persistence layer.
