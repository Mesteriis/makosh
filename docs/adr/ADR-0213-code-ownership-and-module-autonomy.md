# ADR-0213: Конституция кода, ownership и автономность модулей

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Частично реализовано; Cargo ownership, dependency,
storage и test-layout guards существуют, code-shape и lifecycle evidence ещё
не реализованы

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).

Этот ADR определяет, как писать новый backend-код Макошь внутри уже принятых
module и Cargo boundaries. Он применяется к Kernel, platform, domains,
integrations, workflows, engines, services и test support. Он не разблокирует
запрещённые ADR-0208 domains или projections.

## Контекст

Физически раздельные packages ещё не гарантируют модульную систему. Domain
может формально находиться в собственной crate, но скрыто зависеть от другого
owner через глобальный container, чужую модель, shared SQL, общий event bus или
god trait. Integration может иметь отдельный runtime, но требовать
Communications implementation для каждого provider action. В обоих случаях
изоляция существует только в дереве каталогов.

Макошь нужен единый engineering contract, который одновременно сохраняет:

- ownership и независимые причины изменения;
- самостоятельную сборку, тестирование и lifecycle модулей;
- failure isolation;
- простоту локального кода;
- явные side effects и зависимости;
- отсутствие speculative abstractions и generic dumping grounds;
- возможность доказать соблюдение правил tests и executable policy.

SOLID, SRP, KISS, DRY и YAGNI используются как практические ограничения, а не
как требование создавать максимальное число traits, DTO, factories и layers.

## Основная формула ownership

Каждая human-authored единица кода должна удовлетворять формуле:

```text
один owner + одна ответственность + одна причина изменения
```

Owner отвечает на вопрос «чьё это решение». Ответственность отвечает на вопрос
«какую одну роль выполняет код». Причина изменения отвечает на вопрос «какое
будущее требование заставит редактировать эту единицу».

Если ответы называют несколько независимых owners или причин, единицу нужно
разделить по responsibility, lifecycle, side-effect, volatility, dependency или
public contract boundary. Разделение по произвольному числу строк запрещено как
замена анализа ownership.

## Приоритет принципов

При конфликте решений используется порядок:

```text
correctness и безопасность данных
        ↓
ownership и failure isolation
        ↓
ясность и явный control flow
        ↓
простота
        ↓
reuse и локальная оптимизация
```

DRY не может оправдывать cross-owner coupling. SOLID не может оправдывать
абстракцию без реальной вариативности. KISS не может оправдывать смешение
business rules, SQL и transport в одном handler.

## Определение самостоятельного модуля

Domain, integration, workflow или другой independently managed module считается
самостоятельным, если он:

1. собирается без Kernel implementation и packages других owners, кроме явно
   разрешённых public contracts;
2. тестируется без production composition root;
3. запускается в harness с test implementations platform capabilities;
4. имеет собственные startup, readiness, health, drain и shutdown semantics;
5. может быть остановлен, перезапущен или удалён без остановки соседних modules;
6. владеет своими contracts, state, schema namespace, PostgreSQL role и
   operational telemetry identity;
7. получает внешние capabilities явно, а не через global state или service
   locator;
8. не интерпретирует models, tables или private payload другого owner;
9. ограничивает queues, concurrency, retries и resource consumption;
10. сообщает `blocked` или `degraded` явно и не включает silent fallback.

Самостоятельность не означает наличие отдельной копии PostgreSQL, NATS, vault
или blob storage. Это platform capabilities. Stateful module без обязательной
capability может быть функционально недоступен, но процесс обязан оставаться
управляемым и не вызывать restart storm соседей:

| Отказ | Обязательное поведение |
|---|---|
| PostgreSQL/PgBouncer | process live; readiness blocked; bounded reconnect |
| NATS | process live; degraded; durable outbox не теряется |
| Vault/credential lease | затронутый account blocked; secrets не кешируются обходным путём |
| External provider | только integration/account degraded |
| Другой domain | собственные capabilities продолжают работу |
| Workflow | базовые capabilities участвующих owners продолжают работу |

## Разрешённые зависимости по owner

| Owner | Разрешённые compile-time dependencies |
|---|---|
| Domain | собственные packages и необходимые platform protocols |
| Integration | собственные packages, platform protocols и точный `makosh-communications-ingress` |
| Workflow | public contracts участников, собственные state/lifecycle packages и platform protocols |
| Engine | pure mechanism contracts; adapters только при отдельной реальной ответственности |
| Kernel | platform и gateway protocols без owner-specific module packages |
| Gateway | generic transport/session/protocol code без статической module composition |
| Persistence | types/ports своего owner, public storage/events protocols и один concrete storage adapter stack; private Storage Control packages запрещены |
| Provider adapter | integration core своего owner и конкретный provider SDK/protocol |
| Test support | test-only contracts и harness; никогда production composition |

Объявление dependency в `[workspace.dependencies]` выравнивает версию, но не
даёт права использовать её в любом package. Каждый package импортирует только
dependencies, необходимые его ответственности. Default features отключаются,
если они вносят неиспользуемый runtime, transport, TLS stack или codegen.
Optional feature разрешён только для реального поддерживаемого режима, а не для
скрытия нескольких продуктов в одной crate.

## Правила domain

Domain самостоятельно владеет своим ubiquitous language, commands, queries,
events, invariants и durable business truth.

Domain:

- не импортирует другой domain, integration, workflow или Kernel;
- не знает provider identity, provider SDK или operational contract;
- не содержит SQL client, NATS client, HTTP client, filesystem paths или
  runtime bootstrap в contracts/domain core;
- не читает чужие tables и не создаёт cross-owner foreign keys;
- может хранить typed reference на объект другого owner, но не копирует его
  модель и policy;
- не превращает raw provider data или AI output непосредственно в durable
  business truth;
- публикует собственные факты и принимает только собственные commands.

Cross-domain operation принадлежит workflow или target-domain event consumer:

```text
source owner event
        ↓
workflow
        ↓
target owner command
        ↓
target owner event
```

Domain не получает convenience dependency на чужой contract ради одного
синхронного запроса. Cross-owner composition для клиента выполняет Gateway/
frontend application surface, а business coordination — workflow.

## Правила integration

Каждая integration самостоятельно владеет:

- provider protocol и SDK adapter;
- auth/session и credential lease consumption;
- accounts, cursors, rate limits и provider command execution;
- provider-specific operational API и state;
- lifecycle, health и resource budgets;
- преобразование provider observation в neutral evidence;
- собственные outbox/spool и replay semantics.

Integration core не зависит от concrete SDK. Provider adapter зависит от core
port, а owner runtime собирает core, adapter и persistence:

```text
integration core ← provider adapter
integration core ← persistence adapter
api + core + adapters + persistence ← integration runtime
```

Стрелка означает «правый package зависит от левого».

Integration не импортирует business domain. Единственное разрешённое
compile-time пересечение — neutral evidence contract
`makosh-communications-ingress`. Если Communications runtime недоступен,
provider operational capability продолжает работать настолько, насколько это
позволяют provider и local state, а observations ждут bounded durable delivery.

Mail, Telegram, Zulip и будущие integrations используют одинаковые ownership
правила, но не обязаны реализовывать общий operational API. WhatsApp
implementation остаётся host-only hidden WebView; его bridge, evidence и
security boundaries подчиняются тем же правилам автономности и явных side
effects.

## Правила workflow, engine и platform

Workflow:

- координирует один application-level outcome;
- зависит только от public contracts;
- не читает storage участников и не содержит provider SDK;
- явно владеет saga state, compensation, timeout и idempotency, если они нужны;
- не превращается в новый domain или общий orchestration monolith.

Engine:

- реализует один pure или rebuildable mechanism;
- не объявляет canonical business truth;
- не получает persistence «на всякий случай»;
- не маскирует заблокированную product projection техническим названием.

Platform capability:

- предоставляет один technical contract;
- не содержит business types или provider behavior;
- не становится generic SQL proxy или общим mutable state;
- разделяет protocol и concrete adapter, если это уменьшает реальный fan-out.

Storage Control подчиняется тому же правилу: SQL-free orchestration не знает
owner business schema, а PostgreSQL/PgBouncer/migration детали изолированы в
exact adapters ADR-0224. Owner persistence поставляет immutable
`StorageBundleV1`; module runtime не запускает migration и не получает admin
path. Cross-owner raw SQL запрещён. Единственное техническое пересечение для
atomic outbox/inbox выполняется через admitted versioned functions с точным
`EXECUTE` grant, без raw DML на platform tables.

## SRP на каждом уровне

### Function

Function выполняет одну coherent operation. Подозрительна function, которая
одновременно валидирует wire input, принимает business-решение, выполняет I/O,
изменяет persistence, ретраит и форматирует transport response.

Decision и side effect разделяются, если их совместное выполнение скрывает
policy или затрудняет failure testing.

### Type и trait

Type представляет одну роль: entity, value object, policy, lifecycle state,
port, adapter или command handler. Trait выражает реальную substitutable
boundary, а не создаётся автоматически для каждого struct.

`Manager`, `Service`, `Processor`, `Helper` и похожие имена не объясняют
ownership. Они допустимы только в малой и однозначной области; иначе скрытая
ответственность должна быть названа прямо.

### File и Rust module

File/module содержит cohesive набор понятий одного owner и причины изменения.
Наличие нескольких closely related types допустимо. Смешение transport DTO,
domain rules, SQL, provider parsing и lifecycle в одном файле запрещено.

### Cargo package

Package предоставляет одну cohesive capability или adapter boundary. Public API
не является случайным списком helpers. Package не используется как facade,
re-export barrel или directory-shaped junk drawer.

### Runtime и bounded context

Runtime собирает только packages своего owner и platform adapters. Bounded
context владеет одним языком и model of reality. Похожие слова в двух contexts
не являются основанием разделять один Rust type.

## SOLID в Rust

SOLID интерпретируется практично:

- **Single Responsibility:** у единицы один owner и одна причина изменения;
- **Open/Closed:** новый provider или adapter добавляется owner-local package и
  `ModuleDescriptorV1` capability, а не веткой в глобальном `match provider`;
- **Liskov Substitution:** общий trait допускает замену implementation без
  изменения observable contract и подтверждается contract tests;
- **Interface Segregation:** consumer-oriented ports содержат только операции,
  необходимые одному use case family;
- **Dependency Inversion:** domain/integration core зависит от собственных ports
  и stable protocols, concrete I/O adapters зависят от core.

Inheritance и многослойные factories не являются целью. Composition является
default. Trait с одним implementation допустим только когда он уже выражает
настоящую architecture/test boundary; imaginary future consumer не считается.

## KISS, DRY и YAGNI

### KISS

- direct local call используется внутри одного module runtime;
- NATS/event не применяется для сокрытия локального control flow;
- concrete implementation предпочтительнее framework без реальной вариации;
- configuration описывает policy и deployment, но не заменяет читаемый код;
- happy path остаётся видимым, nesting и state transitions — явными.

### DRY

Устраняется duplication знаний: invariant, validation rule, protocol codec,
security policy или idempotency rule. Похожий provider-specific код может
оставаться раздельным, если owners или причины изменения различаются.

Нельзя создавать `common`, `utils`, `helpers` или generic provider framework
только ради удаления одинаковых строк. Shared technical code получает точное
имя, owner и минимум два реальных независимых consumers.

### YAGNI

Запрещены до появления подтверждённой потребности:

- speculative extension points и factories;
- generic repository для каждого use case;
- plugin store, runtime download и remote code;
- универсальный provider/domain enum;
- optional topology или feature, которую product не поддерживает;
- compatibility facade для ещё не существующих consumers.

## Transport, application, domain и adapter responsibilities

Каждый слой принимает только свой вид решений:

| Слой | Ответственность |
|---|---|
| Transport handler | decode, wire validation, identity/deadline context, вызов use case, encode |
| Application use case | одна user/application operation и orchestration ports |
| Domain policy/model | pure business invariants и state transition |
| Persistence adapter | transaction, query, row mapping и storage error translation |
| Provider adapter | external protocol, rate limit, remote error и payload mapping |
| Runtime | dependency composition, task ownership, lifecycle и resource budgets |

Transport handler не выполняет SQL или provider call напрямую. Domain model не
возвращает HTTP/ConnectRPC/NATS response. Persistence adapter не принимает
business decisions, кроме storage consistency, idempotency и transaction
semantics своего owner. Он может вызвать только каноническую versioned
technical function events owner для atomic outbox/inbox operation; это не даёт
ему доступ к platform table или чужому business state.

## Явные side effects и lifecycle

- Dependencies передаются constructor/function arguments или owner-local
  composition; global service locator запрещён.
- Hidden mutable globals, process-wide singleton state и magic task-local
  context запрещены.
- I/O видно из port/adapter name и return type.
- Detached `tokio::spawn` вне runtime task ownership запрещён.
- Каждая long-running task имеет cancellation, deadline/backoff policy и join
  ownership.
- Runtime владеет quiesce, drain, checkpoint и stop своих tasks.
- Retry bounded; ambiguous non-idempotent outcome не повторяется автоматически.
- Queue и concurrency limits обязательны; unbounded channel запрещён.
- Runtime topology не переключается автоматически после failure.

## Errors, security и observability

- Errors typed и принадлежат owner/boundary, где возникло решение.
- Adapter переводит external error в свой typed vocabulary; string не является
  cross-layer error contract.
- `unwrap`/`expect` в runtime path разрешены только для локально доказанного
  invariant с объяснимым failure; user input, I/O и external payload не
  считаются доказанными.
- Catch-and-ignore, log-and-continue без state transition и silent fallback
  запрещены.
- Logs, metrics, traces, health и errors не содержат secrets, credentials,
  cookies, message bodies, documents, media bytes или raw provider payload.
- Telemetry описывает identity, lifecycle, bounded metadata и outcome, но не
  дублирует business event или canonical audit trail.

## Naming и public surface

Имя отвечает на вопрос «что код владеет, решает или преобразует».

Предпочтительны точные роли: `Command`, `Handler`, `Policy`, `Repository`,
`Gateway`, `Client`, `Parser`, `Mapper`, `Codec`, `Runtime`, `Lease`, `Cursor`.
`Entity`, `Aggregate`, `Projection` и `Provider` используются только в их
точном domain/architecture смысле.

Wildcard public re-export, cross-owner re-export и compatibility barrel
запрещены. Public surface минимален; internal implementation не становится
`pub` только ради теста или удобства соседнего owner.

## Complexity budget

Line count, branch count и import count являются сигналами для review, но не
определяют SRP сами по себе. Основной критерий — ownership и причины изменения.

Для human-authored source действуют project thresholds:

- function больше 60 logical lines требует проверки responsibility и control
  flow; больше 100 — split или письменное обоснование;
- type больше 300 logical lines требует проверки state/lifecycle cohesion;
- file больше 700 lines требует зафиксированной причины или refactoring plan;
- file больше 1000 lines является architecture problem и не принимается без
  отдельного документированного решения;
- generated, vendor, lock и machine-produced artifacts оцениваются отдельно.

Дополнительные smells: nesting глубже трёх уровней, parameter train, boolean
mode, повторяющийся `match` по provider/status, больше одной независимой группы
public entry points, hidden I/O, temporal coupling, shotgun surgery и divergent
change.

При обнаружении smell review обязан назвать перегруженную ответственность и
предложить smallest useful split. Механическое уменьшение строк без переноса
ownership не считается исправлением.

## Testing contract

Tests находятся вне `backend/src` по ADR-0211. Production package не зависит от
test-support package; test support не зависит от production composition.

Обязательные уровни evidence:

- pure unit/property tests для invariant и state transition;
- contract tests для каждого public port/protocol и substitutable adapter;
- container-backed persistence tests для admitted bundle migrations,
  generation-scoped grants, transaction/outbox atomicity и idempotency;
- fixture/test-double provider tests без live private accounts;
- process-level lifecycle tests для startup, readiness, degraded, drain,
  restart и forced stop;
- failure tests для timeout, duplicate delivery, stale state, unavailable
  dependency и bounded retry;
- dependency-impact check, доказывающий отсутствие чужих reverse dependencies.

Tests проверяют observable behavior и invariants, а не последовательность
вызовов mocks. Большие snapshots не заменяют assertions. Реальные credentials,
private content и live provider mutations в automated tests запрещены.

## Change discipline

Любое изменение начинается с определения owner и причины изменения.

Новый abstraction/package/dependency допустим, только если автор может указать:

1. конкретную ответственность;
2. owner;
3. разрешённое направление dependency;
4. существующего consumer;
5. failure и lifecycle semantics;
6. способ targeted validation.

Изменение public contract атомарно обновляет owner implementation, generated
clients, consumers и contract tests. Compatibility facade, broad re-export и
временная cross-owner implementation dependency запрещены. Новый architecture
exception file не создаётся; если правило неверно, меняются ADR, общее правило
и negative tests.

## Executable enforcement

Уже реализовано:

- package role/owner/surface metadata;
- domain, integration, Kernel/Gateway и persistence dependency rules;
- запрет aggregate packages и compatibility layout;
- SQL clients и SQL ownership только в persistence surface;
- exact Storage package set, запрет Kernel/module dependency на private Storage
  Control implementation и запрет raw cross-owner platform DML;
- запрет production test code и test-support composition coupling;
- positive/negative architecture self-tests.

Обязательно реализовать вместе с первым production slice, где правило
становится применимо:

- запрет detached task creation вне runtime lifecycle ownership;
- проверку provider SDK только в owner adapter package;
- проверку transport/SQL dependencies по surface;
- owner-local package checks и reverse-dependency impact report;
- lifecycle harness для independently managed runtimes;
- warnings для complexity signals без автоматического line-count splitting;
- contract-test harness для platform, domain и integration ports.

Policy не использует baseline или per-file exceptions. Generated code
исключается по каноническому generated root, а не произвольным allowlist files.

## Definition of Done модуля

Module соответствует этому ADR только при одновременном выполнении условий:

1. owner, responsibility и public contract названы;
2. package-scoped check не собирает Kernel или чужого owner;
3. tests запускаются без production composition;
4. runtime запускается с test platform capabilities без Kernel;
5. startup, readiness, degraded, drain и shutdown доказаны tests;
6. отказ или restart module не завершает соседние modules;
7. storage objects и migration bundle принадлежат только этому owner, bundle
   admitted Storage Control, а runtime не имеет DDL/admin path;
8. contracts не содержат concrete SDK/storage/runtime types;
9. side effects, retries, queues и cancellation явны и bounded;
10. удаление module убирает только его capabilities и не ломает сборку других
    owners;
11. private content и secrets отсутствуют в diagnostics и fixtures;
12. targeted validation и dependency-impact evidence сохранены.

Для integration дополнительно доказывается работа provider operational
capability без Communications implementation и durable ожидание neutral
evidence при временной недоступности ingress consumer.

Для host-only integration требования backend process lifecycle заменяются
эквивалентными Tauri host/WebView lifecycle и bridge isolation tests; backend
implementation/runtime package ради формального соответствия не создаётся.

Для domain дополнительно доказывается работа собственных commands/queries при
недоступности других business owners.

## Последствия

### Положительные

- ownership виден в code shape, dependencies, tests и runtime behavior;
- один provider/domain можно изменять и перезапускать независимо;
- SOLID не превращается в speculative framework;
- failures и side effects становятся локальными и наблюдаемыми;
- code review получает проверяемые критерии вместо оценки «файл выглядит
  большим».

### Цена

- adapters, contracts и lifecycle требуют явного кода и tests;
- cross-owner features должны иметь workflow вместо shortcut import;
- общий protocol change осознанно требует обновления consumers;
- часть code-quality правил потребует новых executable linters и harness.

## Отклонённые варианты

### Только rustfmt/clippy

Отклонено: они не доказывают ownership, module autonomy, failure isolation или
отсутствие hidden I/O.

### Жёстко делить файлы по числу строк

Отклонено: уменьшает размер, но может сохранить смешанные причины изменения и
создать навигационный шум. Threshold запускает ownership review, а split
определяется responsibility.

### Максимально абстрактная Clean Architecture

Отклонено: количество layers, traits и DTO не является целью. Используется
минимальное число boundaries, необходимое для ownership, testability и failure
isolation.

### Общие managers/services/utils

Отклонено: такие containers скрывают ownership и становятся неформальным
composition root. Общая technical capability получает точное имя и contract;
owner-specific code остаётся у owner.
