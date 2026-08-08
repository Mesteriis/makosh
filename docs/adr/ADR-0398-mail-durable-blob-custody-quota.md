# ADR-0398: Mail durable Blob custody quota

- Status: Accepted
- Implementation status: Implemented
- Date: 2026-08-04

## Context

`mail.blob.v1` owns the durable custody scope `mail.attachment.content.v1`.
The descriptor previously requested 16 MiB and reused the per-message
attachment aggregate limit as the Blob quota. Blob quota is cumulative across
all active records in a custody scope; it is not an individual-object limit.

The restored Mail projection already retained almost 16 MiB of attachment
content. A new inbound body therefore could not create the bounded source Blob
needed for the event-backed custody transfer to Communications. Provider sync
continued to materialize metadata, but canonical message bodies were admitted
as unavailable after Blob returned `QuotaExceeded`.

## Decision

Mail requests a 1 GiB cumulative quota for `mail.attachment.content.v1`, equal
to the current Kernel hard ceiling and to the admitted Communications durable
custody budget.

The quota is named `MAIL_BLOB_CUSTODY_QUOTA_BYTES` so it cannot be confused
with the independent per-message attachment bounds retained by Mail contracts
and MIME composition. The capability operations and custody scope do not
change: `mail.blob.v1` remains limited to `write` and `read_range`. Its
capability revision advances from 1 to 2 because increasing an approved quota
is a material grant change that Kernel must reconcile explicitly.

No domain or integration boundary changes. Mail remains the provider
integration; Communications remains canonical body authority after the exact
event-backed custody transfer.

Inbound body observation identity is revision-aware. It remains deterministic
for the same provider record and exact admitted body digest, but differs
between an unavailable admission and a later admitted body, and between two
different admitted body digests. The provider source cursor remains stable, so
Communications appends evidence and updates the same canonical message instead
of treating recovered content as a conflicting replay of the earlier failure.

## Consequences

- Existing retained Mail attachments and new body-transfer source Blobs share
  one explicit cumulative budget without exhausting it at the first 16 MiB.
- Individual message, body, attachment and composed RFC822 limits remain
  independently enforced.
- A resync can repair a previously unavailable canonical body without direct
  integration-to-domain calls or mutation of old evidence.
- Increasing the Kernel hard maximum, splitting custody scopes, or adding
  retention policy requires a separate decision.

## Validation

- Mail descriptor regression test asserts capability revision 2, the exact
  quota, custody scope and allowed operations.
- Body observation identity tests assert stable exact retries and distinct
  revisions for admission recovery and changed content.
- Managed development sync must complete and the selected message body must be
  readable through the Communications content contract without raw MIME.
