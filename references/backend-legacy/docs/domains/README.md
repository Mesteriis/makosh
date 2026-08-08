# Макошь Domain Catalog

Status: documentation package aligned to the current repository structure.

This catalog is the canonical entry point for active Макошь domains. It should
be read together with:

- [Product Master Spec](../product/master-spec.md)
- [World Model](../foundation/world-model.md)
- [Glossary](../foundation/glossary.md)
- [Domain Map](../foundation/domain-map.md)

Макошь is a local-first Personal Memory System. Domains own source-of-truth
entities. Engines build derived memory, context, scores, timelines and
recommendations from those entities.

## Domain Rule

A domain exists when Макошь needs a durable source of truth for an entity type.
A domain does not exist merely because the UI has a page or because an engine
needs a projection.

## Package Shape

Domain documentation mirrors `backend/src/domains/<domain>/` where possible.
Each domain package should use the Zoom-style document set when the content
exists:

- `README.md` for the bounded-context overview or implementation package index;
- `spec.md` for canonical product/domain semantics when the package also has
  implementation-heavy docs;
- `architecture.md` for ownership, flows and boundaries;
- `api.md` for public or local API shape;
- `data-model.md`, `modules.md`, `status.md`, `gap-analysis.md` and
  `blockers.md` when those documents are backed by real current content.

Do not create empty placeholder files just to fill the shape.

## Canonical Domains

| Domain | Canonical document | Status |
|---|---|---|
| Signal Hub | [Signal Hub](signal-hub/spec.md), [package](signal-hub/README.md) | target system domain for source registry, signal control, fixtures, NATS-backed event delivery and ConnectRPC APIs |
| Communications | [Communications](communications/README.md) | implemented as the single communication domain; Mail, Telegram and WhatsApp are channel integrations |
| Personas | [Personas](personas/spec.md), [package](personas/README.md) | partially implemented through `personas` and compatibility migrations |
| Relationships | [Relationships](relationships/README.md) | partially implemented through first-class persistence, graph projection for all current Relationship entity kinds, guarded global suggested review, Organization-Persona link adapters, Persona role adapters, task relation adapters, project link review adapters and Personas workspace review; remaining cross-domain inbox work incomplete |
| Organizations | [Organizations](organizations/spec.md), [package](organizations/README.md) | implemented as a memory anchor domain |
| Projects | [Projects](projects/README.md) | implemented, needs stronger domain documentation |
| Documents | [Documents](documents/README.md) | implemented, needs clearer Knowledge boundary |
| Tasks | [Tasks](tasks/spec.md), [package](tasks/README.md) | implemented, needs stronger Obligation boundary |
| Calendar/Events | [Calendar And Events](calendar/spec.md), [package](calendar/README.md) | implemented under calendar/calls/meetings surfaces |
| Decisions | [Decisions](decisions/README.md) | partially implemented through first-class persistence, accepted graph projection, guarded entity/global API, explicit message/imported-document candidate refresh, email plus Telegram/WhatsApp fixture ingestion refresh, meeting outcome adapter, project link review adapter and Tasks workspace review; adapters incomplete |
| Obligations | [Obligations](obligations/README.md) | partially implemented through first-class persistence, accepted graph projection, guarded entity/global API, obligation-derived task-candidate review-state synchronization, document candidate refresh, email plus Telegram/WhatsApp fixture ingestion refresh, person promise adapter, meeting outcome adapter and Tasks workspace review; adapters incomplete |
| Review | [Review](review/README.md) | implemented as the evidence-backed inbox for review items, approval, dismissal and promotion state per ADR-0096 |
| Knowledge Graph | [Knowledge Graph](graph/README.md) | implemented as graph domain/projection substrate |
| Agents | [Agents](agents/README.md) | partially implemented through AI runtime/control surfaces |
| Notes | [Notes](notes/README.md) | not a first-class domain; treated as document-like artifacts |

## Channel And Provider Capability Specs

Channel and provider capability specs document provider-specific behavior
without promoting a provider into a standalone Макошь domain.

| Provider | Spec | Status |
|---|---|---|
| Telegram | [Telegram Channel Capability Spec](../integrations/telegram/README.md) | target production capability matrix with current implementation baseline |
| Mail | [Email Channel Capability Spec](../integrations/mail/README.md) | implemented email channel framing and current API/status docs |
| WhatsApp | [WhatsApp Provider Stage](../integrations/whatsapp/README.md) | provider/runtime capability docs; not a Макошь domain |
| Zoom | [Zoom Provider Stage](../integrations/zoom/README.md) | provider foundation implemented; not a Макошь domain |

## Engine Documents

Engines are not domains. Engine ownership lives in
[Engines](../foundation/engines.md). Search-specific details live in
[Search Engine Architecture](../engines/search/architecture.md).

## Current Implementation Caveats

The repository still contains historical naming and compatibility boundaries:

- `communications` backend modules still contain some mail-heavy compatibility
  names because email was the first implemented channel.
- Persona-owned and resolved-reference storage covered by ADR-0174 uses
  `persona_id`; bounded legacy input aliases and explicitly out-of-scope
  cross-domain identifiers remain compatibility boundaries. Persona-native API
  surfaces use `/api/v1/personas/*`; legacy `/api/v1/persons/*` routes are
  retired.
- `contacts` appears in migration history because the persistence model evolved
  from contact records into Personas.
- `backend/src/domains/settings` is currently an exported but empty backend
  module. Settings is core application surface, not a product domain; the
  working settings logic lives under [Platform Settings](../platform/settings/README.md).
- Notes have a frontend surface but no ADR that promotes them to a first-class
  domain.
- Agents have product-domain language and a frontend package, but no current
  `backend/src/domains/agents` package. Backend AI/runtime code is split across
  AI, app, platform settings and integration boundaries.
- Decisions and Obligations are canonical product domains. Both have initial
  persistence baselines, accepted graph projection and guarded backend APIs.
  Decisions have explicit message/imported-document candidate refresh, a
  meeting `decision` outcome adapter, Telegram/WhatsApp fixture ingestion
  refresh and a project link review adapter.
  Relationships have organization-person link, task relation and project link
  review adapters.
  Obligations have review-state synchronization for obligation-derived task
  candidates, document commitment candidate refresh, email-sync plus
  Telegram/WhatsApp fixture ingestion candidate refresh, a person promise
  adapter and a meeting `promise`/`task`/`follow_up` outcome adapter. Desktop
  UI, live-provider ingestion and remaining compatibility adapters remain
  incomplete.

These caveats are not new terminology. They are migration facts that must be
resolved by implementation plans and ADRs before code-level renames or schema
changes.
