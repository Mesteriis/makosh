# ADR-0218: Owner/device identity, enrollment и offline recovery

Статус: Принято
Дата: 2026-07-15
Состояние реализации: offline Control Store anchor/export/restore/reset fencing
и public initial-owner record реализованы в `kernel_recovery_only_v1`.
Текущий file-backed ES256 adapter имеет explicit key generation, Kernel-side
proof verification и atomic first claim без development mode. Inherited-FD
ceremony и host integration ещё не созданы.

Зависит от:

- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md).

Уточняется:

- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).
- [ADR-0228: Development simulation profile](ADR-0228-development-simulation-profile.md).

Этот ADR определяет identity владельца и first-party client devices,
proof-of-possession, initial enrollment и authorization recovery operations.
Module identity остаётся отдельным contract ADR-0215. Provider-account, agent,
remote federation и settings semantics этим решением не определяются.
ADR-0222 использует эту OwnerAuthority для operator-managed mutations и может
потребовать fresh operation-bound proof для security-sensitive setting.

## Контекст

Kernel разрешает высокорисковые операции:

- approve, suspend и revoke module grants;
- запуск и остановку managed infrastructure;
- pairing и revoke client devices;
- Vault recovery entrypoints;
- backup validation и восстановление technical control state.

Private local socket и UID/GID доказывают только OS account, но не отличают
Tauri, operator CLI и module process, работающие от того же пользователя.
Постоянный shared token создал бы единую точку компрометации. Owner key из
Vault или Control Store сделал бы восстановление невозможным при отказе именно
этих компонентов.

Одновременно повреждённый Control Store нельзя использовать для проверки
device public keys или revocation state. Поэтому online и offline recovery
должны иметь разные trust roots.

## Решение

### Owner является authority, device является cryptographic principal

Первая версия Макошь имеет одного logical `OwnerAuthority` и любое число
отдельно отзываемых `DeviceIdentity`.

Owner не представлен одним общим private root key или shared secret. Owner
authority реализуется через:

- первоначальную local enrollment ceremony;
- approved device identities;
- typed authorization capabilities;
- offline recovery ceremony при потере trustworthy identity registry.

Каждое desktop, Android или operator device имеет собственную dedicated
signing keypair. Один private key никогда не копируется на другое устройство.
Компрометация или revoke одного device не меняет ключи остальных.

OS account, process name, PID, Tauri window label, IP address и self-declared
`device_id` сами по себе не являются owner identity.

### Identity records принадлежат Kernel Control Store

Trustworthy Control Store хранит только public/control state:

- logical `owner_id`, status и revision;
- `device_id`, owner relation и public signing key;
- protocol/key-suite version;
- granted owner-operation capabilities;
- enrollment provenance и safe timestamps;
- active/revoked/suspended state;
- revocation revision и identity epoch;
- last accepted assurance class как наблюдаемую provenance, а не как
  самостоятельное основание authorization.

Initial owner/device enrollment и последующий device revoke + epoch increment
являются атомарными SQLite transactions.

В Control Store, PostgreSQL, Макошь Vault, backups, logs, events и diagnostics
запрещены:

- owner/device private keys;
- exportable copies private keys;
- authentication challenges после consumption;
- session capability values;
- biometric data или platform authentication secrets.

OS secure storage не становится второй authoritative registry: оно хранит
только private key текущего client device. Public keys, status и revocation
truth принадлежат Control Store.

Owner/device ES256 keypair не используется для Vault encryption или wrapping.
`PlatformWrappingKey`, `VaultRootKey` и `RecoveryKeyV1` принадлежат отдельной
Vault key hierarchy ADR-0223 и не являются owner/device identity. В обратную
сторону `RecoveryKeyV1` даёт только decrypt authority конкретного Vault backup:
он не является `OwnerAuthority`, client session, module grant или способом
восстановить Control Store identity registry.

### Device key suite v1

Wire suite первой версии фиксирован как `ECDSA P-256 + SHA-256` (`ES256`).
Клиент не выбирает произвольный algorithm.

Wire representation:

- public key — SEC1 uncompressed P-256 point, ровно 65 bytes;
- signature — fixed-width `r || s`, ровно 64 bytes;
- protocol version и algorithm enum проверяются до cryptographic operation;
- malformed, non-canonical и unsupported inputs отклоняются fail closed.

P-256 выбран как фиксированный v1 wire suite для file-backed adapter и будущих
replaceable adapters. Ed25519 не входит в первую версию протокола, чтобы не
расширять число key-suite и verification paths.

Kernel зависит от узкого verifier port, а clients — от `DeviceSigner` host
port. Конкретный Rust crypto backend не является wire contract. Для первого
recovery-only slice ADR-0225 разрешает exact `p256` verifier, `getrandom` и
`sha2` profile с explicit security warning, dependency/advisory review и
platform conformance vectors. Самописная криптография запрещена; замена backend
не меняет wire suite, но требует отдельного policy/security evidence.

### Private key остаётся в replaceable signer adapter

Первая реализация — `FileDeviceSigner`: owner-private regular file `0600` в
owner-private data directory. Adapter генерирует ES256 key, возвращает только
public SEC1 key и fixed-width signature, а Kernel работает исключительно через
узкий `DeviceSigner` port. Другой adapter можно добавить позже без изменения
wire contract или Kernel authorization logic.

Vue/WebView, Kernel, modules и provider runtimes не получают private key bytes.
Tauri/Android host bridge может обслуживать только typed identity signing
operation, а не произвольное «подпиши эти bytes».

File adapter не даёт non-exportability или отдельной user-presence ceremony.
Это осознанный риск первого релиза: доступ к private key file
эквивалентен owner device signing authority. Key file нельзя синхронизировать,
передавать в logs, backups или provider storage. Будущий adapter может добавить
дополнительную protection, но не меняет authorisation semantics сам по себе.

Локальная privileged CLI mutation с baseline adapter создаёт fresh
domain-separated proof, привязанный к instance, owner/device identity, Kernel
generation и digest конкретной operation. Kernel сверяет signer public key с
Control Store перед mutation. Этот process-local proof не является transport
session и не открывает remote owner authorization; Gateway session conformance
остаётся отдельным условием `module_control_plane_v1`.

### Challenge-response и sessions

Kernel никогда не просит device подписать произвольный Protobuf payload.

Для proof-of-possession Kernel:

1. создаёт криптографически случайный single-use challenge;
2. server-side связывает его с `instance_id`, `device_id`, purpose,
   digest конкретной privileged operation, Kernel generation и expiry;
3. передаёт только versioned opaque challenge bytes;
4. проверяет domain-separated ES256 signature approved device;
5. атомарно помечает challenge consumed;
6. отклоняет duplicate, expired, revoked, wrong-purpose и stale-generation
   response.

Signature bytes не используются как idempotency key, correlation ID или
session token.

После успешной обычной device authentication Kernel может выдать short-lived
client-session capability. Она:

- существует только в памяти Kernel и client process;
- привязана к device identity, Kernel generation, transport profile, expiry и
  authorized capability set;
- не хранится в URL, browser storage, Control Store, logs или errors;
- немедленно становится недействительной после revoke/epoch increment;
- не заменяет fresh user-presence proof для privileged operation.

Authentication отвечает, какое device доказало владение key. Authorization
отдельно проверяет, разрешена ли этому device конкретная operation.

### Initial desktop enrollment

Первый owner и первый desktop device создаются только для действительно
pristine Макошь instance.

Tauri host:

1. генерирует device key через platform signer;
2. создаёт anonymous inherited pipe/FD до запуска Kernel;
3. получает fresh bootstrap nonce;
4. отправляет один versioned, size-bounded enrollment frame с public key и
   proof-of-possession;
5. закрывает bootstrap channel до запуска любых module runtimes.

Kernel принимает frame только если одновременно:

- data directory выбрана по ADR-0217 и удерживается exclusive lock;
- instance имеет pristine installation anchor;
- trustworthy Control Store создаётся впервые и остаётся unclaimed;
- frame пришёл через ожидаемый inherited FD;
- challenge, version, size и proof-of-possession корректны.

Kernel одной transaction создаёт `OwnerAuthority`, первый `DeviceIdentity`,
identity epoch и instance state. Existing, restored, missing-after-install или
untrusted Control Store всегда отклоняет bootstrap enrollment.

Inherited enrollment FD не совпадает с module control FD ADR-0215 и не
передаётся modules.

### Pristine install и installation anchor

Чтобы удалённая SQLite не выглядела как fresh install, Kernel создаёт в private
data directory маленький non-secret installation anchor до initial enrollment.
Anchor содержит только versioned installation identity и initialization state;
он не является configuration file или authorization registry.

Правила:

- новый отсутствующий data directory может стать pristine instance;
- существующий anchor + missing/untrusted Control Store означает recovery, а
  не automatic re-enrollment;
- unexpected Макошь artifacts без valid anchor также означают recovery;
- crash между anchor и SQLite commit не открывает повторную enrollment;
- полное явное удаление data directory создаёт другой instance;
- restore обязан проверить installation identity либо потребовать explicit
  offline reset/new-instance ceremony.

Initialization protocol обязан fsync/atomically replace anchor и иметь
crash-injection tests для каждой boundary.

### Headless first enrollment

Headless profile не получает публичный claim endpoint. Его эквивалентная first
enrollment выполняется только local interactive offline `init` operation:

- на pristine instance;
- при остановленном Kernel;
- с explicit `--data-dir`;
- под exclusive lock;
- через configured `DeviceSigner` adapter;
- с явным подтверждением владельца.

Remote first enrollment запрещена.

### Online recovery при trustworthy Control Store

Когда Control Store trustworthy, owner operations доступны только
authenticated device session с подходящей typed capability. Privileged
recovery/device/module/infrastructure mutations дополнительно требуют fresh
operation-bound challenge и platform user presence.

Module `GrantSet` ADR-0215 и owner/device capabilities являются разными
authorization namespaces. Module registration или inherited module FD не дают
owner rights.

### Recovery при unavailable/untrusted Control Store

Device signatures нельзя авторизовать через недоверенный registry. Поэтому
online local recovery surface разрешает только:

- sanitized `status`;
- Control Store `validate`;
- Control Store `export`.

Online `restore`, `reset`, owner/device enrollment, module approval,
infrastructure action и любой business data plane запрещены.

`restore` и `reset` являются offline operations того же packaged Kernel
tooling. Для них одновременно обязательны:

- остановленный Kernel и managed children;
- explicit `--data-dir`;
- exclusive instance lock;
- local interactive invocation под owner OS account;
- показ безопасного target/operation summary;
- явное confirmation без silent/automatic mode;
- отсутствие зависимости от PostgreSQL, NATS или Vault.

Offline authority основана на локальном OS account, private filesystem
boundary, владении exclusive lock и явной operator ceremony. Это не переносится
на online authorization.

Offline Control Store reset/restore:

- не удаляет PostgreSQL, Vault, provider sessions, blobs или business data;
- не восстанавливает Vault key hierarchy, credential bindings или leases;
- не читает private device keys;
- для trustworthy Store повышает Kernel generation, identity epoch и grant epochs
  до data plane;
- для missing или untrusted Store `reset` является отдельной destructive
  `new-instance` ceremony: только после offline confirmation он atomically
  заменяет installation anchor и Control Store; новая instance identity
  инвалидирует все прежние sessions/completions вместо имитации старых epochs;
- инвалидирует прежние client sessions и stale module completions;
- требует повторной enrollment только если восстановленный trusted state не
  содержит действующей owner/device identity.

Automatic reset, automatic fallback и non-interactive destructive flag первой
версии запрещены.

### Android device

Android использует отдельный owner-private `FileDeviceSigner` key file и никогда
не получает desktop key или shared owner token. Pairing требует уже
authenticated owner device и одноразовый challenge.

Каждый Android device имеет независимые public key, capabilities, sessions,
replay cursors, audit и revoke. Точные pairing UX, QR/deep-link transport,
remote Kernel certificate identity и Android topology требуют отдельных ADR,
но не меняют эту identity model.

## Threat boundary

Модель защищает от:

- module process, который пытается использовать общий UID как owner identity;
- replay старого challenge/session после restart или revoke;
- потери одного device без компрометации остальных;
- automatic takeover после удаления/повреждения SQLite;
- утечки private key через Kernel, browser storage, Vault backup или logs.

Полный compromise OS account или host administrator остаётся вне первой threat
boundary: такой actor может читать private files, подменять executable и
запускать offline tooling. Online Kernel всё равно не выдаёт owner rights
только по UID.

## Отклонённые варианты

### Доверять local socket и UID

Отклонено: module processes имеют тот же OS identity.

### Общий owner token или общий device key

Отклонено: revoke одного device потребовал бы rotation всех clients, а утечка
одного секрета дала бы полную owner authority.

### Хранить private key в Vault или Control Store

Отклонено: создаёт boot/recovery dependency и расширяет последствия утечки
Kernel data.

### Второй public-key registry в platform secure storage

Отклонено: создаёт split-brain с Control Store. Platform storage владеет только
private key текущего device.

### Online restore/reset недоверенного store

Отклонено: Kernel не может доказать owner identity по registry, которому сам не
доверяет.

### Ed25519 как v1 baseline

Отклонено: это добавило бы второй key suite и отдельный verification path без
необходимости для первого file-adapter release.

## Проверка решения

До изменения `Состояние реализации` обязательны:

- file-backed `DeviceSigner` P-256 sign → Kernel verify conformance tests;
- malformed public key/signature, unsupported algorithm и wrong lengths;
- duplicate, expired, wrong-purpose и stale-generation challenge;
- revoked/suspended device и stale session epoch;
- authentication без требуемой authorization capability;
- file permission, symlink and key-file replacement rejection;
- initial enrollment только через inherited FD на pristine instance;
- existing/anchor-only/corrupt store отклоняет bootstrap enrollment;
- crash между anchor, SQLite create и owner/device transaction;
- private key не пересекает host signer boundary и отсутствует в logs/events,
  diagnostics, fixtures, backups и browser storage;
- online untrusted-store surface допускает только status/validate/export;
- online restore/reset и automatic reset отклоняются;
- offline restore/reset требует stopped Kernel, explicit data directory,
  exclusive lock и confirmation;
- reset/restore не удаляет Vault, PostgreSQL, blobs или provider sessions;
- restore и trusted-store reset повышают generations/epochs до data plane;
- corrupt/missing-store reset создаёт новую instance identity только через
  confirmed offline ceremony и никогда не выполняется автоматически;
- module registration/control FD не авторизует owner operation;
- Android и desktop devices имеют разные keys и независимый revoke.

Exact verifier dependency profile первого recovery-only slice определён
ADR-0225. Hardware attestation policy, Linux/Windows signer fallback,
recovery-key backup/sync и pairing UX проходят отдельные dependency/security
decisions. Signed distribution/update trust определён ADR-0219 и не использует
owner/device keys.

## Последствия

Плюсы:

- Kernel start и read-only recovery не зависят от Vault или identity registry;
- modules не получают owner authority через общий UID;
- каждое device независимо аутентифицируется и отзывается;
- Android входит в ту же модель без shared token;
- повреждённый store нельзя тихо reset-ить или заново claim-ить online.

Стоимость:

- нужны native signer adapters и cross-platform signature conformance;
- owner actions требуют challenge/session и иногда user presence;
- destructive recovery требует остановки Kernel;
- потеря всех device keys требует explicit offline recovery;
- дополнительные identity/anchor migrations и crash tests обязательны.
