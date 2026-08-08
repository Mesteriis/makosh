# Макошь Communications - Email Channel

Status: documentation package aligned to the current repository structure.

Email is a communication channel inside Макошь, not the product identity.
The Mail operational screen is a first-class provider experience. Its bundled
integration plugin preserves source evidence and maps provider records into the
provider-neutral Communications evidence contract without giving the
Communications domain ownership of Mail folders, drafts, cursors or protocol.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

## Principles

- **Personal-first**: the system serves the local owner.
- **Provider-independent**: providers are transport and source-record
  boundaries.
- **Email as evidence**: email can produce Communications, Events, Documents,
  Obligations, Tasks, Decisions and Relationships.
- **AI-assisted, owner-controlled**: AI proposes; the owner or policy confirms.
- **Local-first**: private memory remains local.

## Current Implementation Surface

The current backend exposes email-related communication routes under:

```text
http://127.0.0.1:8080/api/v1/integrations/mail/
```

Implementation metrics and route details live in the API/status documents. This
README describes the domain framing.

## Lifecycle

```text
Email provider record
  -> raw source preservation
  -> RFC 2822 parsing
  -> canonical Communication projection
  -> event creation
  -> engine processing
     - Search Engine indexing
     - Risk Engine spam/phishing signals
     - Obligation Engine candidate extraction
     - Enrichment Engine entity/link candidates
  -> UI/API context
```

## Navigation

- [Architecture](architecture.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
