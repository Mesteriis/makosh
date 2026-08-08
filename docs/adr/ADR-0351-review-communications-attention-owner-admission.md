# ADR-0351: Review Communications attention owner admission

Статус: Принято

Дата: 2026-07-30

Состояние реализации: implemented. Exact client contract, pure core,
owner-local PostgreSQL command/query/realtime persistence, managed runtime,
unsigned release assembly и signed distribution composition реализованы.
Exact Kernel admission, owner-neutral Gateway command/query routes, shared SSE
и restart-safe durable replay подтверждены live managed conformance.
`review_communications_attention_v1` открыт как implemented.

Уточняет:

- [ADR-0207: canonical domain registry](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0213: code ownership and module autonomy](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0253: Communications legacy disposition](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Historical Communications UI смешивал provider actions и Макошь attention
state. Provider read/archive/mute/label принадлежат integration runtime.
Макошь pending/reviewed/dismissed, pin, importance и snooze являются отдельной
business truth и принадлежат Review.

Хранение этих полей в Communications создало бы скрытый Review facade. Импорт
Communications domain из Review или Review из Communications нарушил бы
compile isolation. Review owner ранее был blocked, поэтому contract и runtime
нельзя было добавлять без отдельного phase gate.

## Решение

Review открывается как отдельный domain owner начиная с двух build units:

- `makosh-review-attention-api` — generated command/query/realtime contract;
- `makosh-review-attention-core` — pure attention aggregate and invariants.

Owner-local persistence реализован отдельной unit
`makosh-review-attention-persistence`: operation ID вместе с exact request hash
даёт idempotent replay, а aggregate mutation и operation result фиксируются в
одной PostgreSQL transaction. Owner-local get/list использует bounded keyset
paging, а каждое semantic изменение атомарно добавляет durable realtime
transition для restart-safe SSE replay.

`makosh-review-attention-runtime` является самостоятельным managed domain
process. Он получает module owner `review` и отдельный authenticated human
owner, использует только Review API/core/persistence и platform
Runtime/Storage/Vault contracts, не запрашивает Event Hub и не импортирует
Communications. Durable replay читается один раз при старте runtime; после
этого новые client realtime frames публикуются только вследствие принятой
Review command. Периодический query polling и таймерный pump отсутствуют.

`makosh-review-attention-assembly` отдельно материализует canonical descriptor,
settings schema, Storage migration bundle и отсортированный unsigned release
fragment. Assembly не запускает runtime, не импортирует Kernel/Gateway и не
получает signing authority.

Следующие units реализованы отдельными slices:

- signed distribution и exact Kernel admission;
- generated Gateway routes и shared owner-local SSE;
- app composition, которая связывает opaque Review source reference с
  Communications canonical read только на клиентском/application уровне.

Review packages не зависят от Communications packages. Source evidence
передаётся как opaque stable 16-byte ID. Review не читает Communications SQL,
не вызывает Communications runtime и не декодирует provider identity.

## Attention semantics

Attention state состоит из независимых typed dimensions:

- disposition: `pending`, `reviewed`, `dismissed`;
- pinned flag;
- importance: `normal`, `important`;
- optional bounded snooze deadline.

Первое действие создаёт owner-scoped aggregate с deterministic attention ID.
Каждая mutation требует exact expected revision. Semantic no-op не увеличивает
revision. Dismiss очищает pin и snooze; дальнейшие mutations требуют explicit
restore в `pending`. Snooze должен быть в будущем и не дальше 366 дней.

Provider actions, message content, subjects, addresses, account IDs, provider
locators, credentials и sessions отсутствуют в contract, core, logs и errors.

## Client contract

Command, query и realtime являются разными approval capabilities:

```text
review.communication-attention.command.v1
review.communication-attention.query.v1
review.communication-attention.realtime.v1
```

Command принимает exact operation ID, opaque source evidence ID, expected
revision и одно typed action. Query возвращает только Review-owned state.
Realtime содержит attention ID, revision и client-safe state без source content.

## Cross-owner flow

```text
client action
  -> generated Review command through Core Gateway
  -> Review runtime
  -> Review owner-local state
  -> shared owner-local SSE

application composition
  -> Review query
  -> Communications canonical query
  -> UI composition only
```

Communications не импортирует Review и не выполняет Review command.
Автоматическое создание attention item из Communications event в этом slice
не вводится. Если оно понадобится, отдельный workflow/target consumer должен
преобразовать source event в Review command через event spine.

## Phase gate

Этот ADR разрешает Review owner и exact
contract/core/persistence/runtime/assembly package inventory.
`review_communications_attention_v1` открыт после:

1. release assembly and exact Kernel admission;
2. generated command/query/realtime Gateway routes;
3. restart, replay, stale revision, cross-owner and privacy-negative tests;
4. live managed proof through Gateway and shared SSE.

Live gate запускает подписанный Review runtime без Event Hub grant, применяет
owner-local Storage bundle через Vault-fenced credential, выполняет generated
command/query через Gateway, отклоняет stale revision, проверяет отсутствие
source evidence ID в SSE и после successor restart восстанавливает тот же
durable cursor. Отдельный identity-negative test отклоняет запрос другого
authenticated human owner до business dispatch. Bounded nested dispatcher
обрабатывает следующий client command во время ожидания ACK предыдущей
realtime publication; восемь последовательных busy passes являются жёстким
пределом, после которого runtime fail-closed перезапускается и дочитывает
durable replay.
