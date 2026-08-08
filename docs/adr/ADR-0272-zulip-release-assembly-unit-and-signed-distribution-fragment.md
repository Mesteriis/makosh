# ADR-0272: Zulip release assembly unit and signed distribution fragment

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Реализовано в `81449906e` и `ff2c53983`. Canonical Zulip
runtime descriptor, settings schema и immutable owner-local Storage bundle
материализуются отдельной `makosh-zulip-assembly` unit в exact unsigned
двухэлементный fragment. Generic compiler повторно проверяет и подписывает
runtime/Storage bindings; Rust, distribution и architecture guards доказывают
gates 1–7. `zulip_integration_v1` этим не открыт.

Зависит от:

- ADR-0212: crate topology and compile isolation;
- ADR-0213: code ownership and module autonomy;
- ADR-0219: managed distribution integrity;
- ADR-0221: module descriptor and capability lifecycle;
- ADR-0224: Storage Control and migration lifecycle;
- ADR-0271: Zulip Kernel admission and event-only handoff.

## Контекст

Zulip runtime, generated client/descriptor, settings schema и owner-local
Storage bundle изменяются по разным причинам. Production release должен
связать их exact bytes, но runtime не должен становиться build tool, а
integration не должна получать signing authority.

Zulip не имеет отдельной native runtime dependency, поэтому копирование
Telegram assembly вместе с `tdjson` entry создало бы ложную единицу сборки.

## Решение

Вводится отдельный Cargo package:

```text
package = makosh-zulip-assembly
role    = integration
owner   = zulip
surface = assembly
binary  = makosh-zulip-assembly
```

Assembly является build-time tool. Kernel никогда его не запускает. Он имеет
только односторонние зависимости на canonical Zulip artifact builders:

```text
zulip-assembly -> zulip-runtime library
zulip-assembly -> zulip-persistence library

zulip-runtime      ✕ zulip-assembly
communications-*   ✕ zulip-assembly
kernel/gateway     ✕ zulip-assembly
```

Для exact `build_id` и новых absolute output/source paths tool материализует:

```text
zulip.runtime.descriptor.pb
zulip.runtime.settings.pb
zulip.storage.bundle.pb
zulip.release-artifacts.json
```

Первые три файла являются exact bytes из canonical Rust builders. JSON
содержит только два отсортированных artifact entries:

```text
zulip.runtime.v1 -> module_runtime
zulip.storage.v1 -> storage_bundle
```

Fragment не содержит digest, signature, grant, runtime generation, credential,
provider account, queue cursor, realm URL, message content или Communications
payload. Generic distribution compiler повторно проверяет regular non-symlink
files, вычисляет size/digests и подписывает только полный owner-composed
distribution manifest.

Assembly:

- не читает signing key и не подписывает fragment;
- не регистрирует module и не выдаёт grant;
- не создаёт Storage binding или database;
- не читает runtime settings snapshot/provider state;
- не перезаписывает существующий output;
- не импортируется runtime, Kernel, Gateway или Communications.

## Единицы сборки и SRP

- API/descriptor меняется вместе с public operational/admission contract;
- settings schema меняется вместе с configuration contract;
- persistence bundle меняется вместе с owner-local schema;
- runtime меняется вместе с provider orchestration;
- assembly меняется только вместе с составом release artifacts;
- generic compiler меняется вместе с distribution/signing format.

Физическое включение нескольких units в один release не объединяет owner,
runtime lifecycle или authority.

## Проверяемый gate

1. exact assembly package metadata and one-way dependency graph;
2. deterministic equality canonical descriptor/settings/storage bytes;
3. fail-closed new output directory, regular files and no symlinks;
4. refusal to overwrite or accept duplicate artifact identity/path;
5. exact sorted two-entry fragment without private data;
6. generic signed distribution test for runtime and Storage entries;
7. architecture proof that Kernel/Gateway/Communications/runtime do not depend
   on the assembly unit.

Наличие assembly не открывает `zulip_integration_v1`: managed launch, grants,
fences, live provider and event conformance остаются обязательными по ADR-0271.

Текущее evidence:

- `cargo test -p makosh-zulip-persistence -p makosh-zulip-assembly`: 7 passed;
- strict Clippy для Zulip persistence/runtime/assembly: passed;
- signed distribution compiler test materializes exact Zulip assembly output
  and binds runtime/Storage digests: passed;
- backend architecture/policy/SRP/Cargo/fmt/evidence gates: 464 architecture
  tests passed.

## Отклонённые варианты

### Runtime CLI export

Отклонено: managed provider lifecycle и build composition имеют разные причины
изменения.

### JavaScript повторно кодирует descriptor

Отклонено: появится второй schema source и возможность подписанного drift.

### Integration подписывает fragment

Отклонено: Zulip assembly не владеет release trust root.

### Generic integration assembly с optional native dependency

Отклонено: условный mega-builder смешивает Mail, Telegram и Zulip artifact
inventories. Общим остаётся только distribution compiler.

## Последствия

Zulip получает собственную release assembly unit без превращения integration в
domain, runtime в build tool или generic compiler в provider-aware component.
