# Макошь AI Layer

Status: documentation package aligned to the current repository structure.

The AI layer mirrors `backend/src/ai`.

AI components provide local model access, control-center configuration,
semantic retrieval support, prompt/runtime contracts and agent-facing services.
AI output is never source of truth.

Remote AI routes may process private context only when the selected provider has
explicitly granted content consent in the AI Control Center. Local and remote
routes are selected through the same consent-validated model routing boundary.

## Packages

- [AI Hub](hub.md)
- [Agent Architecture](agents/agent-architecture.md)
- [Local AI Architecture](agents/local-ai-architecture.md)

## Documentation Rule

AI docs may describe model adapters, prompt/runtime contracts, semantic
retrieval and AI control surfaces. Source-backed memory, accepted domain truth
and review workflows stay in their owning domain, engine or workflow docs.
