# ADR-0262: Mail attachment Blob-admission extension

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Частично реализовано, production-positive путь доказан.
Mail core extracts bounded MIME parts, Mail keeps an owner-local
source-to-anchor mapping and durable `requested -> admitted/rejected` outbox
state, and managed IMAP runtime writes exact provider bytes through a one-use
Kernel-issued Blob session. После успешного owner-local Blob commit Mail
атомарно сохраняет отдельный typed Attachment Security scan-candidate outbox;
live managed contour доказывает exact provider-neutral candidate и отсутствие
в нём provider/credential data. Exact signed Mail descriptor/grant и live
Blob-result delivery доказаны вместе с replay/CAS conflict. Полный negative
conformance matrix из admission gate, включая отдельные live Blob
integrity/failure сценарии, ещё не завершён. ADR не расширяет `first_owner_v1`
и не открывает `attachment_security_engine_v1`. Versioned Mail settings schema,
Storage bundle and descriptor artifact are defined by ADR-0263.

Зависит от:

- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0230: Blob opaque-reference boundary](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data session](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0246: attachment admission and safety](ADR-0246-communications-attachment-admission-and-safety.md);
- [ADR-0260: attachment lifecycle event authority](ADR-0260-communications-attachment-lifecycle-event-authority.md);
- [ADR-0261: attachment-anchor handoff](ADR-0261-communications-attachment-anchor-handoff.md).

## Context

The historical Mail/IMAP read-only slice intentionally emits descriptors only.
It exposes no attachment-byte extraction or Blob-result event. The new
Communications anchor handoff makes a later independent producer possible, but
it does not make provider bytes, a Blob lease, or a result-publishing grant
available to Mail.

Adding a direct Communications API, a domain table lookup, or an integration
copy of canonical anchor derivation would violate the clean-room boundary.

## Decision

Mail may add one bounded, provider-local attachment-admission capability in its
own packages. It has this exact sequence:

```text
Mail-owned source observation
  -> public attachment-anchor-recorded event
  -> Mail-owned source-to-anchor mapping
  -> bounded local MIME part extraction
  -> one-use Blob write lease and direct Blob socket write
  -> Mail-owned outbox
  -> communication_attachment_blob_admission_observed.v1
```

The producer receives no Communications database credential, runtime socket or
implementation dependency. Communications receives no MIME bytes, provider
locator, provider session, Blob path, Blob URL or download operation. The
result event carries only the canonical anchor, expected lifecycle state,
transition, evidence ID, observed time and opaque integrity binding defined by
the exact `makosh-communications-attachment-contract` schema.

MIME extraction is bounded by the existing RFC822 byte, depth and part limits.
Malformed, unsupported, oversized, missing or integrity-mismatched parts yield
a typed rejected result or no admission result; they never fall back to
metadata-derived bytes, filesystem paths or a permissive state transition.

Mail persists provider locator/download state, source-to-anchor mapping,
producer inbox/outbox and retries in Mail storage. Blob owns bytes and Blob
metadata. Communications only projects its typed terminal event through the
CAS lifecycle already defined by ADR-0260.

The Mail mapping also stores the non-zero correlation ID from the public
anchor handoff. Every requested or terminal Blob-admission observation reuses
that correlation ID while causation remains the immediately preceding
Mail-owned source observation. Mail must reject a malformed handoff before it
persists this mapping.

## Admission gate

The capability needs a separate atomic production gate before it is active:

1. signed and approved exact Mail descriptor capability for one subscribed
   `communication_attachment_anchor_recorded.v1` contract and one publish
   `communication_attachment_blob_admission_observed.v1` route;
2. signed Mail runtime/settings/descriptor digest and owner-approved grant;
3. exact Mail Storage/Vault/Blob session fences, including revoke and stale
   generation rejection;
4. conformance for replay, source mapping conflict, missing source outbox,
   malformed and oversized MIME parts, Blob write/hash failure, relay restart,
   duplicate event and Communications CAS conflict;
5. compile-isolation proof that Mail sees Communications only through the exact
   public units `makosh-communications-ingress` and
   `makosh-communications-attachment-contract`.

The scanner verdict producer remains outside this decision. No producer may
emit `safe_for_delivery`.

## Evidence 2026-07-24 — live positive path

- conformance-only loopback IMAP transport включается отдельной Cargo feature;
  default/release build сохраняет TLS и exact port `993`;
- реальный IMAP fixture отдаёт RFC822 message с bounded base64 PDF part;
- Mail использует owner capability `mail.blob.v1`, а не generic/foreign Blob
  capability, и Kernel выдаёт process-bound one-use write session;
- Blob binding содержит non-zero integrity proof, но bytes, socket path,
  provider locator и credentials не попадают в Communications event;
- третий provider sync и exact terminal replay не создают вторую admission;
  stale CAS observation оставляет public owner state `blob_admitted`.

## Rejected alternatives

- Embedding attachment bytes in Communications ingress or event subjects.
- Letting Communications download a provider attachment or use a provider SDK.
- Calling Blob from Communications or allowing Mail to query Communications
  persistence for anchors.
- Reusing body admission as an attachment shortcut.
- Admitting a generic "download attachment" API or broad read-all Blob grant.
