# ADR-0070 Tasks as First-Class Domain with Local Overlay

Status: Proposed

## Context

Макошь needs a task management module that unifies local tasks and external trackers (Jira, YouTrack, GitHub Issues, etc.) with a personal context layer. Tasks must be linked to contacts, organizations, projects, emails, meetings, and documents.

## Decision

Tasks is a first-class domain with `task_id = task:v1:{nanos_hex}`. Each task has a local overlay (AI summary, private notes, context, risks) that is never synced to external providers. Multi-provider architecture via `task_provider_accounts` with capabilities model. External task identities stored in `external_task_identities` with per-provider status mapping.

The existing `task_candidates` pipeline (AI extraction from messages/documents) remains intact; confirmed candidates become tasks with full context.

## Consequences

- Tasks are queryable by status, project, source type.
- Local context is permanently separate from provider-synced data.
- Provider sync (Jira/YouTrack/GitHub) is schema-ready but deferred to a future phase.
- Privacy boundaries between local and external context are enforced.
