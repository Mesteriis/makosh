# ADR-0049 V3 Local AI Runtime and Retrieval

Status: Proposed

## Context

This ADR is amended by ADR-0081 for opt-in OmniRoute runtime support.

ADR-0009 selects Ollama as the first local AI runtime. ADR-0007 keeps vector search replaceable. ADR-0022 forbids fine-tuning private data. V3 needs source-backed AI workflows over the V1/V2 memory spine without turning model output into source of truth.

Qwen3 embedding output is 2560 dimensions. pgvector `vector` is not suitable for this dimension in the current stack, while `halfvec(2560)` supports the required shape and approximate cosine search.

## Decision

Implement V3 AI as a thin local runtime over existing canonical projections:

- Ollama is the only V3 model provider.
- Default chat model is `qwen3:4b`.
- Default embedding model is `qwen3-embedding:4b`.
- Embeddings use pgvector `halfvec(2560)` with HNSW cosine indexing.
- Semantic embeddings are derived state for messages, documents, projects, tasks and contacts.
- Source-backed answer generation must retrieve local citations before prompting.
- Retrieved text is treated as untrusted context in prompts.
- AI answers, task extraction runs and meeting prep runs persist `ai_agent_runs` records with status, model config, prompt template version, answer, citations, timings, actor ID and correlation IDs.
- AI run requested/completed/failed and AI task extraction lifecycle events are represented as canonical events.
- V3 task extraction may create only `suggested` task candidates linked to `agent_run_id`; existing review APIs remain the only path to active tasks.
- V3 meeting prep returns a local briefing packet and does not require calendar ingestion.
- V3 protected APIs require `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>` and `X-Макошь-Actor-Id`.

## Non-Goals

- Cloud model providers.
- Fine-tuning or training on private data.
- Autonomous activation.
- External email, calendar, message or task writes.
- Provider adapter implementation.
- Mobile UI.

## Consequences

Positive:

- AI behavior is auditable through persisted runs and canonical events.
- Citations keep local source provenance visible to UI and tests.
- Embeddings remain rebuildable derived state.
- Model/provider replacement remains possible behind the Ollama boundary and semantic store.

Negative:

- Local model latency becomes part of V3 workflow UX.
- `make validate` depends on a reachable live Ollama runtime for AI smoke validation.
- pgvector is now required in the local development PostgreSQL image.
- Prompt quality and retrieval ranking need ongoing evaluation with real local data.
