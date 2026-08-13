# ADR-0408: Identity Resolution rebuildable candidate engine boundary

Статус: Принято
Дата: 2026-08-13
Состояние реализации: В разработке

Зависит от ADR-0200, ADR-0201, ADR-0202, ADR-0402 и ADR-0403.

## Контекст

Persons является единственным владельцем canonical Person, source links и
lineage. Review является единственным владельцем решения о предложенном
attach/merge/split. До этого решения Persons публиковал найденное совпадение
непосредственно в Review, поэтому между extraction evidence и review queue не
было отдельной rebuildable границы.

Identity Resolution не может получать provider payload, raw email/phone,
credential или private locator: такое расширение создало бы второй identity
owner и опасный read-all индекс. Для первого bounded slice уже существующий
Persons candidate event трактуется как санитаризированное evidence: public
Person/source IDs, stable candidate ID, match kind, owner revision и время.

## Решение

### Authority boundary

`identity_resolution` — Engine, а не Domain. Он exact-валидирует admission,
authority, canonical protobuf bytes и deterministic candidate ID входного
Persons evidence, сохраняет owner-local rebuildable observation и публикует
typed `PersonLinkMergeCandidateProposed` для Review.

Engine не имеет Persons command capability, не пишет Persons storage, не
создаёт lineage и не подтверждает merge. Review принимает proposal только от
exact Identity Resolution module authority. Approved action по-прежнему
проходит Review promotion workflow и только затем canonical Persons command.

### Determinism and replay

Stable candidate ID остаётся Persons-derived из logical owner, сортированной
пары public source identities и match kind. Proposal message ID выводится из
exact evidence event ID и candidate ID. Exact inbox replay возвращает уже
сохранённые output bytes до freshness/clock; changed bytes конфликтуют.

Новая evidence для того же candidate разрешена только при строго растущей
Persons owner revision и неубывающем observed_at. Она обновляет rebuildable
latest evidence под lock и создаёт ровно один следующий proposal. Terminal
Review state engine не читает и не копирует.

### Persistence and privacy

PostgreSQL bundle хранит только public IDs/revisions/digests, exact inbox и
sequenced outbox. Все таблицы используют ENABLE/FORCE RLS, tx-local validated
logical owner и effective NOBYPASSRLS evidence. Relay держит owner-scoped
`FOR UPDATE SKIP LOCKED` claim через broker acknowledgement.

Запрещены raw claims, email, phone, provider payload, credential, private
locator, arbitrary JSON, confidence/risk score и cross-owner foreign key.

### Product and release surface

Engine не имеет ClientRpc, Gateway route, generated frontend client или UI.
Release содержит только runtime и Storage artifacts. Development assembly
может включить engine после compiler-consumed fragment test, но production
`currentSlice` меняется только на последовательной admission границе.

## Проверка

- exact API/core deterministic/privacy mutation tests;
- FORCE-RLS bundle и owner-2 NOBYPASSRLS matrix;
- actual Persons evidence -> Identity proposal -> Review submission, replay,
  restart, relay overlap и privacy contour;
- negative proof отсутствия Persons command/storage capability;
- compiler-consumed runtime/Storage artifacts;
- exact package/capability/inventory guards и focused gates.

## Не входит

- raw email/phone index, fuzzy matching, ML scoring или confidence;
- direct attach/merge/split, canonical Person/lineage ownership;
- Review decision/promotion ownership;
- Search, Timeline, Graph, Memory, Risk или Signal Hub;
- frontend route, compatibility alias, stage, commit или push.
