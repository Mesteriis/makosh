# Макошь Calendar And Events

Status: documentation package aligned to the current repository structure.

Calendar is the scheduled-event channel inside Макошь. Макошь is not a calendar
app. Calendar data becomes Events, Communications, Decisions, Obligations,
Tasks, Documents and Relationships in the Personal Memory System.

## Key Capabilities

- multi-provider account/source model;
- local and provider-backed scheduled events;
- event relationships to Personas, Organizations, Projects, Documents, Tasks,
  Communications, Decisions and Obligations;
- meeting preparation as context assembly;
- meeting outcomes as source-backed Decisions, Obligations and Task candidates;
- deadlines and focus blocks as event/context records;
- scheduling support under explicit policy.

## Engine Use

Calendar uses shared engines:

- Timeline Engine for chronological views;
- Memory Engine for meeting context;
- Obligation Engine for commitments and follow-ups;
- Risk Engine for preparation gaps and load risks;
- Search Engine for recall;
- Enrichment Engine for candidate links.

## Navigation

- [API Reference](api.md)
- [Data Model](data-model.md)
- [Architecture](architecture.md)
- [Status and Blockers](status.md)
