# ADR-0098 Provider-Neutral Communications API And Strict Boundaries

Status: Accepted
Date: 2026-06-21

Supersedes:

- ADR-0097 public route decision for channel-scoped
  `/api/v1/communications/{mail,telegram,whatsapp}/*` business routes.

Clarifies:

- ADR-0042 Provider Credential Secret References And Resolver Boundary
- ADR-0076 Host Vault
- ADR-0085 Communication Spine And Consistency / Contradiction Engine
- ADR-0095 Event-Driven Domain Communication And DLQ
- ADR-0097 Communications Channel Domains To Integrations

## Context

ADR-0097 correctly established that Mail, Telegram and WhatsApp are
integrations, not product domains. It still allowed provider-scoped
Communications business routes as an intermediate migration shape:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

That intermediate shape leaves provider identity in the product API and makes it
too easy for integration code, app handlers and frontend modules to keep owning
business message state. The target model is stricter: Communications owns the
business state; providers supply observations, runtime state and command
execution.

## Decision

Макошь business Communications APIs are provider-neutral.

Provider-neutral product routes use:

```text
/api/v1/communications/conversations
/api/v1/communications/messages
/api/v1/communications/messages/{message_id}/...
/api/v1/communications/media
/api/v1/communications/search/...
```

Provider runtime/setup routes use:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

Provider search under `/api/v1/integrations/{provider}/provider-search` is a
runtime/control trigger only. It returns command/status metadata and must not
return projected Communication message, media, conversation or topic items.
Normal user search uses provider-neutral Communications routes such as
`/api/v1/communications/search/messages` and
`/api/v1/communications/search/media`.

The old provider-scoped business route families are removed and are not kept as
compatibility aliases:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

Boundary rules are strict:

- `backend/src/integrations/**` must not import `crate::domains::*`.
- `backend/src/domains/**` must not import `crate::vault::*`.
- `backend/src/platform/**` must not contain SQL ownership of business domain
  tables such as `communication_*`, `task_*`, `calendar_*`, `review_*` or
  `graph_*`; platform owns technical event, observation, settings, audit,
  secret and storage primitives only.
- `backend/src/workflows/**` coordinates through domain command/query ports,
  events and platform contracts; it must not import concrete stores, handlers or
  integration clients.
- `backend/src/app/**` handlers validate, authorize, audit and map responses;
  business orchestration lives in application/workflow services.
- `frontend/src/domains/**` and `frontend/src/integrations/**` must not import
  each other directly. Shared types/helpers live in `frontend/src/shared` or
  `frontend/src/platform`; composition lives in app-level modules.

Architecture checks must enforce these rules structurally. New baseline files,
hardcoded per-file allowlists and linter/guard weakening are forbidden.

## Consequences

Positive:

- Communications has one provider-neutral product API.
- Provider runtimes can evolve without leaking into business routes.
- Guards fail on real boundary leaks instead of hiding them as compatibility
  exceptions.

Negative:

- Existing provider-scoped frontend clients and backend route tests must move in
  the same implementation pass.
- Workflows and app handlers need explicit ports/application services before the
  stricter guards can pass.

## Validation

The repository must enforce:

- no backend `integrations -> domains` imports;
- no backend `domains -> vault` imports;
- no platform SQL against business domain tables;
- no workflow concrete store/handler/integration-client imports;
- no app handler store/runtime/workflow orchestration;
- no frontend `domains <-> integrations` imports;
- no provider-scoped Communications business routes;
- no provider-search business read routes under `/api/v1/integrations/*`;
- no guard baseline or hardcoded per-file allowlist for these rules.
