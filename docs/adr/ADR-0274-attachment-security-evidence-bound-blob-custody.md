# ADR-0274: Attachment Security evidence-bound Blob custody

Статус: Принято
Дата: 2026-07-25
Состояние реализации: Решение принято после live managed conformance, который
доказал, что прямой read integration-owned Blob правильно отклоняется
registration/capability fence. Contract revision 2 и engine-owned custody
transfer реализованы. Live disposable contour доказал target-bound
`mail -> attachment_security` transfer, сохранение target receipt, one-use
target-owned read, clean ClamAV verdict, Communications CAS, exact replay без
повторного scan и отдельный отказ прямого source read. Тот же contour теперь
доказывает threat quarantine и fail-closed malformed/disconnect/timeout без
verdict/outbox и без перевода Communications из `blob_admitted`. NATS outage
после scan сохраняет exact verdict в owner outbox; после остановки Engine и
fenced Communications consumer/Storage successor Engine generation 2 публикует
те же bytes без повторного custody/scan, а stale Communications CAS не меняет
terminal state. Phase gate
теперь дополнительно имеет live fail-closed evidence для stale source runtime,
revoked source/target registrations и недоступных Blob/Vault: retryable job
остаётся без target receipt/outbox/verdict, scanner не вызывается, а
Communications остаётся в `blob_admitted`. Exact replay не обходит authority
fence. Exact `attachment_security_engine_v1` production inventory теперь
допущен executable policy после architecture/SRP/Cargo/full backend gates;
никакой integration или Communications implementation package не добавлен в
engine build unit.
ADR-0275 определяет stable target-bound proof для этого cross-owner handoff.
ADR-0327 уточняет successor semantics: current source registration/grant
обязательны, но benign process generation successor не уничтожает уже
опубликованную target-bound durable delegation.

Зависит от:

- [ADR-0220: canonical durable envelope and contract evolution](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data sessions](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0257: event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0273: Attachment Security engine](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md);
- [ADR-0275: target-bound cross-owner Blob custody](ADR-0275-target-bound-cross-owner-blob-custody-delegation.md).

## Контекст

Integration runtime создаёт attachment Blob под собственными exact
registration, capability, runtime generation и grant epoch. Opaque reference
не является bearer token: Blob Platform обязан отклонять попытку другого
runtime прочитать его даже при том же logical owner.

ADR-0273 требовал от Attachment Security запросить one-use read session по
integration-owned reference. Это противоречит fence ADR-0230/0231 и уже
реализованному решению ADR-0257. Выдача engine исходной integration capability,
generic cross-owner read или особый bypass в Kernel разрушили бы capability
isolation. Передача bytes через Kernel, Communications или NATS превратила бы
control/event plane в content proxy.

Integration уже получает после успешного Blob write bounded Kernel-signed
custody source proof. Этот proof позволяет текущему target runtime запросить у
Blob Platform evidence-bound rewrap в собственную custody без передачи
plaintext через Kernel.

## Решение

### Candidate contract revision 2

`attachment_security_scan_candidate_observed` сохраняет major `1` и получает
additive contract revision `2` с новым полем:

```text
bytes custody_transfer_source_proof = 6;
```

Поле обязательно по owner validation, имеет размер `1..=2048` bytes и содержит
только opaque Kernel-signed proof, возвращённый exact source Blob write
session. Это authority-bearing bounded evidence, но не credential, key,
provider locator, content, filesystem path или business truth.

Изменение schema descriptor создаёт новый revision/hash согласно ADR-0220.
Revision 1 не был открыт production gate и не допускается как fallback.
Producer и required subscriber переходят на revision 2 атомарно через Event
Hub catalog reconciliation.

Candidate envelope остаётся единственным integration-to-engine business
handoff. Exact envelope bytes дают engine:

- candidate message ID как evidence ID;
- SHA-256 exact envelope как evidence-envelope binding;
- source reference, size и receipt digest;
- opaque custody source proof;
- canonical attachment anchor, causation и correlation.

Proof хранится только в engine-owned inbox/candidate persistence. Он не
попадает в scan verdict, Communications event, public query, logs, health или
diagnostics.

### Target-owned custody before scan

После conflict-free join candidate с canonical
`blob_pending -> blob_admitted` engine:

1. запрашивает у Kernel managed Blob custody-transfer session по собственной
   Blob capability, current runtime generation и grant epoch;
2. передаёт exact source reference/size/receipt, source proof, candidate message
   ID и SHA-256 сохранённых exact candidate envelope bytes;
3. Blob Platform проверяет current source и target fences и idempotently
   rewrap/rebind-ит content в target-owned opaque reference;
4. engine сохраняет target receipt в owner-local job state;
5. отдельная one-use read session разрешается только для target-owned
   reference и exact target receipt;
6. bytes идут из private Blob data socket прямо в bounded scanner adapter.

Прямой read source reference остаётся запрещён и является обязательным
negative conformance case. Kernel выдаёт и fence-ит sessions, но не декодирует
candidate payload, не читает bytes, не вызывает scanner и не видит verdict.

Engine использует одну owner-scoped Blob capability
`attachment_security.blob.v1` как единицу approval для собственного custody
quota и read data plane. Эта capability не даёт доступа к integration-owned
reference без exact source proof и evidence binding.

### Failure, replay и retention

- exact candidate replay возвращает тот же deterministic target receipt и не
  запускает второй scan/verdict;
- Blob/Vault/control transport unavailability сохраняет retryable job без
  permissive verdict;
- altered proof, reference, size, receipt, evidence ID/hash, stale source или
  target runtime, revoked grant и expired proof fail closed;
- policy denial не имеет alternate direct-read path и не может быть
  интерпретирован как clean;
- source Blob retention остаётся ответственностью integration; transfer ничего
  не удаляет;
- target Blob retention и retry state принадлежат Attachment Security.

## Единицы сборки и SRP

- integration runtime: provider download, source Blob write, source proof и
  typed candidate outbox;
- `makosh-attachment-security-contract`: revisioned provider-neutral candidate
  schema и builder;
- Attachment Security core: pure join/verdict policy без Blob protocol;
- Attachment Security persistence: inbox, proof/evidence binding, target
  receipt и scan job lifecycle;
- Attachment Security runtime: custody/read orchestration через public Blob
  client;
- Blob Platform: proof verification, rewrap/rebind и private data sessions;
- Kernel/Core: admission, capability routing, leases и fencing only;
- Communications: canonical attachment state и verdict CAS only.

Ни одна unit не импортирует integration implementation или чужое storage.
Custody transfer является platform operation, а не domain/integration RPC и не
делает Blob или Kernel бизнес-владельцем.

## Required evidence

1. Mail owner-local Blob commit атомарно сохраняет revision-2 candidate с exact
   source proof, не раскрывая proof в diagnostics.
2. Engine принимает candidate/state в любом порядке, выполняет один
   evidence-bound custody transfer и читает только target-owned reference.
3. Live loopback ClamAV clean и threat paths доходят через engine outbox, NATS
   и Communications CAS.
4. Exact replay не создаёт второй transfer, scan или verdict.
5. Прямой source read, altered evidence/proof/receipt, stale source/target,
   revoke, Blob/Vault outage и NATS replay fail closed.
6. Negative output не содержит proof, source/target references, bytes, scanner
   identity, endpoint, private socket path или provider data.
7. Architecture/SRP/Cargo/full backend gates подтверждают отсутствие новых
   cross-owner compile/runtime/storage edges.

## Отклонённые варианты

### Cross-owner read exception

Отклонено: opaque reference превратился бы в bearer token, а Blob registration
fence перестал бы быть security boundary.

### Bytes через Kernel или Communications

Отклонено: Kernel стал бы content proxy, а Communications получил бы scanner
orchestration и вторую причину изменения.

### Integration пишет под capability engine

Отклонено: integration получила бы authority другого owner и смешала бы
независимые build/admission units.

### Proof в Kernel/Event Hub side channel

Отклонено: control plane начал бы интерпретировать owner payload и создавать
скрытый второй transport вместо exact durable event.

## Последствия

Attachment Security может сканировать provider attachment только после
явного, evidence-bound перехода в собственную Blob custody. Event-only граница
между integration, engine и Communications сохраняется, прямой cross-owner
Blob access остаётся запрещён, а Kernel/Core не становится ни доменом, ни
интеграцией, ни data-plane facade.
