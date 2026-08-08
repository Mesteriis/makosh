# Product Charter

## Purpose

Макошь creates a personal operational memory layer around communications,
documents, projects, relationships, decisions and obligations. The product helps
the owner understand what happened, what matters, what requires action and how
entities connect.

See the canonical foundation documents:

- [Foundation Vision](../foundation/vision.md)
- [World Model](../foundation/world-model.md)
- [Architecture Principles](../foundation/architecture-principles.md)

## User

The primary user is one technically strong owner who manages personal and
professional communications, documents, projects, relationships, obligations and
knowledge. Макошь is a personal system first; architecture should not block
future family/team modes, but those modes are not the current product identity.

The owner is represented inside the world model by the Owner Persona.

## Core Scenarios

- unified communication context across channels;
- extraction and tracking of obligations and task candidates;
- source-backed search across memory;
- history of relationships with a Persona or Organization;
- linking documents to Projects, Personas, Organizations, Events, Tasks,
  Decisions and Obligations;
- AI-assisted triage with user control;
- analysis of changes over time;
- context preparation before meetings or actions;
- explanation of why Макошь produced a conclusion.

## Product Constraints

- The system is not optimized for a quick MVP.
- Implementation may be incremental, but documentation should describe the
  target model.
- Cloud providers are optional integrations, not the memory layer.
- Personal data is not used for fine-tuning.
- AI features must degrade safely when a model is unavailable.
- Every automatic conclusion must preserve provenance.

## Product Quality

Макошь should feel like a serious personal operating environment, not a
dashboard collection. The UI should be fast, dense, keyboard-first and
contextual.

## Quality Metrics

- ingestion completeness for connected sources;
- identity resolution quality for Personas and Organizations;
- context retrieval latency;
- accuracy of obligation/task extraction;
- share of AI answers with sufficient provenance;
- backup/restore success;
- manual actions required per common workflow.
