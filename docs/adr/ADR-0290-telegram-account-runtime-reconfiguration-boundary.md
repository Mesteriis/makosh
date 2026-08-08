# ADR-0290: Telegram account runtime reconfiguration boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Реализовано для
`telegram_runtime_reconfiguration_v1`. Gate подтверждён typed contract,
owner-local durable state machine, физической заменой TDLib client, fresh
process-bound Vault leases, exact-grant negative conformance, normal managed
replacement и recovery того же target epoch после managed process replacement.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0214: durable jobs and runtime reconfiguration](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0222: Kernel settings registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0240: Telegram clean-room provider boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0266: Telegram admission and event-only handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Historical Telegram runtime manager действительно удалял account actor и
создавал новый TDLib actor. Текущий clean-room `RestartAccount` этого не
делает: client передаёт `topology`, `holder`, lease expiry и `now`, после чего
runtime только меняет owner-local projection и lease в памяти. Provider client
остаётся тем же. Такая операция:

- делегирует client техническую runtime authority;
- не заменяет TDLib session;
- теряет lease при process restart;
- может вернуть success до durable write;
- выдаёт fake atomic restart за реализованный behavior.

Telegram managed process уже является account-scoped: один admitted
configuration instance обслуживает один Telegram account и один TDLib client.
Поэтому account runtime reconfiguration означает замену provider client/session
внутри существующего admitted Telegram process. Это не изменение Kernel module
topology, executable binding, settings revision или managed runtime generation.

## Решение

Вводится отдельный integration-owned client contract:

```text
owner_id  = telegram
module_id = makosh-telegram-runtime
route     = /makosh.telegram.v1.TelegramReconfigurationService/Execute
contract  = telegram.reconfiguration.v1
gate      = telegram_runtime_reconfiguration_v1
```

Functional units остаются разделены по причинам изменения:

```text
makosh-telegram-api          typed request, receipt and status
makosh-telegram-core         epoch/state transition invariants
makosh-telegram-tdlib        replaceable provider-client factory boundary
makosh-telegram-persistence  durable intent, fencing and completion
makosh-telegram-runtime      stop/start/restore orchestration
```

Integration assembly только связывает эти units. Communications, Kernel,
Gateway и release assembly не импортируют Telegram reconfiguration semantics.

### Kernel agreement

Kernel:

- admits exact `telegram.reconfiguration.v1` capability and exact client route;
- validates current registration, runtime generation and grant epoch before
  opaque delivery;
- не декодирует account ID, reconfiguration ID или runtime epoch;
- не принимает Telegram account lifecycle decisions;
- не выдаёт client Telegram topology, holder, lease expiry или process clock.

Telegram account reconfiguration сохраняет current admitted module runtime
generation. Если меняются executable bytes, descriptor, settings binding,
storage binding, grants или managed topology, применяется отдельный Kernel
managed-successor protocol. Telegram route не является обходом этого protocol.

### Client contract

`Begin` содержит только:

- client-generated exact `reconfiguration_id`;
- exact `account_id`;
- `expected_runtime_epoch`.

`Status` содержит только exact `reconfiguration_id`.

Client не передаёт:

- topology;
- holder/runtime instance ID;
- lease expiry;
- current time;
- managed runtime generation или grant epoch;
- provider credentials, TDLib parameters или session paths.

Один route использует typed `oneof Begin | Status`. `Begin` возвращает durable
receipt со state `accepted`; это не terminal completion. Exact retry с теми же
ID/account/expected epoch возвращает существующий record. Повтор ID с другим
payload является collision. Одновременно для одного account разрешён только
один non-terminal reconfiguration.

### Durable state machine

Owner-local state:

```text
accepted -> applying -> completed
                    \-> failed
```

Record содержит reconfiguration ID, account ID, expected epoch, target epoch,
state и bounded sanitized reason code. Target epoch всегда
`expected_runtime_epoch + 1`; overflow rejected.

Acceptance под PostgreSQL transaction:

1. locks current Telegram account row;
2. checks exact expected epoch and running/degraded state;
3. rejects another non-terminal account reconfiguration;
4. inserts collision-safe durable intent;
5. returns `accepted`.

Wrong epoch или collision ничего не останавливает и не меняет.

### Physical provider replacement

После того как accepted response записан клиенту:

1. record становится `applying`;
2. runtime получает fresh process-bound Vault leases для exact API hash и
   session wrapping key, затем готовит новый TDLib client через retained exact
   library handle; secret parameters zeroize после authorization handoff и не
   кэшируются для будущих restart;
3. active call media is fenced and stopped;
4. old TDLib client is physically dropped;
5. fresh client проходит authorization/session restore;
6. Telegram account and all bounded owner projections restore from PostgreSQL;
7. account runtime epoch and completed record commit atomically;
8. status становится `completed` только после доступности replacement runtime.

Новый client нельзя публиковать как running до durable restore. Provider
authorization error оставляет record non-terminal/failed with a sanitized code;
старый client не объявляется восстановленным задним числом.

### Crash recovery and idempotency

Если process завершился после `accepted` или `applying`, новый admitted process
сам является физической заменой provider session. После authorization и
durable restore он находит единственный pending record и атомарно завершает тот
же target epoch. Повторный client request не создаёт второй epoch.

Если crash произошёл после account update, но до client observation, transaction
уже содержит одновременно `completed` record и target epoch. Status query
возвращает этот terminal result.

Process restart может физически создать provider client более одного раза при
повторных crash, но durable target epoch применяется ровно один раз. Макошь не
обещает невозможную атомарность между PostgreSQL и внешним TDLib process; он
гарантирует durable intent, fencing и convergent terminal state.

### Startup, stop and lifecycle aliases

Legacy public `StartAccount`, `StopAccount` и `RestartAccount` aliases не
сохраняются в `telegram.lifecycle.v1`: они либо меняли только projection, либо
передавали client runtime authority. Managed account start является частью
admitted process bootstrap. Explicit provider-session replacement доступен
только через `telegram.reconfiguration.v1`.

Retire account остаётся отдельной durable owner lifecycle operation и не
подменяет runtime stop.

### Privacy and errors

Reconfiguration payload и status owner-private. IDs и epochs не попадают в
subjects, health или logs. Wire error возвращает bounded stable code без
provider text, session path, topology, holder или credential detail.

## Gate `telegram_runtime_reconfiguration_v1`

Gate становится `implemented` только при наличии:

1. separate generated contract, route and exact capability;
2. no client-controlled topology, holder, lease expiry or clock;
3. transactional acceptance with expected-epoch fencing and collision safety;
4. accepted receipt distinct from completion;
5. physical old TDLib client destruction and fresh client creation;
6. active call-media fencing before provider replacement;
7. durable projection restore before terminal completion;
8. atomic account-epoch plus record completion;
9. accepted/applying crash recovery without second target epoch;
10. wrong-epoch and missing-grant live fail-safe evidence;
11. package, Clippy, architecture and managed restart conformance;
12. removal of fake public lifecycle restart aliases.

Gate сам по себе не открывает `telegram_full_operational_v1` и не добавляет
Telegram в production inventory.

## Фактическая реализация

- `makosh-telegram-api` публикует только exact
  `telegram.reconfiguration.v1` Begin/Status contract; fake public
  `StartAccount`, `StopAccount` и `RestartAccount` удалены и зарезервированы на
  wire.
- `makosh-telegram-core` владеет expected/target epoch и terminal transition
  invariants.
- `makosh-telegram-persistence` сохраняет единственный active account intent,
  exact retry/collision fence и одну transaction для account epoch плюс
  completed record.
- `makosh-telegram-tdlib` удерживает exact verified library handle и создаёт
  новый client; drop старого client вызывает `td_json_client_destroy`.
- `makosh-telegram-runtime` после accepted response заново получает exact Vault
  credential leases, останавливает call media, заменяет TDLib client и завершает
  record только после durable restore.
- Managed conformance доказывает exact missing-grant rejection, свежий provider
  client при обычной reconfiguration и recovery того же target epoch после
  process replacement без повторного epoch increment.

## Отклонённые варианты

### Перезаписывать только runtime projection

Отклонено: provider client/session не меняется, поэтому это fake restart.

### Дать client topology, holder, expiry или now

Отклонено: это authority integration runtime и admitted infrastructure, а не
UI.

### Перезапускать весь managed module через Telegram-specific Kernel branch

Отклонено: Kernel стал бы зависеть от owner semantics. Managed successor нужен
для module binding/topology changes, а не для account-scoped provider refresh.

### Считать TDLib client creation terminal success

Отклонено: без authorization и durable restore replacement runtime ещё не
готов.

### Делать generic reconfiguration workflow

Отклонено: операция заменяет только Telegram-owned provider session и не
координирует несколько owners.

## Rollback

Revoke `telegram.reconfiguration.v1` останавливает новые Begin requests.
Accepted record сохраняется и либо converges через admitted Telegram runtime,
либо остаётся с terminal sanitized failure. Legacy lifecycle alias и REST
fallback запрещены.
