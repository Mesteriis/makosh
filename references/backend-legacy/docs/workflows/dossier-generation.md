# Dossier Generation

This workflow explains how Макошь assembles a dossier for a Persona,
Organization, Project or other entity.

## Trigger

The workflow starts when a user or agent requests an entity context view, or when
a background process refreshes a derived read model.

## Flow

```text
entity request
  -> collect identity and relationships
  -> collect accepted memory
  -> collect recent timeline
  -> collect documents, communications, tasks and decisions
  -> collect risk, trust and contradiction observations
  -> assemble cited dossier
  -> expose read model
```

## Persona Dossier Fields

- summary;
- interests;
- projects;
- organizations;
- skills;
- communication patterns;
- ai_observations;
- open obligations;
- recent timeline;
- unresolved contradictions.

## Required Outputs

- dossier summary;
- cited sections;
- freshness metadata;
- confidence or review markers;
- unresolved gaps and contradictions.

## Domain And Engine Boundaries

- Domains own source records.
- Memory Engine assembles durable memory.
- Timeline Engine provides chronological context.
- Trust and Risk Engines provide signals.
- Consistency / Contradiction Engine provides unresolved conflict observations.

## Current Implementation Evidence

Persona and Organization memory/dossier-like concepts exist. A single
cross-domain dossier workflow is not yet implemented.

## Migration Plan

1. Keep dossiers as derived read models.
2. Require citations in every dossier section.
3. Avoid using AI observations as accepted truth without review.
4. Add entity-specific dossier specs only when they reuse this workflow.
