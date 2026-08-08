# ADR-0081 Opt-In OmniRoute AI Runtime

Status: Proposed

## Context

ADR-0009 selected Ollama as the initial local AI runtime boundary and requires remote models to be opt-in and policy controlled if added later.
ADR-0049 implemented V3 AI with Ollama as the only provider, but the local infrastructure now exposes a dedicated OpenAI-compatible OmniRoute gateway for this workstation.

Макошь still handles private communications and documents. Remote or routed model calls must not become implicit defaults.

## Decision

Add an opt-in AI runtime provider named `omniroute` alongside the existing default `ollama` provider.

Rules:

- `ollama` remains the default provider.
- `omniroute` is enabled only by explicit runtime setting or environment override.
- OmniRoute uses an OpenAI-compatible API boundary.
- Non-secret provider settings may live in `application_settings`.
- OmniRoute API keys remain outside `application_settings`; the initial implementation reads `MAKOSH_OMNIROUTE_API_KEY` from process environment.
- The AI run event payload records provider name and model IDs, not API keys or private prompt/document bodies.
- Existing semantic embedding dimension validation remains enforced; changing embedding models still requires compatibility with `halfvec(2560)` unless a future ADR changes the derived index shape.

## Consequences

Positive:

- Макошь can use the owner-managed OmniRoute gateway without hardcoding a cloud provider.
- Local Ollama remains the safe default for private data.
- Provider replacement is isolated behind a runtime client boundary.

Negative:

- Opting into OmniRoute can send private prompts and retrieved context to upstream providers selected by OmniRoute routing.
- Live smoke validation requires an API key that must not be printed or committed.
- Embedding model changes remain constrained by the existing semantic index dimension.

## Non-Goals

- No new database migration.
- No storage of OmniRoute API keys in PostgreSQL settings.
- No graph/schema expansion.
- No audio, image generation, or transcription support in this slice.
