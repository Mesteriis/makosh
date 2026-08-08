# Product Roadmap

This roadmap uses current foundation terminology. Older implementation
milestones that mention person/contact storage should be read as compatibility
records feeding the Persona model.

## Version 0.1 - Architectural Foundation

Goals:

- establish documentation, ADRs and domain boundaries
- define event, graph, storage, search and agent architecture
- create implementation-ready repository structure

Key functions:

- no runtime product functions
- documentation and design assets only

Architectural changes:

- monorepo skeleton
- initial ADR set
- architecture map

Risks:

- overdesign without later validation
- missing provider-specific constraints

Dependencies:

- review of provider APIs
- storage proof-of-concepts in later phases

## Version 1.0 - Local Memory Core

Goals:

- establish local backend, storage and desktop shell
- ingest first communication source
- build event log, projections and full text search

Key functions:

- local app setup
- PostgreSQL persistence
- event ingestion pipeline
- fixture and read-only provider email import
- local account setup for Gmail, iCloud and raw IMAP
- basic Persona-compatible identity projection
- full text search
- document import for Markdown/PDF

Architectural changes:

- Rust backend foundation
- SvelteKit/Tauri shell
- Tantivy index
- event envelope implementation

Risks:

- production secret backup/recovery and optional OS keychain hardening
- projection replay complexity
- local install and migration UX

Dependencies:

- Rust service architecture
- database migration strategy
- secret storage decision

## Version 2.0 - Knowledge Graph and Documents

Goals:

- make graph-backed memory central
- support richer documents and identity resolution
- connect Communications, Personas, Projects and Documents

Key functions:

- first graph core projection from Persona-compatible identity records, messages and documents
- graph relationships with provenance
- Persona identity merge/split
- document OCR and extraction
- project timeline views
- task candidates from messages and documents

Architectural changes:

- graph schema
- document artifact pipeline
- projection replay tools
- confidence and review workflows

Risks:

- false entity merges
- OCR quality variation
- graph UI complexity

Dependencies:

- document processing engine
- entity extraction evaluation
- graph query patterns

Closure tracking:

- [V2 Closure Checklist](v2-closure-checklist.md)

## Version 3.0 - AI Native Workflows

Goals:

- integrate local agents into daily workflows
- support source-backed analysis and action suggestions
- make AI available inside communication, document, task and graph surfaces

Key functions:

- HESTIA coordinator
- MAKOSH communication agent
- MNEMOSYNE memory agent
- ATHENA analytics agent
- source-backed AI search answers
- task extraction review
- meeting preparation
- future Rust-native embedded small-model backend for offline translation,
  categorization and lightweight extraction through AI Hub routes

Architectural changes:

- agent runtime
- tool permission model
- Ollama provider
- embedding provider
- embedded local model runtime research and ADR before implementation
- prompt provenance logging

Risks:

- prompt injection
- hallucinated links
- latency on local models
- embedded runtime memory, model quality and packaging complexity

Dependencies:

- local AI model evaluation
- embedded inference runtime selection
- permission model
- graph/search retrieval planner

Closure tracking:

- [V3 Closure Checklist](v3-closure-checklist.md)

## Version 4.0 - Automation, Plugins and Channel Foundation

Goals:

- expand Telegram provider depth and controlled automation
- introduce plugin host
- harden privacy, security and backup

Key functions:

- Telegram integration
- plugin manifest and capability model
- backup/restore
- automation policies
- advanced spam and relevance scoring

Architectural changes:

- plugin runtime
- provider abstraction hardening
- backup verifier
- policy engine

Risks:

- provider API instability
- plugin security
- automation side effects

Dependencies:

- Telegram adapter research
- capability sandbox design
- backup encryption model

Closure tracking:

- [V4 Closure Checklist](v4-closure-checklist.md)

## Version 5.0 - Long-Term Personal Knowledge OS

Goals:

- mature Макошь into a durable personal knowledge operating system
- support deep memory analytics and explainable recall across years
- make replacement of models and indexes routine

Key functions:

- WhatsApp Web companion integration
- optional WhatsApp Business Platform provider research
- optional SMS integration
- cross-year analytics
- decision history
- relationship evolution
- advanced project memory
- structured exports
- index/model replacement tooling
- mature observability and evaluation

Architectural changes:

- long-horizon retention policies
- advanced graph analytics
- model/index migration workflows
- comprehensive evaluation suites

Risks:

- accumulated data quality debt
- performance at multi-year scale
- UX complexity

Dependencies:

- production-scale datasets
- WhatsApp Web companion runtime validation
- search and graph benchmarking
- long-term backup/restore testing

Closure tracking:

- [V5 Closure Checklist](v5-closure-checklist.md)
