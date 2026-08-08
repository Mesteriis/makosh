# ADR-0267: Kernel-staged runtime artifacts and integration state roots

Статус: Принято
Дата: 2026-07-24
Состояние реализации: owner-neutral wire foundation реализован в
`makosh-runtime-protocol`, а Kernel реализует exact intersection verified
descriptor/effective grants/signed manifest, launch-scoped staging native
dependencies и owner-private integration state root. Telegram runtime больше
не принимает artifact/database paths через settings: он проверяет exact staged
binding и создаёт provider-owned TDLib layout только внутри state root.
Canonical Telegram schema содержит только non-secret account/API identity, а
credential revision выбирается из Telegram-owned operational binding, не из
settings. Native-loader conformance ещё не реализован. Решение является
обязательным prerequisite для `telegram_integration_v1`, но само не открывает
этот gate и не расширяет production inventory.

Telegram-owned PostgreSQL schema также оформлена как immutable
`telegram_state` `StorageBundleV1`. Bundle является отдельным integration
assembly artifact: его digest и revision допускаются Storage Control
независимо от module executable, descriptor и settings schema.

Уточняет:

- [ADR-0204: integration plugins and provider-neutral context boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Vault and scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0238: secure-file FD boundary](ADR-0238-secure-file-fd-boundary.md);
- [ADR-0266: Telegram Kernel admission](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md).

## Контекст

Telegram runtime сейчас получает `telegram.tdjson_artifact_path` и
`telegram.database_directory` через module settings. Это смешивает три
независимые ответственности:

1. release authority над exact native runtime dependency;
2. Kernel authority над managed process topology;
3. integration ownership над high-churn provider session state.

Путь к native library не является пользовательской или operator-managed
настройкой. Он должен выводиться только из verified release или fresh
owner-pinned binding ADR-0219. Иначе settings могут подменить code, а
`ModuleDescriptorV1` становится скрытым package manager.

Каталог TDLib database также не является setting. Его содержимое меняется в
результате обычной работы integration, содержит provider session state и
переживает restart. ADR-0204 и ADR-0223 оставляют такой state у integration
owner, а ADR-0222 прямо запрещает хранить runtime state в Settings Registry.

Kernel при этом не должен знать Telegram, TDLib или layout provider database.
Нужно owner-neutral решение, которое stage-ит platform bindings в managed
runtime, но не превращает Kernel в provider facade, filesystem API или
business peer integration.

## Решение

### Три разные authority

Границы разделены так:

| Responsibility | Authority |
|---|---|
| exact bytes native runtime dependency | signed distribution или fresh owner-pinned binding |
| descriptor request и capability association | integration module |
| grant, manifest intersection, staging и fencing | Kernel |
| provider state layout, migration и contents | integration owner |
| wrapping key для encrypted provider state | Vault lease |
| provider observations и canonical evidence | integration outbox → event spine → Communications |

Descriptor declaration не выдаёт право на artifact. Manifest entry не выдаёт
capability. Staged path не является install authority. Effective binding
существует только как пересечение exact descriptor request, approved
capability, current managed launch binding и Kernel hard policy.

### Runtime artifact является отдельной assembly unit

`DistributionArtifactKindV1` получает отдельный kind
`module_runtime_native_dependency`. Это не `module_runtime`,
`infrastructure_executable`, `storage_bundle`, setting или secret.

Manifest entry для native dependency фиксирует:

- stable `artifact_id`;
- exact `bound_module_id`;
- release-relative path, non-zero bounded size и SHA-256;
- target distribution/architecture через parent manifest;
- required/optional classification.

Entry не содержит synthetic descriptor, settings schema, provider credentials
или произвольный loader command. Один artifact ID не может быть неоднозначно
связан с несколькими modules в одной distribution.

`CapabilityRequestV1` получает typed `RuntimeArtifactRequestV1`:

```text
artifact_id
use = native_dynamic_library
```

Request находится внутри exact capability descriptor. Generic
`HostCapabilityRequestV1`, environment variable, argv, setting,
`resource_reference` или convention-based соседний filename для этого не
используются.

Для Telegram exact logical binding:

```text
module_id   = makosh-telegram-runtime
artifact_id = telegram.tdjson.v1
use         = native_dynamic_library
```

`makosh-telegram-tdlib` остаётся owner-local adapter package, а native TDLib
artifact — отдельная release assembly unit. Ни artifact, ни adapter не входят
в Communications build unit.

### Kernel-staged runtime artifact binding

Перед каждым initial launch, restart и generation replacement Kernel:

1. проверяет managed module executable, descriptor и settings schema по
   ADR-0219;
2. извлекает typed runtime-artifact requests только из exact verified
   descriptor;
3. пересекает их с approved capability grant и единственным manifest/owner
   binding;
4. проверяет artifact kind, `artifact_id`, `bound_module_id`, target, path,
   file type, size, digest и current binding revision;
5. создаёт private one-shot `ManagedRuntimeArtifactBindingV1`;
6. связывает binding с registration ID, runtime instance/generation и grant
   epoch;
7. передаёт binding только через inherited managed-runtime bootstrap channel.

Binding содержит exact artifact ID/use, staged local path, size и SHA-256.
Он не публикуется через Gateway, recovery API, events, health или settings.
Module принимает только объявленный artifact ID/use и повторно проверяет
already-opened regular file через public secure-file boundary до loader call.
Ошибки не включают private path.

`hash(path) → dlopen(path)` сам по себе не закрывает TOCTOU и не считается
production evidence. Production gate требует platform-specific доказательства
того, что verified file identity сохраняется до native loader admission:

- Kernel stage-ит exact bytes в private launch-scoped location из verified
  installed artifact;
- runtime открывает terminal file без symlink traversal, сверяет size/digest и
  удерживает descriptor до loader completion;
- macOS launch использует hardened runtime/library validation и не разрешает
  fallback на другой path;
- conformance подменяет source/staged path до и после verify и доказывает
  fail-closed либо сохранение exact file identity.

Пока эта ceremony и process/host isolation не доказаны, foundation contract
может быть реализован и протестирован, но `telegram_integration_v1` остаётся
закрыт. Нельзя описывать same-UID filesystem как физически изолированный без
отдельного OS-level evidence.

### Integration state root

`ManagedIntegrationRuntimeConfigurationV1` получает optional typed
`IntegrationStateRootV1`. Kernel создаёт его только для admitted integration
capability, запросившей private provider state, и stage-ит:

```text
root_path
state_generation
state_layout_revision
```

Root детерминированно scoped по Макошь instance, logical owner, module
registration и opaque configuration instance. Client, settings и provider не
передают filesystem path. Account label, phone, email, provider body или
credential metadata в path не попадают.

Kernel отвечает только за:

- выбор корня внутри explicit Макошь data directory;
- создание private parent/child directories без symlink traversal;
- permissions, owner binding и current state generation;
- отсутствие path alias между registrations/configuration instances;
- передачу exact binding в admitted runtime.

Integration отвечает только за:

- внутренний layout и versioning под выданным root;
- bounded child names и запрет выхода через `..`, absolute path или symlink;
- provider session database lifecycle;
- atomic local writes, crash recovery и cleanup своих derived files;
- получение `SessionStoreKeyLease` из Vault для encryption, когда это требует
  provider store.

Kernel не читает provider database, не знает TDLib schema, не мигрирует
integration state и не проксирует filesystem operations. Root не является
PostgreSQL namespace, Blob store, settings directory или общей папкой domains.
Restart и settings apply не очищают state root. Revoke закрывает runtime access
и новые leases, но не удаляет durable state. Удаление является отдельной
owner-confirmed lifecycle operation.

### Telegram migration

Canonical Telegram settings schema не содержит:

```text
telegram.tdjson_artifact_path
telegram.database_directory
telegram.api_hash_revision
telegram.session_encryption_key_revision
```

TDJson runtime artifact берётся только из exact staged artifact binding.
TDLib database root берётся только из exact integration state-root binding.
Session-store key выбирается через exact Vault purpose и current
configuration-instance target, а не через secret revision setting.
Active purpose/revision хранится как Telegram-owned non-secret operational
binding в owner-local PostgreSQL и создаётся/заменяется только explicit
provider setup/rotation flow. Binding не содержит Vault record ID или
`secret_ref`. Kernel авторизует статические descriptor purposes, но не хранит
provider credential binding; Vault lease после выбора связан с exact revision.

Оставшиеся Telegram settings могут описывать только настоящую operator
configuration, например opaque configuration instance/account selection и
non-secret provider API identity. Secret values, secret references, artifact
paths и runtime state запрещены.

Это изменение не меняет business data flow ADR-0266:

```text
Telegram integration outbox
        ↓ typed DurableEnvelopeV1
NATS JetStream
        ↓
Communications inbox/domain
```

Kernel не получает и не преобразует observation payload.

## Assembly и SRP

Контракты разделены по причине изменения:

- distribution manifest меняется при release composition;
- descriptor request меняется при module capability contract;
- managed runtime binding меняется при Kernel launch/staging protocol;
- integration state adapter меняется при provider state lifecycle;
- Communications ingress меняется при neutral evidence contract.

Ни один из этих контрактов не объединяется только потому, что один Telegram
process использует их вместе. Cargo dependencies сохраняют направление:

```text
telegram-runtime → telegram-tdlib
telegram-runtime → public platform contracts
telegram-runtime → communications-ingress

kernel → public platform contracts
kernel ✕ telegram-*
communications-* ✕ telegram-*
```

### Порядок реализации

1. Добавить owner-neutral Protobuf contracts и strict validators для native
   runtime artifact request/binding и integration state root.
2. Расширить distribution verification exact module binding и negative tests.
3. Реализовать Kernel staging из verified binding без owner imports.
4. Перевести Telegram bootstrap/TDLib adapter с path settings на staged
   bindings и secure exact-byte verification.
5. Создать canonical Telegram descriptor/settings schema и exact Vault
   purposes.
6. Доказать launch/restart/revoke/stale-generation и filesystem substitution
   conformance.

Каждый крупный slice является отдельным commit.

## Отклонённые варианты

### Оставить paths в settings

Отклонено: settings становятся code-loading и filesystem authority, нарушают
ADR-0219/0222 и позволяют configuration drift менять assembly.

### Положить TDLib внутрь Telegram executable без отдельного inventory

Отклонено как скрытие assembly dependency. Static linking допустим только
после отдельного ADR и exact distribution/SBOM evidence; оно не должно
возникать как неявный обход manifest binding.

### Передать произвольный host path через descriptor

Отклонено: self-declared descriptor не является launch authority, а module
path не входит в signed release binding.

### Хранить provider session database в Kernel, Vault или Communications

Отклонено: Kernel не владеет business/provider state, Vault не является
high-churn database, Communications не импортирует provider operational state.

### Создать один общий integration state directory

Отклонено: ломает owner/configuration isolation, revoke lifecycle и
возможность независимо переносить integration units.

## Проверка решения

До открытия `telegram_integration_v1` tests обязаны доказать:

- неизвестный artifact kind/use, duplicate artifact ID, wrong bound module,
  path traversal, symlink, size/digest mismatch и missing required artifact
  fail closed;
- descriptor request без grant и grant без verified artifact не создают
  staged binding;
- binding с чужими registration/runtime generation/grant epoch отклоняется;
- runtime не принимает artifact/path из settings, argv или environment;
- state roots разных owners/registrations/configuration instances не
  пересекаются и не допускают traversal/symlink alias;
- restart сохраняет state root, revoke не удаляет durable state, explicit
  removal требует owner confirmation;
- Telegram загружает только `telegram.tdjson.v1` exact bytes и использует
  только staged state root;
- Kernel/Gateway не импортируют Telegram packages и не декодируют provider
  payload;
- Communications не импортирует Telegram и получает observation только через
  typed event spine;
- secrets, paths, session metadata и private content отсутствуют в logs,
  events, health и client APIs.

## Последствия

Clean-room Telegram перестаёт маскировать executable binding и provider state
под settings. Kernel получает узкий owner-neutral staging contract, но не
provider semantics. Integration сохраняет автономию над session state, а
Communications остаётся отдельным domain owner, связанным с Telegram только
durable typed events.

Цена решения — новый runtime-protocol slice и platform-specific native-loader
conformance до production admission. Эта цена явная и не компенсируется
facade, fallback path или ложным `done`.
