# Макошь Engines

Engines are reusable mechanisms used by domains. They are not domains and do not
own primary entities.

Detailed engine specs live under [Engine Catalog](../engines/README.md).

## Engine Map

| Engine | Purpose | Outputs | Uses |
|---|---|---|---|
| Memory Engine | Preserve and retrieve durable context. | memory records, context summaries, memory gaps | events, relationships, documents, communications, tasks |
| Timeline Engine | Build chronological views across entities. | timelines, diffs, period summaries | events and dated domain records |
| Trust Engine | Assess relationship and source reliability. | trust signals, confidence adjustments | provenance, relationship history, review outcomes |
| Search Engine | Retrieve source-backed information. | ranked results, snippets, query plans | full text, vectors, graph, source metadata |
| Enrichment Engine | Propose new candidate knowledge. | enrichment candidates, conflicts, observations | approved public/local sources and provider records |
| Obligation Engine | Detect and track commitments. | obligations, follow-ups, task candidates | communications, meetings, documents, decisions |
| Risk Engine | Detect evidence-backed risks. | risk observations, attention signals | tasks, projects, organizations, relationships, obligations |
| Consistency / Contradiction Engine | Detect conflicts between new evidence and accepted memory. | contradiction observations, stale fact warnings, review items | communications, documents, events, decisions, obligations, knowledge |
| Automation Engine | Evaluate owner-approved automation policies and dry-runs. | dry-run results, policy decisions, automation command metadata | templates, policies, provider capabilities, source context |
| Context Packs Engine | Build rebuildable context bundles. | context packs, source links | observations, domains, knowledge, relationships, decisions |
| Identity Resolution Engine | Propose same-entity candidates. | identity candidates | observations, identity traces, source evidence |
| Relationship Candidate Engine | Propose entity relationship candidates. | relationship candidates | observations, graph evidence, source-backed domain records |

## Memory Engine

The Memory Engine assembles source-backed memory across domains. It does not own
Personas, Organizations, Documents or Tasks. It uses their records to build
memory views and identify gaps.

Required properties:

- provenance-first;
- reviewable uncertainty;
- no private-data fine-tuning;
- rebuildable projections where possible.

## Timeline Engine

The Timeline Engine produces chronological views from canonical events and dated
domain records.

It is used by:

- Personas;
- Organizations;
- Projects;
- Documents;
- Communications;
- Tasks;
- Decisions;
- Obligations.

Timeline output is derived. The event log and domain records remain source of
truth.

## Trust Engine

The Trust Engine computes trust and reliability signals. Trust belongs primarily
to Relationships and source reliability, not as a generic root field on every
entity.

Inputs include:

- provenance;
- confirmed or rejected suggestions;
- fulfilled or broken obligations;
- source consistency;
- relationship history.

## Search Engine

The Search Engine combines:

- full text search;
- semantic retrieval;
- graph expansion;
- event/time filters;
- source reliability;
- domain-specific ranking hints.

Indexes are derived and rebuildable.

## Enrichment Engine

The Enrichment Engine proposes new information. It must not silently overwrite
domain truth.

Outputs are:

- candidates;
- observations;
- conflicts;
- reviewable updates.

Approved enrichment sources and policy boundaries are defined by the relevant
domain and ADR.

## Obligation Engine

The Obligation Engine detects commitments, duties and expected actions.

An Obligation may produce:

- a Task;
- a Follow-Up;
- a timeline event;
- a risk observation.

The engine must preserve the source quote or source reference that created the
obligation.

## Risk Engine

The Risk Engine detects evidence-backed conditions that may require attention.

Risk output is not a free-form warning. It must include:

- affected entity;
- risk type;
- source evidence;
- confidence;
- suggested handling state.

Domains decide how risk affects lifecycle, UI and automation.

## Consistency / Contradiction Engine

The Consistency / Contradiction Engine compares new evidence with accepted
Memory and Knowledge. Its user-facing alias is Polygraph.

It detects:

- direct contradictions;
- stale facts;
- disputed claims;
- conflicting decisions;
- mismatched obligations;
- claims that weaken existing trust assumptions.

It must not decide that a person is lying and must not silently overwrite
domain truth. It creates source-backed contradiction observations and review
items.

Required output properties:

- old source reference;
- new source reference;
- affected entities;
- conflict type;
- confidence;
- review state.

## Engine Boundary Rule

Do not create separate copies of engines inside domains.

Wrong:

```text
Persona Timeline
Project Timeline
Document Timeline
Organization Timeline
```

Correct:

```text
Timeline Engine
  used by Personas, Projects, Documents and Organizations.
```
