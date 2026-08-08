# ADR-0317: Communications sender insights derived projection

Дата: 2026-07-28
Статус: accepted
Состояние реализации: implemented

## Контекст

ADR-0282 классифицирует historical Top Senders как
`communications_sender_insights_v1`. Legacy implementation считала только
email-строки, смешивала Communications с mailbox-specific состоянием и
возвращала вычисляемые относительно текущего времени значения. Такой контракт
нельзя переносить напрямую:

- Mail, Telegram, WhatsApp и Zulip остаются отдельными integrations;
- Communications владеет только provider-neutral evidence и derived
  projections;
- provider folders, labels, importance и action state не являются
  Communications truth;
- public client не должен получать provider locator или integration DTO;
- rebuildable projection обязана восстанавливаться из typed canonical
  Communications evidence.

Текущий ingress уже переносит provider-scoped participant cursor, но Mail не
заполняет его, а canonical evidence не несёт bounded display label. Поэтому
полноценный provider-neutral sender insight требует одновременно определить
точную actor semantics и расширить typed evidence без generic metadata map.

## Решение

Принять `communications_sender_insights_v1` как Communications-owned
rebuildable derived projection.

### Owner и communication boundary

- Mail, Telegram, WhatsApp и Zulip integrations остаются владельцами
  provider-specific sender/account/session truth.
- Integration mapper публикует только typed Communications ingress
  observation через существующий durable event path.
- Для message evidence `participant_cursor_sha256` означает provider actor,
  то есть отправителя конкретного сообщения. Для non-message evidence поле
  сохраняет прежнюю participant semantics и не участвует в sender facts.
- Integration может добавить `participant_display_label`: bounded
  provider-neutral display metadata, а не provider locator.
- Communications валидирует label, сохраняет canonical participant и
  материализует message-to-sender fact в той же owner transaction.
- Kernel и Gateway не интерпретируют sender payload. Gateway только проверяет
  exact capability и маршрутизирует opaque generated request в Communications.
- Другие domains и integrations не импортируют Communications implementation
  или storage.

### Typed display label

`participant_display_label` является optional UTF-8 строкой:

- после trim содержит от 1 до 256 bytes;
- не содержит control characters;
- не используется как identity, cursor, authorization input или dedup key;
- не попадает в subject, health, error и logs;
- может содержать private channel address или имя, поэтому остаётся только в
  encrypted/local owner data plane и authorized client response.

Stable identity остаётся provider-scoped participant cursor. Из него
Communications выводит отдельный opaque 16-byte canonical sender ID; observed
participant ID по-прежнему остаётся conversation-scoped. Изменение display
label не меняет sender ID.

Mail mapper передаёт normalized sender mailbox/display value как external
participant identity и label. Telegram mapper передаёт sender ID как external
identity и optional sender display name как label. Отсутствующий label не
блокирует evidence.

### Derived projection

Communications persistence добавляет:

- optional `display_label` в canonical observed participant projection;
- rebuildable sender profile keyed canonical sender ID;
- `communications_message_sender_facts`, где одна active canonical message
  может ссылаться на одного canonical sender, observed participant и account;
- индексы для account-scoped и owner-wide deterministic aggregation.

Fact создаётся только когда одна canonical observation одновременно содержит:

- message projection;
- incoming direction;
- participant projection;
- account projection.

Outgoing, unknown-direction, deleted и не имеющие participant/account
сообщения не считаются top senders. Message edit может обновить label/last
observation, но не создаёт второй message fact. Message delete не удаляет
audit/evidence; aggregate query исключает canonical message tombstone.

Projection rebuildable из canonical Communications evidence. Она не является
canonical truth и не публикует новый cross-owner durable event.

### Public contract

Ввести exact build unit:

```text
makosh-communications-sender-insights-api
```

и generated contract:

```text
makosh.communications.sender_insights.v1
communications.sender-insights.v1
```

Единственная v1 operation:

```text
ListSenderInsights(
  optional canonical account_id,
  bounded limit,
  opaque cursor
)
```

Item содержит:

- opaque canonical sender ID;
- optional bounded display label;
- active incoming message count;
- distinct canonical conversation count;
- first and last observed timestamps.

Contract намеренно не содержит:

- provider ID, provider kind или provider DTO;
- raw participant cursor;
- message body, snippet, subject или Blob reference;
- provider folder/label/read/archive state;
- Review importance, pin, snooze или action score;
- legacy `avg_importance`;
- nondeterministic `last_message_days`.

Порядок:

```text
message_count DESC,
last_observed_at_unix_seconds DESC,
sender_id ASC
```

Opaque cursor checksum-bound и связан с contract revision, optional account
scope, count, last-observed timestamp и sender ID. Подмена scope/revision,
payload corruption или malformed cursor fail closed. Cursor не является
authorization token; authorization заново проверяется на каждом request.

### Client и frontend

Desktop client использует только generated Connect client через общий
transport factory. Communications page показывает отдельный Sender Insights
panel только при exact admitted capability
`communications.sender-insights.v1`.

Panel:

- по умолчанию показывает owner-wide top senders;
- может явно ограничиться выбранным canonical account;
- не делает provider branching;
- не импортирует integration code;
- не превращает insight в Persona, Review item или business truth.

Frontend отсутствие label отображает как короткий canonical sender reference,
не как provider identifier.

### Admission и единицы сборки

`communications_sender_insights_v1` открывается только при наличии:

- ADR-0317 со статусом `accepted`;
- exact public contract build unit;
- ingress/canonical typed actor label conformance;
- atomic sender-fact persistence и deterministic pagination tests;
- Communications runtime handler;
- Gateway exact capability route;
- generated frontend client и Communications-only presentation;
- architecture, SRP, compile-isolation и managed authenticated conformance.

Build unit контракта не является domain или integration runtime. Integration
units не становятся Communications packages, а Communications packages не
получают provider dependencies.

## Отклонённые варианты

### Вернуть legacy email analytics

Отклонено: email-only query и mailbox importance смешивают integration truth,
Review semantics и Communications.

### Считать всех participants conversation как senders

Отклонено: conversation membership не доказывает автора конкретного message.

### Группировать по display label

Отклонено: label изменяем, не является identity и может совпадать у разных
participants.

### Хранить raw provider sender ID

Отклонено: stable provider locator остаётся внутри integration. Communications
получает только scoped cursor и bounded display metadata.

### Вызывать integration operational query из Communications

Отклонено: domain не импортирует integration и не вызывает provider
operational contract.

## Последствия

- Top Senders становится provider-neutral Communications projection.
- Mail и Telegram сохраняют provider ownership, передавая actor только через
  typed ingress observation.
- Display metadata остаётся private owner data и не становится identity.
- Legacy importance analytics не восстанавливается под неверным owner; она
  потребует отдельного Review slice.
- Projection может быть полностью перестроена replay canonical evidence.

## Rollback

Rollback выполняется атомарно:

- revoke `communications.sender-insights.v1`;
- скрыть Sender Insights panel;
- остановить sender-fact materialization;
- оставить canonical evidence и integration operational state нетронутыми;
- rebuildable sender facts и optional display labels могут быть удалены
  отдельной owner migration без потери canonical provider evidence.
