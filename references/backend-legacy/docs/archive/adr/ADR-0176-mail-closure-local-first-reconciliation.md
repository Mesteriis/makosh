# ADR-0176 Mail Closure: Local-First Reconciliation

Status: Accepted
Date: 2026-07-11

Clarifies ADR-0013, ADR-0018, ADR-0092, ADR-0097 and ADR-0098.

## Decision

Макошь is the canonical owner of the user's Mail workspace state. Provider
state is an external replica that Макошь continuously reconciles through
durable Communications provider commands.

- `is_read`, `read_changed_at` and `read_origin` are independent from
  `workflow_state`. `new` means not triaged; an unread marker means the owner
  has not read the message.
- A local Mail action persists the desired local state and its provider command
  before any provider request. A provider failure does not roll back local
  state.
- Workers claim commands only when due, use bounded exponential backoff with
  jitter, and place exhausted commands in dead letter. Provider recovery
  triggers reconciliation rather than overwriting a newer local intent.
- Provider capability is account-scoped. The UI exposes a provider action only
  when the adapter, server capability and required OAuth scope support it.
- Account health is degraded only after the configured number of consecutive
  connection failures. Locked vault, backend restart and a single command
  failure are operational diagnostics, not account degradation.
- Automation is policy-gated and disabled by default. Cross-domain candidates
  require Review unless a scoped policy explicitly promotes them as an
  automation actor.
- Body, attachment and extracted-text egress are independently denied by
  default. Local Ollama is permitted only at the configured local endpoint;
  any external content egress requires an explicit account permission. AI never
  selects forwarding recipients.

## Consequences

Mail adapters remain integrations: they execute protocol operations and report
observations, but do not write Communications business truth. Communications
owns command intent, reconciliation status and user-visible state. Attachment
bytes stay in blob storage, while PostgreSQL holds metadata, verdicts and
references only.

This ADR does not add Microsoft Graph, POP3, EWS, Proton Bridge, iCloud contact
writes, permanent provider deletion or issue-tracker connectors. Those require
separate accepted decisions and capability contracts.

## Verification

Completion is evidence-based: every row in
`docs/integrations/mail/gap-analysis.md` must be `IMPLEMENTED` or explicitly
excluded by an accepted ADR. Status percentages are not readiness evidence.
