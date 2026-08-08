# ADR-0067 Calendar as First-Class Domain with Multi-Provider Architecture

Status: Proposed

## Context

Макошь needs a calendar module that treats events as first-class knowledge graph nodes, not as isolated time blocks. Events must connect to Personas, organizations, projects, documents, tasks, and emails. The system must support multiple calendar providers (Google, Microsoft, Apple, CalDAV, ICS, local) with multiple accounts per provider and multiple calendars per account.

## Decision

Calendar is a first-class domain with `calendar_account_id` = `cal:v1:{uuid}`. Each account binds to a provider with a capabilities model (read/write/delete/recurring/attendees/conference/attachments/reminders/availability/colors/push_sync). Events carry full source identity (`provider` + `account_id` + `source_id` + `source_event_id`). The `calendar_sources` table models individual calendars within an account with per-source sync and read-only flags.

Provider sync is deferred to a future phase (requires OAuth integration similar to email providers). The schema, API, and intelligence layer are fully implemented and ready for provider adapters.

## Consequences

- Events are queryable by account, source, time range, status, and type.
- Each event knows exactly which provider/account/calendar it belongs to.
- Multiple accounts and calendars are supported from day one.
- Provider sync implementation is scoped to a future task; the domain is usable with locally-created events immediately.
