# Canonical Agents Architecture

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: agent architecture and boundaries. This document does not enable new
autonomous writes, provider side effects or plugin execution.

## Purpose

Agents help the Owner Persona operate Макошь. They retrieve context, explain
evidence, propose actions and execute approved tool workflows. They do not own
truth.

## Responsibility

The Agents domain owns:

- agent identity;
- agent run records;
- tool access boundaries;
- capability and confirmation integration;
- proposed observations and actions;
- owner approvals and denials;
- audit trail for tool-mediated actions;
- agent-to-Persona representation when an agent needs graph identity.

## Boundaries

Agents do not own:

- source evidence;
- domain entity state;
- Memory truth;
- Knowledge truth;
- provider credentials;
- task, decision or obligation lifecycle;
- automatic contradiction resolution;
- model training on private data.

Agent output remains derived until an owning domain accepts it under domain
rules.

## Agent Persona Model

When represented in the world model, an AI agent is a Persona:

```yaml
PersonaType: ai_agent
```

The Owner Persona remains the only `is_self = true` Persona. Agents act on
behalf of the owner only through explicit capability and audit boundaries.

## Agent Workflow

```text
Owner intent or system trigger
  -> capability check
  -> context retrieval
  -> cited reasoning output or proposed action
  -> review / confirmation / scoped policy
  -> owning domain command
  -> audit and event evidence
```

## Tool And Capability Rules

Agents must:

- cite source evidence for factual claims;
- distinguish source content from generated text;
- treat imported messages and documents as untrusted input;
- request backend capability checks before side effects;
- keep provider writes, destructive actions, exports, recording and secret
  access behind confirmation or scoped policy;
- write audit metadata without private message bodies, document contents or
  secrets.

Agents must not:

- choose provider destinations from retrieved content alone;
- silently send, delete, export or record;
- mutate durable domain state through direct storage access;
- use private data for fine-tuning;
- hide uncertainty.

## Connections

Agents consume:

- Memory Engine context;
- Search results;
- Timeline views;
- Relationship graph context;
- Communications evidence;
- Documents and extracted text;
- Decisions and Obligations;
- Tasks and Projects;
- Radar/review candidates if that layer becomes accepted.

Agents write through:

- owning domain commands;
- capability runtime;
- event log;
- audit log;
- proposal/review records.

## Reasons For Existence

Макошь can contain more context than the owner can manually inspect. Agents are
useful when they can assemble cited context and propose next actions without
becoming an uncited source of truth or an unsafe automation layer.
