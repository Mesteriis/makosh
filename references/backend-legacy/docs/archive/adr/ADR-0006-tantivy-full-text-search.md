# ADR-0006 Tantivy Full Text Search

Status: Proposed

## Context

Макошь requires fast local full text search over messages, documents, tasks, contacts and projects. The backend target language is Rust.

## Decision

Use Tantivy for full text search.

## Consequences

- Search can run locally without cloud dependencies.
- Rust integration is strong.
- Indexes must be treated as derived and rebuildable.
- Query planning must combine Tantivy results with graph and semantic retrieval.
