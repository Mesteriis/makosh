# ADR-0286: WhatsApp operational read и realtime boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: `whatsapp_operational_read_v1`,
`whatsapp_operational_realtime_v1` и `whatsapp_full_operational_v1`
реализованы. Full gate закрыт только после отдельного frontend cutover поверх
ранее принятых backend gates; frontend не является evidence provider extractor
или backend ownership.

Уточняет:

- [ADR-0201: Core/module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: provider-neutral boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0224: Storage Control](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0265: provider operational client transport](ADR-0265-provider-operational-client-transport-admission.md);
- [ADR-0276: WhatsApp phase gate](ADR-0276-whatsapp-kernel-admission-host-bridge-and-event-only-communications-handoff.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Реализованный `whatsapp_integration_v1` доказывает admission, public command,
operation-status query, private host bridge и event-only Communications
handoff. Он не предоставляет полноценные provider-owned dialogs, message
history, search, participants, runtime status или replayable operational
events.

Generated private WhatsApp contract уже содержит часть typed provider query
DTO. Они являются implementation evidence integration, но не public client
authority. Выставить private host contract через Gateway или расширить
`whatsapp.query.v1` до универсального query endpoint нельзя:

- host bridge и public client имеют разные actors и grants;
- operation status и provider projection имеют разные storage/failure
  semantics;
- один generic response потребовал бы `Any`, raw JSON, opaque bytes или
  decode probing;
- frontend closure не должна определять backend ownership и persistence.

Legacy WhatsApp UI и reference backend используются только для восстановления
подтверждённых use cases. Они не являются source of truth для нового API,
storage или runtime.

## Решение

### Owner и единицы функциональности

WhatsApp остаётся integration owner:

```text
owner_id  = whatsapp
module_id = makosh-whatsapp-runtime
```

Функциональные units размещаются внутри существующих owner packages:

```text
makosh-whatsapp-api          public query/realtime contracts and validation
makosh-whatsapp-core         host observation to operational event mapping
makosh-whatsapp-persistence  projections, search and replay journal
makosh-whatsapp-runtime      route handlers and owner-local orchestration
makosh-whatsapp-assembly     immutable descriptor/storage release artifacts
```

Это не разрешает aggregate runtime или cross-owner package. Read model,
realtime journal и host ingestion должны иметь отдельные source modules,
tests и причины изменения. `makosh-whatsapp-runtime` только композирует
owner-local units.

Communications не импортирует эти packages и не читает их storage.
WhatsApp видит из Communications только разрешённый neutral ingress contract
для event handoff.

### Public operational query

Новая capability:

```text
capability = whatsapp.operational.query.v1
route      = /makosh.whatsapp.operational.v1.WhatsAppOperationalQueryService/Query
gate       = whatsapp_operational_read_v1
```

Public generated request содержит typed oneof:

- `ListMessages`;
- `SearchMessages`;
- `ListDialogs`;
- `ListParticipants`;
- `ListEvents`;
- `GetRuntimeStatus`.

Каждый вариант имеет собственный typed response. Generic map, `Any`, raw JSON,
opaque provider payload и private host DTO запрещены. Pagination использует
bounded owner-issued cursor; cursor должен быть scoped к exact query kind,
account и stable filter. Неизвестный, повреждённый или cross-query cursor
отклоняется.

`whatsapp.query.v1` сохраняет единственную ответственность: terminal status
принятой provider operation. Его route, grant и response не расширяются и не
становятся alias нового operational read.

### Public operational realtime

Новая capability:

```text
capability = whatsapp.operational.realtime.v1
route      = /makosh.whatsapp.operational.v1.WhatsAppOperationalRealtimeService/Replay
gate       = whatsapp_operational_realtime_v1
```

Replay request несёт bounded cursor и limit. Response содержит:

- monotonic owner-local sequence;
- typed operational event oneof;
- `earliest_available_sequence`;
- `latest_sequence`;
- explicit `reset_required`.

Realtime journal является WhatsApp-owned rebuildable operational history, а
не Communications canonical evidence и не внутренним `DurableEnvelopeV1`,
выдаваемым клиенту. Silent cursor gap запрещён. Если retention или
несовместимый upgrade удалил запрошенный диапазон, runtime возвращает explicit
reset и earliest available sequence.

SSE client composition остаётся обязанностью Core Gateway по ADR-0205.
Gateway маршрутизирует opaque generated payload и применяет capability fence,
но не декодирует WhatsApp business fields и не хранит projection.

### Private host observations

`whatsapp.host_bridge.v1` получает additive typed observation variants,
необходимые для полной operational projection:

- message upsert/delete/status;
- dialog upsert/archive;
- participant upsert/remove;
- account/runtime status.

Каждая observation содержит exact account, provider identity, observed time,
causation/correlation и provider revision when available. Provider session
state, cookies, WebView storage, arbitrary DOM snapshot и raw page JSON через
bridge запрещены.

Существующие metadata-only observations остаются валидными. Runtime не
выдумывает message body, flags, roles, counters или timestamps, которых не
было в принятом typed observation.

### Persistence и atomicity

WhatsApp Storage bundle получает additive DDL-only revision с:

- current message projection;
- current dialog projection;
- current participant projection;
- current runtime-status projection;
- append-only typed operational event journal;
- monotonic sequence allocator and replay retention metadata.

Migration не содержит business DML и не читает legacy tables. Host ingestion
в одной owner-local PostgreSQL transaction:

1. validates account/runtime/grant fence;
2. deduplicates exact host observation;
3. applies current operational projection;
4. appends typed realtime event;
5. appends neutral Communications outbox envelope when the observation has a
   valid neutral mapping.

Communications delivery остаётся асинхронной через exact outbox bytes и NATS.
Ошибка neutral mapping не должна превращать provider-specific state в
Communications truth; policy определяет fail-closed acceptance для evidence
eligible observation.

### Upgrade и backfill

Metadata-only V1 observation не содержит полного payload, поэтому из неё
невозможно честно восстановить body, participants, flags или provider status.
Fake backfill запрещён.

После upgrade WhatsApp integration выполняет bounded provider resync через
private host bridge и строит новую projection только из новых typed
observations. До завершения initial resync read/realtime readiness сообщает
явное degraded состояние. Старый или недоказуемый cursor получает explicit
reset; runtime не объявляет непрерывность, которую не может доказать.

### Kernel/Core boundary

Kernel/Core владеют только:

- registration, explicit capability grants and revoke;
- runtime generation and grant epoch fencing;
- exact descriptor/release/settings/storage binding;
- Storage/Event/private-host leases;
- opaque route-specific ClientRpc and client realtime delivery.

Kernel, Gateway и Communications не:

- импортируют `makosh-whatsapp-*`;
- декодируют WhatsApp query/event payload;
- выбирают account/dialog/message/provider action;
- читают WhatsApp tables;
- выполняют projection/search;
- создают provider operational truth.

### Frontend boundary

Frontend controller живёт только в `frontend/src/integrations/whatsapp`.
Он использует generated public operational clients. Communications pages могут
композировать provider-neutral evidence с exact integration navigation, но не
импортируют WhatsApp state/store/controller.

`whatsapp_full_operational_v1` открывается только после backend read/realtime,
generated client, integration-owned controller и UI cutover. Backend gates не
считаются закрытыми по наличию frontend facade.

Frontend cutover реализован отдельными SRP units:

- generated query/replay messages и service descriptors в
  `frontend/src/gen/makosh/whatsapp/operational`;
- route-specific ConnectRPC factories и validating gateways в
  `frontend/src/integrations/whatsapp/api`;
- exact effective-account discovery и независимые read/replay controllers в
  `frontend/src/integrations/whatsapp/queries`;
- pure presentation models и отдельные read/replay panels в
  `frontend/src/integrations/whatsapp/presentation`;
- `WhatsAppOperationalRoute.vue` композирует integration-owned units, а
  `AppLayoutRoot.vue` передаёт только exact admitted capabilities и bootstrap
  modules.

Frontend не импортирует Communications domain, provider WebView host bridge,
NATS, PostgreSQL или handwritten REST/realtime aliases.

## Phase gates

### `whatsapp_operational_read_v1`

Gate открывается только при наличии:

1. generated route-specific public query contract and descriptor capability;
2. owner-local additive DDL-only projections;
3. typed host observation mapping without provider payload leakage;
4. bounded list/search/filter/cursor validation;
5. exact route/grant/runtime-generation/grant-epoch fences;
6. restart-safe PostgreSQL conformance for duplicate/out-of-order ingestion;
7. managed host-to-projection-to-query live conformance;
8. negative evidence for ungranted/stale/cross-account access.

Gate реализован следующими отдельными units:

- `makosh-whatsapp-api` публикует exact
  `whatsapp.operational.query.v1` с отдельным descriptor set и typed query /
  response oneofs;
- `makosh-whatsapp-core::operational` преобразует только typed host
  observations и не восстанавливает отсутствующий metadata-only content;
- WhatsApp Storage bundle revision 2 добавляет DDL-only owner-local
  projections, event journal, timestamped message/participant tombstones и
  resync control state;
- `makosh-whatsapp-persistence::operational` атомарно deduplicate-ит host
  observation, применяет projection, пишет typed event и optional
  Communications outbox;
- `makosh-whatsapp-runtime` проверяет exact configured account и обрабатывает
  новый route только через отдельный granted capability.

Disposable managed conformance запускается так:

```bash
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff node scripts/test-authenticated-storage.mjs 1.97.0
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing node scripts/test-authenticated-storage.mjs 1.97.0
```

Первый contour доказывает duplicate/out-of-order ingestion, bounded
list/search/cursors, message delivery state, delete/remove без stale
resurrection, typed dialogs/participants/events, explicit resync readiness,
cross-account rejection и чтение той же PostgreSQL projection после successor
runtime/storage generation. Второй доказывает ungranted capability, stale
generation и revoke/grant-epoch fencing.

### `whatsapp_operational_realtime_v1`

Зависит от `whatsapp_operational_read_v1` и открывается только при наличии:

1. generated exact replay contract and distinct capability;
2. append-only typed event journal with monotonic sequence;
3. explicit earliest/latest/reset semantics;
4. duplicate-safe and restart-safe replay;
5. managed host-to-replay live conformance;
6. negative stale cursor, grant and privacy evidence.

Gate реализован отдельными units:

- `makosh-whatsapp-api` публикует отдельный protobuf package, descriptor set,
  route и capability `whatsapp.operational.realtime.v1`;
- `makosh-whatsapp-persistence::operational` читает append-only typed event
  journal по exact account, сохраняет monotonic sequence и проверяет hash
  каждого события до replay;
- `makosh-whatsapp-runtime::client_port` принимает replay только под отдельным
  granted contract, а managed composition дополнительно проверяет configured
  account;
- ответ содержит account, earliest/latest sequence, bounded ascending frames,
  next cursor и explicit `reset_required`;
- нулевой cursor начинает replay, exact ранее выданный cursor продолжает его,
  а неизвестный, удалённый или future cursor fail-closed требует reset без
  payload.

Live conformance использует те же disposable managed contours, что и read gate.
Положительный contour доказывает bounded multi-page replay, строгий порядок,
duplicate safety, persisted replay после successor runtime/storage generation,
future/stale cursor reset и отсутствие command payload в journal. Отрицательный
contour доказывает отдельный grant, а положительный contour — cross-account
privacy fence.

### `whatsapp_full_operational_v1`

Gate открыт как `implemented` по одновременному evidence:

- `whatsapp_operational_read_v1`;
- `whatsapp_operational_realtime_v1`;
- public generated frontend clients;
- integration-owned controller and complete WhatsApp UI cutover;
- absence of scoped legacy WhatsApp client state, REST and realtime aliases.

Generated query и replay clients создаются из canonical backend protobuf через
`frontend/scripts/generate-proto.mjs`. Read controller не принимает command или
Communications state, replay controller отдельно хранит monotonic cursor и
показывает `reset_required` без silent restart. Pure panels получают только
presentation models и emit actions; transport остаётся в gateways.

## Последствия

- WhatsApp operational truth не попадает в Communications domain.
- Operation status, operational read и replay имеют разные grants и failure
  boundaries.
- Storage evolution не подделывает данные, отсутствующие в старом signal.
- Frontend становится вторичным consumer принятого backend contract.
- Полный WhatsApp перенос измеряется тремя exact gates, а не наличием одного
  mixed facade.
