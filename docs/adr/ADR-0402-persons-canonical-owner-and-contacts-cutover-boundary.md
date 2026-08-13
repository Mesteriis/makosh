# ADR-0402: Persons как канонический владелец человека и граница Contacts cutover

- Статус: принято
- Дата: 2026-08-09
- Состояние реализации: registry и development allowlist реализованы; Persons
  runtime и production admission не реализованы
- Заменяет: части ADR-0207 о домене `contacts` и части ADR-0208 о разрешении
  разработки `contacts`
- Связанные решения: ADR-0200, ADR-0204, ADR-0207, ADR-0208, ADR-0220,
  ADR-0252, ADR-0379, ADR-0381, ADR-0383, ADR-0384

## Контекст

Clean-room inventory уже содержит admitted Contacts packages и отдельный
`mail_contacts_sync` workflow. Их модель появилась до окончательного решения о
канонической идентичности человека. Reference-код подтверждает полезные
сценарии address book sync, но не является источником schema, REST surface или
runtime topology.

Центральный owner должен представлять человека, а не provider address-book
record. Один человек может иметь несколько заново подключённых источников,
ручной профиль и подтверждённые внешние идентичности. Provider contact books,
account cursors, protocol locators и provider session state имеют другой
lifecycle и остаются внутри integrations.

## Решение

### Канонический owner

`persons` заменяет `contacts` в canonical business-domain registry и
development allowlist. В registry может существовать ровно один центральный
owner человека: только `persons`, никогда одновременно `contacts` и `persons`.

Persons владеет:

- `Person` и его public identifier;
- owner-authored profile fields;
- подтверждёнными внешними идентичностями;
- source links от Person к provider source record;
- merge/split lineage и состоянием provisional/active/archived Person;
- durable owner events, не содержащими provider secrets или private locators.

Organizations остаётся отдельным owner. Persons не хранит организацию как
разновидность человека и не импортирует Organizations implementation или
storage.

### Provider-owned contact books

Mail, Telegram, WhatsApp, Zulip и будущие integrations продолжают владеть raw
provider contacts, provider payloads, account cursors, protocol identifiers,
credentials, sessions и private provider locators. Они наблюдают изменения и
вызывают typed Persons contracts через отдельный workflow; Persons не зависит
от integration implementation или storage и не выполняет provider IO.

Source link имеет identity `(integration_public_id, account_public_id,
provider_source_key)`. `provider_source_key` — стабильный account-scoped opaque
handle конкретного provider record, выпущенный integration, а не raw provider
ID или locator. Тройка уникальна, повторное наблюдение идемпотентно, а один и
тот же provider record в разных accounts не объединяет sources. Private locator
может быть разрешён только integration owner; durable Persons state и events
используют public source identity и bounded normalized facts.

### Identity decisions и производные данные

- новый неизвестный source создаёт provisional Person;
- совпадение email или phone между независимыми sources создаёт Review
  candidate и никогда не выполняет silent merge;
- `review` владеет pending link/merge/split decision, audit state и решением
  принять или отклонить proposal;
- только подтверждённая команда Persons выполняет attach/detach, merge или
  split;
- `relationships` после отдельного admission владеет подтверждёнными
  отношениями между public Person/Organization IDs, но не копирует их profile;
- dossier, timeline, trust, graph, search и signal analytics являются
  rebuildable projections/engines и не получают canonical ownership.

Merge сохраняет полный lineage исходных Persons и source links. Split создаёт
новую подтверждённую ветвь lineage и переносит только явно выбранные source
links/profile facts. Replay одних и тех же accepted commands и owner events
восстанавливает то же состояние без новых идентичностей или потери lineage.

### Removal, failure и восстановление

Удаление provider source отсоединяет source link, но не удаляет Person, пока у
него есть другой source или owner-authored data. Person без sources и ручных
данных архивируется по owner policy; hard delete не следует из provider
deletion.

Provider outage, revoke или недоступный cursor не повреждает Persons state.
Workflow повторяет observation идемпотентно после восстановления provider.
Никакой partial provider response не подтверждает merge, split или deletion.
Credentials, sessions, historical database и Contacts schema не мигрируются:
accounts авторизуются заново, а доступная provider history повторно строит
Persons через тот же typed observation contour.

### Privacy boundary

Persons contracts, events, errors, telemetry и client responses не содержат:

- credentials, refresh tokens, cookies или session databases;
- raw provider payloads;
- private provider locators и account secrets;
- неограниченное provider content.

Provider-specific display data передаётся только как bounded normalized facts
с provenance. Sensitive profile reads остаются owner-authorized и
capability-gated. Review candidate раскрывает только факты, необходимые для
явного решения.

### Атомарный cutover

Это решение не переименовывает существующие Contacts packages и не меняет их
production owner inventory, capabilities, routes, schema или release fragments.
До отдельного atomic cutover Cargo guard допускает только exact Contacts
packages, уже перечисленные в `implementation.productionPackages`, с точным
role/owner/surface. Любой новый Contacts package, alias или попытка использовать
`contacts` как development owner отклоняется.

Cutover выполняется отдельным admission gate после появления Persons
contract/core/persistence/runtime/assembly, Mail-to-Person workflow, Review
promotion и generated client. В одном срезе удаляются Contacts packages,
`mail_contacts_sync` successor получает Persons contracts, а exact production
inventory переходит на `persons`. Aliases, dual-read, legacy schema imports и
сохранение `/api/v1/personas*` не допускаются.

## Отклонённые варианты

### Переименовать provider contacts в Person внутри integration

Отклонено: integration lifecycle, revoke и cursor state не могут владеть
канонической идентичностью человека.

### Автоматически объединять совпавшие email или phone

Отклонено: shared addresses, recycled phone numbers и provider normalization
создают необратимые ошибочные merges. Совпадение является только Review
evidence.

### Сохранить Contacts alias рядом с Persons

Отклонено: два центральных имени создают dual ownership, compatibility schema и
неопределённый contract boundary.

### Импортировать legacy Contacts database

Отклонено: clean room начинает с новой owner schema. Повторная provider
авторизация и sync являются единственным источником provider history; ручной
Person создаётся через новый owner contract.

## Последствия

Положительные:

- canonical identity отделена от address-book protocol state;
- merge/split становятся reviewable и replay-safe;
- новый provider подключается без изменения Persons storage model;
- provider revoke и resync не требуют legacy migration или dual-read;
- executable registry не допускает двух центральных owners.

Отрицательные:

- до atomic cutover registry и admitted package inventory намеренно различаются;
- старые Contacts packages остаются production-admitted только как закрытый
  exact inventory, но не разрешены как направление новой разработки;
- provider history, недоступная после новой авторизации, не восстанавливается и
  не обещается пользователю.

## Проверка решения

- policy schema требует `persons` и отвергает совместный `contacts` owner;
- source/Cargo fixtures принимают Persons development packages;
- Cargo guard принимает только exact pre-cutover Contacts inventory и
  отвергает undeclared Contacts packages;
- dependency fixtures запрещают Persons зависимости на integration
  implementation и persistence;
- `architecture-policy-check` проходит с неизменным exact production inventory;
- `cargo-boundaries-check` подтверждает текущие 316 workspace packages до
  отдельного cutover.
