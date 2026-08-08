# Ollama Integration

Status: code-aligned documentation package created from ADR-0009 and current
backend modules.

Ollama is the initial local AI runtime boundary for Макошь. It is an
integration adapter, not a source of truth.

ADR source of truth:

- [ADR-0009 Local AI Through Ollama](../../archive/adr/ADR-0009-local-ai-through-ollama.md)
- [ADR-0049 V3 Local AI Runtime And Retrieval](../../archive/adr/ADR-0049-v3-local-ai-runtime-and-retrieval.md)

## Current Implementation Evidence

Current backend files:

- `backend/src/integrations/ollama/mod.rs`;
- `backend/src/integrations/ollama/client/config.rs`;
- `backend/src/integrations/ollama/client/chat.rs`;
- `backend/src/integrations/ollama/client/embeddings.rs`;
- `backend/src/integrations/ollama/client/catalog.rs`.

The current client configuration carries a base URL, chat model, embedding
model and timeout. Current result models carry chat content or embedding
vectors plus model metadata.

## Boundary Rule

Ollama can provide local model output and embeddings. AI output remains
proposal or derived state and must not become accepted memory without evidence
and review rules from the owning domain or workflow.

