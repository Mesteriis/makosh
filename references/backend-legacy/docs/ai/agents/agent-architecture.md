# Agent Architecture

## Agent System Goal

Agents are specialized application actors that help classify, connect,
summarize, search and act on the owner's memory. They are not the source of
truth. They operate through typed tools, explicit permissions and source-backed
context.

When represented in the world model, agents are Personas with
`persona_type = ai_agent`.

The backend AI agent registry materializes current agents as compatibility
Personas when `/api/v1/ai/agents` is requested with a configured database. These
Personas use stable `persona:v1:ai_agent:<AGENT_ID>` IDs and compatibility email
identities in the form `<agent>@sh-inc.ru`, for example `hestia@sh-inc.ru`.
The compatibility Persona display name uses the same email-form identity.
AI run records also store `agent_persona_id` and the current
`owner_persona_id` when an Owner Persona exists, so agent actions can be traced
through both the acting agent Persona and the Owner Persona.

## Initial Agents

| Agent | Role |
|---|---|
| HESTIA | coordinator, intent routing, policy mediation |
| MAKOSH | communications triage, threads, drafting, channel context |
| MNEMOSYNE | memory, graph linking, recall, provenance |
| ATHENA | analysis, trend detection, decision support |
| HEPHAESTUS | development, maintenance and tool automation |

## Agent Runtime

```mermaid
flowchart LR
    UserIntent["Owner intent or system event"] --> Hestia["HESTIA"]
    Hestia --> Planner["Plan"]
    Planner --> Policy["Policy and capability check"]
    Policy --> Tools["Typed tools"]
    Tools --> Context["Context from domains and engines"]
    Tools --> External["External providers if permitted"]
    Tools --> Audit["Agent audit events"]
    Context --> Response["Source-backed response"]
    Audit --> Response
```

## Tool Contract

Agent tools must define:

- input schema;
- output schema;
- permissions;
- data access scope;
- side-effect class;
- timeout;
- audit behavior;
- error model.

## Context Use

Agents retrieve context from:

- graph queries;
- Search Engine results;
- Timeline Engine views;
- Memory Engine outputs;
- document extracts;
- Task, Project, Persona and Organization projections.

Agents must distinguish source facts, inferred links and generated summaries.

## Side Effects

Safe read-only actions may execute automatically inside an approved workflow.
External writes, message sending, deletion, provider changes and sensitive
exports require explicit confirmation and audit events.
