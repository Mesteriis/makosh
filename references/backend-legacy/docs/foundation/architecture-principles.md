# Макошь Architecture Principles

## 1. Context Over CRUD

CRUD screens are implementation surfaces. The system value is the ability to
assemble context from evidence, relationships and time.

## 2. Local-First Memory

Private memory belongs under the user's local control. Cloud providers are
sources or optional integrations, not the durable memory layer.

## 3. Evidence Before Inference

Imported records, canonical events and user-reviewed records outrank generated
summaries and scores.

## 4. Events Explain Change

Meaningful changes must be explainable through append-only events or equivalent
source evidence. Projections and views must be rebuildable where practical.

## 5. Relationships Are First-Class

Relationships are records with type, provenance, confidence and validity. They
must not be hidden as ad hoc fields on unrelated entities.

## 6. Domains Own Entities

Domains own source-of-truth entities and lifecycle rules. Domains may consume
engines, but they must not duplicate engine ownership.

## 7. Engines Are Reusable Mechanisms

Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk and Consistency /
Contradiction are engines. They operate across domains and produce projections,
candidates, observations or scores.

## 8. AI Is Derived, Not Canonical

AI can summarize, classify, suggest links, propose tasks and detect risks. AI
does not mutate durable state directly and does not become the source of truth.

## 9. Provenance Is Mandatory

Every imported or AI-derived fact must be traceable to source evidence. If a
record cannot cite its source, it must be treated as incomplete.

## 10. Provider Boundaries Stay Explicit

Gmail, Telegram, WhatsApp, calendars and external task trackers are providers.
Their quirks stay in adapters and canonical observations, not in the core world
model.

## 11. Derived State Is Rebuildable

Search indexes, embeddings, graph views, dossiers, context packs and scores are
derived. Losing a derived artifact must not destroy memory.

## 12. One Term, One Meaning

Active documentation must use the glossary. If a term changes meaning, update
the glossary and the affected domain documents together.
