# ADR-0359: Bounded attachment archive inspection engine

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано. Отдельные
`makosh-attachment-archive-inspection-api`,
`makosh-attachment-archive-inspection-core` и
`makosh-attachment-archive-inspection-zip` units реализуют provider-neutral
client contract, pure bounded policy и ZIP metadata adapter без extraction.
`makosh-attachment-archive-inspection-persistence` реализует owner-local
request idempotency, exact message/hash inbox, порядок-независимый join,
bounded report/realtime storage и job lease fencing по worker, runtime
generation, grant epoch и monotonic fence. Отдельный target-owned
`makosh-attachment-archive-inspection-ingress` реализует typed event routes,
deterministic request/result envelopes, exact target constants и bounded
private proof validation для event-only custody delegation. Archive
persistence после exact three-way join атомарно создаёт deterministic
delegation intent, материализует exact command outbox отдельно от runtime
identity, принимает exact command-linked delegated/rejected result inbox и
создаёт parser job только после fresh proof. Attachment Security реализует
отдельные command-consumer/result-publisher capabilities, exact replay inbox,
owner-local source verification по completed safe verdict и current Blob
custody, fenced delegation jobs, ADR-0360 redelegation через managed control и
exact delegated/rejected result outbox. Archive managed runtime реализован
отдельной engine unit: он открывает fenced Event Hub и Storage/Vault
capabilities, принимает exact candidate/safety/delegation events, публикует
сохранённые custody commands без re-encode, сохраняет target Blob receipt до
использования, выполняет receipt-bound one-use read и вызывает bounded ZIP
metadata adapter. Отдельная assembly unit детерминированно материализует
descriptor, restart-applied settings schema, owner-local Storage bundle и
unsigned sorted release fragment; она не запускает runtime и не подписывает
manifest. Runtime descriptor теперь предоставляет exact owner-local Start/Get
и ClientRealtime surfaces; managed control dispatch валидирует module/owner/
contract identity, а replay publisher читает только owner-local status
transitions и отправляет bounded status через общий Kernel realtime protocol.
Live PostgreSQL/NATS/Blob/Gateway conformance доказывает exact event-only
custody handoff, NATS outage replay, terminal Gateway/SSE replay после restart
без второго Blob transfer или parser run и privacy-negative client output.
Production gate `attachment_archive_inspection_v1` открыт.

Зависит от:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0246](ADR-0246-communications-attachment-admission-and-safety.md);
- [ADR-0273](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md);
- [ADR-0274](ADR-0274-attachment-security-evidence-bound-blob-custody.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Legacy Communications содержал bounded ZIP metadata inspection прямо внутри
domain implementation. Он отклонял path traversal, password-protected и nested
archives, excessive entry count/depth и excessive declared uncompressed size.
Такое поведение требуется восстановить, но переносить parser, Blob read и
archive policy внутрь clean-room Communications нельзя: это создаст вторую
причину изменения domain и вернёт content-processing facade.

Attachment Security уже является отдельным engine и остаётся единственным
authority для `safe_for_delivery`. Archive inspection не является scanner
verdict, не изменяет safety lifecycle и не получает право объявлять вложение
безопасным.

## Решение

Вводится отдельный engine:

```text
owner_id  = attachment_archive_inspection
module_id = makosh-attachment-archive-inspection-runtime
kind      = engine
```

Engine запускается и обновляется независимо от Communications, Attachment
Security и provider integrations. Он меняется по причине изменения bounded
archive policy/parser support.

### On-demand flow

Пользовательский запрос адресуется через Core Gateway в exact typed archive
engine client contract. Он содержит только stable operation ID и canonical
attachment anchor. Blob reference, filename, MIME, provider/account identity,
path и bytes клиент не передаёт.

Engine durably объединяет три независимо поступающих факта:

1. client request для exact attachment anchor;
2. существующий provider-neutral
   `attachment_security_scan_candidate_observed.v1`;
3. canonical Communications transition в `safe_for_delivery`.

Факты могут прийти в любом порядке. Exact anchor join, exact candidate/safety
event identities и transition `blob_admitted -> safe_for_delivery` создают
только custody delegation intent. Runnable parser job появляется после exact
command-linked delegated result с fresh proof. Safety event не содержит
выдуманного correlation field. Exact replay идемпотентен; collision, stale
generation, revoked permit или mismatched candidate fail closed.

```text
client -> Gateway -> archive engine request_rpc
provider scan candidate event -----------\
Communications safe_for_delivery event ----> owner-local durable join
                                            -> delegation command outbox

archive delegation command
  -> Attachment Security inbox
  -> exact safe scan/current custody check
  -> Kernel current-custodian redelegation
  -> Attachment Security result outbox
  -> archive delegation-result inbox
                                            -> target-bound Blob custody
                                            -> one-use bounded Blob read
                                            -> ZIP metadata adapter
                                            -> owner-local result/outbox
```

Engine не вызывает Communications RPC, не читает Communications или
integration storage и не получает shared filesystem path. Cross-owner source
bytes передаются только через evidence-bound Blob custody. Kernel выдаёт и
fence-ит capability, но не читает bytes и не интерпретирует report.

### Event-only custody handoff

Первичный Mail -> Attachment Security proof нельзя переиспользовать для Archive
Inspection: он подписан для другого exact target. После scan proof также может
истечь раньше on-demand запроса. Поэтому joined archive request не создаёт
parser job напрямую. Он атомарно создаёт typed durable command:

```text
RequestArchiveInspectionCustodyDelegationV1
  request_id
  archive_run_id
  attachment_anchor_id
  candidate_message_id
  candidate_envelope_sha256
  safety_message_id
  safety_evidence_id
  logical_owner_id
```

Payload не содержит Blob reference, proof, bytes, provider/account identity,
filesystem path или выбираемый target triple. Target
`owner=attachment_archive_inspection`,
`module=makosh-attachment-archive-inspection-runtime`,
`capability=attachment_archive_inspection.blob.v1` является константой
контракта.

Attachment Security принимает command только через свой exact Event Hub
subscription, проверяет owner-local completed safe scan, exact candidate
message/hash, exact safety evidence и сохранённую current custody. Затем он
вызывает managed control operation ADR-0360 с deterministic delegation ID,
атомарно сохраняет terminal result и exact outbox bytes:

```text
ArchiveInspectionCustodyDelegatedV1
  request_id
  archive_run_id
  attachment_anchor_id
  candidate_message_id
  safety_message_id
  source_reference_id
  declared_size
  receipt_sha256
  custody_transfer_source_proof
  logical_owner_id

ArchiveInspectionCustodyDelegationRejectedV1
  request_id
  archive_run_id
  attachment_anchor_id
  bounded reject_code
  logical_owner_id
```

Archive persistence принимает result только по exact message ID/hash и
command causation, сверяет run/anchor/candidate/safety/owner и только после
этого создаёт parser job. Duplicate exact result является replay; collision,
неполная lineage, rejected delegation или mismatch fail closed. Proof и Blob
reference остаются только во внутренних event/persistence/runtime surfaces и
не попадают в client API, SSE, logs, health или telemetry.

Request/result schema принадлежит target owner в отдельном
`makosh-attachment-archive-inspection-ingress` package. Attachment Security
может импортировать только эту public contract unit, но не archive API,
persistence, runtime, parser или assembly. Это тот же target-owned ingress
pattern, которым integrations публикуют Communications observations; contract
dependency не превращает source engine в target engine и не создаёт
engine-to-engine RPC. Обратный event-consumer edge импортирует только публичный
`makosh-attachment-security-contract`, которому принадлежит scan-candidate
observation. Executable dependency policy разрешает ровно эти две contract
units через exact `engineEngineContractPackages` allowlist; произвольные
engine-to-engine contract или implementation dependencies остаются запрещены.

### Единицы сборки

```text
makosh-attachment-archive-inspection-api
  typed Start/Get/realtime client contract and bounded report schema

makosh-attachment-archive-inspection-ingress
  typed durable custody-delegation command/result contracts and envelopes

makosh-attachment-archive-inspection-core
  pure limits, path normalization, entry policy and terminal decisions

makosh-attachment-archive-inspection-zip
  reviewed ZIP central-directory metadata adapter; never extracts files

makosh-attachment-archive-inspection-persistence
  owner-local request/event inbox, join, fenced jobs, result and exact outbox

makosh-attachment-archive-inspection-runtime
  managed control, request/query, Event Hub, Blob custody/read and orchestration

makosh-attachment-archive-inspection-assembly
  descriptor/settings/Storage artifacts and unsigned release fragment only
```

API/core/parser не зависят от Communications implementation, Attachment
Security implementation, integrations, Kernel, Storage, Blob implementation
или runtime packages. Persistence является единственным SQL owner surface.
Runtime не материализует release artifacts. Assembly не запускается Kernel и
не подписывает manifest. Ingress не зависит от client API, core, persistence,
runtime или assembly и меняется только при изменении durable handoff contract.

### Bounded ZIP policy

Первый production parser поддерживает только ZIP. Он читает central-directory
metadata и не распаковывает entry bytes на диск или в память.

Hard limits принадлежат typed engine settings и не расширяются request/event
payload:

- source archive bytes;
- entry count;
- total declared uncompressed bytes;
- per-entry declared uncompressed bytes;
- normalized UTF-8 path bytes;
- path depth.

Fail-closed отклоняются:

- absolute, drive-prefixed, traversal и control-character paths;
- duplicate normalized paths;
- encrypted entries/archives;
- nested `.zip`, `.rar` и `.7z`;
- symlink и другие non-regular/non-directory Unix entry types;
- malformed ZIP metadata;
- любой limit overflow.

Отказ имеет bounded enum code. Raw parser error, original entry name, Blob
reference, source bytes, private socket/path и provider identity не попадают в
logs, health, telemetry или realtime error.

RAR/7z detection/parsing, recursive sandbox inspection, extraction и CDR не
входят в этот gate. Они требуют отдельного parser adapter и phase gate.

### Result boundary

Ready report содержит только bounded ZIP kind, counts/sizes и normalized entry
paths. Он является derived inspection evidence, а не canonical attachment
safety truth. Search index и UI projection rebuildable; source hash/generation
binding обязаны предотвращать reuse результата после смены Blob evidence.

Client получает Start receipt, а terminal state читает через Get и общий
replayable SSE status. `accepted` не означает completion.

## Phase gate `attachment_archive_inspection_v1`

Gate открывается атомарно только после:

1. exact seven-unit topology и executable dependency policy;
2. reviewed exact ZIP dependency profile;
3. request/status contract без source Blob/private/provider fields;
4. bounded path/type/encryption/nested/size/count/depth policy tests;
5. owner-local request + candidate + safety-state join в любом порядке;
6. exact replay/collision and lease/generation fencing;
7. event-only fresh current-custodian delegation, target-bound Blob custody and
   one-use read;
8. successful real ZIP metadata inspection without extraction;
9. traversal, duplicate, encrypted, nested, symlink/special-entry, malformed,
   entry-count/depth/per-entry/total-size negative matrix;
10. restart/NATS outage replay without second Blob transfer or parser run;
11. Gateway Start/Get and shared SSE terminal replay;
12. privacy-negative output and architecture/SRP/Cargo/full backend gates.

До выполнения всех пунктов inventory state остаётся `planned`.

## Отклонённые варианты

### Вернуть parser в Communications

Отклонено: domain получил бы content parser, Blob data-plane и archive policy.

### Добавить archive parsing в Attachment Security

Отклонено: malware verdict и archive inventory имеют разные причины изменения,
failure semantics и release cadence.

### Передать Blob reference из клиента

Отклонено: клиент не является custody/evidence authority и мог бы выбрать
произвольный source.

### Распаковать во временную директорию

Отклонено: metadata inspection не требует extraction; disk writes увеличивают
path/symlink/race surface без продуктовой необходимости.

### Считать malformed/unsupported archive clean

Отклонено: archive result не является safety verdict, а parser failure не
может повышать trust.
