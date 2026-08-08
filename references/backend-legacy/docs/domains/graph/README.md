# Knowledge Graph Design

Status: documentation package aligned to the current repository structure.

## Purpose

The knowledge graph represents durable relationships between Макошь world-model
entities. It is the primary substrate for relationship-aware memory and context.

The graph is not a generic visualization feature. It stores relationship records
with provenance, confidence and review state.

## Core Entities

- Persona.
- Organization.
- Project.
- Document.
- Communication.
- Event.
- Task.
- Decision.
- Obligation.
- Location.
- ChannelAccount.
- Attachment.

## Relationship Objects

Relationships are first-class records, not anonymous edges. A relationship must
store:

- source entity;
- target entity;
- relationship type;
- directionality where relevant;
- confidence;
- provenance;
- valid time range where relevant;
- created-by source: owner, ingestion, agent, rule or import;
- review state where inferred.

## Relationship Examples

- `persona_member_of_organization`
- `persona_participated_in_event`
- `communication_mentions_project`
- `document_related_to_project`
- `task_created_from_communication`
- `decision_made_in_event`
- `organization_related_to_project`
- `obligation_derived_from_communication`

## Identity Resolution

The graph must support uncertain identity:

- multiple digital traces per Persona;
- provider-specific usernames;
- phone numbers;
- aliases;
- merged and split identities;
- confidence-scored candidates.

Automatic merges are risky. High-confidence suggestions may be staged, but user
review must exist for ambiguous identities.

## Provenance

Every inferred entity or relationship must link back to evidence:

- source record ID;
- communication ID;
- document version;
- event ID;
- extraction run;
- agent run;
- manual owner action.

Graph answers without provenance are incomplete.

## Engine Boundary

The graph is a domain/projection boundary for relationships. Search, Timeline,
Trust, Risk and Memory engines may use graph relationships, but they do not own
the relationship source of truth.
