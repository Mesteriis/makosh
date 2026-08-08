# Макошь Tasks

Status: documentation package aligned to the current repository structure.

Tasks are actionable units inside the Personal Memory System. Макошь is not a
task tracker. External trackers can be providers, but Макошь keeps local
context, evidence and relationships under owner control.

## Domain Boundary

A Task:

- has a lifecycle;
- has an owner/status;
- has source evidence or explicit manual provenance;
- can link to Personas, Organizations, Projects, Documents, Communications,
  Events, Decisions and Obligations.

A Task is not:

- every Obligation;
- every Follow-Up;
- a Project;
- an external issue copy without local context.

## Engine Use

- Obligation Engine creates obligation and task candidates from evidence.
- Risk Engine identifies blockers and attention signals.
- Search Engine finds task context.
- Timeline Engine shows task history.
- Memory Engine assembles task context packs.

## Current Implementation Surface

Implementation details, route counts and status live in:

- [API Reference](api.md)
- [Data Model](data-model.md)
- [Architecture](architecture.md)
- [Status](status.md)

## Navigation

- [Canonical Spec](./spec.md)
- [Architecture](./architecture.md)
- [API Reference](./api.md)
- [Data Model](./data-model.md)
- [Status](./status.md)
