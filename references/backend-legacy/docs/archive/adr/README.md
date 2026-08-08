# Archived Architecture Decision Records

All records in this directory were archived on 2026-07-15. Their status fields
describe their historical lifecycle only; none is an active architecture
decision for the clean-room backend.

ADR status vocabulary:

- Proposed: accepted as initial architecture direction, pending implementation validation.
- Accepted: validated by implementation and retained.
- Temporary: intentionally time-bounded decision that must be revisited before expanding scope.
- Superseded: replaced by a later ADR.

## Index

- [ADR-0001 Event Sourcing as System Spine](ADR-0001-event-sourcing-as-system-spine.md)
- [ADR-0002 Rust Backend](ADR-0002-rust-backend.md)
- [ADR-0003 SvelteKit Frontend](ADR-0003-sveltekit-frontend.md)
- [ADR-0004 Tauri Desktop Shell](ADR-0004-tauri-desktop-shell.md)
- [ADR-0005 PostgreSQL Primary Store](ADR-0005-postgresql-primary-store.md)
- [ADR-0006 Tantivy Full Text Search](ADR-0006-tantivy-full-text-search.md)
- [ADR-0007 Replaceable Vector Search](ADR-0007-replaceable-vector-search.md)
- [ADR-0008 Knowledge Graph First](ADR-0008-knowledge-graph-first.md)
- [ADR-0009 Local AI Through Ollama](ADR-0009-local-ai-through-ollama.md)
- [ADR-0010 Specialized Agent System](ADR-0010-specialized-agent-system.md)
- [ADR-0011 Plugin Architecture](ADR-0011-plugin-architecture.md)
- [ADR-0012 OpenTelemetry Observability](ADR-0012-opentelemetry-observability.md)
- [ADR-0013 Local First Data Ownership](ADR-0013-local-first-data-ownership.md)
- [ADR-0014 Canonical Event Envelope](ADR-0014-canonical-event-envelope.md)
- [ADR-0015 Command Query Separation](ADR-0015-command-query-separation.md)
- [ADR-0016 Secrets and Encryption Boundary](ADR-0016-secrets-and-encryption-boundary.md)
- [ADR-0017 Document Processing Pipeline](ADR-0017-document-processing-pipeline.md)
- [ADR-0018 Provider Adapter Boundary](ADR-0018-provider-adapter-boundary.md)
- [ADR-0019 Persona Identity Resolution](ADR-0019-persona-identity-resolution.md)
- [ADR-0020 Task Candidate Lifecycle](ADR-0020-task-candidate-lifecycle.md)
- [ADR-0021 Calendar as Event Source](ADR-0021-calendar-as-event-source.md)
- [ADR-0022 No Fine Tuning on Private Data](ADR-0022-no-fine-tuning-on-private-data.md)
- [ADR-0023 Rebuildable Projections](ADR-0023-rebuildable-projections.md)
- [ADR-0024 Idempotent Imports](ADR-0024-idempotent-imports.md)
- [ADR-0025 Keyboard First Command Palette](ADR-0025-keyboard-first-command-palette.md)
- [ADR-0026 Desktop First Responsive UI](ADR-0026-desktop-first-responsive-ui.md)
- [ADR-0027 Capability Based Permission Model](ADR-0027-capability-based-permission-model.md)
- [ADR-0028 Backup and Restore as Core Feature](ADR-0028-backup-and-restore-as-core-feature.md)
- [ADR-0029 Explicit Schema Evolution](ADR-0029-explicit-schema-evolution.md)
- [ADR-0030 Documentation First Monorepo](ADR-0030-documentation-first-monorepo.md)
- [ADR-0031 Temporary Desktop Only UI Scope](ADR-0031-temporary-desktop-only-ui-scope.md)
- [ADR-0032 Docker Compose Development Environment](ADR-0032-docker-compose-development-environment.md)
- [ADR-0033 Backend Managed Local Schema Migrations](ADR-0033-backend-managed-local-schema-migrations.md)
- [ADR-0034 Event Replay and Projection Cursors](ADR-0034-event-replay-and-projection-cursors.md)
- [ADR-0035 Local Event API Command Boundary](ADR-0035-local-event-api-command-boundary.md)
- [ADR-0036 Projection Runner Checkpoint Semantics](ADR-0036-projection-runner-checkpoint-semantics.md)
- [ADR-0037 Local Write Capability Token](ADR-0037-local-write-capability-token.md)
- [ADR-0038 Local Event API Capability Token](ADR-0038-local-event-api-capability-token.md)
- [ADR-0039 Local Event API Access Audit Log](ADR-0039-local-event-api-access-audit-log.md)

## Recent Provider ADRs

- [ADR-0102 Zoom Provider Runtime Boundary](ADR-0102-zoom-provider-runtime-boundary.md)
- [ADR-0104 Yandex Telemost Provider Runtime Boundary](ADR-0104-yandex-telemost-provider-runtime-boundary.md)
- [ADR-0105 Zulip Reference Provider and Макошь Lab](ADR-0105-zulip-reference-provider-and-makosh-lab.md)

## Recent Frontend ADRs

- [ADR-0172 Макошь UI Kit на базе shadcn-vue / Reka UI](ADR-0172-frontend-ui-kit-shadcn-vue.md)
- [ADR-0173 Frontend Capability Surfaces and Thin Vue Components](ADR-0173-frontend-capability-surfaces-and-thin-vue-components.md)

## Recent Domain ADRs

- [ADR-0174 Persona-Native Physical Identifier Columns](ADR-0174-persona-native-physical-identifier-columns.md)
- [ADR-0176 Mail Closure: Local-First Reconciliation](ADR-0176-mail-closure-local-first-reconciliation.md)
- [ADR-0177 Isolated Rich Document Extraction](ADR-0177-isolated-rich-document-extraction.md)
- [ADR-0181 Backend Workspace Modularity and Provider Runtime Topology](ADR-0181-backend-workspace-modularity-and-provider-runtime-topology.md)
- [ADR-0182 WhatsApp Hidden WebView Runtime Only](ADR-0182-whatsapp-hidden-webview-runtime-only.md)
- [ADR-0183 Backend API Cutover and Canonical Schema Reset](ADR-0183-backend-api-cutover-and-canonical-schema-reset.md)
- [ADR-0184 Backend Clean-Room Restart](ADR-0184-backend-clean-room-restart.md)
