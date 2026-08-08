# Макошь Integration Catalog

Status: documentation package aligned to the current repository structure.

Integrations are provider and protocol adapters. They observe external systems,
manage provider runtime/setup state, preserve source provenance and emit events
or evidence into owner domains, workflows and engines.

An integration is not a Макошь product domain. Provider-specific runtime state
must not own durable product truth such as Personas, Tasks, Documents,
Decisions, Obligations or Communication business state.

## Package Shape

Integration documentation mirrors `backend/src/integrations/<provider>/` and
`frontend/src/integrations/<provider>/` where possible. The Zoom package is the
current reference shape:

- `README.md` for provider framing and scope;
- `architecture.md` for boundary, flow and ownership;
- `modules.md` for backend/frontend module map;
- `api.md` plus optional `api/` details for route references;
- `status.md` plus optional `status/` evidence logs;
- `gap-analysis.md`, `blockers.md`, `implementation-plan.md`,
  `fixture-test-matrix.md` and `live-smoke-checklist.md` when real current
  content exists.

Do not create empty placeholder files just to fill the shape.

## Providers

| Provider | Package | Backend owner |
|---|---|---|
| Mail | [mail](mail/README.md) | `backend/src/integrations/mail` |
| Telegram | [telegram](telegram/README.md) | `backend/src/integrations/telegram` |
| WhatsApp | [whatsapp](whatsapp/README.md) | `backend/src/integrations/whatsapp` |
| Zoom | [zoom](zoom/README.md) | `backend/src/integrations/zoom` |
| Yandex Telemost | [yandex-telemost](yandex-telemost/README.md) | `backend/src/integrations/yandex_telemost` |
| Zulip | [zulip](zulip/README.md) | `backend/src/integrations/zulip` |
| Ollama | [ollama](ollama/README.md) | `backend/src/integrations/ollama` |
| OmniRoute | [omniroute](omniroute/README.md) | `backend/src/integrations/omniroute` |

## Boundary Rules

- Provider setup/runtime APIs live under `/api/v1/integrations/*`.
- Provider-neutral product APIs live under owning domains such as
  `/api/v1/communications/*`.
- Raw credentials and session material stay behind the secret/vault boundary.
- Integration event payloads must be sanitized before append or broadcast.
- Provider adapters must not import business domains directly.
- AI runtime integrations are still integrations: they may produce model output
  or embeddings, but AI output is never source-of-truth memory.
