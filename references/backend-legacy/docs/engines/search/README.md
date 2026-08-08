# Search Engine

Status: documentation package aligned to the current repository structure.

The Search Engine retrieves source-backed information across Макошь.

Search is an engine, not a domain. It operates over domain records, source
evidence, graph relationships and derived indexes.

## Responsibilities

The Search Engine produces:

- ranked search results;
- snippets;
- query plans;
- graph-expanded result sets;
- semantic recall candidates;
- source-backed answer contexts.

It does not own:

- source records;
- accepted memory;
- documents;
- communications;
- graph relationships.

## Inputs

- normalized text;
- document extraction output;
- communication text;
- event metadata;
- graph relationships;
- embeddings;
- source reliability signals;
- domain-specific ranking hints.

## Output Requirements

Search results must expose:

- source object;
- matched text or explanation;
- related entities;
- event time where known;
- confidence for inferred matches;
- ranking factors where feasible.

## Current Implementation Evidence

Current implementation includes `backend/src/engines/search.rs` and
`backend/src/domains/communications/search.rs`. Existing architecture also names Tantivy as
the full text search foundation.

## Migration Plan

1. Keep search indexes derived and rebuildable.
2. Consolidate search behavior under engine semantics.
3. Avoid making search results source of truth.
4. Preserve explainability for AI answer context.

## Navigation

- [Architecture](./architecture.md)
