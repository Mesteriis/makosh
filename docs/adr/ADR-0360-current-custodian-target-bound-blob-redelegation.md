# ADR-0360: Current-custodian target-bound Blob redelegation

Статус: Принято

Дата: 2026-07-31

Состояние реализации: runtime protocol, Kernel control plane, Blob client и
Blob data-plane structural validation реализованы. Unit evidence доказывает
exact wire operation, signed predecessor lineage, target binding, distinction
между source `Write` и `CustodyTransfer`, deterministic reference при retry и
fail-closed partial lineage. Business request/result events и managed live
conformance первого consumer остаются частью открытого Archive Inspection
gate; существующий proof всё ещё нельзя переиспользовать напрямую.

Зависит от:

- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0257](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0274](ADR-0274-attachment-security-evidence-bound-blob-custody.md);
- [ADR-0275](ADR-0275-target-bound-cross-owner-blob-custody-delegation.md);
- [ADR-0327](ADR-0327-durable-target-bound-blob-delegation-across-source-successors.md);
- [ADR-0359](ADR-0359-bounded-attachment-archive-inspection-engine.md).

## Контекст

Первичный producer выпускает `BlobCustodySourceProofV1` для одного exact
target `owner_id + module_id + capability_id`. Mail attachment proof поэтому
разрешает только `mail -> attachment_security`. После успешного scan текущим
custodian становится Attachment Security, но исходный proof всё ещё адресован
ему и не может авторизовать
`attachment_security -> attachment_archive_inspection`.

Повторное использование исходного proof другим target нарушило бы target
binding ADR-0275. Возврат source bytes в Mail, повторная запись Blob, прямой
request RPC между engines или выбор следующего business target в Kernel также
нарушают clean-room boundary.

Текущий custodian уже имеет достаточное durable evidence своей custody:

- predecessor `BlobCustodySourceProofV1`;
- exact evidence ID и envelope SHA-256, которыми был выполнен transfer;
- deterministic target reference, вычисленный из predecessor proof и evidence;
- собственный current managed runtime/grant и Blob custody scope.

Эта цепочка позволяет Kernel выпустить новую ограниченную delegation authority,
не читая Blob metadata или plaintext и не утверждая, что Blob физически
существует. Окончательную проверку source metadata и custody выполняет Blob
runtime при следующем transfer.

## Решение

### Отдельная managed control operation

Runtime protocol получает отдельные сообщения:

```text
ManagedRuntimeBlobCustodyDelegationRequestV1
  request_id
  capability_id
  current_reference_id
  predecessor_custody_source_proof
  predecessor_evidence_id
  predecessor_evidence_envelope_sha256
  target_owner_id
  target_module_id
  target_capability_id

ManagedRuntimeBlobCustodyDelegationDeliveryV1
  request_id
  custody_transfer_source_proof
```

Это control-plane issuance, а не Blob data session, business RPC или generic
delegation API. Request не содержит bytes, path, provider identity, filename
или произвольный business payload.

### Kernel authorization

Kernel принимает запрос только по authenticated managed channel и:

1. проверяет current registration, runtime instance/generation и grant epoch;
2. разрешает exact `capability_id` только при current Blob quota operation
   `CustodyTransfer`;
3. криптографически проверяет predecessor proof в lineage/release режиме, где
   истёкший transfer TTL допустим, но signature, Kernel instance и структура
   остаются обязательными;
4. проверяет, что predecessor target authorizes exact current
   owner/module/capability;
5. вычисляет current reference из exact predecessor proof bytes, predecessor
   evidence ID и envelope SHA-256 и требует exact equality с request;
6. валидирует полностью заполненный bounded target triple;
7. выпускает новый signed `BlobCustodySourceProofV1` с source fences текущего
   custodian, current reference, size/digest/backup/expiry из predecessor
   lineage и exact новым target triple.

Новый proof получает additive issuance kind
`CURRENT_CUSTODIAN_REDELEGATION_V1`, stable 16-byte delegation ID из
`request_id` и SHA-256 predecessor proof. Legacy proof без этих полей
интерпретируется только как `ORIGINAL_WRITE_V1`.

Kernel не вызывает Blob runtime при issuance, не читает metadata и не обещает
наличие source object. При target transfer Blob runtime остаётся final
authority: он проверяет actual source reference, custody scope, access fence,
receipt digest и current grant.

### Transfer semantics

Для `ORIGINAL_WRITE_V1` source capability по-прежнему обязана иметь Blob
operation `Write`.

Для `CURRENT_CUSTODIAN_REDELEGATION_V1` source capability обязана иметь
`CustodyTransfer`. Partial или unknown issuance fields fail closed.

Target reference для redelegated proof детерминирован stable delegation ID,
source reference, exact target triple и consumed target evidence. Повторный
issuance после retry поэтому не создаёт второй target reference, даже если
короткоживущие issued/expiry timestamps и signature bytes отличаются.

Redelegated proof:

- не является read grant;
- не переносит custody сам по себе;
- работает только для exact target triple;
- требует current source and target registration/grant/runtime fences при
  transfer;
- связывается target runtime с exact consumed durable evidence;
- не даёт Kernel права выбирать следующий owner.

### Event-only business flow

Business owner, которому требуется Blob, публикует typed durable delegation
request contract. Текущий custodian обрабатывает его owner-locally, вызывает
эту managed control operation и атомарно сохраняет новый proof в своём outbox
result/event. Target получает proof только через Event Hub и выполняет обычный
evidence-bound custody transfer.

```text
target owner durable delegation request
  -> current custodian inbox
  -> Kernel managed custody-delegation issuance
  -> current custodian durable delegated event
  -> target owner inbox
  -> ordinary Blob custody transfer
```

Kernel, Blob Platform и Event Hub не интерпретируют business reason. Archive
Inspection будет первым consumer, но platform contract не содержит archive,
security или Communications identifiers.

## Единицы сборки и SRP

- runtime protocol владеет только wire messages и strict validation;
- Kernel Blob control plane владеет lineage verification и proof issuance;
- Blob client владеет typed managed-channel request/response;
- Blob runtime владеет actual custody metadata and transfer execution;
- source owner persistence владеет request replay и atomic result outbox;
- public owner contracts владеют business request/result events.

Kernel не импортирует Attachment Security или Archive Inspection packages.
Attachment Security не импортирует archive runtime/implementation. Archive
Inspection не вызывает Attachment Security через request RPC и не читает его
storage.

## Required evidence

1. Exact predecessor target/current runtime/capability lineage выпускает
   redelegated proof.
2. Wrong predecessor target, reference, evidence ID/hash, target triple,
   signature, Kernel instance или partial fields fail closed.
3. Stale/revoked current source registration, generation or grant fail closed.
4. Original proof требует source `Write`; redelegated proof требует source
   `CustodyTransfer`.
5. Exact retry delegation ID приводит к одному deterministic target reference.
6. Blob runtime отклоняет redelegated proof при absent/mismatched source
   metadata, custody scope, receipt, source grant или target grant.
7. Proof, references and predecessor lineage отсутствуют в public client
   output, logs, health and diagnostics.
8. Runtime protocol, Kernel, Blob client/runtime, architecture, SRP, Cargo and
   managed live conformance проходят.

## Отклонённые варианты

### Переиспользовать Mail -> Attachment Security proof

Отклонено: proof подписан для другого exact target и стал бы bearer authority.

### Attachment Security возвращает bytes в integration

Отклонено: добавляет обратный content path, повторную запись и provider
coupling.

### Archive engine вызывает Attachment Security request RPC

Отклонено: cross-engine business coordination должна быть durable и replayable
через typed commands/events.

### Kernel проверяет business event contract и выбирает target

Отклонено: Kernel начал бы интерпретировать owner payload и стал бы workflow.

### Kernel синхронно читает Blob metadata перед issuance

Отклонено: это не усиливает final transfer authority, но связывает control
issuance с доступностью Blob runtime. Exact lineage и current managed fences
проверяются Kernel, actual custody — Blob runtime атомарно при transfer.

### Создать новый Blob copy

Отклонено: увеличивает private-content surface, quota и cleanup complexity без
необходимости. Custody transfer уже выполняет internal rewrap без plaintext в
module control channel.

## Последствия

Макошь получает безопасную цепочку нескольких independently owned content
processors. Каждый переход остаётся exact target-bound, event-backed и
replayable; integrations, domains, workflows и engines не импортируют чужую
implementation, а Kernel и Blob Platform не становятся business facades.
