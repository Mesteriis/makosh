# ADR-0403: Mail source-contact и Review promotion граница для Persons

- Статус: принято
- Дата: 2026-08-09
- Состояние реализации: решение принято; новые workflow и Review packages ещё
  не реализованы и не admitted
- Связанные решения: ADR-0200, ADR-0204, ADR-0207, ADR-0208, ADR-0379,
  ADR-0381, ADR-0382, ADR-0383, ADR-0384, ADR-0402

## Контекст

ADR-0402 назначает `persons` каноническим владельцем человека, но существующий
admitted `mail_contacts_sync` использует provider-facing Mail events и Contacts
commands. `MailAddressBookEntryObservedV1` содержит raw provider record ID и
ETag, а page messages содержат opaque cursor. Эти поля нужны Mail integration,
но не могут попасть в Persons, Review или cross-owner workflow.

Persons уже выпускает `PersonReviewCandidateRaisedEventV1` при совпадении
нормализованного email или phone. Это только evidence: event не является
Review-owned queue, не представляет решение владельца и не разрешает attach,
merge или split.

## Решение

### Mail public-source contract

`mail` остаётся владельцем raw address-book records, provider IDs, ETags,
locators, cursor state, credentials, sessions и provider payloads. В
`makosh-mail-address-book-contract` добавляется отдельная additive schema family
`makosh.mail.address_book.person_source.v1`.

Новый contract передаёт только:

- logical owner и public integration/account/source IDs;
- bounded normalized display/email/phone claims;
- monotonic source revision, source digest и observed/removed timestamp;
- public run/page correlation и bounded completion/rejection status.

Contract не содержит provider record ID, ETag, cursor, credential reference,
private locator, raw response или provider error detail. Fetch command не
передаёт cursor: Mail runtime сам продолжает owner-local cursor state.

### Mail-to-Person workflow owner

Вводится workflow owner `mail_persons_sync` с exact package family
`makosh-mail-persons-sync-{api,core,persistence,runtime,assembly}`. Он потребляет
только новый public-source contract и публикует только typed Persons
observe/update/remove commands. Он владеет run/page orchestration, scheduler
receipt, inbox/outbox, retry и replay, но не хранит raw Mail rows и не изменяет
Person state напрямую.

Повтор страницы или source event идемпотентен. Changed bytes под существующим
message/observation ID конфликтуют. Provider removal публикует Persons source
tombstone; удаление или архивирование Person остаётся решением Persons core.

### Review-owned person-match queue

`review` получает отдельную package family
`makosh-review-person-match-candidate-{api,core,persistence,runtime,assembly}` и
contract `makosh-review-person-match-candidate-promotion-api`. Review хранит
candidate, expected revisions, action digest, owner decision и promotion status.
Он не хранит normalized email/phone values, provider locator или profile copy.

Review approval содержит ровно один typed attach/merge/split action, expected
Person/source revisions и decision provenance. Reject не создаёт promotion.
Terminal decision immutable и exact replay-safe.

### Confirmed Persons promotion workflow

Вводится workflow owner `reviewed_person_match_candidate_promotion` с package
family `makosh-reviewed-person-match-candidate-promotion-{core,persistence,runtime,assembly}`.
Workflow потребляет только Review-approved event, пересчитывает exact Persons
action digest, публикует ровно одну существующую `PersonsCommandV1`, принимает
её exact terminal result и возвращает bounded promotion result в Review.

Workflow не принимает клиентское решение, не изменяет Review или Persons schema
и не выполняет SQL join/FK между owners.

### Admission boundary

Task 5 добавляет packages в exact production package inventory как
implemented-not-admitted, но не добавляет новые workflow owners в
`ownerInventory`, capabilities, routes, `currentSlice`, reconstruction inventory
или signed release. Client/Gateway routes не активируются.

Существующий admitted `mail_contacts_sync` остаётся единственным live path до
Task 6. Новые и старые workflows не запускаются одновременно для одного source,
не выполняют dual-write и не читают storage друг друга. Task 6 единственным
атомарным gate удаляет Contacts/старый workflow и admits Persons successor.

## Отклонённые варианты

### Потреблять текущий Mail address-book event напрямую

Отклонено: он раскрывает provider ID/ETag/cursor и связывает новый workflow с
provider DTO.

### Выполнять silent merge внутри Persons или Mail sync

Отклонено: совпадение является evidence, а не решением. Изменение Person требует
явного Review approval и typed confirmed command.

### Хранить Review decision внутри Mail workflow

Отклонено: pending decision и audit принадлежат `review`, а Mail workflow владеет
только sync orchestration.

### Включить новый workflow рядом со старым до cutover

Отклонено: это создаёт production dual-write и два источника Person/Contacts
truth.

## Последствия и проверка

- новые owners имеют owner-local schema/runtime/assembly и взаимодействуют
  только durable typed contracts;
- privacy negatives запрещают provider-private поля в новых proto, storage,
  errors, SSE и telemetry;
- deterministic core, RLS, restart/replay, scheduler и actual-binary managed
  conformance обязательны до готовности Task 5;
- architecture inventory доказывает exact 15 packages и одновременно отсутствие
  admission/release/routes;
- atomic cutover и full `make pre-push` остаются Task 6.
