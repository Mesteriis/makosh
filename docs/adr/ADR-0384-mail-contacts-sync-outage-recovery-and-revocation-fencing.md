# ADR-0384: Mail Contacts sync outage recovery and revocation fencing

Статус: Принято

Дата: 2026-08-03

Состояние реализации: implemented для managed outage/recovery/revoke slice.
Signed managed Vault/Storage/PgBouncer/NATS/Mail/workflow/Contacts ensemble
доказывает все failure-isolation условия этого ADR. Общий
`mail_contacts_sync_v1` остаётся planned до отдельного browser
Start/Get/shared-SSE gate; наличие этого ADR само по себе его не открывает.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0379](ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md);
- [ADR-0382](ADR-0382-mail-address-book-provider-execution-and-authority.md);
- [ADR-0383](ADR-0383-contacts-provider-link-reconciliation-after-mail-write.md).

## Контекст

Mail address-book write пересекает две разные failure boundaries. До отправки
provider request недоступность NATS, Vault, Blob или provider endpoint не
создаёт remote mutation. После отправки `POST` или `PATCH` потеря ответа не
позволяет определить, применил ли provider запрос. Автоматический повтор во
втором случае способен создать duplicate contact или перезаписать более новую
provider revision.

Owner revoke, runtime generation и grant epoch являются отдельной authority
boundary. Сохранённый command или client route не может продолжить provider IO
через revoked/stale process только потому, что business operation уже принята.

## Решение

### Outbox и infrastructure outage

Каждый owner сохраняет exact envelope в своём PostgreSQL outbox до publish.
NATS outage не терминализирует business operation и не вызывает provider IO.
После восстановления broker тот же message ID и exact bytes публикуются без
re-encode; consumer inbox подавляет duplicate delivery.

Порядок публикации outbox не является доказательством порядка применения
разных subjects. Если `page_completed` доставлен workflow раньше всех
`entry_observed` этой страницы, persistence возвращает explicit
`PendingPrerequisites`, consumer не делает Ack и повторяет применение после
causal prerequisites. Только `recorded_entries > observed_entries` является
нарушением контракта; нормальная межsubjectная задержка не завершает process и
не расходует bounded restart attempts.

Contacts разделяет canonical mutation и provider provenance refresh. Новые
ETag, `source_revision` и `observed_at` при неизменных нормализованных
presentation fields атомарно обновляют только Contacts-owned provider link,
возвращают `Unchanged`, не повышают `contact_revision` и не публикуют
`ContactChangedForMailSyncV1`. Поэтому recovery может закрепить наблюдённый
provider ETag без feedback write.

Workflow-created cross-owner commands получают единый bounded deadline 300
секунд от момента создания. Это не retry budget и не разрешение повторять
provider mutation: deadline только ограничивает допустимость ещё не начатой
доставки через последовательные managed event pumps. После expiry требуется
новый explicit Start/owner action с новой operation identity.

Для provider observation `entry_digest` доказывает exact material и участвует
в idempotency, но не используется как ordering revision. `source_revision`
равен положительному Mail-owned observed Unix time; Contacts сравнивает его как
freshness fence. Hash-prefix не является монотонной ревизией и не может
отбрасывать более новый ETag как stale.

Недоступность Vault/Blob/provider до отправки write возвращает bounded definite
rejection. Новый provider attempt допускается только новым явным Start с новой
operation identity после восстановления authority. Workflow не содержит
скрытого timer retry provider mutation.

### Outcome unknown после provider write

Любая потеря transport/response после начала HTTP write становится exact
`OUTCOME_UNKNOWN` terminal result. Mail сохраняет результат до Ack, workflow
завершает связанную reverse operation и run как rejected и не выпускает второй
Mail command для той же operation.

Recovery выполняется observation-first:

```text
ambiguous provider write
  -> OUTCOME_UNKNOWN terminal result
  -> no automatic provider retry
  -> later explicit Start in the admitted configuration direction
  -> provider-to-Contacts observation phase
  -> provider observation with provider ID/ETag
  -> Contacts identity reconciliation
  -> Contacts-owned provider link
```

Таким образом provider observation, а не workflow guess, подтверждает remote
truth. Если mutation не была применена, новый explicit reverse operation может
быть создан только после нового Contact change/revision или owner action; старый
command identity никогда не переисполняется автоматически.

### Revoke и stale authority

Owner revoke сначала повышает grant epoch и переводит registration/storage
binding в fenced state, затем останавливает exact managed process. Старые
client routes, event permits, Vault leases, runtime generation и grant epoch
отклоняются до provider IO. Revoke workflow не останавливает Mail, Contacts,
NATS или другой owner; revoke Mail не даёт workflow прямого fallback path.

Kernel и Event Hub выполняют только authority/routing fence и не интерпретируют
account, Contact, provider response или recovery policy.

## Границы и SRP

- provider adapter классифицирует transport phase и provider response;
- Mail persistence владеет dispatch state и exact terminal result;
- workflow persistence владеет operation/run transition и не повторяет Mail
  mutation после `OUTCOME_UNKNOWN`;
- Contacts владеет canonical identity и provider link reconciliation;
- managed conformance fixture управляет outage/revoke, но production runtime не
  получает test switches;
- отдельный browser slice доказывает generated Start/Get и shared SSE; он не
  подменяется managed IPC test.

## Gate

Failure-isolation slice считается implemented только если signed managed
ensemble доказывает:

1. NATS outage сохраняет pending owner-local outbox и нулевой provider IO;
2. broker recovery доставляет exact command один раз;
3. потеря ответа после принятого Google write даёт `OUTCOME_UNKNOWN` и не
   повторяет remote mutation после redelivery/restart;
4. последующий explicit Start в admitted direction выполняет provider
   observation и reconciles provider ID/ETag в Contacts без direct SQL между
   owners;
5. stale generation/grant и owner revoke отклоняют Start/event route до
   provider IO и не останавливают соседние owners;
6. private contact/provider/token material отсутствует в subjects, health,
   sanitized errors и realtime;
7. architecture, Cargo, PostgreSQL, managed runtime и full pre-push gates
   проходят.

## Отклонённые варианты

### Автоматически повторять POST после timeout

Idempotency Google People create не доказана Макошь command ID, поэтому такой
retry создаёт duplicate risk.

### Считать transport failure definite provider rejection

После отправки request это выдумывает external truth и разрешает опасный retry.

### Хранить recovery link в workflow

Provider link является Contacts-owned canonical provenance; workflow projection
не может заменить Contacts mutation authority.
