# V3 Closure Checklist

## Release Goal

Version 3.0 is complete when Макошь exposes local, source-backed AI workflows over the existing memory spine: Ollama runtime health, pgvector semantic retrieval, cited answers, suggested task candidates, meeting prep packets, persisted agent run history and desktop AI surfaces.

## In Scope

- Local Ollama chat and embedding provider boundary.
- pgvector `halfvec(2560)` semantic embedding store for messages, documents, projects, tasks and Personas.
- Semantic indexing from existing canonical projections.
- Retrieval planner combining semantic nearest neighbors and local text match signals.
- Prompt builder that treats retrieved source text as untrusted context.
- Registered agents: `HESTIA`, `MAKOSH`, `MNEMOSYNE`, `ATHENA`.
- Persisted AI run history with model config, prompt template version, citations, answer, timings, constant frontend actor and correlation IDs.
- Canonical events for AI run requested/completed/failed and task extraction completion.
- Protected AI APIs with local shared secret header.
- AI task extraction that creates only `suggested` candidates linked to `agent_run_id`.
- Meeting prep packets backed by local sources, without calendar/provider writes.
- Desktop-only AI Agents and scoped ask/brief/task refresh surfaces.
- Live validation against Ollama at the configured local endpoint.

## Out Of Scope For V3

- Cloud models.
- Fine-tuning private data.
- Autonomous activation.
- External email, calendar, provider or task writes.
- Calendar ingestion as a prerequisite for meeting prep.
- Provider adapter implementation.
- Mobile UI design, implementation or validation.

## Acceptance Gate Status

- [x] ADR-0049 documents the V3 AI runtime, retrieval and provenance policy.
- [x] Docker Compose uses pinned `pgvector/pgvector:0.8.2-pg16`.
- [x] Backend migration enables pgvector and creates `halfvec(2560)` semantic embeddings.
- [x] Backend migration creates persisted AI agent run history.
- [x] `task_candidates` supports `agent_run_id` for AI-suggested candidates.
- [x] Ollama client covers `/api/version`, `/api/tags`, `/api/chat` and `/api/embed`.
- [x] AI APIs expose status, agents, run history, answers, task refresh and meeting prep.
- [x] AI APIs require `X-Макошь-Secret`.
- [x] AI answers return citations and persist completed run history.
- [x] AI task extraction creates suggested candidates only.
- [x] Meeting prep returns a source-backed briefing without calendar writes.
- [x] `make backend-ai-smoke-dev` validates pgvector integration and live Ollama model behavior.
- [x] Desktop AI Agents tab reads live backend AI status, agents, run history, answer form and citations.
- [x] Scoped desktop Ask AI / Prepare brief controls are available on source-backed surfaces.
- [x] AI task extraction action reuses the existing task candidate review queue.
- [x] `make validate`, `make frontend-check` and `make frontend-build` pass.
- [x] Desktop browser smoke validates AI Agents, cited answer, task refresh and meeting prep.
