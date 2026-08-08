# ADR-0276: WhatsApp Kernel admission, host bridge and event-only Communications handoff

Статус: Принято
Дата: 2026-07-25
Состояние реализации: backend phase gate `whatsapp_integration_v1` открыт.
Все двенадцать backend gates и первые пять implementation-слайсов этого
решения реализованы. Runtime использует один
correlation-owned `ManagedControlChannelV2`, передаёт его последовательно
Storage/Vault и Event Hub clients, не clone-ит inherited FD и отправляет
`ready` только после admitted bindings, owner-local persistence и Event Hub.
Generated `whatsapp.command.v1` и `whatsapp.query.v1` имеют разные capability,
Connect path и response marker, а runtime принимает их только через exact
descriptor-bound ClientRpc delivery. Accepted command сохраняет exact bytes в
owner-local queue, конфликтующий operation ID отклоняется, terminal status
читается через отдельный query route. Private `whatsapp.host_bridge.v1`
использует собственные typed operation/response oneofs без provider-query
decode probing; Tauri host проверяет exact contract name, descriptor digest и
route binding. Umbrella `whatsapp.client` удалён из production path.
Canonical descriptor, hidden configuration-scoped `whatsapp.account_id`
settings, immutable owner-local Storage bundle и отдельная unsigned
`makosh-whatsapp-assembly` теперь материализуются детерминированно. Один
admitted runtime обслуживает только свой configured account. Signed
distribution compiler подписывает exact runtime/descriptor/settings/storage
entries assembly unit. Disposable live contour доказывает pending
registration, explicit owner-approved capability subset, exact signed managed
launch, Storage binding через выданный PgBouncer pool alias, private host route
handshake, stale runtime-generation fence и revoke с grant-epoch advance.
Revoke переводит только WhatsApp Storage binding в `Revoking`, останавливает
только WhatsApp, сохраняет Communications active и удаляет exact owner-owned
Unix socket. Второй disposable live contour доказывает accepted public command,
exact native host lease, owner-local terminal result/query, отдельное
metadata-only host observation, transactional WhatsApp outbox, exact-byte NATS
delivery и Communications causation. Terminal provider receipt не становится
Communications evidence. Duplicate delivery не создаёт второй canonical event,
а NATS outage оставляет runtime active и pending outbox replay-ится после
reconnect. Provider body, private socket path и route binding отсутствуют в
durable event bytes. Frontend cutover остаётся отдельным secondary client
slice. Additive `whatsapp_operational_read_v1` по ADR-0286 также реализован:
отдельный capability/descriptor, typed host projection, Storage bundle
revision 2 и restart-safe managed PostgreSQL conformance не расширяют
ответственность command/status contracts этого ADR.

Уточняет:

- ADR-0201: Core/module communication and NATS;
- ADR-0204: integration/provider-neutral context boundary;
- ADR-0215: module admission and capability grants;
- ADR-0219: signed managed distribution;
- ADR-0221: descriptor and capability lifecycle;
- ADR-0241: WhatsApp clean-room provider boundary;
- ADR-0242: versioned host bridge;
- ADR-0256: owner-declared ClientRpc routing;
- ADR-0258: correlated duplex managed control;
- ADR-0265: provider operational client transport admission.

## Контекст

WhatsApp является отдельным integration owner, а не частью Communications.
Его backend packages уже отделяют provider API, policy, persistence и runtime,
а Tauri host владеет hidden WebView и provider session state. Runtime также
имеет owner-local PostgreSQL command queue, Communications outbox и
Kernel-issued Storage/Event/host-bridge configuration.

На момент принятия contour ещё не являлся production admission: inherited
control FD имел несколько V1 readers, umbrella `whatsapp.client` смешивал
private host bridge и public provider client contract, generated Protobuf не
объявлял route-specific Connect services, а exact artifacts, assembly и live
provider evidence отсутствовали. Первые три дефекта устранены; exact
`ModuleDescriptorV1`, settings/storage artifact builders и отдельная release
assembly unit также реализованы. Signed distribution binding и managed
admission/fencing доказаны первым live contour. Второй live contour закрывает
native host command/result, event-only Communications handoff, deduplication,
outage replay и negative privacy evidence без прямого runtime/domain вызова.

Сохранить `/api/v1/communications/*` как временный transport нельзя:
Communications не выполняет WhatsApp provider commands и не владеет его
operational projection.

## Решение

### Owner и единицы сборки

Production identity:

```text
owner_id  = whatsapp
module_id = makosh-whatsapp-runtime
```

Runtime source units:

```text
makosh-whatsapp-api          generated public and host contracts
makosh-whatsapp-core         provider anti-corruption and evidence mapping
makosh-whatsapp-persistence  owner-local queue, projection and outbox
makosh-whatsapp-runtime      managed runtime composition
```

`makosh-whatsapp-assembly` является отдельной integration-owned
build-time unit. Она материализует canonical artifacts, но не запускается
Kernel, не входит в runtime inventory и не имеет signing authority.

Settings schema содержит ровно один hidden operator-managed
`whatsapp.account_id` с `ConfigurationInstance` scope, fresh owner proof и
`RestartModule` apply mode. Runtime декодирует snapshot до admission и
отклоняет public command, private host observation или command lease другого
account. Owner-local Storage bundle создаёт только
`makosh_data.whatsapp_*` tables и не содержит Communications tables, foreign
keys или provider session state.

`makosh-communications-ingress` остаётся единственной разрешённой
WhatsApp → Communications compile dependency. Она предоставляет typed neutral
ingress contract, но не Communications domain/runtime/persistence/API.
Communications, Kernel и Gateway не импортируют WhatsApp implementation.

### Kernel/Core control plane

WhatsApp runtime общается с Kernel/Core для:

- pending registration и explicit owner-approved grants;
- exact signed executable/descriptor/settings/storage admission;
- runtime generation и grant epoch fencing;
- Storage, Event Hub, Blob и scoped credential leases;
- private host-bridge route staging;
- opaque route-specific public ClientRpc delivery.

Это control/admission plane. Kernel не:

- декодирует WhatsApp command/query/observation payload;
- выбирает account, chat, message, recipient или provider action;
- читает WebView session state;
- сохраняет provider projection;
- создаёт Communications evidence;
- вызывает Communications runtime, handler или SQL.

Runtime использует один `ManagedControlChannelV2<UnixStream>` и один
correlation-owned reader для descriptor/ready, Storage/Vault, Event Hub, Blob
и public client delivery. `UnixStream::try_clone`, независимые V1 response
readers и `MSG_PEEK` запрещены.

`ready` отправляется только после успешного получения всех обязательных
admitted bindings и открытия owner-local persistence/event connection.

### Private host bridge

`whatsapp.host_bridge.v1` является private host capability, а не Gateway
business API.

Kernel stage-ит short-lived route descriptor, bound к:

```text
owner_id
registration_id
runtime_instance_id
runtime_generation
grant_epoch
route_binding_sha256
```

Tauri host доказывает exact binding при подключении. Remote WebView не видит
socket path или binding и не может выбрать account/event/command metadata.

Host bridge имеет две независимые операции:

1. submit sanitized typed provider observation;
2. lease bounded pending provider command и submit its terminal result.

Cookies, local storage, IndexedDB, session material, raw page payload,
credential plaintext и произвольный JSON через bridge запрещены.

### Public provider operational contracts

Public client surface разделяется по authority:

| Capability | Generated contract | Responsibility |
|---|---|---|
| `whatsapp.command.v1` | `WhatsAppCommandService/ExecuteCommand` | accepted provider mutation command |
| `whatsapp.query.v1` | `WhatsAppQueryService/GetOperationStatus` | owner-local terminal operation status |

Оба route используют exact descriptor-set digest, `major = 1` и explicit
revision. Command grant не выдаёт query rights; query grant не разрешает
provider mutation.

Runtime dispatches по Kernel-supplied exact contract reference и декодирует
только соответствующий generated request. Umbrella `whatsapp.client`, decode
probing, REST alias, proxy, fallback и dual-write после cutover запрещены.

Command response является accepted receipt. `accepted` не означает provider
success. Terminal result сохраняется WhatsApp owner и читается через
`whatsapp.query.v1` либо отдельно admitted realtime contract.

### Event-only Communications handoff

Inbound evidence flow:

```text
External WhatsApp Web
        ↓
host-owned hidden WebView
        ↓ typed private host bridge
WhatsApp runtime
        ↓ owner-local PostgreSQL transaction/outbox
exact DurableEnvelopeV1 bytes
        ↓
NATS JetStream
        ↓
Communications inbox/deduplication
        ↓
Communications-owned state and events
```

Communications получает только provider-neutral typed observation с source
provenance, causation, correlation и canonical runtime identity. Kernel
ограничивает transport и grants, но не является business producer/consumer.

Запрещены direct WhatsApp → Communications RPC/socket/handler, cross-owner SQL,
shared transaction/table, Communications import of WhatsApp DTO/runtime/store
и WhatsApp import of Communications implementation.

Provider-neutral outbound business intent проходит explicit workflow:

```text
source domain event
  -> workflow
  -> whatsapp.command.v1
  -> WhatsApp owner
```

Domain не вызывает WebView/host bridge напрямую.

## Phase gate `whatsapp_integration_v1`

Gate открывается атомарно только при наличии:

1. exact four-package source inventory и compile-isolation guards;
2. one correlated V2 managed-control channel without cloned readers;
3. route-specific generated command/query contracts и exact descriptor;
4. canonical settings schema и immutable owner-local Storage bundle;
5. separate WhatsApp release assembly и signed distribution binding;
6. pending registration плюс explicit owner-approved capability subset;
7. managed launch с exact Storage/Event/host grants и stale/revoke fencing;
8. exact private host route binding and native host-only execution evidence;
9. one live public query and one accepted command with terminal result;
10. one live provider observation → WhatsApp outbox → NATS →
    Communications inbox flow;
11. duplicate delivery and NATS outage replay without data loss;
12. absence of provider bodies, session state, credentials and private route
    bindings in subjects, route metadata, logs, errors and health.

Frontend cutover является отдельным финальным client slice и не доказывает
backend admission. Gate не расширяет `first_owner_v1`, не добавляет WhatsApp
packages в Communications inventory и не делает integration domain.

## Порядок реализации

1. Перевести inherited control transport на один correlation-owned V2 channel.
2. Разделить private host bridge и public command/query contracts; удалить
   umbrella `whatsapp.client`.
3. Добавить exact descriptor/settings/storage builders и отдельную assembly
   unit без signing authority.
4. Доказать signed managed launch, grants, revoke/generation и host-route
   fencing в disposable managed contour.
5. Доказать live host command/result и event-only Communications delivery с
   duplicate/outage replay.
6. Перевести frontend на generated clients и удалить legacy REST/query/realtime
   facade.

Каждый крупный slice является отдельным commit и проходит focused owner tests,
strict Clippy, architecture/SRP/Cargo boundary gates и relevant live
conformance.

## Отклонённые варианты

### Integration вызывает Communications через Kernel

Отклонено: Kernel маршрутизирует platform/client capabilities, но не является
business mediator. Provider evidence пересекает boundary только как typed
durable event.

### Communications владеет WhatsApp operational API

Отклонено: provider commands, session lifecycle и projection принадлежат
integration owner.

### Host bridge является public client route

Отклонено: private native host authority и owner-facing client authority имеют
разные callers, grants и failure modes.

### Один umbrella capability

Отклонено: command, query и host execution имеют разные authority и причины
изменения.

## Последствия

WhatsApp получает отдельный атомарный production gate, который не смешивает
Kernel control plane, integration provider execution и Communications business
ownership. Backend gate открыт только для exact owner-approved admission с
доказанными fences и event flow; он не активирует runtime автоматически и не
закрывает отдельный frontend cutover. Legacy Communications facade не
сохраняется как совместимость.
