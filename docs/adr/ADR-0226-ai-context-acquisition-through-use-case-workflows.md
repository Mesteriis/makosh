# ADR-0226: Контекст для AI только через use-case workflows

Статус: Принято
Дата: 2026-07-16
Состояние реализации: Governance policy и negative architecture self-tests
реализованы; AI contracts, use-case workflows, context assembly и contract
tests ещё не созданы

Уточняет:

- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0206: Конституция Kernel](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0207: Канонический реестр бизнес-доменов](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0208: Allowlist разработки и запрет проекций](ADR-0208-domain-development-allowlist-and-projection-freeze.md);
- [ADR-0213: Код, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0214: Durable Job Platform и Scheduler](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0220: Канонический durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0224: Storage Control Plane и owner-scoped PostgreSQL](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

AI use case часто требует больше контекста, чем безопасно и разумно помещать в
domain event. Например, анализ communication может учитывать связанный контакт,
организацию, задачи, календарь и документы. Из этого не следует, что AI domain
должен получить read grants ко всем таблицам или стать generic orchestrator
всех domain APIs.

Read-all PostgreSQL даёт короткий путь к joins и batch processing, но превращает
AI в superdomain:

- AI связывается с physical schema и migrations всех owners;
- domain redaction, authorization и semantic invariants обходятся SQL;
- компрометация AI или remote model расширяет privacy blast radius;
- изменение любого owner начинает ломать AI compile/runtime contract;
- AI незаметно получает право определять business meaning чужих данных.

Прямые AI → domain query RPC устраняют SQL coupling, но оставляют AI
оркестратором всех bounded contexts. Он начинает знать, какие domains
существуют, в каком порядке их спрашивать, что обязательно и как трактовать
partial failure. Эта ответственность относится к application use case, а не к
inference owner.

## Решение

### Любой cross-owner AI use case принадлежит workflow

Если операция использует AI вместе хотя бы с одним другим owner, orchestration
принадлежит explicit use-case workflow.

```text
owner event / user command
  -> use-case workflow
  -> typed queries к нужным owner contracts
  -> distinct generated use-case request
  -> AI use-case contract
  -> typed AI result + provenance
  -> workflow policy
  -> target-domain command или review candidate
```

Kernel только авторизует, маршрутизирует и supervises. Он не собирает context,
не выбирает domain sources и не интерпретирует AI payload.

AI domain:

- не читает таблицы других owners;
- не получает PostgreSQL grants на чужие таблицы;
- не вызывает query APIs других domains напрямую;
- не обнаруживает owners через generic catalog и не строит dynamic fan-out;
- не владеет решением, какой business context нужен конкретному use case.

### Workflow создаётся на use case, а не «на весь AI»

Допустимы отдельные workflows, например:

- `communication_summary`;
- `task_candidate_extraction`;
- `daily_brief`;
- `document_classification`.

Запрещён один catch-all `ai_context_workflow`, который принимает arbitrary owner
names, SQL-like filters или generic `GetEverything`. Каждый workflow имеет одну
business-причину изменения, явные input/output contracts и bounded source set.

Workflow может зависеть только от public contract packages нужных owners и AI.
Он не зависит от persistence adapters, runtimes, provider SDK или Kernel
implementation.

### Owner query contracts остаются consumer-neutral

Domain не создаёт API вида `GetDataForAI`. Он публикует typed query contract,
описывающий собственное business meaning и пригодный для любого authorized
consumer.

Workflow:

- вызывает exact owner queries через capability router;
- передаёт owner-specific DTO только внутри собственной assembly logic;
- нормализует выбранные данные в AI-owned request contract;
- не раскрывает одному domain данные другого domain;
- не пишет в owner storage.

Изменение AI prompt или модели не требует изменения domain query contract, если
business semantics запроса не изменились.

### Общий receipt и отдельный request для каждого use case

Глобального универсального context bundle и общего списка разнородных
fragments нет.
AI contract публикует небольшой общий `AiContextReceiptV1`, а каждый use case —
собственный generated request и собственное concrete context message. Например:

```protobuf
message CommunicationSummaryRequestV1 {
  AiContextReceiptV1 receipt = 1;
  CommunicationSummaryContextV1 context = 2;
}
```

`CommunicationSummaryContextV1` содержит только поля, разрешённые именно для
этого use case. Другой use case имеет другой request type и не расширяет
глобальный union.

`AiContextReceiptV1` содержит только общую provenance и consistency metadata:

- stable use-case contract ID и revision;
- exact request message full name, revision и schema SHA-256;
- workflow/run/correlation identifiers;
- `as_of` time;
- source receipts с owner contract revision, observed entity revision,
  required/optional classification и explicit missing/denied/stale reason;
- privacy/egress policy revision, byte/token/time budgets;
- deterministic digest concrete request;
- causation и provenance.

Общий receipt и use-case request не содержат:

- database/table/column names;
- SQL fragments;
- generic owner endpoints;
- credentials, provider sessions или secret references;
- global fragment union, type ID + opaque bytes, arbitrary JSON,
  Protobuf `Any` или generic maps;
- full binary documents/media;
- authority на последующее чтение owner state.

Большой или private content передаётся через expiring authorized `BlobRef` после
открытия `blob_v1`. До этого соответствующий AI use case не реализуется.
Private content не помещается в NATS. Durable trigger переносит только bounded
references; assembled request передаётся локальным typed call либо через
отдельный AI-owned durable acceptance contract, который не копирует content в
event envelope.

### Request является ephemeral, а не projection

Concrete use-case request существует только для конкретного run и не становится
долговечной cross-domain Context projection. Common receipt не является
контейнером context content.

Разрешено сохранять у AI:

- run identity и lifecycle;
- model/prompt/policy revision;
- source references и digests;
- token/cost/timing metadata без private content;
- typed result, confidence и provenance;
- sanitized failure class.

Запрещено сохранять как AI-owned context cache:

- полные копии чужих domain records;
- cross-domain dossiers;
- materialized relationship/context graph;
- reusable context packs;
- generic embeddings/indexes чужих owner data.

Это остаётся projection, заблокированной ADR-0208. Если измеренная нагрузка
потребует durable context projection, новый ADR должен явно открыть owner,
rebuild/source revision/privacy и invalidation contracts. Performance не
разрешает тихо обойти freeze.

### Consistency и partial context

Workflow не обещает одну global PostgreSQL snapshot через разные owners.
Вместо этого `AiContextReceiptV1` фиксирует semantic receipt:

- общий `as_of`;
- revision каждого source;
- время чтения;
- completeness;
- missing/stale reasons.

Нужные owner queries запускаются параллельно с bounded deadlines. Source
объявляется:

- `required` — failure/denial/expiry блокирует AI call;
- `optional` — workflow может продолжить, но receipt явно отмечает отсутствие;
- `forbidden` — source не запрашивается при текущей privacy/egress policy.

AI result обязан переносить context digest и completeness. Он не может
представлять partial context как полный.

### Privacy и remote model egress

Право workflow прочитать owner data не означает право отправить эти данные
remote AI provider.

Перед AI call workflow и AI boundary проверяют:

- effective grants и actor/use-case scope;
- privacy classification каждого concrete context field/source;
- local-only или remote-allowed egress policy;
- redaction/minimization policy;
- byte/token budget;
- model/provider policy revision.

Если policy не разрешает remote egress, remote provider call fail closed.
Secrets, credentials, provider sessions и hidden identifiers не становятся
prompt context. Logs, metrics, traces, errors и health не содержат request
content.

### AI output не является business truth

AI возвращает candidate, summary, classification, extraction или recommendation
со source provenance и confidence. Он не пишет Task, Contact, Organization,
Calendar event, Document truth или Communication state.

Продвижение результата выполняет workflow:

```text
AI result
  -> workflow validation/policy
  -> review candidate or target-domain command
  -> target domain validates and persists truth
```

Target domain может отклонить результат независимо от AI confidence.

### Failure и retry ownership

- Owner query retry принадлежит workflow policy.
- AI execution retry принадлежит AI JobKind/contract.
- NATS redelivery заканчивается после durable acceptance и не удерживает ACK на
  время inference.
- Required-source failure не маскируется пустым context field.
- Ambiguous external AI side effect не повторяется автоматически, если contract
  не доказал idempotency.
- Workflow cancellation/deadline прекращает новые owner reads и AI calls, но не
  делает silent rollback уже сохранённого owner truth.

## Cargo и storage boundaries

Целевое направление dependencies:

```text
use-case workflow
  -> owner domain contract A
  -> owner domain contract B
  -> makosh-ai-contracts
  -> platform contracts

makosh-ai-domain/runtime
  -> makosh-ai-contracts
  -> platform contracts
  -/> other domain contracts
  -/> workflow contracts
  -/> other owner persistence
```

AI persistence использует SQLx только в own persistence package и обращается
только к AI-owned tables в `makosh_data`. Read-only cross-owner SQL остаётся
cross-owner SQL и запрещён так же, как write.

## Отклонённые варианты

### AI получает `SELECT` на все business tables

Отклонено: schema coupling, обход owner semantics и максимальный privacy blast
radius.

### AI напрямую вызывает все domain query APIs

Отклонено: AI становится application orchestrator и знает topology всех
domains.

### Один generic Context API

Отклонено: скрывает use-case authority, превращается в cross-domain god service
и фактически размораживает Context projection.

### Полный context внутри event/NATS payload

Отклонено: нарушает bounded envelope/privacy rules и создаёт durable копии
private content.

### Создавать durable projection сразу ради скорости

Отклонено до измеренной причины и отдельного ADR по ADR-0208.

## Проверка решения

До изменения `Состояние реализации` обязательны:

- AI package dependency на Contacts/Tasks/Documents/Communications contract
  отклоняется Cargo guard;
- AI persistence query к таблице другого owner отклоняется SQL ownership guard;
- workflow может зависеть от explicit AI и owner contracts;
- AI contract не содержит generic owner/query/SQL fields;
- event trigger содержит refs, а не assembled private context;
- required/optional source failure отражается в receipt deterministically;
- global fragment union, opaque payload bytes, `Any` и generic map отсутствуют
  в AI contracts;
- exact request message revision и schema SHA-256 связаны с receipt и result;
- `as_of`, source revisions, completeness и digest сохраняются в result
  provenance;
- remote egress denial fail closed до provider call;
- private content отсутствует в NATS, logs, telemetry, errors и health;
- AI result не мутирует target domain без workflow/domain command;
- `makosh-context-*` production package остаётся заблокированным;
- fixture tests используют synthetic content и не вызывают live private
  accounts или remote model без explicit manual smoke.

## Последствия

Положительные:

- AI остаётся самостоятельным domain, а не superuser database layer;
- domains не знают об AI и не создают AI-specific APIs;
- privacy, completeness и egress решения видимы на уровне use case;
- отказ AI не нарушает owner state;
- изменение physical schema не является AI contract change.

Отрицательные:

- context assembly требует больше typed contracts;
- fan-out добавляет latency и partial-failure handling;
- нет неявной global database snapshot;
- каждый новый cross-owner AI use case требует отдельного workflow;
- projection-performance optimization намеренно отложена.
