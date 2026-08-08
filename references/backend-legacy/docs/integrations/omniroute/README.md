# OmniRoute Integration

Status: code-aligned documentation package created from ADR-0081 and current
backend modules.

OmniRoute is an opt-in OpenAI-compatible AI runtime provider. Ollama remains
the local default.

ADR source of truth:

- [ADR-0081 Opt-In OmniRoute AI Runtime](../../archive/adr/ADR-0081-opt-in-omniroute-ai-runtime.md)
- [ADR-0082 AI Settings Control Center](../../archive/adr/ADR-0082-ai-settings-control-center.md)

## Current Implementation Evidence

Current backend files:

- `backend/src/integrations/omniroute/mod.rs`;
- `backend/src/integrations/omniroute/client/config.rs`;
- `backend/src/integrations/omniroute/client/chat.rs`;
- `backend/src/integrations/omniroute/client/embeddings.rs`;
- `backend/src/integrations/omniroute/client/catalog.rs`.

The current client configuration carries a base URL, chat model, embedding
model, timeout and `ResolvedSecret` API key. Current settings definitions
explicitly say the OmniRoute API key is read from `MAKOSH_OMNIROUTE_API_KEY`
and is not stored in `application_settings`.

## Boundary Rule

OmniRoute may send prompts or retrieved context to the configured upstream
gateway only after explicit opt-in/runtime selection. API keys must stay out of
PostgreSQL settings and event payloads.

