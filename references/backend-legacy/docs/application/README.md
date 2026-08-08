# Макошь Application Services

Status: documentation package aligned to the current repository structure.

Application services mirror `backend/src/application`.

This layer coordinates domain commands, workflows, provider runtime services
and platform contracts without becoming the source of truth for domain data.

## Current Code Areas

- provider runtime service contracts;
- review inbox and review transitions;
- communication send and provider-write orchestration;
- signal replay and signal-derived dispatch;
- Zoom, WhatsApp, Telegram and mail coordination services;
- project, task, person and relationship projection effects.

## Documentation Rule

Use this folder for service-level coordination contracts that are too concrete
for product domain specs and too business-specific for `docs/platform/`.
Durable entity ownership still belongs to `docs/domains/`.
