# ADR-0298: Mail operational read projection and client contract

- Статус: принято
- Дата: 2026-07-26
- Состояние реализации: реализовано полностью. Публичный typed contract,
  owner-local persistence, bounded scoped queries, атомарная IMAP/Gmail sync
  materialization, exact runtime client route и managed Gateway conformance
  подтверждены live host contour. Generated first-party client, Mail-owned
  account discovery, отдельный read controller, responsive UI и visual
  regression cutover подтверждены; gate `mail_operational_read_v1` имеет
  состояние `implemented`.
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0220,
  ADR-0236, ADR-0239, ADR-0270, ADR-0278, ADR-0281, ADR-0282, ADR-0294

## Контекст

Mail является integration owner, а не business domain. Уже реализованный Mail
runtime умеет синхронизировать IMAP/Gmail и публиковать provider-neutral
observations в Communications, но не имеет полного clean-room operational read
контракта для папок, тредов и сообщений.

Historical Mail UI и API смешивали разные authority:

- provider accounts, folders, threads and messages — Mail;
- canonical evidence, authorized body content, saved search and sender
  insights — Communications;
- pin, snooze, important and review — Review;
- AI Reply, translation, summary and extraction — отдельные workflows;
- drafts, templates, signatures and compose — отдельный Mail composition gate;
- provider mutations and delivery — отдельный Mail command gate.

Копирование legacy API вернуло бы integration/domain facade и прямую
зависимость одного owner от другого. Удаление этих функций также не является
переносом. Нужен точный provider operational read boundary.

## Решение

### Ownership и маршрут

`makosh-mail-api` владеет exact Protobuf package
`makosh.mail.operational.v1`. После полного admission один capability
`mail.operational.query.v1` маршрутизируется:

```text
first-party client
  -> Core Gateway authenticated owner route
  -> Mail managed runtime
  -> Mail-owned PostgreSQL role
  -> Mail operational projection
```

Core Gateway проверяет session, GrantSet, runtime generation и exact contract
hash, но не декодирует Mail payload. Kernel/Gateway не импортируют
`makosh-mail-api`. Mail runtime не вызывает Communications query, Blob или
другой owner.

### Exact query surface

Контракт содержит только:

- `ListFolders`;
- `ListThreads`;
- `ListMessages`;
- `GetMessage`.

Account readiness остаётся в `mail.account.query.v1`. Список configuration
instances строит first-party app из owner-neutral module registrations и
отдельных Mail account queries; один Mail runtime не агрегирует другие Mail
runtimes. Sync/subscription/health относится к независимому
`mail_sync_health_v1`.

Все list queries используют opaque scoped cursor и bounded page size
`1..=200`. Cursor не является offset, provider token или сериализованным SQL.
Unknown cursor, wrong scope and deleted anchor fail closed without silent
restart from page one.

### Mail operational projection

Mail persistence получает owner-local additive tables:

- `mail_operational_folders`;
- `mail_operational_threads`;
- `mail_operational_messages`;
- `mail_operational_message_folders`.

Одна sync transaction materializes provider records and enqueues the exact
Communications observation outbox record. Partial projection without the
corresponding outbox intent запрещена. Duplicate provider record updates the
same Mail row and never creates a second provider identity.

IMAP currently exposes only the admitted `INBOX`. Gmail labels become
provider-owned folders. Gmail `threadId` is retained as the provider thread.
For IMAP no heuristic subject grouping is allowed: a record without exact
provider thread identity receives a deterministic single-message thread ID.

Projection may contain only bounded operational data:

- connection, provider message/thread/folder identities;
- bounded subject, sender/recipients and snippet;
- provider timestamp when it is available;
- explicit flags;
- attachment/plain-text availability booleans;
- opaque 16-byte Communications observation anchor;
- monotonic projection revision.

It does not store credentials, sessions, provider cursors in client rows, raw
MIME, HTML, attachment bytes, arbitrary metadata maps or Communications state.
Provider cursor/checkpoint remains separate Mail sync state and is never
returned to clients.

### Content and cross-owner composition

Mail message detail deliberately does not return full body. The Mail projection
returns only a bounded sanitized snippet and the opaque observation anchor.
Authorized content is owned by `communications_content_read_v1`.

The first-party application may compose:

```text
Mail GetMessage
  + Communications GetAuthorizedContent(observation anchor)
  -> Mail operational screen
```

This composition is an `app` unit. Mail does not import Communications, and
Communications does not import Mail. Missing/denied content remains an explicit
state; the client cannot fall back to a Mail database or provider call.

### Functional boundaries

- Protobuf and validation change with Mail operational language.
- Mail sync adapters change with provider protocols.
- Mail projection persistence changes with owner-local SQL/materialization.
- Mail runtime changes with managed routing and lifecycle.
- A generated frontend client changes with one exact service.
- A controller changes with one read use case.
- Presentation changes with Mail operational view state.

Assembly remains a downstream release unit and does not implement queries.
Runtime is not assembly. Mail is not Communications domain.

### Privacy and bounds

- IDs are non-empty, bounded and contain no control characters.
- subject/snippet/address collections and page responses are bounded;
- snippets are plain text and contain no raw HTML;
- no secrets, tokens, cookies, provider request diagnostics or private health
  details enter errors, logs, cursor or client frames;
- no generic map, JSON payload, `Any` or opaque business bytes are admitted;
- clients never connect to Mail process, NATS or PostgreSQL directly.

## Admission

`mail_operational_read_v1` becomes `implemented` only atomically with:

1. exact contract route and descriptor capability;
2. owner-local additive Storage bundle;
3. IMAP and Gmail materialization into the same query model;
4. cursor scope/reset/privacy negative tests;
5. managed Gateway query conformance under current grants/generation;
6. generated first-party client and Mail-owned UI cutover;
7. architecture guards proving no Mail/Communications implementation imports.

ADR or generated code alone does not open the gate.

## Последствия

Mail получает полноценный provider operational read model without becoming a
domain or a cross-owner facade. Communications remains canonical evidence and
content owner. Historical saved searches, sender analytics, AI, Review,
compose and mutations are not lost: they remain explicit gates from ADR-0282
and are restored through their own owners and workflows.
