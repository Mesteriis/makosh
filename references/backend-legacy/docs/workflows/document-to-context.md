# Document To Context

This workflow explains how a document becomes useful context.

## Trigger

The workflow starts when Макошь imports, creates or updates:

- PDF;
- Office document;
- image;
- Markdown document;
- lightweight note;
- attachment promoted to document evidence.

## Flow

```text
document version
  -> preserve artifact and metadata
  -> extract text and structure
  -> classify document type
  -> extract entities, claims and dates
  -> link candidate Personas, Organizations, Projects and Tasks
  -> check contradictions
  -> create context candidates
  -> review or policy gate
  -> update accepted memory or graph links
```

## Required Outputs

- immutable document version;
- extraction artifacts;
- entity and relationship candidates;
- knowledge candidates;
- contradiction observations when conflicts exist;
- context links to owning domains.

## Domain And Engine Boundaries

- Documents owns artifacts, versions and extraction outputs.
- Knowledge Graph owns accepted relationship records.
- Memory Engine assembles context packs.
- Search Engine indexes derived text.
- Consistency / Contradiction Engine detects conflicts with accepted memory.

## Current Implementation Evidence

Documents and document processing exist. Notes are currently document-like
artifacts. Attachment intelligence exists under the Documents domain.

## Migration Plan

1. Keep extracted output derived until reviewed.
2. Preserve document version immutability.
3. Link documents to Projects, Personas, Organizations, Tasks, Decisions and
   Obligations through graph relationships.
4. Avoid treating document summaries as source truth.
