# AI Hub

Status: target architecture with current code boundary.

`backend/src/ai/hub.rs` is the single application boundary for AI work in Макошь.
Provider adapters can still live in `backend/src/integrations/*`, and low-level runtime
contracts can still live in `backend/src/platform/ai_runtime.rs`, but product domains,
workflows and AI services must not call provider runtimes directly.

The rule is deliberately boring:

```text
Domain / Workflow / AI service
↓
AI Hub
↓
local Rust guard OR model route
↓
provider runtime adapter
```

This keeps provider details out of business code, because apparently every system
tries to become a provider-specific hairball unless physically restrained.

For Макошь specifically, direct in-process calls from workflows and app-side
services are valid when they are part of domain orchestration, must return
quickly, and do not require durable user-visible async status before
completion. In these cases, workflow code should call `AiHub` synchronously
and keep the event path for boundaries where eventual-consistency or async
visibility is required.

For this reason workflows and app-side orchestration may still use direct
in-process `AiHub` calls when they need immediate results, while AI requests
that are explicitly user-initiated from public API surfaces may choose to go
through async event/run projections.

Current public AI API behavior:

- `POST /api/v1/ai/answers`
- `POST /api/v1/ai/task-candidates/refresh`
- `POST /api/v1/ai/meeting-prep`

These endpoints return `202 Accepted` with `AiHubRequestAcceptedResponse {
request_id, run_id, status, event_id, correlation_id }`.

The durable result is then read from:

- `GET /api/v1/ai/runs`
- `GET /api/v1/ai/runs/{run_id}`

Public API acceptance appends `ai.hub.requested`, and background execution
appends `ai.hub.completed` or `ai.hub.failed` in addition to the internal
`ai.run.*` run lifecycle events.

Current AI Hub settings download behavior:

- `POST /api/v1/ai/model-downloads`

This endpoint is currently a synchronous control-center operation for built-in
Ollama models. It appends `ai.hub.model_download.requested`, then either
`ai.hub.model_download.completed` or `ai.hub.model_download.failed`.

Completion only marks the model `is_available = true`; it does not assign any
AI route automatically. Route selection remains an explicit Hub settings action.

## Responsibilities

AI Hub owns:

- model route selection by capability;
- local deterministic inspection before remote inference;
- translation entry points;
- chat/reasoning/extraction entry points;
- embedding entry points;
- runtime status and model listing pass-through;
- stable `Source`, `Confidence` and `Evidence` semantics for local AI results.

AI Hub does **not** own:

- domain truth;
- task, person, document or organization creation;
- provider credentials;
- provider protocol details;
- final review decisions.

AI output remains a candidate, suggestion, summary, classification or draft. It is not
a fact accepted into Макошь memory until the owning domain or review workflow records it.

## Two-tier model strategy

Макошь uses two different kinds of AI work.

### 1. Small local Rust agents

Small agents run inside the Rust backend process and must be deterministic, cheap and
safe to execute before any source text leaves the process.

Current local guard responsibilities:

| Capability | Current implementation boundary | Output |
|---|---|---|
| Language hint | `AiHub::detect_language` | `LocalLanguageDetection` |
| Sensitive content scan | `AiHub::detect_sensitive_content` | `Vec<SensitiveFinding>` |
| Combined inspection | `AiHub::inspect_text` | `LocalAiInspection` |
| PEM private key marker | local Rust string/rule scan | `private_key_pem` finding |
| PEM certificate marker | local Rust string/rule scan | `certificate_pem` finding |
| SSH key marker | local Rust string/rule scan | `ssh_public_key` finding |
| Secret-like assignments | local Rust string/rule scan | `secret_assignment` finding |
| Provider token shapes | local Rust string/rule scan | token-specific finding |
| JWT shape | local Rust string/rule scan | `jwt` finding |
| High-entropy token shape | local Rust entropy heuristic | `high_entropy_token` finding |
| IBAN shape | local Rust format heuristic | `iban` finding |
| Payment card number | local Rust Luhn check | `payment_card_number` finding |

These agents are not LLM prompts. They are local guards.

Target extensions for this tier:

- fastText or CLD3 for stronger language detection;
- YARA-like rule packs for secrets and certificates;
- optional embedded GGUF model through Candle, mistral.rs or llama.cpp bindings;
- optional small instruction model for offline categorization when deterministic rules are not enough.

The embedded-model target is intentionally separate from provider adapters. A future
small GGUF model can be mounted as a local hub backend without making domains learn
what GGUF, Candle or llama.cpp are. Humanity may yet recover from leaking adapter names
into product code.

### Future: Rust-native embedded inference

The current built-in local model path uses Ollama as a provider runtime. It does not
run LLM inference inside the Макошь Rust process.

A future AI Hub slice should add a Rust-native embedded model backend for small,
offline tasks where local latency and privacy matter more than model size. Target
uses include:

- short translation and language normalization;
- message/document categorization;
- lightweight extraction for tasks, obligations and decisions;
- offline relevance and routing classification;
- stronger local guard decisions before any external model call.

This future backend should appear to product code as another AI Hub provider or
runtime backend. Domains and workflows should still depend on AI Hub routes, not on
`candle`, `mistral.rs`, `llama.cpp`, GGUF paths or tokenizer details.

Before implementation, create a dedicated ADR that chooses the embedded runtime,
model format, local model cache layout, download/update lifecycle, memory limits,
CPU/GPU behavior, evaluation fixtures and fallback behavior when the embedded model
is missing.

### 2. Large external or provider-backed models

Large models are used only after the local guard path has produced enough inspection
context for the caller to make a safe routing decision.

Current external runtime adapters:

| Provider boundary | Purpose |
|---|---|
| `backend/src/integrations/ollama` | local provider runtime |
| `backend/src/integrations/omniroute` | external provider runtime |
| `backend/src/integrations/ai_runtime.rs` | provider-neutral runtime client enum |
| `backend/src/platform/ai_runtime.rs` | provider-neutral runtime trait and result DTOs |

Current model routes:

| Route | Intended use |
|---|---|
| `default_chat` | generic assistant answering |
| `reasoning` | heavier contextual reasoning |
| `summarization` | summaries and translation fallback |
| `mail_intelligence` | email intelligence workflow |
| `reply_draft` | reply drafting |
| `extraction` | task/action/entity extraction |
| `embeddings` | vector embeddings |
| `meeting_prep` | meeting preparation |

Routes are capability names, not provider names. The model behind a route comes from
AI Hub routing. In a configured workspace Макошь expects explicit route assignment and
fails the request when a required route is missing or the chosen model is unavailable.
For public async API requests, that failure is recorded as a failed run instead of
only surfacing as a synchronous transport error.

## Safety rule before external calls

Before sending source text to an external provider, the caller must be able to run:

```rust
let inspection = AiHub::inspect_text(source_text);
```

If `inspection.sensitivity` is `High` or `Critical`, the caller must either:

- keep the operation local;
- redact before external inference;
- require explicit user confirmation;
- or skip the external inference path.

The current patch provides the guard and central hub boundary. Redaction policy and
confirmation UX remain owner-specific follow-up work.

## Dependency rule

Allowed:

```rust
use crate::ai::hub::{AiHub, SharedAiHub, AiModelRoute};
```

Avoid in domains, workflows and AI services:

```rust
use crate::integrations::ai_runtime::AiRuntimeClient;
use crate::platform::ai_runtime::SharedAiRuntimePort;
```

Provider adapters and app composition may still construct runtime clients. After
construction, business code should receive `SharedAiHub`.

## Result evidence contract

Every local AI result must provide:

```text
Source
Confidence
Evidence
```

For example, a sensitive finding contains:

```rust
SensitiveFinding {
    kind,
    severity,
    confidence,
    evidence,
}
```

Evidence must be bounded and masked when it refers to sensitive values. The hub should
never return full secrets as evidence. The system is supposed to protect secrets, not
collect them like cursed trading cards.

## Migration checklist

When adding new AI behavior:

1. Add or reuse an `AiModelRoute`.
2. Put provider-specific logic in an integration adapter, not in a domain.
3. Expose the behavior through `AiHub`.
4. Run local inspection before external inference when raw user content is involved.
5. Return candidates with confidence and evidence.
6. Let the owning domain or workflow decide whether to record anything as memory.
