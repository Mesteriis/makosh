# ADR-0275: Target-bound cross-owner Blob custody delegation

Статус: Принято
Дата: 2026-07-25
Состояние реализации: Решение принято после live Attachment Security
conformance. Он доказал, что существующий proof правильно разрешает transfer
только между registrations одного module owner и поэтому не поддерживает
реальные `mail -> communications` и `mail -> attachment_security` handoff.
Первоначальный вариант target binding по `registration_id` был отклонён
последующим live evidence: Kernel назначает новый opaque registration ID при
каждой регистрации, поэтому source integration не может знать его без hidden
synchronous dependency. Реализация stable `owner_id + module_id +
capability_id` binding завершена в runtime protocol, public owner contracts,
Kernel proof issuance/verification и Blob data plane. Unit tests доказывают
same-owner fallback, exact/wrong target и distinct source/target fences. Live
disposable Attachment Security contour доказывает настоящий
`mail -> attachment_security` transfer через динамический target registration,
replay и прямой source-read denial. Live authority matrix дополнительно
доказывает stale source runtime generation, revoked source registration,
revoked target registration и недоступность Blob/Vault: transfer остаётся
retryable, scanner и verdict не запускаются. Вместе с unit evidence для
wrong-target/current-runtime fences это закрывает stale/revoke/outage evidence
этого решения. Exact `attachment_security_engine_v1` inventory теперь
допущен отдельным production phase gate; Communications остаётся доменом,
Attachment Security — engine, а integrations в inventory не добавлены.
ADR-0327 последующим live evidence уточняет source fence: exact current source
registration/grant epoch сохраняются, а issuance runtime generation является
provenance и не блокирует approved process successor.

Зависит от:

- [ADR-0215: module admission and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0220: contract evolution](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data sessions](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0257: event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0274: Attachment Security Blob custody](ADR-0274-attachment-security-evidence-bound-blob-custody.md).

## Контекст

`BlobCustodySourceProofV1` связывает source owner, registration, capability,
runtime generation, grant epoch, reference, size и digest. Текущий Kernel
разрешает target transfer только когда `source.owner_id ==
target.owner_id`. Это безопасно для owner-local rebind, но module owner в
clean-room topology — `mail`, `telegram`, `zulip`, `communications` или
`attachment_security`, а не общий human owner.

Предыдущий conformance ADR-0257 использовал fixture source с owner
`communications`, поэтому доказал owner-local transfer, но не реальный
cross-owner integration handoff. Простое удаление equality check превратило бы
source proof в bearer authority: любой runtime с Blob quota и украденным proof
смог бы перенести content в собственную custody.

Kernel не должен определять business recipient по provider, event subject или
payload. Integration также не должна импортировать target runtime
implementation для получения module/capability identity. Kernel
`registration_id` является динамической identity конкретного admission record,
а не публичным именем logical recipient.

## Решение

### Additive target binding

Существующие Protobuf V1 messages получают additive optional поля:

```text
ManagedRuntimeBlobSessionRequestV1:
  string custody_target_owner_id
  string custody_target_module_id
  string custody_target_capability_id

BlobCustodySourceProofV1:
  string target_owner_id
  string target_module_id
  string target_capability_id
```

Все три target fields либо пусты, либо заполнены вместе exact bounded ASCII
tokens. Частично заполненная комбинация invalid.

При owner-local write пустой target сохраняет прежнюю семантику: proof может
быть использован только target capability с тем же owner ID.

При cross-owner write source runtime передаёт exact target audience. Kernel
подписывает target triple вместе со всеми существующими source fences. Такой
proof может быть использован только когда текущий requester одновременно
совпадает по:

- target owner ID из effective Blob quota entry;
- target module ID из текущей fenced managed-runtime expectation;
- target capability ID;
- current opaque registration ID;
- current runtime instance/generation;
- current grant epoch.

Source owner equality в target-bound случае не требуется: source сам явно
делегировал exact content exact recipient capability. Kernel по-прежнему
проверяет current source grant/runtime при transfer. Proof не выдаёт target
read grant; он позволяет только evidence-bound custody-transfer operation.
Opaque target registration ID не входит в proof: это позволяет тому же
logical module обработать durable event после approved successor registration,
но не ослабляет current registration/runtime/grant fences в момент transfer.

### Issuance and availability

Kernel не требует, чтобы target process уже был запущен при source write. Он
валидирует target triple structurally и подписывает source delegation. Current
target admission/grant/runtime проверяются только при transfer. Поэтому
integration outbox может durably сохранить evidence до запуска required
subscriber, не создавая hidden synchronous runtime dependency.

Proof TTL, source runtime/grant fence и exact evidence ID/envelope hash
сохраняются. Expired, stale, revoked, altered или wrong-target proof fail
closed без owner-equality fallback.

### Public custody audience

Exact target triple является частью публичного owner contract, а не runtime
implementation:

- Communications ingress contract публикует canonical body-custody audience;
- Attachment Security candidate contract публикует canonical scan-custody
  audience.

Target runtime descriptor импортирует те же constants для Blob capability и
owner identity. Integration импортирует только уже разрешённый public contract
unit. Duplicate строковые literals в integration/runtime implementation
запрещены architecture test.

Module identity остаётся exact descriptor `module_id` из bundled admission.
Смена module/capability audience требует contract revision и coordinated
producer/subscriber cutover. Смена opaque registration ID при revoke,
re-registration или successor launch не меняет public contract.

### Data and control planes

```text
source integration
  -> Kernel-issued target-bound source proof during own Blob write
  -> exact typed durable owner event
  -> target owner inbox
  -> target requests evidence-bound custody transfer
  -> Kernel verifies signed source + exact target fences
  -> Blob Platform rewraps directly inside private data plane
```

Kernel не декодирует owner event, не получает plaintext и не выбирает target.
Event Hub не хранит второй delegation catalog. Blob Platform не становится
business workflow и не видит provider identity.

## Единицы сборки и SRP

- runtime protocol: wire fields only;
- Blob client: typed target-bound write/session requests;
- Kernel Blob control plane: proof issuance and exact fence verification;
- Blob runtime: signed transfer-grant execution only;
- public owner contract: custody audience constants and event payload;
- source integration: choose the audience declared by the public contract;
- target runtime: own custody/read orchestration.

Ни один source integration package не импортирует target runtime, Kernel
implementation или target storage.

## Required evidence

1. Same-owner proof без target fields сохраняет прежнее owner-equality fence.
2. Target-bound proof разрешает exact source integration -> exact target
   owner/module/capability transfer.
3. Wrong owner, module или capability, partial target, altered signature,
   stale/revoked current registration, source or target и expired proof
   отклоняются.
4. Target может быть offline при source write; durable replay после его
   admission выполняет transfer.
5. Mail/Telegram/Zulip body receipts используют Communications public audience,
   а Mail attachment candidate использует Attachment Security audience.
6. Proof, reference, bytes и target private path отсутствуют в public output,
   logs, health и diagnostics.
7. Live authenticated Blob/Vault/Storage/NATS tests и architecture/SRP/Cargo
   gates подтверждают границы.

## Отклонённые варианты

### Удалить source/target owner equality

Отклонено: unbound proof стал бы transferable bearer authority.

### Kernel выбирает target по event contract

Отклонено: Kernel/Event Hub начал бы интерпретировать business payload и
поддерживать owner-specific routing policy.

### Source импортирует target runtime constants

Отклонено: integration получила бы compile-time зависимость от implementation
другого owner.

### Bind proof к target registration ID

Отклонено: registration ID создаётся Kernel динамически и неизвестен source
runtime при записи Blob. Его discovery добавил бы синхронную зависимость от
target availability, а сохранение ID в public contract сломало бы replay после
re-registration. Stable module ID выбирает logical recipient; Kernel отдельно
проверяет exact current registration, runtime generation и grant epoch.

### Общий owner ID для всех modules

Отклонено: исчезла бы module ownership/capability isolation, а registrations
перестали бы быть независимыми admission units.

## Последствия

Макошь получает явную source-authorized, target-bound cross-owner Blob
delegation без generic read-all и без business логики в Kernel. Реальные
integration -> Communications и integration -> Attachment Security flows
сохраняют event-only boundary, а owner-local legacy proof остаётся совместимым
и строго ограниченным прежним equality fence.
