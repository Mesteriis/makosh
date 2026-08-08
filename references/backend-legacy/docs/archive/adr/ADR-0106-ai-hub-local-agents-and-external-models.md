# ADR-0106: AI Hub, local Rust agents and external model boundary

Status: Accepted

Date: 2026-07-08

## Context

Макошь needs AI for translation, classification, summarization, reply drafting, extraction,
embeddings and intelligence workflows.

The codebase had a provider-neutral runtime abstraction, but some AI consumers still called
runtime methods directly from AI services, domains and workflows. That made the boundary too
weak: provider routing, safety inspection and model capability selection were spread across
call sites instead of flowing through one hub.

Макошь also needs a split between:

- small local agents that run inside Rust and handle cheap deterministic inspection;
- larger local or external models that handle expensive language tasks.

Those are not the same problem, no matter how bravely a single abstraction tries to pretend.

## Decision

Introduce `backend/src/ai/hub.rs` as the single AI application boundary.

Domains, workflows and AI services depend on `SharedAiHub`, not direct provider runtime clients.
Provider adapters remain below the hub:

```text
business caller
↓
AI Hub
↓
platform AI runtime port
↓
integration runtime adapter
```

The hub supports two tiers.

### Tier 1: local Rust agents

Local agents are deterministic and run inside the backend process.

They are used for:

- language hints;
- sensitive content detection;
- password-like assignments;
- private key and certificate markers;
- API-key and token shapes;
- JWT shape detection;
- high-entropy token detection;
- IBAN and payment card heuristics.

Local results must include source, confidence and bounded evidence.

### Tier 2: routed model calls

Larger model calls are routed by capability:

- default chat;
- reasoning;
- summarization;
- mail intelligence;
- reply drafting;
- extraction;
- embeddings;
- meeting preparation.

The hub resolves a capability route to the configured model from AI Hub routing.
Configured workspaces use explicit routes; missing or unavailable routes fail fast instead of
silently falling back to legacy chat or embedding defaults.

## Consequences

Positive:

- one boundary for AI calls;
- provider adapters stay out of domains and workflows;
- local sensitive-content guards can run before external inference;
- model routing is capability-based instead of scattered by provider/model name;
- AI output remains candidate data, not accepted domain truth.

Negative:

- call sites must accept `SharedAiHub` instead of direct runtime ports;
- the hub becomes a required dependency for new AI features;
- local guard coverage is rule-based until stronger embedded models or rule packs are added.

## Rules

1. New AI inference entry points go through `AiHub`.
2. Domains and workflows must not call `AiRuntimeClient` directly.
3. Provider credentials and protocol behavior stay in integrations/platform.
4. Sensitive content inspection must be available before external inference.
5. AI results are suggestions/candidates until the owning domain records them.
6. Evidence for sensitive findings must be masked or bounded.

Additional clarification:

- Synchronous workflow/domain inference through `AiHub` is allowed in the
  orchestration layer when immediate response is required. Event-based execution
  remains required for public API flows that expose user-visible async status,
  trace/audit handoff, or when durable queueing is part of the contract.

For the current public AI API this means:

- `POST /api/v1/ai/answers`
- `POST /api/v1/ai/task-candidates/refresh`
- `POST /api/v1/ai/meeting-prep`

return `202 Accepted` with `AiHubRequestAcceptedResponse`, append
`ai.hub.requested`, and publish completion or failure through
`ai.hub.completed` / `ai.hub.failed` while the durable result remains the
`ai_agent_runs` projection exposed by `GET /api/v1/ai/runs/{run_id}`.

For Hub settings model lifecycle, `POST /api/v1/ai/model-downloads` is the
explicit built-in Ollama download command. It appends
`ai.hub.model_download.requested` and then
`ai.hub.model_download.completed` or `ai.hub.model_download.failed`. Download
completion only makes the model available; route assignment stays explicit.

## Follow-up work

- Add explicit redaction policy for high and critical sensitivity findings.
- Add a local embedded model backend for small offline categorization if deterministic rules are insufficient.
- Add architecture checks that reject direct runtime inference calls outside the hub and provider adapters.
- Persist local inspection provenance when AI-generated candidates are promoted through Radar or Review.
