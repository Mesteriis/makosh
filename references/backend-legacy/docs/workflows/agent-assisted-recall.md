# Agent Assisted Recall

This workflow explains how an agent retrieves and uses Макошь context.

Agents help the Owner Persona operate the Personal Memory System. They do not
own source truth.

## Trigger

The workflow starts when the owner asks an agent for help, or when an approved
policy allows a scoped agent action.

## Flow

```text
owner or policy request
  -> identify acting agent Persona
  -> check capabilities
  -> retrieve context through domains and engines
  -> cite sources
  -> propose answer, observation or action
  -> review or policy gate
  -> write audited event if action is accepted
```

## Required Outputs

- acting agent identity;
- capability decision;
- retrieved context with citations;
- proposed answer or action;
- review or policy result;
- audit event for accepted side effects.

## Domain And Engine Boundaries

- Agents own run and permission records.
- Domains own source-of-truth updates.
- Search and Memory Engines provide context.
- Risk, Trust and Contradiction engines provide signals.
- Owner Persona remains the system owner.

## Current Implementation Evidence

Current implementation includes AI runtime/control center, Ollama integration,
settings and capability infrastructure. Product-level agent behavior still needs
more explicit permission and Persona graph documentation before broader
automation.

## Migration Plan

1. Require capabilities for side effects.
2. Keep retrieved context cited.
3. Treat agent conclusions as proposed observations unless reviewed.
4. Audit accepted actions.
5. Represent durable agents as `PersonaType: ai_agent` when graph identity is
   needed.
