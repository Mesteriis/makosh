# ADR-0338: Client system status over shared realtime

Статус: Принято
Дата: 2026-07-29
Состояние реализации: Реализовано

Связанные решения:

- [ADR-0203](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0337](ADR-0337-capability-routed-managed-client-realtime.md).

## Контекст

Authenticated `ClientBootstrap` возвращает initial snapshot client surfaces,
admitted module/capability inventory и sanitized system status. Frontend
одновременно повторяет весь bootstrap query каждые 15 секунд только ради
client-observed round-trip.

Так смешаны три разные ответственности:

1. initial/recovery snapshot;
2. Kernel/platform/module health transitions;
3. client-observed transport latency.

При каждом latency sample пересоздаются неизменившиеся module inventory,
surface catalog и health tree. Это также дублирует уже принятый один
authenticated multiplexed SSE stream. ADR-0205 требует передавать module
health через SSE, поэтому polling всего bootstrap не является допустимой
окончательной реализацией.

## Решение

### Один transport, разные семантики

Используется только существующий endpoint:

```text
GET /api/realtime/v1/events
```

Новый SSE endpoint, WebSocket, provider-specific stream или domain-owned
transport не создаются.

```text
authenticated initial ClientBootstrap query
    ↓
one owner-scoped Gateway SSE subscription
    ↓
typed platform.system_status.changed event
    ↓
replace only system_status projection
```

`ClientBootstrap` выполняется один раз при запуске authenticated client и
повторяется только после explicit replay gap, protocol error/re-enrollment или
явного recovery action. Периодический client polling всего bootstrap запрещён.

### Authority и ownership

Kernel остаётся единственной authority sanitized system status. Он получает
platform/runtime observations только через существующие narrow control/query
boundaries и формирует provider-neutral projection.

Gateway владеет typed client-safe payload encoding, owner-session
authorization, bounded replay/live delivery и SSE framing.

Communications domain и Mail/Telegram/WhatsApp/Zulip integrations:

- не импортируют Kernel/Gateway system-status implementation;
- не публикуют platform health truth;
- не получают special transport;
- продолжают публиковать integration-owned operational events через отдельные
  admitted contracts.

System status является platform projection, а не domain или integration.

### Typed contract

Gateway protocol вводит:

```text
ClientSystemStatusChangedV1
  revision
  repeated ClientSystemComponentStatusV1 statuses
```

Realtime envelope использует exact значения:

```text
contract_name    = makosh.gateway.system-status
contract_version = 1
event_kind       = platform.system_status.changed
```

Payload — Protobuf bytes этого contract. Generic JSON/map, `Any`, internal
runtime metadata, process addresses, credentials, provider identifiers и
private content запрещены.

Snapshot содержит полный exact inventory system components. Partial patch
запрещён: frontend атомарно заменяет только `bootstrap.systemStatus`, сохраняя
initial surface/module snapshot.

### Reconciliation

Kernel-owned system-status reconciler:

- наблюдает только sanitized platform projection;
- сравнивает canonical encoded snapshot с последним опубликованным;
- публикует событие только при реальном изменении;
- не публикует неизменившийся snapshot по таймеру;
- не выполняет business/domain queries;
- не становится generic telemetry stream.

Bounded reconciliation допустим для independently supervised infrastructure,
у которой health уже получается через narrow probes. Это Kernel-side
observation reconciliation, а не client polling и не новый module transport.

Каждый authenticated owner stream получает current snapshot после
subscription. Subsequent transitions используют тот же owner-local
history/replay source. Replay gap заставляет client заново запросить bootstrap
и открыть SSE; silent cursor reset запрещён.

Первичный browser connect использует уже полученный authenticated bootstrap как
current snapshot и открывает SSE на текущем edge cursor. Gateway не проигрывает
новому client process прежние transition frames: иначе UI последовательно
откатывает свежий bootstrap через исторические состояния. `OPEN` несёт edge
cursor как SSE `id`; bounded history воспроизводится только при настоящем
reconnect с browser-managed `Last-Event-ID`.

### Latency

Initial bootstrap query может показать one-shot round-trip. Он:

- не является Kernel/platform health truth;
- не инициирует periodic bootstrap refresh;
- не изменяет route/module inventory;
- не заставляет health tree повторно раскрывать группы.

Continuous latency требует отдельного bounded diagnostic/heartbeat contract.
До него UI показывает initial round-trip и актуальное SSE stream state.

### Failure semantics

- SSE disconnected: последний snapshot остаётся stale, SSE становится
  unavailable.
- Replay gap: закрыть stream, получить fresh bootstrap, открыть stream заново.
- Invalid payload: закрыть stream и fail closed, не применять partial data.
- Bootstrap unavailable: recovery snapshot без optimistic healthy.
- Reconciler unavailable: SSE transport может быть жив, но status не fresh.

## Phase gate `client_system_status_realtime_v1`

Gate реализован только при наличии:

1. отдельного typed Protobuf payload;
2. exact Gateway contract name/version/event kind;
3. Kernel-owned change-only reconciliation;
4. owner-authenticated shared SSE delivery;
5. initial snapshot и transition event;
6. reconnect/replay и replay-gap bootstrap recovery;
7. frontend без periodic bootstrap polling;
8. frontend atomic system-status replacement;
9. privacy-negative tests;
10. live Gateway test, доказывающего transition без повторного bootstrap query.

Наличие ADR не открывает gate само по себе.

На 2026-07-29 gate подтверждён:

- change-only/replay unit tests Gateway;
- Kernel reconciler unit test;
- live Gateway SSE test без bootstrap polling;
- frontend typed decoding, atomic replacement и malformed-payload tests;
- architecture guard против periodic bootstrap timer и второго transport;
- live `make dev` проверкой: один bootstrap, один SSE и сохранение раскрытого
  health tree после прежнего 15-секундного polling window.

## Отклонённые варианты

### Polling всего ClientBootstrap

Смешивает snapshot, latency и realtime, создаёт лишние module queries и
пересобирает health UI.

### Второй `/system-health/events` SSE

Нарушает правило одного physical foreground stream и создаёт отдельные
cursor/reconnect semantics.

### Provider/domain health через platform contract

Integration operational health остаётся integration-owned, а domain не
становится transport/platform authority.

### Проброс internal lifecycle envelope

Internal runtime identity, generations, routes и control metadata не являются
client-safe contract.

## Последствия

Положительные:

- frontend перестаёт polling-ить immutable bootstrap inventory;
- system changes приходят через существующий SSE;
- latency и platform health больше не смешаны;
- domain/integration/build-unit boundaries сохраняются;
- UI обновляет только изменившуюся projection.

Отрицательные:

- Gateway protocol получает ещё один typed payload;
- Kernel поддерживает bounded status reconciler;
- client обрабатывает replay gap и stale stream;
- initial round-trip больше не выглядит непрерывной метрикой.
