# ADR-0247: Mail SMTP outbound operational capability

Status: Accepted
Date: 2026-07-22
Implementation state: Partial. The bounded plain-text SMTP slice is
implemented: Mail has a separate SMTP package and credential purpose,
independent delivery command/query routes, an owner-local durable queue,
implicit-TLS provider execution, terminal status, and a neutral
Communications observation. Outbound attachment composition and Blob
lease-only attachment reads are not implemented yet and do not inherit this
gate.

## Decision

SMTP send is a Mail integration capability, not a Communications domain
capability. Its clean-room implementation is an explicit extension of the
Mail owner after ADR-0239's IMAP read-only slice.

The exact package split is:

```text
makosh-mail-api
makosh-mail-core
makosh-mail-smtp
makosh-mail-persistence
makosh-mail-runtime
```

`makosh-mail-smtp` may depend on its public Mail API and selected TLS/runtime
libraries only. It must not depend on Mail persistence, Mail runtime,
Communications, Gateway, Blob implementation, Vault implementation or a
provider SDK. Mail runtime is the only composition root and resolves an
SMTP-specific Vault lease. IMAP and SMTP credentials are distinct
`MailCredentialPurpose` values; SMTP never silently reuses an IMAP password.

An outbound request is a typed Mail operational durable command. `accepted`
only means Mail persisted the command; SMTP execution produces a terminal
Mail result. Neither result grants Communications direct access to Mail
operational state. A confirmed send may emit a separate neutral evidence
observation through Communications ingress, with provider receipt identifiers
kept Mail-owned and no SMTP response text, credential, recipient address or
message body in subjects, diagnostics or result errors.

Attachments are read only through an explicit Blob capability lease supplied
to Mail runtime. SMTP does not resolve Communications anchors, read a domain
table, receive provider session state from another integration, or accept a
filesystem path from a client.

## Admission gates

The bounded plain-text SMTP capability is admitted only as the exact package
and capability subset implemented below. Required evidence includes
implicit-TLS SMTP conformance, exact RFC822 serialization and header-injection
rejection, command idempotency, terminal status, Vault purpose/revoke fencing,
compile isolation, and generated Core Gateway contract evidence without an
HTTP compatibility facade.

Outbound attachments remain a separate closed extension. Opening that
extension requires bounded MIME composition and Blob lease-only attachment
streaming evidence; it may not accept client filesystem paths or reuse the
inbound attachment mapping as a cross-owner store lookup.

## Consequences

No SMTP code may be hidden in `makosh-mail-imap`, `makosh-mail-core` may not
open sockets, and Communications cannot become a generic outbound provider
dispatcher. Enabling SMTP does not implicitly enable Gmail mutation or
outbound attachments.

## Evidence 2026-07-25

The implemented plain-text SMTP slice proves:

- `mail.delivery.v1` persists exact command bytes before returning an
  operation receipt; the receipt contains no provider result;
- `mail.delivery.query.v1` is an independently approved route and returns
  typed `pending`, `accepted`, `rejected`, or `outcome_unknown` status;
- Mail persistence owns the queue and delivery-attempt state; exact duplicate
  commands do not execute the provider twice, while conflicting reuse of an
  operation ID is rejected;
- a claimed delivery is never automatically replayed after an ambiguous
  provider outcome, preventing unsafe SMTP double-send;
- `makosh-mail-smtp` performs bounded implicit-TLS SMTP with an optional
  bounded custom CA for conformance and a whole-operation deadline;
- SMTP password resolution uses only `mail_smtp_password` for the admitted
  configuration and never falls back to the IMAP credential;
- provider acceptance and the neutral Communications observation are committed
  atomically in Mail-owned PostgreSQL state and outbox;
- with NATS unavailable, provider execution completes once and the exact
  observation remains pending; after NATS recovery it is replayed through
  Communications with original causation;
- the durable envelope contains neither recipient address nor message body.

The focused live proof is:

```text
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_runtime_accepts_then_completes_smtp_delivery_and_replays_event node scripts/test-authenticated-storage.mjs 1.97.0
```
