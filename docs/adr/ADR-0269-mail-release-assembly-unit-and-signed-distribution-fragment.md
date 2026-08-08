# ADR-0269: Mail release assembly unit and signed distribution fragment

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Реализовано статически. Отдельный Cargo package
материализует exact descriptor/settings/storage bytes и typed unsigned
fragment; generic distribution compiler test запускает assembly binary и
проверяет signed runtime и Storage entries. Compile-isolation guards запрещают
обратную зависимость Mail runtime и зависимость Communications на assembly
unit. Mail production registration, grants и managed launch остаются закрыты
до отдельного executable и live conformance.

Зависит от:

- ADR-0212: crate topology and compile isolation;
- ADR-0213: code ownership, SRP and module autonomy;
- ADR-0219: managed distribution integrity;
- ADR-0221: `ModuleDescriptorV1`;
- ADR-0224: Storage Control and migration lifecycle;
- ADR-0261: Communications attachment-anchor handoff;
- ADR-0262: Mail attachment Blob-admission extension;
- ADR-0263: Mail settings and Storage admission artifacts.

## Контекст

Mail runtime, descriptor/settings builders и owner-local Storage bundle уже
существуют как отдельные Mail packages. Но Kernel может запускать managed Mail
только из signed distribution manifest, который связывает exact executable,
descriptor, settings schema и Storage bundle. Ручная сборка этих записей в
тесте или release script создаст второй источник истины и позволит подписать
bytes, отличающиеся от canonical Rust builders.

Runtime не должен экспортировать release artifacts через operational CLI:
managed provider lifecycle и build composition имеют разные причины изменения.
Communications также не может собирать Mail release — это превратило бы domain
owner в integration facade.

## Решение

Вводится отдельная Cargo assembly unit:

```text
package = makosh-mail-assembly
role    = integration
owner   = mail
surface = assembly
binary  = makosh-mail-assembly
```

Это build-time tool, а не managed runtime. Он не входит в
`first_owner_v1`, не запускается Kernel и не выдаёт capability grants.

Assembly unit имеет только односторонние зависимости на Mail-owned canonical
artifact builders:

```text
mail-assembly -> mail-runtime library
mail-assembly -> mail-persistence library

mail-runtime      x mail-assembly
communications-*  x mail-assembly
kernel            x mail-assembly
gateway           x mail-assembly
```

Для exact `build_id`, absolute Mail runtime source path и нового absolute
output directory tool материализует:

```text
mail.runtime.descriptor.pb
mail.runtime.settings.pb
mail.storage.bundle.pb
mail.release-artifacts.json
```

Первые три файла являются exact Protobuf bytes из canonical Mail builders.
JSON является только unsigned input fragment для generic distribution compiler
и содержит ровно две отсортированные записи:

```text
mail.runtime.v1  -> module_runtime
mail.storage.v1  -> storage_bundle
```

Mail не имеет native runtime dependency, поэтому assembly fragment не создаёт
пустую или фиктивную `module_runtime_native_dependency` запись. Если такая
зависимость появится, она потребует отдельного решения и exact binding.

Generic distribution compiler повторно открывает regular non-symlink source
files, вычисляет size/digests и подписывает только полный owner-controlled
release manifest. Mail assembly:

- не читает signing key и не создаёт подпись;
- не выдаёт grants и не создаёт Storage binding;
- не выбирает runtime executable по environment convention;
- не содержит provider credentials, settings values, session state или
  Communications payload;
- не перезаписывает существующий output directory или artifact;
- не импортируется runtime, Kernel, Gateway или Communications.

## Единицы сборки и SRP

Границы определяются ответственностью:

- Mail runtime владеет provider execution и operational lifecycle;
- Mail descriptor/settings владеют admission contract;
- Mail persistence владеет owner-local schema и Storage bundle;
- Mail assembly владеет только составом Mail release artifacts;
- generic distribution compiler владеет digest/signature полного release;
- Kernel владеет admission, exact launch identity и capability fencing;
- Communications владеет только provider-neutral evidence semantics.

Один signed release объединяет несколько assembly units, но не объединяет их
ownership, runtime state, grants или storage.

## Проверяемый gate

До изменения состояния реализации обязательны:

1. exact package metadata и compile-isolation allowlist;
2. deterministic equality descriptor/settings/storage bytes canonical
   builders;
3. independent validation всех трёх Protobuf artifacts;
4. fail-closed absolute output/runtime paths, symlink, missing/empty runtime,
   duplicate и existing-output cases;
5. exact sorted fragment без secrets, private content, digest или signature;
6. generic distribution compiler test, доказывающий signed runtime и Storage
   entries из Mail fragment;
7. доказательство отсутствия зависимости Kernel/Gateway/Communications и Mail
   runtime на assembly unit.

После этого Mail production admission всё ещё требует exact owner approval,
GrantSet, Storage/Vault/Blob/Event Hub fences, revoke/stale-generation checks и
live event-only attachment flow из ADR-0261/0262.

## Отклонённые варианты

### Копировать descriptor в release JavaScript

Отклонено: появляется второй schema source и подписанный drift.

### Экспортировать contracts через Mail runtime CLI

Отклонено: provider runtime получает build-tool responsibility.

### Собирать Mail artifacts в Communications или Kernel

Отклонено: domain/core начинают зависеть от integration implementation.

### Подписывать fragment внутри Mail assembly

Отклонено: integration assembly не владеет release trust root.

## Последствия

Mail получает собственную проверяемую release assembly unit без смешения
integration runtime, Communications domain и release authority. Цена решения —
явный package и composition step; это обязательная граница перед production
admission, а не фасад над отсутствующим runtime.
