# Макошь App Layer

Status: documentation package aligned to the current repository structure.

The app layer mirrors `backend/src/app`.

It owns HTTP, ConnectRPC, router registration, request guards, response mapping,
SSE/WebSocket surfaces and thin handler boundaries. It does not own business
logic, provider protocol logic or durable domain truth.

## Current Code Areas

- `backend/src/app/router` - route registration.
- `backend/src/app/handlers` - domain-facing route handlers.
- `backend/src/app/provider_runtime_handlers` - integration runtime/setup
  route handlers.
- `backend/src/app/api_support` - DTO and request/response support shared by
  handlers.
- `backend/src/app/error` - public response error mapping.
- `backend/src/app/connectrpc` - ConnectRPC service surfaces.

## Documentation Rule

App documentation should describe API surface, routing, authorization and
response behavior. Cross-domain orchestration belongs in
`docs/application/` or `docs/workflows/`; provider behavior belongs in
`docs/integrations/`.
