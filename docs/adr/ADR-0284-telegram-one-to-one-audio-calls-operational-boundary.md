# ADR-0284: Telegram one-to-one audio calls operational boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: частично реализовано. `telegram_call_history_v1` и
`telegram_call_signaling_v1` реализованы через generated Query/Command/Realtime
contracts, typed `updateCall` projection, owner-local PostgreSQL
history/replay/operation journal и exact managed-runtime routes с
restart/stale-fence conformance. Command executor возобновляет durable
operations в owner runtime loop, а exact TDLib `createCall`, `acceptCall` и
`discardCall` requests проверены отдельно. Managed conformance подтверждает
durable `InitiateAudioCall`/`EndCall`, provider-result reconciliation,
idempotency conflict и restart. `telegram_call_media_v1` и umbrella
`telegram_calls_operational_v1` остаются закрыты. Для media реализованы
отдельные typed contract и
`makosh-telegram-call-media-tgcalls`, exact-source build script, native C ABI,
system-audio implementation patch, exact dylib loader и Kernel-staged
assembly/runtime binding. TDLib `callStateReady` и bidirectional signaling
преобразуются в secret-safe media plan, исполняются через pinned tgcalls port и
дают отдельную durable media projection. Exact V6 storage bundle проходит
additive DDL admission, disposable PostgreSQL test и managed-runtime fixture
conformance для ready/signaling, terminal teardown, revoke/restart и stale
runtime generation. Исторические V3 realtime frames копируются только
restart-safe owner-local Job Platform executor после additive DDL admission:
257-frame PostgreSQL и signed managed-runtime conformance подтверждают bounded
checkpoint/resume, cursor-preserving V3/V4 order, generation/lease fencing и
terminal replay до readiness. Backfill и exact Calls Command/provider-result
prerequisites signaling gate выполнены. Реализованное всё ещё не доказывает
real input/output audio loop, upstream tgcalls memory/thread conformance или
authorized live one-to-one call.
Добавлен отдельный development-only CoreAudio conformance build unit: он
использует те же exact upstream/Bazel/license pins, помечает artifact как
`release_eligible: false` при несовпадении Xcode с release pin и требует явный
runtime-флаг до доступа к microphone/speaker. Probe не входит в Telegram
assembly, не сохраняет input samples, отдаёт в playout только silence,
проверяет bounded full-duplex callbacks и восстанавливает исходный mute state.
Само наличие или сборка probe не открывает gate: требуются его разрешённый
запуск на real devices, exact Xcode 26.2 release build и authorized live call.
Development profile подтверждён сборкой arm64 dylib и test-only probe на
активном Xcode 26.6; provenance зафиксировал `release_eligible: false`, а exact
Rust loader/protocol test прошёл с собранной dylib. Probe не запускался и доступ
к microphone/speaker не запрашивался, поэтому real audio evidence отсутствует.
Historical fixture transcript не является evidence ни для одного gate этого
ADR.

Уточняет:

- [ADR-0213: code ownership and module autonomy](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0240: Telegram clean-room provider boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0256: owner-declared client RPC route admission](ADR-0256-owner-declared-client-rpc-route-admission.md);
- [ADR-0266: Telegram admission and event-only Communications handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0267: Kernel-staged runtime artifacts](ADR-0267-kernel-staged-runtime-artifacts-and-integration-state-roots.md);
- [ADR-0268: Telegram release composition](ADR-0268-telegram-release-assembly-unit-and-signed-distribution-fragment.md);
- [ADR-0282: full Communications and Settings reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Historical implementation подтверждает только общий call metadata REST surface,
PostgreSQL projection и fixture transcript. Она не выполняла TDLib
`createCall`, `acceptCall` или `discardCall`, не обрабатывала live
`updateCall` и не поднимала реальный Telegram audio transport. Архивный ADR
также явно оставлял TDLib, desktop audio capture и transcription заблокированными.

Копирование generic `calls` service или генерация fixture transcript поэтому
создали бы fake completion.

Официальный TDLib contract разделяет responsibilities:

- `createCall` и `acceptCall` принимают поддерживаемый application call
  protocol;
- `updateCall` сообщает создание и изменение provider call state;
- `call.id` является непостоянным, а `call.unique_id` — persistent provider
  identity после назначения сервером;
- `callStateReady` передаёт servers, call config, encryption key и custom
  parameters для `tgcalls`;
- `discardCall` требует фактические duration и connection identity после
  media session.

Следовательно, TDLib signaling без работающего `tgcalls` media adapter не
является перенесённым Telegram call.

## Решение

### Scope и owner

`telegram_calls_operational_v1` принадлежит Telegram integration и закрывает
только one-to-one audio calls:

```text
Telegram Calls UI
        ↓
generated Telegram calls clients
        ↓
Core Gateway owner-neutral routing
        ↓
Telegram runtime
        ├─→ TDLib signaling adapter
        ├─→ Telegram call state/persistence
        └─→ tgcalls media adapter → system input/output
```

В scope входят:

- incoming и outgoing one-to-one audio calls;
- live provider call state and durable owner-local history;
- initiate, accept, decline and end operations;
- exact media-session lifecycle;
- local mute state and default system input/output;
- terminal provider/media result and operation audit.

Не входят:

- video, group calls, screen sharing or live streams;
- hidden recording;
- transcription, summary or AI processing;
- cross-provider Calls aggregation;
- provider-neutral business actions.

Расширение этого scope требует отдельного ADR и phase gate.

### Независимые gates

Umbrella gate не открывается одним package test:

| Gate | Responsibility | Depends on |
|---|---|---|
| `telegram_call_history_v1` | typed `updateCall` projection, query and replay | `telegram_core_operational_v1` |
| `telegram_call_signaling_v1` | initiate/accept/decline/end and durable operation results | `telegram_call_history_v1` |
| `telegram_call_media_v1` | real `tgcalls` audio session, mute, duration and connection result | `telegram_call_signaling_v1` |
| `telegram_calls_operational_v1` | complete provider call experience | all three gates above |

History может быть доказана fixture TDLib stream и disposable PostgreSQL.
Signaling требует exact TDLib request/response conformance. Media gate требует
реальный native adapter and audio loop; fixture PCM или синтетический terminal
result не закрывают production admission.

### Единицы сборки

Calls добавляет Telegram-owned units:

```text
makosh-telegram-calls-api
makosh-telegram-calls-core
makosh-telegram-calls-persistence
makosh-telegram-call-media-contract
makosh-telegram-call-media-tgcalls
```

Их причины изменения:

- `calls-api` — generated client query/command/realtime schemas, exact routes
  and schema digests;
- `calls-core` — bounded call state machine, operation validation, identity
  binding and typed media plans; без PostgreSQL, TDLib, native FFI, Kernel и
  Communications;
- `calls-persistence` — Telegram-owned history, call identity binding,
  operation journal, idempotent results and restart replay;
- `call-media-contract` — narrow owner-local audio-session port and typed
  session/result models without native implementation;
- `call-media-tgcalls` — tgcalls/native FFI, system audio device lifecycle,
  encryption/session material consumption and media callbacks.

Существующие units сохраняют свои responsibilities:

- `makosh-telegram-tdlib` кодирует call signaling и преобразует `updateCall` в
  typed provider updates;
- `makosh-telegram-runtime` координирует calls ports under current
  runtime/storage/grant fences;
- `makosh-telegram-assembly` materializes descriptor, storage and exact native
  artifact binding;
- generic distribution compiler проверяет и подписывает полный release;
- frontend `src/integrations/telegram` владеет generated clients, controller и
  provider presentation.

Media adapter является integration adapter/build unit. Он не является domain,
workflow, assembly или independently managed module runtime. Runtime не
компилирует assembly и не получает signing authority.

### Public contracts

Вводятся три exact client routes:

| Capability | Contract | Route |
|---|---|---|
| `telegram.calls.query.v1` | `telegram.calls.query.v1` | `/makosh.telegram.calls.v1.TelegramCallsQueryService/Query` |
| `telegram.calls.command.v1` | `telegram.calls.command.v1` | `/makosh.telegram.calls.v1.TelegramCallsCommandService/Execute` |
| `telegram.calls.realtime.v1` | `telegram.calls.realtime.v1` | `/makosh.telegram.calls.v1.TelegramCallsRealtimeService/Replay` |

Query contract содержит:

- `ListCalls`;
- `GetCall`;
- `GetActiveCall`;
- `ListCallOperations`;
- `GetCallOperation`.

Command contract содержит:

- `InitiateAudioCall`;
- `AcceptAudioCall`;
- `DeclineCall`;
- `EndCall`;
- `SetLocalMute`.

Realtime contract replayable и переносит только typed provider/media call
frames. Он имеет monotonically increasing owner-local sequence and explicit
reset/gap semantics. Internal durable envelope клиенту не выдаётся.

Client command возвращает durable acceptance/receipt. Accepted не означает
ringing, connected или ended; terminal provider/media result приходит через
realtime или operation query.

### Identity и state

Макошь создаёт stable owner-local `call_session_id`. TDLib `call.id` хранится
только как current runtime-scoped signal identity и никогда не используется
как durable key самостоятельно. Ненулевой TDLib `call.unique_id` атомарно
привязывается к session как persistent provider identity.

Typed call projection содержит:

- Telegram `account_id`;
- owner-local `call_session_id`;
- optional persistent `provider_call_unique_id`;
- exact other-party `provider_user_id`;
- incoming/outgoing direction;
- audio media kind;
- normalized provider/media state;
- created, ringing, connected and ended timestamps when observed;
- terminal discard/error category;
- monotonic projection revision.

Raw TDLib JSON, generic metadata maps, `Any` и opaque payload bytes запрещены.
Private message content и contact display snapshots не являются call identity.

Incoming updates могут создать session до появления persistent provider
identity. Повторный update обязан сходиться к той же session. Conflicting
binding fail closed and emits sanitized diagnostics without overwriting
history.

### Commands, idempotency и fencing

Каждая command имеет client-generated idempotency key, exact account/session
scope and current runtime/grant fence.

- retry same key and same payload возвращает persisted exact receipt/result;
- same key with different payload is rejected;
- stale runtime generation or grant epoch is rejected before provider/media
  mutation;
- only one non-terminal call session per Telegram account is admitted in v1;
- initiate is audio-only and rejects self/unknown provider user identity;
- accept/decline require exact incoming provider state;
- end requires exact active or pending session;
- mute is local media state and never encoded as provider business command.

Crash before durable acceptance creates no accepted receipt. Crash after
acceptance must resume or reconcile using persisted operation and subsequent
TDLib updates; it must not blindly repeat a non-idempotent provider call.
После смены runtime/grant fence неотправленный `accepted` завершается как
permission-fenced, а `dispatching`/`awaiting_provider` с неоднозначным provider
outcome — как explicit `Unknown`; новый runtime не повторяет TDLib mutation.

### TDLib и media boundary

TDLib adapter owns:

- `createCall`;
- `acceptCall`;
- `discardCall`;
- parsing every supported `updateCall` state;
- provider error translation without untrusted detail leakage.

Media adapter starts only from an exact typed ready plan derived from
`callStateReady`. The ready plan contains protocol versions, server endpoints,
P2P allowance, call config, custom tgcalls parameters and ephemeral key
material required by the native library.

Encryption key, raw config and native debug log:

- never persist in PostgreSQL;
- never enter durable events, client frames, health, logs or errors;
- stay in bounded process memory only for the current fenced session;
- are zeroized/released on end, failure, runtime restart or grant revoke.

The exact tgcalls library version and native bytes are release artifacts pinned
by the Telegram assembly fragment. Runtime path settings, arbitrary native
library lookup and fallback to unverified system bytes are forbidden.

Pinned native release inputs для первой реализации:

- Telegram-iOS
  `6ad963e5b62d354da79040f388ae2b9132fb17b8`;
- tgcalls `e3069322a3d1e16ecb11a5e302242e59ddd7f09e`, LGPL-3.0 license bytes
  `da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768`;
- WebRTC `3817e906cb6c22ec9cc62023b073e1a668d9cb33`;
- Bazel 8.4.2 bytes
  `45e9388abf21d1107e146ea366ad080eb93cb6a5f3a4a3b048f78de0bc3faffa`;
- Xcode 26.2, как требует pinned Telegram-iOS `versions.json`.

Telegram-iOS `tgcalls_core` не включает macOS
`AudioDeviceModule::Create` implementation в final consumer. Exact source
patch добавляет отдельный CoreAudio target из pinned WebRTC sources; production
bridge не линкует `FakeAudioDeviceModule`, synthetic recorder или no-op
renderer. Локальная сборка другим Xcode может использоваться только как
development evidence ABI/loader; release artifact обязан собираться exact
script с toolchain check и provenance manifest.

The media adapter reports typed connected/disconnected state, selected
connection identity, monotonic duration and sanitized failure category. These
values are the only source for the corresponding `discardCall` fields.

### Persistence

Calls persistence is part of the Telegram owner storage bundle but remains a
separate package and migration group. It owns:

- call session projection and provider identity binding;
- append-only provider/media state history;
- command acceptance and terminal result journal;
- realtime sequence/outbox and replay cursor;
- local mute projection needed for restart-safe UI state.

Ephemeral call encryption material, raw native config, PCM/audio bytes and
native debug logs are forbidden in PostgreSQL.

### Communications, workflows и Settings

Telegram Calls does not import Communications and does not create generic
cross-provider Call objects.

Future provider-neutral call evidence crosses the owner boundary only through
a separate typed durable event and is consumed under
`communications_call_evidence_v1`. Communications must not receive media keys,
PCM, provider debug logs or Telegram operational commands.

Recording/transcription belongs to `call_transcription_v1`, with explicit user
consent, Blob custody and a separate transcription workflow/engine. It is not
implemented by the Telegram runtime or call media adapter. Historical
`enable_call_transcription` therefore must not appear in Telegram integration
Settings.

Provider-specific Calls UI lives in `frontend/src/integrations/telegram`.
Cross-provider Calls composition, after its own gate, lives in
`frontend/src/app`. Any transcription preference is mounted from the
transcription owner after admission. App or platform Settings do not absorb
Telegram call behavior.

### Admission evidence

`telegram_call_history_v1` opens only after:

1. generated query/realtime contracts and schema digests;
2. exhaustive typed TDLib call-state parsing;
3. disposable PostgreSQL migration and projection conformance;
4. duplicate/out-of-order update, identity binding and process replay tests;
5. exact Gateway/runtime route and stale-fence tests.

`telegram_call_signaling_v1` additionally requires:

1. exact TDLib initiate/accept/discard request tests;
2. durable idempotency and operation reconciliation;
3. durable V3 history-to-V4 realtime backfill through Job Platform;
4. negative-state, authorization and restart tests;
5. live managed route evidence without generic REST fallback.

`telegram_call_media_v1` additionally requires:

1. pinned tgcalls source/version/license and reproducible native artifact;
2. exact assembly/runtime artifact binding;
3. native FFI ownership, thread, callback and memory-safety tests;
4. real input/output audio loop and mute/duration/connection conformance;
5. secret-negative logs/storage/client-output tests;
6. teardown on end, crash, restart, revoke and stale generation;
7. an authorized live one-to-one audio call smoke test.

`telegram_call_signaling_v1` закрыт exact TDLib request tests, disposable
PostgreSQL operation-journal conformance, V3→V4 backfill evidence и signed
managed route, который проверяет durable initiate/end receipt, provider update
reconciliation, idempotency conflict, restart и stale runtime fence. Это не
закрывает `telegram_call_media_v1`.

`telegram_calls_operational_v1` opens only after all three gates, generated
frontend cutover and removal of historical Calls REST/fixture fallbacks.

## Consequences

- Calls no longer masquerade as a generic domain or fixture transcription
  service.
- TDLib call signaling and real audio transport are independently testable.
- Native media changes do not force policy or persistence ownership into one
  package.
- Full production admission requires real media evidence and cannot be claimed
  from contracts or fixtures alone.
- Transcription and cross-provider aggregation remain separately admitted
  workflows/composition.

## External contract evidence

- [TDLib `createCall`](https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1create_call.html)
- [TDLib `acceptCall`](https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1accept_call.html)
- [TDLib `discardCall`](https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1discard_call.html)
- [TDLib `updateCall`](https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1update_call.html)
- [TDLib `callStateReady`](https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1call_state_ready.html)
- [Telegram Calls Library](https://github.com/TelegramMessenger/tgcalls)
