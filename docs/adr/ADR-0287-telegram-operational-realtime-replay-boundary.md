# ADR-0287: Telegram operational realtime replay boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Реализовано в `telegram_core_operational_v1`: exact
capability, generated route, bounded owner-local replay, explicit reset,
restart-safe runtime restore и live managed positive/negative conformance
существуют. Slice остаётся вне production owner inventory до отдельного
Telegram umbrella admission; этот ADR сам по себе capability не открывает.

Уточняет:

- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0221: capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0265: provider operational transport](ADR-0265-provider-operational-client-transport-admission.md);
- [ADR-0266: Telegram admission](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Telegram уже имеет отдельные authorization, lifecycle, command и query
capabilities. Provider event journal и restart-safe replay persistence также
существуют, но public `Replay` намеренно отклоняется: lifecycle/query contract
не должен тихо становиться realtime surface.

ADR-0282 требует command status, replay и reconciliation для полного Telegram
operational client. Закрыть это требование через Query, вернуть internal
`DurableEnvelopeV1` или восстановить umbrella `telegram.client` нельзя. Нужна
отдельная capability с собственной причиной изменения, grant и route.

## Решение

Telegram остаётся одним integration owner:

```text
owner_id  = telegram
module_id = makosh-telegram-runtime
```

Realtime является отдельной функциональной unit внутри существующих
integration-owned packages, а не новым domain/runtime:

```text
makosh-telegram-api          generated replay request/page and validation
makosh-telegram-persistence  owner-local journal bounds and ordered replay
makosh-telegram-runtime      exact route handler
makosh-telegram-assembly     immutable descriptor and release admission
```

Новая capability:

```text
capability = telegram.realtime.v1
contract   = telegram.realtime.v1
route      = /makosh.telegram.v1.TelegramRealtimeService/Replay
gate       = telegram_core_operational_v1
```

Она не является alias lifecycle/query и не принимает их payloads. Shared
descriptor bytes не объединяют grants: contract name, route и capability
identity остаются exact.

### Generated replay contract

Request содержит:

- exact `account_id`;
- `after_sequence`;
- bounded `limit`, `1..=5000`.

Response содержит:

- typed Telegram operational frames в строго возрастающем owner-local
  `sequence`;
- `earliest_available_sequence`;
- `latest_sequence`;
- `next_after_sequence`;
- explicit `reset_required`.

Frame содержит только generated typed provider event, account, sequence и
bounded provider cursor. Internal durable envelope, credentials, session
paths, provider raw JSON, message bodies внутри route/error metadata и generic
maps запрещены.

Replay scoped к одному account. Cross-account frames запрещены. Если requested
cursor новее latest либо retention/upgrade оставил gap до earliest, runtime
возвращает empty page с `reset_required = true`; silent empty success
запрещён. При пустом journal cursor `0` валиден, любой ненулевой cursor требует
reset.

### Kernel, Core и Communications

Kernel/Core только допускают и fence-ят exact opaque ClientRpc route по
registration, runtime generation, GrantSet и grant epoch. Они не декодируют
Telegram event, не читают journal и не выбирают account.

Communications не импортирует Telegram contract/runtime и не потребляет client
replay. Canonical evidence по-прежнему пересекает owner boundary только через
Telegram outbox, NATS и Communications inbox.

Frontend Telegram controller позднее использует generated realtime client
через Core Gateway/SSE composition. Наличие backend route не закрывает
`telegram_full_operational_v1` без frontend cutover.

## Gate

`telegram_core_operational_v1` может учитывать realtime только при наличии:

1. generated exact route и отдельной descriptor capability;
2. bounded account-scoped ordered replay page;
3. explicit future/gap cursor reset;
4. current runtime-generation/grant-epoch fencing и ungranted negative
   conformance;
5. restart-safe managed runtime replay;
6. отсутствия Query/Lifecycle alias и internal envelope leakage;
7. package, Clippy, architecture и live managed evidence.

## Отклонённые варианты

### Вернуть Replay в lifecycle

Отклонено: lifecycle grant получил бы provider event history и вторую
функциональную ответственность.

### Расширить query response

Отклонено: snapshot query и ordered replay имеют разные cursor, retention,
reset и delivery semantics.

### Использовать Communications realtime

Отклонено: Telegram operational truth и canonical Communications evidence
принадлежат разным owners и имеют разные contracts/storage.
