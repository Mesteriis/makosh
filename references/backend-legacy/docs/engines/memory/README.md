# Memory Engine

Status: documentation package aligned to the current repository structure.

The Memory Engine assembles durable, source-backed context across Макошь.

Memory is not a generic note store and not an AI summary. Memory is accepted,
reviewable understanding derived from evidence.

## Responsibilities

The Memory Engine produces:

- memory views;
- context packs;
- memory gaps;
- stale-memory candidates;
- source-backed summaries;
- recall inputs for agents.

It does not own:

- domain entities;
- raw communications;
- document versions;
- graph relationships;
- private-data fine-tuning.

## Inputs

- canonical events;
- accepted facts;
- relationship records;
- communications;
- documents;
- projects;
- tasks;
- decisions;
- obligations;
- owner-reviewed observations.

## Output Requirements

Every durable memory output must include:

- source citations;
- affected entities;
- confidence or review state;
- created or updated time;
- actor or process that produced it;
- invalidation or supersession behavior where relevant.

## Current Implementation Evidence

Memory behavior currently exists inside domain modules such as
`persons/memory.rs` and `organizations/memory.rs`, plus project and document
memory plans. This is acceptable during migration but should not be documented
as separate engines per domain.

The first backend Memory Engine baseline lives in `backend/src/engines/memory.rs`.
It converts deprecated Persona compatibility `persons.notes` text into a
source-backed Persona memory-card draft:

- title: `Compatibility notes`;
- description: trimmed notes text;
- source: `persons.notes:<persona_id>`;
- confidence: `1.0`;
- importance: `5`.

`PersonEnrichmentStore` uses this draft when materializing compatibility
`persona_memory_cards`. Empty notes remove the compatibility memory-card source
and do not create a new card.

The shared engine also builds source-backed accepted Persona fact drafts for
compatibility `persona_facts`. These drafts preserve affected entity, fact type,
value, source citation, confidence, accepted review state and the producing
process before the compatibility store writes the record.

The shared engine now also assembles bounded source-backed entity context packs
from accepted fact drafts and memory-card drafts. A context pack preserves the
affected entity, ordered memory items, deduplicated source citations, aggregate
confidence and producing process. It is a derived recall input, not a source of
truth.

The shared engine also detects required fact gaps for an entity. Gap outputs are
derived review candidates with affected entity, missing fact type, deterministic
source reference, suggested review state and producing process. They do not
create facts by themselves.

The shared engine also emits stale-memory candidates for accepted facts whose
verification timestamp is missing or older than a caller-provided threshold.
These candidates preserve source citation, confidence and last verification
time, and only request review. They do not decay confidence or overwrite the
fact directly.

## Migration Plan

1. Keep compatibility notes-to-card assembly in the Memory Engine.
2. Keep source-backed Persona fact normalization in the Memory Engine.
3. Keep source-backed context pack assembly in the Memory Engine.
4. Keep required fact gap detection in the Memory Engine.
5. Keep stale-memory candidate detection in the Memory Engine.
6. Keep domain-specific memory docs focused on owned source records.
7. Move reusable memory assembly language to this engine spec.
8. Preserve source citations and review state before expanding automation.
9. Treat AI summaries as derived observations until reviewed.
