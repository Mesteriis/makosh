# ADR-0055 Full Email Provider Networking (Read + Write)

Status: Accepted

Supersedes: ADR-0043

## Context

ADR-0043 mandated read-only email provider networking as a temporary safety measure during the initial implementation phase. Макошь has now matured to a point where full email functionality is required: the system must send emails, reply to threads, forward messages, and mutate server-side state (flags, labels, mailbox moves, deletions).

The read-only restriction was always intended to be temporary for a personal local-first system. The owner controls their own data and provider credentials. Макошь is not a multi-tenant SaaS — there is no risk of one user mutating another user's mailbox.

## Decision

Email provider networking supports both read and write operations.

### Read operations (unchanged from ADR-0043)
- Gmail: `users.messages.list`, `users.messages.get` with `format=raw`
- IMAP: `EXAMINE` or `SELECT`, `UID SEARCH`, `UID FETCH BODY.PEEK[]`

### New write operations
- **SMTP sending**: send email through provider SMTP with credentials resolved at runtime
- **IMAP flag mutations**: `UID STORE` for +FLAGS/-FLAGS (Seen, Answered, Flagged, Deleted, Draft)
- **IMAP mailbox mutations**: `UID COPY` + `UID STORE +FLAGS (\Deleted)` + `EXPUNGE` for move/delete
- **Gmail mutations**: `users.messages.modify` for label changes, `users.messages.trash`, `users.messages.send`
- **Gmail drafts**: `users.drafts.create`, `users.drafts.update`, `users.drafts.send`

### Read-only restriction retained only for tests
- Automated integration tests (`backend/tests/`) must use read-only paths where possible to avoid mutating real provider state. Test-specific adapters may use fixture/mock networking.
- IMAP integration tests must continue using `EXAMINE`.
- Gmail API integration tests must use read-only scopes.

### Secret safety (unchanged)
- OAuth tokens, app passwords, and mailbox passwords remain behind the secret boundary from ADR-0016.
- Secret values must not be stored in raw payloads, provenance, checkpoint JSON, logs, or errors.
- Credential lookup must use `account_id` plus secret purpose, never provider kind alone.

### Provider adapter boundary
- Each provider adapter exposes both read and write capabilities.
- Write operations require explicit user action (no auto-send without confirmation).
- The capability runtime from ADR-0052 governs which actions are allowed without confirmation.
- SMTP credentials are stored as separate secret entries with purpose `smtp_password`.

## Consequences

- Макошь gains full email client functionality: compose, reply, forward, flag management, mailbox organization.
- IMAP provider adapters must handle both `EXAMINE` (read) and `SELECT` (read-write) modes.
- SMTP networking introduces a new transport layer alongside existing IMAP/Gmail API clients.
- Test infrastructure must clearly separate read-only fixture tests from optional write-path integration tests.
- Provider account setup must capture SMTP configuration alongside IMAP/Gmail API config.
- The email sync pipeline must distinguish between read-only sync and user-initiated write operations.
- Send operations must be audited through the existing `api_audit_log` infrastructure.
