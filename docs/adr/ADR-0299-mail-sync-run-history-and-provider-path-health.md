# ADR-0299: Mail sync run history and provider-path health

- Статус: принято
- Дата: 2026-07-27
- Состояние реализации: backend contract, V10 owner-local persistence,
  runtime instrumentation, exact query route, IMAP/Gmail managed conformance,
  generated first-party client и Mail-owned status/history UI реализованы.
  Gate `mail_sync_health_v1` открыт как `implemented`.
- Связанные решения: ADR-0204, ADR-0205, ADR-0213, ADR-0214, ADR-0222,
  ADR-0239, ADR-0270, ADR-0282, ADR-0292, ADR-0294, ADR-0298

## Контекст

Mail уже имеет:

- exact manual command `mail.sync.v1`;
- IMAP/Gmail provider adapters;
- owner-local checkpoints и operational projection;
- account readiness в `mail.account.query.v1`.

Но результат sync возвращается только вызывающему client request. После
restart нет public run history, terminal failure receipt или ответа на вопросы
«когда Mail действительно синхронизировался» и «какой provider path сейчас
готов». Historical implementation хранила background sync runs, settings,
progress и sanitized failures, но одновременно:

- запускала собственный polling scheduler;
- проецировала Communications, Personas и Organizations из одного workflow;
- хранила generic JSON checkpoints;
- смешивала provider health с canonical mailbox analytics;
- называла newsletter detection subscriptions.

Такую границу возвращать нельзя.

## Решение

### Ownership

Mail integration владеет:

- состоянием одного provider sync execution;
- bounded history manual/scheduled runs;
- sanitized terminal outcome;
- provider-path readiness;
- observed message count;
- runtime generation и monotonic projection revision.

Mail не владеет:

- canonical Communications mailbox analytics;
- newsletter/subscription discovery;
- Review state;
- Scheduler schedules, leases и retry policy;
- Kernel health или module lifecycle;
- provider cursor/checkpoint в public client contract.

Historical newsletter detection классифицируется как Communications-derived
evidence и восстанавливается через
`communications_saved_search_v1`/`communications_sender_insights_v1`, а не
через Mail sync health.

### Exact client contracts

`makosh-mail-api` добавляет Protobuf package `makosh.mail.sync_health.v1` с
двумя отдельными client routes:

```text
mail.sync.health.query.v1
  GetStatus
  ListRuns
  GetRun

mail.sync.v1
  Sync
```

Query route не является новым sync command. Existing `mail.sync.v1` остаётся
единственным manual trigger.

`MailSyncRunV1` содержит только:

- `operation_id`;
- `connection_id`;
- trigger `MANUAL | SCHEDULED`;
- outcome `RUNNING | SUCCEEDED | FAILED | INTERRUPTED`;
- `observed_messages`;
- `started_at_unix_seconds`;
- optional `completed_at_unix_seconds`;
- optional bounded sanitized `failure_code`;
- `runtime_generation`;
- `projection_revision`.

`MailSyncStatusV1` содержит latest run, provider-path readiness,
consecutive-failure count и last-success timestamp. Он не содержит provider
host, username, credential state, token, cursor, raw exception, private health
diagnostics или Communications counts.

`ListRuns` использует opaque connection-scoped cursor и page size `1..=200`.
Unknown, stale или wrong-scope cursor fail closed; client не начинает page one
молча.

### Persistence and execution

Mail persistence получает additive owner-local tables:

- `mail_sync_runs`;
- `mail_sync_status`.

До provider I/O runtime атомарно создаёт `RUNNING` row по exact
`operation_id`. Один connection может иметь только один current run. Duplicate
operation ID:

- возвращает persisted terminal result без повторного provider I/O;
- не присоединяется к чужому connection;
- не перезапускает `RUNNING` operation.

После provider result runtime атомарно фиксирует `SUCCEEDED` или `FAILED`.
Provider exception преобразуется в закрытый enum sanitized failure codes; raw
diagnostics остаются вне public persistence и client response.

Managed runtime startup помечает незавершённые runs предыдущего runtime
generation как `INTERRUPTED`. Он не объявляет их failed provider operations и
не теряет evidence restart.

### Scheduling and settings

Mail runtime не запускает detached polling timer.

- `mail.sync.window` и `mail.sync.windows` остаются typed Mail Settings и
  определяют bounded provider fetch.
- enabled/interval/retry schedule принадлежит Scheduler.
- Scheduler позже вызывает exact `mail.sync.v1` с durable operation ID.
- `SCHEDULED` trigger становится допустим только через Scheduler-owned
  dispatch evidence; first-party client не может выдать себя за Scheduler.

`mail_sync_health_v1` не зависит от fake local scheduler. Полный scheduled sync
требует отдельно admitted Scheduler job binding; статус уже умеет честно
показать manual run history до этого.

### Functional and build boundaries

- sync-health Protobuf и validation меняются с Mail public language;
- persistence module меняется с Mail run journal;
- runtime instrumentation меняется с provider execution lifecycle;
- client port меняется с exact route mapping;
- assembly только включает обновлённые Mail artifacts и migrations;
- frontend generated client, Gateway adapter, controller и presentation
  меняются после backend admission как отдельные Mail-owned units.

Runtime не является assembly. Mail не является domain. Query не является
command. Kernel/Gateway не импортируют Mail schema и не интерпретируют
failure/outcome.

### Privacy and bounds

- IDs non-empty, UTF-8 bounded and contain no control characters;
- failure code выбирается из closed enum;
- run pages, cursor and response sizes bounded;
- timestamps positive and monotonic within one run;
- successful result never carries failure code;
- failed/interrupted result never masquerades as provider success;
- credentials, sessions, checkpoints, message content and provider diagnostics
  do not enter query payload, logs or errors.

## Admission

`mail_sync_health_v1` становится `implemented` только атомарно с:

1. exact contract, route and descriptor capability;
2. owner-local additive Storage bundle;
3. start/success/failure/interrupted persistence semantics;
4. idempotency, concurrency, cursor-scope and privacy negative tests;
5. IMAP and Gmail managed Gateway conformance;
6. generated first-party client and Mail-owned UI status/history cutover;
7. architecture guards proving no Communications, Scheduler or Kernel
   implementation import.

First-party frontend cutover выполнен поверх exact admitted query capability;
общий gate открыт только после его validation вместе с backend evidence.

## Состояние реализации

Backend и first-party frontend slice реализованы:

- `makosh.mail.sync_health.v1` имеет exact canonical Protobuf mapping;
- Mail descriptor revision 5 предоставляет отдельную capability
  `mail.sync.health.query.v1`;
- Storage bundle revision 10 добавляет `mail_sync_runs` и
  `mail_sync_status`;
- manual sync создаёт `RUNNING` до provider I/O и фиксирует sanitized terminal
  outcome после IMAP/Gmail result;
- exact terminal replay возвращает сохранённый result без второго provider
  вызова;
- stale runtime generation переводится в `INTERRUPTED` с closed code
  `RUNTIME_RESTARTED`;
- scoped cursor, cross-account, privacy и stale-generation negative paths
  покрыты static/managed conformance;
- live authenticated managed contours подтверждены отдельно для IMAP и Gmail.
- `frontend/scripts/generate-proto.mjs` включает exact
  `makosh.mail.sync_health.v1` contract в generated client bundle;
- отдельные Mail-local Connect client и Gateway adapter реализуют `GetStatus`,
  `ListRuns` и `GetRun` без handwritten REST;
- controller выбирает только admitted Mail connection с exact capability
  `mail.sync.health.query.v1`, fail closed при её отсутствии и сохраняет
  connection-scoped cursor semantics;
- Mail operational page показывает provider-path readiness, last success,
  consecutive failures и bounded restart-safe run history, не раскрывая
  provider cursor, credential state, raw diagnostics или Communications data;
- architecture guard подтверждает generated contract, client/controller/
  presentation boundaries и отсутствие запрещённых owner imports.

## Последствия

Mail sync получает restart-safe operational evidence без возврата legacy
workflow facade. Scheduler остаётся platform owner расписаний, Communications
остаётся canonical evidence owner, а newsletter analytics не маскируется под
provider subscription state. First-party UI показывает только persisted
Mail-owned evidence и не подменяет его session-only результатом команды.
