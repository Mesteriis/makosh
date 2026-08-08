# Canonical Memory Architecture

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: memory, knowledge and context architecture. This document does not define
new tables, APIs or migrations.

## Purpose

Memory is the durable reason Макошь exists. The system should turn evidence into
source-backed context that helps the owner understand what happened, what
changed and what should be done.

## Responsibility

Memory architecture is responsible for:

- preserving evidence before inference;
- creating reviewable candidates from evidence;
- storing accepted domain truth in owning domains;
- building context across domains;
- detecting stale or contradictory memory;
- distinguishing durable memory from derived summaries;
- keeping AI output cited and reviewable.

## Boundary

Memory is not one table that owns everything. It is an architecture layer built
from:

- canonical observations;
- canonical events;
- domain records;
- Relationship records;
- Decisions and Obligations;
- reviewed facts and memory cards;
- engine observations;
- derived context views.

The Memory Engine assembles and retrieves memory. It does not own Personas,
Organizations, Communications, Documents, Tasks, Decisions or Obligations.

## Evidence-To-Memory Flow

```text
Source evidence
  -> canonical observation
  -> canonical event or import record
  -> domain projection
  -> extraction candidate
  -> review or policy acceptance
  -> owning domain record / reviewed memory
  -> relationships and graph projection
  -> context pack / dossier / timeline / search result
```

## Memory States

| State | Meaning | Owner |
|---|---|---|
| Source evidence | Preserved provider/local record. | Provider/import boundary. |
| Candidate | Proposed fact, link, task, obligation, decision or observation. | Producing engine/domain until reviewed. |
| Suggested | Reviewable but not accepted truth. | Producing owner. |
| Accepted | Durable source-backed domain truth or reviewed memory. | Owning domain or memory policy. |
| Rejected | Preserved review decision that prevents silent resurrection. | Candidate/observation owner. |
| Superseded | Previously accepted memory replaced by later accepted evidence. | Owning domain/memory policy. |
| Derived | View, summary, index, score or context pack. | Engine/projection. |

## Knowledge Boundary

Knowledge is evidence-backed understanding across domains. It should not become
a generic wiki silo that duplicates Documents, Relationships, Decisions,
Obligations or Memory.

Accepted knowledge must answer:

- what is known;
- what source supports it;
- what entity it is about;
- when it was observed;
- whether it was reviewed;
- what supersedes or contradicts it.

Open issue: the exact storage owner for generic reviewed Knowledge Items needs
a future ADR before new tables or domain code are created.

## Consistency / Contradiction

The Consistency / Contradiction Engine, user-facing alias Polygraph, compares
new evidence against accepted memory.

It may create:

- direct contradiction observations;
- stale fact warnings;
- disputed claim observations;
- conflicting decision or obligation observations.

It must not:

- label a Persona as dishonest;
- overwrite memory automatically;
- mutate another domain without review or explicit policy;
- hide old and new source references.

## Connections

| Source | Memory connection |
|---|---|
| Communications | Main intake for interaction evidence, obligations, decisions, relationship signals and context. |
| Documents | Durable artifacts and extracted content. |
| Calendar/Events | Time-bound evidence, meetings and timeline anchors. |
| Personas | Subject memory anchors and Owner Persona context. |
| Organizations | Collective actor memory and procedures. |
| Projects | Bounded context and linked work history. |
| Relationships | Semantic structure for memory traversal. |
| Tasks | Executable actions derived from memory and obligations. |
| Decisions | Remembered choices and rationale. |
| Obligations | Commitments and duties. |
| Agents | Proposed summaries/actions with citations and audit. |

## Reasons For Existence

Without a memory architecture, Макошь would be a set of provider clients and
CRUD surfaces. Memory turns raw evidence into durable understanding while
preserving uncertainty, review and provenance.
