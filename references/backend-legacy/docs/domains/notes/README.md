# Notes Boundary

Status: documentation package aligned to the current repository structure.

Notes are lightweight capture artifacts in the current Макошь model.

They are not a first-class domain unless a future ADR promotes them.

## Current Definition

A Note is a document-like artifact or memory input that may contain:

- owner-written text;
- meeting notes;
- quick observations;
- pasted evidence;
- draft thinking;
- temporary capture.

Notes can become evidence for Knowledge, Tasks, Decisions, Obligations,
Projects, Personas or Organizations after review or linking.

## Boundary

Notes do not own:

- durable truth;
- global knowledge;
- task lifecycle;
- document versioning beyond the Documents domain rules;
- memory state without review.

## Current Implementation Evidence

The frontend contains a Notes surface, but the backend domain list does not
include a dedicated notes module. Existing document documentation treats
lightweight notes as document-like artifacts.

## Migration Plan

1. Continue treating Notes as document-like artifacts.
2. Do not introduce a Notes domain in documentation without an ADR.
3. If Notes become first-class later, define how they differ from Documents,
   Knowledge and Memory.
4. Keep note-derived facts reviewable and evidence-backed.
