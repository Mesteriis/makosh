# ADR-0291: Zulip account, history, operational query и replay boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: `zulip_history_sync_v1`,
`zulip_operational_read_v1`, `zulip_operational_realtime_v1` и
`zulip_account_lifecycle_v1` и `zulip_full_operational_v1` реализованы. Full
gate закрыт только после generated frontend client, sealed owner credential
provisioning и integration-owned UI cutover поверх ранее принятых backend
gates.

Уточняет:

- [ADR-0201: Core/module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: provider-neutral boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0222: Kernel settings registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0265: provider operational client transport](ADR-0265-provider-operational-client-transport-admission.md);
- [ADR-0271: Zulip Kernel admission](ADR-0271-zulip-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0292: managed settings apply and credential binding](ADR-0292-managed-integration-settings-apply-and-credential-binding.md).

## Контекст

Реализованный `zulip_integration_v1` доказывает managed admission, provider
commands, terminal operation status, event queue и event-only handoff в
Communications. Он не доказывает полный provider experience:

- account lifecycle доступен только как незафиксированная композиция settings;
- owner-local message history и search отсутствуют;
- stream/topic/direct projection отсутствует;
- client не может прочитать attachments и lifecycle mutations;
- replayable provider operational journal отсутствует;
- текущий frontend умеет только отправку и operation status.

Удалённый legacy REST/UI не считается переносом. Reference-код подтверждает
send/edit/delete/reaction/file/event-queue use cases, но не является authority
для clean-room contracts. Отсутствующие history/query/replay contracts
определяются этим ADR по общим правилам Макошь.

## Решение

### Owner и функциональные units

Zulip остаётся integration, а не domain:

```text
owner_id  = zulip
module_id = makosh-zulip-runtime
```

Причины изменения разделены внутри существующих owner packages:

```text
makosh-zulip-api          account/read/replay public contracts and validation
makosh-zulip-core         provider event to neutral evidence mapping only
makosh-zulip-http         Zulip HTTPS command, history and event-queue adapter
makosh-zulip-persistence  owner-local projection, sync cursor and replay journal
makosh-zulip-runtime      admitted orchestration and exact route handlers
makosh-zulip-assembly     immutable release composition only
```

API, provider protocol, persistence, runtime и assembly не объединяются.
Assembly не владеет lifecycle/query semantics. Communications не импортирует
Zulip packages и не читает Zulip storage.

### Account lifecycle и Settings

Zulip API-key account не имеет отдельного OAuth lifecycle. Его lifecycle
является композицией трёх authorities:

1. app/integration Settings UI записывает typed desired settings через
   Kernel Settings Registry;
2. Vault хранит API key и выдаёт exact process-bound
   `zulip.credentials.v1` lease;
3. Zulip runtime применяет effective settings и публикует только sanitized
   account/projection state через provider-owned query.

Client не получает API key, Vault lease, runtime generation, grant epoch,
Storage binding или process topology. `configure`, `replace credential` и
`retire account` не становятся Zulip business REST aliases:

- settings revision применяется supervised managed replacement protocol;
- credential replacement создаёт новую Vault revision и новую effective
  settings revision;
- retire удаляет desired configuration и отзывает credential authority через
  platform workflow.

Zulip runtime не может сам изменить Kernel desired settings или удалить Vault
secret. Kernel не интерпретирует realm, bot identity или provider account
semantics.

Gate `zulip_account_lifecycle_v1` требует typed settings schema, separate
secret revision, fresh Vault lease, supervised replacement, sanitized account
status и negative stale/revoked-grant evidence.

### Provider history convergence

Event queue доставляет только новые изменения и не является полной historical
projection. После admitted startup runtime:

1. регистрирует event queue;
2. выполняет bounded pages через Zulip `GET /api/v1/messages` с raw Markdown;
3. сохраняет history sync cursor и typed message snapshots;
4. между страницами poll-ит queue, чтобы не потерять concurrent updates;
5. продолжает до provider `found_oldest`;
6. только после этого выставляет `history_ready`.

Каждая page bounded; бесконечный request и unbounded in-memory accumulation
запрещены. Provider authorization scope определяет доступную историю. Макошь
не заявляет доступ к сообщениям, которые Zulip account не имеет права читать.

Backfill snapshot не перезаписывает более свежую event-driven edit/delete.
Existing event projection имеет precedence. Crash продолжает с durable oldest
provider message cursor. Queue expiration требует fresh registration и
продолжение того же history cursor, а не reset owner storage.

Gate `zulip_history_sync_v1` требует bounded provider fixture, multi-page
restart convergence, queue/backfill race evidence и explicit partial readiness.

### Operational query

Вводится отдельная capability:

```text
capability = zulip.operational.query.v1
route      = /makosh.zulip.operational.v1.ZulipOperationalQueryService/Query
gate       = zulip_operational_read_v1
```

Typed oneof поддерживает:

- `ListMessages`;
- `SearchMessages`;
- `ListConversations`;
- `ListEvents`;
- `GetAccountStatus`.

Message projection содержит exact account/message/conversation/sender
identities, direction, raw Markdown content when authorized, edit/delete state,
typed attachment metadata и current reactions. Conversation различает
stream/topic и direct recipient identities. Generic map, `Any`, raw provider
JSON, internal durable envelope и private event-queue DTO запрещены.

Pagination bounded. Owner-issued cursor scoped к exact query kind, account и
filters. Повреждённый или cross-query cursor отклоняется. Search выполняется
только по Zulip-owned projection и не становится Communications search.

Existing `zulip.query.v1` остаётся только terminal provider-operation status.
Расширять его до generic provider query или принимать оба payload shape на
одном route запрещено.

### Realtime replay

Вводится отдельная capability:

```text
capability = zulip.operational.realtime.v1
route      = /makosh.zulip.operational.realtime.v1.ZulipOperationalRealtimeService/Replay
gate       = zulip_operational_realtime_v1
```

Owner-local append-only journal хранит monotonic sequence и exact typed
operational event. Replay response содержит:

- `earliest_available_sequence`;
- `latest_available_sequence`;
- ordered bounded frames;
- `next_sequence`;
- explicit `reset_required`.

Internal `DurableEnvelopeV1` клиенту не выдаётся. Gateway может включить
provider frame в общий client realtime transport, но не декодирует и не
проецирует Zulip payload. Missing cursor после retention/incompatible rebuild
возвращает explicit reset; silent gap запрещён.

### Persistence и atomicity

Additive immutable Storage bundle revision добавляет:

- current message projection;
- conversation projection;
- attachment and reaction projections;
- account/history sync state;
- append-only operational event journal.

Event ingestion одной PostgreSQL transaction:

1. продвигает exact provider queue cursor;
2. применяет Zulip operational projection;
3. пишет typed replay event;
4. пишет neutral Communications outbox records.

Duplicate/stale event не создаёт вторую mutation или replay frame.
Communications outbox остаётся event-only boundary. Provider content не
копируется в Communications через query/store import.

History page transaction сохраняет snapshots и sync cursor. Она не пишет
Communications observations задним числом и не выдумывает event causation для
старых provider messages.

Storage migration является DDL-only. Runtime не выполняет production DDL,
cross-owner SQL или legacy-table migration.

### Kernel/Core agreement

Kernel/Core согласуют только:

- exact descriptor sets, routes and capability grants;
- current registration, runtime generation and grant epoch fences;
- Settings/Vault/Storage/Event leases;
- opaque ClientRpc delivery и client realtime composition;
- supervised replacement после effective settings revision.

Kernel, Gateway и Settings не:

- импортируют `makosh-zulip-*`;
- декодируют query/replay payload;
- выбирают stream/topic/direct/history filters;
- читают Zulip tables;
- хранят provider message content;
- создают Zulip или Communications business truth.

Integration общается с Communications только через typed durable observations.
Integration не вызывает Communications runtime/query/storage.

### Frontend boundary

Generated provider clients и controller живут только в
`frontend/src/integrations/zulip`. Communications page может показывать
provider-neutral evidence и ссылку на Zulip operational experience, но не
импортирует Zulip store/controller.

`zulip_full_operational_v1` открывается только после:

1. `zulip_account_lifecycle_v1`;
2. `zulip_history_sync_v1`;
3. `zulip_operational_read_v1`;
4. `zulip_operational_realtime_v1`;
5. generated frontend client;
6. integration-owned UI cutover;
7. удаления scoped handwritten/legacy aliases.

Frontend cutover реализован отдельными SRP units:

- generated query/replay messages и service descriptors в
  `frontend/src/gen/makosh/zulip/operational`;
- route-specific ConnectRPC factories и validating gateways в
  `frontend/src/integrations/zulip/api`;
- exact effective-account discovery и независимые read/replay controllers в
  `frontend/src/integrations/zulip/queries`;
- pure account/history/message/event/replay presentation models и panels в
  `frontend/src/integrations/zulip/presentation`;
- `ZulipOperationalRoute.vue` композирует только integration-owned units, а
  `AppLayoutRoot.vue` передаёт exact admitted capabilities и bootstrap modules.

Read controller не владеет Settings/Vault mutation, command status или
Communications state. Replay controller отдельно хранит monotonic cursor и
показывает `reset_required` без silent restart. Pure panels не импортируют
transport/controller code. Handwritten REST, event-queue и provider HTTP
aliases в active client отсутствуют.

## Phase gates

### `zulip_account_lifecycle_v1`

Gate требует:

1. typed settings schema без credential plaintext;
2. independent Vault secret revision and integration-owned binding, never a
   Settings value/reference;
3. supervised desired/effective settings application;
4. sanitized provider-owned account status;
5. fresh process-bound credential lease after replacement;
6. stale/revoked lease and cross-account negative evidence.

### `zulip_history_sync_v1`

Gate требует:

1. bounded typed provider history adapter;
2. durable oldest-message cursor and explicit readiness;
3. multi-page restart convergence;
4. queue/backfill precedence and no stale resurrection;
5. authorization-limited history semantics;
6. no fake backfill from metadata-only Communications evidence.

### `zulip_operational_read_v1`

Gate требует:

1. separate generated descriptor and exact route capability;
2. owner-local messages/conversations/attachments/reactions projection;
3. typed bounded list/search/status query;
4. query-scoped opaque cursors;
5. exact route/grant/runtime-generation/grant-epoch fences;
6. duplicate/out-of-order and cross-account negative evidence;
7. `zulip_history_sync_v1`.

### `zulip_operational_realtime_v1`

Gate требует:

1. separate generated descriptor and exact route capability;
2. monotonic owner-local typed journal;
3. bounded ordered replay and explicit reset;
4. restart-safe sequence/cursor evidence;
5. exact route/grant and cross-account negative evidence.

Ни один backend gate сам по себе не открывает
`zulip_full_operational_v1` и не добавляет integration в domain inventory.

### `zulip_full_operational_v1`

Gate открыт как `implemented` по совместному evidence account lifecycle,
history sync, operational read/realtime и frontend cutover. Generated query и
replay clients создаются из canonical backend protobuf через
`frontend/scripts/generate-proto.mjs`; frontend не становится authority
provider history, credential binding или replay retention.

## Фактическая реализация

- `makosh-zulip-api` публикует отдельные descriptor sets и exact capabilities
  `zulip.operational.query.v1` и `zulip.operational.realtime.v1`; command,
  operation status, content read и replay не имеют общего route/grant.
- `makosh-zulip-http::history` выполняет bounded
  `GET /api/v1/messages` pages с `apply_markdown=false`, exact anchor и
  explicit `found_oldest`; event queue сохраняет typed stream/topic/direct,
  attachments, reactions, edit/delete fields.
- Zulip Storage bundle revision 2 остаётся additive и DDL-only. Он добавляет
  account/history state, messages/conversations/attachments/reactions,
  mutation fence и append-only operational journal, не изменяя immutable
  revision 1.
- `makosh-zulip-persistence::operational` одной transaction продвигает queue
  cursor, применяет projection, пишет replay event и Communications outbox.
  Backfill пишет отдельную transaction и не перезаписывает более свежий
  mutation sequence.
- Query cursors bind-ятся к exact account/query/filter digest; cross-query и
  malformed cursors отклоняются. Runtime дополнительно сверяет exact
  configured account до owner-local SQL.
- Managed runtime interleave-ит event polling и одну bounded history page на
  tick. Provider history failure переводит partial projection в sanitized
  degraded state и не завершает event/outbox runtime.
- Invalid cross-account client payload возвращает bounded protocol error и не
  останавливает managed process.
- Zulip Storage bundle revision 3 добавляет только owner-local CAS credential
  binding. Settings schema major 2 остаётся non-secret, а managed runtime
  поддерживает explicit configuration-only, active и retired states.
- Generic Kernel Settings apply и Zulip account lifecycle реализованы по
  ADR-0292. Live contour доказал Vault revision rotation, managed successor,
  stale predecessor fence, configuration-only retirement и fail-closed
  `blocked_config` для отсутствующей credential revision.

Live managed conformance:

```bash
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff node scripts/test-authenticated-storage.mjs 1.97.0
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_zulip_runtime_uses_kernel_leases_and_route_specific_admission node scripts/test-authenticated-storage.mjs 1.97.0
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_zulip_account_rotation_and_retirement_use_settings_successors node scripts/test-authenticated-storage.mjs 1.97.0
```

Первый contour доказал real TLS provider fixture, две bounded history pages,
raw Markdown search, stream/topic и direct conversations, current reaction,
cross-account rejection, typed replay, Communications event-only handoff,
NATS outage replay и сохранение history/query/replay state после managed
successor generation. Второй contour повторно доказал exact query grant,
ungranted command, stale generation, owner-authorized revoke и Storage fence.

Третий contour закрыл `zulip_account_lifecycle_v1`: desired/effective Settings
revision применяется только Kernel-managed successor, fresh credential
revision разрешается через Vault только в новом generation, retire остаётся
configuration-only, а отсутствующая revision оставляет desired intent в
`blocked_config` без rollback.

## Provider protocol evidence

Clean-room HTTP adapter следует public Zulip protocol:

- [Get messages](https://zulip.com/api/get-messages);
- [Register an event queue](https://zulip.com/api/register-queue);
- [Get events](https://zulip.com/api/get-events).

## Отклонённые варианты

### Читать Zulip через Communications

Отклонено: Communications хранит provider-neutral evidence и не является
provider operational facade.

### Выставить event queue напрямую клиенту

Отклонено: queue ID является private provider session state, не replayable
public cursor и не переживает expiration.

### Использовать один `zulip.query.v1`

Отклонено: operation status, content read и realtime имеют разные grants,
storage semantics и failure modes.

### Считать event queue полной историей

Отклонено: queue доставляет только доступные новые события и может истечь.

### Выполнять history/search в Kernel или Gateway

Отклонено: control plane стал бы provider-specific business runtime.

## Последствия

Zulip остаётся автономной integration единицей. Kernel/Core сохраняют
provider-neutral control/transport authority. Communications получает только
typed neutral observations через durable events. Полный provider experience
становится измеряемой совокупностью независимых gates, а не утверждением по
факту удаления legacy UI.
