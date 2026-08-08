# ADR-0308: Mail message identity, IMAP mailbox roles and location authority

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: реализовано полностью для
  `mail_provider_location_identity_v1` и отдельного
  `mail_message_location_command_v1`. Permanent delete остаётся отдельным gate
  и реализован независимо по ADR-0312.
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0220,
  ADR-0222, ADR-0223, ADR-0278, ADR-0282, ADR-0298, ADR-0299, ADR-0307,
  ADR-0312

## Контекст

Mail operational read и bounded flag mutations реализованы, но текущий IMAP
контур материализует только `INBOX` и использует UID как
`provider_message_id`. Это недостаточно для archive/trash/restore/move:

- IMAP UID уникален только внутри exact mailbox и его `UIDVALIDITY` epoch;
- `MOVE` может выдать новый destination UID;
- stale UID после mailbox recreation не должен мутировать другое письмо;
- `\Archive` и `\Trash` являются provider-discovered special-use roles, а не
  hardcoded именами;
- Gmail message ID стабилен, а location является набором system/user labels;
- повторная materialization после IMAP move не должна создавать второе
  Communications evidence для того же сообщения.

Нельзя исправить это добавлением mailbox name в client command. Client не
является authority текущего provider locator, а mailbox name может содержать
private owner vocabulary. Kernel/Core Gateway также не должны декодировать
IMAP/Gmail semantics.

## Решение

### Stable Mail message identity

Публичный Mail operational contract использует `message_id` как стабильную
Mail-owned identity внутри `connection_id`.

```text
MailMessageRefV1
  connection_id
  message_id
```

`message_id`:

- opaque для клиента;
- не содержит mailbox name, UIDVALIDITY, UID, Gmail label или secret;
- не меняется при archive/trash/restore/move;
- является ключом Mail projection, flag/location journals и client selection;
- не является Communications identity.

Для Gmail initial `message_id` может быть exact Gmail message ID, потому что
provider гарантирует его стабильность в mailbox. Для IMAP initial identity
получается из bounded digest first observed locator. Текущий locator остаётся
private Mail state.

Старое имя поля `provider_message_id` в clean-room v1 contract заменяется на
`message_id` с сохранением wire field number. Это не compatibility facade:
новые generated sources, Rust types и frontend используют только правильную
семантику. Старое имя не остаётся public alias.

Storage migration остаётся additive по platform policy. Поэтому существующее
физическое поле `provider_message_id` не переименовывается destructive DDL:
V13 добавляет generated `message_id` как owner-private compatibility column,
private locator table и constraints, а V14 отдельно создаёт stable indexes
после materialization generated columns. Persistence пишет исходное физическое
поле и читает stable alias. Этот storage seam не экспортируется в API и может
быть схлопнут только отдельной offline owner-data migration.

### Private provider locator

Mail persistence атомарно хранит IMAP locator:

```text
connection_id
message_id
mailbox_id
uid_validity
uid
observed_at
```

Инварианты:

- locator unique внутри connection;
- `uid_validity` и `uid` строго положительны;
- mailbox bounded и не содержит control characters;
- locator связан только с существующей Mail operational message;
- client query/status/error не возвращает locator;
- Gmail access token, IMAP password и provider response не входят в locator.

Sync сначала разрешает current locator в existing `message_id`. Если locator
новый, Mail создаёт initial opaque identity. Provider move с доказанным
destination locator атомарно переносит mapping к тому же `message_id`.
Communications observation anchor остаётся прежним.

### IMAP mailbox discovery

IMAP adapter выполняет bounded `LIST "" "*"` и сохраняет только selectable
mailboxes. Special-use attributes преобразуются в typed Mail roles:

```text
inbox
archive
trash
sent
drafts
spam
all
provider_folder
```

`INBOX` определяется case-insensitive canonical name. `\Archive`, `\Trash`,
`\Sent`, `\Drafts`, `\Junk` и `\All` берутся только из provider response.
Folder display name остаётся provider-owned operational data и может
показываться только в Mail surface.

Archive/trash auto-target разрешён, только если найден ровно один selectable
mailbox соответствующей role. При нуле или ambiguity команда fail closed.
Explicit move принимает `target_folder_id`, но persistence проверяет, что
folder принадлежит той же connection и является selectable.

`restore` имеет exact provider-owned semantics, а не generic "вернуть куда
было":

- IMAP restore переносит письмо только в единственный selectable mailbox роли
  `inbox`;
- Gmail restore использует exact `messages.untrash`;
- отсутствие или ambiguity IMAP Inbox завершают операцию как `unsupported`;
- runtime не угадывает previous folder, не использует hardcoded mailbox name и
  не выполняет fallback copy/delete.

Для Gmail explicit move допускает только сохранённый folder роли `inbox` или
`provider_folder`: target label добавляется, а `INBOX` снимается для
`provider_folder`. Остальные system roles имеют отдельные exact commands.
После Gmail mutation runtime повторно читает exact provider message labels и
только затем атомарно reconciles Mail projection.

### UIDVALIDITY fence

Любая IMAP mutation:

1. получает current locator из Mail persistence;
2. выбирает exact locator mailbox;
3. сравнивает provider `UIDVALIDITY`;
4. только после совпадения выполняет UID command.

Missing locator, missing UIDVALIDITY, stale epoch, non-selectable mailbox или
ambiguous role являются definite rejection до provider mutation.

Flag command ADR-0307 переводится на этот locator boundary. Парсинг client
`message_id` как UID и hardcoded `SELECT INBOX` удаляются.

### Location and permanent-delete capabilities

Безопасные location mutations и необратимое удаление имеют разные reasons to
change, grants и confirmation semantics:

```text
mail.message-location.command.v1
  archive | trash | restore | move

mail.message-location.query.v1
  sanitized durable status

mail.message-permanent-delete.command.v1
  explicit permanent delete only

mail.message-permanent-delete.query.v1
  sanitized durable status
```

`mail_message_location_command_v1` не включает permanent delete.
`mail_message_permanent_delete_command_v1` является отдельным gate.
Closure `mail_operational_command_v1` зависит от обоих.

Gmail archive/move использует typed label mutation, trash/restore — exact
`messages.trash`/`messages.untrash`. Permanent Gmail delete требует отдельного
owner-approved reauthorization с broad `https://mail.google.com/` scope;
существующий `gmail.modify` не повышается автоматически.

IMAP archive/trash/restore/move использует `UID MOVE` только после
UIDVALIDITY fence. Успех должен содержать exact `COPYUID`, чтобы Mail получил
destination UIDVALIDITY/UID и атомарно reconciled locator. Server без
`MOVE`/`UIDPLUS` получает explicit unsupported outcome, а не unsafe
copy-delete fallback.

Permanent IMAP delete использует `\Deleted` плюс `UID EXPUNGE` только при
`UIDPLUS`. Обычный `EXPUNGE`, способный удалить чужие pending messages,
запрещён.

### Ownership и build units

```text
makosh-mail-api
  public message identity and exact command/query contracts

makosh-mail-imap
  LIST/special-use, UIDVALIDITY, MOVE/COPYUID and UID EXPUNGE protocol

makosh-mail-gmail
  exact Gmail label/trash/untrash/delete HTTP adapter

makosh-mail-persistence
  stable identity, private locators, folders and durable journals

makosh-mail-runtime
  current-fence orchestration and provider result classification

frontend Mail integration
  generated client, one-use-case controllers and presentation
```

Communications не импортирует Mail и не знает provider location. Mail не
вызывает Communications query/store. Kernel/Core Gateway переносят exact
opaque bytes и проверяют capability/grant/runtime fences.

## Gate `mail_provider_location_identity_v1`

Prerequisite gate становится `implemented` только атомарно при наличии:

1. public `message_id` cutover без production alias старого field name;
2. private owner-local IMAP locator table;
3. bounded selectable mailbox discovery и typed special-use roles;
4. exact UIDVALIDITY captured during sync;
5. locator-aware flag mutation without hardcoded `INBOX`;
6. atomic materialization of message, locator, folders, Communications outbox
   and sync checkpoint;
7. restart-safe locator restore and duplicate-locator rejection;
8. live managed positive and stale-UIDVALIDITY negative conformance;
9. architecture/SRP/Cargo guards preserving owner and build-unit boundaries.

Gate реализован атомарно: public Rust/Protobuf/frontend используют
`message_id`; Mail storage bundle V13/V14 хранит private locator и stable
indexes; managed Docker conformance подтверждает mailbox roles, opaque identity,
provider mutation exactly once, restart-safe locator restore и отказ при
изменившемся UIDVALIDITY до `UID STORE`.

## Gate `mail_message_location_command_v1`

Gate становится `implemented` только атомарно при наличии:

1. независимых exact command/query contracts и capability grants;
2. owner-local V15 journal с exact canonical command bytes/hash;
3. archive/trash/restore/move без permanent delete;
4. IMAP `UID MOVE` только после `UIDVALIDITY` fence и только при
   `MOVE` + `UIDPLUS` с exact `COPYUID`;
5. Gmail exact trash/untrash/label mutation и provider-label reread;
6. atomic provider locator, folder membership, counts, projection revision и
   terminal status reconciliation;
7. exact replay без повторной provider mutation и restart-safe continuation;
8. managed positive, stale locator и unsupported capability conformance;
9. generated frontend command/query clients, SRP controller и Mail-only
   presentation actions;
10. architecture/Cargo guards, сохраняющих integration, Communications,
    Kernel, Gateway и permanent-delete boundaries.

Gate реализован атомарно: Mail storage bundle V15 хранит durable journal;
IMAP adapter требует `MOVE`/`UIDPLUS` и exact `COPYUID`; Gmail adapter
перечитывает exact labels после mutation; managed Docker conformance
подтверждает replay, restart, stale UIDVALIDITY и unsupported server behavior.
Frontend показывает reversible actions только при наличии обоих exact grants.

## Последствия

Location commands получают реальную provider identity foundation вместо
угадывания mailbox/UID. Mail client работает со stable identity, provider
locators остаются private, а Communications evidence не дублируется из-за
перемещения. Permanent delete не получает authority вместе с обычным move.
