# ADR-0261: Communications attachment-anchor handoff for independent producers

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Реализовано для первого Mail producer. Communications
publisher, schema-bound handoff через единственный integration boundary
`makosh-communications-ingress`, atomic outbox insert и Mail-owned durable
mapping реализованы. Signed managed Mail runtime получает exact Kernel
subscription permit, принимает anchor event, проверяет исходный Mail outbox
record и сохраняет mapping с correlation ID до Blob-result continuation.
Другой integration producer требует собственного admission и conformance.

Зависит от:

- [ADR-0201: Core module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0246: Communications attachment admission and safety](ADR-0246-communications-attachment-admission-and-safety.md);
- [ADR-0260: attachment lifecycle event authority](ADR-0260-communications-attachment-lifecycle-event-authority.md).

## Context

Mail already owns bounded provider parsing, its Blob write operation and its
own durable outbox. It can publish a provider-neutral attachment descriptor,
but it cannot know the `attachment_anchor_id` that Communications derives from
its canonical message and media identities. Re-deriving that ID in Mail would
copy domain identity semantics into an integration and make provider code a
hidden Communications implementation.

The Blob-admission observation from ADR-0260 intentionally accepts only a
canonical anchor ID. Therefore an integration needs a typed owner event that
binds its already published source observation to the anchor created by
Communications.

## Decision

Communications publishes one canonical durable event
`communication_attachment_anchor_recorded.v1` when it creates an attachment
anchor from an admitted descriptor observation. Its payload contains only:

- `attachment_anchor_id` (16 bytes);
- `source_observation_id` (the admitted observation/envelope ID, 16 bytes);
- `media_cursor_sha256` (32 bytes);
- the initial state `descriptor_only`;
- observed time.

The event is inserted into the Communications canonical outbox in the same
transaction as the inbox record, evidence projection and attachment anchor.
It uses the source observation envelope as causation. A duplicate observation
does not publish another handoff event.
It also retains the source observation correlation ID, so the handoff remains
in the same durable process rather than introducing an anchor-specific
correlation.

An integration that opts into attachment Blob admission keeps the mapping from
its own outbox observation ID to this anchor **and the received correlation
ID** in its own storage. After a provider-local download and a separately
granted Blob write, it persists its own
`communication_attachment_blob_admission_observed.v1` envelope with the
stored correlation ID and only then relays it through its separately granted
Event Hub publish route. Its causation remains the Mail-owned source
observation ID.

The handoff is a public owner event, not a domain-to-integration call:

- Communications does not subscribe to Mail or call a provider/Blob scanner;
- Mail consumes only the typed public event and never imports Communications
  implementation, persistence or domain identity code;
- the Mail admission may publish Blob admission facts, but never safety
  verdicts;
- a security engine needs its own future admission to publish safety verdicts.

## Rejected alternatives

- Placing `attachment_anchor_id` into the provider descriptor observation:
  it would make an integration assert an owner-controlled canonical identity.
- Reproducing Communications hash derivation in Mail: duplicated domain
  semantics are a cross-owner facade even if the code is small.
- A query RPC lookup by provider locator: it would expose provider operational
  identifiers to Communications and create a synchronous integration/domain
  dependency.

## Required implementation and evidence

1. Add the public canonical anchor-recorded schema, publisher capability and
   atomic owner outbox insert.
2. Add an integration-local typed anchor mapping consumer/store in the Mail
   assembly unit; it must validate causation against the Mail-owned original
   outbox record and durably retain the non-zero handoff correlation ID for
   each future Blob-admission continuation.
3. Admit Mail's Blob-result publisher in a separate exact phase slice, then
   prove replay, grant revoke, stale runtime generation and CAS conflict.
4. Admit the scanner verdict producer separately. `safe_for_delivery` remains
   unreachable before that admission.

## Evidence 2026-07-24 — Mail handoff

Live managed conformance выполняет полный provider descriptor → canonical
anchor → Mail mapping путь. Anchor source равен `communications-runtime`,
causation указывает на Mail-owned source observation, а mapping хранится только
в Mail PostgreSQL. Продолжение использует сохранённые canonical anchor и
correlation ID; Mail не вызывает Communications RPC, не читает её storage и не
повторяет owner identity derivation.
