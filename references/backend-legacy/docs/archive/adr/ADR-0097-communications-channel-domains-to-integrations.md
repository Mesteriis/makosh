# ADR-0097 Communications Channel Domains To Integrations

Status: Accepted
Date: 2026-06-20

Supersedes:

- ADR-0094 Telegram Base Domain Completion Boundary, for the use of
  "Telegram Channel" as an operating surface or bounded-context label.

Clarifies:

- ADR-0041 Email Provider Ingestion Foundation
- ADR-0051 WhatsApp Web Companion Boundary
- ADR-0055 Full Email Provider Networking
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0091 Telegram Production Client Capability Model
- ADR-0092 Mail Provider Capability Tiers
- ADR-0093 Frontend Platform Migration to Vue 3
- ADR-0095 Event-Driven Domain Communication and DLQ

Route note: ADR-0098 supersedes the intermediate provider-scoped business route
decision from this ADR. Channels remain integrations, but business
Communications APIs are now provider-neutral under `/api/v1/communications/*`;
provider setup/runtime APIs live under `/api/v1/integrations/*`.

## Context

Макошь accumulated channel-shaped implementation surfaces while building email,
Telegram and WhatsApp support. Email started as the first communication
implementation. Telegram later gained a large account/runtime/message UI and was
documented as a completed "base domain". WhatsApp documentation also described a
future channel-specific workbench.

That language is now misleading. A channel is not a domain. A channel is an
integration. A communication is the domain object.

The long-term product boundary is Communications:

```text
Communication -> Source Evidence -> Extracted Knowledge -> Memory -> Context
```

Mail, Telegram and WhatsApp provide source records, account metadata, runtime
state and provider commands. They do not own durable product state such as
messages, conversations, participants, attachments, drafts, outbox, search,
AI/workflow state or provider command envelopes.

## Decision

Макошь has one Communications domain.

Rules:

- `communications` owns account, channel, identity, conversation, participant,
  message, attachment, message version, tombstone, reaction, folder, draft,
  outbox, search, AI/workflow and provider command state.
- Mail, Telegram and WhatsApp are integration adapters.
- Provider/protocol/runtime code lives under `backend/src/integrations`.
- Frontend provider setup/runtime panels live under `frontend/src/integrations`.
- User-facing communication workspace lives at `/communications`.
- Public channel-scoped API routes use:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

- Public legacy route families are removed:

```text
/api/v1/<legacy-provider-root>/*
```

where `<legacy-provider-root>` was `email-accounts`, `telegram` or `whatsapp`.

- Mail, Telegram and WhatsApp must not reappear as backend or frontend product
  domains.
- Frontend route families `/telegram` and `/whatsapp` are removed.
- Provider-specific frontend query keys rooted directly at provider names are
  not domain cache keys. Provider-scoped communication cache keys use
  `['communications', provider, ...]`.
- Realtime domain events patch Communications caches. Integration runtime
  events may patch only integration-owned runtime panels when such panels exist.

## DTO And Naming Contract

Domain DTOs use provider-neutral names:

```text
CommunicationAccount
CommunicationChannel
CommunicationIdentity
CommunicationParticipant
CommunicationConversation
CommunicationMessage
CommunicationAttachment
CommunicationMessageVersion
CommunicationMessageTombstone
CommunicationReaction
CommunicationProviderCommand
```

Provider-specific DTOs such as `TelegramMessage`, `WhatsappMessage`,
`MailMessage` or `EmailMessage` are allowed only inside integration/runtime
modules and integration-scoped tests.

Historical database tables and compatibility Rust names may remain until
explicit migrations remove them. New runtime/domain code must not treat
provider-prefixed tables as the owning domain state.

## Data Contract

The canonical communication table family is:

```text
communication_accounts
communication_channels
communication_identities
communication_conversations
communication_conversation_participants
communication_messages
communication_attachments
communication_message_versions
communication_message_tombstones
communication_message_reactions
communication_message_refs
communication_folders
communication_drafts
communication_outbox
communication_provider_commands
communication_sync_runs
communication_sync_checkpoints
communication_raw_records
communication_raw_payloads
```

Historical provider-prefixed tables may remain for upgrade compatibility and
migration traceability. PostgreSQL stores metadata, provenance, observations,
projections, commands and workflow state. Secrets and session material remain in
the host vault or integration runtime storage according to ADR-0076.

## Consequences

Positive:

- Communications has one owner and one public workspace.
- Channels stop duplicating product-domain logic.
- Provider runtimes can evolve without changing the product domain boundary.
- Realtime, search, AI/workflow state, outbox and provider commands gain a
  shared communication model.

Negative:

- Existing Telegram and mail-heavy code requires large mechanical moves.
- Historical docs and tests that used domain language need cleanup.
- Some provider-specific type names remain temporarily inside integration code
  and tests until the final compatibility cleanup.

## Validation

The repository must enforce:

- no backend mail domain directory;
- no frontend Telegram domain directory;
- no frontend WhatsApp domain directory;
- no public `/api/v1/<legacy-provider-root>/*` route families for
  `email-accounts`, `telegram` or `whatsapp`;
- channel-scoped communication API routes under `/api/v1/communications/*`;
- no user-facing frontend query keys rooted directly at Telegram or WhatsApp;
- canonical communication migration after `0148`.
