# Canonical Communications Architecture

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: Communications target model and channel ownership. ADR-0097 defines the
channel/domain split, and ADR-0098 is the controlling decision for
provider-neutral business routes, integration runtime routes and strict
frontend/backend boundary guards.

## Purpose

Communications are the primary intake spine for Макошь. Messages, calls,
meetings, provider events and communication attachments enter Макошь as source
evidence, then feed memory, relationships, decisions, obligations, tasks,
projects, documents and context.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Decisions / Obligations / Tasks / Projects
```

## Responsibility

The Communications domain owns:

- channel accounts and non-secret account metadata;
- conversations and provider-thread projections;
- canonical messages and communication records;
- participants as observed by the source;
- raw/source communication metadata;
- communication attachment metadata and local blob references;
- draft, delivery and provider-command state;
- communication-to-entity links and source evidence references;
- channel capability surfaces.

The Communications domain does not own:

- Persona truth;
- Organization truth;
- Project lifecycle;
- Task lifecycle;
- Decision truth;
- Obligation truth;
- Memory truth;
- global Timeline;
- Search indexes;
- AI conclusions.

## Channel Structure

Target implementation and documentation structure:

```text
backend/src/domains/communications/
  conversations/
  messages/
  participants/
  attachments/
  search/
  realtime/

backend/src/integrations/
  mail/
  telegram/
  whatsapp/
```

The current repository has `docs/integrations/mail`,
`docs/integrations/telegram`, `docs/integrations/whatsapp` and
`docs/integrations/zoom`. Those directories should be interpreted as channel or
provider capability specs, not as separate product domains. Mail, Telegram,
WhatsApp and Zoom provider/runtime panels belong under integration modules and
may be embedded into the Communications workspace.

## Shared Abstractions

| Abstraction | Owner | Applies to |
|---|---|---|
| ChannelAccount | Communications | Email, Telegram, WhatsApp and future channels. |
| Conversation | Communications | Email threads, Telegram chats/topics, WhatsApp dialogs. |
| Message / CommunicationRecord | Communications | Provider messages, calls and meeting-related communication records. |
| Participant | Communications for observed source role; Personas for accepted identity. | Sender, recipient, member, caller, attendee, mentioned actor. |
| Attachment | Communications at message boundary; Documents after explicit promotion. | Email MIME parts, Telegram media, WhatsApp media, voice notes, contact cards. |
| ProviderCommand | Communications/integration boundary. | Send, reply, forward, reaction, delete, pin, archive, media upload. |
| Capability | Backend capability contract. | Read, local write, provider write, destructive, export, secret access, recording. |
| Realtime Event | Shared event bus. | `telegram.*`, `whatsapp.*`, mail sync and communication patch events. |
| Search Result | Search Engine over Communications projections. | Message, media, participant and attachment retrieval. |

## Channel Boundaries

### Email

Email is a channel under Communications. Current implementation is mail-heavy
because email was first. Email provider records, raw MIME, attachments, drafts,
SMTP/IMAP/Gmail writes and local trash behavior feed the shared communication
model.

### Telegram

Telegram is a completed base Communication Channel for the supported desktop
scope. It supplies source evidence, provider commands, communication
projections, realtime events, identity traces, timeline evidence and media
evidence. Deferred Telegram initiatives such as Bot Runtime, Voice, Calls,
Session Import/Export, Proxy and AI review flows are not base-channel gaps.

### WhatsApp

WhatsApp is a planned Communication Channel through an explicit WhatsApp Web
desktop companion boundary. It must remain owner-visible and capability-gated.
It must not become hidden scraping, a separate messenger product or a Persona,
Task, Decision, Obligation or Memory owner.

## Lifecycle Rules

Communication channel lifecycles must preserve:

- account-scoped source identity;
- append-only raw records where available;
- idempotent provider record identity;
- local projection into canonical messages/conversations;
- durable provider-command state for writes;
- provider-observed reconciliation before marking writes complete;
- redacted audit for high-risk actions;
- sanitized realtime events;
- attachment scanner state;
- source-backed extraction into candidates.

## Attachments

Communication attachments start as source evidence. They may later be promoted
or linked to Documents, but promotion must be explicit and must preserve source
provenance.

Rules:

- bytes stay out of PostgreSQL;
- metadata, hashes, scanner state and blob paths live in database records;
- no attachment is marked `clean` without a real scanner backend;
- channel media viewers should reuse shared attachment preview/search
  boundaries when possible.

## API And UI Boundary

The only user-facing communication workspace is `/communications`. Channel
filters, account setup panels, runtime status panels and capability panels may
appear there, but Telegram, WhatsApp and Mail do not get top-level product
routes.

Public channel-scoped API routes use:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

Removed public route families:

```text
/api/v1/<legacy-provider-root>/*
```

where `<legacy-provider-root>` was `email-accounts`, `telegram` or `whatsapp`.

Frontend channel work must use:

- Communications-scoped API clients and query keys;
- TanStack Query for server state;
- shared realtime bootstrap;
- backend capability states before exposing provider operations;
- no direct provider/runtime calls from components.

## Reasons For Existence

Communications exist because real-world context mostly enters Макошь through
interaction. Макошь must understand not just message bodies, but timing,
participants, silence, replies, attachments, delivery, source provenance and
provider state. Without a shared Communications model, each channel would
duplicate lifecycle, capability, audit, search, attachment and extraction logic.
