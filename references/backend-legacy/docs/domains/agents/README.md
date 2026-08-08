# Agents Domain

Status: documentation package aligned to the current repository structure.

Agents are tool-mediated actors that help the Owner Persona operate Макошь.

Agents are not source-of-truth owners. They work through permissions, audit
records, source evidence and reviewable actions.

## Responsibilities

The Agents domain owns:

- agent identity;
- capability policy;
- tool access boundaries;
- action audit trails;
- agent run records;
- proposed actions;
- owner approvals and denials;
- agent-to-Persona representation where needed.

The Agents domain does not own:

- private data truth;
- domain entity state outside approved actions;
- provider credentials;
- automatic source-of-truth overwrites;
- fine-tuning on private data.

## Agent Persona

AI agents can exist in the Persona graph as:

```yaml
PersonaType: ai_agent
```

This allows Макошь to represent HESTIA and future agents as actors with
relationships, permissions and provenance. The Owner Persona remains the only
`is_self: true` Persona.

## Agent Workflow

```text
context request
  -> permission check
  -> retrieval from domains and engines
  -> proposed observation/action
  -> review or policy gate
  -> audited domain event
```

Agents must cite source records when producing conclusions.

## Current Implementation Evidence

Current implementation includes:

- AI runtime and control center migrations `0018` and `0057`;
- Ollama integration;
- AI-related route groups;
- settings and capability infrastructure;
- frontend Agents and Settings surfaces.

The agent domain is partially implemented as runtime/control infrastructure, but
the product-level agent model still needs explicit capability and Persona graph
documentation before broader automation.

## Migration Plan

1. Keep agents permissioned and audited.
2. Represent durable AI actors as `PersonaType: ai_agent` when they need graph
   identity.
3. Require cited context for agent conclusions.
4. Keep private data out of fine-tuning.
5. Add ADRs before allowing autonomous write behavior beyond narrow approved
   policies.
