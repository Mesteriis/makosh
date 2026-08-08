# Projects Domain

Status: documentation package aligned to the current repository structure.

Projects are bounded work contexts that connect communications, documents,
tasks, decisions, obligations, Personas, Organizations and events.

Макошь is not a project management tool. A Project is a context boundary inside
the Personal Memory System.

## Responsibilities

The Projects domain owns:

- project identity and lifecycle state;
- project goals and scope;
- project context pack;
- links to related entities;
- project-specific decisions;
- project evidence;
- review state for candidate links;
- project timeline view through the Timeline Engine.

The Projects domain does not own:

- Organization identity;
- Task lifecycle;
- Communication source records;
- document versioning;
- global graph traversal;
- global memory.

## Project Versus Organization

An Organization is a durable collective actor. A Project is a bounded work
context.

An organization can sponsor or participate in many projects. A project can
involve many organizations. Neither entity should be modeled as a field of the
other.

## Project Versus Task

A Task is a concrete actionable unit with lifecycle. A Project is a larger
context that may contain many tasks, documents, communications, decisions and
obligations.

Tasks can be linked to Projects, but project state must not be derived only from
task status.

## Project Context Pack

A project context pack should include:

- summary;
- goals;
- current state;
- open obligations;
- active tasks;
- key decisions;
- important documents;
- recent communications;
- involved Personas and Organizations;
- risk and blocker observations;
- source citations.

The context pack is a derived read model, not the source of truth.

## Current Implementation Evidence

Current backend implementation includes:

- `backend/src/domains/projects/core.rs`;
- `backend/src/domains/projects/link_reviews.rs`;
- migrations `0013_create_projects_and_extend_graph.sql` and
  `0014_create_project_link_reviews.sql`;
- `/api/v1/projects/*` route registration;
- Projects frontend page.

The implementation exists, but domain documentation was incomplete compared to
Personas, Organizations and Tasks.

## Migration Plan

1. Make this document the canonical Projects domain description.
2. Expand future architecture/data-model docs if implementation changes require
   deeper detail.
3. Keep candidate links reviewable and provenance-backed.
4. Define Decision and Obligation links before introducing automated project
   health conclusions.
5. Avoid treating Projects as external issue trackers or CRM opportunities.
