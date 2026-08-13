# ADR-0410: Memory, Consistency, Risk и Signal Hub composition boundary

Статус: Принято
Дата: 2026-08-13

## Контекст

После независимых Search, Timeline и Graph продукту нужны три более узких
производных представления: подтверждённая память из явно переданного evidence,
детектор противоречивых claims и ограниченный риск-view из typed signals.
Истина по Person, Knowledge, Task, Obligation, Decision и Relationship уже
принадлежит их canonical owners. Новый generic Context или AI read-all слой
создал бы вторую копию owner state и нарушил ADR-0207/ADR-0226.

Исторический Signal Hub contract объединял provider configuration, policies,
health, fixtures и команды в один façade. Он не соответствует clean-room
границе: Signal Hub должен быть application composition над публичными
read-only owner contracts, а не новым backend owner.

## Решение

Разблокировать ровно три независимых производных контура:

1. `memory` — rebuildable use-case projection только из exact typed
   `MemoryEvidenceObservedV1`; хранит public source identity, evidence digest,
   bounded memory kind, revision и time, но не plaintext content или owner
   object copy;
2. `consistency` — bounded engine из exact typed claims; противоречие существует
   только между двумя активными claims одного subject/predicate с различными
   opaque value digests. Engine не выбирает истину и не мутирует owner;
3. `risk` — bounded engine из exact typed risk signals с закрытым reason code,
   severity и expiry. Итог является производным максимумом активных signals,
   а не canonical score или автоматическим решением.

Каждый контур имеет отдельные API/core/persistence/runtime/assembly, Storage
namespace, read-only ClientRpc, один exact durable input и один rebuild
generation. Между ними нет SQL/FK и production dependency. Все входы
canonical protobuf, exact-byte idempotent и authority-bound; unknown/private
bytes отклоняются до durable write.

## Storage, rebuild и privacy

Все таблицы используют ENABLE+FORCE RLS и transaction-local owner context.
Exact replay возвращает сохранённый результат, изменённые bytes под тем же ID
конфликтуют. Rebuild заполняет shadow generation и переключает active
generation одним CAS; незавершённая generation не видна client. Tombstone или
expiry удаляет производное состояние и restart не воскрешает его.

Durable state не содержит raw provider payload, credentials, private locator,
arbitrary JSON, plaintext evidence/value/signal text или generic context pack.
Ошибки, client responses и supervised diagnostics проходят sentinel negatives.

## Signal Hub

Signal Hub остаётся только frontend/app composition над read-only Search,
Timeline, Graph, Memory, Consistency и Risk clients, а также отдельно admitted
Kernel/Telemetry/integration panels. Он не имеет backend service, storage,
runtime, policy, connection or provider command contract. Старый generated
generic SignalHub client удаляется и не является compatibility surface.

## Admission

Task 25 добавляет ровно пятнадцать production-role packages и три managed
modules. Каждый контур добавляет client, typed input consumer и storage
capability. Admission требует NOBYPASSRLS, rebuild/replay/tombstone or expiry,
restart, release compiler, generated clients, actual managed execution и
отсутствие generic Context/Signal Hub backend surfaces.

Zoom, Telemost и OmniRoute остаются вне этого решения до Task 26.

## Отклонено

- один Memory/Context warehouse для всех owner objects;
- AI embedding/vector index или inference как implicit memory;
- consistency engine, который выбирает canonical truth;
- risk score, автоматически выполняющий owner command;
- Signal Hub backend façade, provider settings owner или command router;
- direct SQL чтение таблиц canonical owners.
