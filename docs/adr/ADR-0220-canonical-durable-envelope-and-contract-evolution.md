# ADR-0220: Канонический durable envelope и эволюция контрактов

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Executable architecture policy, negative self-tests и
`makosh-events-protocol` canonical V1 schema с bounded outer-envelope decoder
реализованы. `makosh-events-jetstream` foundation проверяет outer envelope,
вычисляет exact subject и публикует original bytes with `Nats-Msg-Id`; Docker
test доказывает broker deduplication и exact-byte delivery. Его NATS credential
lease adapter использует только HPKE-protected Vault route и не имеет local
secret fallback; Vault audience lease can be revoked through that route.
Test-only owner delivery scaffolds now exercise a real PostgreSQL adapter for
the seven currently designable owners. Each uses its own fixed schema and its
own `durable_outbox_v1` / `durable_inbox_v1` tables; it stores original bytes,
marks publication only after a receipt and classifies a same-ID retry as either
duplicate or hash conflict. A disposable PostgreSQL+JetStream contour relays
one independently identified record byte-for-byte through a fenced test runtime
and changes the owner-local row only after the JetStream receipt. It also keeps
the row pending after an unavailable real NATS endpoint and publishes it after
reconnect. The scaffolds
contain no domain behaviour and are not production owner packages, migrations
or public contracts. Event Hub catalog-to-broker authority delivery, broker
credential rotation/revoke and full runtime conformance are now present, so
`nats_data_plane_v1` is open as a platform gate. A production outbox/inbox
adapter remains owner-owned work under `first_owner_v1`.

Their only direct SQL client is the explicitly named development dependency
`makosh-events-jetstream-testkit:dev:sqlx`. The executable Cargo policy rejects
the same client in every production package, every other test package and every
other dependency kind; this exception does not grant a PostgreSQL capability to
the Event protocol or a future owner runtime.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md).

Связано с:

- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md).

Этот ADR определяет только внутренний durable data plane Макошь. `OutboxRecordV1`
retains validated exact envelope bytes and `InboxRecordV1` classifies retries by
message ID plus SHA-256, so equal IDs with different bytes fail as a conflict.
These are owner-local persistence contracts, not a shared event database.
Синхронный
control/query/request RPC остаётся контрактом ADR-0201, а внешний realtime
desktop/Android — отдельным client envelope ADR-0205.

## Контекст

ADR-0201 уже фиксирует PostgreSQL outbox/inbox, NATS JetStream и at-least-once
delivery, но перечисление полей не задаёт однозначный wire contract. Без
отдельного решения каждый owner может по-разному интерпретировать:

- версию outer envelope и версию business payload;
- `message_kind`, обязательные поля и допустимые пустые комбинации;
- `message_id`, command identity, idempotency и три разных вида attempt;
- source identity, actor provenance и fencing epoch;
- schema identity и совместимость Protobuf;
- связь outbox bytes с опубликованным NATS message;
- durable `Ack` и broker-level JetStream ACK;
- terminal result, progress event и `unknown_outcome`;
- dead-letter ownership, privacy и повторную доставку;
- внутренний envelope и клиентский SSE frame.

Если разрешить owner-local варианты outer envelope, Mail, Telegram, Zulip,
Communications, AI, Tasks и Scheduler получат несовместимые data planes. Если
поместить owner payload types в общий package, любое изменение одного owner
начнёт пересобирать Kernel и соседние модули.

## Решение

### Один platform-owned outer envelope

Все durable commands, events, observations, results и acknowledgements
используют `DurableEnvelopeV1` из единственного package:

```text
backend/src/platform/events/protocol/  makosh-events-protocol
```

Cargo metadata package:

```toml
[package.metadata.makosh]
role = "platform"
owner = "events"
surface = "contract"
```

Package владеет только:

- outer envelope и kind-specific delivery metadata;
- ID, time, contract reference, trace и fencing wire primitives;
- dead-letter technical record;
- валидацией собственных structural invariants;
- envelope versioning и conformance vectors.

Он не содержит owner payload messages, NATS/PostgreSQL clients, outbox/inbox
storage, Event Hub implementation, HTTP, filesystem paths, Vault или runtime
bootstrap. `async-nats`, `nats`, SQL/SQLite clients и JSON serialization
являются запрещёнными dependencies этого package.

Owner contract package владеет конкретным Protobuf payload и его business
semantics. Event Hub catalog связывает outer reference с точным payload
descriptor, но Kernel, Event Hub и relay не декодируют business payload.

### Normative Protobuf shape

Первая реализация `.proto` обязана сохранить следующие field numbers и
семантику. Это normative target schema, а не утверждение о существующем
generated code:

```proto
syntax = "proto3";

package makosh.events.v1;

import "google/protobuf/timestamp.proto";

message DurableEnvelopeV1 {
  uint32 envelope_major = 1;          // MUST be 1
  uint32 envelope_revision = 2;       // initial value 1
  bytes message_id = 3;               // exactly 16 UUIDv7 bytes
  ContractRefV1 contract = 4;
  SourceRefV1 source = 5;
  google.protobuf.Timestamp recorded_at = 6;
  bytes partition_key = 7;            // opaque, empty or <= 64 bytes
  bytes causation_message_id = 8;     // empty or exactly 16 bytes
  bytes correlation_id = 9;           // exactly 16 UUID bytes
  ActorRefV1 actor = 10;
  TraceContextV1 trace = 11;          // optional message presence
  SourceFenceV1 source_fence = 12;    // optional; catalog can require it

  oneof semantics {
    CommandMetadataV1 command = 20;
    EventMetadataV1 event = 21;
    ObservationMetadataV1 observation = 22;
    ResultMetadataV1 result = 23;
    AckMetadataV1 ack = 24;
  }

  bytes payload = 30;                 // owner Protobuf binary only

  reserved 13 to 19, 25 to 29, 31 to 39;
}

message ContractRefV1 {
  string owner = 1;
  string name = 2;
  uint32 major = 3;
  uint32 revision = 4;
  bytes schema_sha256 = 5;            // exactly 32 bytes
}

message SourceRefV1 {
  string module_id = 1;
  bytes runtime_instance_id = 2;      // exactly 16 UUID bytes
  uint64 runtime_generation = 3;      // starts at 1
}

message ActorRefV1 {
  ActorKindV1 kind = 1;
  bytes actor_id = 2;                 // opaque, 1..64 bytes
}

message TraceContextV1 {
  bytes trace_id = 1;                 // exactly 16 bytes
  bytes parent_span_id = 2;           // exactly 8 bytes
  uint32 trace_flags = 3;             // only defined low bits accepted
}

message SourceFenceV1 {
  FenceKindV1 kind = 1;
  bytes scope_id = 2;                 // opaque, 1..64 bytes
  uint64 epoch = 3;                   // starts at 1
}
```

Точные enum zero-values всегда `*_UNSPECIFIED = 0` и не принимаются на
production boundary. Начальные значения:

```text
ActorKindV1:
  SYSTEM
  OWNER_DEVICE
  MODULE

FenceKindV1:
  GRANT_EPOCH
  RUNTIME_LEASE
```

`message_kind` не дублируется отдельным enum/string: единственный источник
истины — заполненный вариант `oneof semantics`. Ровно один вариант обязателен.
`content_type` отсутствует: V1 принимает только binary Protobuf payload.
Unrestricted `google.protobuf.Any`, `type_url`, JSON, TextProto, CBOR, arbitrary
maps, compression и content negotiation в durable envelope запрещены.

Удалённые field numbers и names резервируются и никогда не переиспользуются.
Field number, wire type и существующая semantics не меняются внутри V1.

### Общие поля и их смысл

`message_id` является globally unique UUIDv7, создаётся один раз до записи
outbox и не меняется при broker redelivery. В Protobuf он хранится как 16 bytes,
а в `Nats-Msg-Id` кодируется canonical lowercase hyphenated UUID string.

`recorded_at` — UTC-время фиксации canonical envelope в producer outbox. Оно не
заменяет время возникновения domain event или внешнего observation.
`google.protobuf.Timestamp` обязан быть normalized: nanos находятся в
диапазоне `0..999999999`, invalid/out-of-range значение отклоняется.

`correlation_id` обязателен. Для root message он равен `message_id`; для
продолжения процесса сохраняет ID исходной correlation. `causation_message_id`
пуст только у root message, иначе содержит exact `message_id` непосредственной
причины.

`partition_key` является opaque routing token. Это не email, account name,
chat title, phone, raw provider ID или другой отображаемый identifier. Contract
определяет его стабильность и ordering scope. Пустое значение означает, что
contract не обещает partition ordering.

`source` указывает logical module и конкретную process generation. Поле является
provenance, но не authority само по себе. Publish authorization задаётся
выданной NATS identity/ACL, effective GrantSet и Event Hub catalog. Consumer не
принимает business/security решение только на основании self-asserted
`module_id`, `actor` или `source_fence`.

`actor` сохраняет только opaque technical principal reference. Email, username,
device label и provider identity в общем header запрещены. Actor provenance не
заменяет authorization evidence владельца operation.

Trace context содержит только fixed-size trace/span IDs и flags. `tracestate`,
baggage, arbitrary key/value maps и user-controlled identifiers запрещены.

`source_fence` относится только к authority producer. Catalog определяет, когда
он обязателен, его kind и expected scope. Current grant/runtime epoch из
control-plane state является authority; stale или несовпадающий token
отклоняется до payload decode и owner mutation. Job execution lease, provider
account lease и другие owner-specific fences остаются в typed owner payload и
не маскируются одним общим `lease_epoch`.

### Kind-specific metadata

Normative kind messages:

```proto
message CommandMetadataV1 {
  bytes command_id = 1;               // exactly 16 UUID bytes
  string target_capability = 2;
  bytes idempotency_key = 3;           // 1..64 opaque bytes
  google.protobuf.Timestamp deadline = 4;
  uint32 logical_attempt = 5;          // starts at 1
}

message EventMetadataV1 {
  google.protobuf.Timestamp occurred_at = 1;
}

message ObservationMetadataV1 {
  bytes observation_id = 1;            // exactly 16 UUID bytes
  google.protobuf.Timestamp observed_at = 2;
  google.protobuf.Timestamp occurred_at = 3; // optional presence
  bytes source_cursor_sha256 = 4;      // empty or exactly 32 bytes
  optional uint64 source_sequence = 5;
}

message ResultMetadataV1 {
  bytes command_id = 1;
  bytes command_message_id = 2;
  ResultOutcomeV1 outcome = 3;
  google.protobuf.Timestamp completed_at = 4;
  uint32 execution_attempt = 5;        // starts at 1
}

message AckMetadataV1 {
  bytes acknowledged_message_id = 1;
  AckStageV1 stage = 2;
  AckDispositionV1 disposition = 3;
  google.protobuf.Timestamp acknowledged_at = 4;
}
```

Начальные terminal result outcomes:

```text
SUCCEEDED
FAILED
CANCELLED
EXPIRED
REJECTED
UNKNOWN_OUTCOME
```

`result` всегда terminal. Progress, percentage и non-terminal state являются
owner event, а не result. `unknown_outcome` terminal, не ретраится автоматически
и не переносится в DLQ только из-за своего outcome.

`command_id` связывает один logical command. Явный permitted retry создаёт новый
`message_id`, сохраняет `command_id` и `idempotency_key`, увеличивает
`logical_attempt` и ставит causation на предыдущую попытку. JetStream
redelivery не создаёт новую попытку: она повторяет те же bytes и IDs.

`logical_attempt`, JetStream delivery count и owner `execution_attempt` — три
разных счётчика. Relay и broker никогда не переписывают их друг в друга.

Начальные Ack stages:

```text
DURABLE_ACCEPTANCE
CANONICAL_PERSISTENCE
TERMINAL_HANDLING
```

Начальные dispositions:

```text
APPLIED
DUPLICATE
REJECTED
```

Durable `AckMetadataV1` является отдельным Макошь message, подтверждающим
зафиксированную стадию обработки исходного message. JetStream ACK — broker
protocol после local commit; он не является `AckMetadataV1` и не создаёт
durable Ack-envelope автоматически.

Observation переносит stable `observation_id`; replay одного внешнего
наблюдения сохраняет его. Raw provider cursor, offset с private identifier или
session material не попадает в outer envelope. При необходимости correlation
используется bounded SHA-256 cursor digest, а exact cursor остаётся в
integration-owned protected state. `source_sequence` не создаёт глобального
ordering promise.

### Payload binding и schema identity

Payload — непрозрачные binary Protobuf bytes конкретного owner contract. Identity
контракта:

```text
(owner, name, major)
```

`owner` и `name` являются lowercase ASCII tokens `[a-z][a-z0-9-]{0,63}` без
`.v1` suffix. `major >= 1`; `revision >= 1`.

Event Hub catalog для identity содержит:

- exact message kind;
- fully qualified root Protobuf message name;
- разрешённые envelope revisions;
- единственный `schema_sha256` для каждой contract revision;
- publishers/subscribers и required/optional semantics;
- partition policy;
- payload и total-envelope limits;
- privacy classification и разрешённые opaque reference types;
- retry/dead-letter policy.

Publisher/consumer declarations поступают только из exact validated
`CapabilityDescriptorV1` внутри `ModuleDescriptorV1` ADR-0221. Descriptor
ссылается на эту contract identity и exact schema hash, но не встраивает owner
payload schema и не создаёт второй catalog. Settings schema ADR-0222 не является
durable payload contract и не даёт publish/subscribe rights.

`schema_sha256` — SHA-256 canonical binary `FileDescriptorSet` root payload и
его transitive imports. Это schema descriptor artifact, а не
`ModuleDescriptorV1`. Макошь canonicalizer:

1. строит descriptor closure pinned Protobuf toolchain;
2. удаляет `source_code_info`;
3. сортирует file descriptors по exact file name;
4. сохраняет declaration и dependency order внутри каждого file;
5. сериализует canonical descriptor artifact deterministic encoder;
6. вычисляет SHA-256 exact artifact bytes.

Canonical schema descriptor artifact является versioned build artifact contract
package. Consumer использует catalog artifact/compiled schema descriptor и не
доверяет произвольной schema, пришедшей вместе с message. Hash идентифицирует
schema, но не является подписью payload или publisher authentication.

Изменение schema descriptor всегда создаёт новую contract revision/hash. Revision
разрешает только additive wire-compatible изменение с прежней semantics.
Несовместимое wire или semantic изменение создаёт новый major. Один revision не
может иметь два schema hashes.

Producer cutover на новый revision/hash разрешён только после Event Hub
reconciliation: все actual subscribers либо объявили exact support, либо
explicitly disabled/drained. Несовместимый required subscriber блокирует
readiness. Optional subscriber может быть отключён со scoped degraded state,
но не получает заведомо несовместимые messages.

### Envelope evolution

`envelope_major` и owner contract version изменяются независимо.

- `envelope_major = 1` определяет structural wire family;
- `envelope_revision` увеличивается только для additive optional fields с
  безопасным default и неизменной semantics существующих fields;
- новый kind, перенос field в существующий `oneof`, изменение wire type или
  meaning требует нового envelope major;
- неизвестный major всегда fail closed;
- неизвестный revision принимается только если catalog и local adapter явно
  объявляют совместимость; guessing запрещён;
- неизвестный enum, contract major/revision или schema hash fail closed до
  owner mutation;
- удалённые numbers/names резервируются навсегда.

Protobuf binary сохраняет unknown fields при поддерживающем runtime, но Макошь
не полагается на parse/re-encode в relay. JSON/TextProto conversion и
field-by-field copy для durable transport запрещены. Compatibility проверяется
golden wire vectors и automated schema breaking checks. Начальным кандидатом
для Rust code generation является [`prost`](https://github.com/tokio-rs/prost),
а для mechanical contract checks —
[`buf breaking`](https://buf.build/docs/breaking/) с conservative `FILE`
category и дополнительными Макошь semantic tests.

### Canonical bytes: outbox → JetStream

Producer adapter выполняет один pipeline:

```text
construct typed owner payload
  → validate payload contract
  → construct and validate DurableEnvelopeV1
  → serialize once
  → compute envelope_sha256
  → persist exact bytes + technical indexes in owner outbox transaction
```

Outbox row хранит exact envelope bytes, SHA-256, message ID, subject/catalog key
и publish state. Redundant index fields обязаны совпадать с parsed header.

Relay может parse/validate header и сверить indexes, но публикует исходный byte
buffer без decode/re-encode, JSON conversion, field copying, compression или
payload wrapping. Publish acknowledgement меняет только outbox delivery state,
не envelope bytes.

Единственный обязательный business-independent NATS header:

```text
Nats-Msg-Id: <canonical message UUID>
```

Допустимы server-defined optimistic publish headers, если их требует adapter,
но contract, causation, correlation, actor, trace, partition и payload metadata
не дублируются в headers. `Nats-Msg-Id` deduplication window является bounded
broker optimization и не заменяет durable inbox.

Consumer pipeline:

```text
receive exact bytes
  → enforce total/header limits
  → decode and validate outer envelope
  → validate subject/kind/catalog/version/hash/source fence
  → decode owner payload with exact descriptor
  → BEGIN owner transaction
       compare inbox message_id + envelope_sha256
       apply owner mutation or return stored outcome
       append emitted outbox messages
       mark inbox processed
    COMMIT
  → JetStream ACK
```

Повтор того же `message_id` и того же SHA-256 является normal duplicate. Тот же
`message_id` с другими bytes — `message_id_collision`: business mutation
запрещена, обе hashes сохраняются как sanitized technical evidence, message
переводится в quarantine и fail closed. Последние bytes не перезаписывают
первые.

Envelope не подписывается отдельным message key в V1. Authentication и
authorization обеспечивают module session, NATS credentials/ACL, Event Hub
catalog и PostgreSQL role boundary. Header provenance и schema hash не должны
использоваться как замена этим механизмам. Per-message signatures требуют
отдельного threat-model ADR.

### Subject grammar

Durable subjects:

```text
makosh.command.v1.<owner>.<contract>.v<contract-major>
makosh.event.v1.<owner>.<contract>.v<contract-major>
makosh.observation.v1.<owner>.<contract>.v<contract-major>
makosh.result.v1.<owner>.<contract>.v<contract-major>
makosh.ack.v1.<owner>.<contract>.v<contract-major>
```

Первый `v1` — версия subject/envelope transport grammar. Последний token —
owner contract major. Contract revision и schema hash находятся только в
envelope/catalog. Subject kind обязан совпадать с заполненным `oneof`.

Account, entity, actor, partition, provider, correlation и user-controlled
identifiers в subject запрещены. Точный subject создаётся из Event Hub catalog,
а не ad hoc в owner implementation.

### Dead-letter и quarantine

Dead letter не является шестым business message kind. `makosh-events-protocol`
определяет отдельный technical `DeadLetterRecordV1`:

```text
dead_letter_id
original_message_id, если outer header был валиден
original_envelope_sha256
original contract/kind, если они были валидны
consumer identity
sanitized reason enum
delivery count
first_failed_at / terminal_failed_at
opaque quarantine reference
```

После terminal delivery/processing failure consuming owner adapter:

1. сохраняет exact original envelope bytes в bounded owner-scoped quarantine;
2. сохраняет sanitized technical failure record;
3. публикует только `DeadLetterRecordV1` в subject
   `makosh.dead.v1.<owner>.<contract>.v<contract-major>`;
4. завершает broker delivery согласно contract policy.

Dead stream не содержит original payload bytes. Event Hub видит counters и
sanitized metadata, но не читает quarantine или payload. Quarantine reference
не является filesystem path и доступен только authorized operator/owner path.

Automatic replay запрещён. Explicit redelivery неизменённого message использует
те же bytes и `message_id`. Исправление/migration payload создаёт новый envelope
с новым `message_id` и causation на original message. `unknown_outcome` не
replay-ится и не превращается в DLQ без отдельной operator decision.

Malformed/oversized outer envelope, unknown major, subject-kind mismatch,
unknown schema hash и stale fence никогда не доходят до owner mutation. Adapter
сохраняет только bounded technical evidence и применяет contract quarantine
policy; он не пытается угадать формат или переключиться на JSON/старую schema.

### Внутренний envelope не является client SSE contract

`DurableEnvelopeV1` запрещено сериализовать клиенту напрямую. Core Gateway
использует отдельный `ClientRealtimeFrameV1` из `makosh-gateway-protocol`:

```text
ClientRealtimeFrameV1
  oneof frame
    ClientEventV1
    ReplayGapV1
    StreamStateV1
```

Owner публикует только явно catalog-declared `client_safe` event contract либо
создаёт отдельный client event через свой adapter. Gateway добавляет
device-local replay cursor и применяет session capabilities, не превращая
произвольный internal event в public payload.

Client frame не содержит:

- source module/runtime instance и generation;
- partition key, idempotency key или source fence;
- raw actor/source provenance;
- provider cursor/digest/sequence;
- NATS subject, delivery attempts или DLQ/quarantine metadata;
- internal schema descriptors;
- private body/media/session/secret data.

Internal `message_id` может стать client `event_id` только для явно client-safe
event; cursor, replay gap и client filtering остаются независимой semantics
ADR-0205.

### Bounded limits V1

Platform hard maximums:

- serialized `DurableEnvelopeV1`: 262144 bytes;
- outer header без `payload`: 16384 bytes;
- owner/contract token: 64 ASCII bytes;
- module/capability ID: 128 ASCII bytes;
- actor ID, fence scope, idempotency key и partition key: 64 bytes;
- cursor digest и schema hash: exactly 32 bytes;
- UUID/message/runtime IDs: exactly 16 bytes;
- trace ID: 16 bytes; span ID: 8 bytes;
- arbitrary repeated metadata/maps/baggage: запрещены.

Конкретный contract обязан установить меньший или равный payload/total limit.
Message bodies, documents, prompts, media, credentials, provider sessions и
другой private bulk content передаются только через разрешённый opaque
`BlobRef`/`EvidenceRef` и owner authorization. Превышение limit fail closed и не
увеличивает NATS maximum динамически.

## Отклонённые варианты

### JSON или generic metadata map

Отклонено: stringly typed fields, неоднозначные default values, schema drift и
риск утечки произвольной metadata.

### Отдельный outer envelope каждого owner

Отклонено: Kernel/adapters должны знать owner types, а owner implementations
получают общий compile fan-out.

### Unrestricted `google.protobuf.Any`

Отклонено: произвольный `type_url` не доказывает catalog ownership, exact schema
revision, allowed kind или compatibility.

### Повторная сериализация relay

Отклонено: меняет authoritative bytes, может терять unknown fields и делает
message collision/deduplication неоднозначными.

### Dead letter как шестой kind

Отклонено: transport failure не является business fact исходного owner и не
должен попадать в обычную event semantics.

### Прямо отдавать internal envelope в SSE

Отклонено: раскрывает runtime topology, fencing/delivery metadata и связывает
mobile client compatibility с внутренним data plane.

### Exactly-once claim

Отклонено: broker deduplication не устраняет crash boundaries между PostgreSQL,
publish acknowledgement, consumer transaction и broker ACK. Макошь остаётся
at-least-once с durable inbox/idempotency.

## Проверка решения

До изменения `Состояние реализации` необходимы:

- exact package metadata и отсутствие NATS/SQL/JSON dependencies;
- generated `.proto` соответствует frozen field numbers и reserved ranges;
- golden Protobuf byte vectors для каждого kind и `DeadLetterRecordV1`;
- unknown/missing kind, zero enum, invalid ID/time/token/limit fail closed;
- unknown envelope major/revision, contract major/revision и schema hash;
- schema hash canonicalization reproducible из pinned schema descriptor
  artifact;
- publisher/subscriber reference в `ModuleDescriptorV1` совпадает с exact
  catalog contract revision/schema hash;
- mechanical Protobuf breaking check и Макошь semantic compatibility tests;
- outbox сохраняет один canonical byte buffer;
- relay публикует byte-for-byte тот же buffer;
- `Nats-Msg-Id` точно соответствует binary `message_id`;
- broker redelivery сохраняет message ID, bytes и logical attempt;
- explicit logical retry создаёт новый message ID и повышает attempt;
- duplicate same ID/same hash даёт stored result/no-op;
- same ID/different hash даёт collision quarantine без mutation;
- stale source fence rejected before payload decode/mutation;
- durable Ack и JetStream ACK не взаимозаменяемы;
- result имеет только terminal outcomes; progress идёт event contract;
- `unknown_outcome` не ретраится и не replay-ится автоматически;
- source cursor digest не раскрывает raw cursor/private identifier;
- subject kind/owner/contract/major совпадают с envelope/catalog;
- required consumer blocks incompatible producer cutover;
- DLQ сохраняет exact original bytes только в owner quarantine и публикует
  sanitized record;
- replay требует explicit operation; transformed payload получает новый ID;
- Event Hub не находится на normal payload hot path;
- Gateway не выдаёт `DurableEnvelopeV1` и stripping проверяется negative tests;
- secrets/private content отсутствуют в subjects, headers, logs, telemetry,
  diagnostics, SSE и dead-letter record;
- crash до/после outbox commit, publish ack, inbox commit и broker ACK;
- NATS outage/reconnect не изменяет canonical bytes.

## Последствия

Положительные:

- все owner modules используют один проверяемый durable transport contract;
- owner payload остаётся independently versioned и compile-isolated;
- Event Hub управляет catalog/topology без business payload parsing;
- outbox, NATS, inbox и DLQ имеют однозначную byte identity;
- command/result/ack, broker delivery и retries больше не смешиваются;
- desktop/Android realtime не зависит от internal runtime metadata.

Цена:

- high-fanout `makosh-events-protocol` требует особенно строгого contract review;
- schema descriptors, hashes и compatibility matrix становятся обязательными
  build artifacts;
- producer cutover требует reconciliation consumers;
- каждый consumer обязан иметь durable inbox/hash validation и quarantine;
- owner contracts должны явно проектировать payload/reference и retry limits.

## Ссылки

- [Protocol Buffers: Updating A Message Type](https://protobuf.dev/programming-guides/proto3/#updating)
- [Buf breaking change detection](https://buf.build/docs/breaking/)
- [prost — Protocol Buffers implementation for Rust](https://github.com/tokio-rs/prost)
- [NATS JetStream streams and `Nats-Msg-Id`](https://docs.nats.io/nats-concepts/jetstream/streams)
- [NATS JetStream headers](https://docs.nats.io/nats-concepts/jetstream/headers)
