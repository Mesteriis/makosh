# Макошь Workflow Catalog

Workflows describe how evidence moves through Макошь.

They are not APIs and not implementation modules. They define product behavior
and architectural boundaries that future implementation plans must respect.

## Core Principle

Communication is the primary ingestion spine:

```text
Communication
  -> Evidence
  -> Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
```

Documents, meetings, calls and notes can also produce evidence, but
Communications are the most common entry point.

## Workflow Specs

| Workflow | Spec |
|---|---|
| Communication to Knowledge | [Communication To Knowledge](communication-to-knowledge.md) |
| Communication to Obligation | [Communication To Obligation](communication-to-obligation.md) |
| Meeting to Decisions | [Meeting To Decisions](meeting-to-decisions.md) |
| Document to Context | [Document To Context](document-to-context.md) |
| Contradiction Review | [Contradiction Review](contradiction-review.md) |
| Dossier Generation | [Dossier Generation](dossier-generation.md) |
| Agent Assisted Recall | [Agent Assisted Recall](agent-assisted-recall.md) |

## Boundary Rule

Workflows coordinate domains and engines. They do not own durable entities.
Durable state must be written by the owning domain or as a reviewed engine
observation.
