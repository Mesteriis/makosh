# ADR-0266: Telegram Kernel admission and event-only Communications handoff

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Phase gate; `telegram_integration_v1` не открыт.
Owner-neutral ClientRpc admission ADR-0256, Telegram generated Protobuf,
integration-owned runtime client port и integration-owned Communications
outbox уже существуют. Exact Telegram `ModuleDescriptorV1`, пять
route-specific client capabilities, отдельные platform capability units и
canonical non-secret settings schema реализованы. Telegram persistence
публикует immutable owner-local `StorageBundleV1`, а отдельная
`makosh-telegram-assembly` materializes exact descriptor/settings/storage
artifacts и unsigned fragment для generic signed distribution compiler. Эти
units не входят в Communications inventory и не дают runtime права сами по
себе. Telegram runtime теперь использует один correlated V2 control frame pump
для descriptor/ready, Storage/Vault, provider credential, Event, Blob и
client-delivery operations, совместимый с подписанным protocol-major `2`.
Owner transition в `suspended` или `revoked` теперь сначала фиксирует durable
registration/grant-epoch fence, затем идемпотентно останавливает exact managed
worker. Managed launch использует monotonic per-registration generation
high-watermark, старые reservation/grant epoch и managed client route
отклоняются до relay. Замена signed runtime binding сначала durable фиксирует
новую revision, затем останавливает старый worker; client, Blob, Event и Vault
issuance/relay routes дополнительно требуют exact current binding revision.
Suspend/revoke после grant-epoch fence атомарно переводит все active Storage
bindings integration registration в `revoking`, запускает существующий Storage
Control physical fence и независимо пытается остановить integration worker;
недоступный Storage оставляет durable incomplete revocation вместо ложного
успеха, а exact retry той же Storage binding revision повторно использует
revocation reservation. Live conformance теперь доказывает signed managed
launch exact Telegram runtime и native dependency, Kernel-issued
Storage/Vault/provider/Event/Blob leases, lifecycle query и provider command с
отдельными состояниями `accepted` и `completed`. Inbound путь проходит только
как provider frame → Telegram-owned PostgreSQL outbox → exact bytes в JetStream
→ Communications inbox/canonical event. Повторная доставка exact observation
подтверждает inbox deduplication и не создаёт второе Communications event.
Отдельный broker-process outage test останавливает NATS после provider command,
сохраняет новое observation pending в Telegram outbox, подтверждает, что
integration runtime остаётся доступным, затем запускает тот же broker и
доказывает replay до нового durable Communications evidence. Kernel при этом
управляет admission, capabilities, leases и routing, но не вызывает
Communications и не интерпретирует business payload. Backend privacy evidence
также закрыто: subject и route metadata выводятся только из fixed admitted
contract, Telegram не добавляет owner-private health surface, TDLib credential,
session path, QR link, password hint и authorization diagnostics имеют
redacted `Debug`, а недоверенный provider error message отбрасывается на
adapter boundary. Phase gate остаётся закрытым до финального frontend
cutover без legacy integration REST/fallback. Telegram
`/api/v1/communications/*` business facade, его query/inspector chain и
Communications-prefixed provider realtime caches уже удалены без alias или
replacement facade. Frontend generator и четыре независимых
`TelegramAuthorizationService`, `TelegramLifecycleService`,
`TelegramOperationalService` command/query и `TelegramRealtimeService` client
units реализованы; их наличие не считается cutover, пока provider experience
не удалит оставшиеся integration REST calls и не докажет live
generated-client flow.

Уточняет:

- [ADR-0201: Core/module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: integration plugins and provider-neutral context boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0215: module admission and capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0240: Telegram clean-room provider boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0256: owner-declared ClientRpc route admission](ADR-0256-owner-declared-client-rpc-route-admission.md);
- [ADR-0265: provider operational client transport admission](ADR-0265-provider-operational-client-transport-admission.md).

## Контекст

Фраза «integration общается с Kernel» смешивает две разные границы:

1. platform control plane, где Kernel допускает, запускает, ограничивает и
   останавливает integration runtime;
2. business data plane, где provider observation пересекает owner boundary и
   становится canonical Communications evidence.

Kernel имеет authority только над первой границей. Он не является
Communications domain, provider facade или посредником business semantics.
Telegram не вызывает Communications runtime/domain, а Communications не
вызывает Telegram runtime. Их inbound связь существует только как durable
typed event delivery.

Текущий Telegram client port использует один umbrella contract
`telegram.client` и пытается распознать lifecycle, command, query и
authorization payloads последовательно. Такой decode-by-probing смешивает
разные функциональные ответственности, не позволяет выдать узкий grant на
конкретный surface и становится неоднозначным при эволюции Protobuf. До
production admission этот временный seam должен быть заменён exact
route-directed decoding.

## Решение

### Независимые owner и единицы сборки

Telegram является integration owner с exact module identity:

```text
owner_id  = telegram
module_id = makosh-telegram-runtime
```

Его clean-room build unit состоит только из:

```text
makosh-telegram-api
makosh-telegram-core
makosh-telegram-tdlib
makosh-telegram-persistence
makosh-telegram-runtime
```

Communications остаётся отдельным domain owner и отдельной build unit:

```text
makosh-communications-ingress
makosh-communications-api
makosh-communications-domain
makosh-communications-persistence
makosh-communications-runtime
```

`makosh-communications-ingress` является public typed event contract, а не
domain implementation facade. Telegram может зависеть от него только для
создания neutral observation envelope. Telegram не импортирует
Communications API/domain/persistence/runtime. Communications не импортирует
ни один Telegram package. Kernel, Gateway и platform packages не импортируют
ни Telegram, ни Communications implementation.

Наличие нескольких Cargo packages внутри одной integration не превращает их в
несколько owners. Configuration instance/account также не является owner,
module или отдельной build unit.

### Kernel control plane

Telegram runtime взаимодействует с Kernel только через owner-neutral platform
contracts. Kernel:

- принимает bounded `pending` registration и exact descriptor bytes;
- хранит owner-approved capability grants отдельно от descriptor declarations;
- проверяет signed or owner-pinned executable, descriptor, settings schema и
  storage bundle bindings перед каждым managed launch;
- создаёт runtime generation и grant epoch fences;
- согласует Storage, Vault, Blob, Event Hub/NATS, settings и client-route
  capabilities;
- маршрутизирует opaque ClientRpc payload только по approved descriptor route;
- отзывает effective routes and leases при suspend, revoke, descriptor
  replacement, grant epoch change или runtime generation replacement;
- наблюдает lifecycle/health без чтения provider state.

Kernel не:

- декодирует Telegram Protobuf или Communications evidence payload;
- создаёт provider commands, observations или business events;
- выбирает Telegram account, provider operation или business workflow;
- хранит TDLib state, Telegram projection или Communications evidence;
- предоставляет Telegram прямой socket/query/store доступ к Communications;
- расширяет grant из self-declared descriptor.

Следовательно, Kernel является admission, routing и fencing authority, но не
business peer integration.

### Provider operational client routes

Provider-specific UI использует только generated Telegram client через Core
Gateway и owner-neutral ADR-0256 routing:

```text
Telegram provider experience
        ↓
generated Telegram client
        ↓
Core Gateway
        ↓
approved owner-declared ClientRpc route
        ↓
makosh-telegram-runtime
```

Telegram descriptor объявляет пять независимых client capabilities. Каждая
из них предоставляет ровно один exact ClientRpc route и один exact contract
reference:

| Capability | Contract name | Connect path |
|---|---|---|
| `telegram.authorization.v1` | `telegram.authorization.v1` | `/makosh.telegram.v1.TelegramAuthorizationService/Authorize` |
| `telegram.lifecycle.v1` | `telegram.lifecycle.v1` | `/makosh.telegram.v1.TelegramLifecycleService/Execute` |
| `telegram.command.v1` | `telegram.command.v1` | `/makosh.telegram.v1.TelegramOperationalService/ExecuteCommand` |
| `telegram.query.v1` | `telegram.query.v1` | `/makosh.telegram.v1.TelegramOperationalService/ExecuteQuery` |
| `telegram.realtime.v1` | `telegram.realtime.v1` | `/makosh.telegram.v1.TelegramRealtimeService/Replay` |

Все пять contracts имеют `major = 1`, `revision = 4` и exact SHA-256 generated
descriptor set. Совпадение schema digest не объединяет contracts: их stable
names, routes, semantics и grants различны.

Platform dependencies не прячутся в этих client grants. Descriptor отдельно
объявляет `telegram.blob.v1`, `telegram.credentials.v1`,
`telegram.events.v1`, `telegram.runtime.v1` и `telegram.storage.v1`.
`telegram.runtime.v1` содержит exact `telegram.tdjson.v1` artifact request и
state-layout revision; это отдельная assembly/readiness unit, а не пятая
provider operation.

Kernel выбирает route по approved descriptor metadata и передаёт opaque
payload вместе с exact contract reference. Telegram runtime проверяет
`module_id`, `owner_id`, request identity и exact contract reference, затем
декодирует только соответствующий generated request type. Последовательное
«попробовать lifecycle, затем command, затем query» и общий
`telegram.client` fallback запрещены.

Authorization, lifecycle, command и query остаются разными функциональными
ports даже если один runtime process реализует их все. Это SRP по причине
изменения и authority, а не по размеру файла.

Provider command возвращает typed accepted receipt с operation identity.
`accepted` не является provider completion. Terminal state читается через
Telegram-owned operation query/replay или доставляется через отдельно
admitted Core Gateway realtime contract. Gateway не имитирует успешное
provider выполнение.

### Event-only handoff в Communications

Provider observation пересекает границу только так:

```text
External Telegram
        ↓
Telegram runtime
        ├─→ Telegram operational projection
        └─→ typed neutral Communications observation
                ↓
          Telegram-owned PostgreSQL outbox
                ↓ exact DurableEnvelopeV1 bytes
          NATS JetStream
                ↓
          Communications inbox/deduplication
                ↓
          Communications application/domain
                ↓
          Communications-owned mutation and outbox
```

Telegram фиксирует provider operational mutation и observation outbox record в
своей owner-local transaction, когда им нужна атомарность. Relay публикует
exact stored envelope bytes. Communications подтверждает broker delivery
только после inbox deduplication и своей owner-local transaction.

Для этого handoff запрещены:

- direct Telegram → Communications local/query/request RPC;
- запуск Communications runtime из Telegram process или наоборот;
- общий handler/service object;
- cross-owner SQL, shared tables или database role;
- импорт Communications domain/runtime в Telegram;
- импорт Telegram operational contract/provider SDK в Communications;
- Gateway/Kernel conversion provider payload → Communications semantics;
- REST alias, proxy, dual-write или fallback через
  `/api/v1/communications/*`.

`owner_id`, `module_id`, contract, runtime generation, grant epoch, causation
и correlation должны оставаться согласованными с admitted runtime. Для
Telegram exact module identity в client and observation envelopes —
`makosh-telegram-runtime`; сокращённый альтернативный ID не допускается.

### Cross-domain outbound actions

Business/context domain не вызывает Telegram operational contract. Если
domain event должен привести к внешнему действию, отдельный workflow:

1. consumes typed source-domain event;
2. сохраняет evidence, causation и correlation;
3. формирует provider-neutral intent;
4. выбирает разрешённые integration/configuration instance по explicit policy;
5. отправляет exact Telegram command через public operational port.

Такой workflow не получает доступ к storage Telegram или source domain.
До отдельного workflow ADR business-triggered provider action не admitted.

## Phase gate `telegram_integration_v1`

Gate открывается атомарно только когда существует всё evidence:

1. exact Telegram package inventory и Cargo isolation guards;
2. canonical `ModuleDescriptorV1` с пятью capabilities/routes, storage,
   Vault, Blob, event, settings и runtime budget requests;
3. deterministic descriptor/settings/storage bundle artifacts и exact digests;
4. signed distribution manifest or explicit owner-pinned managed binding;
5. pending registration без data-plane rights и explicit owner approval;
6. managed launch с exact runtime generation/grant epoch and stale-fence
   rejection;
7. generic Gateway routing без Telegram dependency и exact route-directed
   generated decoding внутри Telegram runtime;
8. revoke/suspend/restart/descriptor-change unmount and lease invalidation;
9. one live provider operational query and one accepted command with terminal
   result/replay evidence;
10. one live Telegram outbox → NATS → Communications inbox flow, including
    duplicate delivery and NATS outage replay;
11. absence of private bodies, credentials and provider sessions in route
    metadata, subjects, logs, errors and health;
12. removal of corresponding legacy frontend REST calls, tests and fallbacks
    in the same final cutover slice.

Backend descriptor, admission, runtime and event evidence выполняются до
frontend cutover. Frontend не используется как proof backend admission.

Открытие `telegram_integration_v1`:

- не расширяет `first_owner_v1`;
- не добавляет Telegram packages в Communications inventory;
- не делает integration domain;
- не разрешает следующий provider;
- не доказывается одним ADR или static architecture test.

## Порядок реализации

1. Создать exact Telegram descriptor builder и разделить client dispatch на
   пять capability-owned ports без decode probing.
2. Добавить descriptor, settings/storage artifacts и signed managed-launch
   admission profile.
3. Доказать Kernel route/grant/revoke/generation fences и managed runtime
   conformance.
4. Доказать exact-byte observation outbox delivery в Communications inbox.
5. Перевести Telegram frontend на generated clients и удалить legacy paths.

Каждый крупный slice является отдельным commit и проходит минимальные owner
tests плюс architecture/SRP/Cargo boundary gates.

## Отклонённые варианты

### Integration вызывает Communications runtime

Отклонено: создаёт direct cross-owner runtime dependency и обходит durable
event delivery, inbox deduplication и failure isolation.

### Kernel является business message broker

Отклонено: Kernel пришлось бы импортировать provider/domain contracts,
декодировать payload и стать owner-specific facade.

### Один `telegram.client` capability

Отклонено: authorization, lifecycle, command и query имеют разные authority,
failure modes и reasons to change. Один grant выдаёт лишние права, а
decode-by-probing не является typed routing.

### Добавить Telegram в `first_owner_v1`

Отклонено: `first_owner_v1` является exact Communications domain inventory.
Integration admission имеет отдельный gate и не меняет domain ownership.

## Rollback

Owner revokes Telegram capabilities, Kernel fences current generation,
unmounts routes and stops the managed runtime. Telegram storage, pending
outbox, provider session state and already persisted Communications evidence
сохраняются. Rollback не восстанавливает legacy REST facade, direct
Communications call или старый umbrella contract.
