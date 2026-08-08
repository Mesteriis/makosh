# ADR-0400: Mail sync deadline terminal-state schema

- Status: Accepted
- Implementation status: Implemented
- Date: 2026-08-04

## Context

Mail sync health defines `DeadlineExceeded` as failure code 10. The owner-owned
PostgreSQL constraint admitted only codes 1 through 9. When an IMAP provider
operation exceeded its bounded deadline, Mail could not persist the terminal
failure. The run remained `Running`, subsequent health/client RPCs failed, and
the runtime retried the same impossible transition indefinitely.

## Decision

Mail storage revision 31 adds the owner-local `deadline_exceeded BOOLEAN`
marker to `mail_sync_runs`. The existing constrained `failure_code` stores the
compatible provider-unavailable category (`6`), while the marker preserves the
exact `DeadlineExceeded` meaning exposed by the Mail contract. Reads accept the
marker only together with that category and reconstruct the exact failure code.

This additive representation follows Storage Control migration admission;
historical schema steps and their existing `1..=9` constraint remain immutable.
No Communications, Kernel or integration-external table is changed.

Revision 30 was materialized during development with a destructive constraint
replacement and rejected by Storage Control before PostgreSQL application. Its
digest can remain pinned in an existing Control Store, so those bytes are not
rewritten or reused. Revision 31 advances directly from the last applied Mail
revision 29 and contains only the admitted additive step.

The runtime storage bundle composes the new revision after the existing iCloud
CardDAV credential revision 29. `DeadlineExceeded` remains a terminal
failed outcome and does not silently become success or trigger an unbounded
retry.

## Consequences

- Timed-out IMAP sync runs can reach a durable terminal state.
- A later manual sync is no longer blocked by a stale unique running row.
- Sync health remains restart-safe and accurately distinguishes provider
  timeout from runtime interruption.

## Validation

- Storage bundle tests must prove the revision-31 successor and exact additive
  Mail-only marker migration.
- Managed validation must show a previously stuck run becoming terminal and a
  new server sync being admitted.
