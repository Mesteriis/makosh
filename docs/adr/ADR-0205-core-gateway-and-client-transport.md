# ADR-0205: Core Gateway и транспорт клиентских приложений

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Частично реализован foundation: typed Gateway contracts,
browser device/session foundation, exact local/paired listener profiles,
отдельный direct-private-LAN developer profile ADR-0235,
loopback-only HTTP/1.1 serving, paired-remote TLS+HTTP/2 и HTTP/3 serving и
detached technical/browser-auth/realtime routers. Kernel может явно включить
local loopback TLS либо paired-remote profile через полный набор
operator-supplied origin, RP ID и DER certificate/key; remote profile требует
отдельного CLI flag, не получает сертификат автоматически и запускает TCP/TLS
HTTP/2 и UDP/QUIC HTTP/3 на том же configured address/port. H3 endpoint
advertises only `h3`, keeps TLS early data disabled and routes through the same
Gateway adapter; executable conformance currently proves an actual H3
`/healthz` request. Private owner-control теперь может создать
одноразовый browser pairing, а local HTTPS Gateway выдаёт его server-held
WebAuthn registration options и принимает exact-Origin finish. Signed installed
macOS release может также отдать только exact required `browser.bootstrap`
artifact на `/`: bytes повторно проверяются against signed manifest и остаются
в памяти Gateway; filesystem/static-tree fallback запрещён. Single-document
browser bootstrap выполняет owner-approved pairing, WebAuthn registration и
cookie-backed sign-in без external assets или browser token/session storage.
После sign-in он также вызывает typed owner-neutral ConnectRPC
`BrowserSessionService/GetStatus`, который подтверждает только bounded expiry
текущей cookie session и не выдаёт owner data, opaque session ID, credential
или grant. После той же cookie session локальный `ClientBootstrapService/GetBootstrap`
возвращает только approved composition текущего owner: module/registration,
effective capability IDs, grant epoch и client-visible typed settings values.
`hidden` values, raw schema/snapshot bytes, credentials и data другого owner
fail closed и не попадают в response; при non-current settings revision
`sections_enabled=false`. Это bootstrap для включения client sections, не
generic settings mutation/query API и не owner business contract. Owner-neutral local browser contour открыт отдельным
`browser_client_v1` gate по ADR-0232; в нём по-прежнему нет client-safe realtime
owner, ConnectRPC owner contracts, durable receipt mapping, client-safe
realtime replay or full public-client conformance. Поэтому `client_gateway_v1`
остаётся закрытым: remote transport сам по себе не даёт public owner API.

Bootstrap также содержит закрытый static catalog client surfaces. Product
surface может стать `available` только при exact approved capability
`client.surface.<surface>.v1` в composition текущего owner и только когда
его module settings находятся в effective `current` state; та же capability
при non-current settings даёт sanitized `blocked` state. `Settings` остаётся
локальным recovery surface и не требует module grant. Gateway передаёт только
ID/state/reason/contract major: labels, routes, chunks, icons и любой UI-код
остаются compiled в client bundle. До появления owner-specific generated
adapter локальный клиент дополнительно оставляет product route disabled.

Тот же authenticated bootstrap возвращает закрытый typed inventory состояния
Kernel, Control Store, Module Control Plane, Gateway, Vault, Storage Control,
PostgreSQL, PgBouncer, NATS, Event Hub, Scheduler, Clock, Blob, Telemetry и SSE.
Core components считаются healthy только потому, что authenticated snapshot
успешно собран; platform runtime без отдельного live status proof получает
`degraded` или `not_admitted`, а не optimistic healthy. Browser отдельно
измеряет one-shot round-trip initial bootstrap RPC; это client-observed network
latency, она не подменяет server-side subsystem readiness. Периодический
polling всего bootstrap запрещён [ADR-0338](ADR-0338-client-system-status-over-shared-realtime.md):
последующие system-status transitions идут через общий authenticated SSE.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md).

Связано с:

- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

Макошь имеет desktop-клиент в Tauri WebView и планирует Android-клиент. Ни один
клиент не является доверенным участником внутреннего module data plane. Если
клиент знает адреса module processes, NATS subjects, PostgreSQL schema или
внутренние Unix sockets, он обходит capability router, lifecycle state и
failure isolation Kernel.

Одного transport недостаточно для всех видов данных:

- query и typed request требуют немедленного результата;
- durable provider command завершается асинхронно;
- realtime UI требует reconnect и replay;
- большие blobs требуют streaming и range requests;
- OAuth callback зависит от desktop browser redirect или Android app link;
- file picker, hidden WebView, Android lifecycle и системные уведомления
  принадлежат host platform.

Reference frontend уже содержит общий `ApiClient`, realtime bootstrap, cursor
persistence и SSE client. Это подтверждает потребность в одном внешнем entry
point и replayable realtime, но legacy REST surface, handwritten DTO и общий
provider-aware client не являются целевой архитектурой.

Desktop и Android являются единственными first-party product consumers local
application API. Desktop bundle может обновляться вместе с Kernel, но Android
release может временно отставать. Поэтому public client contracts требуют
явной bounded compatibility policy, не превращаясь при этом в permanent legacy
API.

## Решение

### Единственная внешняя граница

Клиентские приложения общаются только с **Core Gateway** Kernel:

```text
Vue / Tauri WebView или Android client
        ↓
Core Gateway
        ↓
identity + capability + contract router
        ↓
domain / workflow / integration runtime через internal IPC или NATS
```

Core Gateway является transport/security adapter, а не business module. Он:

- аутентифицирует client session;
- проверяет origin, когда применимо, capability, contract и protocol version;
- применяет bounded request limits и deadlines;
- добавляет request, trace и correlation identity;
- маршрутизирует запрос к владельцу capability;
- предоставляет generated technical client для Kernel Settings Registry без
  доступа client к module process или Control Store;
- переводит transport/lifecycle failure в typed public error;
- публикует client-safe realtime stream.

Его implementation packages могут напрямую компоноваться только Kernel runtime
как exact Core Gateway adapter; это не разрешает Kernel импортировать любой
domain, integration, workflow или owner implementation package.

Core Gateway не:

- принимает business decisions;
- читает module-owned tables;
- интерпретирует provider payload;
- строит скрытые cross-domain transactions;
- становится generic SQL proxy;
- предоставляет clients доступ к NATS или module sockets.

Module runtimes не слушают client-accessible ports. Клиент не выполняет service
discovery и не выбирает конкретный runtime instance.

### Client topology profiles

Transport contracts не зависят от того, где запущен Kernel.

Поддерживаются два профиля:

1. `local_embedded` — desktop Tauri подключается к Kernel на loopback; если в
   будущем Kernel будет исполняться непосредственно на Android, тот же профиль
   использует Android host bootstrap;
2. `paired_remote` — Android подключается к уже работающему Kernel через
   явно включённый защищённый listener после подтверждённого pairing.

Решение о запуске полного Kernel внутри Android, удалённом подключении к
desktop/home node или поддержке обеих topology фиксируется отдельным ADR.
Настоящее решение гарантирует, что application contracts и generated clients
не придётся менять при выборе topology.

Remote listener выключен по умолчанию. Его включение не делает module runtimes
сетевыми services: наружу по-прежнему доступен только Core Gateway.

### HTTP/2 и HTTP/3 transport profiles

Application contracts и Core Gateway handlers не зависят от TCP или QUIC.
Один и тот же ConnectRPC/HTTP handler обслуживается transport adapters:

```text
local_embedded
  → HTTP/1.1 или HTTP/2 на private loopback transport

paired_remote
  → HTTP/2 over TLS как обязательный baseline
  → HTTP/3 over QUIC как preferred negotiated transport
```

HTTP/3 уже входит в listener abstraction и explicit paired-remote CLI profile.
H3 endpoint исполняет тот же Gateway router, а TCP/TLS HTTP/2 остаётся
обязательным fallback. Recovery-only local IPC ADR-0225 не имеет network
listener. HTTP/3 не откладывается после public gateway как новый application
protocol и не требует других Protobuf contracts.

Технический H3 spike доказывает TLS/ALPN, запрет early data и live `/healthz`
request. Для paired Android client HTTP/3 остаётся запрещён до conformance,
доказывающего:

- Connect unary requests и typed errors поверх выбранного HTTP/3 stack;
- SSE response streaming, heartbeat, cancellation и cursor replay;
- BlobRef upload/download и range semantics;
- authorization metadata, device identity и request limits;
- graceful shutdown и connection drain;
- observability без private payload.

Android может использовать HTTP stack с поддержкой HTTP/3/QUIC. Desktop client
не обязан использовать QUIC на loopback, где его преимущества не компенсируют
UDP listener и дополнительный runtime complexity.

Если UDP/HTTP/3 недоступен, paired client может явно и наблюдаемо перейти на
HTTP/2 с теми же TLS, device identity и capability requirements. Это transport
negotiation, а не замена module implementation или runtime topology. Gateway и
client health показывают фактический `http_version`; silent downgrade до
plaintext или remote HTTP/1.1 запрещён.

QUIC connection migration не считается authentication. Authorization не
привязывается к исходному IP address и продолжает проверять device/session
identity после изменения network path.

HTTP/3 0-RTT/early data для Макошь выключены. Никакие query, command, OAuth,
pairing или blob operations не выполняются в early data до отдельного security
ADR и anti-replay proof.

Не вводится собственный RPC protocol поверх raw QUIC, QUIC datagrams или
WebTransport. HTTP/3 является только transport для тех же HTTP semantics.

Проверенная внешняя основа решения:

- [RFC 9114: HTTP/3](https://www.rfc-editor.org/rfc/rfc9114.html) определяет
  HTTP semantics поверх QUIC, TLS 1.3 transport protection и replay risk 0-RTT;
- [RFC 9000: QUIC](https://www.rfc-editor.org/rfc/rfc9000.html) определяет
  streams и connection migration;
- [Android Cronet](https://developer.android.com/develop/connectivity/cronet)
  поддерживает HTTP/1.1, HTTP/2 и HTTP/3 over QUIC;
- [Connect protocol](https://github.com/connectrpc/connect-go) использует
  обычные HTTP handlers и документирует HTTP/1.1/HTTP/2; HTTP/3 поэтому требует
  отдельной Макошь conformance проверки выбранного server/client stack.

### Command/query transport

Основным client transport для typed query, request и command принимается
**ConnectRPC с Protobuf contracts** поверх HTTP transport profiles, описанных
выше.

Каждый contract принадлежит одному owner/capability, например:

```text
mail.operational.v1
telegram.operational.v1
zulip.operational.v1
communications.evidence.v1
personas.v1
tasks.v1
knowledge.v1
```

Protobuf package владеет wire schema и генерирует Rust, TypeScript и client code
для выбранного Android stack из одного source. Generated code не редактируется
вручную. Каждый клиент использует один transport factory для authentication,
deadlines, tracing и typed error details, но не универсальный handwritten
`ApiClient` с business методами.

Provider experience может использовать собственный operational client и
neutral context clients. Он не получает общий union DTO всех providers.
Business/context UI не импортирует provider operational client.

Общий contract package содержит только действительно технические primitives:
identifiers, timestamps, pagination, request metadata, field violations и
typed transport errors. Он не становится `common`-контейнером для business
models разных owners.

### Settings control plane

Settings desktop и Android получают через отдельный generated technical
contract Kernel Settings Registry ADR-0222. Gateway может вернуть composed
catalog sections нескольких owners для одного экрана, но каждая entry сохраняет
собственные registration/capability target, authority, schema hash, desired и
effective revisions.

Mutation всегда адресует ровно один owner target, содержит expected desired
revision и проходит OwnerAuthority. Cross-owner transaction, общий merged
settings document и скрытый вызов module endpoint запрещены. Client contract
предоставляет write только для сочетания `operator_managed + editable`.
`operator_managed + read_only`, `operator_managed + hidden` и все
`kernel_managed` entries не имеют generic client write method.

Authority и `client_visibility` являются независимыми осями ADR-0222.
Authorized `read_only` entry любого authority class может вернуть typed value
либо schema-defined sanitized representation. `hidden` entry не возвращает
value обычному settings query; diagnostics может вернуть только sanitized
state/reason. `kernel_managed` никогда не является `editable`. Gateway не
интерпретирует setting semantics и не читает SQLite rows напрямую.

### Query semantics

- Query является read-only и возвращает typed snapshot с revision/cursor,
  когда последующее realtime обновление должно быть упорядочено.
- Pagination всегда bounded; unbounded list endpoint запрещён.
- Runtime unavailability возвращается как typed module/capability state, а не
  пустой успешный результат.
- Клиент может выполнять независимые domain queries параллельно.
- Повторяющаяся cross-domain read composition оформляется отдельным versioned
  application read contract. Она не встраивается как business logic в Core
  Gateway.

### Command semantics

Локальный request с завершённым результатом может вернуть typed response сразу.
Durable, realtime или внешний provider command возвращает только receipt:

```text
CommandReceipt
  command_id
  initial_message_id
  idempotency_key
  accepted_at
  initial_state = accepted
  correlation_id
```

`accepted` означает durable acceptance, а не успешное внешнее действие.
`command_id` является identity logical command, а `initial_message_id` — exact
первого `DurableEnvelopeV1`. Broker redelivery сохраняет оба значения; явный
logical retry сохраняет `command_id`/idempotency key, но получает новый
`message_id` по ADR-0220.
Terminal state приходит через realtime event и доступен отдельным status query:

```text
succeeded | failed | cancelled | expired | rejected | unknown_outcome
```

Повторная отправка того же idempotency key возвращает тот же receipt/result.
Клиент не ретраит non-idempotent command после ambiguous transport failure
без получения его status. `unknown_outcome` никогда не маскируется как success.

### Realtime transport

Backend-to-client realtime использует один authenticated multiplexed
**Server-Sent Events** stream:

```text
GET /api/realtime/v1/events
```

Каждый активный foreground client process поддерживает одно физическое
соединение. Core Gateway фильтрует stream по capabilities client session.
Client platform layer декодирует `ClientRealtimeFrameV1` один раз и передаёт
client-safe event owner-specific cache handler. Cursor принадлежит конкретному
client/device и не изменяет replay position других устройств.

`ClientRealtimeFrameV1` принадлежит `makosh-gateway-protocol` и физически
отделён от внутреннего `DurableEnvelopeV1` ADR-0220. Gateway никогда не
сериализует internal envelope клиенту напрямую. Frame имеет `oneof` для
client event, replay gap и stream state.

Минимальный client event envelope содержит:

- `event_id` и opaque monotonic `cursor`;
- `contract_name` и `contract_version`;
- `event_kind`;
- `occurred_at`;
- `causation_id`, `correlation_id` и `trace_id`, когда применимо;
- bounded versioned payload.

Internal source/runtime generation, partition key, source fence, idempotency,
provider cursor, NATS subject/delivery и quarantine metadata в client frame не
попадают. Owner должен объявить event contract `client_safe` либо создать
отдельный client-safe event; похожая форма payload не является разрешением
автоматически пробросить internal event.

Reconnect передаёт последний подтверждённый device-local cursor. Повторное
событие дедуплицируется по `event_id`. Если replay cursor больше недоступен,
Gateway явно отправляет gap/lag state, а client invalidates затронутые owner
queries и получает новый snapshot. Silent skip запрещён.

SSE сообщает state transition, changed identifiers, revisions, progress,
module health, settings apply metadata и terminal command results. Raw settings
values/diffs, private message bodies, document contents, media bytes,
credentials и provider sessions в общий realtime envelope не помещаются.
Client получает authorized values отдельным settings query, а private content —
отдельным query/blob request. Высокочастотные invalidations объединяются в
bounded batch.

SSE client использует fetch-stream или другой механизм с authorization header.
Client-session capability запрещено передавать в query string. Native browser
`EventSource` допускается только для exact same-origin route с HttpOnly browser
session cookie: он не принимает token/cursor в URL и использует стандартный
`Last-Event-ID` только для device-local reconnect replay. `EventSource`,
требующий token в URL, не используется.

### Android lifecycle, offline и notifications

Android не гарантирует постоянно работающий background connection. Поэтому SSE
является foreground realtime transport, а не механизмом фонового исполнения.
При pause/termination Android сохраняет только безопасный replay cursor, а при
resume восстанавливает stream и догоняет события через replay.

Отсутствие активного Android process не влияет на ingestion, provider runtimes
или domain workflows Kernel. Клиентская offline cache не становится canonical
truth.

Если Android должен принимать commands без связи, они сохраняются в отдельном
device outbox с globally unique idempotency keys и отправляются после
reconnect. Такая очередь не выдаёт локальное принятие за acceptance Kernel и
требует отдельного offline-sync ADR до реализации.

Push notifications не являются transport для domain state. Возможный FCM или
другой push provider рассматривается отдельным privacy ADR; push может нести
только opaque wake-up/invalidation signal без private content. Полное состояние
после wake-up читается через Core Gateway.

Connect server streaming и WebSocket не используются как базовый realtime
transport. Calls, live presence, typing или другая доказанная bidirectional
low-latency capability может получить отдельный transport только через новый
ADR.

### HTTP endpoints вне ConnectRPC

Обычный HTTP остаётся только там, где он лучше RPC:

- `GET /healthz` и `GET /readyz`;
- OAuth redirect callbacks с обязательными state/PKCE checks;
- SSE stream;
- blob upload, download, preview и range requests.

Blob API принимает только opaque expiring `BlobRef`/capability. Client не
передаёт filesystem path и не получает произвольный доступ к локальной файловой
системе. Blob bytes не инкапсулируются в Protobuf, SSE или NATS.

Business REST/JSON endpoints не создаются параллельно ConnectRPC services.

### Host-specific client bridges

Desktop Tauri IPC используется только для desktop/OS capabilities:

- bootstrap адреса Gateway, inherited first-enrollment channel и platform
  `DeviceSigner`;
- file/folder picker;
- window и application lifecycle;
- system notifications;
- hidden provider WebView и строго ограниченный bridge;
- platform-specific secure user interaction с vault.

Android host bridge аналогично ограничен Android lifecycle, secure storage,
file/media picker, app links и system notifications. Он не является вторым
application API.

Tasks, Personas, communications, provider operational queries/commands и
другие business operations через Tauri/Android host bridge запрещены. Иначе
появятся несколько несогласованных public API и обход Core Gateway.

### Client transport security

Для `local_embedded` profile Core Gateway:

- слушает только loopback interface или эквивалентный private app transport;
- проверяет точный Tauri origin/host и не использует wildcard CORS;
- аутентифицирует отдельную persistent desktop `DeviceIdentity` через
  proof-of-possession;
- выдаёт отдельную ephemeral client-session capability на каждый Kernel run;
- принимает initial owner/device enrollment только через one-time inherited FD
  на pristine instance, а не через публичный unauthenticated endpoint.

Для `paired_remote` profile:

- remote listener выключен по умолчанию и включается только явным owner action;
- используется encrypted authenticated transport; plaintext remote HTTP
  запрещён;
- TCP/HTTP/2 и UDP/HTTP/3 listeners используют одну server identity и
  эквивалентную authorization policy;
- pairing требует явного подтверждения владельца и одноразового challenge;
- Android генерирует отдельную non-exportable ES256 device key в Android
  Keystore;
- Gateway выдаёт revocable device identity с явными capabilities;
- клиент проверяет identity Kernel и не доверяет произвольному LAN endpoint;
- каждый device имеет независимые sessions, replay cursors, audit и revoke;
- компрометация одного device не выдаёт credentials другого device или module.

Для обоих profiles Gateway:

- применяет challenge-response, device revocation и session fencing ADR-0218;
- принимает session capability/proof только в authorization metadata/header;
- не принимает tokens в URL, logs или error payload;
- ограничивает body size, concurrency, deadlines и idle connections;
- завершает client sessions при shutdown/restart Kernel;
- не открывает module runtime ports через remote listener.

Client-session capability хранится только в памяти client process и не попадает
в `localStorage`, IndexedDB, Android preferences, URL, analytics, crash report
или logs. Долговременная desktop/Android device key хранится только в platform
signer, не экспортируется и не доступна Vue/WebView или Kernel. Persisted
realtime cursor не является credential и не даёт права на чтение.

### Errors

Public typed error включает минимум:

- стабильный machine-readable `error_code`;
- retryability;
- field violations для validation failure;
- correlation ID;
- module/capability state, когда применимо;
- безопасные display details без private content.

Localization принадлежит client: backend возвращает code и bounded
parameters, но не владеет русскими/английскими UI strings. Internal stack
traces, SQL errors, provider payload и secrets не пересекают Gateway.

### Contract evolution и cutover

Backward-compatible изменение public client contract:

- только добавляет optional fields/methods или новые enum values с безопасным
  unknown handling;
- не переиспользует удалённые Protobuf field numbers;
- не меняет semantics существующего field без version bump;
- проверяется generated clients всех поддерживаемых platforms.

Несовместимое изменение создаёт новый major contract. Desktop bundle
переключается вместе с Kernel. Для Android Gateway поддерживает текущий и один
непосредственно предшествующий public client major (`N` и `N-1`) в bounded
release window. После завершения window старый client получает typed
`UpgradeRequired` до выполнения business operation.

Каждый major cutover включает:

1. Protobuf contract;
2. runtime implementation;
3. generated Rust, TypeScript и Android client code;
4. desktop/Android query, mutation и cache handlers;
5. tests/fixtures;
6. удаление истёкшего `N-1` contract/path/client после release window.

Compatibility window является явной public mobile policy, а не разрешением на
permanent facade. Compatibility aliases без отдельного versioned contract,
неограниченная поддержка старых major и скрытые fallback запрещены.

## Запрещено

- client → module runtime напрямую;
- client → NATS, PostgreSQL, PgBouncer или module Unix socket;
- client-visible port отдельного domain/integration runtime;
- generic handwritten `ApiClient` с методами всех owners;
- business REST endpoint рядом с эквивалентным ConnectRPC method;
- business/domain operations через Tauri/Android host bridge;
- token/capability в URL, SSE query string, persistent browser storage или
  unprotected Android storage;
- private bodies, media, secrets или provider sessions в SSE envelope;
- silent realtime gap или cursor reset;
- client retry non-idempotent command без status reconciliation;
- GraphQL gateway, WebSocket или Connect streaming как скрытый второй default
  transport;
- HTTP/3-only remote API без защищённого HTTP/2 fallback;
- raw QUIC/WebTransport protocol, дублирующий ConnectRPC/SSE contracts;
- 0-RTT/early data для Макошь requests;
- business composition внутри Core Gateway;
- cross-owner settings mutation или client write к `kernel_managed` field;
- raw settings value/diff в SSE, logs, health или transport error.

## Отклонённые варианты

### Клиент подключается к каждому module process

Отклонено: раскрывает topology, усложняет discovery/auth и позволяет client
обходить capability router и lifecycle state.

### Только REST/JSON

Отклонено как основной business transport: handwritten DTO и clients быстро
расходятся, а contract ownership становится неявным. HTTP остаётся для
streaming/redirect primitives.

### GraphQL gateway

Отклонено: единая schema поощряет cross-domain ownership и скрытую business
composition в Gateway. Отдельные application read contracts выражают реальные
составные use cases явно.

### WebSocket для всего

Отклонено: bidirectional connection добавляет собственные ack, reconnect,
ordering и backpressure semantics без текущей необходимости. Commands остаются
typed ConnectRPC, realtime — replayable SSE.

### Только HTTP/3

Отклонено: UDP может быть заблокирован сетью, а Android должен сохранять
доступность через защищённый HTTP/2. HTTP/3 является preferred transport, а не
единственной точкой входа.

### Собственный RPC поверх raw QUIC

Отклонено: заставляет Макошь заново проектировать framing, errors, flow control,
interoperability и security вместо использования стандартных HTTP semantics.

### NATS в клиенте

Отклонено: desktop/Android client не является module runtime, а NATS subjects и
credentials — внутренняя capability boundary.

### Host bridge как business API

Отклонено: связывает доменные contracts с desktop/Android shell и создаёт второй
путь в обход Gateway.

## Последствия

Положительные:

- desktop и Android имеют один защищённый entry point;
- Rust, TypeScript и Android client используют одну typed schema;
- domain/provider ownership остаётся видимым в service packages;
- realtime имеет reconnect, deduplication и replay;
- durable command не выдаёт acceptance за completion;
- blobs и OS capabilities используют подходящий им transport;
- module restart не требует переподключать clients к новому process address;
- Android может безопасно догнать события после background suspension;
- bounded `N`/`N-1` policy учитывает несовпадающий release cadence;
- HTTP/3 можно включить для Android без изменения application contracts;
- защищённый HTTP/2 сохраняет работу в сетях без QUIC/UDP;
- composed settings UI не требует подключения client к module runtimes и не
  стирает owner/revision границы.

Отрицательные:

- необходимо сопровождать Protobuf codegen для Rust, TypeScript и Android;
- Gateway должен иметь строгие auth, limits и error translation;
- query snapshot и SSE cursor должны быть согласованы;
- client cache handlers должны поддерживать replay gap и invalidation;
- один user action иногда проходит как command receipt + realtime completion;
- Tauri/Android identity protocol определён ADR-0218, но его native signer,
  pairing и remote transport требуют отдельного security testing;
- Android offline outbox и push notifications потребуют отдельных решений до
  реализации;
- HTTP/3 добавляет UDP exposure, transport metrics, certificate и conformance
  surface;
- одновременная поддержка HTTP/2 и HTTP/3 увеличивает integration test matrix.

## Проверка решения

До признания реализации завершённой должны существовать executable checks:

- generated Rust, TypeScript и Android client code строятся из одного
  descriptor set;
- desktop/Android import guard запрещает handwritten business REST clients;
- client не может подключиться к module process, NATS или PostgreSQL;
- неизвестный service/contract version и отсутствующая capability fail closed;
- client-session token отсутствует в URL, unprotected persistent storage, logs
  и errors;
- desktop first enrollment проходит только через inherited FD на pristine
  instance;
- desktop и Android используют разные device keys; revoke одного не меняет
  ключ другого;
- privileged owner operation требует fresh operation-bound challenge и user
  presence;
- wildcard origin/CORS, unauthorized remote bind и plaintext remote transport
  отклоняются;
- Android pairing, Kernel identity verification, device revoke и account
  mismatch;
- ConnectRPC unary/error conformance через HTTP/2 и HTTP/3;
- HTTP/3 blocked/timeout выполняет наблюдаемый fallback на защищённый HTTP/2;
- Wi-Fi/LTE path change не теряет device identity и восстанавливает stream;
- 0-RTT request отклоняется до application handler;
- независимые desktop/Android cursors, SSE reconnect, duplicate event, replay,
  lag и unrecoverable gap;
- crash/restart module runtime не завершает Gateway и соседние screens;
- durable command duplicate, timeout, terminal result и `unknown_outcome`;
- desktop/Android clients не считают `accepted` завершённым provider action;
- private content и secrets отсутствуют в SSE envelope;
- settings catalog сохраняет owner target/schema hash/desired/effective
  revisions, а mutation требует expected revision;
- authority/client visibility matrix соблюдается полностью: client меняет
  только `operator_managed + editable`, authorized `read_only` получает typed
  value либо schema-defined sanitized representation, а `hidden` value не
  возвращается;
- client не может менять `kernel_managed` setting или одним request изменить
  targets нескольких owners;
- settings SSE содержит только sanitized revision/apply metadata без raw
  values/diffs;
- BlobRef expiry, account mismatch, range request и path traversal denial;
- Tauri/Android host bridge guard допускает только утверждённые platform
  capabilities;
- `N`, `N-1` и `UpgradeRequired` проходят contract tests;
- истёкший compatibility window не оставляет старый route/client/facade;
- Android resume после process death восстанавливает состояние через replay;
- push payload, если он будет введён отдельным ADR, не содержит private content.
