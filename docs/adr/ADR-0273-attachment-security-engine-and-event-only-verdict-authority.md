# ADR-0273: Attachment Security engine and event-only verdict authority

Статус: Принято
Дата: 2026-07-24
Состояние реализации: production gate `attachment_security_engine_v1` открыт
атомарно. Attachment-specific
Communications schemas выделены без facade/duplicate source в
`makosh-communications-attachment-contract`; executable dependency policy и
отдельный managed Engine launch path реализованы. Engine-owned typed candidate
contract, pure join/verdict core и bounded loopback ClamAV `INSTREAM` adapter
реализованы отдельными Cargo units с unit и architecture coverage. Owner-local
PostgreSQL persistence владеет exact inbox ID/hash, anchor-serialized
order-independent join, bounded attempt/lease-fenced scan jobs, quarantine
evidence и exact verdict outbox. Отдельный managed runtime получает только
Kernel-issued Storage/Blob/Event capabilities, а отдельная assembly unit
материализует canonical descriptor/settings/Storage bytes и unsigned release
fragment для generic distribution compiler. Первый реальный producer —
managed Mail runtime — после owner-local Blob commit атомарно сохраняет typed
candidate в отдельном Mail outbox и публикует exact bytes по отдельной
owner-approved capability; live contour доказывает replay и privacy boundary.
Отдельный live managed contour также доказывает signed Engine
executable/descriptor/settings/Storage binding, exact five-capability GrantSet,
owner-local PostgreSQL, Event Hub credentials и readiness при loopback ClamAV
endpoint из typed settings snapshot. Exact live payload contour теперь
доказывает revision-2 candidate, target-bound cross-owner Blob transfer,
engine-owned receipt, one-use read, loopback ClamAV clean response, typed
verdict event, Communications CAS до `safe_for_delivery` и exact replay без
второго transfer/verdict/scan. Тот же contour отдельно доказывает, что прямой
read integration-owned source Blob отклоняется data-plane access fence. Live
scanner matrix теперь также доказывает threat verdict до `quarantined` и
fail-closed malformed response, disconnect/I/O и timeout: Communications
остаётся в `blob_admitted`, verdict/outbox не создаётся, а exact replay не
дублирует первую custody/scan attempt. Дополнительный live contour удерживает
clean scan до NATS outage, сохраняет exact verdict в owner outbox, останавливает
Engine, перезапускает Communications consumer через fenced Storage successor и
запускает Engine generation 2 с новым runtime/Storage fence; новый relay
публикует те же bytes без повторного scan.
Отдельный stale-CAS verdict не изменяет terminal Communications state.
Тот же authenticated contour теперь доказывает fail-closed custody authority:
stale source runtime generation, revoked source registration, revoked target
registration, остановленный Vault и остановленный Blob сохраняют retryable
engine job без target receipt/outbox/verdict, не запускают scanner и оставляют
Communications в `blob_admitted`; exact candidate replay также не создаёт
terminal fact. Executable policy теперь допускает один exact Engine inventory:
contract, core, ClamAV adapter, persistence, runtime и assembly; owner inventory
содержит `attachment_security` только в `engines`, а пять capabilities совпадают
с descriptor. Production persistence не имеет Cargo feature switches;
admin-only diagnostics перенесены в test-only recovery harness.
`safe_for_delivery` допускается только через этот signed managed Engine path;
невалидный или недоступный scanner/Vault/Blob оставляет его fail closed.
ADR-0274 фиксирует обязательный custody path, а ADR-0275 — стабильную target
identity без зависимости от динамического Kernel registration ID.

Зависит от:

- [ADR-0201: Core module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0212: crate topology and compile isolation](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: code ownership and module autonomy](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: module admission](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: module descriptor and capabilities](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0224: Storage Control](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0230: Blob opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data sessions](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0246: attachment admission and safety](ADR-0246-communications-attachment-admission-and-safety.md);
- [ADR-0260: attachment lifecycle event authority](ADR-0260-communications-attachment-lifecycle-event-authority.md);
- [ADR-0274: Attachment Security Blob custody](ADR-0274-attachment-security-evidence-bound-blob-custody.md).

## Контекст

Communications уже принимает отдельный typed contract
`communication_attachment_safety_verdict_observed.v1`, но реального producer
нет. Нельзя закрыть этот разрыв scanner-вызовом из Communications: domain
получит Blob data-plane и scanner implementation, нарушит SRP и превратится в
integration facade.

Интеграция получает от Kernel/Core platform admission, settings, Storage,
Blob и Event Hub leases. Это control plane, а не business-вызов ядра.
Attachment bytes и verdict не должны проходить через Kernel. Межвладельческий
business flow остаётся только event-only:

```text
Integration-owned admitted Blob
  -> typed scan candidate observation
  -> Attachment Security inbox

Communications canonical blob_admitted event
  -> Attachment Security inbox

joined exact candidate + canonical state
  -> evidence-bound transfer to Attachment Security Blob custody
  -> one-use target-owned Blob read session
  -> local ClamAV INSTREAM adapter
  -> Attachment Security outbox
  -> communication_attachment_safety_verdict_observed.v1
  -> Communications inbox/CAS/outbox
```

Один integration event недостаточен: verdict может обогнать canonical
`blob_admitted` transition в другом consumer. Engine обязан durably join-ить
provider-neutral scan candidate с canonical Communications lifecycle event, а
не полагаться на timing, subject ordering или direct query.

Текущий Kernel умеет запускать managed domain и integration runtimes, но не
имеет отдельного Engine runtime configuration/owner operation. Использовать
domain launch contract означало бы назвать engine доменом; использовать
integration contract означало бы передать ему provider configuration,
integration state root и host-bridge semantics. Оба варианта нарушают единицы
сборки и authority.

## Решение

### Owner и production identity

Вводится отдельный engine owner:

```text
owner_id  = attachment_security
module_id = makosh-attachment-security-runtime
kind      = engine
```

Это не Communications domain, не provider integration, не platform service и
не workflow. Engine меняется по причине изменения attachment scanning policy
и её исполнения.

### Единицы сборки

```text
makosh-communications-attachment-contract
  domain-owned public attachment event/observation schemas

makosh-attachment-security-contract
  engine-owned provider-neutral scan-candidate schema

makosh-attachment-security-core
  pure validation, durable join decision and closed verdict policy

makosh-attachment-security-clamav
  bounded clamd INSTREAM protocol adapter

makosh-attachment-security-persistence
  owner-local inbox, candidate/state join, retry job and exact outbox

makosh-attachment-security-runtime
  managed control, Event Hub, Blob read, scanner and relay orchestration

makosh-attachment-security-assembly
  descriptor/settings/Storage release composition only
```

`makosh-communications-attachment-contract` становится единственным
attachment-specific public contract package Communications. Existing
attachment anchor, Blob-admission, safety-verdict и lifecycle schemas
переносятся туда без facade, duplicate schema source или re-encoding adapter.
General provider evidence остаётся в `makosh-communications-ingress`.

Engine может зависеть только от exact allowlisted
`makosh-communications-attachment-contract`, собственного contract/core/
adapter/persistence и platform contracts. Он не импортирует
`makosh-communications-api`, domain, runtime или persistence. Communications
не импортирует ни один `makosh-attachment-security-*` package.

Integration может зависеть от `makosh-attachment-security-contract`, но не от
engine core/adapter/persistence/runtime/assembly. Kernel, Gateway и platform
packages не зависят от owner-specific engine packages.

Runtime не материализует release artifacts. Assembly не запускается Kernel,
не подписывает manifest, не выдаёт grants и не читает runtime state.

### Typed input contracts

Integration, завершившая Blob write и owner-local commit, публикует
`attachment_security_scan_candidate_observed.v1`. Payload закрыт и содержит
только:

- canonical `attachment_anchor_id`;
- opaque Blob `reference_id`;
- exact `declared_size`;
- non-zero Blob receipt SHA-256;
- observed time.

Provider locator, account, filename, MIME body, filesystem path, socket path,
credential, scanner policy, arbitrary label/map и content bytes запрещены.
Envelope сохраняет исходные causation/correlation IDs. Candidate не является
verdict и не меняет Communications.

Engine отдельно подписывается на canonical
`communication_attachment_safety_state_changed.v1` и принимает для scan join
только точный переход `blob_pending -> blob_admitted` с тем же
`attachment_anchor_id` и correlation ID. Candidate и canonical transition
могут прийти в любом порядке. Каждый сохраняется в engine-owned inbox; scan job
становится runnable только после conflict-free exact join.

Повтор exact bytes идемпотентен. Тот же message ID с другими bytes, разные Blob
reference/receipt для одного anchor/correlation, malformed contract, stale
source generation или revoked permit fail closed и не запускают scanner.

### Blob и scanner boundary

По ADR-0274 engine сначала запрашивает у Kernel evidence-bound custody transfer
по capability `attachment_security.blob.v1`, exact source proof, candidate
message ID/envelope hash, reference, declared size, receipt SHA-256, current
runtime generation и grant epoch. Только полученный target-owned reference
может использоваться в one-use `BlobDataOperationReadRangeV1` session. Kernel
выдаёт/fence-ит sessions, но не читает bytes и не знает scanner verdict.

Blob bytes идут напрямую из private Blob data socket в bounded engine buffer и
затем в `makosh-attachment-security-clamav`. Maximum attachment size является
hard descriptor/settings policy и никогда не расширяется payload-ом.

Первый production scanner — explicit local clamd `INSTREAM` adapter:

- chunk length имеет bounded `u32` framing;
- total bytes обязаны совпасть с declared size и receipt SHA-256;
- command, response и timeout ограничены;
- только exact `stream: OK` создаёт clean decision;
- exact non-empty `... FOUND` создаёт threat decision;
- timeout, I/O error, oversized/malformed/`ERROR` response не считаются clean,
  сохраняют retryable job и не публикуют permissive verdict;
- scanner signature и raw response не покидают engine-owned private evidence и
  не попадают в logs/errors/health.

Clamd endpoint и timeout принадлежат typed engine settings schema. Environment
fallback и generic endpoint map запрещены. Kernel применяет revision settings,
но не интерпретирует endpoint semantics.

### Verdict authority

`makosh-attachment-security-core` имеет закрытое отображение:

```text
clamd OK       -> safe_for_delivery
clamd FOUND    -> quarantined
scanner error  -> retry, no verdict event
invalid join   -> quarantine evidence, no scan
```

Runtime публикует существующий
`communication_attachment_safety_verdict_observed.v1` только из
engine-owned durable outbox. Payload содержит canonical anchor, expected
`blob_admitted`, closed verdict, evidence ID и observed time. Он не содержит
bytes, Blob reference, scanner identity/signature, provider data или
объяснение.

Communications остаётся единственным владельцем lifecycle CAS. Engine не
вызывает Communications RPC, не читает её storage и не утверждает canonical
state вне typed verdict.

### Managed Engine control plane

Runtime protocol получает отдельный
`ManagedEngineRuntimeConfigurationV1`:

- logical owner/registration/runtime identity;
- runtime generation и grant epoch;
- owner-local Storage binding;
- Event Hub topology revision;
- exact applied settings revision.

В нём нет provider credentials, integration artifacts/state root, host bridge
или business payload. Gateway owner control получает отдельную
`StartReservedEngineRuntime` operation. Kernel проверяет registration kind
`engine`, signed exact executable/descriptor/settings/Storage bindings,
effective grants и reservation fences, stage-ит typed settings snapshot и
запускает runtime через отдельный managed Engine launch path.

Kernel не использует domain/integration launch как fallback, не декодирует
engine events, не подключается к clamd и не становится Blob/scanner proxy.

### Engine capabilities

Exact descriptor содержит отдельные capability units:

```text
attachment_security.candidate.observe.v1
attachment_security.communications-state.observe.v1
attachment_security.verdict.publish.v1
attachment_security.blob.v1
attachment_security.storage.v1
```

Event Hub permits bind exact contract/schema/direction/subject. Blob grant не
даёт generic read-all: каждый data session дополнительно binding-ится к
reference, size, receipt, current runtime generation и grant epoch.

### Release assembly

Engine assembly материализует только:

```text
attachment-security.runtime.descriptor.pb
attachment-security.runtime.settings.pb
attachment-security.storage.bundle.pb
attachment-security.release-artifacts.json
```

Generic distribution compiler повторно проверяет exact bytes и подписывает
полный manifest. Assembly не имеет signing authority и не включает clamd
binary: clamd является явно настроенной local external dependency. Production
admission требует доказанной доступности и fail-closed scanner behavior;
автоматический download/install/fallback запрещён.

## Phase gate `attachment_security_engine_v1`

Gate открывается атомарно только после:

1. exact contract extraction без duplicate/facade schemas;
2. executable dependency policy для exact engine/domain contract allowlist;
3. отдельного managed Engine runtime protocol, owner operation и launch path;
4. exact five-package engine implementation плюс две contract units;
5. canonical settings schema, immutable Storage bundle и release assembly;
6. signed executable/descriptor/settings/Storage admission и owner grants;
7. candidate/state join в любом порядке, exact replay и collision quarantine;
8. evidence-bound custody transfer и one-use target-owned Blob read с
   size/hash/session/generation/grant fences;
9. live loopback clamd clean и threat responses;
10. scanner timeout/I/O/malformed response без clean verdict;
11. exact engine outbox -> NATS -> Communications CAS flow;
12. duplicate, stale CAS, relay restart, NATS outage replay, stale/revoked
    source, revoked target и Blob/Vault outage evidence;
13. negative-output scanner без bytes, reference, signature, provider data,
    settings endpoint или private socket path;
14. architecture/SRP/Cargo/full backend gates.

Gate открыт отдельной exact production inventory revision
`attachment_security_engine_v1`. Она сохраняет Communications единственным
доменом, добавляет только engine owner и не допускает integration или workflow
packages.

## Отклонённые варианты

### Scanner внутри Communications

Отклонено: domain получил бы Blob/scanner implementation и вторую причину
изменения.

### Engine запускается как domain или integration

Отклонено: ложная module kind даёт неверный configuration/authority contract.

### Integration публикует safety verdict

Отклонено: владелец provider download не является scanner authority.

### Kernel/Core переносит bytes или verdict

Отклонено: control plane стал бы business/data-plane broker.

### Только scan candidate без canonical join

Отклонено: race между NATS consumers может навсегда потерять ранний verdict.

### No-op/heuristic scanner может выдать clean

Отклонено: отсутствие реального scanner verdict не является
`safe_for_delivery`.

## Последствия

Communications завершает attachment safety lifecycle через typed durable
events, оставаясь чистым domain owner. Интеграции владеют provider download,
Attachment Security — scanner policy/execution, Blob — bytes, Kernel/Core —
только admission/leases/routing/fencing. Каждая ответственность имеет
отдельную единицу сборки и отдельную authority.
