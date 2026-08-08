# ADR-0082 AI Settings Control Center

Status: Superseded by ADR-0106

This decision was superseded by [ADR-0106](ADR-0106-ai-hub-local-agents-and-external-models.md).
The original motivation remains valid, but the implemented surface is now `AI Hub`
inside the `Hub` settings group, with explicit route assignment and download-first
local model lifecycle instead of bootstrap-enabled local defaults.

## Context

ADR-0049 introduced the V3 local AI runtime with Ollama defaults. ADR-0081 added explicit OmniRoute support, but the current settings surface still exposes AI as generic `application_settings` rows. That does not scale to built-in runtime management, CLI-backed local agents, remote API providers, per-capability model routing, or editable prompt templates.

Макошь handles private communications and documents. AI provider configuration must preserve the local-first posture, keep secrets in the host vault from ADR-0076, and make remote-context consent explicit.

## Decision

Add an AI Hub domain surfaced from Settings inside the `Hub` section.

Rules:

- `application_settings` remains the allowlisted non-secret fallback surface, but AI provider accounts, model inventory, routing and prompt studio state live in AI domain tables.
- AI provider accounts support `built_in`, `cli` and `api` provider kinds.
- API provider secrets are stored only through host-vault secret references. Environment-backed OmniRoute remains a legacy/bootstrap fallback.
- Remote/API providers require explicit provider-level consent before they can be used for private-context workflows.
- CLI agents are provider bridges only. They may execute only allowlisted fixed command/argument presets and must not become autonomous workflow actors in this slice.
- Built-in Ollama runtime management is desktop/macOS-first. Макошь may install/start/update the runtime automatically, but curated local model downloads require explicit user confirmation and do not auto-assign routes.
- Model routing uses stable capability slots instead of one global chat model. Embedding routes must keep the current 2560-dimension constraint until a future ADR changes the semantic index shape.
- Prompt templates are versioned. System prompts are seeded/read-only, while user prompts and active versions are stored as domain records.
- Prompt evaluation runs may persist model output and metadata, but audit/event payloads must not store raw private source text, API keys or provider secret values.

## Consequences

Positive:

- AI configuration becomes understandable as a product area instead of a generic settings list.
- Provider setup, model catalog, routing and prompt templates can evolve independently.
- Secrets remain behind the host-vault resolver boundary.
- AI runs can record provider/model/prompt provenance without leaking credentials.

Negative:

- A new migration and API surface are required.
- Runtime management introduces OS/process concerns that must stay behind allowlisted adapters.
- Remote provider consent becomes part of user workflow before some models can be selected.

Risk handling:

- Seed safe local Ollama defaults.
- Treat remote/API provider state as unavailable until consent and a vault-backed credential are present.
- Keep CLI command presets static and validated.
- Add regression tests for secret-like payload rejection and no private text in event/audit metadata.
