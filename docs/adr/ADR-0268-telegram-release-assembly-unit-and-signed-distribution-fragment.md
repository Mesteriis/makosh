# ADR-0268: Telegram release assembly unit and signed distribution fragment

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Реализовано статически. Отдельный Cargo package
материализует exact descriptor/settings/storage bytes и typed artifact
fragment без signing authority; generic release compiler безопасно объединяет
fragment с полным release input и signed-manifest test проверяет runtime,
storage и две bound native dependency entries. Runtime теперь fail-closed
получает exact TDJson и tgcalls artifacts из Kernel-staged configuration.
Disposable managed conformance подтверждает этот binding вместе с Vault,
Storage, NATS, exact V6 storage bundle, history query/replay, TDLib
ready/signaling, pinned media lifecycle, restart и stale fence; test-only
tgcalls ABI fixture не является evidence real audio. Job Platform backfill
закрыт owner-local executor и managed conformance. Exact Calls Command route
включён отдельной capability и managed conformance подтверждает
initiate/end/provider reconciliation без generic REST fallback. Real media и
authorized live-call conformance остаются prerequisite для
`telegram_call_media_v1` и Calls umbrella.

Зависит от:

- ADR-0212: crate topology and compile isolation;
- ADR-0213: code ownership, SRP and module autonomy;
- ADR-0219: managed distribution integrity;
- ADR-0221: ModuleDescriptorV1;
- ADR-0224: Storage Control and migration lifecycle;
- ADR-0266: Telegram Kernel admission and event-only Communications handoff;
- ADR-0267: Kernel-staged runtime artifacts and integration state roots.

## Контекст

Telegram runtime, descriptor/settings builders, owner-local storage bundle,
TDLib и tgcalls native dependencies меняются по разным причинам. Release composition
должна собрать их в один exact artifact set, но не должна:

- превращать integration runtime в build/release tool;
- переносить integration artifacts в Communications domain package;
- давать Kernel или release compiler знание Telegram business/provider
  semantics;
- передавать signing key integration code;
- считать наличие descriptor или manifest entry выданным grant.

Ручное копирование Protobuf bytes или повторное кодирование descriptor в
JavaScript создаёт второй источник истины. Runtime-флаг `--export-contracts`
смешивает managed process lifecycle с release assembly и нарушает SRP.

## Решение

Вводится отдельная Cargo assembly unit:

```text
package = makosh-telegram-assembly
role    = integration
owner   = telegram
surface = assembly
binary  = makosh-telegram-assembly
```

Это build-time tool, а не managed runtime. Он находится вне
`first_owner_v1`, не входит в Communications inventory и никогда не
запускается Kernel.

Assembly unit имеет только односторонние зависимости на Telegram-owned
canonical artifact builders:

```text
telegram-assembly → telegram-runtime library
telegram-assembly → telegram-persistence library

telegram-runtime ✕ telegram-assembly
communications-* ✕ telegram-assembly
kernel ✕ telegram-assembly
```

Для exact `build_id` и явных absolute source paths tool создаёт новый
не существовавший output directory и материализует:

```text
telegram.runtime.descriptor.pb
telegram.runtime.settings.pb
telegram.storage.bundle.pb
telegram.release-artifacts.json
```

Первые три файла являются exact Protobuf bytes из canonical Rust builders.
JSON является только release compiler artifact fragment и содержит ровно четыре
отсортированные assembly entries:

```text
telegram.runtime.v1  -> module_runtime
telegram.storage.v1  -> storage_bundle
telegram.tdjson.v1   -> module_runtime_native_dependency
                        bound_module_id = makosh-telegram-runtime
telegram.tgcalls.v1  -> module_runtime_native_dependency
                        bound_module_id = makosh-telegram-runtime
```

Fragment не содержит digest, signature, grant, runtime generation, credential,
provider session или Communications payload. Generic distribution compiler
читает source paths, повторно проверяет regular non-symlink files, вычисляет
size/digests и подписывает только полный distribution manifest после
owner-controlled composition с platform artifacts.

Assembly tool:

- не читает signing key и не создаёт подпись;
- не выдаёт capability grants;
- не выбирает установленные TDLib/tgcalls по convention или environment;
- не создаёт Storage binding;
- не импортируется runtime, Kernel, Gateway или Communications;
- не перезаписывает существующие output files/directories.

## Единицы сборки и SRP

Разделение определяется причиной изменения:

- Telegram runtime меняется при provider execution/lifecycle;
- descriptor/settings меняются при module admission contract;
- storage bundle меняется при owner-local schema evolution;
- assembly unit меняется при составе Telegram release artifacts;
- generic distribution compiler меняется при signing/materialization format;
- Communications меняется только при provider-neutral evidence semantics.

Один release включает несколько units, но это не объединяет их ownership или
runtime lifecycle.

## Проверяемый gate

До изменения состояния реализации нужны:

1. package topology/compile-isolation guard для exact assembly unit;
2. deterministic byte equality для одинакового `build_id`;
3. independent validation descriptor, settings schema и storage bundle;
4. fail-closed output path, symlink, duplicate/existing output и missing native
   source tests;
5. exact sorted artifact fragment without secrets or private content;
6. generic release compiler test, который включает fragment в полный
   distribution input и проверяет signed runtime, storage и обе native
   dependency entries;
7. подтверждение, что Kernel/Gateway/Communications packages не зависят от
   assembly unit.

Даже после этих checks `telegram_integration_v1` остаётся закрыт до
managed-launch, revoke/generation, native-loader и live event-flow conformance
из ADR-0266/0267. Наличие tgcalls entry и успешная загрузка ABI сами по себе не
открывают Calls command capability; capability открыта только после отдельного
signaling conformance из ADR-0284 и не является evidence real media.

## Отклонённые варианты

### Экспортировать artifacts через runtime CLI

Отклонено: build tooling и managed provider lifecycle получают разные причины
изменения и authority.

### Кодировать Telegram descriptor в JavaScript release script

Отклонено: появляется второй schema source и возможен подписанный drift между
runtime-advertised и release-bound descriptor.

### Положить artifacts в Communications package

Отклонено: Communications является provider-neutral business domain и не
собирает integration executable, provider schema или native SDK.

### Подписывать fragment внутри integration tool

Отклонено: integration assembly не владеет release trust root или signing key.
Подписывается полный distribution manifest generic release compiler.

## Последствия

Telegram получает отдельную, проверяемую release assembly unit без смешения
domain, integration runtime и release authority. Цена решения — ещё один
package и явный composition step. Эта цена предпочтительнее скрытой сборки,
ручного descriptor drift или выдачи signing authority provider runtime.
