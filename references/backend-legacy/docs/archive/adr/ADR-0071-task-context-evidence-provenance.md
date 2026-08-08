# ADR-0071 Task Context and Evidence Provenance

Status: Proposed

## Context

Макошь Tasks must track where each task came from, why it exists, and what context surrounds it. AI-extracted tasks from emails, meetings, and documents need evidence provenance. Tasks need a materialized context pack for instant retrieval.

## Decision

Task Context Pack is a materialized JSONB snapshot containing summary, open questions, blockers, risks, and suggested next action. Task Evidence stores `source_type`, `source_id`, `quote`, and `confidence` for AI-extracted tasks. Low-confidence tasks route to the suggested inbox for user review. All facts carry `source` and `confidence` fields.

## Consequences

- Every AI-generated task has verifiable evidence.
- Context packs enable instant task understanding without cross-domain joins.
- User review flow prevents low-confidence tasks from polluting the active task list.
