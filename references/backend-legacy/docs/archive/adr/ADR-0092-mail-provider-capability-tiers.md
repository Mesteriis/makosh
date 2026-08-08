# ADR-0092 Mail Provider Capability Tiers

Status: Proposed
Date: 2026-06-13

Clarifies:

- ADR-0041 Email Provider Ingestion Foundation
- ADR-0055 Full Email Provider Networking
- ADR-0076 Host Vault on macOS
- ADR-0080 Mail Background Sync, Progress and Local Trash

## Context

Макошь mail has a provider-neutral storage boundary, Gmail OAuth setup,
iCloud/generic IMAP account setup, SMTP sending for IMAP-backed accounts,
background sync status, local trash and message projections.

The requested working mail scope is wider than the current provider model:
POP3, Exchange, Microsoft 365, Fastmail, Mail.ru, Yandex and Proton have
different protocol semantics. Treating all of them as identical `imap`
accounts would hide real capability differences:

- POP3 has no durable server folders, labels or flag mutation contract.
- Microsoft 365 and Exchange Online should prefer Microsoft Graph OAuth over
  IMAP basic credentials.
- Legacy/on-prem Exchange may require EWS or a local bridge.
- Proton support should normally use Proton Mail Bridge locally; Макошь must not
  attempt to handle Proton account passwords directly.
- Fastmail, Mail.ru and Yandex can work as IMAP/SMTP presets before they need
  first-class provider kinds.

## Decision

Mail providers are modeled as capability tiers. A provider account exposes
capabilities instead of implying that every account can read, send, mutate
folders, mutate flags, sync labels, use OAuth, or expose provider-native
threads.

Initial tiers:

| Tier | Examples | Storage provider kind | Notes |
|---|---|---|---|
| Native API | Gmail, Microsoft 365 | provider-specific | Uses OAuth and provider APIs for read/write when implemented. |
| Standards IMAP/SMTP | iCloud, Fastmail, Mail.ru, Yandex, generic IMAP | `icloud` or `imap` | Uses IMAP for read/sync and SMTP for send. Provider presets are UI/config helpers, not new domain kinds by default. |
| POP3/SMTP | legacy mailboxes | future `pop3` ADR/migration | Ingestion-only mailbox semantics; no provider folder or flag mutation contract. |
| Exchange legacy | on-prem Exchange | future adapter | Requires EWS or a local bridge; not modeled as generic IMAP unless explicitly configured that way. |
| Proton Bridge | Proton Mail | `imap` with bridge metadata | Connects only to a user-run local Proton Bridge IMAP/SMTP endpoint. |

Rules:

- Provider account records continue to store only non-secret metadata and
  adapter configuration.
- Credential lookup remains account-scoped by `account_id` and secret purpose.
- Runtime/UI capability checks must decide whether actions such as send, move,
  copy, label mutation, server delete and folder sync are available.
- UI must not present unavailable provider operations as working actions.
- Local trash remains the default delete behavior from ADR-0080 and must not be
  silently converted into provider delete.
- Adding a durable provider kind outside `gmail`, `icloud` and `imap` requires a
  schema migration and explicit tests for capability reporting.

## Consequences

Positive:

- The account UI can support common providers through presets without inventing
  false native semantics.
- Provider-specific write behavior stays explicit and testable.
- POP3, Microsoft Graph, EWS and Proton Bridge can be added incrementally
  without breaking existing Gmail/IMAP accounts.

Negative:

- Some requested provider names initially map to presets or unsupported
  capability rows rather than full native adapters.
- Capability reporting becomes a required part of account management.
- The current provider account table check constraint must be expanded before
  adding durable non-Gmail/IMAP provider kinds.

## Follow-Up

- Add account management API that returns provider capabilities and sanitized
  account config.
- Add provider presets for Fastmail, Mail.ru, Yandex, Microsoft and Proton
  Bridge in the frontend account wizard.
- Design separate migrations before introducing `pop3`, `microsoft_graph` or
  `exchange_ews` provider kinds.
