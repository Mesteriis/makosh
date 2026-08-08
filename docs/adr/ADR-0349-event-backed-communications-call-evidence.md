# ADR-0349: Event-backed Communications call evidence

Статус: Принято

Дата: 2026-07-30

Состояние реализации: реализовано. Ingress/client contracts, core, owner-local
persistence, revision 15 существующего Communications storage bundle, managed
event consumer, generated list/get, replayable client realtime и
Telegram-owned producer/outbox adapter подтверждены live managed conformance.
Gate `communications_call_evidence_v1` открыт. Telegram Calls остаётся отдельной
integration build unit; это не открывает `telegram_call_media_v1` и не добавляет
Telegram runtime в production inventory.

Уточняет:

- [ADR-0201: Core module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: bundled integration plugins](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0240: canonical Communications migration](ADR-0240-canonical-communications-owner-clean-room-migration.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0284: Telegram one-to-one audio calls](ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md);
- [ADR-0350: explicit human owner context](ADR-0350-explicit-human-owner-context-for-managed-domain-and-integration-runtimes.md).

## Контекст

Provider integrations владеют operational call lifecycle. Communications
должен показывать provider-neutral metadata evidence и позже отдавать его
Review, transcription и application composition. Прямой вызов Telegram,
WhatsApp или будущего meeting integration из Communications нарушил бы owner
boundary. Общий `CallsService`, который проксирует provider commands или читает
integration tables, был бы фасадом, а не clean-room переносом.

Legacy хранил raw provider call/account/chat identifiers, generic JSON metadata,
transcript policy и transcript text в общей calls-модели. Эти поля не
переносятся:

- provider locators остаются integration-owned;
- transcript и recording принадлежат отдельным consent-bound workflows;
- arbitrary JSON, `Any`, generic maps и provider command state запрещены;
- media keys, PCM, native logs, credentials и provider sessions не являются
  evidence.

## Решение

`communications_call_evidence_v1` принадлежит Communications domain и состоит
из независимых функциональных единиц:

```text
integration runtime
    └─ provider call projection/outbox producer
           ↓ typed durable observation
makosh-communications-call-evidence-ingress
           ↓
makosh-communications-call-evidence-core
           ↓
makosh-communications-call-evidence-persistence
           ↓
makosh-communications-runtime adapter
           ↓
generated Communications call query + shared client realtime
```

Communications assembly включает contract descriptors и storage migration в
существующий Communications release fragment. Нового managed Calls runtime,
generic call platform или provider facade не вводится.

### Build-unit ownership

- `call-evidence-ingress` владеет только exact Protobuf observation, schema
  digest, route requests, opaque locator derivation и durable envelope builder;
- `call-evidence-core` владеет только provider-neutral validation и monotonic
  evidence projection;
- `call-evidence-persistence` владеет inbox/hash fence, canonical rows,
  lifecycle history, query cursors и realtime outbox;
- `call-evidence-api` владеет только generated list/get и client-safe realtime
  schema без source cursors и provider locators;
- integration adapter преобразует только собственную operational projection в
  public Communications contract и сохраняет exact envelope bytes в своём
  outbox;
- Communications runtime декодирует exact contract, применяет core transition
  и атомарно сохраняет inbox/projection/realtime;
- Communications API владеет generated read contract; application composition
  не получает provider operational commands.

Integration не импортирует Communications implementation или persistence.
Communications не импортирует integration API, runtime, SDK или storage.

## Provider-neutral observation

Observation содержит только:

- stable 16-byte `call_evidence_id`;
- opaque SHA-256 cursors для source call, account и optional
  conversation/participant;
- typed provenance enum;
- incoming/outgoing direction;
- audio/meeting media kind;
- normalized lifecycle state and optional terminal disposition;
- monotonic source revision;
- observed/start/connected/end timestamps;
- optional bounded duration;
- optional bounded display label, которое не является identity.

Raw account, chat, participant and call locators используются только внутри
producer builder для deterministic cursor derivation и не сериализуются.

V1 lifecycle:

```text
observed
→ ringing | connecting
→ active
→ ended
```

Higher source revision может пропустить промежуточное состояние, но не может
переоткрыть terminal evidence. Exact duplicate возвращает previous result.
Same message ID или source revision с другими bytes fail closed.

`ended` требует terminal disposition. Non-terminal state запрещает terminal
disposition и ended timestamp. Timestamp ordering должно быть
`started <= connected <= ended`. Duration не является источником истины для
media и не разрешает provider command.

## Event contract

Exact contract:

```text
owner: communications
name: call_evidence_observed
major: 1
revision: 2
kind: observation
```

Producer получает только publish grant. Communications runtime получает только
required consume grant. Producer сохраняет exact envelope bytes в
integration-owned outbox; relay публикует без re-encode. Consumer проверяет
envelope, source/runtime fence, inbox ID/hash и ACK-ит только после durable
commit.

Subject, descriptor и grants выводятся из exact contract reference. Kernel,
Gateway и Event Hub не декодируют call payload.

## Public read and realtime

Generated client query предоставляет:

- list call evidence with opaque cursor pagination;
- get exact call evidence by canonical ID;
- typed provider/direction/state/media filters.

Shared replayable client SSE переносит только canonical ID, revision, state and
bounded display metadata. Он не переносит internal durable envelope, source
cursors, provider locators or transcript/media material.

Provider-specific Calls UI продолжает использовать integration operational
client. App-level Calls/Meetings composition читает только Communications
evidence contract.

## Privacy and downstream workflows

Call evidence не создаёт Persona, Task, Project, Review item или transcript.
Review получает отдельное domain event после
`review_communications_attention_v1`. Recording/transcription требует explicit
consent, Blob custody и `call_transcription_v1`.

Запрещены в payload, persistence, logs, errors, health и client realtime:

- phone numbers, usernames and raw provider IDs;
- media encryption keys and signaling bytes;
- PCM/audio/video;
- provider/native debug logs;
- credentials, cookies and sessions;
- transcript text or recording references without their own admitted workflow.

## Completion gate

`communications_call_evidence_v1` открывается только после:

1. exact ingress contract and pure core build units;
2. at least one integration-owned outbox producer without Communications
   implementation imports;
3. owner-local additive persistence with inbox/hash/revision fences;
4. managed Communications consumer with generation/grant/storage fencing;
5. generated query and shared replayable realtime surface;
6. duplicate, conflicting replay, out-of-order, terminal immutability, restart,
   NATS outage and privacy-negative tests;
7. live managed proof from integration outbox through NATS to Communications
   query/realtime after runtime restart.

Live conformance
`managed_call_evidence_survives_nats_outage_and_replays_through_gateway_sse`
подтверждает managed Telegram producer, exact outbox retention при NATS outage,
доставку в Communications query и shared SSE, privacy-negative payload check и
replay после рестарта Communications. ADR и static package presence сами по
себе gate не открывают.
