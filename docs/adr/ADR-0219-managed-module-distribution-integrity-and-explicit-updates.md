# ADR-0219: Целостность managed modules, DistributionManifestV1 и explicit updates

Статус: Принято
Дата: 2026-07-15
Состояние реализации: `managed_launch_trust_v1` открыт на файловой release
authority. Реализованы bounded wire contract и structural validation
`DistributionManifestV1`, verifier raw signed bytes against explicit pinned
P-256 release key, а также `ReleaseTrustRootV1`, содержащий только
release-bound P-256 public keys. Private signing material не имеет пути в
Kernel. Installed bundle проверяется по target triple, stable size/SHA-256 и
без symlink traversal; Kernel создаёт private staged executable copy, а
process adapter принимает только typed staged artifact, выполняет bounded
timeout/retry и не наследует environment. SQLite Control Store хранит bundled
binding и launch record, fenced by binding, Kernel/runtime generation и current
grant epoch. Runtime generation выделяется из persisted per-registration
high-watermark, поэтому suspend/reapprove с новым grant epoch не переиспользует
generation и не блокирует последующий launch. Managed supervisor создаёт fresh
inherited Unix FD на каждую попытку и требует exact `Describe`
descriptor/settings bytes до допуска
process; mismatched digest, module identity или stale fence fail closed.
Замена bundled binding сначала durable фиксирует новую revision, затем
останавливает старый managed worker; client, Blob, Event и Vault data-plane
routes сверяют launch с current binding revision до relay или lease issuance.
Vault routing различает module-registration binding и constitutional platform
process binding: отсутствие module registration у Storage/Vault process не
обходится fallback-авторизацией и не смешивает две независимые единицы сборки.
Production macOS owner-control принимает только artifact ID и bind/start-ит
selected installed release через durable binding; client не передаёт path или
digest. Release compiler выпускает `ReleaseTrustRootV1` и
raw-byte-signed `DistributionManifestV1` из explicit local artifact input и
owner-private P-256 release key; в trust root можно добавить строго
упорядоченные P-256 public keys следующей release authority, private key
material для rotation reject-ится.

Apple Developer ID signing, notarization/stapling и Linux OCI Cosign preflight
сохраняются как optional independent hardening. Они не являются условием
file-backed gate и не заявлены как runtime evidence. Пока отдельная
file-signed OCI binding не реализована, Linux containers остаются только
`external`: Kernel их authenticate, health-check и fence-ит, но не получает
Docker authority и не считает image tag/name/PID/container identity.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md).

Уточняется:

- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).

Этот ADR определяет provenance и integrity boundary executable, которые Kernel
запускает как `managed`, а также границу download/install/update/rollback.
Открытая registration external modules, runtime grants и owner identity
остаются контрактами ADR-0215 и ADR-0218.

## Контекст

ADR-0215 разрешает любому локальному process создать недоверенную `pending`
registration. Это сознательно не требует publisher certificate или включения
в закрытый plugin store.

Но после перевода registration в `managed` Kernel сам запускает и
перезапускает executable. Если launch configuration хранит только path,
заменённые либо устаревшие bytes могут получить прежний launch record и grants.
Проверка только при approval также недостаточна: файл можно заменить между
approval, restart и очередным supervisor launch.

Нужно разделить три разных утверждения:

1. registration доказывает существование локального module process;
2. authorization определяет его effective `GrantSet`;
3. managed-launch integrity доказывает, какие именно executable bytes Kernel
   собирается запустить.

Ни одно из этих утверждений не заменяет остальные.

## Решение

### Registration, authorization и launch integrity независимы

External module по-прежнему может:

- подключиться через private local registration endpoint;
- стать `pending` без publisher signature и digest allowlist;
- после owner approval работать в lifecycle mode `external` с ограниченным
  `GrantSet` и proof-of-possession своей registration identity.

Первый production external-runtime proof обязан включать exact 32-byte digest
observed distribution artifact вместе с registration ID, runtime ID/generation,
Kernel instance ID, current grant epoch и one-shot challenge. Только после
проверки owner-bound ES256 key Kernel создаёт external attestation. Это digest
binding runtime session, а не publisher signature, OCI verification или
managed-launch authority.

Это не даёт ему права быть запущенным Kernel.

Любой переход в `managed` требует отдельного `ManagedLaunchBinding`. Kernel не
запускает managed process, если exact executable bytes, descriptor artifact и,
когда settings объявлены, settings schema artifact не проверены для этой
binding revision.

### Разрешены только два источника managed launch

`ManagedLaunchBinding` имеет один из двух origins:

1. `bundled_distribution` — executable входит в подписанный immutable
   `DistributionManifestV1` текущей установленной Макошь release вместе с exact
   descriptor/settings schema artifact digests;
2. `owner_pinned` — владелец явно выбрал local executable и подтвердил его
   exact executable, descriptor и optional settings schema digests fresh
   operation-bound proof ADR-0218.

Самообъявленный digest из `ModuleDescriptorV1`, observed path, platform
metadata, предыдущий успешный launch или совпавший `module_id` не являются
launch authority. Authority имеет только digest exact received artifact,
зафиксированный signed distribution либо fresh owner approval.

Третьего implicit origin, directory scanning и fallback на соседний executable
нет.

### Signed bundled DistributionManifestV1

Каждая release, содержащая managed Kernel modules или managed platform
services, поставляет один bounded versioned `DistributionManifestV1` и detached
signature.

`DistributionManifestV1` фиксирует минимум:

- `DistributionManifestV1` protocol version;
- stable distribution ID, release version, build ID и target platform/arch;
- Kernel protocol compatibility range;
- для каждого entry exact discriminated `artifact_kind`:
  `module_runtime`, `infrastructure_executable` либо `storage_bundle`;
- `module_runtime`: stable module ID, required/optional classification,
  relative executable path/size/SHA-256, exact `ModuleDescriptorV1`, optional
  `SettingsSchemaV1` artifacts и supported lifecycle/registration ranges;
- `infrastructure_executable`: stable component/tool ID, target platform/arch,
  relative executable path/size/SHA-256 и compatibility range без фиктивного
  `ModuleDescriptorV1`; сюда относятся bundled PostgreSQL, PgBouncer и
  admission-approved companion tools;
- `storage_bundle`: exact owner/persistence package identity, target storage
  revision, artifact relative path/size/SHA-256 и связанный managed module;
  этот artifact не является executable и не выдаёт capability;
- distribution generation и release timestamp как provenance, но не как локальный
  authorization clock.

`DistributionManifestV1` не объявляет и не дублирует capability/dependency
graph. Единственным runtime semantic declaration является exact validated
`ModuleDescriptorV1` ADR-0221. `StorageBundleV1` содержит только admitted
owner-local schema artifact ADR-0224 и не расширяет runtime grants. Settings
values в distribution не попадают.

В wire-contract `DistributionManifestArtifactV1` descriptor и optional settings
schema имеют собственные relative path, bounded size и SHA-256 fields. Одного
digest недостаточно: Kernel должен иметь однозначный bounded file target для
проверки exact bytes до managed launch.

Signature покрывает raw `DistributionManifestV1` bytes. Verifier не
сериализует непроверенный document заново и не принимает remote includes,
absolute paths, `..`, duplicate entries, unknown required fields или ambiguous
component IDs.

Signing key `DistributionManifestV1` является release authority и не совпадает
с:

- owner/device key ADR-0218;
- external module registration key ADR-0215;
- provider credentials;
- Vault keys;
- Tauri client session capability.

Public verification key pin `DistributionManifestV1` поставляется внутри
Kernel release.
Private release key существует только в release pipeline/offline signing
boundary и никогда не попадает в application bundle, Kernel Control Store,
Vault, logs или backups Макошь.

Outer desktop update artifact и inner `DistributionManifestV1` имеют разные
purpose-specific signing keys. Это не позволяет ошибочно использовать owner
или updater capability как managed-launch authority.

Key rotation требует overlap release, уже подтверждённой прежним trust root.
Kernel не получает новые release keys из сети или `ModuleDescriptorV1`.

### Проверка перед каждым managed launch и storage admission

До каждого initial launch, restart, crash recovery и topology transition
Kernel обязан:

1. определить единственный approved `ManagedLaunchBinding`;
2. открыть exact target без следования неожиданной symlink/reparse boundary;
3. проверить canonical target, file type, size и SHA-256 executable digest;
4. для `module_runtime` проверить exact descriptor и optional settings schema
   artifact bytes, size и SHA-256 digests; для
   `infrastructure_executable` проверить exact component/tool binding без
   synthetic descriptor;
5. для bundled entry проверить signature и identity `DistributionManifestV1`;
6. связать launch record с binding revision, всеми artifact digests, Kernel
   generation и runtime instance generation;
7. только после этого создать inherited control channel и process; data-plane
   grants выдаются после совпадения bytes из `Describe` с pinned descriptor.

`storage_bundle` никогда не spawn-ится. Kernel проверяет manifest artifact
identity, а Storage Control перед admission повторно проверяет exact bytes,
digest, owner/persistence identity, previous accepted digest и PostgreSQL AST
по ADR-0224. Running module не может заменить artifact или передать migration
SQL через descriptor/control RPC.

Implementation обязан закрыть verify-then-swap/TOCTOU между чтением bytes и
spawn через platform-safe file identity/handle либо эквивалентную проверяемую
launch ceremony. Обычная последовательность `hash(path)` → `spawn(path)` без
защиты от замены не считается выполнением ADR.

Если signature, path, size, executable/descriptor/settings digest, file
identity, `DistributionManifestV1` version или compatibility не совпали:

- process не запускается;
- registration/capability получает состояние `blocked_integrity`;
- sanitized reason доступен owner diagnostics;
- никакой другой executable не выбирается;
- automatic rollback, downgrade или lifecycle fallback не выполняется.

Required managed component с integrity failure оставляет Kernel в
`recovery_only`. Optional component переводит только зависимые capabilities и
Kernel в `degraded`.

### Owner-pinned managed executable

Переход external registration в `managed` разрешён без publisher certificate,
но требует fresh owner authorization ADR-0218.

Owner approval фиксирует:

- `ModuleRegistrationId`;
- explicit canonical executable target;
- SHA-256 digest и exact size;
- exact `ModuleDescriptorV1` artifact size и SHA-256 digest;
- optional `SettingsSchemaV1` artifact size и SHA-256 digest;
- optional exact `StorageBundleV1` artifact owner/revision/size/SHA-256 digest,
  когда managed module имеет durable owner storage ADR-0224;
- observed platform file identity;
- launch arguments schema без secret values;
- working-directory policy;
- binding revision и owner authorization audit reference.

Изменение executable/descriptor/settings schema/storage bundle bytes, path,
arguments schema или file identity создаёт новую pending launch-binding
revision. Оно не наследует approval автоматически.

Kernel не утверждает, что owner-pinned executable имеет trusted publisher.
Он утверждает только, что владелец явно разрешил запуск exact inspected bytes.

### External lifecycle остаётся открытым

Для lifecycle mode `external` executable signature/digest не являются
обязательным registration condition. Kernel не запускает такой process и не
обещает integrity его installation.

External module всё равно ограничен:

- private local endpoint;
- proof-of-possession registration identity;
- owner-approved grants;
- grant epoch и session fencing;
- hard Kernel policy;
- explicit health/disconnect semantics.

Unmanaged external process не получает storage mutation/migration capability.
Она появляется только после owner-pinned exact `StorageBundleV1` и transition
в managed lifecycle по ADR-0224.

Observed executable path, digest и platform code-signing metadata могут
показываться как untrusted provenance. Они не меняют grants и не становятся
managed binding без отдельного owner transition.

### Kernel не скачивает и не устанавливает executable code

Kernel и module runtime не имеют capability загрузки, распаковки, замены или
установки executable code.

Для desktop release download и signature verification выполняет host update
boundary. В существующем Tauri stack базовым кандидатом является официальный
[Tauri Updater](https://v2.tauri.app/plugin/updater/), который использует
подписанные update artifacts. OS package manager или platform distributor
может выполнять ту же роль для соответствующего installation profile.

Android application update остаётся platform client concern и не превращает
Android в downloader Kernel modules.

Plugin store, runtime download по `ModuleDescriptorV1`, remote frontend code и
загрузка dynamic library из NATS/PostgreSQL/Vault запрещены.

### Update lifecycle

Update выполняется вне Kernel process:

1. host updater получает и проверяет signed update artifact;
2. Kernel quiesce/drain и останавливает managed children;
3. host atomically устанавливает целую release, а не отдельный module поверх
   работающей distribution;
4. новый Kernel при bootstrap проверяет inner signed
   `DistributionManifestV1`;
5. каждый managed/infrastructure executable повторно проверяется перед launch,
   а descriptor/settings schema artifacts — только для `module_runtime`;
6. каждый target `StorageBundleV1` повторно проверяется до Storage Control
   migration admission;
7. только compatible и verified distribution получает data plane.

Kernel не hot-load-ит новый Rust code и не меняет executable во время работы.
Module code update всегда создаёт новый process generation.

Partial install, mixed release, неизвестный `DistributionManifestV1` или
неуспешная проверка оставляют только безопасный recovery surface.

### Rollback только explicit

Rollback является отдельной owner-authorized host operation на конкретную
ранее подписанную и доступную release.

Перед rollback проверяются:

- update artifact signature;
- inner `DistributionManifestV1` signature;
- target platform/arch;
- Kernel protocol и Control Store schema compatibility;
- compatibility старой binary с уже применёнными owner storage bundles;
- executable digests после installation.

Kernel не выбирает N-1 release после crash, failed health probe или integrity
failure. Supervisor retry не является rollback.

Если старая release несовместима с текущим Control Store или уже применённым
`StorageBundleV1`, rollback fail closed и оставляет recovery instructions.
Automatic destructive down-migration либо reset запрещены.

### Persisted control state

Kernel Control Store хранит для managed registration только technical binding:

- origin `bundled_distribution` или `owner_pinned`;
- distribution identity либо owner-pinned target identity;
- executable, descriptor и optional settings schema SHA-256 digests/sizes плюс
  bounded executable file identity metadata;
- binding revision, approval reference и current state;
- last verified release/generation как diagnostics, не как замена проверки;
- `blocked_integrity` reason code без private path leakage клиентам без права
  diagnostics.

Binding record не хранит exact artifact bytes. Module Registry и Settings
Registry отдельно сохраняют exact bounded validated descriptor/settings schema
bytes для recovery и diff по ADR-0221/0222.

Control Store не хранит:

- release/update private keys;
- downloaded artifacts или executable bytes;
- arbitrary installer commands;
- automatic fallback target;
- publisher trust claim для owner-pinned module.

Binding update + lifecycle transition + generation/epoch increment сохраняются
атомарно до запуска новых bytes.

## Threat boundary

Решение защищает от:

- случайной или злонамеренной замены managed executable после approval;
- запуска stale/mixed release;
- превращения открытой registration в право на managed launch;
- повторного использования старого digest после изменения bytes;
- silent fallback на другой path/version;
- загрузки executable через compromised business/module data plane.

Полный compromise host administrator, release-signing environment либо самого
Kernel executable остаётся вне первой threat boundary. Platform code signing,
notarization и Tauri artifact signature остаются дополнительными независимыми
слоями и не отменяют inner managed-launch verification.

## Выбор libraries и готовых механизмов

- desktop artifact download/install использует Tauri Updater либо platform
  package manager, а не новый Kernel downloader;
- SHA-256 берётся из audited Rust cryptography implementation, не пишется
  вручную;
- [`minisign-verify`](https://github.com/jedisct1/rust-minisign-verify)
  является компактным кандидатом для detached `DistributionManifestV1`
  verification;
- точные dependency versions, signature suite и platform-safe spawn adapters
  выбираются dependency/security review до открытия первого
  `managed_launch_trust_v1` path по ADR-0225. Recovery-only Kernel без managed
  launch не блокируется этой будущей зависимостью.

TUF/Sigstore/marketplace infrastructure не вводятся в первую local
distribution: Kernel не является remote package repository client. Это решение
можно пересмотреть только при появлении реального multi-publisher distribution
channel.

## Отклонённые варианты

### Требовать publisher signature для любой registration

Отклонено: связывает локальное module discovery с release infrastructure и
противоречит открытой `pending` registration ADR-0215.

### Проверять digest только при owner approval

Отклонено: bytes могут измениться до restart или supervisor retry.

### Доверять path, `module_id` или platform metadata

Отклонено: эти значения не доказывают exact launched bytes.

### Разрешить Kernel скачивать modules

Отклонено: расширяет Kernel до package manager, создаёт network/parser/archive
attack surface и противоречит запрету plugin store.

### Автоматически откатываться после crash

Отклонено: crash не доказывает, что старая release безопасна или совместима, и
может создать downgrade loop либо скрытую потерю state.

### Один shared signing key для owner, modules и releases

Отклонено: смешивает независимые authority domains и расширяет последствия
компрометации одного key.

## Проверка решения

До изменения `Состояние реализации` обязательны tests:

- unsigned external process может стать только `pending`, не получает grants;
- approved external runtime работает без publisher signature, но Kernel его не
  запускает;
- переход external → managed без owner-pinned digest отклоняется;
- bundled managed launch без valid `DistributionManifestV1` signature
  отклоняется;
- duplicate/unknown distribution fields, duplicate IDs, path traversal,
  absolute path и unsupported version отклоняются;
- wrong target/arch, size или executable/descriptor/settings SHA-256 digest дают
  `blocked_integrity`;
- verification выполняется перед каждым initial launch и restart;
- file replacement между verification и spawn не запускает substituted bytes;
- owner-pinned binding не принимает self-reported descriptor digest;
- изменение owner-pinned executable, descriptor или settings schema bytes
  требует новой approval revision;
- integrity failure required component оставляет `recovery_only`;
- integrity failure optional component даёт scoped `degraded`;
- Kernel не имеет executable download/install capability;
- partial/mixed install не запускает data plane;
- update сначала drain/stops old managed children и создаёт новые generations;
- unsigned, wrong-platform и incompatible rollback отклоняются;
- crash/health failure не вызывает automatic rollback или fallback;
- private signing keys, artifact bytes и private paths отсутствуют в Control
  Store, logs, errors, health, telemetry и client-visible diagnostics;
- key rotation принимает новый key только через release, доверенную прежним
  trust root.

## Последствия

Положительные:

- открытая local module registration не требует marketplace или PKI;
- Kernel запускает только exact approved bytes;
- bundled и owner-pinned modules используют одну fail-closed launch model;
- update/download responsibility остаётся за существующим host boundary;
- crash не вызывает скрытый downgrade или смену lifecycle topology.

Стоимость:

- release pipeline должен подписывать inner `DistributionManifestV1` и outer
  desktop artifact;
- нужен platform-safe launch adapter без verify-then-swap;
- owner-pinned executable update требует нового approval;
- rollback зависит от совместимости technical state и не может быть полностью
  автоматическим;
- conformance, tamper и partial-install tests обязательны до production.
